//! The specification checks itself.
//!
//! `spec/index.md` claims to be the single source of truth, and §13 backs
//! that with a table mapping every rule to the test that pins it. A table
//! like that is worth exactly as much as its accuracy: one renamed test and
//! it becomes a list of reassuring fiction. So this test reads the table and
//! checks that every test it names still exists.
//!
//! It deliberately checks names, not behavior — whether a test still tests
//! what its name says is a question for a reader, not a regex. What this
//! catches is the mechanical half: renames, deletions, and typos.

use std::collections::BTreeSet;
use std::fs;

/// Every test function defined in this crate and in `hl7-v2-derive`, named
/// the way `cargo test` prints it: `v2::message::tests::name` for a unit
/// test, bare `name` for an integration test.
fn defined_tests() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    // (directory, module path prefix for a unit test in it)
    let sources = [
        ("src", Some("")),
        ("src/v2", Some("v2::")),
        ("tests", None),
        ("../hl7-v2-derive/src", Some("")),
        ("../hl7-v2-derive/tests", None),
    ];
    for (directory, prefix) in sources {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let module = path.file_stem().unwrap().to_string_lossy().to_string();
            let text = fs::read_to_string(&path).unwrap();

            for (index, line) in text.lines().enumerate() {
                let previous = index.checked_sub(1).map(|i| text.lines().nth(i).unwrap());
                if !previous.is_some_and(|line| line.trim_start().starts_with("#[test]")) {
                    continue;
                }
                let Some(rest) = line.trim_start().strip_prefix("fn ") else {
                    continue;
                };
                let name = rest.split('(').next().unwrap().trim();
                found.insert(match prefix {
                    // `mod.rs` is the module itself, so its tests are
                    // `v2::tests::name`, not `v2::mod::tests::name`.
                    Some(prefix) if module == "mod" => format!("{prefix}tests::{name}"),
                    Some(prefix) if module == "lib" => format!("{prefix}tests::{name}"),
                    Some(prefix) => format!("{prefix}{module}::tests::{name}"),
                    None => name.to_string(),
                });
            }
        }
    }
    found
}

/// The test names §13 refers to. A name is anything in backticks that looks
/// like a Rust path of lowercase identifiers — module-qualified, which is
/// what separates a test name from the API names the same column mentions
/// (`get_all`, `set_er7`) — optionally ending in `*` to name a whole group,
/// which lets the table say `validate::tests::*` for a section every test
/// in a module covers.
fn referenced_tests(spec: &str) -> Vec<String> {
    let start = spec
        .find("## 13. Traceability")
        .expect("the spec has a traceability section");
    let end = spec[start..]
        .find("## 14.")
        .map(|offset| start + offset)
        .unwrap_or(spec.len());
    let mut names = Vec::new();
    for chunk in spec[start..end].split('`').skip(1).step_by(2) {
        for name in chunk.split(", ") {
            let name = name.trim();
            let identifier = name.trim_end_matches('*').trim_end_matches("::");
            let plausible = !identifier.is_empty()
                && identifier
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == ':');
            if plausible && identifier.contains("::") {
                names.push(name.to_string());
            }
        }
    }
    assert!(names.len() > 30, "the table should name most of the suite");
    names
}

#[test]
fn every_test_the_specification_names_exists() {
    let spec = fs::read_to_string("spec/index.md").expect("spec/index.md is where it says it is");
    let defined = defined_tests();
    let mut missing = Vec::new();

    for reference in referenced_tests(&spec) {
        // `integration::` and `derive::` in the table name the file a bare
        // test name lives in; Rust does not use them.
        let candidates = [
            reference.clone(),
            reference.replace("integration::", ""),
            reference.replace("derive::", ""),
        ];
        let found = if let Some(prefix) = reference.strip_suffix('*') {
            let prefixes: Vec<String> = candidates
                .iter()
                .filter_map(|candidate| candidate.strip_suffix('*').map(str::to_string))
                .collect();
            defined
                .iter()
                .any(|test| prefixes.iter().any(|p| test.starts_with(p)))
                || defined.iter().any(|test| test.starts_with(prefix))
        } else {
            candidates
                .iter()
                .any(|candidate| defined.contains(candidate))
        };
        if !found {
            missing.push(reference);
        }
    }

    assert!(
        missing.is_empty(),
        "spec/index.md §13 names tests that no longer exist: {missing:#?}\n\
         Either restore the test, or update the table — the spec is the source \
         of truth, so it does not get to be out of date."
    );
}

#[test]
fn every_bundled_release_the_specification_lists_is_shipped() {
    // §3.4 promises a dictionary for every release `Version` knows, whether
    // it has a file of its own or shares its base release's.
    let spec = fs::read_to_string("spec/index.md").unwrap();
    assert!(
        spec.contains("2.7.1, 2.8.1, 2.8.2"),
        "§3.4 lists the shared files"
    );
    for version in hl7_2::version::ALL {
        let dictionary = version.dictionary();
        assert!(
            dictionary.segment_fields("MSH").is_some(),
            "v{version} has no MSH"
        );
        assert!(
            spec.contains(version.as_str()),
            "spec/index.md does not mention release {version}"
        );
    }
}
