//! [`Message`] and [`ControlAct`]: the three-level HL7 v3 envelope,
//! serialized as nested objects.

use crate::{Cd, Element, Ii};

object_wrapper! {
    /// A Serde-enabled [`hl7_3::Message`] — the crate's main entry point:
    /// the transport wrapper, with the control act wrapper and the domain
    /// payload nested inside it.
    ///
    /// Serializes as an object with six keys: `"id"` ([`Ii`] or `null`),
    /// `"creationTime"` (string or `null`), `"interactionId"` ([`Ii`] or
    /// `null`), `"sender"` and `"receiver"` ([`Element`] or `null`), and
    /// `"controlAct"` ([`ControlAct`] or `null`). Every key is optional on
    /// the way back in, matching `hl7-3`'s own rule that a missing wrapper
    /// reads as `None` rather than failing.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Message;
    ///
    /// let message = Message::parse(r#"
    ///   <QUQI_IN000001UV01 xmlns="urn:hl7-org:v3">
    ///     <id root="2.16.840.1.113883.19.5" extension="MSG00001"/>
    ///     <controlActProcess classCode="CACT" moodCode="EVN">
    ///       <code code="QUQI_TE000001UV01"/>
    ///     </controlActProcess>
    ///   </QUQI_IN000001UV01>"#)?;
    ///
    /// let json = serde_json::to_value(&message)?;
    /// assert_eq!(json["id"]["extension"], "MSG00001");
    /// assert_eq!(json["controlAct"]["code"]["code"], "QUQI_TE000001UV01");
    /// assert_eq!(json["sender"], serde_json::Value::Null);
    ///
    /// let back: Message = serde_json::from_value(json)?;
    /// assert_eq!(back, message);
    /// # Ok(())
    /// # }
    /// ```
    Message wraps hl7_3::Message,
    expecting "a Message object with \"id\", \"creationTime\", \"interactionId\", \"sender\", \"receiver\", and \"controlAct\"",
    derive [Default],
    fields {
        id: optw Ii => "id",
        creation_time: opt String => "creationTime",
        interaction_id: optw Ii => "interactionId",
        sender: optw Element => "sender",
        receiver: optw Element => "receiver",
        control_act: optw ControlAct => "controlAct",
    }
}

impl Message {
    /// Parse HL7 v3 XML directly into a Serde-enabled [`Message`] — a thin
    /// wrapper over [`hl7_3::message::parse`].
    ///
    /// # Errors
    ///
    /// Returns [`hl7_3::Error`] exactly as [`hl7_3::message::parse`] does:
    /// the input is not well-formed XML. Nothing is added here.
    pub fn parse(xml: &str) -> Result<Message, hl7_3::Error> {
        hl7_3::message::parse(xml).map(Message)
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::ControlAct`]: the control act wrapper, and
    /// the domain payload it carries.
    ///
    /// Serializes as an object with four keys: `"classCode"` and
    /// `"moodCode"` (strings, required), `"code"` ([`Cd`] or `null`, the
    /// trigger event), and `"domain"` ([`Element`] or `null`, the payload
    /// as a raw element).
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::ControlAct;
    ///
    /// let json = r#"{"classCode":"CACT","moodCode":"EVN","code":{"code":"X"}}"#;
    /// let control_act: ControlAct = serde_json::from_str(json)?;
    /// assert_eq!(control_act.code.as_ref().unwrap().code, "X");
    /// assert!(control_act.domain.is_none());
    /// # Ok(())
    /// # }
    /// ```
    ControlAct wraps hl7_3::ControlAct,
    expecting "a ControlAct object with \"classCode\", \"moodCode\", \"code\", and \"domain\"",
    derive [Default],
    fields {
        class_code: req String => "classCode",
        mood_code: req String => "moodCode",
        code: optw Cd => "code",
        domain: optw Element => "domain",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Strict;

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
    fn round_trips_a_full_message() {
        let message = Message::parse(SAMPLE).unwrap();
        let json = serde_json::to_string(&message).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back, message);
        let domain = back.0.control_act.unwrap().domain.unwrap();
        assert_eq!(domain.local_name(), "observation");
        assert_eq!(
            domain.child("id").unwrap().attribute("extension"),
            Some("1")
        );
    }

    #[test]
    fn keys_are_the_camel_case_field_names() {
        let json = serde_json::to_value(Message::parse(SAMPLE).unwrap()).unwrap();
        let keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        // serde_json's default map keeps insertion order off, so compare as a set.
        let mut keys = keys;
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "controlAct",
                "creationTime",
                "id",
                "interactionId",
                "receiver",
                "sender"
            ]
        );
        assert_eq!(json["creationTime"], "20260101120000");
        assert_eq!(json["sender"]["name"], "device");
        assert_eq!(json["controlAct"]["classCode"], "CACT");
    }

    #[test]
    fn an_empty_message_is_all_nulls_and_reads_back() {
        let message = Message::parse("<EMPTY/>").unwrap();
        let json = serde_json::to_string(&message).unwrap();
        assert_eq!(
            json,
            r#"{"id":null,"creationTime":null,"interactionId":null,"sender":null,"receiver":null,"controlAct":null}"#
        );
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back, message);
        let bare: Message = serde_json::from_str("{}").unwrap();
        assert_eq!(bare, message);
    }

    #[test]
    fn control_act_requires_its_codes() {
        let err = serde_json::from_str::<ControlAct>(r#"{"classCode":"CACT"}"#).unwrap_err();
        assert!(err.to_string().contains("moodCode"), "{err}");
    }

    #[test]
    fn rejects_a_duplicate_key() {
        let err = serde_json::from_str::<Message>(r#"{"creationTime":"1","creationTime":"2"}"#)
            .unwrap_err();
        assert!(err.to_string().contains("duplicate"), "{err}");
    }

    #[test]
    fn strict_reaches_every_nested_level() {
        // Top level.
        let err = serde_json::from_str::<Strict<Message>>(r#"{"idd":null}"#).unwrap_err();
        assert!(err.to_string().contains("idd"), "{err}");
        // Inside a wrapped Ii.
        let err = serde_json::from_str::<Strict<Message>>(r#"{"id":{"root":"1","ext":"x"}}"#)
            .unwrap_err();
        assert!(err.to_string().contains("ext"), "{err}");
        // Inside the control act, its code, and the domain element's children.
        let json = r#"{"controlAct":{"classCode":"CACT","moodCode":"EVN","code":{"code":"X","codeSystm":"y"}}}"#;
        let err = serde_json::from_str::<Strict<Message>>(json).unwrap_err();
        assert!(err.to_string().contains("codeSystm"), "{err}");
        let json = r#"{"controlAct":{"classCode":"CACT","moodCode":"EVN","domain":{"name":"observation","children":[{"name":"id","attrs":{}}]}}}"#;
        assert!(
            serde_json::from_str::<Message>(json).is_ok(),
            "plain stays tolerant"
        );
        let err = serde_json::from_str::<Strict<Message>>(json).unwrap_err();
        assert!(err.to_string().contains("attrs"), "{err}");
    }

    #[test]
    fn strict_accepts_a_real_message_with_no_typos() {
        let message = Message::parse(SAMPLE).unwrap();
        let json = serde_json::to_string(&message).unwrap();
        let back = serde_json::from_str::<Strict<Message>>(&json).unwrap();
        assert_eq!(back.0, message);
    }

    #[test]
    fn deref_and_from_work_both_ways() {
        let message = Message::parse(SAMPLE).unwrap();
        assert_eq!(message.creation_time.as_deref(), Some("20260101120000"));
        let inner: hl7_3::Message = message.clone().into();
        let outer: Message = inner.into();
        assert_eq!(outer, message);
    }
}
