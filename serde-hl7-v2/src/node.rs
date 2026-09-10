//! [`Node`]: one node of `hl7-2`'s generic-mode tree, serialized as an
//! object — `Serialize` only.

use std::fmt;
use std::ops::Deref;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::NodeKind;

/// A Serde-enabled [`hl7_2::Node`]: the dictionary-named tree
/// [`hl7_2::Message::tree`] builds, as an object per node.
///
/// Serializes as an object with six fields, always all present:
///
/// | key | value | from |
/// |---|---|---|
/// | `"name"` | `"PID.5"`, `"XPN.1"`, `"ORU_R01"` | [`hl7_2::Node::name`] |
/// | `"path"` | the `er7` path, `"PID[1]-5[1].1"`; empty at the root | [`hl7_2::Node::path`] |
/// | `"kind"` | `"Group"`, `"Segment"`, `"Field"`, `"Component"`, `"Subcomponent"` | [`hl7_2::Node::kind`] via [`NodeKind`] |
/// | `"text"` | the decoded text of this node and everything beneath | [`hl7_2::Node::text`] |
/// | `"null"` | `true` when the sender wrote the explicit null `""` here | [`hl7_2::Node::is_null`] |
/// | `"children"` | an array of nodes, `[]` at a leaf | [`hl7_2::Node::children`] |
///
/// # Serialize only
///
/// There is no `Deserialize` impl. `hl7-2` builds a tree only from a
/// parsed message — [`hl7_2::Node`] has no public constructor — so the
/// only way back to a tree is through a [`crate::Message`]. That is a
/// deliberate asymmetry, stated in the spec (rule S14), not an omission: a
/// tree is a *view* of a message, and the message is what round-trips.
///
/// Example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde_hl7_v2::{Message, Node};
///
/// let message = Message::parse("MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\r\
///                               PID|1||241900||SMITH^JOHN")?;
/// let json = serde_json::to_value(Node(message.tree()))?;
///
/// assert_eq!(json["name"], "ORU_R01");
/// let pid = &json["children"][1];
/// assert_eq!(pid["name"], "PID");
/// // PID-1, PID-3, PID-5: three non-empty fields, three children.
/// let name = &pid["children"][2];
/// assert_eq!(name["name"], "PID.5");
/// assert_eq!(name["children"][0]["name"], "XPN.1");
/// assert_eq!(name["children"][0]["text"], "SMITH");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node(pub hl7_2::Node);

impl From<hl7_2::Node> for Node {
    fn from(inner: hl7_2::Node) -> Node {
        Node(inner)
    }
}

impl From<Node> for hl7_2::Node {
    fn from(outer: Node) -> hl7_2::Node {
        outer.0
    }
}

impl Deref for Node {
    type Target = hl7_2::Node;

    fn deref(&self) -> &hl7_2::Node {
        &self.0
    }
}

impl fmt::Display for Node {
    /// The node's text; see [`hl7_2::Node::text`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.text())
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        NodeRef(&self.0).serialize(serializer)
    }
}

/// A borrowed node, so serializing a tree walks it once rather than cloning
/// every subtree into an owning [`Node`] at each level.
struct NodeRef<'a>(&'a hl7_2::Node);

impl Serialize for NodeRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let children: Vec<NodeRef<'_>> = self.0.children().iter().map(NodeRef).collect();
        let mut state = serializer.serialize_struct("Node", 6)?;
        state.serialize_field("name", self.0.name())?;
        state.serialize_field("path", self.0.path())?;
        state.serialize_field("kind", &NodeKind(self.0.kind()))?;
        state.serialize_field("text", self.0.text())?;
        state.serialize_field("null", &self.0.is_null())?;
        state.serialize_field("children", &children)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "MSH|^~\\&|hphis||EPIC||20131011093851||ORU^R01|14AAACVDD|P|2.5";

    fn tree(text: &str) -> serde_json::Value {
        let message = hl7_2::parse(text).unwrap();
        serde_json::to_value(Node(message.tree())).unwrap()
    }

    fn check_six_fields(node: &serde_json::Value) {
        let object = node.as_object().unwrap();
        assert_eq!(object.len(), 6, "{object:?}");
        for key in ["name", "path", "kind", "text", "null", "children"] {
            assert!(object.contains_key(key), "missing {key}");
        }
        for child in node["children"].as_array().unwrap() {
            check_six_fields(child);
        }
    }

    #[test]
    fn serializes_six_fields_on_every_node() {
        check_six_fields(&tree(&format!("{HEADER}\rPID|1||241900||TEST^FOUAZ")));
    }

    #[test]
    fn names_paths_and_kinds_come_from_the_dictionary() {
        let json = tree(&format!("{HEADER}\rPID|1||241900||TEST^FOUAZ"));
        assert_eq!(json["name"], "ORU_R01");
        assert_eq!(json["kind"], "Group");
        assert_eq!(json["path"], "");
        let pid = &json["children"][1];
        assert_eq!(pid["name"], "PID");
        assert_eq!(pid["kind"], "Segment");
        assert_eq!(pid["path"], "PID[1]");
        let name = &pid["children"][2];
        assert_eq!(name["name"], "PID.5");
        assert_eq!(name["kind"], "Field");
        assert_eq!(name["text"], "TEST^FOUAZ");
        let given = &name["children"][1];
        assert_eq!(given["name"], "XPN.2");
        assert_eq!(given["kind"], "Component");
        assert_eq!(given["path"], "PID[1]-5[1].2");
        assert_eq!(given["text"], "FOUAZ");
        assert_eq!(given["children"], serde_json::json!([]));
    }

    #[test]
    fn the_explicit_null_survives() {
        let json = tree(&format!("{HEADER}\rPID|1||\"\""));
        let field = &json["children"][1]["children"][1];
        assert_eq!(field["name"], "PID.3");
        assert_eq!(field["null"], true);
        assert_eq!(json["children"][1]["children"][0]["null"], false);
    }

    #[test]
    fn repetitions_are_separate_siblings() {
        let json = tree(&format!("{HEADER}\rPID|1||A~B~C"));
        let pid = json["children"][1]["children"].as_array().unwrap();
        let ids: Vec<&str> = pid
            .iter()
            .filter(|n| n["name"] == "PID.3")
            .map(|n| n["text"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["A", "B", "C"]);
    }

    #[test]
    fn deref_and_display_reach_the_inner_node() {
        let message = hl7_2::parse(&format!("{HEADER}\rPID|1||241900")).unwrap();
        let node = Node(message.tree());
        assert!(node.find("PID").is_some());
        assert!(node.to_string().starts_with("MSH|"));
    }
}
