//! Serialize the dictionary-named tree — `PID.5`, `XPN.1`, and the rest —
//! and the validation findings, the two views this crate adds over the
//! bare ER7 text.
//!
//! Run with: `cargo run -p serde-hl7-v2 --example v2_log_the_tree_as_json`
#![forbid(unsafe_code)]

use serde_hl7_v2::{Diagnostic, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260815081500||ORU^R01^ORU_R01|MSG00042|P|2.5\r\
                PID|1||444333222^^^ACME^MR||EVERYWOMAN^EVE^E||19620320|F\r\
                ZPD|local^extension\r\
                OBR|1||LAB0042|2093-3^Cholesterol^LN\r\
                OBX|1|NM|2093-3^Cholesterol^LN||one eight seven|mg/dL|<200|N|||F";

    let message = hl7_2::parse(text)?;

    // The tree: every node an object with name, path, kind, text, null,
    // and children. Serialize-only — a tree is a view of the message, and
    // the message is what round-trips (see `v2_round_trip_via_json`).
    let tree = serde_json::to_value(Node(message.tree()))?;
    let pid = tree["children"]
        .as_array()
        .and_then(|groups| groups.iter().find_map(|g| find(g, "PID")))
        .expect("PID is in the tree");
    println!("PID, as JSON:\n{}\n", serde_json::to_string_pretty(pid)?);

    // The findings: what validation thought of the message, as an array
    // of objects a log pipeline or an API response can carry as-is. These
    // round-trip, so the receiving side can read them back as diagnostics.
    let findings: Vec<Diagnostic> = message.validate().into_iter().map(Diagnostic).collect();
    let json = serde_json::to_string_pretty(&findings)?;
    println!("Validation findings, as JSON:\n{json}\n");
    let back: Vec<Diagnostic> = serde_json::from_str(&json)?;
    assert_eq!(back, findings);
    Ok(())
}

/// Depth-first search of a serialized tree for a node by name.
fn find<'a>(node: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
    if node["name"] == name {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|child| find(child, name))
}
