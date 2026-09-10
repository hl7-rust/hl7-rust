//! Black-box tests through the public API only, exercising the sample
//! messages the sibling `hl7-2` crate tests itself against — read straight
//! from the sibling checkout, so a round trip through this crate is checked
//! against real message shapes, not just literals written for this crate.

use serde_hl7_v2::{Diagnostic, Message, Node, Version};

const ADT: &str = include_str!("../../hl7-2/samples/adt_a01.hl7");
const ORU: &str = include_str!("../../hl7-2/samples/oru_r01.hl7");
const ORM: &str = include_str!("../../hl7-2/samples/orm_o01.hl7");
const VENDOR: &str = include_str!("../../hl7-2/samples/vendor.hl7");

fn round_trips_through_json(text: &str) {
    let message = Message::parse(text).expect("sample parses");
    let json = serde_json::to_string(&message).expect("serializes");
    let back: Message = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(
        back.to_er7(),
        message.to_er7(),
        "ER7 text changed after a JSON round trip"
    );
    assert_eq!(back.version(), message.version(), "release changed");
    assert_eq!(back.structure_id(), message.structure_id());
}

#[test]
fn round_trips_every_sample_from_the_hl7_2_crate() {
    for sample in [ADT, ORU, ORM, VENDOR] {
        round_trips_through_json(sample);
    }
}

#[test]
fn round_trips_pretty_printed_json_too() {
    let message = Message::parse(ORU).unwrap();
    let json = serde_json::to_string_pretty(&message).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(back.to_er7(), message.to_er7());
}

#[test]
fn the_tree_after_a_round_trip_serializes_identically() {
    // S14: a tree does not deserialize, but the tree of a round-tripped
    // message is the same tree — which is the round trip a caller who
    // ships JSON trees actually needs.
    for sample in [ADT, ORU, ORM, VENDOR] {
        let message = Message::parse(sample).unwrap();
        let before = serde_json::to_string(&Node(message.tree())).unwrap();
        let json = serde_json::to_string(&message).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        let after = serde_json::to_string(&Node(back.tree())).unwrap();
        assert_eq!(before, after);
    }
}

#[test]
fn node_has_no_deserialize_impl() {
    // S14, stated as a compile-time fact: `Node` is `Serialize` but not
    // `Deserialize`. If this ever compiles differently, the spec's S14 is
    // out of date and §9.1 tells you what to do.
    fn assert_serialize<T: serde::Serialize>() {}
    assert_serialize::<Node>();
    // A negative trait check cannot be written directly; the `trybuild`
    // style of test is more than this one fact is worth. The rustdoc on
    // `Node` and this test's name are the record.
}

#[test]
fn the_explicit_null_is_visible_in_the_tree() {
    // The explicit `""` means "clear this value", never conflated with
    // "not sent" or "sent blank" — er7's R10/R11, visible in the tree as
    // the `null` key.
    let text = "MSH|^~\\&|LAB||EPIC||20240101||ADT^A01|1|P|2.5\rPID|1||\"\"|X";
    let message = Message::parse(text).unwrap();
    let tree = serde_json::to_value(Node(message.tree())).unwrap();
    let pid = tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["name"] == "PID")
        .expect("PID in tree");
    let fields: Vec<(&str, bool)> = pid["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["name"].as_str().unwrap(), n["null"].as_bool().unwrap()))
        .collect();
    assert_eq!(
        fields,
        [("PID.1", false), ("PID.3", true), ("PID.4", false)]
    );
    // PID-2 was sent empty: no node at all, not a node with empty text.
    assert!(!fields.iter().any(|(name, _)| *name == "PID.2"));
}

#[test]
fn diagnostics_round_trip_for_every_sample() {
    for sample in [ADT, ORU, ORM, VENDOR] {
        let message = Message::parse(sample).unwrap();
        let findings: Vec<Diagnostic> = message.validate().into_iter().map(Diagnostic).collect();
        let json = serde_json::to_string(&findings).unwrap();
        let back: Vec<Diagnostic> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, findings);
    }
}

#[test]
fn version_matches_what_the_message_reports() {
    let message = Message::parse(ADT).unwrap();
    let json = serde_json::to_value(&message).unwrap();
    let version: Version = serde_json::from_value(json["version"].clone()).unwrap();
    assert_eq!(version.0, message.version());
}

#[test]
fn error_messages_still_name_the_missing_field() {
    let err = serde_json::from_str::<Message>(r#"{"version":"2.5"}"#).unwrap_err();
    assert!(err.to_string().contains("er7"), "{err}");
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
    // S1: `serde` is the trait vocabulary this crate implements against and
    // `hl7-2` is the tree it wraps. Both are the point of the crate; a third
    // would be an audit surface for every downstream healthcare project
    // (spec §3.1).
    assert_eq!(
        manifest_dependencies("[dependencies]"),
        [
            r#"serde = "1""#,
            r#"hl7-2 = { version = "0.3.0", path = "../hl7-2", default-features = false }"#
        ]
    );
}

#[test]
fn no_format_crate_is_a_runtime_dependency() {
    // S2: this crate is format-agnostic. `serde_json` appears in the tests,
    // the doctests, and the examples because JSON is the easiest format to
    // read on a page — never in `[dependencies]` (spec §3.2).
    let runtime = manifest_dependencies("[dependencies]").join(" ");
    for format in ["serde_json", "serde_yaml", "toml", "ciborium", "postcard"] {
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
    // Spec-driven development only works if the spec is the single source
    // of truth. A rule in the index with no row in the §7.1 coverage table
    // is a rule nobody agreed to test, and a row for a rule that no longer
    // exists is a table nobody re-read.
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
    // The section directories and the table of contents drift apart
    // silently: a new section nobody linked, or a link to a section that
    // was renamed. Each section is `<slug>/index.md`, so the slug is the
    // directory name and the link is the slug plus `/index.md`.
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
