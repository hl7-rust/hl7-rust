# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

Serde support for `hl7-2`: a same-named wrapper type per public `hl7-2`
value (`Message`, `Node`, `Diagnostic`, `Version`, and the three enums as
`NodeKind`, `Severity`, `DiagnosticKind`), each with a hand-written
`Serialize` and, where the type can be rebuilt, `Deserialize`. It is the
HL7® v2 half of the `serde-hl7` umbrella (`serde_hl7::v2`), and it is
built the way `serde-er7` in the sibling `er7-rust` workspace is built:
spec first, two dependencies, no derive, `Deref` everywhere, `Strict<T>`
for fixtures.

```
er7                    the ER7 encoding; serde-er7 is its Serde bridge
  |
hl7-2               the dictionary layer
  |
  +-- serde-hl7-v2    this crate: Serde for hl7-2's own types
```

**The layer boundary is the point.** This crate does not define a tree
shape for ER7 (that is `serde-er7`'s), does not know what any field means
(that is `hl7-2`'s), and does not pick a format (that is the caller's).
If a change here needs to know what a segment means, it belongs in
`hl7-2`; if it needs to know what JSON looks like, it belongs in a test.

See `README.md` for the user-facing tour and `spec/index.md` for the
exact, normative rules — **`spec/index.md` is the single source of truth
for behavior.**

## Layout

```
src/lib.rs          Crate docs, module list, re-exports, `pub use hl7_2`.
src/message.rs      Message: {"version", "er7"}; MessageSeed for Options.
src/node.rs         Node: six-key object, Serialize only (S14).
src/diagnostic.rs   Diagnostic: four-key object, round-trips.
src/kind.rs         NodeKind, Severity, DiagnosticKind via variant_string!.
src/version.rs      Version: the MSH-12 release string.
src/strict.rs       Strict<T> for Message and Diagnostic.
tests/integration.rs  Black-box tests over hl7-2's samples/*.hl7, plus the
                    manifest and spec self-checks.
examples/           Runnable programs, one concept each (see its README).
spec/               Normative specification, one directory per section,
                    with an S-numbered rule index and a §7.1 coverage table
                    that `cargo test` checks against it.
```

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block;
anything crossing module boundaries or using a real sample goes in
`tests/integration.rs`.

## Working conventions

- **Rust edition 2024. Exactly two runtime dependencies**, `serde` and
  `hl7-2` with `default-features = false` (spec S1). `serde_json` is a
  dev-dependency only (S2). A test asserts both from the manifest text.
- **No derive.** Every impl is hand-written against `Serializer`/
  `Deserializer`/`Visitor`. `macro_rules!` is fine (`variant_string!`),
  a proc-macro dependency is not.
- **The wire shape is API.** Changing any key, shape, or required-ness in
  spec §2 is a breaking change (S10): bump minor, update §2 first.
- **Tolerant by default, strict by request.** `T::deserialize` ignores
  unknown keys (S8); `Strict<T>` rejects them (S13). Never flip the
  default.
- **Never invent message state.** `Message` carries what `hl7-2` parsed
  and nothing more; deserializing re-parses through `hl7_2`, it does not
  reconstruct a `Message` by hand.
- Every public item must have a doc comment with an `Example:` block;
  `src/lib.rs` carries `#![warn(missing_docs, clippy::pedantic)]` and
  `#![forbid(unsafe_code)]`.
- Before finishing a change, from the workspace root:
  ```sh
  cargo test -p serde-hl7-v2
  cargo clippy -p serde-hl7-v2 --all-targets -- -D warnings
  cargo fmt --check
  cargo rustdoc -p serde-hl7-v2 --lib -- -W missing-docs
  ```
  and the workspace-level `bin/check-trademarks` and `bin/check-docs` if
  you touched prose.

## Making a spec-affecting change

1. **Update the matching `spec/NN-*/index.md` first**, and the rule index
   in `spec/index.md` if a rule changes.
2. Implement it, matching the module boundaries above.
3. Add or update the tests that pin it.
4. **Add the rule to the §7.1 coverage table**, naming those tests —
   `every_rule_has_a_coverage_row` fails otherwise.
5. Update `README.md` only if the user-facing summary changes.
6. Run the checks above.

## Non-goals (don't "fix" these without discussion)

- **A tree shape for `Message`.** Declined in spec §2.3; the ER7 tree is
  `serde-er7`'s.
- **`Deserialize` for `Node`.** Needs a constructor in `hl7-2` first
  (spec §9.1).
- **Serde for `Dictionary`** or for struct-mode types.
- **A `to_json_string` convenience**, or any format named in `src/`.
- **Pulling in `serde_derive`**, or a third runtime dependency of any kind.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
