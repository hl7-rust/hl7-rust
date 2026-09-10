# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

Serde support for `hl7-3`: a same-named wrapper type per public `hl7-3`
value — the envelope (`Message`, `ControlAct`), the six RIM classes, the
six data types, and the XML `Element` tree — each with `Serialize` and
`Deserialize`. It is the HL7® v3 half of the `serde-hl7` umbrella
(`serde_hl7::v3`), and it is built the way `serde-er7` in the sibling
`er7-rust` workspace and `serde-hl7-v2` here are built: spec first, two
dependencies, no derive, `Deref` everywhere, `Strict<T>` for fixtures.

```
hl7-2-xml-lite-helper    the XML reader
  |
hl7-3                 RIM, data types, the envelope
  |
  +-- serde-hl7-v3      this crate: Serde for hl7-3's own types
```

**The layer boundary is the point.** This crate serializes `hl7-3`'s
values; it does not write XML (neither does `hl7-3`), does not validate
codes against a vocabulary (neither does `hl7-3`), and does not pick a
format (that is the caller's).

See `README.md` for the user-facing tour and `spec/index.md` for the
exact, normative rules — **`spec/index.md` is the single source of truth
for behavior.**

## Layout

```
src/lib.rs          Crate docs, module list, re-exports, `pub use hl7_3`.
src/object.rs       The `object_wrapper!` macro every struct wrapper is
                    written with, the field-mode helper, and the three seed
                    types that carry strictness into nested values.
src/element.rs      Element: {"name", "attributes", "text", "children"}.
src/message.rs      Message and ControlAct.
src/vocabulary.rs   Ii, Cd, Ivl, Pq, Ed.
src/null_flavor.rs  NullFlavor: a bare string, hand-written, never fails.
src/rim.rs          Act, Entity, Role, Participation, ActRelationship, RoleLink.
src/strict.rs       Strict<T>; the per-type From impls live in the macro.
tests/integration.rs  One realistic interaction through every type, the §4
                    guarantees, the manifest and spec self-checks.
examples/           Runnable programs, one concept each (see its README).
spec/               Normative specification, one directory per section,
                    with an S-numbered rule index and a §7.1 coverage table
                    that `cargo test` checks against it.
```

## Working conventions

- **Rust edition 2024. Exactly two runtime dependencies**, `serde` and
  `hl7-3` with `default-features = false` (spec S1). `serde_json` is a
  dev-dependency only (S2). A test asserts both from the manifest text.
- **One macro, fourteen types.** Every struct wrapper is an
  `object_wrapper!` invocation (S15); a test counts them. A new `hl7-3`
  type means a new invocation with a field list, not a hand-written impl.
  The field modes (`req`, `opt`, `dflt`, `reqw`, `optw`, `vecw`) are
  documented at the top of `src/object.rs`.
- **Keys are lowerCamelCase field names** (S3) — the HL7 v3 XML names
  wherever a field has one. Every key is serialized every time.
- **The wire shape is API.** Changing any key, shape, or required-ness in
  spec §2 is a breaking change (S10): bump minor, update §2 first.
- **Tolerant by default, strict by request.** `T::deserialize` ignores
  unknown keys (S8); `Strict<T>` rejects them at any depth (S13). Never
  flip the default, and never let a nested value escape the strict flag —
  that is what the seeds in `object.rs` are for.
- Every public item must have a doc comment with an `Example:` block;
  `src/lib.rs` carries `#![warn(missing_docs, clippy::pedantic)]` and
  `#![forbid(unsafe_code)]`.
- Before finishing a change, from the workspace root:
  ```sh
  cargo test -p serde-hl7-v3
  cargo clippy -p serde-hl7-v3 --all-targets -- -D warnings
  cargo fmt --check
  cargo rustdoc -p serde-hl7-v3 --lib -- -W missing-docs
  ```
  and the workspace-level `bin/check-trademarks` and `bin/check-docs` if
  you touched prose.

## Making a spec-affecting change

1. **Update the matching `spec/NN-*/index.md` first**, and the rule index
   in `spec/index.md` if a rule changes.
2. Implement it — usually a field list in an `object_wrapper!` call.
3. Add or update the tests that pin it.
4. **Add the rule to the §7.1 coverage table**, naming those tests —
   `every_rule_has_a_coverage_row` fails otherwise.
5. Update `README.md` only if the user-facing summary changes.
6. Run the checks above.

## Non-goals (don't "fix" these without discussion)

- **An XML writer**, or XML as a Serde format.
- **Strict-checking `Element::attributes`.** Attribute names are the
  document's data (spec §9.2).
- **Rejecting an unknown `NullFlavor` code.** `hl7-3` carries it; so do
  we (S6).
- **Serde for struct-mode (`FromElement`) types.**
- **Pulling in `serde_derive`**, or a third runtime dependency of any kind.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
