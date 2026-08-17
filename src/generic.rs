//! Generic mode: parse anything into a navigable tree.
//!
//! This is the mode for the vendor whose messages you have never seen. It
//! asks nothing of the message beyond a readable MSH header: whatever
//! segments arrive get a node, whatever the dictionary recognises gets a
//! named one, and whatever it does not gets a positional name and is still
//! there to read. Nothing is dropped and nothing is an error.
//!
//! Node names follow the same rules as the sibling conversion crates, so a
//! path through this tree reads the same as a key in
//! `hl7-v2-from-er7-into-json`'s output or an element in
//! `hl7-v2-from-er7-into-xml`'s:
//!
//! | level | known type | unknown type |
//! |---|---|---|
//! | segment | `PID` | `ZPD` |
//! | field | `PID.5` | `ZPD.2` |
//! | component | `XPN.1` (the field's type) | `ZPD.2.1` |
//! | subcomponent | `FN.1` (the component's type) | `ZPD.2.1.1` |
//!
//! Alongside the name, every node carries the `er7` path that locates it
//! ([`Node::path`]) — `PID[1]-5[1].1.2` — which is what turns "I found
//! something here" into "…and here is how to read or write it", including
//! for [`crate::Message::set`] and for validation diagnostics.

use crate::dictionary::{Dictionary, VARIABLE};
use er7::{Component, Repetition, Segment, Separators};

/// Which level of the HL7 tree a node sits at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The whole message, or a message-structure group such as
    /// `ORU_R01.ORDER_OBSERVATION`.
    Group,
    /// One segment occurrence.
    Segment,
    /// One repetition of one field.
    Field,
    /// One component of a field.
    Component,
    /// One subcomponent of a component.
    Subcomponent,
}

/// One node of the generic tree.
///
/// A node is either a container (a group, a segment, or a field/component
/// whose type the dictionary can break apart) or a leaf holding text.
/// [`Node::text`] answers for both: on a container it is the decoded text
/// of everything beneath, delimiters included, so `PID.5` reads as
/// `SMITH^JOHN` whether or not the dictionary knew what an `XPN` is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    name: String,
    path: String,
    kind: Kind,
    text: String,
    null: bool,
    children: Vec<Node>,
}

impl Node {
    /// This node's dictionary-derived name, e.g. `PID.5` or `XPN.1`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The `er7` path that locates this node in the message, e.g.
    /// `PID[1]-5[1].1.2`. Pass it to [`crate::Message::get`],
    /// [`crate::Message::set`], or `er7`'s own query API.
    ///
    /// The root node has an empty path: it is the message itself.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Which level of the tree this node sits at.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// The decoded text of this node and everything beneath it, with
    /// structural delimiters intact.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// True when the sender wrote the HL7 explicit null `""` here, meaning
    /// "clear this value" rather than "I have nothing to say". The
    /// difference matters on the way to a database, so it survives parsing.
    pub fn is_null(&self) -> bool {
        self.null
    }

    /// True when this node has no children: a value, not a container.
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// This node's children, in message order.
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// The first child named `name`.
    ///
    /// ```
    /// let message = hl7_v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1||9||SMITH^JOHN")?;
    /// let tree = message.tree();
    /// let pid = tree.find("PID").unwrap();
    /// assert_eq!(pid.child("PID.5").unwrap().child("XPN.2").unwrap().text(), "JOHN");
    /// # Ok::<(), hl7_v2::Error>(())
    /// ```
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|child| child.name == name)
    }

    /// Every child named `name`, in order. This is how repetitions are
    /// read: a field sent as `A~B` is two `PID.3` children, not one.
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |child| child.name == name)
    }

    /// The first node named `name` anywhere beneath this one, searched
    /// depth-first. Handy for reaching a segment without knowing which
    /// groups a structure nests it in.
    pub fn find(&self, name: &str) -> Option<&Node> {
        self.descendants().find(|node| node.name == name)
    }

    /// Every node named `name` anywhere beneath this one, in message order.
    pub fn find_all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.descendants().filter(move |node| node.name == name)
    }

    /// Every node beneath this one, depth-first, excluding this node.
    pub fn descendants(&self) -> Descendants<'_> {
        Descendants {
            stack: self.children.iter().rev().collect(),
        }
    }
}

/// Depth-first iterator over a node's descendants; see [`Node::descendants`].
#[derive(Debug)]
pub struct Descendants<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<&'a Node> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children.iter().rev());
        Some(node)
    }
}

/// Build the root node for a message whose segments have already been
/// arranged by [`crate::structure`], or left flat.
pub(crate) fn root(name: &str, children: Vec<Node>) -> Node {
    let text = children
        .iter()
        .map(|child| child.text.as_str())
        .collect::<Vec<&str>>()
        .join("\r");
    Node {
        name: name.to_string(),
        path: String::new(),
        kind: Kind::Group,
        text,
        null: false,
        children,
    }
}

/// Build a group node, named the way the family names groups: the message
/// structure ID, a dot, then the group name — `ORU_R01.ORDER_OBSERVATION`.
pub(crate) fn group(root_name: &str, name: &str, children: Vec<Node>) -> Node {
    let mut node = root(&format!("{root_name}.{name}"), children);
    node.path = String::new();
    node
}

/// Build the node for one segment occurrence.
///
/// `occurrence` is 1-based among segments of the same name, and becomes the
/// `[n]` in every path beneath — so a node found in the second `OBX` knows
/// it came from `OBX[2]`.
pub(crate) fn segment(
    seg: &Segment,
    occurrence: usize,
    dictionary: &Dictionary,
    separators: &Separators,
) -> Node {
    let base = format!("{}[{occurrence}]", seg.name);
    // OBX-5's data type is whatever OBX-2 says it is.
    let variable = dictionary.variable_type(seg).map(str::to_string);
    let mut children = Vec::new();
    for (index, field) in seg.fields.iter().enumerate() {
        if field.is_empty() {
            continue;
        }
        let number = index + 1;
        let name = format!("{}.{number}", seg.name);
        let data_type = match dictionary.field_type(&seg.name, number) {
            Some(VARIABLE) => variable.as_deref(),
            other => other,
        };
        for (repetition, occurrence) in field.repetitions.iter().enumerate() {
            if occurrence.is_empty() {
                continue;
            }
            children.push(field_node(
                &name,
                &format!("{base}-{number}[{}]", repetition + 1),
                data_type,
                occurrence,
                dictionary,
                separators,
            ));
        }
    }
    Node {
        name: seg.name.clone(),
        path: base,
        kind: Kind::Segment,
        text: seg.to_text(separators),
        null: false,
        children,
    }
}

/// One field repetition. Expanded into components when the dictionary knows
/// the field's type, kept whole when it does not and the value is simple,
/// and given positional names when it is neither.
fn field_node(
    name: &str,
    path: &str,
    data_type: Option<&str>,
    repetition: &Repetition,
    dictionary: &Dictionary,
    separators: &Separators,
) -> Node {
    let text = repetition.to_text(separators);
    let mut node = Node {
        name: name.to_string(),
        path: path.to_string(),
        kind: Kind::Field,
        text,
        null: repetition.is_null(),
        children: Vec::new(),
    };
    if repetition.is_null() {
        return node;
    }
    if let Some(components) = data_type.and_then(|dt| dictionary.composite_components(dt)) {
        let data_type = data_type.unwrap_or_default();
        for (index, component) in repetition.components.iter().enumerate() {
            if component.is_empty() {
                continue;
            }
            node.children.push(component_node(
                &format!("{data_type}.{}", index + 1),
                &format!("{path}.{}", index + 1),
                components.get(index).map(String::as_str),
                component,
                dictionary,
                separators,
            ));
        }
        return node;
    }
    // Unknown or primitive type. A single value stays a leaf; anything with
    // internal structure still gets nodes, named positionally.
    if let [only] = repetition.components.as_slice()
        && only.subcomponents.len() <= 1
    {
        return node;
    }
    for (index, component) in repetition.components.iter().enumerate() {
        if component.is_empty() {
            continue;
        }
        node.children.push(component_node(
            &format!("{name}.{}", index + 1),
            &format!("{path}.{}", index + 1),
            None,
            component,
            dictionary,
            separators,
        ));
    }
    node
}

/// One component. A component whose own type is composite — `XPN.1` is an
/// `FN`, `CX.4` is an `HD` — expands into subcomponents named after that
/// type; everything else is a leaf or positional.
fn component_node(
    name: &str,
    path: &str,
    data_type: Option<&str>,
    component: &Component,
    dictionary: &Dictionary,
    separators: &Separators,
) -> Node {
    let mut node = Node {
        name: name.to_string(),
        path: path.to_string(),
        kind: Kind::Component,
        text: component.to_text(separators),
        null: component.is_null(),
        children: Vec::new(),
    };
    if component.is_null() || component.subcomponents.len() <= 1 {
        return node;
    }
    let composite = data_type.filter(|dt| dictionary.is_composite(dt));
    for (index, subcomponent) in component.subcomponents.iter().enumerate() {
        if subcomponent.is_empty() {
            continue;
        }
        let number = index + 1;
        node.children.push(Node {
            name: match composite {
                Some(data_type) => format!("{data_type}.{number}"),
                None => format!("{name}.{number}"),
            },
            path: format!("{path}.{number}"),
            kind: Kind::Subcomponent,
            text: subcomponent.value(separators).into_owned(),
            null: subcomponent.is_null(),
            children: Vec::new(),
        });
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(text: &str) -> Node {
        crate::parse(text).unwrap().tree()
    }

    const HEADER: &str = "MSH|^~\\&|hphis||EPIC||20131011093851||ORU^R01|14AAACVDD|P|2.5";

    #[test]
    fn names_known_types_after_the_type_and_the_rest_positionally() {
        let tree = tree(&format!("{HEADER}\rPID|1||241900||TEST^FOUAZ\rZPD|a^b"));
        let pid = tree.find("PID").unwrap();
        let name = pid.child("PID.5").unwrap();
        assert_eq!(name.text(), "TEST^FOUAZ");
        // XPN.1 is an FN, so its first subcomponent is named FN.1.
        assert_eq!(name.child("XPN.1").unwrap().text(), "TEST");
        assert_eq!(name.child("XPN.2").unwrap().text(), "FOUAZ");
        // A Z-segment has no dictionary entry, so names stay positional and
        // nothing is lost.
        let zpd = tree.find("ZPD").unwrap();
        assert_eq!(
            zpd.child("ZPD.1").unwrap().child("ZPD.1.1").unwrap().text(),
            "a"
        );
        assert_eq!(
            zpd.child("ZPD.1").unwrap().child("ZPD.1.2").unwrap().text(),
            "b"
        );
    }

    #[test]
    fn every_node_carries_the_path_that_reads_it_back() {
        let message = crate::parse(&format!("{HEADER}\rPID|1||241900||TEST^FOUAZ")).unwrap();
        let tree = message.tree();
        let given = tree.find("XPN.2").unwrap();
        assert_eq!(given.path(), "PID[1]-5[1].2");
        assert_eq!(message.get(given.path()).unwrap().as_deref(), Some("FOUAZ"));
    }

    #[test]
    fn repetitions_are_separate_siblings() {
        let tree = tree(&format!("{HEADER}\rPID|1||A~B~C"));
        let pid = tree.find("PID").unwrap();
        let ids: Vec<&str> = pid.children_named("PID.3").map(Node::text).collect();
        assert_eq!(ids, ["A", "B", "C"]);
        assert_eq!(pid.child("PID.3").unwrap().path(), "PID[1]-3[1]");
        assert_eq!(
            pid.children_named("PID.3").nth(2).unwrap().path(),
            "PID[1]-3[3]"
        );
    }

    #[test]
    fn the_explicit_null_survives() {
        let tree = tree(&format!("{HEADER}\rPID|1||\"\""));
        let field = tree.find("PID").unwrap().child("PID.3").unwrap();
        assert!(field.is_null(), "explicit null must not read as absent");
        assert!(tree.find("PID").unwrap().child("PID.4").is_none());
    }

    #[test]
    fn obx_5_takes_its_type_from_obx_2() {
        let coded = tree(&format!("{HEADER}\rOBX|1|CE|X||a^b^c"));
        let value = coded.find("OBX").unwrap().child("OBX.5").unwrap();
        assert_eq!(value.child("CE.1").unwrap().text(), "a");
        // A primitive OBX-2 leaves OBX-5 a plain value.
        let numeric = tree(&format!("{HEADER}\rOBX|1|NM|X||7.4"));
        assert_eq!(
            numeric.find("OBX").unwrap().child("OBX.5").unwrap().text(),
            "7.4"
        );
    }

    #[test]
    fn groups_nest_under_the_structure_id() {
        let tree = tree(&format!("{HEADER}\rPID|1\rOBR|1\rOBX|1|NM|X||7"));
        assert_eq!(tree.name(), "ORU_R01");
        let result = tree.child("ORU_R01.PATIENT_RESULT").unwrap();
        let order = result.child("ORU_R01.ORDER_OBSERVATION").unwrap();
        assert!(
            order
                .child("ORU_R01.OBSERVATION")
                .unwrap()
                .child("OBX")
                .is_some()
        );
        // ... and find() reaches through them without knowing the nesting.
        assert!(tree.find("OBX").is_some());
    }

    #[test]
    fn descendants_walks_everything_once() {
        let tree = tree(&format!("{HEADER}\rPID|1||A~B"));
        let count = tree.descendants().count();
        let named = tree.find_all("PID.3").count();
        assert_eq!(named, 2);
        assert!(count > named);
    }
}
