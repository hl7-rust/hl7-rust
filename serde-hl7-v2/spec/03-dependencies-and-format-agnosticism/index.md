[index](../index.md) → §3 Dependencies and format-agnosticism

# §3 Dependencies and format-agnosticism

## 3.1 Rule S1: exactly two runtime dependencies

```toml
[dependencies]
serde = "1"
hl7-2 = { version = "0.3.0", path = "../hl7-2", default-features = false }
```

Both are the point of the crate: `serde` is the trait vocabulary being
implemented against, and `hl7-2` is the value tree being wrapped. Neither
can be dropped without dropping the crate's purpose. `default-features =
false` keeps `hl7-2`'s opt-in `derive` feature off, so no proc-macro crate
enters the tree through this one; `hl7-2`'s own single dependency, `er7`,
is the whole transitive surface. No third runtime dependency should be
added without updating this section and explaining, here, what it buys
that these two do not.

The `path` alongside `version` is the workspace convention (root
`AGENTS.md`): the path is what a local build uses, the version is what
crates.io users get, and Cargo strips the path on publish.

## 3.2 Rule S2: no format-specific crate is a runtime dependency

`serde_json`, `serde_yaml`, `bincode`, and every other format crate appear
**only** under `[dev-dependencies]`, for tests, doctests, and examples.
This is what "Serde support" means as opposed to "JSON support": the whole
value of building against `serde::Serializer`/`Deserializer` rather than
writing an HL7®-to-JSON function directly — which the workspace already has,
in `hl7-2-from-er7-into-json` — is that the format is the caller's choice,
made once, in their own `Cargo.toml`.

A consequence: nothing in `src/` may call a function from a format crate,
construct a format-specific type, or reference a format's name in a way
that would break if that dev-dependency were swapped for a different one.
`examples/`, `README.md`, and `spec/` may use `serde_json` freely to
demonstrate, because JSON is simply the most legible format to show in
prose — that is a documentation choice, not an API commitment.

## 3.3 Why not re-export a format

This crate ships no `to_json_string`. A crate that did would have picked
JSON, regardless of what its `Cargo.toml` says. A caller who wants that
convenience writes `serde_json::to_string(&message)` themselves; it is one
call, and it keeps the choice visible at the call site.

## 3.4 The `hl7_2` re-export

`serde-hl7-v2` re-exports the whole `hl7-2` crate as `serde_hl7_v2::hl7_2`,
mirroring the convenience `hl7-2` itself extends for `er7`. This lets a
caller depend on `serde-hl7-v2` alone and still name `hl7_2::Options`,
`hl7_2::Error`, and `hl7_2::er7::Message` without a second, separately
versioned dependency. The re-export is exempt from S1/S2: it does not add a
dependency, it exposes one that already exists.

## 3.5 Lints

`src/lib.rs` carries `#![forbid(unsafe_code)]` and
`#![warn(missing_docs, clippy::pedantic)]`, the same pair every crate in
this workspace carries. The four checks run `cargo clippy --all-targets --
-D warnings` ([§7.4](../07-testing-strategy/index.md)), so a pedantic
finding fails the build. `forbid` rather than `deny` for `unsafe`, so no
`#[allow]` further down can reopen it. Where a pedantic lint is wrong for
a line, the fix is an `#[allow]` next to that line with a reason — not a
hole in the group.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
