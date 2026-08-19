//! The dictionary document this crate writes, and how it is written.
//!
//! The shape is `hl7-v2`'s dictionary format (its `spec/index.md` §3):
//! composite data types, segment field lists, trigger-event aliases, and
//! message structures. This module models exactly that and serializes it;
//! it knows nothing about XML Schema, which is `crate::schema`'s job.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// One field of a segment: its data type, and what the schema said about
/// how often it may appear.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Field {
    /// The field's data type name, or empty when the schema left the
    /// position unstated.
    pub data_type: String,
    /// `minOccurs >= 1`.
    pub required: bool,
    /// `maxOccurs` is `unbounded` or greater than one.
    pub repeats: bool,
}

/// One entry in a message structure: a segment, or a named group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A segment reference.
    Segment {
        /// The segment name, e.g. `PID`.
        name: String,
        /// The structure requires it.
        required: bool,
        /// It may appear more than once here.
        repeats: bool,
    },
    /// A named group of items.
    Group {
        /// The group name without its structure prefix, e.g. `PATIENT`.
        name: String,
        /// The structure requires it.
        required: bool,
        /// It may appear more than once here.
        repeats: bool,
        /// What the group contains, in order.
        items: Vec<Item>,
    },
}

/// A whole dictionary document, ready to write.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    /// The HL7 release the schemas describe, e.g. `2.5.1`.
    pub version: Option<String>,
    /// A sentence saying where this document came from.
    pub description: Option<String>,
    /// A bundled release to layer this document over.
    pub inherits: Option<String>,
    /// Composite data types mapped to their component data types.
    pub types: BTreeMap<String, Vec<String>>,
    /// Segments mapped to their field lists.
    pub segments: BTreeMap<String, Vec<Field>>,
    /// `CODE_TRIGGER` mapped to the structure that carries it.
    pub aliases: BTreeMap<String, String>,
    /// Message structure IDs mapped to their grammars.
    pub structures: BTreeMap<String, Vec<Item>>,
}

impl Document {
    /// Write the document as JSON, indented two spaces, with a trailing
    /// newline — the shape the bundled dictionaries are checked in as, so a
    /// generated file and a hand-written one diff against each other.
    ///
    /// A field is written as a bare data type name when the schema said
    /// nothing beyond the type, which is most of them, and as an object
    /// when it did: `{"type": "XTN", "repeats": true}`.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n");
        let mut first = true;
        if let Some(version) = &self.version {
            member(&mut out, &mut first, 1, "version", &quote(version));
        }
        if let Some(description) = &self.description {
            member(&mut out, &mut first, 1, "description", &quote(description));
        }
        if let Some(inherits) = &self.inherits {
            member(&mut out, &mut first, 1, "inherits", &quote(inherits));
        }
        if !self.types.is_empty() {
            member(&mut out, &mut first, 1, "types", &self.types_json());
        }
        if !self.segments.is_empty() {
            member(&mut out, &mut first, 1, "segments", &self.segments_json());
        }
        if !self.aliases.is_empty() {
            member(&mut out, &mut first, 1, "aliases", &self.aliases_json());
        }
        if !self.structures.is_empty() {
            member(
                &mut out,
                &mut first,
                1,
                "structures",
                &self.structures_json(),
            );
        }
        out.push('\n');
        out.push_str("}\n");
        out
    }

    fn types_json(&self) -> String {
        object(
            2,
            self.types.iter().map(|(name, components)| {
                let items: Vec<String> = components.iter().map(|c| quote(c)).collect();
                (name.clone(), array(3, &items))
            }),
        )
    }

    fn segments_json(&self) -> String {
        object(
            2,
            self.segments.iter().map(|(name, fields)| {
                let items: Vec<String> = fields.iter().map(field_json).collect();
                (name.clone(), array(3, &items))
            }),
        )
    }

    fn aliases_json(&self) -> String {
        object(
            2,
            self.aliases
                .iter()
                .map(|(from, to)| (from.clone(), quote(to))),
        )
    }

    fn structures_json(&self) -> String {
        object(
            2,
            self.structures
                .iter()
                .map(|(id, items)| (id.clone(), items_json(items, 3))),
        )
    }
}

fn field_json(field: &Field) -> String {
    if !field.required && !field.repeats {
        return quote(&field.data_type);
    }
    let mut parts = vec![format!("\"type\": {}", quote(&field.data_type))];
    if field.required {
        parts.push("\"required\": true".to_string());
    }
    if field.repeats {
        parts.push("\"repeats\": true".to_string());
    }
    format!("{{{}}}", parts.join(", "))
}

fn items_json(items: &[Item], depth: usize) -> String {
    let rendered: Vec<String> = items
        .iter()
        .map(|item| match item {
            Item::Segment {
                name,
                required,
                repeats,
            } => format!(
                "{{\"segment\": {}, \"required\": {required}, \"repeats\": {repeats}}}",
                quote(name)
            ),
            Item::Group {
                name,
                required,
                repeats,
                items,
            } => {
                let pad = "  ".repeat(depth + 1);
                format!(
                    "{{\n{pad}\"group\": {}, \"required\": {required}, \"repeats\": {repeats},\n{pad}\"items\": {}\n{}}}",
                    quote(name),
                    items_json(items, depth + 1),
                    "  ".repeat(depth)
                )
            }
        })
        .collect();
    array(depth, &rendered)
}

fn member(out: &mut String, first: &mut bool, depth: usize, key: &str, value: &str) {
    if !*first {
        out.push_str(",\n");
    }
    *first = false;
    let _ = write!(out, "{}{}: {value}", "  ".repeat(depth), quote(key));
}

fn object(depth: usize, entries: impl Iterator<Item = (String, String)>) -> String {
    let pad = "  ".repeat(depth);
    let rendered: Vec<String> = entries
        .map(|(key, value)| format!("{pad}{}: {value}", quote(&key)))
        .collect();
    if rendered.is_empty() {
        return "{}".to_string();
    }
    format!("{{\n{}\n{}}}", rendered.join(",\n"), "  ".repeat(depth - 1))
}

fn array(depth: usize, items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let pad = "  ".repeat(depth);
    format!(
        "[\n{pad}{}\n{}]",
        items.join(&format!(",\n{pad}")),
        "  ".repeat(depth - 1)
    )
}

/// A JSON string literal. Control characters are escaped as `\uXXXX`; the
/// rest is written through, so a description keeps whatever it says.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_bare_name_when_the_schema_said_nothing_more() {
        assert_eq!(
            field_json(&Field {
                data_type: "ST".into(),
                required: false,
                repeats: false
            }),
            "\"ST\""
        );
        assert_eq!(
            field_json(&Field {
                data_type: "XTN".into(),
                required: false,
                repeats: true
            }),
            "{\"type\": \"XTN\", \"repeats\": true}"
        );
        assert_eq!(
            field_json(&Field {
                data_type: "CX".into(),
                required: true,
                repeats: true
            }),
            "{\"type\": \"CX\", \"required\": true, \"repeats\": true}"
        );
    }

    #[test]
    fn escapes_what_json_requires() {
        assert_eq!(quote("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(quote("a\u{1}b"), "\"a\\u0001b\"");
    }

    #[test]
    fn an_empty_document_is_still_valid_json() {
        assert_eq!(Document::default().to_json(), "{\n\n}\n");
    }
}
