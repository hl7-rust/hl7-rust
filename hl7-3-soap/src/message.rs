//! What a SOAP body is carrying, and how to read it.
//!
//! Unlike `hl7-2-soap`, there is only one shape here: HL7 v3 is XML
//! natively, so the payload is always a complete v3 message — the same
//! transport wrapper, control act wrapper, and domain payload `hl7-3`
//! reads, root element named for the interaction
//! (`PRPA_IN201305UV02` and so on). There is no second, ER7-in-a-wrapper
//! carriage to read, the way `hl7-2-soap` reads both v2.xml and wrapped
//! ER7 — HL7 v3 was designed with SOAP as (historically) its primary
//! transport, so "the message" and "the SOAP payload" are the same thing.
//!
//! This module reads the pieces a receiver needs before it hands the
//! payload to `hl7-3` for full decoding: which interaction this is, the
//! message's own control ID, and which system it claims to be from. It
//! deliberately reads these directly off the raw element rather than
//! depending on `hl7-3` — see `spec/index.md` §1 for why.

use crate::fault::Fault;
use hl7_2_xml_lite_helper::Element;

/// The interaction this payload is — its own root element name, the same
/// way `hl7-3::message::parse` finds it. `PRPA_IN201305UV02` for
/// `<PRPA_IN201305UV02>`.
#[must_use]
pub fn interaction_id(payload: &Element) -> &str {
    payload.local_name()
}

/// The message's own control ID: its `id` child's `extension` attribute.
///
/// This is the v3 analogue of `hl7-2-soap::message::control_id` reading
/// MSH-10 — but where v2 spells a control ID as one string, v3 spells an
/// `II` as `root`/`extension` together; `extension` alone is what a log
/// line or a retry key wants, so that is what this returns. Read the whole
/// `II` (both `root` and `extension`) from `payload.child("id")` yourself
/// when you need the assigning scheme too.
#[must_use]
pub fn control_id(payload: &Element) -> Option<&str> {
    payload.child("id")?.attribute("extension")
}

/// The system this message claims to be from.
///
/// Tried in the order an interface agreement usually states them: the
/// message's own `id/@root` (the OID naming the system or domain that
/// assigned this message's identifier), then the `sender`'s device `id`
/// child's `root`. The first that carries a value wins — a message naming
/// neither cannot be attributed to a system, which is a question of
/// authorization rather than of formatting.
#[must_use]
pub fn assigning_authority(payload: &Element) -> Option<&str> {
    payload
        .child("id")
        .and_then(|id| id.attribute("root"))
        .or_else(|| {
            payload
                .child("sender")
                .and_then(|sender| sender.child("device"))
                .and_then(|device| device.child("id"))
                .and_then(|id| id.attribute("root"))
        })
}

/// Check a v3 payload against what this interface accepts.
///
/// Both lists come from the interface agreement, not from HL7: a receiver
/// accepts the interactions it was built for, from the systems it was told
/// about. An empty list means "no restriction", which is what a receiver
/// with nothing configured has to assume.
///
/// # Errors
///
/// A `Client.Validation` fault (HTTP 400) when the interaction is not one
/// the interface accepts, or when authorities are restricted and the
/// payload names none. A `Client.Authorization` fault (HTTP 403) when it
/// names one that is not permitted — a different status because it is a
/// different problem, and the sender should not retry either.
pub fn check(
    payload: &Element,
    allowed_interactions: &[String],
    allowed_authorities: &[String],
) -> Result<(), Fault> {
    let interaction = interaction_id(payload);
    if !allowed_interactions.is_empty() && !allowed_interactions.iter().any(|a| a == interaction) {
        return Err(Fault::validation(format!(
            "Unsupported HL7 v3 interaction '{interaction}'. Allowed values: {}",
            listed(allowed_interactions)
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

    const V3_MESSAGE: &str = r#"<Envelope><Body>
        <PRPA_IN201305UV02>
          <id root="2.16.840.1.113883.19.5" extension="CTRL7"/>
          <sender><device><id root="2.16.840.1.113883.19.6"/></device></sender>
        </PRPA_IN201305UV02></Body></Envelope>"#;

    #[test]
    fn reads_a_v3_payload() {
        let payload = payload_of(V3_MESSAGE);
        assert_eq!(interaction_id(&payload), "PRPA_IN201305UV02");
        assert_eq!(control_id(&payload), Some("CTRL7"));
        assert_eq!(
            assigning_authority(&payload),
            Some("2.16.840.1.113883.19.5")
        );
    }

    #[test]
    fn falls_through_to_the_sender_device_when_the_message_has_no_root() {
        let from_sender = payload_of(
            r#"<Envelope><Body><PRPA_IN201305UV02>
                 <sender><device><id root="2.16.840.1.113883.19.6"/></device></sender>
               </PRPA_IN201305UV02></Body></Envelope>"#,
        );
        assert_eq!(
            assigning_authority(&from_sender),
            Some("2.16.840.1.113883.19.6")
        );

        let from_nowhere = payload_of(r#"<Envelope><Body><PRPA_IN201305UV02/></Body></Envelope>"#);
        assert_eq!(assigning_authority(&from_nowhere), None);
    }

    #[test]
    fn checks_the_interaction_and_the_authority() {
        let payload = payload_of(V3_MESSAGE);
        let interactions = vec!["PRPA_IN201305UV02".to_string()];
        let authorities = vec!["2.16.840.1.113883.19.5".to_string()];
        assert!(check(&payload, &interactions, &authorities).is_ok());

        // No restriction configured means no restriction applied.
        assert!(check(&payload, &[], &[]).is_ok());

        let wrong_interaction =
            check(&payload, &["PRPA_IN201306UV02".to_string()], &authorities).unwrap_err();
        assert_eq!(wrong_interaction.status, 400);
        assert!(wrong_interaction.reason.contains("PRPA_IN201305UV02"));

        let wrong_authority = check(&payload, &interactions, &["9.9.9".to_string()]).unwrap_err();
        assert_eq!(
            wrong_authority.status, 403,
            "not authorised is not a validation error"
        );
    }

    #[test]
    fn a_payload_with_no_authority_cannot_be_authorised() {
        let payload = payload_of(r#"<Envelope><Body><PRPA_IN201305UV02/></Body></Envelope>"#);
        let error = check(&payload, &[], &["2.16.840.1.113883.19.5".to_string()]).unwrap_err();
        assert!(
            error
                .reason
                .contains("Unable to determine assigning authority")
        );
    }
}
