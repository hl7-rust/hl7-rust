//! MLLP — the Minimal Lower Layer Protocol, which is how HL7 v2 messages
//! actually cross a network.
//!
//! A TCP stream is bytes without edges, and an HL7 v2 message carries no
//! length prefix and no self-delimiting syntax, so a receiver reading a
//! socket cannot tell where one message stops and the next begins. MLLP is
//! the three-byte answer to that, and nothing more: wrap each message in a
//! start block and an end block.
//!
//! ```text
//! <VT> message <FS><CR>
//! 0x0B          0x1C 0x0D
//! ```
//!
//! That is the whole protocol. It is deliberately minimal — no length, no
//! checksum, no session, no negotiation, no encryption — and everything
//! else people expect of a messaging layer is either HL7's own
//! acknowledgement messages ([`ack`]), TLS underneath, or the caller's
//! business.
//!
//! ## What is here
//!
//! | | |
//! |---|---|
//! | [`encode`], [`decode`] | one frame in hand |
//! | [`Framer`] | a byte stream, where frames arrive split across reads or several to a read — this is the one a socket needs |
//! | [`Transport`], [`IoTransport`] | frames over anything that reads and writes bytes |
//! | [`ack`] | turning a received message into the acknowledgement HL7 expects back |
//!
//! ```
//! use hl7_2_mllp as mllp;
//!
//! let message = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|99|P|2.5\rPID|1";
//! let frame = mllp::encode(message.as_bytes());
//!
//! assert_eq!(frame[0], mllp::START_BLOCK);
//! assert_eq!(&frame[frame.len() - 2..], &[mllp::END_BLOCK, mllp::CARRIAGE_RETURN]);
//! assert_eq!(mllp::decode(&frame)?, message.as_bytes());
//! # Ok::<(), mllp::Error>(())
//! ```
//!
//! ## Reading a socket
//!
//! ```no_run
//! use hl7_2_mllp::{IoTransport, Transport};
//! use std::net::TcpListener;
//!
//! let listener = TcpListener::bind("127.0.0.1:2575")?;
//! for stream in listener.incoming() {
//!     let mut transport = IoTransport::new(stream?);
//!     while let Some(message) = transport.receive()? {
//!         // ... process the message, then answer ...
//!         # let _ = &message;
//!     }
//! }
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! See `examples/tcp_listener.rs` for a complete server that also
//! acknowledges, and `examples/tcp_sender.rs` for the other end.
//!
//! ## Strictness
//!
//! By default this crate is strict: a frame must start with `<VT>`, end
//! with `<FS><CR>`, and contain neither block character in between. Real
//! senders are not always strict — a missing `<CR>` after `<FS>`, and stray
//! bytes between frames, are the two common sins — so the `noncompliance`
//! feature relaxes exactly those two and nothing else.
//!
//! It is off by default because a receiver that quietly accepts malformed
//! framing is how a truncated message becomes a clinical record. Turn it on
//! when you have a specific sender that needs it, and know which of the two
//! you are forgiving.
//!
//! `spec/index.md` in the repository is the normative specification of
//! everything above; where this documentation and that document disagree,
//! that document is right.

#![warn(missing_docs, clippy::pedantic)]

#[cfg(feature = "ack")]
pub mod ack;
mod framer;
mod transport;

#[cfg(feature = "ack")]
pub use ack::AckCode;
pub use framer::{Framer, Tolerance};
pub use transport::{IoTransport, Transport};

/// The HL7 v2 crate acknowledgements are built with, re-exported so callers
/// can name [`hl7_2::Message`] without adding their own dependency.
#[cfg(feature = "ack")]
pub use hl7_2;

use std::fmt;

/// `<VT>`, the start block: a vertical tab, `0x0B`. Begins every frame.
pub const START_BLOCK: u8 = 0x0B;

/// `<FS>`, the end block: a file separator, `0x1C`. Ends every frame,
/// followed by [`CARRIAGE_RETURN`].
pub const END_BLOCK: u8 = 0x1C;

/// `<CR>`, the carriage return `0x0D` that follows [`END_BLOCK`].
///
/// It is also HL7 v2's segment terminator, which is why a message's own
/// trailing `\r` — if the sender wrote one — sits harmlessly before the
/// `<FS>` rather than being mistaken for this one.
pub const CARRIAGE_RETURN: u8 = 0x0D;

/// The default cap on how much a [`Framer`] buffers while waiting for an
/// end block: 16 MiB.
///
/// MLLP has no length field, so a sender that never sends `<FS>` — or a
/// peer speaking some other protocol entirely, or a port scanner — would
/// otherwise grow the buffer until the process dies. A real HL7 v2 message
/// is kilobytes; megabytes only when it carries a document in OBX-5.
pub const DEFAULT_LIMIT: usize = 16 * 1024 * 1024;

/// Wrap a payload in a frame: `<VT>` + payload + `<FS><CR>`.
///
/// The payload is neither inspected nor modified. MLLP has no escaping — it
/// cannot, having defined no escape character — so a payload containing
/// `<VT>` or `<FS>` cannot be framed unambiguously. [`is_framable`] is the
/// check; HL7 v2 text never contains either byte.
#[must_use]
pub fn encode(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len() + 3);
    frame.push(START_BLOCK);
    frame.extend_from_slice(payload);
    frame.push(END_BLOCK);
    frame.push(CARRIAGE_RETURN);
    frame
}

/// Whether `payload` can be framed unambiguously — that is, whether it is
/// free of the two bytes MLLP reserves.
///
/// [`encode`] does not check, because for HL7 v2 text the answer is always
/// yes and a caller framing something else already knows what they are
/// doing. Check when the payload came from somewhere you do not control.
#[must_use]
pub fn is_framable(payload: &[u8]) -> bool {
    !payload
        .iter()
        .any(|&byte| byte == START_BLOCK || byte == END_BLOCK)
}

/// Unwrap one complete frame, returning the payload.
///
/// This is for a frame already in hand: a test fixture, a file, a datagram.
/// Against a stream use [`Framer`], which handles a frame split across
/// reads and several frames in one read.
/// # Errors
///
/// [`Error`] when the bytes are not one complete frame: no start block, no
/// end block, or a missing carriage return after it.
pub fn decode(frame: &[u8]) -> Result<&[u8], Error> {
    decode_with(frame, Tolerance::default())
}

/// Unwrap one complete frame at a chosen [`Tolerance`].
/// # Errors
///
/// The same conditions as [`decode`], less whatever `tolerance` forgives.
pub fn decode_with(frame: &[u8], tolerance: Tolerance) -> Result<&[u8], Error> {
    let Some((&first, rest)) = frame.split_first() else {
        return Err(Error::Incomplete);
    };
    if first != START_BLOCK {
        return Err(Error::NoStartBlock);
    }
    let Some(end) = rest.iter().position(|&byte| byte == END_BLOCK) else {
        return Err(Error::Incomplete);
    };
    let payload = &rest[..end];
    if payload.contains(&START_BLOCK) {
        return Err(Error::EmbeddedStartBlock);
    }
    match &rest[end + 1..] {
        [CARRIAGE_RETURN] => Ok(payload),
        [] if tolerance.allows_missing_carriage_return() => Ok(payload),
        [] => Err(Error::Incomplete),
        [CARRIAGE_RETURN, extra @ ..] => Err(Error::TrailingBytes(extra.len())),
        extra if tolerance.allows_missing_carriage_return() => {
            Err(Error::TrailingBytes(extra.len()))
        }
        _ => Err(Error::NoCarriageReturn),
    }
}

/// What can go wrong reading a frame.
///
/// Every variant means the bytes on the wire are not MLLP. None of them
/// means the *message* is wrong — that question belongs one layer up, to
/// `hl7_2::Message::validate`, and can only be asked once framing has
/// succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The frame does not begin with `<VT>`.
    NoStartBlock,
    /// `<FS>` was not followed by `<CR>`.
    NoCarriageReturn,
    /// The frame stops mid-way: no `<FS><CR>` yet.
    ///
    /// Against a stream this is not an error but a "read more", which is
    /// what [`Framer`] does with it; from [`decode`] it means the caller was
    /// handed a partial frame.
    Incomplete,
    /// A complete frame was followed by bytes that do not begin another.
    /// Carries how many.
    TrailingBytes(usize),
    /// A `<VT>` inside the payload, which makes where the frame ends
    /// ambiguous.
    EmbeddedStartBlock,
    /// Bytes arrived before `<VT>`, outside any frame. Carries how many.
    /// The `noncompliance` feature discards them instead.
    LeadingBytes(usize),
    /// More bytes accumulated than the limit allows without a complete
    /// frame arriving; see [`DEFAULT_LIMIT`].
    TooLarge {
        /// How many bytes had accumulated.
        buffered: usize,
        /// The limit that was exceeded.
        limit: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoStartBlock => write!(f, "frame does not begin with the MLLP start block"),
            Error::NoCarriageReturn => {
                write!(f, "the MLLP end block is not followed by a carriage return")
            }
            Error::Incomplete => write!(f, "frame is incomplete: no end block yet"),
            Error::TrailingBytes(count) => {
                write!(
                    f,
                    "{count} byte(s) follow the frame without beginning another"
                )
            }
            Error::EmbeddedStartBlock => {
                write!(
                    f,
                    "a start block inside the payload makes the frame ambiguous"
                )
            }
            Error::LeadingBytes(count) => write!(f, "{count} byte(s) arrived outside any frame"),
            Error::TooLarge { buffered, limit } => write!(
                f,
                "buffered {buffered} bytes without a complete frame, over the {limit}-byte limit"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for std::io::Error {
    /// A framing error reaching a caller through [`Transport`] is an I/O
    /// error of kind `InvalidData`: the connection worked, what came over
    /// it did not.
    fn from(error: Error) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MESSAGE: &str = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|99|P|2.5\rPID|1";

    #[test]
    fn wraps_and_unwraps_a_message() {
        let frame = encode(MESSAGE.as_bytes());
        assert_eq!(frame[0], START_BLOCK);
        assert_eq!(frame[frame.len() - 2], END_BLOCK);
        assert_eq!(frame[frame.len() - 1], CARRIAGE_RETURN);
        assert_eq!(decode(&frame).unwrap(), MESSAGE.as_bytes());
    }

    #[test]
    fn leaves_the_payload_exactly_as_it_was() {
        // A message's own segment terminators are the same byte as the
        // frame's trailer, and must survive untouched.
        assert_eq!(decode(&encode(b"A\rB\r")).unwrap(), b"A\rB\r");
        assert_eq!(decode(&encode(b"")).unwrap(), b"");
        assert_eq!(decode(&encode(&[0u8, 255, 128])).unwrap(), &[0u8, 255, 128]);
    }

    #[test]
    fn refuses_framing_that_is_not_framing() {
        // Pinned to strict, so this says the same thing whether or not the
        // `noncompliance` feature is on.
        fn strict(bytes: &[u8]) -> Result<&[u8], Error> {
            decode_with(bytes, Tolerance::strict())
        }
        assert_eq!(strict(b""), Err(Error::Incomplete));
        assert_eq!(strict(b"MSH|"), Err(Error::NoStartBlock));
        assert_eq!(strict(b"\x0bMSH|"), Err(Error::Incomplete));
        assert_eq!(strict(b"\x0bMSH|\x1c"), Err(Error::Incomplete));
        assert_eq!(strict(b"\x0bMSH|\x1cX"), Err(Error::NoCarriageReturn));
        assert_eq!(strict(b"\x0bMSH|\x1c\rextra"), Err(Error::TrailingBytes(5)));
        assert_eq!(strict(b"\x0bA\x0bB\x1c\r"), Err(Error::EmbeddedStartBlock));
    }

    #[test]
    fn the_feature_chooses_the_default_and_nothing_else() {
        // Whichever way this build is compiled, the default is one of the
        // two tolerances, and both remain reachable by name.
        let missing_carriage_return = b"\x0bMSH|\x1c";
        if cfg!(feature = "noncompliance") {
            assert_eq!(Tolerance::default(), Tolerance::Lenient);
            assert_eq!(decode(missing_carriage_return).unwrap(), b"MSH|");
        } else {
            assert_eq!(Tolerance::default(), Tolerance::Strict);
            assert_eq!(decode(missing_carriage_return), Err(Error::Incomplete));
        }
        assert_eq!(
            decode_with(missing_carriage_return, Tolerance::strict()),
            Err(Error::Incomplete)
        );
        assert_eq!(
            decode_with(missing_carriage_return, Tolerance::lenient()).unwrap(),
            b"MSH|"
        );
    }

    #[test]
    fn knows_what_cannot_be_framed() {
        assert!(is_framable(MESSAGE.as_bytes()));
        assert!(!is_framable(b"before\x0bafter"));
        assert!(!is_framable(b"before\x1cafter"));
    }

    #[test]
    fn tolerance_forgives_a_missing_carriage_return_and_nothing_else() {
        let lenient = Tolerance::lenient();
        assert_eq!(decode_with(b"\x0bMSH|\x1c", lenient).unwrap(), b"MSH|");
        assert_eq!(decode_with(b"\x0bMSH|\x1c\r", lenient).unwrap(), b"MSH|");
        // Still not a frame, however tolerant we are being.
        assert_eq!(decode_with(b"MSH|\x1c", lenient), Err(Error::NoStartBlock));
        assert_eq!(
            decode_with(b"\x0bA\x0bB\x1c\r", lenient),
            Err(Error::EmbeddedStartBlock)
        );
    }

    #[test]
    fn errors_carry_across_the_io_boundary() {
        let error = std::io::Error::from(Error::NoStartBlock);
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("start block"));
    }
}
