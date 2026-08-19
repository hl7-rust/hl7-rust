//! Tests for the macro, run against the real `hl7-3` crate.
//!
//! A proc macro can only really be tested by compiling code that uses it,
//! so these are integration tests rather than unit tests: each one defines
//! a struct, derives, and checks what the generated code does.

use hl7_3::rim::Act;
use hl7_3::xml::Element;
use hl7_3_derive::FromElement as DeriveFromElement;

const XML: &str = r#"
<observation classCode="OBS" moodCode="EVN">
  <id root="2.16.840.1.113883.19.5" extension="1"/>
  <note>elevated</note>
  <component classCode="ACT" moodCode="RQO"/>
</observation>
"#;

#[derive(Debug, Default, DeriveFromElement)]
struct Observation {
    #[element("classCode")]
    class_code: String,
    #[element("moodCode")]
    mood_code: String,
    #[element(child = "note")]
    note: Option<String>,
    #[element(nested = "component")]
    component: Act,
    #[element(raw)]
    raw: Element,
    // No attribute: not read from the element at all.
    processed: bool,
}

#[test]
fn reads_each_annotated_field_from_the_element() {
    use hl7_3::typed::FromElement;
    let element = hl7_3::xml::parse(XML).unwrap();
    let observation = Observation::from_element(&element);
    assert_eq!(observation.class_code, "OBS");
    assert_eq!(observation.mood_code, "EVN");
    assert_eq!(observation.note.as_deref(), Some("elevated"));
    assert_eq!(observation.component.class_code, "ACT");
    assert_eq!(observation.component.mood_code, "RQO");
    assert!(!observation.processed, "an unannotated field is defaulted");
    assert_eq!(observation.raw.local_name(), "observation");
    // The escape hatch reaches what the struct does not model.
    assert_eq!(
        observation
            .raw
            .child("id")
            .and_then(|id| id.attribute("extension")),
        Some("1")
    );
}

#[derive(Debug, Default, DeriveFromElement)]
struct Required {
    #[element("classCode")]
    class_code: String,
}

#[test]
fn a_missing_attribute_degrades_to_the_default_not_an_error() {
    use hl7_3::typed::FromElement;
    let element = hl7_3::xml::parse("<empty/>").unwrap();
    let required = Required::from_element(&element);
    assert_eq!(required.class_code, "");
}

#[derive(Debug, Default, DeriveFromElement)]
struct Generic<T>
where
    T: hl7_3::typed::FromElementValue,
{
    #[element("classCode")]
    value: T,
}

#[test]
fn generic_structs_carry_their_bounds_through() {
    use hl7_3::typed::FromElement;
    let element = hl7_3::xml::parse(r#"<act classCode="42"/>"#).unwrap();
    let typed: Generic<u32> = Generic::from_element(&element);
    assert_eq!(typed.value, 42);
    let text: Generic<String> = Generic::from_element(&element);
    assert_eq!(text.value, "42");
}
