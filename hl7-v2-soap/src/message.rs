//! What a SOAP body is carrying, and how to read it.
//!
//! HL7 travels over SOAP in two shapes, and an interface picks one:
//!
//! - **As v2.xml.** The body holds the message's own elements, so the
//!   payload element is named for the message structure
//!   (`<ADT_A05>...</ADT_A05>`). Everything is addressable as XML, which is
//!   why a receiver can validate it against the published schemas.
//! - **As ER7 in a wrapper.** The body holds an operation element with a
//!   single text child carrying the pipe-delimited message, escaped
//!   (`<SendHL7Message><hl7Message>MSH|^~\&amp;|...</hl7Message></SendHL7Message>`).
//!   Nothing inside is addressable; the receiver parses the text.
//!
//! This module reads both, and writes the second. It deliberately does not
//! convert between v2.xml and ER7 — that is what
//! `hl7-v2-from-xml-into-er7` and its sibling are for, and a transport
//! crate that also converted formats would be two crates.

use crate::fault::{Fault, SOAP_NS};
use crate::xml::{self, Element};

/// The element name the ER7 carriage wraps a message in.
pub const OPERATION: &str = "SendHL7Message";

/// The element name holding the ER7 text.
pub const ER7_ELEMENT: &str = "hl7Message";

/// The message structure a v2.xml payload declares, which is its element
/// name: `ADT_A05` for `<ADT_A05>`.
#[must_use]
pub fn structure_id(payload: &Element) -> &str {
    payload.local_name()
}

/// MSH-10, the message control ID, from a v2.xml payload.
#[must_use]
pub fn control_id(payload: &Element) -> Option<&str> {
    payload.text_at(&["MSH", "MSH.10"])
}

/// The assigning authority a v2.xml payload comes from.
///
/// Tried in the order an interface agreement usually states them: the
/// sending application, then the sending facility, then the authority on
/// the patient identifier. The first that carries a value wins — a message
/// that names none of them cannot be attributed to a system, which is a
/// question of authorization rather than of formatting.
#[must_use]
pub fn assigning_authority(payload: &Element) -> Option<&str> {
    payload
        .text_at(&["MSH", "MSH.3", "HD.1"])
        .or_else(|| payload.text_at(&["MSH", "MSH.4", "HD.1"]))
        .or_else(|| payload.text_at(&["PID", "PID.3", "CX.4", "HD.1"]))
}

/// The ER7 text a payload carries, when the interface uses that carriage.
///
/// Accepts the wrapper element with its `hl7Message` child, and also a bare
/// element whose own text is the message, because senders differ on
/// whether they nest it.
///
/// The text is trimmed. XML does not preserve the difference between a
/// sender's trailing segment terminator and a serializer's indentation —
/// both are whitespace in a text node — so a reader cannot honour one
/// without inventing the other. Trimming is safe because an ER7 parser
/// accepts a final segment with or without its terminator; what it must
/// not do is see a stray newline as the start of a segment.
#[must_use]
pub fn er7(payload: &Element) -> Option<&str> {
    if let Some(element) = payload.child(ER7_ELEMENT) {
        let text = element.text.trim();
        if !text.is_empty() {
            return Some(text);
        }
    }
    let text = payload.text.trim();
    if text.starts_with("MSH") {
        Some(text)
    } else {
        None
    }
}

/// Wrap an ER7 message in a request envelope.
///
/// The message is escaped, because ER7 uses `&` as its subcomponent
/// separator and would otherwise close the document early.
///
/// ```
/// let envelope = hl7_v2_soap::message::wrap_er7("MSH|^~\\&|APP||||1||ADT^A01|9|P|2.5");
/// let parsed = hl7_v2_soap::parse(&envelope)?;
/// let payload = parsed.payload()?;
/// assert_eq!(
///     hl7_v2_soap::message::er7(payload),
///     Some("MSH|^~\\&|APP||||1||ADT^A01|9|P|2.5"),
/// );
/// # Ok::<(), hl7_v2_soap::Fault>(())
/// ```
#[must_use]
pub fn wrap_er7(er7_text: &str) -> String {
    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            r#"<soapenv:Envelope xmlns:soapenv="{}">"#,
            "<soapenv:Header/>",
            "<soapenv:Body>",
            "<{}><{}>{}</{}></{}>",
            "</soapenv:Body>",
            "</soapenv:Envelope>",
        ),
        SOAP_NS,
        OPERATION,
        ER7_ELEMENT,
        xml::escape(er7_text),
        ER7_ELEMENT,
        OPERATION,
    )
}

/// Check a v2.xml payload against what this interface accepts.
///
/// Both lists come from the interface agreement, not from HL7: a receiver
/// accepts the structures it was built for, from the systems it was told
/// about. An empty list means "no restriction", which is what a receiver
/// with nothing configured has to assume.
///
/// # Errors
///
/// A `Client.Validation` fault (HTTP 400) when the structure is not one the
/// interface accepts, or when authorities are restricted and the payload
/// names none. A `Client.Authorization` fault (HTTP 403) when it names one
/// that is not permitted — a different status because it is a different
/// problem, and the sender should not retry either.
pub fn check(
    payload: &Element,
    allowed_structures: &[String],
    allowed_authorities: &[String],
) -> Result<(), Fault> {
    let structure = structure_id(payload);
    if !allowed_structures.is_empty() && !allowed_structures.iter().any(|a| a == structure) {
        return Err(Fault::validation(format!(
            "Unsupported HL7 message structure '{structure}'. Allowed values: {}",
            listed(allowed_structures)
        )));
    }
    if !allowed_authorities.is_empty() {
        let authority = assigning_authority(payload).ok_or_else(|| {
            Fault::validation("Unable to determine assigning authority from payload.")
        })?;
        if !allowed_authorities.iter().any(|a| a == authority) {
            return Err(Fault::authorization(format!(
                "Assigning authority '{authority}' is not authorised. Allowed values: {}",
                listed(allowed_authorities)
            )));
        }
    }
    Ok(())
}

/// Allowed values, in order, for an error message.
fn listed(values: &[String]) -> String {
    let mut sorted: Vec<&str> = values.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn payload_of(xml_text: &str) -> Element {
        parse(xml_text).unwrap().payload().unwrap().clone()
    }

    const V2XML: &str = r#"<Envelope><Body>
        <ADT_A05>
          <MSH><MSH.3><HD.1>252</HD.1></MSH.3><MSH.10>CTRL7</MSH.10></MSH>
          <PID><PID.3><CX.4><HD.1>NHS</HD.1></CX.4></PID.3></PID>
        </ADT_A05></Body></Envelope>"#;

    #[test]
    fn reads_a_v2xml_payload() {
        let payload = payload_of(V2XML);
        assert_eq!(structure_id(&payload), "ADT_A05");
        assert_eq!(control_id(&payload), Some("CTRL7"));
        assert_eq!(assigning_authority(&payload), Some("252"));
    }

    #[test]
    fn falls_through_to_the_facility_then_the_patient_identifier() {
        let from_facility = payload_of(
            r#"<Envelope><Body><ADT_A05><MSH><MSH.4><HD.1>FAC</HD.1></MSH.4></MSH></ADT_A05></Body></Envelope>"#,
        );
        assert_eq!(assigning_authority(&from_facility), Some("FAC"));

        let from_pid = payload_of(
            r#"<Envelope><Body><ADT_A05><PID><PID.3><CX.4><HD.1>NHS</HD.1></CX.4></PID.3></PID></ADT_A05></Body></Envelope>"#,
        );
        assert_eq!(assigning_authority(&from_pid), Some("NHS"));

        let from_nowhere = payload_of(r#"<Envelope><Body><ADT_A05/></Body></Envelope>"#);
        assert_eq!(assigning_authority(&from_nowhere), None);
    }

    #[test]
    fn reads_er7_carriage_in_both_shapes() {
        let wrapped = payload_of(&wrap_er7("MSH|^~\\&|A|B"));
        assert_eq!(er7(&wrapped), Some("MSH|^~\\&|A|B"));

        let bare =
            payload_of("<Envelope><Body><Anything>MSH|^~\\&amp;|A|B</Anything></Body></Envelope>");
        assert_eq!(er7(&bare), Some("MSH|^~\\&|A|B"));

        // A v2.xml payload is not ER7 carriage.
        assert_eq!(er7(&payload_of(V2XML)), None);
    }

    #[test]
    fn escaping_survives_the_round_trip() {
        let message = "MSH|^~\\&|A&B|<x>|\"q\"";
        let payload = payload_of(&wrap_er7(message));
        assert_eq!(er7(&payload), Some(message));
    }

    #[test]
    fn checks_the_structure_and_the_authority() {
        let payload = payload_of(V2XML);
        let structures = vec!["ADT_A05".to_string()];
        let authorities = vec!["252".to_string()];
        assert!(check(&payload, &structures, &authorities).is_ok());

        // No restriction configured means no restriction applied.
        assert!(check(&payload, &[], &[]).is_ok());

        let wrong_structure = check(&payload, &["ADT_A39".to_string()], &authorities).unwrap_err();
        assert_eq!(wrong_structure.status, 400);
        assert!(wrong_structure.reason.contains("ADT_A05"));

        let wrong_authority = check(&payload, &structures, &["999".to_string()]).unwrap_err();
        assert_eq!(
            wrong_authority.status, 403,
            "not authorised is not a validation error"
        );
    }

    #[test]
    fn a_payload_with_no_authority_cannot_be_authorised() {
        let payload = payload_of(r#"<Envelope><Body><ADT_A05/></Body></Envelope>"#);
        let error = check(&payload, &[], &["252".to_string()]).unwrap_err();
        assert!(
            error
                .reason
                .contains("Unable to determine assigning authority")
        );
    }
}
