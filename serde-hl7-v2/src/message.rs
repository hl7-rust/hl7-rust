//! [`Message`]: a parsed HL7 v2 message, serialized as its release plus its
//! ER7 text.

use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Strict, Version};

/// A Serde-enabled [`hl7_2::Message`] — the crate's main entry point.
///
/// Serializes as an object with two fields: `"version"`, the release the
/// message was read as ([`hl7_2::Message::version`], through
/// [`Version`]), and `"er7"`, the message as ER7 text
/// ([`hl7_2::Message::to_er7`]). Deserializing parses that text again,
/// through [`hl7_2::parse_with_options`] with the version pinned, so the
/// value that comes back carries a resolved dictionary exactly as a freshly
/// parsed message would — the dictionary itself is never on the wire.
///
/// # Why text, not a tree
///
/// `hl7-2` is the dictionary layer over the `er7` encoding layer, and the
/// encoding layer's tree already has a Serde crate of its own: `serde-er7`,
/// which wraps the [`hl7_2::er7`] types reachable from
/// [`hl7_2::Message::raw`]. Repeating that tree here would be a second
/// wire shape for the same bytes. The ER7 text is the one form every HL7
/// v2 system already agrees on, it round-trips exactly, and the
/// dictionary-aware view this crate adds lives in [`crate::Node`] instead.
///
/// # What round-trips
///
/// `Message::parse(text)?` through any Serde format and back reproduces
/// the same ER7 `hl7_2::parse(text)?.to_er7()` would, and the same
/// [`hl7_2::Message::version`]. A message read through a caller's own
/// dictionary (`hl7-2`'s schema mode) comes back through the bundled
/// dictionary for its release unless it is deserialized with
/// [`Message::seed`], which carries the [`hl7_2::Options`] to use.
///
/// Example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde_hl7_v2::Message;
///
/// let text = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260815120000||ORU^R01|MSG9|P|2.5\r\
///             PID|1||12345^^^ACME^MR||SMITH^JOHN^Q||19800101|M";
/// let message = Message::parse(text)?;
///
/// let json = serde_json::to_value(&message)?;
/// assert_eq!(json["version"], "2.5");
/// assert_eq!(json["er7"], text);
///
/// let back: Message = serde_json::from_value(json)?;
/// assert_eq!(back.to_er7(), text);
/// assert_eq!(back.get("PID-5.1")?.as_deref(), Some("SMITH"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Message(pub hl7_2::Message);

impl Message {
    /// Parse ER7 text directly into a Serde-enabled [`Message`], reading the
    /// release from MSH-12 — a thin wrapper over [`hl7_2::parse`].
    ///
    /// # Errors
    ///
    /// Returns [`hl7_2::Error`] exactly as [`hl7_2::parse`] does: the input
    /// has no usable MSH header. Nothing is added here.
    pub fn parse(text: &str) -> Result<Message, hl7_2::Error> {
        hl7_2::parse(text).map(Message)
    }

    /// Parse ER7 text under `options` — a thin wrapper over
    /// [`hl7_2::parse_with_options`], for a forced release, a caller's own
    /// dictionary, or strict validation.
    ///
    /// # Errors
    ///
    /// Returns [`hl7_2::Error`] exactly as [`hl7_2::parse_with_options`]
    /// does.
    pub fn parse_with_options(
        text: &str,
        options: &hl7_2::Options,
    ) -> Result<Message, hl7_2::Error> {
        hl7_2::parse_with_options(text, options).map(Message)
    }

    /// A [`DeserializeSeed`] that reads a [`Message`] under `options`,
    /// for a message that has to come back through a caller's own
    /// dictionary or with strict validation on.
    ///
    /// The plain `Deserialize` impl has no way to be handed options, so it
    /// reads every message through the bundled dictionary for the release
    /// on the wire. This seed is the way to say otherwise. An
    /// [`hl7_2::Options::version`] set here wins over the `"version"` key
    /// on the wire, matching what that option means for
    /// [`hl7_2::parse_with_options`]; left `None`, the wire's version is
    /// used.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::sync::Arc;
    /// use serde::de::DeserializeSeed;
    /// use serde_hl7_v2::Message;
    ///
    /// let dictionary = hl7_2::Dictionary::from_json(
    ///     r#"{"inherits": "2.5", "segments": {"ZPD": ["ST", "XPN"]}}"#,
    ///     "acme",
    /// )?;
    /// let options = hl7_2::Options::new().with_dictionary(Arc::new(dictionary));
    ///
    /// let json = r#"{"version":"2.5","er7":"MSH|^~\\&|ACME||||1||ADT^A01|1|P|2.5\rZPD|7|SMITH^JOHN"}"#;
    /// let mut deserializer = serde_json::Deserializer::from_str(json);
    /// let message = Message::seed(&options).deserialize(&mut deserializer)?;
    ///
    /// // The vendor's own segment reads through the vendor's dictionary.
    /// assert_eq!(message.tree().find("XPN.2").unwrap().text(), "JOHN");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn seed(options: &hl7_2::Options) -> MessageSeed<'_> {
        MessageSeed {
            options,
            strict: false,
        }
    }
}

impl From<hl7_2::Message> for Message {
    fn from(inner: hl7_2::Message) -> Message {
        Message(inner)
    }
}

impl From<Message> for hl7_2::Message {
    fn from(outer: Message) -> hl7_2::Message {
        outer.0
    }
}

impl Deref for Message {
    type Target = hl7_2::Message;

    fn deref(&self) -> &hl7_2::Message {
        &self.0
    }
}

impl DerefMut for Message {
    fn deref_mut(&mut self) -> &mut hl7_2::Message {
        &mut self.0
    }
}

impl fmt::Display for Message {
    /// The message as ER7; see [`hl7_2::Message::to_er7`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_er7())
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Message", 2)?;
        state.serialize_field("version", &Version(self.0.version()))?;
        state.serialize_field("er7", &self.0.to_er7())?;
        state.end()
    }
}

const FIELDS: &[&str] = &["version", "er7"];

/// Deserializes a [`Message`] under a caller's [`hl7_2::Options`]; see
/// [`Message::seed`]. Implements [`DeserializeSeed`], so it is used through
/// `seed.deserialize(deserializer)` rather than a format's `from_str`.
#[derive(Debug, Clone, Copy)]
pub struct MessageSeed<'a> {
    options: &'a hl7_2::Options,
    strict: bool,
}

impl MessageSeed<'_> {
    /// The same seed, but rejecting an unknown key the way
    /// [`Strict<Message>`](Strict) does.
    #[must_use]
    pub fn strict(self) -> Self {
        MessageSeed {
            strict: true,
            ..self
        }
    }
}

impl<'de> DeserializeSeed<'de> for MessageSeed<'_> {
    type Value = Message;

    fn deserialize<D>(self, deserializer: D) -> Result<Message, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "Message",
            FIELDS,
            MessageVisitor {
                options: Some(self.options),
                strict: self.strict,
            },
        )
    }
}

/// `strict` distinguishes the ordinary tolerant `Deserialize` entry point
/// (rule S8) from [`Strict`]`<Message>`'s (rule S13). `options`, when
/// present, comes from a [`MessageSeed`] and is what the re-parse runs
/// under; absent, the re-parse runs under default options with the wire's
/// version pinned.
struct MessageVisitor<'a> {
    options: Option<&'a hl7_2::Options>,
    strict: bool,
}

impl<'de> Visitor<'de> for MessageVisitor<'_> {
    type Value = Message;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Message object with \"er7\" and, optionally, \"version\"")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Message, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut version: Option<Version> = None;
        let mut er7: Option<String> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "version" => {
                    if version.is_some() {
                        return Err(de::Error::duplicate_field("version"));
                    }
                    version = Some(map.next_value()?);
                }
                "er7" => {
                    if er7.is_some() {
                        return Err(de::Error::duplicate_field("er7"));
                    }
                    er7 = Some(map.next_value()?);
                }
                _ if self.strict => {
                    return Err(de::Error::unknown_field(&key, FIELDS));
                }
                _ => {
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        let er7 = er7.ok_or_else(|| de::Error::missing_field("er7"))?;
        let mut options = self.options.cloned().unwrap_or_default();
        if options.version.is_none() {
            options.version = version.map(|v| v.0);
        }
        hl7_2::parse_with_options(&er7, &options)
            .map(Message)
            .map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "Message",
            FIELDS,
            MessageVisitor {
                options: None,
                strict: false,
            },
        )
    }
}

impl<'de> Deserialize<'de> for Strict<Message> {
    /// See [`Strict`] (rule S13): an unrecognized key is a
    /// `serde::de::Error::unknown_field` rather than being ignored.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_struct(
                "Message",
                FIELDS,
                MessageVisitor {
                    options: None,
                    strict: true,
                },
            )
            .map(Strict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADT: &str = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260815120000||ADT^A08^ADT_A01|MSG9|P|2.5\r\
                       PID|1||12345^^^ACME&1.2.3&ISO^MR||SMITH^JOHN^Q||19800101|M|||||\
                       555-1111~555-2222\r\
                       OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL\r\
                       OBX|2|ST|X^Note^L||\"\"";

    #[test]
    fn round_trips_a_full_message_through_json() {
        let message = Message::parse(ADT).unwrap();
        let json = serde_json::to_string(&message).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back.to_er7(), ADT);
        assert_eq!(back.version(), message.version());
        assert_eq!(back.structure_id(), "ADT_A01");
    }

    #[test]
    fn serializes_version_and_er7_only() {
        let message = Message::parse(ADT).unwrap();
        let json = serde_json::to_value(&message).unwrap();
        let object = json.as_object().unwrap();
        assert_eq!(object.len(), 2);
        assert_eq!(object["version"], "2.5");
        assert_eq!(object["er7"], ADT);
    }

    #[test]
    fn a_forced_version_survives_the_round_trip() {
        // The wire carries the release the message was *read as*, not what
        // MSH-12 says, so a forced read comes back the same way.
        let options = hl7_2::Options::new().with_version(hl7_2::Version::V2_3_1);
        let message = Message::parse_with_options(ADT, &options).unwrap();
        assert_eq!(message.version(), hl7_2::Version::V2_3_1);
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains(r#""version":"2.3.1""#));
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version(), hl7_2::Version::V2_3_1);
    }

    #[test]
    fn version_is_optional_and_falls_back_to_msh_12() {
        let json = format!(r#"{{"er7":{}}}"#, serde_json::to_string(ADT).unwrap());
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version(), hl7_2::Version::V2_5);
    }

    #[test]
    fn rejects_a_message_missing_er7() {
        let err = serde_json::from_str::<Message>(r#"{"version":"2.5"}"#).unwrap_err();
        assert!(err.to_string().contains("er7"), "{err}");
    }

    #[test]
    fn rejects_er7_without_a_header_as_a_deserialize_error() {
        let err = serde_json::from_str::<Message>(r#"{"er7":"PID|1"}"#).unwrap_err();
        // hl7_2's own error text, surfaced through the format's error type.
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn rejects_a_duplicate_key() {
        let err = serde_json::from_str::<Message>(r#"{"er7":"MSH|^~\\&|A","er7":"MSH|^~\\&|B"}"#)
            .unwrap_err();
        assert!(err.to_string().contains("duplicate"), "{err}");
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{"version":"2.5","er7":"MSH|^~\\&|LAB","extra":true}"#;
        let back: Message = serde_json::from_str(json).unwrap();
        assert_eq!(back.to_er7(), "MSH|^~\\&|LAB");
    }

    #[test]
    fn strict_rejects_an_unknown_field() {
        let json = r#"{"version":"2.5","er7":"MSH|^~\\&|LAB","verison":"2.5"}"#;
        let err = serde_json::from_str::<Strict<Message>>(json).unwrap_err();
        assert!(err.to_string().contains("verison"), "{err}");
    }

    #[test]
    fn strict_still_requires_er7() {
        let err = serde_json::from_str::<Strict<Message>>(r#"{"version":"2.5"}"#).unwrap_err();
        assert!(err.to_string().contains("er7"), "{err}");
    }

    #[test]
    fn strict_accepts_a_real_message_with_no_typos() {
        let message = Message::parse(ADT).unwrap();
        let json = serde_json::to_string(&message).unwrap();
        let back = serde_json::from_str::<Strict<Message>>(&json).unwrap();
        assert_eq!(back.to_er7(), ADT);
    }

    #[test]
    fn plain_deserialize_is_unaffected_by_strict_existing() {
        let json = r#"{"version":"2.5","er7":"MSH|^~\\&|LAB","extra":true}"#;
        assert!(serde_json::from_str::<Message>(json).is_ok());
    }

    #[test]
    fn seed_reads_through_the_callers_dictionary() {
        use std::sync::Arc;
        let dictionary = hl7_2::Dictionary::from_json(
            r#"{"inherits": "2.5", "segments": {"ZPD": ["ST", "XPN"]}}"#,
            "acme",
        )
        .unwrap();
        let options = hl7_2::Options::new().with_dictionary(Arc::new(dictionary));
        let json =
            r#"{"version":"2.5","er7":"MSH|^~\\&|ACME||||1||ADT^A01|1|P|2.5\rZPD|7|SMITH^JOHN"}"#;

        let plain: Message = serde_json::from_str(json).unwrap();
        assert!(
            plain.tree().find("XPN.2").is_none(),
            "bundled dictionary knows no ZPD"
        );

        let mut deserializer = serde_json::Deserializer::from_str(json);
        let seeded = Message::seed(&options)
            .deserialize(&mut deserializer)
            .unwrap();
        assert_eq!(seeded.tree().find("XPN.2").unwrap().text(), "JOHN");
    }

    #[test]
    fn seed_version_option_wins_over_the_wire() {
        let options = hl7_2::Options::new().with_version(hl7_2::Version::V2_3);
        let json = r#"{"version":"2.5","er7":"MSH|^~\\&|LAB"}"#;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let message = Message::seed(&options)
            .deserialize(&mut deserializer)
            .unwrap();
        assert_eq!(message.version(), hl7_2::Version::V2_3);
    }

    #[test]
    fn seed_can_be_strict() {
        let options = hl7_2::Options::new();
        let json = r#"{"er7":"MSH|^~\\&|LAB","extra":1}"#;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let err = Message::seed(&options)
            .strict()
            .deserialize(&mut deserializer)
            .unwrap_err();
        assert!(err.to_string().contains("extra"), "{err}");
    }

    #[test]
    fn deref_reaches_the_inner_api() {
        let message = Message::parse(ADT).unwrap();
        assert_eq!(message.get("PID-5.1").unwrap().as_deref(), Some("SMITH"));
        assert_eq!(message.to_string(), ADT);
    }
}
