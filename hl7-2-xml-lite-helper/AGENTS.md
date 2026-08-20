# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this repository. `CLAUDE.md` is a pointer to this file — keep this one
canonical and don't fork the content between the two.

## What this is

A small, dependency-free XML reader shared by three crates in the `hl7-2`
family, so each does not carry its own copy:

```
hl7-2-xml-lite-helper
    |
    +-- hl7-2-soap                          reads SOAP envelopes
    +-- hl7-2-from-xml-into-er7             reads HL7 v2.xml messages
    +-- hl7-2-from-xsd-into-json-dictionary reads HL7 v2.xml XSD schemas
```

It reads the subset of XML that carries meaning in a data document —
elements, attributes, text, and nesting — and skips the rest: no
validation, no schema, no DTD, no namespace resolution, no streaming. It is
scoped to the documents those three callers read, not offered as a
general-purpose parser, and does not claim a general-purpose name.

See `README.md` for the user-facing pitch and `spec/index.md` for the exact,
normative rules — **`spec/index.md` is the single source of truth for
behavior.**

## Layout

```
src/lib.rs          parse, Element, Error, escape, and the crate
                     documentation. The whole crate lives in this one file —
                     small enough to read in one sitting is the point.
tests/integration.rs  Black-box tests exercising every rule in spec/index.md.
spec/index.md        Normative specification (source of truth).
```

## The one thing to understand before touching this crate

**Namespace prefixes are ignored, not resolved.** `local_name`, and every
lookup built on it (`attribute`, `child`, `children_named`, `find`,
`text_at`), matches on the part after the first colon. `soapenv:Body`,
`soap:Body`, and `Body` are the same element. This is a deliberate trade
(spec/index.md §3.2): it is what lets a SOAP or v2.xml consumer accept
documents from any tool that serializes with a different prefix, at the
cost of being unable to distinguish two namespaces that happen to share a
local name. Do not "fix" this into real namespace resolution without
discussion — it would break the one thing every caller relies on.

## Dependencies

None, deliberately — see `Cargo.toml`, which declares an empty
`[dependencies]` table on purpose. The crate's whole argument is that it is
small enough to read and audit yourself; a dependency you'd also have to
read undermines that. Do not add one. `dev-dependencies`, if ever needed
for testing, are a different question — but production dependencies stay
at zero.

## Working conventions

- **Rust edition 2024.**
- The whole crate is `src/lib.rs`. Keep it that way unless the crate
  outgrows a single file — splitting it is a spec-affecting-sized decision,
  not a routine refactor.
- Every public item must have a doc comment; match the existing register
  (a one-paragraph summary of *what*, plus a short rationale where the
  *why* isn't obvious).
- **Deliberately out of scope** (spec/index.md §1): validation, schemas,
  DTDs, entity declarations, namespace resolution, streaming, XPath,
  mutation, and serialization of a tree beyond `escape`. Do not add these;
  a caller that needs them needs a different crate (`quick-xml`,
  `roxmltree`), and the README says so.
- Before finishing a change, run:
  ```sh
  cargo test -p hl7-2-xml-lite-helper
  cargo clippy -p hl7-2-xml-lite-helper --all-targets -- -D warnings
  cargo fmt --check
  ```
- After any change, confirm the three callers still build against it:
  ```sh
  cargo build -p hl7-2-soap -p hl7-2-from-xml-into-er7 \
    -p hl7-2-from-xsd-into-json-dictionary
  ```

## Making a spec-affecting change

1. **Update `spec/index.md` first** so it states the new behavior precisely.
2. Implement it in `src/lib.rs`.
3. Add or update the tests in `tests/integration.rs` that pin it.
4. Update `README.md` only if the user-facing summary or examples change.
5. Run the checks above, including the three-caller build check.

## Non-goals (don't "fix" these without discussion)

- **Namespace resolution.** See above — prefixes are ignored on purpose.
- **Speed.** spec/index.md §6 states plainly that this is not fast: it
  allocates a `String` per name, value, and text node, and copies text it
  could borrow. That is the right trade for an envelope, a schema, or a
  message; it is the wrong crate for a gigabyte of XML.
- **Becoming a general-purpose XML crate.** Scope creep here defeats the
  reason it exists — one small, auditable reader for three specific
  callers, not a fourth general-purpose parser to choose between.
- **Recovering from malformed XML.** A document that is not well-formed is
  an error, not a best guess (spec/index.md §3.6).

## Benchmarks

Criterion benchmarks live in `benches/` and measure one conversion over a
short message and a 200-observation result — both synthetic, never real
patient data. `criterion` is a development dependency, so it is compiled
for `cargo bench` and never linked into the library or the binary; the
runtime dependency rule above is unchanged.

```sh
cargo bench -p hl7-2-xml-lite-helper
cargo bench -p hl7-2-xml-lite-helper -- --save-baseline before   # then compare a change
```

## Fuzzing

`fuzz/` is a cargo-fuzz workspace of its own (nightly plus
`libfuzzer-sys`), outside the workspace above so that neither reaches the
crate's dependency list.

```sh
cargo +nightly fuzz run parse -- -max_total_time=60
```

The target asserts that reading is **total**: any bytes at all produce an
`Element` or an `Error`, never a panic and never a stack overflow. The
stack is the interesting half — reading is recursive, so nesting depth is
stack depth, and before `MAX_DEPTH` existed a few kilobytes of open tags
aborted the process. A crash writes its input to `fuzz/artifacts/parse/`;
reproduce it with `cargo +nightly fuzz run parse <that file>`. Corpus and
artifacts are gitignored.
