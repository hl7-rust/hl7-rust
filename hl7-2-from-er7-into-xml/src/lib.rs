//! Convert HL7® v2.5 messages from ER7 (pipe-delimited) encoding to the
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
//! let xml = hl7_2_from_er7_into_xml::convert(er7).unwrap();
//! assert!(xml.contains("<ORM_O01 xmlns=\"urn:hl7-org:v2xml\">"));
//! assert!(xml.contains("<XPN.1>"));
//! ```
//!
//! # Trademarks
//!
//! HL7®, and FHIR® are the registered trademarks of Health Level Seven
//! International and their use of these trademarks does not constitute an
//! endorsement by HL7.

// No `unsafe` anywhere in this crate, enforced rather than merely true:
// `forbid` cannot be lifted by an `allow` further down, so this is a
// property a reviewer can rely on without reading the sources. See
// SECURITY.md.
#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
// XML literals keep their `r#"..."#` delimiters even where no `"` currently
// forces them: these are documents, and adding a quoted attribute to one
// should not also mean changing its delimiter.
#![allow(clippy::needless_raw_string_hashes)]

pub mod structure;
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
    /// Treat the dictionary as a schema that describes the document's exact
    /// shape, rather than as a table of what things mean.
    ///
    /// Off, the message decides which elements appear: every field it
    /// carries is written, every repetition becomes its own element, and a
    /// field it leaves empty is absent. That is the right reading for the
    /// bundled releases, whose tables say what a field *is* and nothing
    /// about how often it may appear.
    ///
    /// On, the dictionary decides. A field it marks `required` is written
    /// even when the message leaves it empty, so the position stays visible;
    /// a field it does not mark `repeats` keeps its repetition separator as
    /// ordinary text instead of becoming several elements; and no element is
    /// written for a field the dictionary does not declare. Use it with a
    /// dictionary generated from XML Schema — one produced by
    /// `hl7-2-from-xsd-into-json-dictionary` — where the answer to all
    /// three questions came from the schema the output is validated against.
    /// See `spec/index.md` §4.
    pub schema_shape: bool,
}

/// Convert one ER7 message to a v2.xml document with default options.
/// # Errors
///
/// [`Hl7Error`] when the input has no usable MSH header: no segments at
/// all, a first segment that is not MSH, or a header whose delimiters
/// cannot be read. Everything below the header degrades rather than
/// failing.
pub fn convert(er7_text: &str) -> Result<String, Hl7Error> {
    convert_with_options(er7_text, Options::default())
}

/// Convert one ER7 message to a v2.xml document, using the bundled HL7 v2.5
/// dictionary.
///
/// v2.5 is used whatever MSH-12 says, which is what this crate has always
/// done; pass a dictionary to [`convert_with_dictionary`] to convert against
/// another release or a vendor dialect.
/// # Errors
///
/// [`Hl7Error`] when the input has no usable MSH header: no segments at
/// all, a first segment that is not MSH, or a header whose delimiters
/// cannot be read. Everything below the header degrades rather than
/// failing.
pub fn convert_with_options(er7_text: &str, options: Options) -> Result<String, Hl7Error> {
    convert_with_dictionary(er7_text, &hl7_2::Version::V2_5.dictionary(), options)
}

/// Convert one ER7 message to a v2.xml document against a given dictionary.
///
/// The dictionary supplies everything this crate used to hard-code: which
/// data type each field carries, what a composite type is made of, and how
/// the message's segments group. A dictionary built from a vendor's own XML
/// Schema therefore produces that vendor's document shape rather than the
/// standard's — which is the point, since the output is usually validated
/// against those same schemas.
///
/// ```
/// let dictionary = hl7_2::Version::V2_5.dictionary();
/// let xml = hl7_2_from_er7_into_xml::convert_with_dictionary(
///     "MSH|^~\\&|APP||||1||ACK|1|P|2.5\rMSA|AA|1",
///     &dictionary,
///     Default::default(),
/// ).unwrap();
/// assert!(xml.contains("<ACK xmlns=\"urn:hl7-org:v2xml\">"));
/// ```
/// # Errors
///
/// [`Hl7Error`] when the input has no usable MSH header: no segments at
/// all, a first segment that is not MSH, or a header whose delimiters
/// cannot be read. Everything below the header degrades rather than
/// failing.
pub fn convert_with_dictionary(
    er7_text: &str,
    dictionary: &hl7_2::Dictionary,
    options: Options,
) -> Result<String, Hl7Error> {
    let message = er7::parse(&normalize(er7_text))?;
    let root_name = root_name(&message, dictionary);
    let separators = &message.separators;
    let seg_nodes: Vec<(String, xml::Node)> = message
        .segments
        .iter()
        .map(|s| {
            (
                s.name.clone(),
                xml::segment_to_node(s, separators, dictionary, options.schema_shape),
            )
        })
        .collect();
    let grouped = if options.flat {
        None
    } else {
        dictionary
            .structure(&root_name)
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
/// MSH-9.3 when present, otherwise the structure the dictionary says carries
/// this message code and trigger event.
///
/// Which trigger events share a structure is dictionary knowledge — an A04
/// admit and an A08 update are both carried by `ADT_A01` — so it comes from
/// the `"aliases"` section rather than from a match arm here.
fn root_name(message: &er7::Message, dictionary: &hl7_2::Dictionary) -> String {
    if let Some(structure_id) = message.message_structure() {
        return structure_id;
    }
    dictionary.structure_id(
        &message.message_code().unwrap_or_default(),
        &message.trigger_event().unwrap_or_default(),
    )
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
