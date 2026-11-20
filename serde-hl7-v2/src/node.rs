//! [`Node`]: one node of `hl7-2`'s generic-mode tree, serialized as an
//! object that round-trips as a detached tree snapshot.

use std::fmt;
use std::ops::Deref;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{NodeKind, Strict};

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
/// Deserializing restores all six fields recursively as detached
/// [`hl7_2::Node`] snapshots. A restored node is not attached to a
/// [`crate::Message`], just as a node returned by `Message::tree` is a value
/// rather than a mutable handle into that message.
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

const FIELDS: &[&str] = &["name", "path", "kind", "text", "null", "children"];

#[derive(Clone, Copy)]
struct NodeSeed {
    strict: bool,
}

impl<'de> DeserializeSeed<'de> for NodeSeed {
    type Value = Node;

    fn deserialize<D>(self, deserializer: D) -> Result<Node, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "Node",
            FIELDS,
            NodeVisitor {
                strict: self.strict,
            },
        )
    }
}

struct NodeVisitor {
    strict: bool,
}

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a Node object with \"name\", \"path\", \"kind\", \"text\", \"null\", and \"children\"",
        )
    }

    fn visit_map<V>(self, mut map: V) -> Result<Node, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut name: Option<String> = None;
        let mut path: Option<String> = None;
        let mut kind: Option<NodeKind> = None;
        let mut text: Option<String> = None;
        let mut null: Option<bool> = None;
        let mut children: Option<Vec<Node>> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "name" => read_once(&mut name, "name", &mut map)?,
                "path" => read_once(&mut path, "path", &mut map)?,
                "kind" => read_once(&mut kind, "kind", &mut map)?,
                "text" => read_once(&mut text, "text", &mut map)?,
                "null" => read_once(&mut null, "null", &mut map)?,
                "children" => {
                    if children.is_some() {
                        return Err(de::Error::duplicate_field("children"));
                    }
                    children = Some(map.next_value_seed(ChildrenSeed {
                        strict: self.strict,
                    })?);
                }
                _ if self.strict => return Err(de::Error::unknown_field(&key, FIELDS)),
                _ => {
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        let children = children
            .ok_or_else(|| de::Error::missing_field("children"))?
            .into_iter()
            .map(|child| child.0)
            .collect();
        Ok(Node(hl7_2::Node::new(
            name.ok_or_else(|| de::Error::missing_field("name"))?,
            path.ok_or_else(|| de::Error::missing_field("path"))?,
            kind.ok_or_else(|| de::Error::missing_field("kind"))?.0,
            text.ok_or_else(|| de::Error::missing_field("text"))?,
            null.ok_or_else(|| de::Error::missing_field("null"))?,
            children,
        )))
    }
}

fn read_once<'de, T, V>(
    slot: &mut Option<T>,
    field: &'static str,
    map: &mut V,
) -> Result<(), V::Error>
where
    T: Deserialize<'de>,
    V: MapAccess<'de>,
{
    if slot.is_some() {
        return Err(de::Error::duplicate_field(field));
    }
    *slot = Some(map.next_value()?);
    Ok(())
}

#[derive(Clone, Copy)]
struct ChildrenSeed {
    strict: bool,
}

impl<'de> DeserializeSeed<'de> for ChildrenSeed {
    type Value = Vec<Node>;

    fn deserialize<D>(self, deserializer: D) -> Result<Vec<Node>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ChildrenVisitor {
            strict: self.strict,
        })
    }
}

struct ChildrenVisitor {
    strict: bool,
}

impl<'de> Visitor<'de> for ChildrenVisitor {
    type Value = Vec<Node>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an array of Node objects")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Vec<Node>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut children = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(child) = sequence.next_element_seed(NodeSeed {
            strict: self.strict,
        })? {
            children.push(child);
        }
        Ok(children)
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        NodeSeed { strict: false }.deserialize(deserializer)
    }
}

impl<'de> Deserialize<'de> for Strict<Node> {
    /// See [`Strict`] (rule S13): unknown fields are rejected recursively.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        NodeSeed { strict: true }
            .deserialize(deserializer)
            .map(Strict)
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
        let node = Node::from(message.tree());
        assert!(node.find("PID").is_some());
        assert!(node.to_string().starts_with("MSH|"));
        let inner: hl7_2::Node = node.into();
        assert!(inner.find("PID").is_some());
    }

    #[test]
    fn rejects_a_missing_node_field() {
        for field in FIELDS {
            let mut value = serde_json::json!({
                "name": "PID.5",
                "path": "PID[1]-5[1]",
                "kind": "Field",
                "text": "X",
                "null": false,
                "children": [],
            });
            value.as_object_mut().unwrap().remove(*field);
            let error = serde_json::from_value::<Node>(value).unwrap_err();
            assert!(error.to_string().contains(field), "{field}: {error}");
        }
    }

    #[test]
    fn rejects_every_duplicate_node_field() {
        let duplicates = [
            (
                "name",
                r#"{"name":"A","name":"B","path":"","kind":"Field","text":"X","null":false,"children":[]}"#,
            ),
            (
                "path",
                r#"{"name":"A","path":"","path":"X","kind":"Field","text":"X","null":false,"children":[]}"#,
            ),
            (
                "kind",
                r#"{"name":"A","path":"","kind":"Field","kind":"Group","text":"X","null":false,"children":[]}"#,
            ),
            (
                "text",
                r#"{"name":"A","path":"","kind":"Field","text":"X","text":"Y","null":false,"children":[]}"#,
            ),
            (
                "null",
                r#"{"name":"A","path":"","kind":"Field","text":"X","null":false,"null":true,"children":[]}"#,
            ),
            (
                "children",
                r#"{"name":"A","path":"","kind":"Field","text":"X","null":false,"children":[],"children":[]}"#,
            ),
        ];
        for (field, json) in duplicates {
            let error = serde_json::from_str::<Node>(json).unwrap_err();
            assert!(error.to_string().contains(field), "{field}: {error}");
        }
    }

    #[test]
    fn rejects_wrong_node_container_shapes() {
        let error = serde_json::from_str::<Node>("[]").unwrap_err();
        assert!(error.to_string().contains("Node object"), "{error}");

        let json = r#"{"name":"A","path":"","kind":"Group","text":"","null":false,"children":{}}"#;
        let error = serde_json::from_str::<Node>(json).unwrap_err();
        assert!(error.to_string().contains("array of Node"), "{error}");
    }

    #[test]
    fn ignores_an_unknown_node_field_by_default() {
        let json = r#"{"name":"PID.5","path":"PID[1]-5[1]","kind":"Field","text":"X","null":false,"children":[],"future":1}"#;
        let node = serde_json::from_str::<Node>(json).unwrap();
        assert_eq!(node.text(), "X");
    }

    #[test]
    fn strict_node_rejects_an_unknown_field_at_any_depth() {
        let json = r#"{"name":"ROOT","path":"","kind":"Group","text":"X","null":false,"children":[{"name":"PID.5","path":"PID[1]-5[1]","kind":"Field","text":"X","null":false,"children":[],"future":1}]}"#;
        let error = serde_json::from_str::<Strict<Node>>(json).unwrap_err();
        assert!(error.to_string().contains("future"), "{error}");
    }
}
