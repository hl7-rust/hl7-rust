// temporary: verify the generated NHS dictionaries load
use std::fs;
#[test]
fn nhs_dictionaries_load() {
    let root = "/Users/jph/github/dhcw-digital-health-and-care-wales/Integration-Hub-Beta/shared_libs/hl7_validation/hl7_validation/resources";
    for flow in ["chemo", "mosaiq", "paris", "phw", "pims", "wds"] {
        let text = fs::read_to_string(format!("{root}/{flow}/dictionary.json")).unwrap();
        let d =
            hl7_v2::Dictionary::from_json(&text, flow).unwrap_or_else(|e| panic!("{flow}: {e}"));
        let structures: Vec<_> = d.structure_ids().collect();
        println!(
            "{flow}: version={:?} types={} segments={} structures={:?} PID-3={:?} card(PID,3)={:?} card(MSH,7)={:?}",
            d.version(),
            d.type_names().count(),
            d.segment_names().count(),
            structures,
            d.field_type("PID", 3),
            d.field_cardinality("PID", 3),
            d.field_cardinality("MSH", 7),
        );
        assert!(!structures.is_empty());
    }
}
