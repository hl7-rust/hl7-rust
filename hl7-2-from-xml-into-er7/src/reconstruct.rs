//! Rebuilding the ER7 value tree from the parsed XML [`Element`] tree.
//!
//! This is the inverse of the sibling `hl7-2-from-er7-into-xml` crate's
//! `src/xml.rs`. That crate names every element after either an HL7 v2.5
//! data type (`<XPN.1>`) or a bare position (`<PID.5.1>`) — but in both
//! cases, the number after the element name's *last* dot is always the
//! 1-based position at that level: field number under a segment, component
//! number under a field, subcomponent number under a component. That is
//! the one fact this module leans on, and it means reconstruction needs no
//! HL7 v2.5 data-type dictionary at all — see `spec/index.md` §3 for the
//! full rule set this implements.
//!
//! Every name this module reads is an element's *local* name: a document
//! that binds `urn:hl7-org:v2xml` to a prefix writes `<ns0:MSH>` and
//! `<ns0:MSH.1>` for the same elements the forward crate writes as `<MSH>`
//! and `<MSH.1>`, and which prefix — if any — a serializer chose says
//! nothing about the message. See `spec/index.md` §2.1.

use crate::Hl7Error;
use crate::xml::Element;
use er7::escape::{Escape, escape, escapes};
use er7::{Component, Field, Message, Repetition, Segment, Separators, Subcomponent};
use std::collections::BTreeMap;

/// Reconstruct a full [`Message`] from a parsed v2.xml document's root
/// element.
/// # Errors
///
/// [`Hl7Error`] when the tree holds no segments, when the first is not MSH,
/// or when the header does not carry the delimiters the rest is written in.
pub fn reconstruct(root: &Element) -> Result<Message, Hl7Error> {
    let mut flat = Vec::new();
    flatten_segments(root, &mut flat);
    let header = flat.first().ok_or(Hl7Error::Empty)?;
    if !is_header_name(header.local_name()) {
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
/// A child is a group, not a segment, exactly when its local name contains
/// a `.` — real segment IDs never do, while every group element the sibling
/// crate emits is named `{message-structure}.{group}` regardless of how
/// deeply it is nested (`hl7-2-from-er7-into-xml` spec §3.2). This is why
/// reconstruction needs no message-structure grammar either: it flattens
/// every group away, which is exactly what that crate's own `--flat` option
/// produces, and a flat segment sequence is all ER7 needs.
fn flatten_segments<'a>(node: &'a Element, out: &mut Vec<&'a Element>) {
    for kid in &node.children {
        if kid.local_name().contains('.') {
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
fn header_separators(header: &Element) -> Result<Separators, Hl7Error> {
    let name = header.local_name();
    let field_separator = field_text(header, 1).ok_or_else(|| {
        Hl7Error::BadMshHeader(format!(
            "{name} has no {name}.1 field (the field separator)"
        ))
    })?;
    let encoding = field_text(header, 2).unwrap_or("");
    // `Separators::from_header` reads the field separator as the character
    // right after the segment name, then the encoding characters up to the
    // next field separator — so appending the field separator again gives
    // it the terminator it expects, exactly as a real "MSH|^~\&|..." line
    // would.
    let synthetic = format!("{name}{field_separator}{encoding}{field_separator}");
    Separators::from_header(&synthetic).map_err(|e| Hl7Error::BadMshHeader(e.to_string()))
}

/// The decoded text of `segment`'s `.n` child, if present and non-null.
fn field_text(segment: &Element, n: usize) -> Option<&str> {
    let target = format!("{}.{n}", segment.local_name());
    segment
        .children
        .iter()
        .find(|kid| kid.local_name() == target)
        .and_then(leaf_text)
}

fn build_segment(node: &Element, separators: &Separators) -> Segment {
    let mut fields = build_fields(&node.children, separators);
    if is_header_name(node.local_name()) {
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
        name: node.local_name().to_owned(),
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
fn build_fields(kids: &[Element], separators: &Separators) -> Vec<Field> {
    let groups = group_by_index(kids);
    pad(groups, |occurrences| Field {
        repetitions: occurrences
            .iter()
            .map(|node| build_repetition(node, separators))
            .collect(),
    })
}

/// One field repetition, or one component: a childless element with text
/// is a leaf value, a childless element with none is *empty* (§4.4), and an
/// element with children recurses one level down (components under a
/// repetition, subcomponents under a component).
fn build_repetition(node: &Element, separators: &Separators) -> Repetition {
    if !node.children.is_empty() {
        Repetition {
            components: build_components(&node.children, separators),
        }
    } else if let Some(text) = leaf_text(node) {
        Repetition {
            components: vec![Component {
                subcomponents: vec![Subcomponent::new(to_raw(text, separators))],
            }],
        }
    } else {
        Repetition::default()
    }
}

/// Every component of one field repetition, 1-based, positioned the same
/// way as [`build_fields`] positions fields — but a component number never
/// repeats within one repetition, so a duplicate keeps only its first
/// occurrence rather than becoming a list.
fn build_components(kids: &[Element], separators: &Separators) -> Vec<Component> {
    let groups = group_by_index(kids);
    pad(groups, |occurrences| {
        build_component(occurrences[0], separators)
    })
}

fn build_component(node: &Element, separators: &Separators) -> Component {
    if !node.children.is_empty() {
        Component {
            subcomponents: build_subcomponents(&node.children, separators),
        }
    } else if let Some(text) = leaf_text(node) {
        Component {
            subcomponents: vec![Subcomponent::new(to_raw(text, separators))],
        }
    } else {
        Component::default()
    }
}

/// Every subcomponent of one component, 1-based, positioned the same way
/// as [`build_components`].
fn build_subcomponents(kids: &[Element], separators: &Separators) -> Vec<Subcomponent> {
    let groups = group_by_index(kids);
    pad(groups, |occurrences| {
        build_subcomponent(occurrences[0], separators)
    })
}

/// A subcomponent is always a leaf; an element with children this deep is
/// outside what the forward crate ever emits, and reads as empty rather
/// than inventing a value for it.
fn build_subcomponent(node: &Element, separators: &Separators) -> Subcomponent {
    match leaf_text(node) {
        Some(text) => Subcomponent::new(to_raw(text, separators)),
        None => Subcomponent::default(),
    }
}

/// The text of a leaf, or `None`.
///
/// An element with children has none, however much text sits between them:
/// this crate neither emits nor expects mixed content, and reading a stray
/// character as a value would put it somewhere it does not belong. The
/// reader this crate used to carry dropped such text before it was ever
/// seen; `hl7-2-xml-lite-helper` keeps it, so the rule is stated here instead.
fn leaf_text(node: &Element) -> Option<&str> {
    if node.children.is_empty() {
        node.text_opt()
    } else {
        None
    }
}

/// The highest position this crate will honour, at any level.
///
/// Reconstruction is dense: position `n` costs `n` slots, because every
/// position below it has to exist for `n` to be the `n`th. A name is just
/// text, so `<PID.100000000>` — a hundred bytes of input — otherwise asks
/// for a hundred million fields, and a larger number asks for more memory
/// than the machine has. Real segments run to tens of fields; this is far
/// above anything HL7 defines and far below anything that hurts. A position
/// past it is treated like a name with no position at all (below).
const MAX_POSITION: usize = 10_000;

/// Group `kids` by the integer after each name's last `.`, preserving
/// first-appearance order of occurrences within a group. A name with no
/// parseable trailing index — which should not arise from this crate's
/// intended input — is assigned the position right after the highest index
/// seen so far, so malformed input still lands somewhere instead of being
/// dropped (this crate never fails below the header; see `spec/index.md`
/// §5).
fn group_by_index(kids: &[Element]) -> BTreeMap<usize, Vec<&Element>> {
    let mut groups: BTreeMap<usize, Vec<&Element>> = BTreeMap::new();
    let mut next = 1usize;
    for kid in kids {
        let index = trailing_index(kid.local_name())
            .filter(|&i| (1..=MAX_POSITION).contains(&i))
            .unwrap_or(next);
        next = index + 1;
        groups.entry(index).or_default().push(kid);
    }
    groups
}

/// Turn an index -> occurrences map into a dense, 1-based `Vec`, filling
/// any index the map skipped with `T::default()`.
fn pad<T: Default>(
    groups: BTreeMap<usize, Vec<&Element>>,
    mut build: impl FnMut(&[&Element]) -> T,
) -> Vec<T> {
    let len = groups.keys().max().copied().unwrap_or(0);
    let mut by_index: BTreeMap<usize, T> = groups
        .into_iter()
        .map(|(index, occurrences)| (index, build(&occurrences)))
        .collect();
    (1..=len)
        .map(|i| by_index.remove(&i).unwrap_or_default())
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
/// already valid ER7 and is written back unchanged. The one exception is
/// [`Escape::Unterminated`] — an escape character with nothing closing it,
/// which is what a `\E\` in the original message decodes to — and it is
/// re-escaped as data, because emitting it raw would produce ER7 the next
/// reader cannot parse.
fn to_raw(text: &str, separators: &Separators) -> String {
    let mut out = String::with_capacity(text.len());
    for token in escapes(text, separators) {
        match token {
            // An unterminated escape character can only be data: no
            // sequence closes it, so writing it back as-is would emit ER7
            // that no receiver can parse. Escaping it restores the `\E\`
            // the forward crate decoded away.
            Escape::Text(run) | Escape::Unterminated(run) => {
                out.push_str(&escape(run, separators));
            }
            other => other.write_er7(&mut out, separators),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::parse;

    fn reconstruct_er7(xml: &str) -> String {
        reconstruct(&parse(xml).unwrap()).unwrap().to_er7()
    }

    #[test]
    fn rebuilds_the_header_delimiters_literally() {
        let xml = r#"<ORM_O01><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2><MSH.10>1</MSH.10></MSH></ORM_O01>"#;
        let message = reconstruct(&parse(xml).unwrap()).unwrap();
        assert_eq!(message.query("MSH-1").unwrap().as_deref(), Some("|"));
        assert_eq!(message.query("MSH-2").unwrap().as_deref(), Some(r"^~\&"));
        assert_eq!(message.query("MSH-10").unwrap().as_deref(), Some("1"));
        assert!(message.to_er7().starts_with(r"MSH|^~\&|"));
    }

    #[test]
    fn pads_missing_field_and_component_positions() {
        let xml = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <PID><PID.5><XPN.2>FOUAZ</XPN.2></PID.5></PID></X>"#;
        let message = reconstruct(&parse(xml).unwrap()).unwrap();
        assert_eq!(message.query("PID-5").unwrap().as_deref(), Some("^FOUAZ"));
        assert_eq!(message.query("PID-5.1").unwrap().as_deref(), Some(""));
        assert_eq!(message.query("PID-5.2").unwrap().as_deref(), Some("FOUAZ"));
    }

    #[test]
    fn an_empty_element_is_empty_and_the_literal_quotes_are_the_null() {
        // The XML Encoding Rules: an empty element "is treated as not
        // existing", while `""` says the sender deleted the value. Reading
        // the first as the second turns a padded document into a message
        // full of deletion markers, so they must stay apart.
        let head = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>"#;
        let empty = format!("{head}<PID><PID.2/><PID.3></PID.3></PID></X>");
        let message = reconstruct(&parse(&empty).unwrap()).unwrap();
        let pid = message.segment("PID").unwrap();
        assert!(pid.field(2).unwrap().is_empty());
        assert!(!pid.field(2).unwrap().is_null());
        assert!(pid.field(3).unwrap().is_empty());
        // Both positions render as empty fields — separators with nothing
        // in them, which is what an ER7 sender writes for "not sent".
        assert_eq!(reconstruct_er7(&empty), "MSH|^~\\&\rPID|||");

        let null = format!(r#"{head}<PID><PID.2>""</PID.2></PID></X>"#);
        let message = reconstruct(&parse(&null).unwrap()).unwrap();
        assert!(message.segment("PID").unwrap().field(2).unwrap().is_null());
        assert!(reconstruct_er7(&null).contains("PID||\"\""));
    }

    #[test]
    fn an_empty_element_below_a_field_is_empty_too() {
        // Schema-shaped documents pad a value out to every component its
        // type declares; those are empty, not null.
        let xml = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <PID><PID.3><CX.1>7</CX.1><CX.2/><CX.4><HD.1/><HD.2/></CX.4></PID.3></PID></X>"#;
        // The padding survives as empty components and subcomponents —
        // structure with no values in it, never the null `""`.
        let er7 = reconstruct_er7(xml);
        assert!(er7.ends_with("PID|||7^^^&"), "got {er7:?}");
        assert!(
            !er7.contains('"'),
            "padding must not become a null: {er7:?}"
        );
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
    fn re_escapes_a_bare_escape_character() {
        // `\E\` decodes to a lone `\` on the way in, and a lone `\` is an
        // unterminated escape sequence: writing it back raw produced ER7
        // whose next reader would swallow the rest of the value.
        let xml = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <NTE><NTE.3>a\b</NTE.3></NTE></X>"#;
        assert!(reconstruct_er7(xml).ends_with(r"NTE|||a\E\b"));
    }

    #[test]
    fn an_absurd_position_does_not_allocate_the_world() {
        // A hundred bytes of input asked for a hundred million fields
        // before MAX_POSITION capped it; a larger number asked for more
        // memory than the machine has.
        let xml = r#"<X><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <PID><PID.100000000>x</PID.100000000></PID></X>"#;
        let er7 = reconstruct_er7(xml);
        assert!(er7.len() < 1000, "output ballooned to {} bytes", er7.len());
        // The value still lands somewhere rather than being dropped.
        assert!(er7.contains('x'), "got {er7:?}");
    }

    #[test]
    fn flattens_group_elements() {
        let xml = r#"<ORM_O01><MSH><MSH.1>|</MSH.1><MSH.2>^~\&amp;</MSH.2></MSH>
            <ORM_O01.PATIENT><PID><PID.1>1</PID.1></PID></ORM_O01.PATIENT></ORM_O01>"#;
        let message = reconstruct(&parse(xml).unwrap()).unwrap();
        assert_eq!(message.segments.len(), 2);
        assert_eq!(message.segments[1].name, "PID");
    }

    #[test]
    fn reads_prefixed_element_names_the_same_as_unprefixed_ones() {
        // The same document a serializer that binds the v2.xml namespace to
        // a prefix, rather than making it the default, would write.
        let xml = r#"<ns0:ORM_O01 xmlns:ns0="urn:hl7-org:v2xml">
            <ns0:MSH><ns0:MSH.1>|</ns0:MSH.1><ns0:MSH.2>^~\&amp;</ns0:MSH.2><ns0:MSH.10>1</ns0:MSH.10></ns0:MSH>
            <ns0:ORM_O01.PATIENT><ns0:PID><ns0:PID.5>
                <ns0:XPN.1><ns0:FN.1>TEST</ns0:FN.1></ns0:XPN.1><ns0:XPN.2>FOUAZ</ns0:XPN.2>
            </ns0:PID.5></ns0:PID></ns0:ORM_O01.PATIENT></ns0:ORM_O01>"#;
        let message = reconstruct(&parse(xml).unwrap()).unwrap();
        // The header is recognized, and the prefix is not part of a segment ID.
        assert_eq!(message.segments[0].name, "MSH");
        // The group element is still seen as a group, and flattened away.
        assert_eq!(message.segments.len(), 2);
        assert_eq!(message.segments[1].name, "PID");
        // Positions still come from the local name's trailing index, at
        // every level.
        assert_eq!(message.query("MSH-10").unwrap().as_deref(), Some("1"));
        assert_eq!(message.query("PID-5.1.1").unwrap().as_deref(), Some("TEST"));
        assert_eq!(message.query("PID-5.2").unwrap().as_deref(), Some("FOUAZ"));
        assert!(message.to_er7().starts_with(r"MSH|^~\&|"));
    }

    #[test]
    fn rejects_a_document_with_no_msh() {
        let root = parse("<X><PID><PID.1>1</PID.1></PID></X>").unwrap();
        assert!(matches!(reconstruct(&root), Err(Hl7Error::MissingMsh)));
    }
}
