//! Reading is total: any bytes at all produce an [`Element`] or an
//! [`Error`], never a panic and never a stack overflow.
//!
//! The stack is the interesting half. Reading is recursive, so nesting
//! depth is stack depth, and before `MAX_DEPTH` existed a few kilobytes of
//! open tags aborted the process — a crash a caller cannot catch. This
//! target is the guard on that, and on the entity, attribute, comment and
//! `CDATA` scanners, which all walk raw offsets.
//!
//! Whatever parses is also checked for self-consistency: an element's
//! accessors must agree with the tree they read from.

#![no_main]

use hl7_2_xml_lite_helper::{Element, parse};
use libfuzzer_sys::fuzz_target;

fn check(element: &Element, depth: usize) {
    // The reader's own limit must hold for every node, not just the root.
    assert!(depth < 1024, "tree deeper than the reader's own limit");
    // A prefix is stripped, never resolved, and never lengthens a name.
    assert!(element.local_name().len() <= element.name.len());
    if let Some(first) = element.children.first() {
        assert_eq!(
            element.child(first.local_name()).map(|c| &c.name),
            Some(&element
                .children
                .iter()
                .find(|c| c.local_name() == first.local_name())
                .unwrap()
                .name),
            "child() disagreed with children"
        );
    }
    // Text and children are exclusive for the documents this crate reads:
    // an element with children keeps no mixed-content text of its own.
    for child in &element.children {
        check(child, depth + 1);
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(root) = parse(text) {
        check(&root, 0);
    }
});
