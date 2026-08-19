//! The reply a receiver sends, and how a sender reads one.
//!
//! HL7 v3 has an unambiguous answer to "did that get through?", the same
//! way v2's MSA-1 does: `acknowledgement/typeCode` inside an
//! `MCCI_IN000002UV01` (Master Information Composite / Control Act
//! acknowledgement interaction) — the real HL7 v3 acknowledgement message,
//! not a convention this crate invented. SOAP still leaves three places to
//! look and no agreement between implementations about which is
//! authoritative, so this module reads all three, in the order that cannot
//! be talked out of a rejection: the HTTP status, then a SOAP `Fault`, then
//! the acknowledgement's own `typeCode`.

use crate::xml;

/// The interaction a receiver answers with on success.
pub const ACK_INTERACTION: &str = "MCCI_IN000002UV01";

/// HL7's acknowledgement type codes that mean accepted — `AA` (application
/// accept) and `CA` (enhanced-mode commit accept). Shared with v2's MSA-1,
/// which draws from the same conceptual vocabulary.
const ACCEPTED: [&str; 2] = ["AA", "CA"];

/// What a response says happened to the message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The receiver has the message.
    Accepted,
    /// The receiver does not, and says why.
    Rejected(String),
}

impl Outcome {
    /// Whether the message got through.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        matches!(self, Outcome::Accepted)
    }
}

/// Build the success acknowledgement for a received message.
///
/// `control_id` is the extension of the `id` the original message carried
/// ([`crate::message::control_id`]) — `targetMessage/id` echoes it back so
/// the sender can match the acknowledgement to the message it answers, the
/// same purpose MSA-2 serves in v2.
#[must_use]
pub fn success(control_id: &str) -> String {
    format!(
        concat!(
            r#"<soapenv:Envelope xmlns:soapenv="{}">"#,
            "<soapenv:Body>",
            r#"<{} xmlns="urn:hl7-org:v3">"#,
            "<acknowledgement>",
            r#"<typeCode code="AA"/>"#,
            "<targetMessage>",
            r#"<id extension="{}"/>"#,
            "</targetMessage>",
            "</acknowledgement>",
            "</{}>",
            "</soapenv:Body>",
            "</soapenv:Envelope>",
        ),
        crate::fault::SOAP_NS,
        ACK_INTERACTION,
        xml::escape(control_id),
        ACK_INTERACTION,
    )
}

/// Decide what an HTTP response says about the message.
///
/// The order is the point:
///
/// 1. **A non-success status is a rejection.** Whatever the body says, the
///    transport already said no.
/// 2. **A `Fault` element is a rejection**, even under HTTP 200. Some
///    stacks answer 200 and put the refusal in the body; believing the
///    status there would lose messages silently.
/// 3. **`acknowledgement/typeCode/@code` decides**, when the interface
///    returns one: `AA`/`CA` accepted, anything else rejected.
/// 4. **Otherwise accepted.** A bare 200 with an empty or unrecognised body
///    is what a minimal endpoint returns, and treating that as failure
///    would resend every message forever.
#[must_use]
pub fn evaluate(status: u16, body: &str) -> Outcome {
    if !(200..300).contains(&status) {
        return Outcome::Rejected(format!("HTTP {status}"));
    }

    let Ok(root) = xml::parse(body) else {
        // Unparseable under a success status: nothing here contradicts the
        // transport, and rule 4 applies.
        return Outcome::Accepted;
    };

    if let Some(fault) = root.find("Fault") {
        let code = fault
            .child("faultcode")
            .map(|element| element.text.trim())
            .unwrap_or_default();
        let reason = fault
            .child("faultstring")
            .map(|element| element.text.trim())
            .unwrap_or_default();
        return Outcome::Rejected(match (code.is_empty(), reason.is_empty()) {
            (true, true) => "SOAP fault".to_string(),
            (true, false) => reason.to_string(),
            (false, true) => code.to_string(),
            (false, false) => format!("{code}: {reason}"),
        });
    }

    if let Some(acknowledgement) = root.find("acknowledgement") {
        let code = acknowledgement
            .child("typeCode")
            .and_then(|type_code| type_code.attribute("code"))
            .unwrap_or_default();
        if ACCEPTED.contains(&code) {
            return Outcome::Accepted;
        }
        return Outcome::Rejected(format!("typeCode {code}"));
    }

    Outcome::Accepted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_success_response_names_the_message_it_answers() {
        let body = success("CTRL7");
        let root = xml::parse(&body).unwrap();
        assert_eq!(root.find("typeCode").unwrap().attribute("code"), Some("AA"));
        assert_eq!(
            root.find("targetMessage")
                .unwrap()
                .child("id")
                .unwrap()
                .attribute("extension"),
            Some("CTRL7")
        );
        assert_eq!(evaluate(200, &body), Outcome::Accepted);
    }

    #[test]
    fn a_control_id_cannot_break_out_of_the_response() {
        let body = success("a<b>&c");
        assert!(body.contains("a&lt;b&gt;&amp;c"));
        assert_eq!(
            xml::parse(&body)
                .unwrap()
                .find("targetMessage")
                .unwrap()
                .child("id")
                .unwrap()
                .attribute("extension"),
            Some("a<b>&c")
        );
    }

    #[test]
    fn a_failing_status_is_a_rejection_whatever_the_body_says() {
        assert_eq!(
            evaluate(500, &success("1")),
            Outcome::Rejected("HTTP 500".into())
        );
        assert!(!evaluate(404, "").is_accepted());
    }

    #[test]
    fn a_fault_under_http_200_is_still_a_rejection() {
        let body = crate::Fault::validation("bad interaction").to_envelope();
        assert_eq!(
            evaluate(200, &body),
            Outcome::Rejected("Client.Validation: bad interaction".into())
        );
    }

    #[test]
    fn an_hl7_acknowledgement_type_code_decides_when_one_is_returned() {
        let with_code = |code: &str| {
            format!(
                "<Envelope><Body><MCCI_IN000002UV01><acknowledgement>\
                 <typeCode code=\"{code}\"/></acknowledgement></MCCI_IN000002UV01></Body></Envelope>"
            )
        };
        assert!(evaluate(200, &with_code("AA")).is_accepted());
        assert!(evaluate(200, &with_code("CA")).is_accepted());
        assert!(!evaluate(200, &with_code("AE")).is_accepted());
        assert!(!evaluate(200, &with_code("AR")).is_accepted());
        assert_eq!(
            evaluate(200, &with_code("AE")),
            Outcome::Rejected("typeCode AE".into())
        );
    }

    #[test]
    fn a_minimal_endpoint_that_says_nothing_is_believed() {
        assert!(evaluate(200, "").is_accepted());
        assert!(evaluate(202, "OK").is_accepted());
        assert!(evaluate(200, "<Envelope><Body/></Envelope>").is_accepted());
    }
}
