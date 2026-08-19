//! Reading HL7 v2.xml XML Schema documents into dictionary tables.
//!
//! The v2.xml encoding is published as a set of schemas, and a site that
//! customises HL7 customises these files. Three of them describe everything
//! every message shares, and one more describes each message structure:
//!
//! ```text
//! <prefix>_types.xsd       composite data types and their components
//! <prefix>_fields.xsd      every SEG.n element and the data type it carries
//! <prefix>_segments.xsd    each segment's field list, with cardinality
//! ADT_A05.xsd, ...         one abstract message structure each
//! ```
//!
//! What each contributes, and the indirection that has to be followed, is
//! in `spec/index.md` §3.

use crate::dictionary::{Field, Item};
use crate::xml::Element;
use std::collections::BTreeMap;

/// The suffix HL7's schemas give the complexType carrying an element's
/// content: `PID.CONTENT` for the `PID` element.
pub const CONTENT: &str = ".CONTENT";

/// What the dictionary writes for a position the schema leaves unstated.
const UNSTATED: &str = "";

/// A reference from a sequence, with the cardinality the schema gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// The `ref` attribute's value.
    pub name: String,
    /// `minOccurs >= 1`. An absent `minOccurs` is 1, per XML Schema, which
    /// is what makes `<xsd:element ref="MSH"/>` required.
    pub required: bool,
    /// `maxOccurs` is `unbounded` or greater than one. An absent
    /// `maxOccurs` is 1.
    pub repeats: bool,
}

/// The `ref`s of a complexType's sequence, with their cardinality.
#[must_use]
pub fn sequence_references(complex_type: &Element) -> Vec<Reference> {
    let Some(sequence) = complex_type.child("sequence") else {
        return Vec::new();
    };
    sequence
        .children_named("element")
        .filter_map(|element| {
            Some(Reference {
                name: element.attribute("ref")?.to_string(),
                required: is_required(element),
                repeats: repeats(element),
            })
        })
        .collect()
}

fn is_required(element: &Element) -> bool {
    match element.attribute("minOccurs") {
        None => true,
        Some(raw) => raw.parse::<u64>().map_or(true, |n| n >= 1),
    }
}

fn repeats(element: &Element) -> bool {
    match element.attribute("maxOccurs") {
        None => false,
        Some("unbounded") => true,
        Some(raw) => raw.parse::<u64>().is_ok_and(|n| n > 1),
    }
}

/// The three lookups that turn an element name into a data type name.
///
/// `element_type` maps `XPN.1` to the complexType carrying its content
/// (`XPN.1.CONTENT`); `children` maps a composite type to the elements it is
/// made of; `base` maps a content type to the type it extends. Resolving a
/// data type means following `base` until a type with children — a composite
/// — or one with neither — a primitive — is reached.
#[derive(Debug, Default)]
pub struct Types {
    element_type: BTreeMap<String, String>,
    children: BTreeMap<String, Vec<String>>,
    base: BTreeMap<String, String>,
}

impl Types {
    /// Add one schema's element, composite, and extension declarations.
    ///
    /// Both `complexContent` and `simpleContent` extensions are read: the
    /// first is how a composite component names its own composite type
    /// (`XPN.1.CONTENT` extends `FN`), the second is how a leaf names its
    /// primitive (`HD.1.CONTENT` extends `IS`).
    pub fn absorb(&mut self, root: &Element) {
        for element in root.children_named("element") {
            if let (Some(name), Some(type_name)) =
                (element.attribute("name"), element.attribute("type"))
            {
                self.element_type
                    .insert(name.to_string(), type_name.to_string());
            }
        }
        for complex_type in root.children_named("complexType") {
            let Some(name) = complex_type.attribute("name") else {
                continue;
            };
            let references = sequence_references(complex_type);
            if !references.is_empty() {
                self.children.insert(
                    name.to_string(),
                    references.into_iter().map(|r| r.name).collect(),
                );
            }
            for wrapper in ["complexContent", "simpleContent"] {
                if let Some(base) = complex_type
                    .child(wrapper)
                    .and_then(|content| content.child("extension"))
                    .and_then(|extension| extension.attribute("base"))
                {
                    self.base.insert(name.to_string(), base.to_string());
                }
            }
        }
    }

    /// The data type of an element, following `.CONTENT` wrappers through.
    ///
    /// `MSH.3` is declared `type="HD"` and resolves straight to `HD`.
    /// `XPN.1` is declared `type="XPN.1.CONTENT"`, whose `complexContent`
    /// extends `FN`, so it resolves to `FN`. The walk stops at a type with
    /// children of its own, and guards against extensions that cycle.
    #[must_use]
    pub fn data_type(&self, element_name: &str) -> &str {
        let Some(mut current) = self.element_type.get(element_name) else {
            return UNSTATED;
        };
        let mut seen: Vec<&str> = Vec::new();
        while !self.children.contains_key(current) {
            let Some(next) = self.base.get(current) else {
                break;
            };
            if seen.contains(&current.as_str()) {
                break;
            }
            seen.push(current);
            current = next;
        }
        current
    }

    /// Composite data types mapped to their component data types.
    ///
    /// Only real data type names are emitted. An HL7 data type is a short
    /// alphanumeric name such as `XPN` or `CX`, never a dotted one, so the
    /// `PID.CONTENT` and `XPN.1.CONTENT` entries that share this table are
    /// filtered out here.
    #[must_use]
    pub fn composites(&self) -> BTreeMap<String, Vec<String>> {
        self.children
            .iter()
            .filter(|(name, _)| !name.contains('.'))
            .map(|(name, refs)| {
                (
                    name.clone(),
                    refs.iter().map(|r| self.data_type(r).to_string()).collect(),
                )
            })
            .collect()
    }
}

/// Every segment's field list, read from a `<prefix>_segments.xsd` root.
///
/// A segment whose sequence is empty contributes nothing: `Hxx.CONTENT` is
/// HL7's placeholder for an arbitrary Z-segment, and a dictionary that does
/// not mention a segment already means "unknown", which is what a wildcard
/// is.
#[must_use]
pub fn segments(root: &Element, types: &Types) -> BTreeMap<String, Vec<Field>> {
    let mut out = BTreeMap::new();
    for complex_type in root.children_named("complexType") {
        let Some(name) = complex_type.attribute("name").and_then(strip_content) else {
            continue;
        };
        if name.contains('.') {
            continue;
        }
        let mut fields: Vec<Field> = Vec::new();
        for reference in sequence_references(complex_type) {
            let Some(position) = field_position(&reference.name) else {
                continue;
            };
            if fields.len() < position {
                fields.resize(position, Field::default());
            }
            fields[position - 1] = Field {
                data_type: types.data_type(&reference.name).to_string(),
                required: reference.required,
                repeats: reference.repeats,
            };
        }
        if !fields.is_empty() {
            out.insert(name.to_string(), fields);
        }
    }
    out
}

/// The 1-based field number in a `SEG.n` reference.
fn field_position(reference: &str) -> Option<usize> {
    reference
        .rsplit_once('.')
        .and_then(|(_, tail)| tail.parse::<usize>().ok())
        .filter(|position| *position > 0)
}

fn strip_content(name: &str) -> Option<&str> {
    name.strip_suffix(CONTENT)
}

/// One message structure, read from its own schema.
///
/// The root complexType is `<ID>.CONTENT`; a group is a reference to
/// another element in the same file that has a `.CONTENT` type of its own,
/// such as `ADT_A05.PROCEDURE`. Groups are named in the dictionary without
/// the structure prefix, matching how the reading crates name group nodes:
/// the structure ID, a dot, then the group name.
#[must_use]
pub fn structure(root: &Element, structure_id: &str) -> Option<Vec<Item>> {
    let contents: BTreeMap<&str, &Element> = root
        .children_named("complexType")
        .filter_map(|complex_type| {
            let name = complex_type.attribute("name").and_then(strip_content)?;
            Some((name, complex_type))
        })
        .collect();
    contents.get(structure_id)?;
    Some(items_of(
        structure_id,
        &contents,
        structure_id,
        &mut Vec::new(),
    ))
}

fn items_of(
    owner: &str,
    contents: &BTreeMap<&str, &Element>,
    structure_id: &str,
    open: &mut Vec<String>,
) -> Vec<Item> {
    let Some(complex_type) = contents.get(owner) else {
        return Vec::new();
    };
    open.push(owner.to_string());
    let items = sequence_references(complex_type)
        .into_iter()
        .map(|reference| {
            let is_group =
                contents.contains_key(reference.name.as_str()) && !open.contains(&reference.name);
            if is_group {
                let group = reference
                    .name
                    .strip_prefix(&format!("{structure_id}."))
                    .unwrap_or(&reference.name)
                    .to_string();
                Item::Group {
                    name: group,
                    required: reference.required,
                    repeats: reference.repeats,
                    items: items_of(&reference.name, contents, structure_id, open),
                }
            } else {
                Item::Segment {
                    name: reference.name,
                    required: reference.required,
                    repeats: reference.repeats,
                }
            }
        })
        .collect();
    open.pop();
    items
}

/// The base-file prefix a structure schema includes, e.g. `2_5_1`.
///
/// Every structure schema includes `<prefix>_segments.xsd`, so any one of
/// them names the prefix. Reading it from the schemas rather than from the
/// directory name lets a directory be named for the sending system rather
/// than for the HL7 release.
#[must_use]
pub fn included_prefix(root: &Element) -> Option<String> {
    root.children_named("include")
        .filter_map(|include| include.attribute("schemaLocation"))
        .find_map(|location| {
            let file = location.rsplit(['/', '\\']).next().unwrap_or(location);
            file.strip_suffix("_segments.xsd").map(str::to_string)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::parse;

    const TYPES: &str = r#"
      <xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema">
        <xsd:simpleType name="ST"><xsd:restriction base="xsd:string"/></xsd:simpleType>
        <xsd:complexType name="HD">
          <xsd:sequence>
            <xsd:element ref="HD.1"/><xsd:element ref="HD.2"/>
          </xsd:sequence>
        </xsd:complexType>
        <xsd:complexType name="HD.1.CONTENT">
          <xsd:simpleContent><xsd:extension base="IS"/></xsd:simpleContent>
        </xsd:complexType>
        <xsd:element name="HD.1" type="HD.1.CONTENT"/>
        <xsd:complexType name="HD.2.CONTENT">
          <xsd:simpleContent><xsd:extension base="ST"/></xsd:simpleContent>
        </xsd:complexType>
        <xsd:element name="HD.2" type="HD.2.CONTENT"/>
        <xsd:complexType name="XPN">
          <xsd:sequence><xsd:element ref="XPN.1"/></xsd:sequence>
        </xsd:complexType>
        <xsd:complexType name="XPN.1.CONTENT">
          <xsd:complexContent><xsd:extension base="HD"/></xsd:complexContent>
        </xsd:complexType>
        <xsd:element name="XPN.1" type="XPN.1.CONTENT"/>
      </xsd:schema>"#;

    fn types() -> Types {
        let mut types = Types::default();
        types.absorb(&parse(TYPES).unwrap());
        types
    }

    #[test]
    fn resolves_a_primitive_through_its_content_wrapper() {
        assert_eq!(types().data_type("HD.1"), "IS");
        assert_eq!(types().data_type("HD.2"), "ST");
    }

    #[test]
    fn resolves_a_composite_component_to_its_own_type() {
        assert_eq!(types().data_type("XPN.1"), "HD");
    }

    #[test]
    fn an_unknown_element_has_no_data_type() {
        assert_eq!(types().data_type("ZZZ.9"), "");
    }

    #[test]
    fn composites_leave_out_the_dotted_helper_types() {
        let composites = types().composites();
        assert_eq!(
            composites.get("HD"),
            Some(&vec!["IS".to_string(), "ST".to_string()])
        );
        assert_eq!(composites.get("XPN"), Some(&vec!["HD".to_string()]));
        assert!(!composites.contains_key("HD.1.CONTENT"));
    }

    #[test]
    fn reads_cardinality_the_way_xml_schema_defines_it() {
        let root = parse(
            r#"<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                 <xsd:complexType name="PID.CONTENT"><xsd:sequence>
                   <xsd:element ref="PID.1"/>
                   <xsd:element minOccurs="0" ref="PID.2"/>
                   <xsd:element minOccurs="0" maxOccurs="unbounded" ref="PID.3"/>
                   <xsd:element minOccurs="0" maxOccurs="10" ref="PID.4"/>
                   <xsd:element minOccurs="0" maxOccurs="1" ref="PID.5"/>
                 </xsd:sequence></xsd:complexType>
               </xsd:schema>"#,
        )
        .unwrap();
        let fields = segments(&root, &types());
        let pid = &fields["PID"];
        assert!(pid[0].required && !pid[0].repeats); // absent minOccurs is 1
        assert!(!pid[1].required && !pid[1].repeats);
        assert!(pid[2].repeats); // unbounded
        assert!(pid[3].repeats); // a bound above one still repeats
        assert!(!pid[4].repeats); // a bound of one does not
    }

    #[test]
    fn a_gap_in_the_field_numbers_is_left_unstated() {
        let root = parse(
            r#"<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                 <xsd:complexType name="PID.CONTENT"><xsd:sequence>
                   <xsd:element ref="PID.3"/>
                 </xsd:sequence></xsd:complexType>
               </xsd:schema>"#,
        )
        .unwrap();
        let fields = segments(&root, &types());
        assert_eq!(fields["PID"].len(), 3);
        assert_eq!(fields["PID"][0], Field::default());
        assert_eq!(fields["PID"][2].data_type, "");
    }

    #[test]
    fn a_segment_with_no_fields_is_left_out_entirely() {
        let root = parse(
            r#"<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                 <xsd:complexType name="Hxx.CONTENT"><xsd:sequence/></xsd:complexType>
               </xsd:schema>"#,
        )
        .unwrap();
        assert!(segments(&root, &types()).is_empty());
    }

    #[test]
    fn reads_a_structure_and_names_groups_without_the_prefix() {
        let root = parse(
            r#"<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                 <xsd:include schemaLocation="2_5_1_segments.xsd"/>
                 <xsd:complexType name="ADT_A05.CONTENT"><xsd:sequence>
                   <xsd:element ref="MSH"/>
                   <xsd:element minOccurs="0" maxOccurs="unbounded" ref="ADT_A05.PROCEDURE"/>
                 </xsd:sequence></xsd:complexType>
                 <xsd:element name="ADT_A05" type="ADT_A05.CONTENT"/>
                 <xsd:complexType name="ADT_A05.PROCEDURE.CONTENT"><xsd:sequence>
                   <xsd:element ref="PR1"/>
                   <xsd:element minOccurs="0" maxOccurs="unbounded" ref="ROL"/>
                 </xsd:sequence></xsd:complexType>
               </xsd:schema>"#,
        )
        .unwrap();
        assert_eq!(included_prefix(&root).as_deref(), Some("2_5_1"));
        let items = structure(&root, "ADT_A05").unwrap();
        assert_eq!(
            items[0],
            Item::Segment {
                name: "MSH".into(),
                required: true,
                repeats: false
            }
        );
        match &items[1] {
            Item::Group {
                name,
                required,
                repeats,
                items,
            } => {
                assert_eq!(name, "PROCEDURE");
                assert!(!required && *repeats);
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Item::Segment { name, .. } if name == "PR1"));
            }
            segment @ Item::Segment { .. } => panic!("expected a group, got {segment:?}"),
        }
    }

    #[test]
    fn a_structure_the_file_does_not_declare_is_none() {
        let root = parse(r#"<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema"/>"#).unwrap();
        assert!(structure(&root, "ADT_A05").is_none());
    }
}
