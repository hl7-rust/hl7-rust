//! Convert HL7 v2.5 messages from the typed JSON representation the
//! sibling `hl7-v2-from-er7-into-json` crate produces back to ER7
//! (pipe-delimited) encoding.
//!
//! This is the inverse of that crate. It names every JSON key after either
//! an HL7 v2.5 data type or a bare position, but in both cases the number
//! after a key's *last* dot is always the 1-based position at that level —
//! field under a segment, component under a field, subcomponent under a
//! component. Reconstruction leans on that one fact rather than an HL7
//! v2.5 data-type dictionary, so this crate carries none; see
//! `spec/index.md` for the exact rules and their limits.
//!
//! ```
//! let json = r#"{
//!   "ORM_O01": {
//!     "MSH": { "MSH.1": "|", "MSH.2": "^~\\&", "MSH.9": {"MSG.1": "ORM", "MSG.2": "O01"} },
//!     "ORM_O01.PATIENT": {
//!       "PID": { "PID.5": { "XPN.1": {"FN.1": "TEST"}, "XPN.2": "FOUAZ" } }
//!     }
//!   }
//! }"#;
//! let er7 = hl7_v2_from_json_into_er7::convert(json).unwrap();
//! assert!(er7.starts_with(r"MSH|^~\&|"));
//! assert!(er7.contains("PID|||||TEST^FOUAZ"));
//! ```

#![warn(missing_docs, clippy::pedantic)]

pub mod json;
pub mod reconstruct;

/// The ER7 encoding layer this crate writes onto, re-exported so callers
/// can name [`er7::Message`], [`er7::Separators`], and
/// [`er7::RenderOptions`] without adding their own dependency.
pub use er7;

use std::fmt;

/// Errors that can occur while turning JSON into an [`er7::Message`].
///
/// As with the forward crate, this is deliberately narrow: below the
/// header, no shape of input is rejected — a key with an unparseable
/// position, an unexpected scalar type, or a segment this crate has never
/// heard of all reconstruct into *something* rather than failing (see
/// `spec/index.md` §5). Only a document with no usable
/// `{"...": {"MSH": ...}}` shape, or that is not well-formed JSON at all,
/// produces an `Err`.
#[derive(Debug)]
pub enum Hl7Error {
    /// The input is not well-formed JSON.
    Json(json::JsonError),
    /// The document isn't shaped like a converted message: not a
    /// single-key object over an object of segments, or that object has no
    /// segment entries at all.
    Empty,
    /// The first segment is not `MSH`, `FHS`, or `BHS`.
    MissingMsh,
    /// The header segment's `.1`/`.2` fields don't declare a usable
    /// delimiter set.
    BadMshHeader(String),
}

impl fmt::Display for Hl7Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hl7Error::Json(e) => write!(f, "malformed JSON: {e}"),
            Hl7Error::Empty => write!(f, "document contains no HL7 segments"),
            Hl7Error::MissingMsh => write!(f, "document does not start with an MSH segment"),
            Hl7Error::BadMshHeader(detail) => write!(f, "malformed MSH header: {detail}"),
        }
    }
}

impl std::error::Error for Hl7Error {}

impl From<json::JsonError> for Hl7Error {
    fn from(error: json::JsonError) -> Hl7Error {
        Hl7Error::Json(error)
    }
}

/// Parse a converted-JSON document into an [`er7::Message`], reconstructing
/// its full ER7 value tree — segments, fields, repetitions, components,
/// and subcomponents.
///
/// Prefer this over [`convert`] when the caller wants to query or edit the
/// message (via `er7`'s own API) rather than just its ER7 text.
/// # Errors
///
/// [`Hl7Error`] when the text is not valid JSON, or when what it contains
/// is not an HL7 message: no segments, or a first segment that is not MSH.
pub fn parse(json_text: &str) -> Result<er7::Message, Hl7Error> {
    let document = json::parse_document(json_text)?;
    reconstruct::reconstruct(&document)
}

/// Convert one JSON document to ER7 text, with default rendering:
/// carriage-return segment terminators, and no trailing terminator.
/// # Errors
///
/// [`Hl7Error`] when the text is not valid JSON, or when what it contains
/// is not an HL7 message: no segments, or a first segment that is not MSH.
pub fn convert(json_text: &str) -> Result<String, Hl7Error> {
    convert_with_options(json_text, er7::RenderOptions::default())
}

/// Convert one JSON document to ER7 text, choosing the segment terminator
/// and whether the last segment gets one too — see [`er7::RenderOptions`].
/// # Errors
///
/// [`Hl7Error`] when the text is not valid JSON, or when what it contains
/// is not an HL7 message: no segments, or a first segment that is not MSH.
pub fn convert_with_options(
    json_text: &str,
    options: er7::RenderOptions,
) -> Result<String, Hl7Error> {
    Ok(parse(json_text)?.to_er7_with(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_json_errors_onto_this_crates_type() {
        assert!(matches!(convert("not json"), Err(Hl7Error::Json(_))));
    }

    #[test]
    fn maps_missing_header_errors() {
        assert!(matches!(
            convert(r#"{"X": {"PID": {"PID.1": "1"}}}"#),
            Err(Hl7Error::MissingMsh)
        ));
        assert!(matches!(convert(r#"{"X": {}}"#), Err(Hl7Error::Empty)));
        assert!(matches!(convert("{}"), Err(Hl7Error::Empty)));
    }
}
