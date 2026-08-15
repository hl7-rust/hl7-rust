//! Building the typed JSON tree from parsed segments, and serializing it.
//!
//! Building (§4.1) proceeds in two stages, mirroring this crate's sibling
//! `hl7-2-5-to-xml-using-rust`:
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

/// Convert one segment into its `Node` tree, keyed the same way as this
/// crate's typed field/component naming (`spec/index.md` §4.2).
///
/// `separators` is the delimiter set the message declared; it is needed
/// because `er7` stores subcomponent text exactly as it arrived and decodes
/// escape sequences on demand, so decoding happens here, at the point the
/// text becomes JSON.
pub fn segment_to_node(seg: &Segment, separators: &Separators) -> Node {
    let seg_name = seg.name.clone();
    let mut node = Node::group(&seg_name);
    // OBX-5 has a variable type declared by the value of OBX-2.
    let obx_type = if seg.name == "OBX" {
        first_value(seg, 2, 1, separators)
            .map(|v| v.trim().to_ascii_uppercase())
            .filter(|v| composite_components(v).is_some())
    } else {
        None
    };
    for (i, field) in seg.fields.iter().enumerate() {
        if field.is_empty() {
            continue;
        }
        let n = i + 1;
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

/// The decoded text of SEG-`field`.`component`, taking the first repetition
/// and first subcomponent, and treating an empty result as absent.
///
/// `er7` offers this shape as a path query (`OBX-2.1`), but a query searches
/// the whole message for the first matching segment, and here the segment is
/// already in hand — this is the same lookup scoped to one segment.
fn first_value(
    seg: &Segment,
    field: usize,
    component: usize,
    separators: &Separators,
) -> Option<String> {
    let text = seg
        .component(field, component)?
        .subcomponent(1)?
        .value(separators)
        .into_owned();
    Some(text).filter(|t| !t.is_empty())
}

/// One field repetition -> one field node.
fn repeat_node(name: &str, dt: Option<&str>, rep: &Repetition, separators: &Separators) -> Node {
    // Explicit null for the whole field: node with no text -> JSON null.
    if rep.is_null() {
        return Node::group(name);
    }
    if let Some(comps) = dt.and_then(composite_components) {
        // Known composite type: children named after the type's components.
        let dt = dt.unwrap();
        let mut node = Node::group(name);
        for (ci, comp) in rep.components.iter().enumerate() {
            if comp.is_empty() {
                continue;
            }
            let cname = format!("{}.{}", dt, ci + 1);
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
    for (ci, comp) in rep.components.iter().enumerate() {
        if comp.is_empty() {
            continue;
        }
        let cname = format!("{}.{}", name, ci + 1);
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
        for (si, sub) in comp.subcomponents.iter().enumerate() {
            if sub.is_empty() {
                continue;
            }
            node.kids.push(Node::leaf(
                format!("{}.{}", cdt, si + 1),
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
    for (si, sub) in comp.subcomponents.iter().enumerate() {
        if sub.is_empty() {
            continue;
        }
        node.kids.push(Node::leaf(
            format!("{}.{}", cname, si + 1),
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
pub fn nodes_to_object(nodes: &[Node]) -> Value {
    Value::Object(group_children(nodes))
}

/// Convert one node to a `Value`: a container becomes an object (with
/// same-named children collapsed into arrays), a text leaf becomes a
/// string, and a childless, textless leaf becomes `null`.
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
    let mut order: Vec<&str> = Vec::new();
    let mut groups: Vec<(&str, Vec<&Node>)> = Vec::new();
    for node in nodes {
        match groups.iter_mut().find(|(name, _)| *name == node.name) {
            Some((_, kids)) => kids.push(node),
            None => {
                order.push(&node.name);
                groups.push((&node.name, vec![node]));
            }
        }
    }
    order
        .into_iter()
        .map(|name| {
            let occurrences = &groups.iter().find(|(n, _)| *n == name).unwrap().1;
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
pub fn render_pretty(value: &Value) -> String {
    let mut out = String::new();
    write_pretty(value, 0, &mut out);
    out.push('\n');
    out
}

/// Serialize a `Value` as compact JSON: no insignificant whitespace, one
/// line. Used when `Options::compact` (`--compact` on the CLI) is set.
pub fn render_compact(value: &Value) -> String {
    let mut out = String::new();
    write_compact(value, &mut out);
    out
}

fn write_pretty(value: &Value, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    let inner_pad = "  ".repeat(depth + 1);
    match value {
        Value::Object(entries) if entries.is_empty() => out.push_str("{}"),
        Value::Object(entries) => {
            out.push_str("{\n");
            for (i, (key, val)) in entries.iter().enumerate() {
                out.push_str(&inner_pad);
                write_string(key, out);
                out.push_str(": ");
                write_pretty(val, depth + 1, out);
                if i + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push('}');
        }
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&inner_pad);
                write_pretty(item, depth + 1, out);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
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
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_repeated_names_into_arrays() {
        let nodes = vec![
            Node::leaf("PID.3", "111"),
            Node::leaf("PID.3", "222"),
            Node::leaf("PID.5", "SMITH"),
        ];
        let Value::Object(entries) = nodes_to_object(&nodes) else {
            panic!("expected object")
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "PID.3");
        assert!(matches!(&entries[0].1, Value::Array(v) if v.len() == 2));
        assert_eq!(entries[1].0, "PID.5");
        assert!(matches!(&entries[1].1, Value::String(s) if s == "SMITH"));
    }

    #[test]
    fn escapes_strings() {
        let mut out = String::new();
        write_string("a\"b\\c\nd\u{1}e", &mut out);
        assert_eq!(out, "\"a\\\"b\\\\c\\nd\\u0001e\"");
    }

    #[test]
    fn renders_null_and_empty_containers() {
        assert_eq!(render_compact(&Value::Null), "null");
        assert_eq!(render_compact(&Value::Object(vec![])), "{}");
        assert_eq!(render_compact(&Value::Array(vec![])), "[]");
    }
}
