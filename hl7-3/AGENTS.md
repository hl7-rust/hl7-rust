# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

The Reference Information Model (RIM) backbone classes, the `II`/`CD` data
types, and the three-level message envelope for HL7 v3 — **a foundation,
not a complete implementation**. Read `spec/index.md` §1 before assuming
anything is missing is a bug; it almost certainly is documented scope.

Unlike the `hl7-v2` family, this crate has no sibling transport or format
crates (no `hl7-3-mllp`, nothing like `hl7-v2-from-er7-into-json`) and no
encoding-layer crate underneath it — HL7 v3 is XML natively, so
`hl7-v2-xml-lite-helper` (a sibling crate, not something this one wraps in
its own module) fills the role `er7` plays for `hl7-2`.

See `README.md` for the user-facing pitch and `spec/index.md` for the
exact, normative rules — **`spec/index.md` is the single source of truth
for behavior.**

## Layout

```
src/lib.rs           Crate docs, the Error type, the xml re-export.
src/vocabulary.rs     II and CD: the two data types every RIM attribute
                     reads through — spec/index.md §3.
src/rim.rs            The six backbone classes and their from_element
                     readers — spec/index.md §4.
src/message.rs        Message, ControlAct, and parse() — the three-level
                     envelope — spec/index.md §5.
spec/index.md          Normative specification (source of truth).
```

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block —
this crate has no `tests/integration.rs` yet; add one if a test needs more
than one module's types together (nothing has, so far).

## Working conventions

- **Rust edition 2024.** One dependency: `hl7-v2-xml-lite-helper`. Don't
  add another without discussion — the whole reason this crate reads
  through the shared helper instead of writing its own XML parser is to
  keep this crate's audit surface as small as `hl7-v2-soap`'s or
  `hl7-v2-from-xml-into-er7`'s.
- **Every `from_element` reader is infallible and total.** A missing
  optional attribute or child reads as `None` or an empty `Vec`, never a
  panic and never an error — matching `hl7-2` generic mode's "degrade,
  don't reject" philosophy. Only [`message::parse`] returns a `Result`,
  and its only error is malformed XML (`Error::Xml`); see spec/index.md §5.2.
- **A required RIM attribute** (`classCode`, `moodCode`, `typeCode`)
  **absent from an element reads as an empty string**, not an error or a
  panic — this crate reads what a message says, including a nonconforming
  one, rather than refusing it. Don't change this to a `Result` without
  discussion; it changes every reader's signature.
- **No vocabulary domain validation, on purpose.** A `CD.code` is read
  verbatim; this crate does not bundle or check against `ActClass`,
  `ActMood`, or any other domain's allowed values. See spec/index.md §6
  for why, and don't add partial validation for one domain without a plan
  for the rest — a coded field that's checked sometimes and not others is
  worse than one that's never checked.
- Every public item needs a doc comment; `src/lib.rs` carries
  `#![warn(missing_docs, clippy::pedantic)]` to enforce it.
- Before finishing a change, run:
  ```sh
  cargo test -p hl7-3
  cargo clippy -p hl7-3 --all-targets -- -D warnings
  cargo fmt --check -p hl7-3
  ```

## Making a spec-affecting change

1. **Update `spec/index.md` first** so it states the new behavior
   precisely, including the §7 traceability table entry.
2. Implement it, matching the module boundaries above.
3. Add or update the tests that pin it.
4. Update `README.md` only if the user-facing summary or examples change.
5. Run the checks above.

## Non-goals (don't "fix" these without discussion)

- **Modeling HL7 v3's full data type hierarchy** (`PQ`, `IVL<T>`, `ED`,
  and the rest). Two data types (§3) cover every RIM backbone attribute
  this crate currently reads; adding a third means a real caller needs it,
  not that the table looks incomplete.
- **Parsing `effective_time` and other raw-string timestamp fields** into
  a structured time or interval type. Read as written until a caller's
  need shapes what "parsed" should mean here.
- **Vocabulary domain validation** — see spec/index.md §6.
- **A CDA document model.** CDA reuses the RIM but has its own
  section/entry/narrative structure; that's a different crate's job if it
  happens.
- **A CLI**, unlike `hl7-2`. Nothing about this crate's current scope
  needs one.
