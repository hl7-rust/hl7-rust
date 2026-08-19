//! Building the v2.xml element tree from parsed segments, and serializing it.

use er7::{Component, Field, Repetition, Segment, Separators, Subcomponent};
use hl7_2::Dictionary;
use hl7_2::dictionary::VARIABLE;
use std::fmt::Write as _;

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
#[must_use]
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
/// `dictionary` supplies the field data types and, when `schema_shape` is
/// set, what the schema says about how often each field may appear.
/// `separators` is the delimiter set the message declared; it is needed
/// because `er7` stores subcomponent text exactly as it arrived and decodes
/// escape sequences on demand, so decoding happens here, at the point the
/// text becomes XML.
pub fn segment_to_node(
    seg: &Segment,
    separators: &Separators,
    dictionary: &Dictionary,
    schema_shape: bool,
) -> Node {
    let seg_name = xml_name(&seg.name);
    let mut node = Node::group(&seg_name);
    // OBX-5 has a variable type declared by the value of OBX-2.
    let variable = dictionary.variable_type(seg).map(str::to_string);

    // In schema mode the dictionary decides which positions exist, so that a
    // required-but-empty field still gets an element and nothing outside the
    // schema appears. Otherwise the message decides, as it always has.
    let declared = dictionary.segment_fields(&seg.name).map(<[String]>::len);
    let count = match (schema_shape, declared) {
        (true, Some(declared)) => declared,
        _ => seg.fields.len(),
    };

    for index in 0..count {
        let n = index + 1;
        let name = format!("{seg_name}.{n}");
        let data_type = match dictionary.field_type(&seg.name, n) {
            Some(VARIABLE) => variable.as_deref(),
            other => other,
        };
        let field = seg.fields.get(index);
        let cardinality = dictionary.field_cardinality(&seg.name, n);

        let present = |field: &&Field| {
            if schema_shape {
                !written_text(field, separators).is_empty()
            } else {
                !field.is_empty()
            }
        };

        let Some(field) = field.filter(present) else {
            // A field the schema requires is written even when the message
            // leaves it empty, so its position stays visible to a validator,
            // and it is written as the full tree its data type declares.
            if schema_shape && cardinality.required {
                node.kids
                    .push(declared_empty(&name, data_type, dictionary, 0));
            }
            continue;
        };

        // A field the schema does not let repeat keeps its repetition
        // separator as ordinary text: splitting it would emit more elements
        // than the schema allows, and the document would stop validating.
        if schema_shape && !cardinality.repeats && field.repetitions.len() > 1 {
            let joined = flatten(field, separators);
            node.kids.push(repeat_node(
                &name,
                data_type,
                &joined,
                separators,
                dictionary,
                schema_shape,
            ));
            continue;
        }

        for rep in &field.repetitions {
            // A repeating field that arrived as `~900001` has an empty first
            // repetition, and in schema mode that position is written too:
            // the schema counts elements, and the sender said there were two.
            if rep.is_empty() && !schema_shape {
                continue;
            }
            node.kids.push(repeat_node(
                &name,
                data_type,
                rep,
                separators,
                dictionary,
                schema_shape,
            ));
        }
    }
    node
}

/// What the sender actually wrote in a field, for deciding whether the
/// position is occupied at all.
///
/// Trailing separators are not content: a `PID-9` of `^^` is three empty
/// components, which is the same as saying nothing, and the schema has no
/// reason to carry an element for it. Two *repetitions* are different — a
/// `PID-13` of `^^^~` says there are two of them, and the schema counts
/// elements — so the repetition separator survives the trim and keeps the
/// field occupied.
fn written_text(field: &Field, separators: &Separators) -> String {
    field
        .repetitions
        .iter()
        .map(|repetition| {
            repetition
                .to_er7(separators)
                .trim_end_matches([separators.component, separators.subcomponent])
                .to_string()
        })
        .collect::<Vec<String>>()
        .join(&separators.repetition.to_string())
}

/// Collapse a field's repetitions into the single repetition a
/// non-repeating field is read as.
///
/// The field's own ER7 text is re-split on the component and subcomponent
/// separators, so a `PID-13` of `A^B~C^D` under a schema that allows one
/// repetition reads as the three components `A`, `B~C`, `D` — the
/// repetition separator having become ordinary text in the second one.
/// Escapes are left encoded here exactly as they arrive; the leaf writers
/// decode them.
fn flatten(field: &Field, separators: &Separators) -> Repetition {
    let text = field.to_er7(separators);
    Repetition {
        components: text
            .split(separators.component)
            .map(|component| Component {
                subcomponents: component
                    .split(separators.subcomponent)
                    .map(Subcomponent::new)
                    .collect(),
            })
            .collect(),
    }
}

/// One field repetition -> one field element.
fn repeat_node(
    name: &str,
    dt: Option<&str>,
    rep: &Repetition,
    separators: &Separators,
    dictionary: &Dictionary,
    schema_shape: bool,
) -> Node {
    // Explicit null for the whole field: empty field element.
    if rep.is_null() {
        return Node::group(name);
    }
    if let Some(comps) = dt.and_then(|dt| dictionary.composite_components(dt)) {
        // Known composite type: children named after the type's components.
        let dt = dt.unwrap();
        let mut node = Node::group(name);
        // In schema mode the type decides how many components there are, so
        // every one it declares is written and anything past the end of the
        // declaration is dropped; otherwise the value decides.
        let count = if schema_shape {
            comps.len()
        } else {
            rep.components.len()
        };
        for ci in 0..count {
            let cname = format!("{}.{}", dt, ci + 1);
            let cdt = comps.get(ci).map(String::as_str);
            match rep.components.get(ci) {
                Some(comp) if !comp.is_empty() => node.kids.push(component_node(
                    &cname,
                    cdt,
                    comp,
                    separators,
                    dictionary,
                    schema_shape,
                )),
                // A declared component the value does not reach is written
                // as the empty tree its own type declares, because the
                // schema requires the elements to be there.
                _ if schema_shape => node.kids.push(declared_empty(&cname, cdt, dictionary, 0)),
                _ => {}
            }
        }
        return node;
    }
    if schema_shape {
        // The schema says this element has no children, so whatever the
        // sender put there is its text — separators included. Inventing
        // positional children would put elements in the document that the
        // schema has no declaration for.
        return Node::leaf(name, &rep.to_text(separators));
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
    dictionary: &Dictionary,
    schema_shape: bool,
) -> Node {
    if let Some(cdt) = cdt.filter(|t| dictionary.is_composite(t)) {
        // Composite component: subcomponents named after its own components,
        // e.g. XPN.1 containing FN.1, or CX.4 containing HD.1/HD.2/HD.3.
        let subtypes = dictionary.composite_components(cdt).unwrap_or_default();
        let count = if schema_shape {
            subtypes.len()
        } else {
            comp.subcomponents.len()
        };
        let mut node = Node::group(cname);
        for si in 0..count {
            let sname = format!("{}.{}", cdt, si + 1);
            match comp.subcomponents.get(si) {
                Some(sub) if !sub.is_empty() => {
                    node.kids.push(Node::leaf(sname, &sub.value(separators)));
                }
                _ if schema_shape => node.kids.push(declared_empty(
                    &sname,
                    subtypes.get(si).map(String::as_str),
                    dictionary,
                    0,
                )),
                _ => {}
            }
        }
        return node;
    }
    if let [sub] = comp.subcomponents.as_slice() {
        return Node::leaf(cname, &sub.value(separators));
    }
    generic_component_node(cname, comp, separators)
}

/// The tree a data type declares, with no values in it.
///
/// A schema that says `<xsd:element ref="HD.1"/>` with no `minOccurs`
/// requires the element, so a composite the message never reaches still has
/// to appear in full — `<CX.4><HD.1/><HD.2/><HD.3/></CX.4>` rather than
/// `<CX.4/>`. A primitive is a childless element, which renders self-closing.
///
/// `depth` guards against a dictionary whose types contain one another; a
/// well-formed one nests only a few levels.
fn declared_empty(name: &str, dt: Option<&str>, dictionary: &Dictionary, depth: usize) -> Node {
    const MAX_DEPTH: usize = 8;
    let mut node = Node::group(name);
    if depth >= MAX_DEPTH {
        return node;
    }
    let Some(dt) = dt else {
        return node;
    };
    let Some(components) = dictionary.composite_components(dt) else {
        return node;
    };
    for (index, component) in components.iter().enumerate() {
        node.kids.push(declared_empty(
            &format!("{}.{}", dt, index + 1),
            Some(component),
            dictionary,
            depth + 1,
        ));
    }
    node
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
#[must_use]
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
        let _ = writeln!(out, "{pad}<{}{}>", node.name, attrs);
        for kid in &node.kids {
            write_node(kid, depth + 1, false, out);
        }
        let _ = writeln!(out, "{pad}</{}>", node.name);
    } else if let Some(text) = &node.text {
        let _ = writeln!(
            out,
            "{pad}<{name}{attrs}>{}</{name}>",
            escape_xml(text),
            name = node.name,
        );
    } else {
        let _ = writeln!(out, "{pad}<{}{}/>", node.name, attrs);
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
