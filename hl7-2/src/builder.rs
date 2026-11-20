//! Building a message from nothing.
//!
//! Reading is the common case, but a system that reads HL7 usually has to
//! answer in it too — an `ACK` at minimum. A [`Builder`] starts from a
//! well-formed MSH header for a chosen release and adds segments and
//! values from there.
//!
//! Errors are collected rather than returned at each step, so a chain of
//! calls stays a chain and [`Builder::build`] reports what went wrong. The
//! builder takes no timestamp from the clock and invents no control ID:
//! both are the caller's, because a message that made up its own would be
//! untraceable and untestable.

use crate::{Error, Message, Options, Version};
use std::sync::Arc;

/// Assemble a message segment by segment; see the module documentation.
///
/// ```
/// use hl7_2::{Builder, Version};
///
/// let message = Builder::new(Version::V2_5)
///     .message_type("ADT", "A01")
///     .control_id("MSG00001")
///     .timestamp("20240101093851")
///     .sending("HIS", "HOSPITAL")
///     .receiving("EPIC", "CLINIC")
///     .segment("EVN")
///     .set("EVN-1", "A01")
///     .segment("PID")
///     .set("PID-3.1", "241900")
///     .set("PID-5.1.1", "SMITH")
///     .set("PID-5.2", "JOHN")
///     .build()?;
///
/// assert_eq!(message.structure_id(), "ADT_A01");
/// assert!(message.to_er7().contains("PID|||241900||SMITH^JOHN"));
/// # Ok::<(), hl7_2::Error>(())
/// ```
#[derive(Debug)]
pub struct Builder {
    message: Message,
    failures: Vec<Error>,
}

impl Builder {
    /// Start a message for `version`, with the standard delimiters
    /// (`|^~\&`), a processing ID of `P` (production), and MSH-12 set.
    ///
    /// Everything else — message type, control ID, timestamp, sender,
    /// receiver — is empty until set, and MSH-9 and MSH-10 being empty is
    /// exactly what [`Message::validate`] reports, so a half-built message
    /// says so rather than looking finished.
    #[must_use]
    /// # Panics
    ///
    /// Never in practice: the bundled dictionaries are embedded at compile
    /// time and parsed on first use, so a failure here would mean this
    /// crate shipped a malformed one.
    pub fn new(version: Version) -> Builder {
        let header = format!("MSH|^~\\&|||||||||P|{version}");
        let message = crate::parse_with_options(&header, &Options::new().with_version(version))
            .expect("a builder's own header is always well formed");
        Builder {
            message,
            failures: Vec::new(),
        }
    }

    /// Start a message for `version`, read through `dictionary` — the
    /// schema-mode counterpart of [`Builder::new`].
    #[must_use]
    /// # Panics
    ///
    /// Never in practice: the bundled dictionaries are embedded at compile
    /// time and parsed on first use, so a failure here would mean this
    /// crate shipped a malformed one.
    pub fn with_dictionary(version: Version, dictionary: Arc<crate::Dictionary>) -> Builder {
        let header = format!("MSH|^~\\&|||||||||P|{version}");
        let options = Options::new()
            .with_version(version)
            .with_dictionary(dictionary);
        let message = crate::parse_with_options(&header, &options)
            .expect("a builder's own header is always well formed");
        Builder {
            message,
            failures: Vec::new(),
        }
    }

    /// Start from an existing message, to add to it or answer it.
    #[must_use]
    pub fn from_message(message: Message) -> Builder {
        Builder {
            message,
            failures: Vec::new(),
        }
    }

    /// Set MSH-9: the message code and trigger event, e.g. `("ADT",
    /// "A01")`. The structure ID (MSH-9.3) is filled in from the
    /// dictionary, so `ADT^A04` correctly declares `ADT_A01`.
    #[must_use]
    pub fn message_type(mut self, code: &str, trigger: &str) -> Builder {
        let structure = self.message.dictionary().structure_id(code, trigger);
        self = self.set("MSH-9.1", code);
        self = self.set("MSH-9.2", trigger);
        self.set("MSH-9.3", &structure)
    }

    /// Set MSH-10, the message control ID: the sender's own identifier for
    /// this message, which the receiver echoes in its acknowledgement.
    #[must_use]
    pub fn control_id(self, id: &str) -> Builder {
        self.set("MSH-10", id)
    }

    /// Set MSH-7, the date and time the message was sent, as HL7 writes it
    /// (`YYYYMMDDHHMMSS`, optionally with a fraction and an offset).
    #[must_use]
    pub fn timestamp(self, timestamp: &str) -> Builder {
        self.set("MSH-7.1", timestamp)
    }

    /// Set MSH-3 and MSH-4, the sending application and facility.
    #[must_use]
    pub fn sending(self, application: &str, facility: &str) -> Builder {
        self.set("MSH-3.1", application).set("MSH-4.1", facility)
    }

    /// Set MSH-5 and MSH-6, the receiving application and facility.
    #[must_use]
    pub fn receiving(self, application: &str, facility: &str) -> Builder {
        self.set("MSH-5.1", application).set("MSH-6.1", facility)
    }

    /// Set MSH-11, the processing ID: `P` production, `T` training, `D`
    /// debugging. A builder starts at `P`.
    #[must_use]
    pub fn processing_id(self, id: &str) -> Builder {
        self.set("MSH-11.1", id)
    }

    /// Append an empty segment. Subsequent [`Builder::set`] calls that name
    /// this segment address this occurrence, because paths without an
    /// explicit `[n]` mean the first — so add a segment, fill it, then add
    /// the next.
    #[must_use]
    pub fn segment(mut self, name: &str) -> Builder {
        self.message.append_segment(name);
        self
    }

    /// Set a value, escaping delimiters in it; see [`Message::set`].
    #[must_use]
    pub fn set(mut self, path: &str, value: &str) -> Builder {
        if let Err(error) = self.message.set(path, value) {
            self.failures.push(error);
        }
        self
    }

    /// Set a value from text that is already ER7-encoded; see
    /// [`Message::set_er7`].
    #[must_use]
    pub fn set_er7(mut self, path: &str, er7_text: &str) -> Builder {
        self.failures
            .extend(self.message.set_er7(path, er7_text).err());
        self
    }

    /// Write a [`crate::ToHl7`] value's fields into the message being
    /// built — struct mode's other direction.
    #[must_use]
    pub fn encode(mut self, value: &impl crate::ToHl7) -> Builder {
        if let Err(error) = value.to_hl7(&mut self.message) {
            self.failures.push(error);
        }
        self
    }

    /// Finish, returning the message, or the first error a step hit.
    /// # Errors
    ///
    /// The first [`Error`] recorded while building — a path that could not
    /// be written, reported here rather than at the call that caused it, so
    /// a chain of setters reads as one expression.
    pub fn build(self) -> Result<Message, Error> {
        match self.failures.into_iter().next() {
            Some(error) => Err(error),
            None => Ok(self.message),
        }
    }

    /// Finish, and reject a message that does not pass validation. Use it
    /// for an outbound message, where sending something malformed costs
    /// more than noticing here.
    /// # Errors
    ///
    /// As [`Builder::build`], and additionally when the built message does
    /// not validate against its dictionary.
    pub fn build_valid(self) -> Result<Message, Error> {
        let message = self.build()?;
        let failures: Vec<crate::Diagnostic> = message
            .validate()
            .into_iter()
            .filter(|diagnostic| diagnostic.severity == crate::Severity::Error)
            .collect();
        if failures.is_empty() {
            Ok(message)
        } else {
            Err(Error::Invalid(failures))
        }
    }
}

/// Build an `ACK` acknowledging `message`, in the release `message` speaks.
///
/// The reply uses the received message's separators and swaps its complete
/// sending and receiving application and facility fields. MSA-2 echoes the
/// received control ID so the sender can match the reply to the request.
/// The caller supplies the reply's own control ID and timestamp.
///
/// # Panics
///
/// Only if the crate's `Message` invariant is broken internally and the
/// message no longer contains its required MSH header. Public constructors
/// and parsers preserve that invariant.
///
/// ```
/// let message = hl7_2::parse("MSH|^~\\&|LAB|L|EPIC|E|20240101||ORU^R01|99|P|2.5\rPID|1")?;
/// let ack = hl7_2::builder::acknowledge(&message, "AA", "ACK00001", "20240101093900").build()?;
/// assert_eq!(ack.get("MSA-2")?.as_deref(), Some("99"));
/// // The answer goes back where it came from.
/// assert_eq!(ack.get("MSH-5.1")?.as_deref(), Some("LAB"));
/// # Ok::<(), hl7_2::Error>(())
/// ```
#[must_use]
pub fn acknowledge(message: &Message, code: &str, control_id: &str, timestamp: &str) -> Builder {
    let value = |path: &str| message.get(path).ok().flatten().unwrap_or_default();
    let version = message.version();
    let mut builder = Builder::new(version);
    let received = message
        .raw()
        .header()
        .expect("Message invariant guarantees an MSH header");
    let reply = builder.message.raw_mut();
    reply.separators = *message.separators();
    // Copy the delimiter declaration and the swapped routing fields as
    // parsed ER7, preserving component boundaries and escapes.
    for (index, field) in [1, 2, 5, 6, 3, 4].into_iter().enumerate() {
        reply.segments[0].fields[index] = received.field(field).cloned().unwrap_or_default();
    }
    builder = builder.set("MSH-12", &version.to_string());
    builder
        .message_type("ACK", &value("MSH-9.2"))
        .control_id(control_id)
        .timestamp(timestamp)
        .processing_id(&value("MSH-11.1"))
        .segment("MSA")
        .set("MSA-1", code)
        .set("MSA-2", &value("MSH-10"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_message_from_nothing() {
        let message = Builder::new(Version::V2_5)
            .message_type("ADT", "A01")
            .control_id("1")
            .timestamp("20240101093851")
            .segment("EVN")
            .set("EVN-1", "A01")
            .segment("PID")
            .set("PID-3.1", "241900")
            .segment("PV1")
            .set("PV1-2", "I")
            .build()
            .unwrap();
        assert_eq!(message.version(), Version::V2_5);
        assert_eq!(message.structure_id(), "ADT_A01");
        assert_eq!(message.get("PID-3.1").unwrap().as_deref(), Some("241900"));
        // Built messages are valid messages.
        assert_eq!(message.validate(), []);
        // And they parse back to themselves.
        let text = message.to_er7();
        assert_eq!(crate::parse(&text).unwrap().to_er7(), text);
    }

    #[test]
    fn resolves_the_structure_id_through_the_dictionary() {
        let message = Builder::new(Version::V2_5)
            .message_type("ADT", "A08")
            .build()
            .unwrap();
        assert_eq!(message.get("MSH-9.3").unwrap().as_deref(), Some("ADT_A01"));
    }

    #[test]
    fn reports_the_first_failure_at_build_time() {
        let error = Builder::new(Version::V2_5)
            .set("PID-3.1", "241900") // no PID segment yet
            .build()
            .unwrap_err();
        assert!(matches!(error, Error::NoSuchSegment { .. }), "{error}");
    }

    #[test]
    fn alternate_constructors_and_fallible_steps_are_exercised() {
        struct Invalid;
        impl crate::ToHl7 for Invalid {
            fn to_hl7(&self, message: &mut Message) -> Result<(), Error> {
                message.set("PID-1", "1")
            }
        }

        let message = Builder::with_dictionary(Version::V2_5, Version::V2_5.dictionary())
            .message_type("ACK", "A01")
            .control_id("1")
            .build()
            .unwrap();
        assert_eq!(message.structure_id(), "ACK");
        assert!(Builder::from_message(message).build().is_ok());
        assert!(
            Builder::new(Version::V2_5)
                .encode(&Invalid)
                .build()
                .is_err()
        );
        assert!(
            Builder::new(Version::V2_5)
                .set_er7("PID-0", "already\\F\\encoded")
                .build()
                .is_err()
        );
    }

    #[test]
    fn build_valid_rejects_an_incomplete_message() {
        // No message type and no control ID: not something to send.
        let error = Builder::new(Version::V2_5).build_valid().unwrap_err();
        assert!(matches!(&error, Error::Invalid(diagnostics) if diagnostics.len() == 2));
    }

    #[test]
    fn acknowledges_a_message() {
        let message = crate::parse(
            "MSH|^~\\&|LAB|LAB1|EPIC|CLINIC|20240101||ORU^R01|99|P|2.5\rPID|1\rOBR|1\rOBX|1|NM|X||7",
        )
        .unwrap();
        let ack = acknowledge(&message, "AA", "ACK1", "20240101093900")
            .build_valid()
            .unwrap();
        assert_eq!(ack.structure_id(), "ACK");
        assert_eq!(ack.get("MSA-1").unwrap().as_deref(), Some("AA"));
        assert_eq!(ack.get("MSA-2").unwrap().as_deref(), Some("99"));
        assert_eq!(ack.get("MSH-5.1").unwrap().as_deref(), Some("LAB"));
        assert_eq!(ack.get("MSH-6.1").unwrap().as_deref(), Some("LAB1"));
        assert_eq!(ack.get("MSH-3.1").unwrap().as_deref(), Some("EPIC"));
    }

    #[test]
    fn swaps_complete_routing_fields() {
        let message = crate::parse(concat!(
            "MSH|^~\\&|LAB^1.2.3^ISO|SITE^1.2.4^ISO",
            "|EHR^urn:ehr^URI|^1.2.5^ISO|20260814||ORU^R01|MSG1|P|2.5",
        ))
        .unwrap();
        let ack = acknowledge(&message, "AA", "ACK1", "20260814")
            .build_valid()
            .unwrap();
        let received = message.segment("MSH").unwrap();
        let sent = ack.segment("MSH").unwrap();
        for (from, to) in [(3, 5), (4, 6), (5, 3), (6, 4)] {
            assert_eq!(sent.field(to), received.field(from), "MSH-{to}");
        }
    }

    #[test]
    fn acknowledges_in_the_senders_release() {
        let message =
            crate::parse("MSH|^~\\&|LAB|L|EPIC|E|20240101||ORU^R01|99|P|2.3\rPID|1").unwrap();
        let ack = acknowledge(&message, "AE", "1", "20240101")
            .build()
            .unwrap();
        assert_eq!(ack.version(), Version::V2_3);
        assert_eq!(ack.get("MSH-12").unwrap().as_deref(), Some("2.3"));
    }
}
