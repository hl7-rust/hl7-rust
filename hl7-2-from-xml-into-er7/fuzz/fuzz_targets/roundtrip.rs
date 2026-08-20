//! ER7 -> v2.xml -> ER7, over the pair of crates that claims to be each
//! other's inverse.
//!
//! Equality with the original is deliberately *not* asserted: the forward
//! crate drops values that are present but blank (a blank repetition, a
//! trailing empty component), which is documented and not recoverable from
//! its output (spec §5). What must hold is weaker and still strong enough
//! to catch corruption:
//!
//! 1. Neither direction panics on anything `er7::parse` accepts.
//! 2. The result is a **fixed point**: a second trip through both crates
//!    changes nothing. Anything the pair drops, it drops on the first pass;
//!    a value that keeps changing is a value being mangled.
//! 3. The explicit null and an empty value stay distinct — they instruct a
//!    receiver to do opposite things.

#![no_main]

use libfuzzer_sys::fuzz_target;

fn trip(er7_text: &str) -> Option<String> {
    let xml = hl7_2_from_er7_into_xml::convert(er7_text).ok()?;
    hl7_2_from_xml_into_er7::convert(&xml).ok()
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(message) = er7::parse(text) else {
        return;
    };
    let source = message.to_er7();
    let Some(once) = trip(&source) else {
        return;
    };
    let Some(twice) = trip(&once) else {
        panic!("a converted message no longer survives the round trip: {once:?}");
    };
    assert_eq!(once, twice, "the round trip is not a fixed point");

    // A null must never appear where the source had none: reading an empty
    // element as `""` turns a padded document into deletion instructions.
    let nulls = |text: &str| {
        er7::parse(text).map_or(0, |m| {
            m.segments
                .iter()
                .flat_map(|s| &s.fields)
                .flat_map(|f| &f.repetitions)
                .flat_map(|r| &r.components)
                .flat_map(|c| &c.subcomponents)
                .filter(|s| s.is_null())
                .count()
        })
    };
    assert!(
        nulls(&once) <= nulls(&source),
        "the round trip invented an explicit null: {source:?} -> {once:?}"
    );
});
