# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

A thin umbrella crate: `src/lib.rs` and nothing else, whose entire content
is `pub use serde_hl7_v2 as v2;` and `pub use serde_hl7_v3 as v3;`, each
behind a Cargo feature of the same name (both on by default), plus its own
doc comment. It exists so a caller can `cargo add serde-hl7` and get
`serde_hl7::v2` and `serde_hl7::v3` instead of depending on
`serde-hl7-v2` or `serde-hl7-v3` by name — one module per HL7® standard,
mirroring the `hl7` umbrella's `hl7::v2` and `hl7::v3`.

## Layout

```
src/lib.rs   The whole crate: two feature-gated `pub use` lines and the
             crate-level doc comment (which is also the one doctest this
             crate has).
```

There is no `spec/index.md` here — this crate is too thin to need one.
`serde-hl7-v2`'s spec covers `serde_hl7::v2`, and `serde-hl7-v3`'s covers
`serde_hl7::v3`; this crate only re-exports them. `plan.md` and
`tasks.md` alongside this file record the evaluation that led to all three
crates and the checklist their first release followed.

## Working conventions

- **This crate carries no logic of its own.** Every wrapper, every impl,
  every rule lives in the per-standard crate. If a change here does
  anything beyond adding or wiring up a `pub use`, it belongs in
  `serde-hl7-v2` or `serde-hl7-v3` instead.
- **The features stay one-to-one with the modules.** `v2` enables
  `serde-hl7-v2` and exposes `serde_hl7::v2`; `v3` likewise. Both are
  default. A build with neither compiles to an empty crate, and that is
  fine.
- **No tests of its own** beyond the doctest in the crate-level doc
  comment — behavior gets tested where it lives.
- **Adding a standard** (`serde_hl7::fhir`, say, when `hl7::fhir` exists)
  means adding a new sibling crate with its own spec, tests, and examples,
  then one feature and one `pub use` line here.
- Every public item must have a doc comment; `src/lib.rs` carries
  `#![warn(missing_docs, clippy::pedantic)]` and `#![forbid(unsafe_code)]`.
- Before finishing a change, from the workspace root:
  ```sh
  cargo test -p serde-hl7 --all-features
  cargo check -p serde-hl7 --no-default-features --features v2
  cargo check -p serde-hl7 --no-default-features --features v3
  cargo clippy -p serde-hl7 --all-targets --all-features -- -D warnings
  cargo fmt --check
  cargo rustdoc -p serde-hl7 --lib -- -W missing-docs
  ```

## Non-goals (don't "fix" these without discussion)

- Implementing Serde behavior directly in this crate.
- Flattening the `v2`/`v3` modules away. A `Message` in one standard is
  not a `Message` in the other; the per-standard namespace is the point.
- Making one feature imply the other.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
