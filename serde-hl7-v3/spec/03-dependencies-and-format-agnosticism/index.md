[index](../index.md) → §3 Dependencies and format-agnosticism

# §3 Dependencies and format-agnosticism

## 3.1 Rule S1: exactly two runtime dependencies

```toml
[dependencies]
serde = "1"
hl7-3 = { version = "0.2.0", path = "../hl7-3", default-features = false }
```

Both are the point of the crate: `serde` is the trait vocabulary being
implemented against, and `hl7-3` is the value tree being wrapped.
`default-features = false` keeps `hl7-3`'s opt-in `derive` feature off, so
no proc-macro crate enters the tree through this one; `hl7-3`'s own single
dependency, `hl7-2-xml-lite-helper`, is the whole transitive surface, and
it has none of its own. No third runtime dependency should be added without
updating this section and explaining what it buys that these two do not.

The `path` alongside `version` is the workspace convention (root
`AGENTS.md`): the path is what a local build uses, the version is what
crates.io users get, and Cargo strips the path on publish.

## 3.2 Rule S2: no format-specific crate is a runtime dependency

`serde_json`, `serde_yaml`, `bincode`, and every other format crate appear
**only** under `[dev-dependencies]`. Nothing in `src/` may call a function
from a format crate, construct a format-specific type, or reference a
format's name in a way that would break if that dev-dependency were
swapped. `examples/`, `README.md`, and `spec/` use `serde_json` to
demonstrate, because JSON is the most legible format to show in prose.

## 3.3 Why not re-export a format

This crate ships no `to_json_string`. A caller writes
`serde_json::to_string(&message)` themselves; it is one call, and it keeps
the choice visible at the call site.

## 3.4 The `hl7_3` re-export

`serde-hl7-v3` re-exports the whole `hl7-3` crate as `serde_hl7_v3::hl7_3`,
mirroring the convenience `hl7-3` itself extends for its XML reader
(`hl7_3::xml`). A caller can depend on `serde-hl7-v3` alone and still
name `hl7_3::Error`, `hl7_3::rim::Act::from_element`, and
`hl7_3::xml::parse`. The re-export is exempt from S1/S2: it exposes a
dependency that already exists.

## 3.5 Lints

`src/lib.rs` carries `#![forbid(unsafe_code)]` and
`#![warn(missing_docs, clippy::pedantic)]`, the same pair every crate in
this workspace carries. The four checks run `cargo clippy --all-targets --
-D warnings`, so a pedantic finding fails the build. Where a pedantic lint
is wrong for a line, the fix is an `#[allow]` next to that line with a
reason — not a hole in the group.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
