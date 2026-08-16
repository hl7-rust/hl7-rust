//! Rebuilding the ER7 value tree from the parsed JSON [`Value`] tree.
//!
//! This is the inverse of the sibling `hl7-v2-from-er7-into-json` crate's
//! `src/json.rs`. That crate names every key after either an HL7 v2.5 data
//! type (`"XPN.1"`) or a bare position (`"PID.5.1"`) — but in both cases,
//! the number after the key's *last* dot is always the 1-based position at
//! that level: field number under a segment, component number under a
//! field, subcomponent number under a component. That is the one fact this
//! module leans on, and it means reconstruction needs no HL7 v2.5
//! data-type dictionary at all — see `spec/index.md` §3 for the full rule
//! set this implements.
//!
//! The one shape JSON adds beyond XML: a key that occurred more than once
//! in the source message is a JSON array here rather than a repeated
//! sibling element (the forward crate's own spec §4.3), so every level
//! below reads "one or more occurrences of this key" through the private
//! `occurrences` helper instead of grouping same-named XML siblings.

use crate::Hl7Error;
use crate::json::Value;
use er7::escape::{Escape, escape, escapes};
use er7::message::NULL;
use er7::{Component, Field, Message, Repetition, Segment, Separators, Subcomponent};
use std::collections::BTreeMap;

/// Reconstruct a full [`Message`] from a parsed JSON document: a
/// single-key object whose value holds the message's top-level segments
/// and groups (the forward crate's spec §4.7).
pub fn reconstruct(document: &Value) -> Result<Message, Hl7Error> {
    let Value::Object(root_entries) = document else {
        return Err(Hl7Error::Empty);
    };
    let [(_, body)] = root_entries.as_slice() else {
        return Err(Hl7Error::Empty);
    };
    let Value::Object(body_entries) = body else {
        return Err(Hl7Error::Empty);
    };
    let mut flat = Vec::new();
    flatten_segments(body_entries, &mut flat);
    let (header_name, header_value) = flat.first().ok_or(Hl7Error::Empty)?;
    if !is_header_name(header_name) {
        return Err(Hl7Error::MissingMsh);
    }
    let separators = header_separators(header_name, header_value)?;
    let segments = flat
        .into_iter()
        .map(|(name, value)| build_segment(name, value, &separators))
        .collect();
    Ok(Message {
        separators,
        segments,
    })
}

/// Every occurrence a key's value represents: the items of a JSON array, or
/// the value itself as the sole occurrence (the forward crate's own spec
/// §4.3 — a key that occurred exactly once is never wrapped in a
/// one-element array).
fn occurrences(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(items) => items.iter().collect(),
        other => vec![other],
    }
}

/// Walk `entries`, collecting the segment occurrences in document order and
/// descending into (but not keeping) group entries.
///
/// An entry is a group, not a segment, exactly when its key contains a
/// `.` — real segment IDs never do, while every group key the sibling
/// crate emits is `{message-structure}.{group}` regardless of how deeply it
/// is nested (`hl7-v2-from-er7-into-json` spec §4.7). This is why
/// reconstruction needs no message-structure grammar either: it flattens
/// every group away, which is exactly what that crate's own `--flat`
/// option produces.
fn flatten_segments<'a>(entries: &'a [(String, Value)], out: &mut Vec<(&'a str, &'a Value)>) {
    for (key, value) in entries {
        for occurrence in occurrences(value) {
            if key.contains('.') {
                if let Value::Object(kids) = occurrence {
                    flatten_segments(kids, out);
                }
                // A group occurrence that isn't an object is malformed
                // input; skipping it rather than failing matches this
                // crate's fallback-first philosophy (spec/index.md §5).
            } else {
                out.push((key, occurrence));
            }
        }
    }
}

/// True for the three segment names that declare a message's delimiters —
/// mirrors `er7`'s own (private) rule; see that crate's `spec/index.md`
/// §3.2.
fn is_header_name(name: &str) -> bool {
    matches!(name, "MSH" | "FHS" | "BHS")
}

/// Recover the delimiter set from a header segment's own `.1`/`.2` fields
/// by reassembling a synthetic header line and handing it to
/// [`er7::Separators::from_header`] — the same parsing `er7` applies to a
/// real ER7 header, reused rather than duplicated.
fn header_separators(name: &str, segment: &Value) -> Result<Separators, Hl7Error> {
    let field_separator = field_text(name, segment, 1).ok_or_else(|| {
        Hl7Error::BadMshHeader(format!(
            "{name} has no {name}.1 field (the field separator)"
        ))
    })?;
    let encoding = field_text(name, segment, 2).unwrap_or_default();
    // `Separators::from_header` reads the field separator as the character
    // right after the segment name, then the encoding characters up to the
    // next field separator — so appending the field separator again gives
    // it the terminator it expects, exactly as a real "MSH|^~\&|..." line
    // would.
    let synthetic = format!("{name}{field_separator}{encoding}{field_separator}");
    Separators::from_header(&synthetic).map_err(|e| Hl7Error::BadMshHeader(e.to_string()))
}

/// The decoded text of `segment`'s `.n` field, if present, a plain string,
/// and non-null.
fn field_text(name: &str, segment: &Value, n: usize) -> Option<String> {
    let Value::Object(entries) = segment else {
        return None;
    };
    let target = format!("{name}.{n}");
    let value = entries.iter().find(|(k, _)| *k == target).map(|(_, v)| v)?;
    match value {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn build_segment(name: &str, value: &Value, separators: &Separators) -> Segment {
    let entries: &[(String, Value)] = match value {
        Value::Object(entries) => entries,
        _ => &[],
    };
    let mut fields = build_fields(entries, separators);
    if is_header_name(name) {
        // Fields 1 and 2 of a header are the delimiters themselves, stored
        // literally rather than escaped (`er7` spec §3.4) — the generic
        // build above cannot know that, so its guesses for these two slots
        // are replaced here.
        while fields.len() < 2 {
            fields.push(Field::default());
        }
        fields[0] = literal_field(field_text(name, value, 1).unwrap_or_default());
        fields[1] = literal_field(field_text(name, value, 2).unwrap_or_default());
    }
    Segment {
        name: name.to_string(),
        fields,
    }
}

/// A field holding one exact, unescaped string — used only for a header's
/// delimiter fields.
fn literal_field(raw: impl Into<String>) -> Field {
    Field {
        repetitions: vec![Repetition {
            components: vec![Component {
                subcomponents: vec![Subcomponent::new(raw)],
            }],
        }],
    }
}

/// Every non-null field, 1-based, built from a segment object's entries:
/// key `{seg}.n` names field `n`, and its occurrences (`occurrences`, above)
/// are that field's repetitions, in order. A field number this segment
/// never mentions is absent (`Field::default()`), not empty — matching how
/// the forward crate omits a key entirely rather than emitting `null` for
/// an absent field.
fn build_fields(entries: &[(String, Value)], separators: &Separators) -> Vec<Field> {
    let indexed = index_entries(entries);
    pad_fields(indexed, separators)
}

fn pad_fields(indexed: BTreeMap<usize, &Value>, separators: &Separators) -> Vec<Field> {
    let len = indexed.keys().max().copied().unwrap_or(0);
    (1..=len)
        .map(|n| match indexed.get(&n) {
            Some(value) => Field {
                repetitions: occurrences(value)
                    .into_iter()
                    .map(|occ| build_repetition(occ, separators))
                    .collect(),
            },
            None => Field::default(),
        })
        .collect()
}

/// Map each entry's key to the position encoded in its trailing `.N`,
/// keeping only the first entry for a given position — a duplicate key at
/// the same nominal position is malformed input the forward crate never
/// produces, and this crate degrades gracefully rather than failing on it
/// (`spec/index.md` §5).
fn index_entries(entries: &[(String, Value)]) -> BTreeMap<usize, &Value> {
    let mut map = BTreeMap::new();
    let mut next = 1usize;
    for (key, value) in entries {
        let index = trailing_index(key).filter(|&i| i >= 1).unwrap_or(next);
        next = index + 1;
        map.entry(index).or_insert(value);
    }
    map
}

/// One field repetition, or one component: an object recurses one level
/// down (components under a repetition, subcomponents under a component),
/// a string is a leaf value, and `null` is the explicit HL7 null.
fn build_repetition(value: &Value, separators: &Separators) -> Repetition {
    match value {
        Value::Object(entries) => Repetition {
            components: build_components(entries, separators),
        },
        Value::Null => Repetition {
            components: vec![null_component()],
        },
        other => Repetition {
            components: vec![Component {
                subcomponents: vec![leaf_subcomponent(other, separators)],
            }],
        },
    }
}

/// Every component of one field repetition, 1-based, positioned the same
/// way as [`build_fields`] positions fields — but a component number never
/// repeats within one repetition, so an array here (which should not arise
/// from this crate's intended input) is read as its first item only.
fn build_components(entries: &[(String, Value)], separators: &Separators) -> Vec<Component> {
    let indexed = index_entries(entries);
    let len = indexed.keys().max().copied().unwrap_or(0);
    (1..=len)
        .map(|n| match indexed.get(&n) {
            Some(value) => build_component(first_occurrence(value), separators),
            None => Component::default(),
        })
        .collect()
}

fn build_component(value: &Value, separators: &Separators) -> Component {
    match value {
        Value::Object(entries) => Component {
            subcomponents: build_subcomponents(entries, separators),
        },
        Value::Null => null_component(),
        other => Component {
            subcomponents: vec![leaf_subcomponent(other, separators)],
        },
    }
}

/// Every subcomponent of one component, 1-based, positioned the same way
/// as [`build_components`].
fn build_subcomponents(entries: &[(String, Value)], separators: &Separators) -> Vec<Subcomponent> {
    let indexed = index_entries(entries);
    let len = indexed.keys().max().copied().unwrap_or(0);
    (1..=len)
        .map(|n| match indexed.get(&n) {
            Some(value) => build_subcomponent(first_occurrence(value), separators),
            None => Subcomponent::default(),
        })
        .collect()
}

/// A subcomponent is always a leaf; an object this deep is outside what the
/// forward crate ever emits, and reads as the explicit null rather than
/// losing the value silently.
fn build_subcomponent(value: &Value, separators: &Separators) -> Subcomponent {
    match value {
        Value::Object(_) | Value::Null => Subcomponent::new(NULL),
        other => leaf_subcomponent(other, separators),
    }
}

fn first_occurrence(value: &Value) -> &Value {
    match value {
        // An empty array here is malformed input this crate never produces
        // itself; falling back to the array value (which `build_component`
        // /`build_subcomponent` then read as the explicit null, via their
        // catch-all arm) keeps this infallible rather than panicking.
        Value::Array(items) => items.first().unwrap_or(value),
        other => other,
    }
}

fn null_component() -> Component {
    Component {
        subcomponents: vec![Subcomponent::new(NULL)],
    }
}

/// A leaf subcomponent from a scalar JSON value: a string is re-escaped
/// (§`to_raw`); a number or boolean — never emitted by the forward crate,
/// but tolerated in hand-edited input — is written from its literal source
/// text.
fn leaf_subcomponent(value: &Value, separators: &Separators) -> Subcomponent {
    match value {
        Value::String(s) => Subcomponent::new(to_raw(s, separators)),
        Value::Number(raw) => Subcomponent::new(to_raw(raw, separators)),
        Value::Bool(b) => Subcomponent::new(if *b { "1" } else { "0" }),
        Value::Object(_) | Value::Array(_) | Value::Null => Subcomponent::new(NULL),
    }
}

/// The integer after `key`'s last `.`, e.g. `5` from `"PID.5"` or `1` from
/// `"XPN.1"`.
fn trailing_index(key: &str) -> Option<usize> {
    key.rsplit('.').next()?.parse().ok()
}

/// Turn decoded leaf text back into raw ER7: the inverse of
/// `Subcomponent::value`.
///
/// The forward crate's JSON string is not fully decoded — `unescape` (`er7`
/// spec §5.2) resolves only the five delimiter sequences and `\Xdd..\`,
/// leaving every other escape sequence (`\.br\`, `\H\`, `\Zdd..\`, …)
/// exactly as written. So the text handed to this function is a mix of
/// literal delimiter characters and untouched escape sequences, and
/// re-encoding it means telling those apart rather than escaping every
/// character blindly. Retokenizing with [`escapes`] does exactly that: a
/// run with no escape character in it ([`Escape::Text`]) is data and gets
/// [`escape`]d for the delimiters it contains, while every other token is
/// already valid ER7 and is written back unchanged.
fn to_raw(text: &str, separators: &Separators) -> String {
    let mut out = String::with_capacity(text.len());
    for token in escapes(text, separators) {
        match token {
            Escape::Text(run) => out.push_str(&escape(run, separators)),
            other => other.write_er7(&mut out, separators),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse_document;

    fn reconstruct_er7(json: &str) -> String {
        reconstruct(&parse_document(json).unwrap())
            .unwrap()
            .to_er7()
    }

    #[test]
    fn rebuilds_the_header_delimiters_literally() {
        let json = r#"{"ORM_O01": {"MSH": {"MSH.1": "|", "MSH.2": "^~\\&", "MSH.10": "1"}}}"#;
        let message = reconstruct(&parse_document(json).unwrap()).unwrap();
        assert_eq!(message.query("MSH-1").unwrap().as_deref(), Some("|"));
        assert_eq!(message.query("MSH-2").unwrap().as_deref(), Some(r"^~\&"));
        assert_eq!(message.query("MSH-10").unwrap().as_deref(), Some("1"));
    }

    #[test]
    fn pads_missing_field_and_component_positions() {
        let json = r#"{"X": {
            "MSH": {"MSH.1": "|", "MSH.2": "^~\\&"},
            "PID": {"PID.5": {"XPN.2": "FOUAZ"}}
        }}"#;
        let message = reconstruct(&parse_document(json).unwrap()).unwrap();
        assert_eq!(message.query("PID-5").unwrap().as_deref(), Some("^FOUAZ"));
        assert_eq!(message.query("PID-5.1").unwrap().as_deref(), Some(""));
        assert_eq!(message.query("PID-5.2").unwrap().as_deref(), Some("FOUAZ"));
    }

    #[test]
    fn explicit_null_round_trips_from_json_null() {
        let json = r#"{"X": {
            "MSH": {"MSH.1": "|", "MSH.2": "^~\\&"},
            "PID": {"PID.2": null}
        }}"#;
        let message = reconstruct(&parse_document(json).unwrap()).unwrap();
        assert!(message.segment("PID").unwrap().field(2).unwrap().is_null());
        assert!(reconstruct_er7(json).contains("PID||\"\""));
    }

    #[test]
    fn repeating_fields_come_from_a_json_array() {
        let json = r#"{"X": {
            "MSH": {"MSH.1": "|", "MSH.2": "^~\\&"},
            "PID": {"PID.3": [{"CX.1": "111"}, {"CX.1": "222"}]}
        }}"#;
        let message = reconstruct(&parse_document(json).unwrap()).unwrap();
        assert_eq!(message.query_all("PID-3.1").unwrap(), vec!["111", "222"]);
    }

    #[test]
    fn re_escapes_decoded_delimiters_and_keeps_other_sequences_literal() {
        let json = r#"{"X": {
            "MSH": {"MSH.1": "|", "MSH.2": "^~\\&"},
            "NTE": {"NTE.3": "A&B | C^D\\.br\\next"}
        }}"#;
        let er7 = reconstruct_er7(json);
        assert!(
            er7.ends_with(r"NTE|||A\T\B \F\ C\S\D\.br\next"),
            "got {er7:?}"
        );
    }

    #[test]
    fn flattens_group_entries_including_repeated_groups() {
        let json = r#"{"ORU_R01": {
            "MSH": {"MSH.1": "|", "MSH.2": "^~\\&"},
            "ORU_R01.OBSERVATION": [
                {"OBX": {"OBX.1": "1"}},
                {"OBX": {"OBX.1": "2"}}
            ]
        }}"#;
        let message = reconstruct(&parse_document(json).unwrap()).unwrap();
        assert_eq!(message.segments.len(), 3);
        assert_eq!(message.query_all("OBX-1").unwrap(), vec!["1", "2"]);
    }

    #[test]
    fn rejects_a_document_with_no_msh() {
        let root = parse_document(r#"{"X": {"PID": {"PID.1": "1"}}}"#).unwrap();
        assert!(matches!(reconstruct(&root), Err(Hl7Error::MissingMsh)));
    }
}
