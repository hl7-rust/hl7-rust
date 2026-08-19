# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this repository. `CLAUDE.md` is a pointer to this file — keep this one
canonical and don't fork the content between the two.

## What this is

A Rust crate (and command-line tool) that reads a directory of HL7 version 2
XML Schema Definition (XSD) files — the v2.xml encoding, as HL7 published it
or as a vendor customised it — and writes the JSON dictionary format that
`hl7-2` reads. It runs at authoring time and produces a build artifact; it
is the tool that generates a dictionary from schemas, not something a
runtime message-processing pipeline links against.

It sits at the schema-authoring end of the `hl7-2` family:

```
hl7-2-from-xsd-into-json-dictionary     schemas -> dictionary (this crate)
              |
              v
hl7-2       reads the dictionary (its spec/index.md §3)
              |
              +-- hl7-2-from-er7-into-xml     converts against it, e.g. in
                                                schema mode
              +-- ...
```

This crate does not depend on `hl7-2` to build. It depends on it only as a
dev-dependency, to test that a dictionary it writes actually loads in the
crate meant to read it (spec/index.md §0, §7).

See `README.md` for the user-facing pitch and `spec/index.md` for the exact,
normative rules — **`spec/index.md` is the single source of truth for
behavior.**

## Layout

```
src/lib.rs         convert_directory, Options, Error, and the crate
                   documentation. Re-exports hl7-2-xml-lite-helper as `xml`.
src/schema.rs      Reading XSD: cardinality, Types (data type resolution),
                   segments, and structure (recursive groups/segments).
src/dictionary.rs  The output types (Document, Field, Item) and JSON writing.
src/main.rs        The command-line tool: argument parsing and I/O.
spec/index.md      Normative specification (source of truth).
samples/example/   A small schema set used by the integration test.
```

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block;
the end-to-end conversion-then-load check lives in `tests/integration.rs`
(spec/index.md §7).

## Dependencies

One: [`hl7-2-xml-lite-helper`](../hl7-2-xml-lite-helper), the small,
dependency-free XML reader shared with `hl7-2-soap` and
`hl7-2-from-xml-into-er7`, re-exported from `src/lib.rs` as `xml` so a
caller can name `xml::Element` without adding its own dependency on it.
There is no `src/xml.rs` — do not reintroduce a hand-written XML reader
here; the shared helper is the reader (spec/index.md §2.1).

`hl7-2` is a **dev-dependency only**, used to check that a generated
dictionary loads (spec/index.md §6, §7). Do not promote it to a normal
dependency without discussion — the point of §0 is that this crate does
not need `hl7-2` to build.

Writing the dictionary needs a JSON writer, which lives in `src/dictionary.rs`
because it writes one document shape and nothing else — not worth a crate,
and not worth a dependency, per spec/index.md §5–§6.

## Working conventions

- **Rust edition 2024.**
- Match the existing doc style: a one-paragraph summary of *what* and,
  where the *why* isn't obvious, a short rationale — see any existing `///`
  comment for the register to match.
- `src/lib.rs` carries `#![warn(missing_docs, clippy::pedantic)]`; keep
  public items documented.
- **A construct not modeled in spec/index.md §3 is skipped, never an
  error.** A schema that carries more than this crate models still
  converts; do not add validation-style errors for constructs out of scope
  (spec/index.md §1).
- **`spec/index.md` §3 is the mapping from XSD shapes to dictionary output.**
  If a change alters what is read or what it becomes, update that section
  first.
- Before finishing a change, run:
  ```sh
  cargo test -p hl7-2-from-xsd-into-json-dictionary
  cargo clippy -p hl7-2-from-xsd-into-json-dictionary --all-targets -- -D warnings
  cargo fmt --check
  ```

## Making a spec-affecting change

1. **Update `spec/index.md` first** so it states the new behavior precisely.
2. Implement it, matching the module boundaries above.
3. Add or update the tests that pin it (unit test next to the code, or
   extend `samples/example/` and `tests/integration.rs` for an end-to-end
   case).
4. Update `README.md` only if the user-facing summary or examples change.
5. Run the checks above.

## Non-goals (don't "fix" these without discussion)

- **Validating a document against a schema.** This crate reads schemas to
  write a dictionary; it does not validate ER7 or XML instances against
  anything (spec/index.md §1).
- **General XML Schema support** — `xsd:restriction` facets, `xsd:choice`,
  `xsd:any`, imports, redefines. Out of scope by spec/index.md §1.
- **Resolving `<xsd:include>`** beyond reading the base-file prefix out of
  it (spec/index.md §1).
- **A general-purpose XML reader.** `hl7-2-xml-lite-helper` is scoped to
  the documents this family reads; extend it there, in that crate, not by
  adding a second reader here.
- **Adding `hl7-2` as a normal dependency.** It stays a dev-dependency; see
  spec/index.md §0.
