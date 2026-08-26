//! Loads dictionaries generated from a local NHS Wales corpus, when one is
//! present.
//!
//! The corpus is not part of this repository and is not public, so this test
//! **skips** rather than fails when the directory is absent — which is the
//! case for CI, for every contributor, and for the maintainer on a different
//! machine. It previously read the path unconditionally and panicked, which
//! passed on exactly one computer.
//!
//! Point `HL7_2_NHS_DICTIONARIES` at a directory of `<flow>/dictionary.json`
//! files to run it.

use std::fs;
use std::path::PathBuf;

/// Where the corpus lives, if it is anywhere on this machine.
fn corpus() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var("HL7_2_NHS_DICTIONARIES").ok()?);
    path.is_dir().then_some(path)
}

#[test]
fn nhs_dictionaries_load() {
    let Some(root) = corpus() else {
        eprintln!("skipping: set HL7_2_NHS_DICTIONARIES to a dictionary corpus to run this");
        return;
    };

    for flow in ["chemo", "mosaiq", "paris", "phw", "pims", "wds"] {
        let file = root.join(flow).join("dictionary.json");
        if !file.is_file() {
            eprintln!("skipping {flow}: no {}", file.display());
            continue;
        }
        let text = fs::read_to_string(&file).unwrap();
        let dictionary =
            hl7_2::Dictionary::from_json(&text, flow).unwrap_or_else(|e| panic!("{flow}: {e}"));
        let structures: Vec<_> = dictionary.structure_ids().collect();
        println!(
            "{flow}: version={:?} types={} segments={} structures={:?} PID-3={:?} card(PID,3)={:?} card(MSH,7)={:?}",
            dictionary.version(),
            dictionary.type_names().count(),
            dictionary.segment_names().count(),
            structures,
            dictionary.field_type("PID", 3),
            dictionary.field_cardinality("PID", 3),
            dictionary.field_cardinality("MSH", 7),
        );
        assert!(!structures.is_empty(), "{flow}: no structures");
    }
}
