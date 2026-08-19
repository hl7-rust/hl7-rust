//! Answering a message.
//!
//! MLLP itself has no acknowledgement: it frames bytes and stops there. The
//! acknowledgement HL7 expects is an HL7 message of its own — an `ACK`,
//! carrying an `MSA` whose second field echoes the control ID of the
//! message being answered — framed and sent back over the same connection.
//! That echo is the whole mechanism: it is what lets a sender match an
//! answer to the question, and what lets it know that message 99, and not
//! merely *a* message, arrived.
//!
//! ```
//! use hl7_v2_mllp::{AckCode, ack};
//!
//! let received = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|99|P|2.5\rPID|1";
//!
//! let frame = ack::acknowledge(
//!     received.as_bytes(),
//!     AckCode::Accept,
//!     "ACK00001",              // this acknowledgement's own control ID
//!     "20260814080100",        // and its own timestamp
//! )?;
//!
//! let text = String::from_utf8(hl7_v2_mllp::decode(&frame)?.to_vec()).unwrap();
//! assert!(text.contains("MSA|AA|99"));      // 99: the message being answered
//! assert!(text.starts_with("MSH|^~\\&|EHR|CLINIC|LAB|ACME|"));  // sender and receiver swap
//! # Ok::<(), ack::Error>(())
//! ```
//!
//! ## The clock is opt-in
//!
//! Every call here takes the timestamp and the control ID as arguments,
//! because a message that invents its own is untestable — the output
//! changes every run — and untraceable, since the control ID is what an
//! operator greps for when a sender asks what happened to message 99.
//!
//! With the `clock` feature, [`acknowledge_now`] fills in the timestamp
//! from the system clock for callers who genuinely just want the current
//! time. The control ID stays the caller's: only they know what their own
//! messages are called.

use crate::{Error as FrameError, encode};
use hl7_v2 as v2;
use std::fmt;

/// What an acknowledgement says happened, in MSA-1.
///
/// The three `Commit*` codes are the enhanced acknowledgement mode of
/// HL7 v2.3.1 and later, where a receiver can say "I have it safely" before
/// it says "I processed it". Use them only if the sender asked for enhanced
/// mode in MSH-15/MSH-16; senders that did not ask expect `AA`, `AE`, `AR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckCode {
    /// `AA` — accepted and processed.
    Accept,
    /// `AE` — an error in the message; the sender should fix it and may
    /// resend.
    Error,
    /// `AR` — rejected for a reason unrelated to the message's content: the
    /// receiver is shutting down, unconfigured, out of sequence.
    Reject,
    /// `CA` — enhanced mode: committed to safe storage, not yet processed.
    CommitAccept,
    /// `CE` — enhanced mode: could not commit, because of the message.
    CommitError,
    /// `CR` — enhanced mode: could not commit, for other reasons.
    CommitReject,
}

impl AckCode {
    /// The two-letter code as it goes in MSA-1.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AckCode::Accept => "AA",
            AckCode::Error => "AE",
            AckCode::Reject => "AR",
            AckCode::CommitAccept => "CA",
            AckCode::CommitError => "CE",
            AckCode::CommitReject => "CR",
        }
    }

    /// Whether this code says the message was accepted (`AA` or `CA`).
    #[must_use]
    pub fn is_accept(self) -> bool {
        matches!(self, AckCode::Accept | AckCode::CommitAccept)
    }
}

impl fmt::Display for AckCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why an acknowledgement could not be produced.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The received payload is not valid UTF-8, so it is not a message this
    /// crate can read. (HL7 v2 may declare another character set in MSH-18;
    /// decode it yourself and use [`acknowledge_message`].)
    NotText,
    /// The received payload is not a readable HL7 v2 message.
    NotHl7(v2::Error),
    /// The acknowledgement was built but could not be assembled.
    Build(v2::Error),
    /// The received bytes are not a valid MLLP frame.
    Framing(FrameError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotText => write!(f, "the received payload is not valid UTF-8"),
            Error::NotHl7(error) => {
                write!(f, "the received payload is not an HL7 message: {error}")
            }
            Error::Build(error) => write!(f, "could not build the acknowledgement: {error}"),
            Error::Framing(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<FrameError> for Error {
    fn from(error: FrameError) -> Error {
        Error::Framing(error)
    }
}

/// Build the acknowledgement for a received message, framed and ready to
/// write back to the socket.
///
/// `payload` is the message as [`crate::Transport::receive`] returned it —
/// unframed. `control_id` and `timestamp` belong to the acknowledgement
/// itself, not to the message being answered; see the module documentation
/// on why they are arguments.
/// # Errors
///
/// [`Error::NotText`] when the payload is not UTF-8, [`Error::NotHl7`] when
/// it is text but not a readable HL7 message, and [`Error::Build`] when the
/// acknowledgement could not be assembled from it.
pub fn acknowledge(
    payload: &[u8],
    code: AckCode,
    control_id: &str,
    timestamp: &str,
) -> Result<Vec<u8>, Error> {
    let message = parse(payload)?;
    let ack = acknowledge_message(&message, code, control_id, timestamp).map_err(Error::Build)?;
    Ok(encode(ack.to_er7().as_bytes()))
}

/// Build the acknowledgement for a message already parsed — when the
/// receiver needs to look at the message before deciding what to say about
/// it, which is the usual case.
///
/// Returns the acknowledgement as a message, so a caller can add an `ERR`
/// segment or a text explanation before framing it with [`crate::encode`].
///
/// ```
/// use hl7_v2_mllp::{AckCode, ack, encode};
///
/// let message = hl7_v2::parse("MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814||ORU^R01|99|P|2.5")?;
///
/// // Say what was wrong, not merely that something was.
/// let mut nack = ack::acknowledge_message(&message, AckCode::Error, "N1", "20260814080100")?;
/// nack.set("MSA-3", "OBR-4 is required")?;
/// let frame = encode(nack.to_er7().as_bytes());
/// # assert!(String::from_utf8(frame).unwrap().contains("MSA|AE|99|OBR-4 is required"));
/// # Ok::<(), hl7_v2::Error>(())
/// ```
/// # Errors
///
/// [`v2::Error`] when the acknowledgement cannot be assembled — a message
/// whose header is missing what an acknowledgement has to echo back.
pub fn acknowledge_message(
    message: &v2::Message,
    code: AckCode,
    control_id: &str,
    timestamp: &str,
) -> Result<v2::Message, v2::Error> {
    v2::builder::acknowledge(message, code.as_str(), control_id, timestamp).build()
}

/// Build the acknowledgement for a received message, timestamped from the
/// system clock. Requires the `clock` feature.
#[cfg(feature = "clock")]
pub fn acknowledge_now(payload: &[u8], code: AckCode, control_id: &str) -> Result<Vec<u8>, Error> {
    acknowledge(payload, code, control_id, &now())
}

/// The current local time as HL7 writes it: `YYYYMMDDHHMMSS`.
///
/// Local, not UTC, and with no offset, because that is what the installed
/// base does — an interface engine comparing timestamps expects the wall
/// clock of the machine that sent them. Requires the `clock` feature.
#[cfg(feature = "clock")]
pub fn now() -> String {
    chrono::Local::now().format("%Y%m%d%H%M%S").to_string()
}

/// Read a received payload as an HL7 v2 message.
///
/// The two failure modes are distinguished — not text at all, versus text
/// that is not HL7 — because they mean different things about the peer:
/// the first is usually a character-set or protocol mismatch, the second a
/// sender writing something else into the frame.
/// # Errors
///
/// [`Error::NotText`] when the payload is not UTF-8, and [`Error::NotHl7`]
/// when it is text but not an HL7 v2 message.
pub fn parse(payload: &[u8]) -> Result<v2::Message, Error> {
    let text = std::str::from_utf8(payload).map_err(|_| Error::NotText)?;
    v2::parse(text).map_err(Error::NotHl7)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECEIVED: &str =
        "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|99|P|2.5\rPID|1\rOBR|1";

    #[test]
    fn answers_a_message_with_its_own_control_id() {
        let frame = acknowledge(
            RECEIVED.as_bytes(),
            AckCode::Accept,
            "ACK00001",
            "20260814080100",
        )
        .unwrap();
        let text = String::from_utf8(crate::decode(&frame).unwrap().to_vec()).unwrap();
        assert_eq!(
            text,
            "MSH|^~\\&|EHR|CLINIC|LAB|ACME|20260814080100||ACK^R01^ACK|ACK00001|P|2.5\r\
             MSA|AA|99"
        );
    }

    #[test]
    fn sender_and_receiver_change_places() {
        let frame = acknowledge(RECEIVED.as_bytes(), AckCode::Accept, "1", "20260814").unwrap();
        let ack = parse(crate::decode(&frame).unwrap()).unwrap();
        assert_eq!(ack.get("MSH-3.1").unwrap().as_deref(), Some("EHR"));
        assert_eq!(ack.get("MSH-5.1").unwrap().as_deref(), Some("LAB"));
        assert_eq!(ack.get("MSH-6.1").unwrap().as_deref(), Some("ACME"));
    }

    #[test]
    fn every_code_reaches_msa_1() {
        for (code, expected) in [
            (AckCode::Accept, "AA"),
            (AckCode::Error, "AE"),
            (AckCode::Reject, "AR"),
            (AckCode::CommitAccept, "CA"),
            (AckCode::CommitError, "CE"),
            (AckCode::CommitReject, "CR"),
        ] {
            let frame = acknowledge(RECEIVED.as_bytes(), code, "1", "20260814").unwrap();
            let ack = parse(crate::decode(&frame).unwrap()).unwrap();
            assert_eq!(ack.get("MSA-1").unwrap().as_deref(), Some(expected));
            assert_eq!(code.to_string(), expected);
        }
        assert!(AckCode::Accept.is_accept());
        assert!(AckCode::CommitAccept.is_accept());
        assert!(!AckCode::Error.is_accept());
    }

    #[test]
    fn answers_in_the_release_the_sender_spoke() {
        let received = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814||ORU^R01|99|P|2.3\rPID|1";
        let frame = acknowledge(received.as_bytes(), AckCode::Accept, "1", "20260814").unwrap();
        let ack = parse(crate::decode(&frame).unwrap()).unwrap();
        assert_eq!(ack.version(), v2::Version::V2_3);
        assert_eq!(ack.get("MSH-12").unwrap().as_deref(), Some("2.3"));
    }

    #[test]
    fn a_payload_that_is_not_a_message_says_which_way_it_failed() {
        assert_eq!(
            acknowledge(&[0xff, 0xfe], AckCode::Accept, "1", "2"),
            Err(Error::NotText)
        );
        match acknowledge(b"not hl7", AckCode::Accept, "1", "2") {
            Err(Error::NotHl7(_)) => {}
            other => panic!("expected a parse failure, got {other:?}"),
        }
    }

    #[test]
    fn a_receiver_can_say_what_was_wrong() {
        let message = v2::parse(RECEIVED).unwrap();
        let mut nack =
            acknowledge_message(&message, AckCode::Error, "N1", "20260814080100").unwrap();
        nack.set("MSA-3", "OBR-4 is required").unwrap();
        assert!(nack.to_er7().contains("MSA|AE|99|OBR-4 is required"));
    }

    #[cfg(feature = "clock")]
    #[test]
    fn the_clock_feature_fills_in_the_timestamp() {
        let stamp = now();
        assert_eq!(stamp.len(), 14, "{stamp}");
        assert!(stamp.bytes().all(|b| b.is_ascii_digit()), "{stamp}");
        let frame = acknowledge_now(RECEIVED.as_bytes(), AckCode::Accept, "1").unwrap();
        let ack = parse(crate::decode(&frame).unwrap()).unwrap();
        assert_eq!(ack.get("MSH-7.1").unwrap().unwrap().len(), 14);
        assert_eq!(ack.validate(), []);
    }
}
