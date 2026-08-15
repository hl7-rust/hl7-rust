//! Building the v2.xml element tree from parsed segments, and serializing it.

use crate::types::{composite_components, field_type};
use er7::{Component, Repetition, Segment, Separators};

/// XML namespace of the root element in every document this crate emits.
pub const V2XML_NAMESPACE: &str = "urn:hl7-org:v2xml";

/// One element of the output XML tree. Exactly one of `text` and `kids` is
/// meaningful at a time: a node with children is rendered as a container
/// element, a childless node with `text` as a text-bearing leaf, and a
/// childless node without `text` as a self-closing empty element (used for
/// the HL7 explicit null `""`).
#[derive(Debug, Clone)]
pub struct Node {
    /// XML element name, already sanitized by [`xml_name`].
    pub name: String,
    /// Element text content, when this is a childless, non-null leaf.
    pub text: Option<String>,
    /// Child elements, when this is a container.
    pub kids: Vec<Node>,
}

impl Node {
    /// A container node with no children yet (a group, segment, or group
    /// element such as `<ORM_O01.PATIENT>`).
    pub fn group(name: impl Into<String>) -> Self {
        Node {
            name: name.into(),
            text: None,
            kids: Vec::new(),
        }
    }

    /// A leaf node holding `value`. The HL7 explicit null (`""`) and the
    /// empty string both produce an element with no text (`<FOO/>`) rather
    /// than an element containing the literal two-character text `""`.
    pub fn leaf(name: impl Into<String>, value: &str) -> Self {
        // The HL7 explicit null `""` becomes an empty element.
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

/// Keep only characters that are safe in an XML element name.
pub fn xml_name(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
        .collect();
    if cleaned
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    {
        cleaned
    } else {
        format!("X{cleaned}")
    }
}

/// Convert one segment into its `<SEG>` element with typed field children.
///
/// `separators` is the delimiter set the message declared; it is needed
/// because `er7` stores subcomponent text exactly as it arrived and decodes
/// escape sequences on demand, so decoding happens here, at the point the
/// text becomes XML.
pub fn segment_to_node(seg: &Segment, separators: &Separators) -> Node {
    let seg_name = xml_name(&seg.name);
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

/// One field repetition -> one field element.
fn repeat_node(name: &str, dt: Option<&str>, rep: &Repetition, separators: &Separators) -> Node {
    // Explicit null for the whole field: empty field element.
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

/// Serialize a document with XML declaration; the root element carries the
/// v2.xml namespace.
pub fn render_document(root: &Node) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    write_node(root, 0, true, &mut out);
    out
}

fn write_node(node: &Node, depth: usize, is_root: bool, out: &mut String) {
    let pad = "  ".repeat(depth);
    let attrs = if is_root {
        format!(" xmlns=\"{V2XML_NAMESPACE}\"")
    } else {
        String::new()
    };
    if !node.kids.is_empty() {
        out.push_str(&format!("{pad}<{}{}>\n", node.name, attrs));
        for kid in &node.kids {
            write_node(kid, depth + 1, false, out);
        }
        out.push_str(&format!("{pad}</{}>\n", node.name));
    } else if let Some(text) = &node.text {
        out.push_str(&format!(
            "{pad}<{name}{attrs}>{}</{name}>\n",
            escape_xml(text),
            name = node.name,
        ));
    } else {
        out.push_str(&format!("{pad}<{}{}/>\n", node.name, attrs));
    }
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_names() {
        assert_eq!(xml_name("PID"), "PID");
        assert_eq!(xml_name("Z<S>1"), "ZS1");
        assert_eq!(xml_name("2DX"), "X2DX");
    }

    #[test]
    fn escapes_text() {
        assert_eq!(escape_xml("a&b<c>d"), "a&amp;b&lt;c&gt;d");
    }
}
