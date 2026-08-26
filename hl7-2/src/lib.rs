//! Parse, navigate, validate, modify, and write HL7 v2 messages, in three
//! modes that share one set of internals.
//!
//! HL7 v2 is the format most healthcare data still moves in, and most of
//! the difficulty in reading it is not the syntax — that is pipes and
//! carets, and the [`er7`] crate this one is built on already handles it —
//! but knowing what the pipes and carets *mean* in the release the sender
//! speaks. This crate owns that knowledge: the per-release data-type
//! tables, the message structures, and the three ways to apply them.
//!
//! Published standalone as `hl7-2`; most users get it through the `hl7`
//! umbrella crate instead, which re-exports this crate as `hl7::v2`:
//!
//! ```toml
//! [dependencies]
//! hl7 = "0.1"
//! ```
//!
//! ## Three modes
//!
//! **Generic** — for the vendor whose messages you have never seen and need
//! to explore. Parse anything into a navigable tree; nothing is rejected
//! and nothing is dropped.
//!
//! ```
//! let message = hl7_2::parse("MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\r\
//!                              PID|1||241900||SMITH^JOHN\r\
//!                              OBR|1||X|GLU\r\
//!                              OBX|1|NM|GLU^Glucose||7.4|mmol/L")?;
//! let tree = message.tree();
//! assert_eq!(tree.name(), "ORU_R01");
//! assert_eq!(tree.find("XPN.1").unwrap().text(), "SMITH");
//! assert_eq!(message.get("OBX-5")?.as_deref(), Some("7.4"));
//! # Ok::<(), hl7_2::Error>(())
//! ```
//!
//! **Schema-based** — for the vendor whose quirks you have learned but
//! whose format is not frozen. Write the shape as JSON, load it at runtime,
//! and adding a field needs no recompile.
//!
//! ```
//! use std::sync::Arc;
//!
//! let dictionary = hl7_2::Dictionary::from_json(r#"{
//!   "inherits": "2.5",
//!   "segments": { "ZPD": ["ST", "XPN"] }
//! }"#, "acme")?;
//! let options = hl7_2::Options::new().with_dictionary(Arc::new(dictionary));
//! let message = hl7_2::parse_with_options(
//!     "MSH|^~\\&|ACME||||1||ADT^A01|1|P|2.5\rZPD|7|SMITH^JOHN",
//!     &options,
//! )?;
//! // The vendor's own segment now reads like any standard one.
//! assert_eq!(message.tree().find("XPN.2").unwrap().text(), "JOHN");
//! # Ok::<(), hl7_2::Error>(())
//! ```
//!
//! **Struct-based** — for the stable, long-lived feed where you want the
//! compiler's help. See [`typed`] for the derive macros, and for the [`Raw`]
//! field that keeps the generic escape hatch open on the same object.
//!
//! ## What this crate is, and is not
//!
//! It is the HL7 v2 dictionary layer: releases 2.1 through 2.9, data types,
//! message structures, three modes, mutation, and validation. It is not the
//! ER7 encoding layer — parsing, delimiters, escape sequences, and
//! byte-for-byte rendering all belong to [`er7`], which is this crate's
//! only runtime dependency and has none of its own. It is also not a
//! transport: MLLP, files, and queues are the caller's business.
//!
//! `spec/index.md` in the repository is the normative specification of
//! everything above; where this documentation and that document disagree,
//! that document is right.

// No `unsafe` anywhere in this crate, enforced rather than merely true:
// `forbid` cannot be lifted by an `allow` further down, so this is a
// property a reviewer can rely on without reading the sources. See
// SECURITY.md.
#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

pub mod builder;
pub mod dictionary;
pub mod generic;
pub mod json;
pub mod message;
pub mod structure;
pub mod typed;
pub mod validate;
pub mod version;

pub use builder::Builder;
pub use dictionary::Dictionary;
pub use generic::Node;
pub use message::Message;
pub use typed::{FromHl7, FromHl7Text, FromHl7Value, Raw, ToHl7, ToHl7Text, ToHl7Value};
pub use validate::{Diagnostic, Severity};
pub use version::Version;

/// The `#[derive(FromHl7)]` and `#[derive(ToHl7)]` macros, re-exported so
/// the `hl7-2-derive` crate does not have to be named as a dependency.
/// Requires the `derive` feature.
#[cfg(feature = "derive")]
pub use hl7_2_derive::{FromHl7, ToHl7};

/// The ER7 encoding layer this crate is built on, re-exported so callers
/// can name [`er7::Message`], [`er7::Separators`], [`er7::Path`] and the
/// rest without adding their own dependency.
pub use er7;

use std::fmt;
use std::sync::Arc;

/// What can go wrong.
///
/// Reading is deliberately lenient below the MSH header: unknown segments,
/// unknown data types, and structure mismatches are never errors — they
/// degrade to positional names and a flat reading, and are reported by
/// [`Message::validate`] if the caller wants to know. Only a message with
/// no usable header, a path that is not a path, a dictionary that will not
/// load, and (in struct mode) a value that does not fit its Rust type will
/// fail a call.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Input contained no segments.
    Empty,
    /// The first segment is not MSH, so the message never declared its
    /// delimiters.
    MissingMsh,
    /// The MSH header is malformed: no delimiters, or an unusable set.
    BadMshHeader(String),
    /// A path such as `PID-5.1` could not be read; carries the reason.
    Path(String),
    /// A write named a segment the message does not have. Add it with
    /// [`Message::append_segment`] or build the message with [`Builder`].
    NoSuchSegment {
        /// The segment name that was asked for.
        name: String,
        /// Which occurrence of it.
        occurrence: usize,
    },
    /// A write named something that cannot be written — a whole segment,
    /// or a repeating value without a field.
    UnwritablePath(String),
    /// Struct mode: a non-optional field's path names nothing in the
    /// message.
    MissingField {
        /// The path that was empty.
        path: String,
    },
    /// Struct mode: a value is present but does not fit the Rust type.
    BadValue {
        /// Where the value is.
        path: String,
        /// What was expected there.
        expected: String,
        /// The text that was found.
        found: String,
    },
    /// A dictionary could not be loaded; see [`dictionary::Error`].
    Dictionary(dictionary::Error),
    /// [`Options::strict`] was set and the message did not pass
    /// validation. Carries every [`Severity::Error`] diagnostic.
    Invalid(Vec<Diagnostic>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Empty => write!(f, "input contains no HL7 segments"),
            Error::MissingMsh => write!(f, "message does not start with an MSH segment"),
            Error::BadMshHeader(detail) => write!(f, "malformed MSH header: {detail}"),
            Error::Path(detail) => write!(f, "invalid HL7 path: {detail}"),
            Error::NoSuchSegment { name, occurrence } => {
                write!(
                    f,
                    "message has no {name} segment at occurrence {occurrence}"
                )
            }
            Error::UnwritablePath(detail) => write!(f, "cannot write to this path: {detail}"),
            Error::MissingField { path } => write!(f, "{path}: required value is missing"),
            Error::BadValue {
                path,
                expected,
                found,
            } => write!(f, "{path}: expected {expected}, found {found:?}"),
            Error::Dictionary(error) => write!(f, "invalid dictionary: {error}"),
            Error::Invalid(diagnostics) => {
                write!(f, "message failed validation:")?;
                for diagnostic in diagnostics {
                    write!(f, "\n  {diagnostic}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<er7::Error> for Error {
    fn from(error: er7::Error) -> Error {
        match error {
            er7::Error::Empty => Error::Empty,
            er7::Error::MissingHeader(_) => Error::MissingMsh,
            er7::Error::BadHeader(detail) => Error::BadMshHeader(detail),
            er7::Error::BadPath(detail) => Error::Path(detail),
        }
    }
}

impl From<dictionary::Error> for Error {
    fn from(error: dictionary::Error) -> Error {
        Error::Dictionary(error)
    }
}

/// How to read a message: which release, which dictionary, and whether
/// validation failures are fatal.
///
/// ```
/// let options = hl7_2::Options::new()
///     .with_version(hl7_2::Version::V2_3_1)  // ignore what MSH-12 says
///     .strict();                              // reject what does not conform
/// # let _ = options;
/// ```
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Read every message as this release, whatever MSH-12 declares. Use
    /// it for a sender known to mislabel its version.
    pub version: Option<Version>,
    /// Read through this dictionary instead of the bundled one for the
    /// release — schema mode.
    pub dictionary: Option<Arc<Dictionary>>,
    /// Fail the parse when validation reports a [`Severity::Error`], rather
    /// than returning a message the caller must remember to check.
    pub strict: bool,
}

impl Options {
    /// Default options: release from MSH-12, bundled dictionary, lenient.
    #[must_use]
    pub fn new() -> Options {
        Options::default()
    }

    /// Read as `version` whatever MSH-12 says.
    #[must_use]
    pub fn with_version(mut self, version: Version) -> Options {
        self.version = Some(version);
        self
    }

    /// Read through `dictionary` — schema mode.
    #[must_use]
    pub fn with_dictionary(mut self, dictionary: Arc<Dictionary>) -> Options {
        self.dictionary = Some(dictionary);
        self
    }

    /// Reject a message that does not conform; see [`Options::strict`].
    #[must_use]
    pub fn strict(mut self) -> Options {
        self.strict = true;
        self
    }
}

/// Parse one message, reading the release from MSH-12.
/// # Errors
///
/// [`Error`] when the message has no usable MSH header: no segments,
/// a first segment that is not MSH, or delimiters that cannot be read.
pub fn parse(text: &str) -> Result<Message, Error> {
    Message::parse(text, &Options::default())
}

/// Parse one message under `options`.
/// # Errors
///
/// [`Error`] when the message has no usable MSH header: no segments,
/// a first segment that is not MSH, or delimiters that cannot be read.
pub fn parse_with_options(text: &str, options: &Options) -> Result<Message, Error> {
    Message::parse(text, options)
}

/// Split input that may hold several messages, or an HL7 batch file, into
/// individual messages — one per MSH segment. Batch envelope segments
/// (FHS, BHS, BTS, FTS) are dropped.
pub fn split_messages(text: &str) -> Vec<String> {
    // Normalize first: `er7::split_messages` identifies a segment by its
    // leading run of letters and digits, so a line indented for readability
    // would not be recognized as the MSH that starts a message.
    er7::split_messages(&normalize(text))
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// Tidy input before parsing: drop a byte-order mark, split on either
/// terminator, trim each line, drop blank ones, and rejoin with `\r`.
///
/// `er7` deliberately trims nothing, because it guarantees a message it
/// parses can be written back byte for byte and it cannot know whether a
/// trailing space is data. This crate makes the same round-trip promise for
/// messages it did not modify, but only after this normalization — an
/// indented first line would otherwise be a missing header rather than a
/// message.
pub(crate) fn normalize(text: &str) -> String {
    text.trim_start_matches('\u{feff}')
        .split(['\r', '\n'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_before_parsing() {
        assert_eq!(normalize("MSH|A\r\n\r\n  PID|1  \n"), "MSH|A\rPID|1");
        assert_eq!(normalize("\u{feff}MSH|A"), "MSH|A");
        // Which means an indented message still parses, where the `er7`
        // parser alone would report a missing header.
        assert!(parse("  MSH|^~\\&|APP||||1||ACK|1|P|2.5\r  MSA|AA|1").is_ok());
    }

    #[test]
    fn maps_er7_errors_onto_this_crates_type() {
        assert!(matches!(parse(""), Err(Error::Empty)));
        assert!(matches!(parse("PID|1"), Err(Error::MissingMsh)));
        assert!(matches!(parse("MSH"), Err(Error::BadMshHeader(_))));
    }

    #[test]
    fn strict_mode_turns_diagnostics_into_a_failure() {
        let text = "MSH|^~\\&|A||||20240101||ACK^A01|1|P|2.5"; // no MSA
        assert!(parse(text).is_ok(), "lenient by default");
        let strict = Options::new().strict();
        match parse_with_options(text, &strict) {
            Err(Error::Invalid(diagnostics)) => {
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].kind, validate::Kind::SegmentMissing);
            }
            other => panic!("expected a validation failure, got {other:?}"),
        }
        // Warnings alone do not fail: an unknown structure is this crate's
        // gap, not the sender's error.
        let text = "MSH|^~\\&|A||||20240101||ZZZ^Z01|1|P|2.5";
        assert!(parse_with_options(text, &strict).is_ok());
    }

    #[test]
    fn forcing_a_version_overrides_the_header() {
        let text = "MSH|^~\\&|A||||1||ACK^A01|1|P|2.5\rMSA|AA|1";
        let options = Options::new().with_version(Version::V2_3);
        let message = parse_with_options(text, &options).unwrap();
        assert_eq!(message.version(), Version::V2_3);
        // v2.3's ERR has one field where v2.5's has twelve.
        assert_eq!(message.dictionary().segment_fields("ERR").unwrap().len(), 1);
    }

    #[test]
    fn splits_batches_into_messages() {
        let batch = "FHS|^~\\&|A\rBHS|^~\\&|A\r\
                     MSH|^~\\&|A||||1||ACK|1|P|2.5\rMSA|AA|1\r\
                     MSH|^~\\&|A||||2||ACK|2|P|2.5\rMSA|AA|2\r\
                     BTS|2\rFTS|1";
        let messages = split_messages(batch);
        assert_eq!(messages.len(), 2);
        assert!(messages[1].contains("MSA|AA|2"));
        assert!(messages.iter().all(|text| parse(text).is_ok()));
    }
}
