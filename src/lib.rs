//! Convert HL7 v2.5 messages from ER7 (pipe-delimited) encoding to the
//! HL7 v2.xml XML representation (`urn:hl7-org:v2xml`).
//!
//! The ER7 encoding itself — parsing, delimiters, escape sequences — comes
//! from the [`er7`] crate. This crate adds the layer above it: the HL7 v2.5
//! data-type tables that name XML elements, the message-structure grammars
//! that group segments, and the XML renderer. See `spec/index.md` for the
//! exact conversion rules (source of truth).
//!
//! ```
//! let er7 = "MSH|^~\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\r\
//!            PID|1||241900||MEDIANO^FOUAZ\r\
//!            ORC|NW|ORD1";
//! let xml = hl7_2_5_to_xml::convert(er7).unwrap();
//! assert!(xml.contains("<ORM_O01 xmlns=\"urn:hl7-org:v2xml\">"));
//! assert!(xml.contains("<XPN.1>"));
//! ```

#![warn(missing_docs)]

pub mod structure;
pub mod types;
pub mod xml;

/// The ER7 encoding layer this crate is built on, re-exported so callers can
/// name [`er7::Message`], [`er7::Separators`], and the rest without adding
/// their own dependency.
///
/// Until version 0.2.0 this was a module inside this crate. It is now the
/// standalone `er7` crate, which owns the encoding and guarantees a
/// byte-for-byte round trip; the type names changed slightly in the move
/// (`Segment::id` is now `Segment::name`, `Repeat` is now
/// [`er7::Repetition`], and subcomponents are [`er7::Subcomponent`] values
/// that decode on demand rather than pre-decoded `String`s).
pub use er7;

use std::fmt;

/// Errors that can occur while turning ER7 text into a [`er7::Message`]
/// or, in turn, into v2.xml.
///
/// Parsing is deliberately lenient below the MSH header: unknown segments,
/// unknown data types, and structure mismatches never produce an error, they
/// degrade to generic names or a flat rendering (see the crate docs and
/// `spec/index.md`). Only a message that has no usable MSH header fails.
///
/// This is a distinct type from [`er7::Error`] rather than a re-export,
/// because the `er7` crate can also report a malformed HL7 path — something
/// this crate never asks it for. Converting is automatic via `?`.
#[derive(Debug)]
pub enum Hl7Error {
    /// Input contained no segments.
    Empty,
    /// The first segment is not MSH.
    MissingMsh,
    /// The MSH header line is malformed.
    BadMshHeader(String),
}

impl fmt::Display for Hl7Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hl7Error::Empty => write!(f, "input contains no HL7 segments"),
            Hl7Error::MissingMsh => write!(f, "message does not start with an MSH segment"),
            Hl7Error::BadMshHeader(detail) => write!(f, "malformed MSH header: {detail}"),
        }
    }
}

impl std::error::Error for Hl7Error {}

impl From<er7::Error> for Hl7Error {
    /// Map an `er7` parse failure onto this crate's narrower error type.
    ///
    /// `er7::Error::BadPath` cannot arise here: it comes from the path-query
    /// API, and this crate reads the message through its own accessors. It
    /// is mapped to [`Hl7Error::BadMshHeader`] so the detail survives rather
    /// than being swallowed by a panic that could never fire.
    fn from(error: er7::Error) -> Hl7Error {
        match error {
            er7::Error::Empty => Hl7Error::Empty,
            er7::Error::MissingHeader(_) => Hl7Error::MissingMsh,
            er7::Error::BadHeader(detail) | er7::Error::BadPath(detail) => {
                Hl7Error::BadMshHeader(detail)
            }
        }
    }
}

/// Options controlling how a message is converted; see [`convert_with_options`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Always emit segments flat under the root element, never grouped into
    /// message-structure groups such as `ORM_O01.PATIENT`.
    pub flat: bool,
}

/// Convert one ER7 message to a v2.xml document with default options.
pub fn convert(er7_text: &str) -> Result<String, Hl7Error> {
    convert_with_options(er7_text, Options::default())
}

/// Convert one ER7 message to a v2.xml document.
pub fn convert_with_options(er7_text: &str, options: Options) -> Result<String, Hl7Error> {
    let message = er7::parse(&normalize(er7_text))?;
    let root_name = root_name(&message);
    let separators = &message.separators;
    let seg_nodes: Vec<(String, xml::Node)> = message
        .segments
        .iter()
        .map(|s| (s.name.clone(), xml::segment_to_node(s, separators)))
        .collect();
    let grouped = if options.flat {
        None
    } else {
        structure::structure_for(&root_name)
            .and_then(|items| structure::group_segments(&root_name, items, &seg_nodes))
    };
    let mut root = xml::Node::group(xml::xml_name(&root_name));
    root.kids = grouped.unwrap_or_else(|| seg_nodes.into_iter().map(|(_, n)| n).collect());
    Ok(xml::render_document(&root))
}

/// Tidy input into the shape `spec/index.md` §2.1 describes, before handing
/// it to the `er7` parser: split on either terminator, trim each line, drop
/// blank ones, and rejoin with `\r`.
///
/// `er7` deliberately trims nothing, because it guarantees that a message it
/// parses can be written back byte for byte and it cannot know whether a
/// trailing space is data (`er7` spec §4.1). This crate makes no such
/// promise — it renders XML, where stray whitespace around a segment is
/// noise, and where an indented first line would otherwise turn a readable
/// message into a `MissingMsh` error. So the trimming this crate has always
/// documented happens here instead.
fn normalize(text: &str) -> String {
    text.trim_start_matches('\u{feff}')
        .split(['\r', '\n'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\r")
}

/// Split input that may hold several messages (or an HL7 batch file) into
/// individual ER7 messages, one per MSH segment. Batch envelope segments
/// (FHS, BHS, BTS, FTS) are dropped.
pub fn split_messages(text: &str) -> Vec<String> {
    // Normalize first: `er7::split_messages` identifies a segment by its
    // leading run of letters and digits, so a line indented for readability
    // would not be recognized as the `MSH` that starts a message.
    let normalized = normalize(text);
    er7::split_messages(&normalized)
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// Derive the message structure ID (and root element name) from MSH-9:
/// MSH-9.3 when present, otherwise `MSG.1_MSG.2` with the trigger-event
/// aliases resolved for the structures this crate knows about.
fn root_name(message: &er7::Message) -> String {
    if let Some(structure_id) = message.message_structure() {
        return structure_id;
    }
    let code = message.message_code().unwrap_or_default();
    let trigger = message.trigger_event().unwrap_or_default();
    match (code.as_str(), trigger.as_str()) {
        ("", _) => "HL7Message".to_string(),
        ("ACK", _) => "ACK".to_string(),
        ("ADT", "A01" | "A04" | "A08" | "A13") => "ADT_A01".to_string(),
        (code, "") => code.to_string(),
        (code, trigger) => format!("{code}_{trigger}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_before_parsing() {
        // The crate's own §2.1: any terminator, trimmed lines, no blanks.
        assert_eq!(normalize("MSH|A\r\n\r\n  PID|1  \n"), "MSH|A\rPID|1");
        assert_eq!(normalize("\u{feff}MSH|A"), "MSH|A");
        // Which means an indented message still converts, where the `er7`
        // parser alone would report a missing header.
        assert!(convert("  MSH|^~\\&|APP||||1||ACK|1|P|2.5\r  MSA|AA|1").is_ok());
    }

    #[test]
    fn maps_er7_errors_onto_this_crates_type() {
        assert!(matches!(convert(""), Err(Hl7Error::Empty)));
        assert!(matches!(convert("PID|1"), Err(Hl7Error::MissingMsh)));
        match convert("MSH") {
            Err(Hl7Error::BadMshHeader(detail)) => assert!(!detail.is_empty()),
            other => panic!("expected a bad-header error, got {other:?}"),
        }
    }
}
