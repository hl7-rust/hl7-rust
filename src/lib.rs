//! Convert HL7 v2.5 messages from ER7 (pipe-delimited) encoding to the
//! HL7 v2.xml XML representation (`urn:hl7-org:v2xml`).
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

pub mod er7;
pub mod structure;
pub mod types;
pub mod xml;

use std::fmt;

/// Errors that can occur while turning ER7 text into a [`Message`](er7::Message)
/// or, in turn, into v2.xml.
///
/// Parsing is deliberately lenient below the MSH header: unknown segments,
/// unknown data types, and structure mismatches never produce an error, they
/// degrade to generic names or a flat rendering (see the crate docs and
/// `spec/index.md`). Only a message that has no usable MSH header fails.
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
    let message = er7::parse_message(er7_text)?;
    let root_name = root_name(&message);
    let seg_nodes: Vec<(String, xml::Node)> = message
        .segments
        .iter()
        .map(|s| (s.id.clone(), xml::segment_to_node(s)))
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

/// Split input that may hold several messages (or an HL7 batch file) into
/// individual ER7 messages, one per MSH segment. Batch envelope segments
/// (FHS, BHS, BTS, FTS) are dropped.
pub fn split_messages(text: &str) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    for line in text
        .trim_start_matches('\u{feff}')
        .split(['\r', '\n'])
        .map(str::trim)
        .filter(|l| !l.is_empty())
    {
        let id: String = line.chars().take(3).collect();
        let next_is_alnum = line
            .chars()
            .nth(3)
            .is_some_and(|c| c.is_ascii_alphanumeric());
        if matches!(id.as_str(), "FHS" | "BHS" | "BTS" | "FTS") && !next_is_alnum {
            continue;
        }
        if line.starts_with("MSH") || messages.is_empty() {
            messages.push(String::new());
        }
        let current = messages.last_mut().expect("just ensured non-empty");
        if !current.is_empty() {
            current.push('\r');
        }
        current.push_str(line);
    }
    messages
}

/// Derive the message structure ID (and root element name) from MSH-9:
/// MSH-9.3 when present, otherwise `MSG.1_MSG.2` with the trigger-event
/// aliases resolved for the structures this crate knows about.
fn root_name(message: &er7::Message) -> String {
    let msh = &message.segments[0];
    if let Some(structure_id) = msh.value(9, 3) {
        return structure_id.to_string();
    }
    let code = msh.value(9, 1).unwrap_or("");
    let trigger = msh.value(9, 2).unwrap_or("");
    match (code, trigger) {
        ("", _) => "HL7Message".to_string(),
        ("ACK", _) => "ACK".to_string(),
        ("ADT", "A01" | "A04" | "A08" | "A13") => "ADT_A01".to_string(),
        (code, "") => code.to_string(),
        (code, trigger) => format!("{code}_{trigger}"),
    }
}
