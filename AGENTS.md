# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this repository. `CLAUDE.md` is a pointer to this file — keep this one
canonical and don't fork the content between the two.

## What this is

A Rust crate + CLI that reads, navigates, validates, modifies, and writes
HL7 v2 messages in three modes: generic, schema-based, and struct-based.

It is the **dictionary layer** of the `hl7-rust` family, and that is the
whole point of its existence:

```
er7            the ER7 encoding (delimiters, escapes, paths, rendering)
  |
hl7-rust       this crate (imported as `hl7`, API in `hl7::v2`):
               releases 2.1-2.9, data types, message structures,
               three modes, mutation, validation
  |
  +-- hl7-v2-mllp    transport (MLLP over TCP)
  +-- hl7-v2-from-er7-into-json / -into-xml / from-json / from-xml
```

Anything about *how ER7 is written* belongs in `er7`, not here. Anything
about *what a segment or field means* belongs here and not in the four
conversion crates — which today still carry their own copies of the v2.5
tables. `schemas/v2.5.json` was generated from those copies and is
table-for-table identical, so the conversion crates can eventually depend on
this crate instead of duplicating it. **If you change the v2.5 tables or the
node-naming rules, check those crates**: their key and element names follow
the same rules on purpose, and a divergence is a bug in whichever moved.

See `README.md` for the user-facing pitch and `spec/index.md` for the exact,
normative rules — **`spec/index.md` is the single source of truth for
behavior.** If you change what this crate does, update that file in the same
change.

## Layout

```
er7 (dependency)   ER7 parsing, delimiters, escapes, paths, rendering,
                   batch splitting. Not in this repo — see spec/index.md §2.
src/lib.rs         Public API: parse(), parse_with_options(), Options,
                   Error, split_messages(), normalize(), re-exports.
src/v2/version.rs     Version (2.1 ... 2.9), MSH-12 resolution, nearest-older
                   fallback, and the bundled-dictionary cache.
src/v2/dictionary.rs  Dictionary: segment field types, composite component
                   types, structures, aliases; the JSON format, including
                   `inherits` deltas and sparse position overrides.
src/v2/json.rs        The hand-written JSON reader dictionaries load through.
src/v2/generic.rs     Generic mode: the Node tree and its naming rules.
src/v2/structure.rs   The greedy matcher that groups segments into a structure.
src/v2/message.rs     Message: version, structure ID, get/set/clear, segments,
                   tree, layout, round-tripping.
src/v2/typed.rs       Struct mode: FromHl7/ToHl7, value conversion, Raw.
src/v2/validate.rs    Diagnostics, severities, and the format checks.
src/v2/builder.rs     Builder and acknowledge().
src/main.rs        CLI: tree / query / check / er7, edits, schema, strict.
schemas/*.json     Bundled dictionaries. v2.5 is complete; the rest are
                   deltas of it (spec/index.md §3.4).
tests/integration.rs  Black-box tests through the public API and the CLI.
tests/spec.rs      The specification checking itself: every test named in
                   spec/index.md §13 must still exist.
samples/*.hl7      Example messages for the README and manual testing.
samples/acme.json  A vendor dialect, for the README's schema-mode example
                   and the schema-mode CLI test.
spec/index.md      Normative specification (source of truth).
```

The derive macros live in the sibling repo `hl7-v2-derive`, behind this
crate's optional `derive` feature, so the default build keeps exactly one
dependency.

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block;
anything crossing module boundaries — the three modes over one message,
round trips, the CLI contract — goes in `tests/integration.rs` instead.

## Working conventions

- **Rust edition 2024**, exactly one runtime dependency by default:
  [`er7`](https://crates.io/crates/er7), which itself has none. Keep it that
  way unless the user asks otherwise; a two-crate tree is part of this
  crate's value proposition in a domain where dependencies get audited. The
  `derive` feature is the one sanctioned exception, and it is opt-in.
- Hand-rolling the JSON reader (`src/v2/json.rs`) instead of pulling in
  `serde_json` is deliberate, matching the family's zero-dependency stance.
- **Fallback-first.** Reading never fails below the MSH header: unknown
  segments, unknown types, unmodelled releases, and structure mismatches
  degrade to positional names and a flat tree, and are *reported* by
  `validate()` rather than raised. Preserve that; don't turn a "use generic
  names" case into an error.
- **Dictionary knowledge goes in `schemas/`, not in code.** Structure
  aliases, per-release differences, and vendor dialects are data. If you
  find yourself writing a `match` on a segment name in Rust, ask whether it
  belongs in JSON.
- Every public item must have a doc comment; `src/lib.rs` carries
  `#![warn(missing_docs)]` to enforce it.
- Match the existing doc style: a one-paragraph summary of *what* and,
  where the *why* isn't obvious from the code, a short rationale — see any
  existing `///` comment for the register to match.
- Before finishing a change, run:
  ```sh
  cargo test --all-features
  cargo clippy --all-targets --all-features -- -D warnings
  cargo fmt --check
  cargo rustdoc --lib -- -W missing-docs   # no undocumented public items
  ```
  All four are clean on `main`; keep them that way. Run the derive crate's
  tests too when you touch anything the macros generate against.
- New behavior needs a test. Prefer a unit test next to the code for naming
  and parsing rules, and an integration test for anything touching the
  public API or the CLI contract.

## Making a spec-affecting change

The spec is not documentation that trails the code; it is the artifact the
code implements, and `tests/spec.rs` enforces part of that mechanically.

1. **Update `spec/index.md` first** so it states the new intended behavior
   precisely. If you cannot write the rule down, you do not yet know what
   you are building.
2. Implement it, matching the module boundaries above.
3. Add or update the tests that pin it.
4. **Add the rule to the §13 traceability table**, naming those tests.
   `tests/spec.rs` fails if the table names a test that does not exist, so
   a rename or a deletion cannot quietly hollow the spec out. It cannot
   check the other direction — that a test still tests what its name says —
   so that part is on you.
5. Update `README.md` only if the change affects the user-facing summary or
   examples there. The README is a tour; the spec is the truth, and when
   they disagree the README is what gets corrected.
6. Run the checks above.

Numbered sections are referenced from doc comments and from the sibling
crates' files, so renumbering is a rename across the repo — `grep -rn '§'`
before and after.

## Adding dictionary coverage

Per-release coverage is deliberately incremental (`spec/index.md` §3.4), and
that is a design position, not a to-do: v2.5 is complete and every other
release is a delta covering what this crate models today.

When adding to it:

- Edit the release's file in `schemas/`, not the Rust tables.
- Use the sparse `{"12": "ID"}` form for a single field rather than
  restating a whole segment — restating one claims a field count for that
  release, and an over-claim is worse than a gap.
- **Add only differences you can source.** A wrong data type is silently
  wrong data; a missing one is a positional name that still reads. When in
  doubt, leave it inherited and say so.
- Update the coverage table in `spec/index.md` §3.4 in the same change.
- Add a test that the difference actually shows up — see
  `a_release_difference_changes_how_a_field_reads` in
  `tests/integration.rs`.

## Non-goals (don't "fix" these without discussion)

- Becoming a conformance validator: HL7 vocabulary tables, field lengths,
  and conformance profiles are out of scope (`spec/index.md` §11).
- Transport: MLLP framing belongs to `hl7-v2-mllp`; files and queues to the
  caller.
- Converting to JSON or XML — that is what the four sibling crates do.
  `src/v2/json.rs` reads dictionaries; it is not a message renderer, and it
  should not grow into one.
- Partial message-structure grouping. It is all-or-nothing on purpose
  (`spec/index.md` §4.5): a partial match would have to guess where an
  unexpected segment belongs.
- Making the builder read the clock or invent control IDs — both would make
  outbound messages untestable and untraceable.
- Pulling in `serde`/`serde_json`, or making `derive` a default feature.
- Adding message-structure grammars speculatively; add one when a real need
  (a failing case, a user request) motivates it, and give it the same
  treatment as the four already in `schemas/v2.5.json`.
