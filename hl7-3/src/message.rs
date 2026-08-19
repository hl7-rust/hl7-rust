//! The three-level structure every HL7 v3 message shares, whatever domain
//! it carries: a transport wrapper, a control act wrapper, and a domain
//! payload.
//!
//! ```
//! let xml = r#"
//! <QUQI_IN000001UV01 xmlns="urn:hl7-org:v3">
//!   <id root="2.16.840.1.113883.19.5" extension="MSG00001"/>
//!   <creationTime value="20260101120000"/>
//!   <interactionId root="2.16.840.1.113883.1.6" extension="QUQI_IN000001UV01"/>
//!   <processingCode code="P"/>
//!   <sender typeCode="SND"><device classCode="DEV" determinerCode="INSTANCE"/></sender>
//!   <receiver typeCode="RCV"><device classCode="DEV" determinerCode="INSTANCE"/></receiver>
//!   <controlActProcess classCode="CACT" moodCode="EVN">
//!     <code code="QUQI_TE000001UV01"/>
//!     <subject>
//!       <observation classCode="OBS" moodCode="EVN">
//!         <id root="2.16.840.1.113883.19.5" extension="1"/>
//!       </observation>
//!     </subject>
//!   </controlActProcess>
//! </QUQI_IN000001UV01>
//! "#;
//! let message = hl7_3::message::parse(xml)?;
//! assert_eq!(message.interaction_id.unwrap().extension.as_deref(), Some("QUQI_IN000001UV01"));
//! assert_eq!(message.control_act.unwrap().code.unwrap().code, "QUQI_TE000001UV01");
//! # Ok::<(), hl7_3::Error>(())
//! ```

use crate::Error;
use crate::vocabulary::{Cd, Ii};
use hl7_2_xml_lite_helper::Element;

/// Level 1: the transport wrapper.
///
/// The root element's own tag is the interaction's wire name (analogous to
/// [`hl7-2`](https://crates.io/crates/hl7-2)'s message structure ID) —
/// this crate does not currently expose it as a field; read
/// [`Element::name`] on the value [`parse`] was given if you need it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Message {
    /// This message's own identifier — not the identifier of anything it
    /// carries.
    pub id: Option<Ii>,
    /// When the message was created, as raw text (see
    /// [`crate::rim::Act::effective_time`] for why this crate doesn't
    /// parse timestamps further yet).
    pub creation_time: Option<String>,
    /// Which interaction this is — the wire contract that names the
    /// trigger event and the payload shape a receiver should expect. An
    /// `II`, not a `CD`: `root` names the interaction catalog (almost
    /// always `2.16.840.1.113883.1.6`) and `extension` names the specific
    /// interaction (`"QUQI_IN000001UV01"`).
    pub interaction_id: Option<Ii>,
    /// The sender, read as a raw element rather than a modeled [`Device`]
    /// — see `spec/index.md` §1 for why.
    ///
    /// [`Device`]: crate::rim::Entity
    pub sender: Option<Element>,
    /// The receiver, read the same way as `sender`.
    pub receiver: Option<Element>,
    /// Level 2 and 3: the control act wrapper and the domain payload it
    /// carries, when the message has one.
    pub control_act: Option<ControlAct>,
}

/// Level 2: the control act wrapper — identifies the real-world trigger
/// event and carries level 3, the domain payload.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ControlAct {
    /// Almost always `"CACT"` — kept as read, not assumed, since a
    /// nonconforming sender is something a caller may want to notice
    /// rather than have silently papered over.
    pub class_code: String,
    /// Almost always `"EVN"`, for the same reason.
    pub mood_code: String,
    /// The trigger event code — which real-world event caused this
    /// message to be sent.
    pub code: Option<Cd>,
    /// Level 3: the domain-specific payload, read as a raw element.
    ///
    /// Its shape is defined by the interaction (`interaction_id`), not by
    /// anything this crate knows in general — decode it with [`crate::rim`]
    /// types yourself, matching what that interaction's schema says to
    /// expect. This crate finds it (the first element under a `subject`
    /// wrapper, the common case) and stops there; see `spec/index.md` §3
    /// for the exact rule and its limits.
    pub domain: Option<Element>,
}

/// Parse one HL7 v3 XML message into its three-level structure.
///
/// # Errors
///
/// [`Error::Xml`] when `xml_text` is not well-formed XML. This function
/// does not fail when a wrapper element is missing — an absent `id`,
/// `interactionId`, `sender`, or `controlActProcess` reads as `None`, the
/// same way [`hl7-2`](https://crates.io/crates/hl7-2)'s generic mode
/// degrades rather than rejecting; see `spec/index.md` §3.
pub fn parse(xml_text: &str) -> Result<Message, Error> {
    let root = hl7_2_xml_lite_helper::parse(xml_text).map_err(Error::Xml)?;
    Ok(Message {
        id: root.child("id").and_then(Ii::from_element),
        creation_time: root
            .child("creationTime")
            .and_then(|time| time.attribute("value"))
            .map(str::to_string),
        interaction_id: root.child("interactionId").and_then(Ii::from_element),
        sender: root
            .child("sender")
            .and_then(|wrapper| wrapper.children.first())
            .cloned(),
        receiver: root
            .child("receiver")
            .and_then(|wrapper| wrapper.children.first())
            .cloned(),
        control_act: root
            .child("controlActProcess")
            .map(control_act_from_element),
    })
}

fn control_act_from_element(element: &Element) -> ControlAct {
    ControlAct {
        class_code: element
            .attribute("classCode")
            .unwrap_or_default()
            .to_string(),
        mood_code: element
            .attribute("moodCode")
            .unwrap_or_default()
            .to_string(),
        code: element.child("code").and_then(Cd::from_element),
        domain: element
            .child("subject")
            .and_then(|subject| subject.children.first())
            .cloned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        <QUQI_IN000001UV01 xmlns="urn:hl7-org:v3">
          <id root="2.16.840.1.113883.19.5" extension="MSG00001"/>
          <creationTime value="20260101120000"/>
          <interactionId root="2.16.840.1.113883.1.6" extension="QUQI_IN000001UV01"/>
          <sender typeCode="SND"><device classCode="DEV" determinerCode="INSTANCE"/></sender>
          <receiver typeCode="RCV"><device classCode="DEV" determinerCode="INSTANCE"/></receiver>
          <controlActProcess classCode="CACT" moodCode="EVN">
            <code code="QUQI_TE000001UV01"/>
            <subject>
              <observation classCode="OBS" moodCode="EVN">
                <id root="2.16.840.1.113883.19.5" extension="1"/>
              </observation>
            </subject>
          </controlActProcess>
        </QUQI_IN000001UV01>
    "#;

    #[test]
    fn reads_the_transport_wrapper() {
        let message = parse(SAMPLE).unwrap();
        assert_eq!(message.id.unwrap().extension.as_deref(), Some("MSG00001"));
        assert_eq!(message.creation_time.as_deref(), Some("20260101120000"));
        assert_eq!(
            message.interaction_id.unwrap().extension.as_deref(),
            Some("QUQI_IN000001UV01")
        );
        assert_eq!(message.sender.unwrap().local_name(), "device");
    }

    #[test]
    fn reads_the_control_act_wrapper_and_trigger_event() {
        let message = parse(SAMPLE).unwrap();
        let control_act = message.control_act.unwrap();
        assert_eq!(control_act.class_code, "CACT");
        assert_eq!(control_act.mood_code, "EVN");
        assert_eq!(control_act.code.unwrap().code, "QUQI_TE000001UV01");
    }

    #[test]
    fn reads_the_domain_payload_as_a_raw_element() {
        let message = parse(SAMPLE).unwrap();
        let domain = message.control_act.unwrap().domain.unwrap();
        assert_eq!(domain.local_name(), "observation");
        assert_eq!(domain.attribute("classCode"), Some("OBS"));
        let act = crate::rim::Act::from_element(&domain);
        assert_eq!(act.id[0].extension.as_deref(), Some("1"));
    }

    #[test]
    fn missing_wrappers_read_as_none_not_an_error() {
        let message = parse(r"<EMPTY_MESSAGE/>").unwrap();
        assert_eq!(message.id, None);
        assert_eq!(message.interaction_id, None);
        assert_eq!(message.control_act, None);
    }

    #[test]
    fn malformed_xml_is_an_error() {
        assert!(matches!(parse("<unclosed>"), Err(Error::Xml(_))));
    }
}
