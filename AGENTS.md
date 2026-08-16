# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this repository. `CLAUDE.md` is a pointer to this file — keep this one
canonical and don't fork the content between the two.

## What this is

A small Rust crate + CLI that converts HL7 v2.5 messages
from pipe-delimited ER7 text to the official v2.xml XML representation. It
is the XML sibling of
[`hl7-v2-from-er7-into-json`](https://github.com/hl7-rust/hl7-v2-from-er7-into-json)
(same parser, same data-type tables, same message-structure grammars,
different output format) — when in doubt about a shared-logic question,
check how the sibling repo handles it, and keep the two consistent unless
there's an XML-specific reason not to.

**This crate's element-naming convention is load-bearing for a fourth
crate.** [`hl7-v2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-v2-from-xml-into-er7)
reverses this crate's output without an HL7 v2.5 dictionary of its own,
relying entirely on the rule that the number after an element name's last
`.` is always that level's position (`src/xml.rs`, `src/types.rs`;
see that crate's `spec/index.md` §1.1). If you change how fields,
components, or subcomponents are named — not just what they're named,
but where the positional number appears — check that crate before
merging, or its round trip silently breaks.

See `README.md` for the user-facing pitch and `spec/index.md` for the exact,
normative conversion rules — **`spec/index.md` is the single source of
truth for behavior.** If you change what the converter does, update that
file in the same change; if you're unsure whether a change is a bug fix or
a behavior change, check it against the spec first.

## Layout

```
er7 (dependency)  ER7 parsing, delimiters, escape sequences, batch
                   splitting. Not in this repo — see spec/index.md §2.
src/lib.rs        Public API: convert(), convert_with_options(), Options,
                   Hl7Error, split_messages(), normalize(), root_name.
src/types.rs       Data-type tables: segment field types, composite
                   component types (drives typed XML element naming).
src/structure.rs   Message-structure grammars (ACK, ADT_A01, ORM_O01,
                   ORU_R01) and the greedy matcher that groups segments.
src/xml.rs         The Node tree, element-name sanitizing, XML rendering.
src/main.rs        CLI: argument parsing, stdin/file I/O, --flat/-o.
tests/integration.rs  Black-box tests through the public API, incl. one
                   golden full-document comparison.
spec/index.md      Normative specification (source of truth).
samples/*.hl7      Example ER7 input files used by README examples and
                   manual testing.
```

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block;
cross-cutting behavior (message-structure grouping, batch splitting, CLI
contract) is covered in `tests/integration.rs` instead.

## Working conventions

- **Rust edition 2024**, exactly one runtime dependency: the
  [`er7`](https://crates.io/crates/er7) crate, which supplies the ER7
  encoding layer and itself has none. Keep it that way unless the user asks
  for another; a two-crate tree is part of this crate's value proposition
  in a domain where dependencies get audited.
- **The layer boundary is the point.** This crate owns the HL7 v2.5
  dictionary — data-type tables, message structures, the XML renderer.
  `er7` owns the encoding — delimiters, the value tree, escape sequences,
  batch splitting. Anything about *how ER7 is written* belongs in `er7`,
  not here; see `spec/index.md` §2 for exactly which guarantees are
  inherited.
- Every public item must have a doc comment; `src/lib.rs` carries
  `#![warn(missing_docs)]` to enforce it. Run `cargo doc --no-deps` (or the
  check below) after adding public API.
- Match the existing doc style: a one-paragraph summary of *what* and, where
  the *why* isn't obvious from the code, a short rationale — see any
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
  or the CLI contract.
- Fallback-first design: this crate never fails on unrecognized input below
  the MSH header (unknown segments, unknown types, non-matching structures
  all degrade gracefully — see `spec/index.md` §3.3, §4.2, §6). Preserve
  that property; don't turn a "use generic names" case into an error.

## Making a spec-affecting change

1. Update `spec/index.md` first (or alongside the code) so it states the
   new intended behavior precisely.
2. Implement it, matching the module boundaries above.
3. Add/update tests that pin the new behavior.
4. Update `README.md` only if the change affects the user-facing summary or
   examples there (README intentionally doesn't restate everything the spec
   covers — see its own "See also" pointer).
5. Run the checks in the previous section.

## Non-goals (don't "fix" these without discussion)

- Adding HL7 table/vocabulary or XSD validation — this crate is explicitly
  not a validator (`spec/index.md` §6).
- Mapping formatting escape sequences (`\.br\`, `\H\`, …) to `<escape/>`
  elements — currently out of scope by design.
- Adding message-structure grammars beyond ACK/ADT_A01/ORM_O01/ORU_R01
  speculatively; add one when a real need (a failing test case, a user
  request) motivates it, and give it the same grammar-table treatment as
  the existing four in `src/structure.rs`.
