# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

A thin umbrella crate: `src/lib.rs` and nothing else, about thirty lines,
whose entire content is `pub use hl7_2 as v2;` plus its own doc comment.
It exists so a caller can `cargo add hl7` and get `hl7::v2` instead of
depending on `hl7-2` by name — one module per HL7 standard, so a
"message", a "segment", or a "code" in one standard is never confused with
the same word in another.

Today that's just `hl7::v2`. Room is left for `hl7::v3` and `hl7::fhir` as
those standards get implemented, each its own crate, re-exported here the
same way.

## Layout

```
src/lib.rs   The whole crate: `pub use hl7_2 as v2;`, the `derive` feature
             forwarding to hl7-2/derive, and the crate-level doc comment
             (which is also the one doctest this crate has).
```

There is no `spec/index.md` here — this crate is too thin to need one.
`hl7-2`'s spec covers the actual behavior; this crate only re-exports it.

## Working conventions

- **This crate carries no logic of its own.** Parsing, validation,
  dictionaries, all of it lives in `hl7-2` (and, in time, the other
  per-standard crates). If a change here does anything beyond adding or
  wiring up a `pub use`, it almost certainly belongs in the crate being
  re-exported instead.
- The `derive` feature does nothing but forward to `hl7-2`'s own `derive`
  feature (which in turn pulls in `hl7-2-derive`). Don't grow it into
  anything else.
- No tests of its own beyond the doctest in the crate-level doc comment —
  that's intentional, not a gap. Behavior gets tested where it lives, in
  `hl7-2`.
- Adding a new standard (`hl7::v3`, `hl7::fhir`, ...) means adding a new
  sibling crate with its own dictionary, tests, and spec, then one
  `pub use` line here. It does not mean growing this crate's own logic.
- Every public item must have a doc comment; `src/lib.rs` carries
  `#![warn(missing_docs)]` to enforce it.
- Before finishing a change, run:
  ```sh
  cargo test -p hl7 --all-features
  cargo clippy -p hl7 --all-targets --all-features -- -D warnings
  cargo fmt --check
  ```

## Non-goals (don't "fix" these without discussion)

- Implementing HL7 behavior directly in this crate. That's `hl7-2`'s job
  (and future standard-specific crates' job), never this one's.
- Flattening the `v2` module away, or otherwise hiding which standard a
  type belongs to. The per-standard namespace is the point.
