//! Loading a schema-mode dictionary from JSON is total, and the structures
//! it builds stay within the JSON reader's own depth bound.
//!
//! [`Dictionary::from_json`] is schema mode's public entry point (spec §3):
//! a vendor writes their dialect as JSON and loads it at runtime, so this
//! is exactly the untrusted structured input `SECURITY.md` names as an open
//! gap — this target closes it. Two properties:
//!
//! 1. **Total.** Any bytes produce a [`Dictionary`] or an [`Error`], never
//!    a panic and never a stack overflow.
//! 2. **Bounded.** `structures` nests [`Item::Group`] the same way JSON
//!    objects nest, so the JSON reader's own `MAX_DEPTH` (256, in
//!    `src/json.rs`) already bounds it — checked here rather than assumed.
//!
//! Every lookup a caller could make against a successfully-loaded
//! dictionary is exercised too: the three "list the names, then look one
//! up" pairs (`segment_names`/`segment_fields`, `type_names`/
//! `is_composite`+`composite_components`, `structure_ids`/`structure`) must
//! agree with each other, and `field_type` must never leak the internal
//! "unstated" sentinel (an empty string, left by a sparse delta) as if it
//! were a real data type.

#![no_main]

use hl7_2::Dictionary;
use hl7_2::dictionary::Item;
use libfuzzer_sys::fuzz_target;

fn walk(item: &Item, depth: usize) {
    assert!(
        depth <= 256,
        "a structure nested past the JSON reader's own depth limit"
    );
    // Not asserted true or false — `can_start` recurses through a group's
    // own items, so calling it is the exercise, not the name it is called
    // with.
    let _ = item.can_start(item.name());
    if let Item::Group { items, .. } = item {
        for child in items {
            walk(child, depth + 1);
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(dictionary) = Dictionary::from_json(text, "fuzz") else {
        return;
    };

    for segment in dictionary.segment_names() {
        let fields = dictionary
            .segment_fields(segment)
            .expect("segment_names() named a segment segment_fields() does not know");
        for field in 1..=fields.len() {
            if let Some(kind) = dictionary.field_type(segment, field) {
                assert!(
                    !kind.is_empty(),
                    "field_type() leaked the unstated sentinel as a data type"
                );
            }
        }
    }

    for data_type in dictionary.type_names() {
        assert!(
            dictionary.is_composite(data_type),
            "type_names() named a type is_composite() denies"
        );
        assert!(
            dictionary.composite_components(data_type).is_some(),
            "type_names() named a type composite_components() denies"
        );
    }

    for id in dictionary.structure_ids() {
        let items = dictionary
            .structure(id)
            .expect("structure_ids() named a structure structure() does not know");
        for item in items {
            walk(item, 0);
        }
    }
});
