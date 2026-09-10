//! Black-box tests through the public API only: one realistic interaction
//! parsed from XML and pushed through every type in the crate, the
//! guarantees in spec §4, and the manifest and spec self-checks.

use serde_hl7_v3::{
    Act, ActRelationship, Cd, ControlAct, Ed, Element, Entity, Ii, Ivl, Message, NullFlavor,
    Participation, Pq, Role, RoleLink,
};

/// A patient-registry query, the kind of interaction v3 was actually
/// deployed for. Synthetic throughout.
const QUERY: &str = r#"
<PRPA_IN201305UV02 xmlns="urn:hl7-org:v3">
  <id root="2.16.840.1.113883.19.5" extension="MSG00042"/>
  <creationTime value="20260815081500"/>
  <interactionId root="2.16.840.1.113883.1.6" extension="PRPA_IN201305UV02"/>
  <processingCode code="P"/>
  <sender typeCode="SND">
    <device classCode="DEV" determinerCode="INSTANCE"><id root="2.16.840.1.113883.19.5.1"/></device>
  </sender>
  <receiver typeCode="RCV">
    <device classCode="DEV" determinerCode="INSTANCE"><id root="2.16.840.1.113883.19.5.2"/></device>
  </receiver>
  <controlActProcess classCode="CACT" moodCode="EVN">
    <code code="PRPA_TE201305UV02" codeSystem="2.16.840.1.113883.1.6"/>
    <subject>
      <patient classCode="PAT">
        <id root="2.16.840.1.113883.19.5" extension="444333222"/>
        <statusCode code="active"/>
        <effectiveTime value="20260101"/>
        <patientPerson classCode="PSN" determinerCode="INSTANCE">
          <name nullFlavor="MSK"/>
        </patientPerson>
      </patient>
    </subject>
  </controlActProcess>
</PRPA_IN201305UV02>"#;

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let json = serde_json::to_string(value).expect("serializes");
    serde_json::from_str(&json).expect("deserializes")
}

#[test]
fn the_sample_message_serializes_to_the_documented_shape() {
    let message = Message::parse(QUERY).unwrap();
    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["id"]["root"], "2.16.840.1.113883.19.5");
    assert_eq!(json["id"]["extension"], "MSG00042");
    assert_eq!(json["creationTime"], "20260815081500");
    assert_eq!(json["interactionId"]["extension"], "PRPA_IN201305UV02");
    assert_eq!(json["sender"]["name"], "device");
    assert_eq!(json["sender"]["attributes"]["classCode"], "DEV");
    assert_eq!(json["sender"]["children"][0]["name"], "id");
    assert_eq!(json["controlAct"]["classCode"], "CACT");
    assert_eq!(
        json["controlAct"]["code"]["codeSystem"],
        "2.16.840.1.113883.1.6"
    );
    let domain = &json["controlAct"]["domain"];
    assert_eq!(domain["name"], "patient");
    assert_eq!(
        domain["children"][3]["children"][0]["attributes"]["nullFlavor"],
        "MSK"
    );
}

#[test]
fn every_type_round_trips_through_pretty_json() {
    // S14. Pretty-printing changes only whitespace between tokens.
    let message = Message::parse(QUERY).unwrap();
    let json = serde_json::to_string_pretty(&message).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(back, message);

    let domain = message
        .control_act
        .as_ref()
        .unwrap()
        .domain
        .as_ref()
        .unwrap();
    let element = Element(domain.clone());
    assert_eq!(round_trip(&element), element);

    let role = Role(hl7_3::rim::Role::from_element(domain));
    assert_eq!(role.class_code, "PAT");
    assert_eq!(round_trip(&role), role);
    let act = Act(hl7_3::rim::Act::from_element(domain));
    assert_eq!(round_trip(&act), act);
    let person = Entity(hl7_3::rim::Entity::from_element(
        domain.child("patientPerson").unwrap(),
    ));
    assert_eq!(person.class_code, "PSN");
    assert_eq!(round_trip(&person), person);

    let participation = Participation(hl7_3::rim::Participation {
        type_code: "AUT".into(),
        time: Some("20260101".into()),
        function_code: Some(hl7_3::Cd {
            code: "PCP".into(),
            code_system: None,
            display_name: Some("primary care".into()),
        }),
    });
    assert_eq!(round_trip(&participation), participation);
    let relationship = ActRelationship(hl7_3::rim::ActRelationship {
        type_code: "COMP".into(),
        inversion_ind: Some(true),
    });
    assert_eq!(round_trip(&relationship), relationship);
    let link = RoleLink(hl7_3::rim::RoleLink {
        type_code: "DIRAUTH".into(),
    });
    assert_eq!(round_trip(&link), link);

    let control_act = ControlAct(message.control_act.clone().unwrap());
    assert_eq!(round_trip(&control_act), control_act);
    let ii = Ii(message.id.clone().unwrap());
    assert_eq!(round_trip(&ii), ii);
    let cd = Cd(control_act.code.clone().unwrap());
    assert_eq!(round_trip(&cd), cd);
    let ivl = Ivl(hl7_3::Ivl {
        value: None,
        low: Some("20260101".into()),
        high: Some("20261231".into()),
    });
    assert_eq!(round_trip(&ivl), ivl);
    let pq = Pq(hl7_3::Pq {
        value: Some("5".into()),
        unit: Some("mg".into()),
    });
    assert_eq!(round_trip(&pq), pq);
    let ed = Ed(hl7_3::Ed {
        media_type: Some("text/plain".into()),
        representation: None,
        text: Some("note".into()),
    });
    assert_eq!(round_trip(&ed), ed);
    let flavor = NullFlavor(hl7_3::NullFlavor::of(domain.find("name").unwrap()).unwrap());
    assert_eq!(flavor.0, hl7_3::NullFlavor::Unrecognized("MSK".into()));
    assert_eq!(round_trip(&flavor), flavor);
}

#[test]
fn absent_and_empty_stay_distinct() {
    // §4.4: `None` is `null`, `Some("")` is `""`, never merged.
    let none = Ii(hl7_3::Ii {
        root: "1".into(),
        extension: None,
    });
    let empty = Ii(hl7_3::Ii {
        root: "1".into(),
        extension: Some(String::new()),
    });
    assert_eq!(
        serde_json::to_string(&none).unwrap(),
        r#"{"root":"1","extension":null}"#
    );
    assert_eq!(
        serde_json::to_string(&empty).unwrap(),
        r#"{"root":"1","extension":""}"#
    );
    assert_eq!(round_trip(&none), none);
    assert_eq!(round_trip(&empty), empty);
    assert_ne!(none, empty);
}

#[test]
fn error_messages_still_name_the_missing_field() {
    let err = serde_json::from_str::<Ii>(r#"{"extension":"7"}"#).unwrap_err();
    assert!(err.to_string().contains("root"), "{err}");
    let err = serde_json::from_str::<Element>(r#"{"attributes":{}}"#).unwrap_err();
    assert!(err.to_string().contains("name"), "{err}");
}

#[test]
fn every_object_wrapper_is_written_by_the_macro() {
    // S15: fourteen object types, fourteen `object_wrapper!` invocations,
    // and no hand-written `impl Serialize` for a struct anywhere in src/.
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut invocations = 0;
    let mut hand_written_struct_impls = 0;
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name == "object.rs" {
            continue;
        }
        invocations += text.matches("object_wrapper! {").count();
        if name != "null_flavor.rs" && name != "strict.rs" {
            hand_written_struct_impls += text.matches("impl Serialize for").count()
                + text.matches("impl<'de> Deserialize<'de> for").count();
        }
    }
    assert_eq!(invocations, 14, "one macro invocation per object type");
    assert_eq!(
        hand_written_struct_impls, 0,
        "no hand-written struct impls beside the macro"
    );
}

/// The `[dependencies]` and `[dev-dependencies]` tables of this crate's
/// own manifest, as raw lines.
fn manifest_dependencies(table: &str) -> Vec<String> {
    let manifest = include_str!("../Cargo.toml");
    let mut inside = false;
    let mut lines = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == table;
            continue;
        }
        if !inside || line.is_empty() || line.starts_with('#') {
            continue;
        }
        lines.push(line.to_string());
    }
    lines
}

#[test]
fn the_crate_has_exactly_two_runtime_dependencies() {
    // S1 (spec §3.1).
    assert_eq!(
        manifest_dependencies("[dependencies]"),
        [
            r#"serde = "1""#,
            r#"hl7-3 = { version = "0.2.0", path = "../hl7-3", default-features = false }"#
        ]
    );
}

#[test]
fn no_format_crate_is_a_runtime_dependency() {
    // S2 (spec §3.2).
    let runtime = manifest_dependencies("[dependencies]").join(" ");
    for format in [
        "serde_json",
        "serde_yaml",
        "toml",
        "ciborium",
        "postcard",
        "quick-xml",
    ] {
        assert!(
            !runtime.contains(format),
            "{format} is a runtime dependency"
        );
    }
    assert!(
        manifest_dependencies("[dev-dependencies]")
            .iter()
            .any(|line| line.starts_with("serde_json")),
        "serde_json belongs in dev-dependencies, where the tests use it"
    );
}

/// Every `| S<n> |` cell at the start of a table row in `text`.
fn rule_ids(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("| S"))
        .filter_map(|rest| rest.split_once(" |"))
        .map(|(number, _)| format!("S{number}"))
        .filter(|id| id[1..].chars().all(|c| c.is_ascii_digit()))
        .collect()
}

#[test]
fn every_rule_has_a_coverage_row() {
    let declared = rule_ids(include_str!("../spec/index.md"));
    let covered = rule_ids(include_str!("../spec/07-testing-strategy/index.md"));

    assert_eq!(declared.len(), 15, "the rule index should hold S1–S15");
    let missing: Vec<&String> = declared.iter().filter(|s| !covered.contains(s)).collect();
    assert!(missing.is_empty(), "no row in §7.1 for {missing:?}");
    let orphan: Vec<&String> = covered.iter().filter(|s| !declared.contains(s)).collect();
    assert!(
        orphan.is_empty(),
        "§7.1 covers {orphan:?}, which the rule index does not declare"
    );
}

#[test]
fn every_spec_section_is_indexed_and_present() {
    let index = include_str!("../spec/index.md");
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");

    let mut linked: Vec<String> = index
        .match_indices("](")
        .filter_map(|(at, _)| index[at + 2..].split_once(')').map(|(name, _)| name))
        .filter_map(|name| name.strip_suffix("/index.md"))
        .filter(|slug| slug.len() > 3 && slug.as_bytes()[2] == b'-' && !slug.contains('/'))
        .map(str::to_string)
        .collect();
    linked.sort();
    linked.dedup();

    let mut on_disk: Vec<String> = std::fs::read_dir(&directory)
        .expect("spec directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|slug| slug.as_bytes()[0].is_ascii_digit())
        .collect();
    on_disk.sort();

    assert_eq!(linked, on_disk, "spec/index.md and spec/ disagree");
}
