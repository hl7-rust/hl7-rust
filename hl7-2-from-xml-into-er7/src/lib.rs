//! Convert HL7® v2.5 messages from the HL7 v2.xml XML representation
//! (`urn:hl7-org:v2xml`) back to ER7 (pipe-delimited) encoding.
//!
//! This is the inverse of the sibling
//! [`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml)
//! crate. That crate names every XML element after either an HL7 v2.5 data
//! type or a bare position, but in both cases the number after an element
//! name's *last* dot is always the 1-based position at that level — field
//! under a segment, component under a field, subcomponent under a
//! component. Reconstruction leans on that one fact rather than an HL7
//! v2.5 data-type dictionary, so this crate carries none; see
//! `spec/index.md` for the exact rules and their limits.
//!
//! ```
//! let xml = r#"<ORM_O01 xmlns="urn:hl7-org:v2xml">
//!   <MSH>
//!     <MSH.1>|</MSH.1>
//!     <MSH.2>^~\&amp;</MSH.2>
//!     <MSH.9><MSG.1>ORM</MSG.1><MSG.2>O01</MSG.2></MSH.9>
//!   </MSH>
//!   <ORM_O01.PATIENT>
//!     <PID><PID.5><XPN.1><FN.1>TEST</FN.1></XPN.1><XPN.2>FOUAZ</XPN.2></PID.5></PID>
//!   </ORM_O01.PATIENT>
//! </ORM_O01>"#;
//! let er7 = hl7_2_from_xml_into_er7::convert(xml).unwrap();
//! assert!(er7.starts_with(r"MSH|^~\&|"));
//! assert!(er7.contains("PID|||||TEST^FOUAZ"));
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

pub mod reconstruct;
/// The XML reader this crate is built on, re-exported so callers can name
/// [`xml::Element`] without adding their own dependency.
///
/// Until 0.5.0 this was a module inside this crate, with a `Node` type of
/// its own. It is now the standalone `hl7-2-xml-lite-helper` crate, which
/// reads the same subset; the type is [`xml::Element`], its children are
/// `children` rather than `kids`, and its text is a `String` that is empty
/// rather than an `Option` that is `None`.
pub use hl7_2_xml_lite_helper as xml;

/// The ER7 encoding layer this crate writes onto, re-exported so callers
/// can name [`er7::Message`], [`er7::Separators`], and
/// [`er7::RenderOptions`] without adding their own dependency.
pub use er7;

use std::fmt;

/// Errors that can occur while turning v2.xml into an [`er7::Message`].
///
/// As with the forward crate, this is deliberately narrow: below the
/// header, no shape of input is rejected — an element with an unparseable
/// position, an unexpected type, or a segment this crate has never heard
/// of all reconstruct into *something* rather than failing (see
/// `spec/index.md` §5). Only a document with no usable `MSH`/`FHS`/`BHS`
/// header, or that is not well-formed XML at all, produces an `Err`.
#[derive(Debug)]
pub enum Hl7Error {
    /// The input is not well-formed XML.
    Xml(xml::Error),
    /// The document has no segments at all (an empty or absent root
    /// element).
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
            Hl7Error::Xml(e) => write!(f, "malformed XML: {e}"),
            Hl7Error::Empty => write!(f, "document contains no HL7 segments"),
            Hl7Error::MissingMsh => write!(f, "document does not start with an MSH segment"),
            Hl7Error::BadMshHeader(detail) => write!(f, "malformed MSH header: {detail}"),
        }
    }
}

impl std::error::Error for Hl7Error {}

impl From<xml::Error> for Hl7Error {
    fn from(error: xml::Error) -> Hl7Error {
        Hl7Error::Xml(error)
    }
}

/// Parse a v2.xml document into an [`er7::Message`], reconstructing its
/// full ER7 value tree — segments, fields, repetitions, components, and
/// subcomponents.
///
/// Prefer this over [`convert`] when the caller wants to query or edit the
/// message (via `er7`'s own API) rather than just its ER7 text.
/// # Errors
///
/// [`Hl7Error`] when the document cannot be read as XML, or when what it
/// contains is not an HL7 message: no segments, or a first segment that is
/// not MSH.
pub fn parse(xml_text: &str) -> Result<er7::Message, Hl7Error> {
    let root = xml::parse(xml_text)?;
    reconstruct::reconstruct(&root)
}

/// Convert one v2.xml document to ER7 text, with default rendering:
/// carriage-return segment terminators, and no trailing terminator.
/// # Errors
///
/// [`Hl7Error`] when the document cannot be read as XML, or when what it
/// contains is not an HL7 message: no segments, or a first segment that is
/// not MSH.
pub fn convert(xml_text: &str) -> Result<String, Hl7Error> {
    convert_with_options(xml_text, er7::RenderOptions::default())
}

/// Convert one v2.xml document to ER7 text, choosing the segment
/// terminator and whether the last segment gets one too — see
/// [`er7::RenderOptions`].
/// # Errors
///
/// [`Hl7Error`] when the document cannot be read as XML, or when what it
/// contains is not an HL7 message: no segments, or a first segment that is
/// not MSH.
pub fn convert_with_options(
    xml_text: &str,
    options: er7::RenderOptions,
) -> Result<String, Hl7Error> {
    Ok(parse(xml_text)?.to_er7_with(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_xml_errors_onto_this_crates_type() {
        assert!(matches!(convert("not xml"), Err(Hl7Error::Xml(_))));
    }

    #[test]
    fn maps_missing_header_errors() {
        assert!(matches!(
            convert("<X><PID><PID.1>1</PID.1></PID></X>"),
            Err(Hl7Error::MissingMsh)
        ));
        assert!(matches!(convert("<X></X>"), Err(Hl7Error::Empty)));
    }
}
