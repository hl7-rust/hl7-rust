//! End-to-end: the sample schemas in `samples/example/` in, a dictionary
//! out, and that dictionary loaded by the crate that has to read it.

use hl7_v2_from_xsd_into_json_dictionary::{Item, Options, convert_directory};
use std::path::PathBuf;

fn samples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/example")
}

fn document() -> hl7_v2_from_xsd_into_json_dictionary::Document {
    convert_directory(&samples(), &Options::default()).expect("the sample schemas convert")
}

#[test]
fn reads_the_release_from_the_base_file_prefix() {
    assert_eq!(document().version.as_deref(), Some("2.5"));
}

#[test]
fn a_composite_lists_its_component_data_types() {
    let document = document();
    // Primitives resolve through their `.CONTENT` simpleContent extension.
    assert_eq!(
        document.types.get("HD").map(Vec::as_slice),
        Some(["IS".to_string(), "ST".to_string(), "ID".to_string()].as_slice())
    );
    // A composite component resolves through complexContent to its own type.
    assert_eq!(
        document.types.get("CX").map(Vec::as_slice),
        Some(["ST".to_string(), "HD".to_string()].as_slice())
    );
    // The dotted helper types are not data types and are not listed.
    assert!(!document.types.contains_key("HD.1.CONTENT"));
    assert!(!document.types.contains_key("PID.CONTENT"));
}

#[test]
fn a_segment_carries_its_field_types_and_cardinality() {
    let document = document();
    // Positions are field numbers, not declaration order: the sample
    // declares PID.1, PID.3 and PID.5, so index 1 (PID-2) stays unstated.
    let pid = &document.segments["PID"];
    assert_eq!(pid.len(), 5);
    assert_eq!(pid[0].data_type, "SI");
    assert!(!pid[0].required && !pid[0].repeats);
    assert_eq!(pid[1].data_type, "");
    assert_eq!(pid[2].data_type, "CX");
    assert!(pid[2].required, "an absent minOccurs is 1");
    assert!(pid[2].repeats, "maxOccurs=unbounded");
    assert_eq!(pid[4].data_type, "ST");

    // A field number the schema skips is left unstated rather than guessed.
    let msh = &document.segments["MSH"];
    assert_eq!(msh.len(), 10);
    assert_eq!(msh[3].data_type, ""); // MSH.4 is not declared
    assert_eq!(msh[6].data_type, "TS"); // MSH.7 is
}

#[test]
fn a_segment_with_no_declared_fields_is_left_out() {
    // Hxx is HL7's arbitrary-Z-segment placeholder: an empty sequence.
    assert!(!document().segments.contains_key("Hxx"));
}

#[test]
fn a_structure_keeps_its_groups_and_their_cardinality() {
    let document = document();
    let items = &document.structures["ADT_A39"];
    assert!(
        matches!(&items[0], Item::Segment { name, required, .. } if name == "MSH" && *required)
    );
    match &items[2] {
        Item::Group {
            name,
            required,
            repeats,
            items,
        } => {
            assert_eq!(name, "PATIENT", "the structure prefix is stripped");
            assert!(*required && *repeats);
            assert!(matches!(&items[0], Item::Segment { name, .. } if name == "PID"));
            assert!(matches!(&items[1], Item::Segment { name, .. } if name == "MRG"));
        }
        segment @ Item::Segment { .. } => panic!("expected a group, got {segment:?}"),
    }
}

#[test]
fn options_add_what_the_schemas_cannot_say() {
    let mut options = Options {
        name: Some("example".into()),
        inherits: Some("2.5".into()),
        version: Some("2.5.1".into()),
        ..Default::default()
    };
    options.aliases.insert("ADT_A40".into(), "ADT_A39".into());
    let document = convert_directory(&samples(), &options).unwrap();
    assert_eq!(document.version.as_deref(), Some("2.5.1"));
    assert_eq!(document.inherits.as_deref(), Some("2.5"));
    assert_eq!(document.aliases["ADT_A40"], "ADT_A39");
    assert!(document.description.unwrap().starts_with("example: "));
}

#[test]
fn converting_only_some_structures() {
    let mut options = Options {
        structures: vec!["ADT_A39".into()],
        ..Default::default()
    };
    assert_eq!(
        convert_directory(&samples(), &options)
            .unwrap()
            .structures
            .len(),
        1
    );

    options.structures = vec!["ADT_A01".into()];
    let error = convert_directory(&samples(), &options).unwrap_err();
    assert!(error.to_string().contains("ADT_A01"), "{error}");
}

/// The whole point: what this crate writes, `hl7-2` reads.
#[test]
fn the_dictionary_loads_in_the_crate_that_consumes_it() {
    let json = document().to_json();
    let dictionary = hl7_2::Dictionary::from_json(&json, "example")
        .unwrap_or_else(|error| panic!("generated dictionary did not load: {error}"));

    assert_eq!(dictionary.version(), Some("2.5"));
    assert_eq!(dictionary.field_type("PID", 3), Some("CX"));
    assert_eq!(
        dictionary.composite_components("CX").map(<[String]>::len),
        Some(2)
    );

    // Cardinality survives the round trip, which is what schema mode needs.
    let cardinality = dictionary.field_cardinality("PID", 3);
    assert!(cardinality.required && cardinality.repeats);
    assert_eq!(
        dictionary.field_cardinality("PID", 1),
        hl7_2::dictionary::Cardinality::default()
    );

    // And so does the structure, groups included.
    let items = dictionary.structure("ADT_A39").expect("ADT_A39 is defined");
    assert_eq!(items[2].name(), "PATIENT");
    assert!(items[2].can_start("PID"));
}

#[test]
fn reports_a_directory_it_cannot_read_as_a_dictionary() {
    let error =
        convert_directory(&PathBuf::from("no/such/directory"), &Options::default()).unwrap_err();
    assert!(error.to_string().contains("no/such/directory"), "{error}");
}
