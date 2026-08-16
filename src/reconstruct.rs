//! Rebuilding the ER7 value tree from the parsed XML [`Node`] tree.
//!
//! This is the inverse of the sibling `hl7-v2-from-er7-into-xml` crate's
//! `src/xml.rs`. That crate names every element after either an HL7 v2.5
//! data type (`<XPN.1>`) or a bare position (`<PID.5.1>`) — but in both
//! cases, the number after the element name's *last* dot is always the
//! 1-based position at that level: field number under a segment, component
//! number under a field, subcomponent number under a component. That is
//! the one fact this module leans on, and it means reconstruction needs no
//! HL7 v2.5 data-type dictionary at all — see `spec/index.md` §3 for the
//! full rule set this implements.

use crate::Hl7Error;
use crate::xml::Node;
use er7::escape::{Escape, escape, escapes};
use er7::message::NULL;
use er7::{Component, Field, Message, Repetition, Segment, Separators, Subcomponent};
use std::collections::BTreeMap;

/// Reconstruct a full [`Message`] from a parsed v2.xml document's root
/// element.
pub fn reconstruct(root: &Node) -> Result<Message, Hl7Error> {
    let mut flat = Vec::new();
    flatten_segments(root, &mut flat);
    let header = flat.first().ok_or(Hl7Error::Empty)?;
    if !is_header_name(&header.name) {
        return Err(Hl7Error::MissingMsh);
    }
    let separators = header_separators(header)?;
    let segments = flat
        .into_iter()
        .map(|node| build_segment(node, &separators))
        .collect();
    Ok(Message {
        separators,
        segments,
    })
}

/// Walk `node`'s children, collecting the segment elements in document
/// order and descending into (but not keeping) group elements.
///
/// A child is a group, not a segment, exactly when its name contains a
/// `.` — real segment IDs never do, while every group element the sibling
/// crate emits is named `{message-structure}.{group}` regardless of how
/// deeply it is nested (`hl7-v2-from-er7-into-xml` spec §3.2). This is why
/// reconstruction needs no message-structure grammar either: it flattens
/// every group away, which is exactly what that crate's own `--flat` option
/// produces, and a flat segment sequence is all ER7 needs.
fn flatten_segments<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
    for kid in &node.kids {
        if kid.name.contains('.') {
            flatten_segments(kid, out);
        } else {
            out.push(kid);
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
fn header_separators(header: &Node) -> Result<Separators, Hl7Error> {
    let field_separator = field_text(header, 1).ok_or_else(|| {
        Hl7Error::BadMshHeader(format!(
            "{} has no {}.1 field (the field separator)",
            header.name, header.name
        ))
    })?;
    let encoding = field_text(header, 2).unwrap_or("");
    // `Separators::from_header` reads the field separator as the character
    // right after the segment name, then the encoding characters up to the
    // next field separator — so appending the field separator again gives
    // it the terminator it expects, exactly as a real "MSH|^~\&|..." line
    // would.
    let synthetic = format!(
        "{}{field_separator}{encoding}{field_separator}",
        header.name
    );
    Separators::from_header(&synthetic).map_err(|e| Hl7Error::BadMshHeader(e.to_string()))
}

/// The decoded text of `segment`'s `.n` child, if present and non-null.
fn field_text(segment: &Node, n: usize) -> Option<&str> {
    let target = format!("{}.{n}", segment.name);
    segment
        .kids
        .iter()
        .find(|kid| kid.name == target)
        .and_then(|kid| kid.text.as_deref())
}

fn build_segment(node: &Node, separators: &Separators) -> Segment {
    let mut fields = build_fields(&node.kids, separators);
    if is_header_name(&node.name) {
        // Fields 1 and 2 of a header are the delimiters themselves, stored
        // literally rather than escaped (`er7` spec §3.4) — the generic
        // build above cannot know that, so its guesses for these two slots
        // are replaced here.
        while fields.len() < 2 {
            fields.push(Field::default());
        }
        fields[0] = literal_field(field_text(node, 1).unwrap_or_default());
        fields[1] = literal_field(field_text(node, 2).unwrap_or_default());
    }
    Segment {
        name: node.name.clone(),
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

/// Every non-null field, 1-based, built from a segment element's children:
/// element `{seg}.n` names field `n`, and repeated siblings sharing the
/// same `n` are that field's repetitions, in document order. A field
/// number this segment never mentions is absent (`Field::default()`), not
/// empty — matching how the forward crate omits a field entirely rather
/// than rendering an empty element for it.
fn build_fields(kids: &[Node], separators: &Separators) -> Vec<Field> {
    let groups = group_by_index(kids);
    pad(groups, |occurrences| Field {
        repetitions: occurrences
            .iter()
            .map(|node| build_repetition(node, separators))
            .collect(),
    })
}

/// One field repetition, or one component: a childless element with text
/// is a leaf value, a childless element with none is the explicit HL7 null,
/// and an element with children recurses one level down (components under
/// a repetition, subcomponents under a component).
fn build_repetition(node: &Node, separators: &Separators) -> Repetition {
    if !node.kids.is_empty() {
        Repetition {
            components: build_components(&node.kids, separators),
        }
    } else if let Some(text) = &node.text {
        Repetition {
            components: vec![Component {
                subcomponents: vec![Subcomponent::new(to_raw(text, separators))],
            }],
        }
    } else {
        Repetition {
            components: vec![null_component()],
        }
    }
}

/// Every component of one field repetition, 1-based, positioned the same
/// way as [`build_fields`] positions fields — but a component number never
/// repeats within one repetition, so a duplicate keeps only its first
/// occurrence rather than becoming a list.
fn build_components(kids: &[Node], separators: &Separators) -> Vec<Component> {
    let groups = group_by_index(kids);
    pad(groups, |occurrences| {
        build_component(occurrences[0], separators)
    })
}

fn build_component(node: &Node, separators: &Separators) -> Component {
    if !node.kids.is_empty() {
        Component {
            subcomponents: build_subcomponents(&node.kids, separators),
        }
    } else if let Some(text) = &node.text {
        Component {
            subcomponents: vec![Subcomponent::new(to_raw(text, separators))],
        }
    } else {
        null_component()
    }
}

/// Every subcomponent of one component, 1-based, positioned the same way
/// as [`build_components`].
fn build_subcomponents(kids: &[Node], separators: &Separators) -> Vec<Subcomponent> {
    let groups = group_by_index(kids);
    pad(groups, |occurrences| {
        build_subcomponent(occurrences[0], separators)
    })
}

/// A subcomponent is always a leaf; an element with children this deep is
/// outside what the forward crate ever emits, and reads as the explicit
/// null rather than losing the value silently.
fn build_subcomponent(node: &Node, separators: &Separators) -> Subcomponent {
    match &node.text {
        Some(text) => Subcomponent::new(to_raw(text, separators)),
        None => Subcomponent::new(NULL),
    }
}

fn null_component() -> Component {
    Component {
        subcomponents: vec![Subcomponent::new(NULL)],
    }
}

/// Group `kids` by the integer after each name's last `.`, preserving
/// first-appearance order of occurrences within a group. A name with no
/// parseable trailing index — which should not arise from this crate's
/// intended input — is assigned the position right after the highest index
/// seen so far, so malformed input still lands somewhere instead of being
/// dropped (this crate never fails below the header; see `spec/index.md`
/// §5).
fn group_by_index(kids: &[Node]) -> BTreeMap<usize, Vec<&Node>> {
    let mut groups: BTreeMap<usize, Vec<&Node>> = BTreeMap::new();
    let mut next = 1usize;
    for kid in kids {
        let index = trailing_index(&kid.name)
            .filter(|&i| i >= 1)
            .unwrap_or(next);
        next = index + 1;
        groups.entry(index).or_default().push(kid);
    }
    groups
}

/// Turn an index -> occurrences map into a dense, 1-based `Vec`, filling
/// any index the map skipped with `T::default()`.
fn pad<T: Default>(
    groups: BTreeMap<usize, Vec<&Node>>,
    mut build: impl FnMut(&[&Node]) -> T,
) -> Vec<T> {
    let len = groups.keys().max().copied().unwrap_or(0);
    let mut built: BTreeMap<usize, T> = groups
        .into_iter()
        .map(|(index, occurrences)| (index, build(&occurrences)))
        .collect();
    (1..=len)
        .map(|i| built.remove(&i).unwrap_or_default())
        .collect()
}

/// The integer after `name`'s last `.`, e.g. `5` from `"PID.5"` or `1` from
/// `"XPN.1"`.
fn trailing_index(name: &str) -> Option<usize> {
    name.rsplit('.').next()?.parse().ok()
}

/// Turn decoded leaf text back into raw ER7: the inverse of
/// `Subcomponent::value`.
///
/// The forward crate's XML text is not fully decoded — `unescape` (`er7`
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
    use crate::xml::parse_document;

    fn reconstruct_er7(xml: &str) -> String {
        reconstruct(&parse_document(xml).unwrap()).unwrap().to_er7()
    }

    #[test]
    fn rebuilds_the_header_delimiters_literally() {
        let xml = r#"<ORM_O01><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2><MSH.10>1</MSH.10></MSH></ORM_O01>"#;
        let message = reconstruct(&parse_document(xml).unwrap()).unwrap();
        assert_eq!(message.query("MSH-1").unwrap().as_deref(), Some("|"));
        assert_eq!(message.query("MSH-2").unwrap().as_deref(), Some(r"^~\&"));
        assert_eq!(message.query("MSH-10").unwrap().as_deref(), Some("1"));
        assert!(message.to_er7().starts_with(r"MSH|^~\&|"));
    }

    #[test]
    fn pads_missing_field_and_component_positions() {
        let xml = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <PID><PID.5><XPN.2>FOUAZ</XPN.2></PID.5></PID></X>"#;
        let message = reconstruct(&parse_document(xml).unwrap()).unwrap();
        assert_eq!(message.query("PID-5").unwrap().as_deref(), Some("^FOUAZ"));
        assert_eq!(message.query("PID-5.1").unwrap().as_deref(), Some(""));
        assert_eq!(message.query("PID-5.2").unwrap().as_deref(), Some("FOUAZ"));
    }

    #[test]
    fn explicit_null_round_trips_from_an_empty_element() {
        let xml = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH><PID><PID.2/></PID></X>"#;
        let message = reconstruct(&parse_document(xml).unwrap()).unwrap();
        assert!(message.segment("PID").unwrap().field(2).unwrap().is_null());
        assert!(reconstruct_er7(xml).contains("PID||\"\""));
    }

    #[test]
    fn re_escapes_decoded_delimiters_and_keeps_other_sequences_literal() {
        let xml = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <NTE><NTE.3>A&amp;B | C^D\.br\next</NTE.3></NTE></X>"#;
        let er7 = reconstruct_er7(xml);
        assert!(
            er7.ends_with(r"NTE|||A\T\B \F\ C\S\D\.br\next"),
            "got {er7:?}"
        );
    }

    #[test]
    fn flattens_group_elements() {
        let xml = r#"<ORM_O01><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <ORM_O01.PATIENT><PID><PID.1>1</PID.1></PID></ORM_O01.PATIENT></ORM_O01>"#;
        let message = reconstruct(&parse_document(xml).unwrap()).unwrap();
        assert_eq!(message.segments.len(), 2);
        assert_eq!(message.segments[1].name, "PID");
    }

    #[test]
    fn rejects_a_document_with_no_msh() {
        let root = parse_document("<X><PID><PID.1>1</PID.1></PID></X>").unwrap();
        assert!(matches!(reconstruct(&root), Err(Hl7Error::MissingMsh)));
    }
}
