//! Building the typed JSON tree from parsed segments, and serializing it.
//!
//! Building (§4.1) proceeds in two stages, mirroring this crate's sibling
//! `hl7-2-from-er7-into-xml`:
//!
//! 1. [`segment_to_node`] turns one [`Segment`] into a [`Node`] tree keyed
//!    the same way the sibling crate names XML elements (`PID.5`, `XPN.1`,
//!    ...), using the [`crate::types`] data-type tables. A `Node` may have
//!    several children with the same name — one per field repetition, or
//!    one per repeating segment/group.
//! 2. [`node_to_value`] turns a `Node` (and, at the document level, a list
//!    of top-level nodes) into a [`Value`], collapsing same-named siblings
//!    into a JSON array so the result is valid, idiomatic JSON — see
//!    `spec/index.md` §4 for the exact rules.

use crate::types::{composite_components, field_type};
use er7::{Component, Repetition, Segment, Separators};
use std::fmt::Write as _;

/// One node of the intermediate tree built from parsed segments, before
/// same-named siblings are collapsed into JSON arrays by [`node_to_value`].
#[derive(Debug, Clone)]
pub struct Node {
    /// Key this node will render under, e.g. `"PID"`, `"PID.5"`, `"XPN.1"`.
    pub name: String,
    /// Leaf text content, when this node has no children.
    pub text: Option<String>,
    /// Child nodes, when this node is a container (segment, group, or
    /// composite field/component).
    pub kids: Vec<Node>,
}

impl Node {
    /// A container node with no children yet.
    pub fn group(name: impl Into<String>) -> Self {
        Node {
            name: name.into(),
            text: None,
            kids: Vec::new(),
        }
    }

    /// A leaf node holding `value`. The HL7 explicit null (`""`) and the
    /// empty string both produce a node with no text, which
    /// [`node_to_value`] renders as JSON `null`.
    pub fn leaf(name: impl Into<String>, value: &str) -> Self {
        let text = if value.is_empty() || value == "\"\"" {
            None
        } else {
            Some(value.to_string())
        };
        Node {
            name: name.into(),
            text,
            kids: Vec::new(),
        }
    }
}

/// The JSON key for a segment ID, with any `.` removed.
///
/// The reverse sibling crate has no message-structure grammar and does not
/// need one: it tells a group key from a segment key by the `.` in a
/// group's `{structure}.{group}` name, which works because real segment IDs
/// never contain one (that crate's spec §3.1). `er7` is lenient about what
/// it accepts as a segment ID, though, so a message can carry `Z.1` — and
/// writing that out as the key `"Z.1"` would hand the reverse crate
/// something it must read as a group, flattening the segment and every
/// value in it out of existence. Dropping the `.` here keeps the one
/// invariant the pair rests on true by construction.
fn segment_key(name: &str) -> String {
    if name.contains('.') {
        name.replace('.', "")
    } else {
        name.to_string()
    }
}

/// Convert one segment into its `Node` tree, keyed the same way as this
/// crate's typed field/component naming (`spec/index.md` §4.2).
///
/// `separators` is the delimiter set the message declared; it is needed
/// because `er7` stores subcomponent text exactly as it arrived and decodes
/// escape sequences on demand, so decoding happens here, at the point the
/// text becomes JSON.
#[must_use]
pub fn segment_to_node(seg: &Segment, separators: &Separators) -> Node {
    let seg_name = segment_key(&seg.name);
    let mut node = Node::group(&seg_name);
    // OBX-5 has a variable type declared by the value of OBX-2.
    let obx_type = if seg.name == "OBX" {
        seg.first_value(2, 1, separators)
            .map(|v| v.trim().to_ascii_uppercase())
            .filter(|v| composite_components(v).is_some())
    } else {
        None
    };
    for (n, field) in (1_usize..).zip(&seg.fields) {
        if field.is_empty() {
            continue;
        }
        let name = format!("{seg_name}.{n}");
        let dt = match field_type(&seg.name, n) {
            Some("VAR") => obx_type.as_deref(),
            other => other,
        };
        for rep in &field.repetitions {
            if rep.is_empty() {
                continue;
            }
            node.kids.push(repeat_node(&name, dt, rep, separators));
        }
    }
    node
}

/// One field repetition -> one field node.
fn repeat_node(name: &str, dt: Option<&str>, rep: &Repetition, separators: &Separators) -> Node {
    // Explicit null for the whole field: node with no text -> JSON null.
    if rep.is_null() {
        return Node::group(name);
    }
    if let Some((dt, comps)) = dt.and_then(|dt| composite_components(dt).map(|comps| (dt, comps))) {
        // Known composite type: children named after the type's components.
        let mut node = Node::group(name);
        for ((ci, number), comp) in (0_usize..).zip(1_usize..).zip(&rep.components) {
            if comp.is_empty() {
                continue;
            }
            let cname = format!("{dt}.{number}");
            node.kids.push(component_node(
                &cname,
                comps.get(ci).copied(),
                comp,
                separators,
            ));
        }
        return node;
    }
    if let [only] = rep.components.as_slice()
        && let [sub] = only.subcomponents.as_slice()
    {
        // Primitive (or unknown) type with a single value.
        return Node::leaf(name, &sub.value(separators));
    }
    // Unknown structure: positional generic names SEG.n.m / SEG.n.m.k.
    let mut node = Node::group(name);
    for (number, comp) in (1_usize..).zip(&rep.components) {
        if comp.is_empty() {
            continue;
        }
        let cname = format!("{name}.{number}");
        node.kids
            .push(generic_component_node(&cname, comp, separators));
    }
    node
}

fn component_node(
    cname: &str,
    cdt: Option<&str>,
    comp: &Component,
    separators: &Separators,
) -> Node {
    if let Some(cdt) = cdt.filter(|t| composite_components(t).is_some()) {
        // Composite component: subcomponents named after its own components,
        // e.g. XPN.1 containing FN.1, or CX.4 containing HD.1/HD.2/HD.3.
        let mut node = Node::group(cname);
        for (number, sub) in (1_usize..).zip(&comp.subcomponents) {
            if sub.is_empty() {
                continue;
            }
            node.kids.push(Node::leaf(
                format!("{cdt}.{number}"),
                &sub.value(separators),
            ));
        }
        return node;
    }
    if let [sub] = comp.subcomponents.as_slice() {
        return Node::leaf(cname, &sub.value(separators));
    }
    generic_component_node(cname, comp, separators)
}

fn generic_component_node(cname: &str, comp: &Component, separators: &Separators) -> Node {
    if let [sub] = comp.subcomponents.as_slice() {
        return Node::leaf(cname, &sub.value(separators));
    }
    let mut node = Node::group(cname);
    for (number, sub) in (1_usize..).zip(&comp.subcomponents) {
        if sub.is_empty() {
            continue;
        }
        node.kids.push(Node::leaf(
            format!("{cname}.{number}"),
            &sub.value(separators),
        ));
    }
    node
}

/// A JSON value, restricted to what this crate ever produces: no booleans
/// and no numbers, since HL7 field text is never coerced to another type
/// (`spec/index.md` §4.4) — every scalar is either a JSON string or, for
/// the explicit HL7 null, JSON `null`.
#[derive(Debug, Clone)]
pub enum Value {
    /// An ordered set of key/value pairs. Order is insertion order (first
    /// appearance in the source segments), not sorted, so the JSON mirrors
    /// the message's own field/segment order.
    Object(Vec<(String, Value)>),
    /// A JSON array, used for repeating fields, repeating segments, and
    /// repeating groups (`spec/index.md` §4.3).
    Array(Vec<Value>),
    /// A field's text content.
    String(String),
    /// The explicit HL7 null (`""`), or a group left empty.
    Null,
}

/// Turn a flat, ordered list of top-level nodes (segments and/or groups)
/// into one [`Value::Object`], collapsing same-named siblings into arrays
/// exactly as [`node_to_value`] does for a single node's children.
#[must_use]
pub fn nodes_to_object(nodes: &[Node]) -> Value {
    Value::Object(group_children(nodes))
}

/// Convert one node to a `Value`: a container becomes an object (with
/// same-named children collapsed into arrays), a text leaf becomes a
/// string, and a childless, textless leaf becomes `null`.
#[must_use]
pub fn node_to_value(node: &Node) -> Value {
    if !node.kids.is_empty() {
        Value::Object(group_children(&node.kids))
    } else if let Some(text) = &node.text {
        Value::String(text.clone())
    } else {
        Value::Null
    }
}

/// Group a list of nodes by name, preserving first-appearance order; a name
/// that appears once maps directly to its value, a name that appears more
/// than once maps to a `Value::Array` of each occurrence's value, in order.
fn group_children(nodes: &[Node]) -> Vec<(String, Value)> {
    // `groups` is itself in first-appearance order: a name is pushed the
    // first time it is seen and only extended after that.
    let mut groups: Vec<(&str, Vec<&Node>)> = Vec::new();
    for node in nodes {
        if let Some((_, kids)) = groups.iter_mut().find(|(name, _)| *name == node.name) {
            kids.push(node);
        } else {
            groups.push((&node.name, vec![node]));
        }
    }
    groups
        .into_iter()
        .map(|(name, occurrences)| {
            let value = if let [only] = occurrences.as_slice() {
                node_to_value(only)
            } else {
                Value::Array(occurrences.iter().map(|n| node_to_value(n)).collect())
            };
            (name.to_string(), value)
        })
        .collect()
}

/// Serialize a `Value` as pretty-printed JSON (two-space indent, trailing
/// newline) — the default rendering.
#[must_use]
pub fn render_pretty(value: &Value) -> String {
    let mut out = String::new();
    write_pretty(value, "", &mut out);
    out.push('\n');
    out
}

/// Serialize a `Value` as compact JSON: no insignificant whitespace, one
/// line. Used when `Options::compact` (`--compact` on the CLI) is set.
#[must_use]
pub fn render_compact(value: &Value) -> String {
    let mut out = String::new();
    write_compact(value, &mut out);
    out
}

/// `pad` is the indentation of the line this value closes on; each level
/// of nesting adds two spaces to it, so no depth is counted.
fn write_pretty(value: &Value, pad: &str, out: &mut String) {
    let inner_pad = format!("{pad}  ");
    match value {
        Value::Object(entries) if entries.is_empty() => out.push_str("{}"),
        Value::Object(entries) => {
            out.push_str("{\n");
            for (i, (key, val)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&inner_pad);
                write_string(key, out);
                out.push_str(": ");
                write_pretty(val, &inner_pad, out);
            }
            out.push('\n');
            out.push_str(pad);
            out.push('}');
        }
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&inner_pad);
                write_pretty(item, &inner_pad, out);
            }
            out.push('\n');
            out.push_str(pad);
            out.push(']');
        }
        Value::String(s) => write_string(s, out),
        Value::Null => out.push_str("null"),
    }
}

fn write_compact(value: &Value, out: &mut String) {
    match value {
        Value::Object(entries) => {
            out.push('{');
            for (i, (key, val)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                write_compact(val, out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_compact(item, out);
            }
            out.push(']');
        }
        Value::String(s) => write_string(s, out),
        Value::Null => out.push_str("null"),
    }
}

/// Write `s` as a JSON string literal: quotes, backslashes, and control
/// characters are escaped per RFC 8259; other characters (including
/// non-ASCII UTF-8) pass through unescaped.
fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_output_is_exact_for_empty_and_multi_item_containers() {
        let text = |s: &str| Value::String(s.to_string());
        assert_eq!(render_pretty(&Value::Object(vec![])), "{}\n");
        assert_eq!(render_pretty(&Value::Array(vec![])), "[]\n");
        assert_eq!(
            render_pretty(&Value::Array(vec![text("a"), text("b"), text("c")])),
            "[\n  \"a\",\n  \"b\",\n  \"c\"\n]\n"
        );
        assert_eq!(
            render_pretty(&Value::Object(vec![
                ("x".into(), text("1")),
                ("y".into(), Value::Array(vec![])),
            ])),
            "{\n  \"x\": \"1\",\n  \"y\": []\n}\n"
        );
    }

    #[test]
    fn collapses_repeated_names_into_arrays() {
        let nodes = vec![
            Node::leaf("PID.3", "111"),
            Node::leaf("PID.3", "222"),
            Node::leaf("PID.5", "SMITH"),
        ];
        let value = nodes_to_object(&nodes);
        assert_eq!(
            render_compact(&value),
            r#"{"PID.3":["111","222"],"PID.5":"SMITH"}"#
        );
    }

    #[test]
    fn escapes_strings() {
        let mut out = String::new();
        write_string("a\"b\\c\rd\te\nf\u{1}g", &mut out);
        assert_eq!(out, "\"a\\\"b\\\\c\\rd\\te\\nf\\u0001g\"");
    }

    #[test]
    fn renders_null_and_empty_containers() {
        assert_eq!(render_compact(&Value::Null), "null");
        assert_eq!(render_compact(&Value::Object(vec![])), "{}");
        assert_eq!(render_compact(&Value::Array(vec![])), "[]");
        assert_eq!(
            render_compact(&Value::Array(vec![Value::Null, Value::String("x".into())])),
            "[null,\"x\"]"
        );
    }

    #[test]
    fn degradation_trees_keep_only_meaningful_positions() {
        let separators = Separators::default();
        assert!(Node::leaf("EMPTY", "").text.is_none());
        assert!(Node::leaf("NULL", "\"\"").text.is_none());

        let component = Component {
            subcomponents: vec![
                er7::Subcomponent::new(""),
                er7::Subcomponent::new("middle"),
                er7::Subcomponent::new(""),
            ],
        };
        let generic = component_node("PAIR.1", Some("ST"), &component, &separators);
        assert_eq!(generic.kids.len(), 1);
        assert_eq!(generic.kids[0].text.as_deref(), Some("middle"));

        let nested = component_node("CX.4", Some("HD"), &component, &separators);
        assert_eq!(nested.kids.len(), 1);

        let positional = repeat_node(
            "ZED.1",
            None,
            &Repetition {
                components: vec![Component::default(), component],
            },
            &separators,
        );
        assert_eq!(positional.kids.len(), 1);
        assert_eq!(positional.kids[0].name, "ZED.1.2");
    }

    #[test]
    fn blank_field_repetitions_are_skipped() {
        let message = er7::parse("MSH|^~\\&|APP\rZED|one~~two").unwrap();
        let node = segment_to_node(&message.segments[1], &message.separators);
        assert_eq!(node.kids.len(), 2);
    }
}
