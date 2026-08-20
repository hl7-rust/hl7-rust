# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one
canonical and don't fork the content between the two.

## What this is

A small Rust crate + CLI that converts HL7 v2.5 messages
from pipe-delimited ER7 text to a typed JSON representation. It is the JSON
sibling of
[`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml)
(same parser, same data-type tables, same message-structure grammars,
different output format) — when in doubt about a shared-logic question,
check how the sibling crate handles it, and keep the two consistent unless
there's a JSON-specific reason not to
(`spec/index.md` §0 documents exactly where they're meant to diverge).

**This crate's key-naming convention is load-bearing for a fourth crate.**
[`hl7-2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7)
reverses this crate's output without an HL7 v2.5 dictionary of its own,
relying entirely on the rule that the number after a key's last `.` is
always that level's position (`src/json.rs`, `src/types.rs`; see that
crate's `spec/index.md` §1.1). If you change how fields, components, or
subcomponents are keyed — not just what they're keyed, but where the
positional number appears — check that crate before merging, or its round
trip silently breaks.

See `README.md` for the user-facing pitch and `spec/index.md` for the
exact, normative conversion rules — **`spec/index.md` is the single source
of truth for behavior.** If you change what the converter does, update
that file in the same change; if you're unsure whether a change is a bug
fix or a behavior change, check it against the spec first.

## Layout

```
er7 (dependency)  ER7 parsing, delimiters, escape sequences, batch
                   splitting. Not in this crate — see spec/index.md §2.
src/lib.rs        Public API: convert(), convert_with_options(), Options,
                   Hl7Error, split_messages(), normalize(), root_name.
src/types.rs       Data-type tables: segment field types, composite
                   component types (drives typed JSON key naming). Shared
                   verbatim with the XML sibling.
src/structure.rs   Message-structure grammars (ACK, ADT_A01, ORM_O01,
                   ORU_R01) and the greedy matcher that groups segments.
                   Same grammars as the XML sibling; builds json::Node
                   instead of xml::Node.
src/json.rs        The Node tree (segment/field/component -> keyed tree,
                   same naming rules as the XML sibling's Node), the Value
                   enum (Object/Array/String/Null), the Node -> Value
                   conversion that collapses same-named siblings into JSON
                   arrays, and pretty/compact rendering.
src/main.rs        CLI: argument parsing, stdin/file I/O, --flat/--compact/-o.
tests/integration.rs  Black-box tests through the public API, incl. one
                   golden full-document comparison.
spec/index.md      Normative specification (source of truth).
samples/*.hl7      Example ER7 input files used by README examples and
                   manual testing (identical files to the XML sibling's
                   samples/ — ER7 input is format-agnostic).
```

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block;
cross-cutting behavior (message-structure grouping, array collapsing, batch
splitting, the CLI contract) is covered in `tests/integration.rs` instead.

## Working conventions

- **Rust edition 2024**, exactly one runtime dependency: the
  [`er7`](https://crates.io/crates/er7) crate, which supplies the ER7
  encoding layer and itself has none. Keep it that way unless the user asks
  for another; a two-crate tree is part of this crate's value proposition
  in a domain where dependencies get audited.
- **The layer boundary is the point.** This crate owns the HL7 v2.5
  dictionary — data-type tables, message structures, the JSON renderer.
  `er7` owns the encoding. Anything about *how ER7 is written* belongs in
  `er7`, not here; see `spec/index.md` §2 for the inherited guarantees.
- Hand-rolling the JSON writer
  (`src/json.rs`) instead of pulling in `serde_json` is a deliberate part
  of this crate's value proposition, matching the XML sibling's zero-dep
  stance.
- Every public item must have a doc comment; `src/lib.rs` carries
  `#![warn(missing_docs)]` to enforce it. Run `cargo doc --no-deps` (or the
  check below) after adding public API.
- Match the existing doc style: a one-paragraph summary of *what* and,
  where the *why* isn't obvious from the code, a short rationale — see any
  existing `///` comment for the register to match.
- Before finishing a change, run:
  ```sh
  cargo test               # unit + integration tests
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  cargo rustdoc --lib -- -W missing-docs   # confirms no undocumented public items
  ```
  All four are clean on `main`; keep them that way.
- New behavior needs a test. Prefer a unit test next to the code it tests
  for parsing/naming rules, and an integration test in
  `tests/integration.rs` for anything that touches the public API surface
  or the CLI contract. When adding a golden full-document test, generate
  the expected string by running the CLI and reading the output back
  (`cargo run -- <input> > /tmp/x.json`, then read the file) rather than
  hand-typing indentation — it's easy to get JSON nesting depth wrong by
  hand, and a mismatched test failure doesn't tell you which of the two is
  wrong.
- Fallback-first design: this crate never fails on unrecognized input below
  the MSH header (unknown segments, unknown types, non-matching structures
  all degrade gracefully — see `spec/index.md` §3, §4.2, §6). Preserve that
  property; don't turn a "use generic names" case into an error.

## Making a spec-affecting change

1. Update `spec/index.md` first (or alongside the code) so it states the
   new intended behavior precisely. If the change also applies to the XML
   sibling's shared logic (ER7 parsing, data types, grammars), consider
   whether it should be made in both crates (one PR, since it's one
   workspace).
2. Implement it, matching the module boundaries above.
3. Add/update tests that pin the new behavior.
4. Update `README.md` only if the change affects the user-facing summary or
   examples there (README intentionally doesn't restate everything the spec
   covers — see its own "See also" pointer).
5. Run the checks in the previous section.

## Non-goals (don't "fix" these without discussion)

- Adding HL7 table/vocabulary or schema validation — this crate is
  explicitly not a validator (`spec/index.md` §6).
- Mapping formatting escape sequences (`\.br\`, `\H\`, …) to a dedicated
  JSON structure — currently out of scope by design.
- Emitting JSON numbers/booleans for numeric or boolean-looking HL7 fields
  — deliberately every scalar is a JSON string (`spec/index.md` §4.2, §6).
- Auto-normalizing single values into one-element arrays for keys that can
  repeat — that ambiguity is documented, intentional, and left to callers
  (`spec/index.md` §4.3, §6).
- Adding message-structure grammars beyond ACK/ADT_A01/ORM_O01/ORU_R01
  speculatively; add one when a real need (a failing test case, a user
  request) motivates it, and give it the same grammar-table treatment as
  the existing four in `src/structure.rs`.
- Pulling in `serde`/`serde_json` — see the dependency note above; the
  hand-rolled writer is deliberate.

## Benchmarks

Criterion benchmarks live in `benches/` and measure one conversion over a
short message and a 200-observation result — both synthetic, never real
patient data. `criterion` is a development dependency, so it is compiled
for `cargo bench` and never linked into the library or the binary; the
runtime dependency rule above is unchanged.

```sh
cargo bench -p hl7-2-from-er7-into-json
cargo bench -p hl7-2-from-er7-into-json -- --save-baseline before   # then compare a change
```
