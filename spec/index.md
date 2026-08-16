# Specification: HL7 v2.5 ER7 → v2.xml conversion

This is the single source of truth for what this crate converts and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7_v2_from_er7_into_xml` as implemented. Every rule
below is exercised by a unit test (next to the code that implements it, e.g.
`src/er7.rs`'s `#[cfg(test)]` module) or an integration test
(`tests/integration.rs`). A change to this document that isn't backed by a
test, or a code change that isn't reflected here, is a bug.

## 1. Scope

Convert one or more HL7 v2.5 messages, encoded in the traditional
pipe-delimited **ER7** ("Encoding Rule 7") syntax, into the official HL7
**v2.xml** XML representation (namespace `urn:hl7-org:v2xml`), as described
in HL7's XML encoding rules (see [References](#7-references)).

This crate is **not a validator**. It does not check cardinality, table
values, or data-type constraints, and it does not use or require an XSD. It
performs a structural, name-preserving translation: whatever ER7 the input
contains, well-formed XML comes out, with best-effort typed element names.

## 2. ER7 parsing (the [`er7`] crate)

[`er7`]: https://crates.io/crates/er7

Since 0.2.0 this crate does not parse ER7 itself. The encoding layer —
delimiters, the six-level value tree, escape sequences, batch splitting — is
the [`er7`] crate, whose own `spec/index.md` is normative for all of it.
This section states only what *this* crate relies on, and the one place it
differs.

The split is deliberate: ER7 is one small, stable encoding shared by every
HL7 v2 release, while the data-type tables and message structures below
(§3, §4) are specific to v2.5. Keeping them apart means the encoding is
maintained and tested once, and this crate is only the v2.5 dictionary on
top of it.

### 2.1 Input normalization (`normalize` in `src/lib.rs`)

Before parsing, input is tidied to the shape this crate has always
documented:

- A leading UTF-8 BOM (`\u{FEFF}`) is stripped.
- Input is split into lines on `\r` or `\n`; **each line is trimmed of
  surrounding whitespace**; empty lines are dropped.
- The surviving lines are rejoined with `\r` and handed to `er7::parse`.

The trimming is this crate's, not `er7`'s. `er7` deliberately trims nothing
because it guarantees a byte-for-byte round trip and cannot know whether a
trailing space is data (`er7` spec §4.1, rule R16). This crate makes no such
promise — it renders XML, where stray whitespace around a segment is noise,
and where an indented first line would otherwise become a `MissingMsh`
error rather than a converted document.

### 2.2 What `er7` guarantees, and this crate depends on

| Behavior | `er7` spec |
|----------|-----------|
| The first segment must be `MSH` (or `FHS`/`BHS`); nothing below the header can fail | §4.2, rules R5, R6 |
| The five delimiters are read from MSH-1/MSH-2, never hardcoded; omitted encoding characters fall back | §3.2, rules R1, R3 |
| A delimiter set that reuses one character for two roles is rejected | §3.3, rule R2 |
| MSH-1/MSH-2 (and FHS/BHS equivalents) are taken literally, never split or escape-decoded | §4.4.2, rule R8 |
| A field sent empty has no repetitions; empty positions below the field keep their places | §4.4.1, rule R7 |
| `\F\ \S\ \T\ \R\ \E\` and a well-formed `\Xhh..\` decode; every other sequence, and an unterminated escape character, is kept literally | §6.2, rule R13 |
| The explicit HL7 null `""` stays distinct from an empty value | §5.3, rules R10, R11 |

Two consequences worth stating, because they shape §4:

- **Decoding is on demand.** `er7` stores subcomponent text exactly as it
  arrived; this crate decodes it with `Subcomponent::value` at the point the
  text becomes XML (`src/xml.rs`), which is why the node builders take the
  message's `Separators`.
- **The explicit null is asked, not compared.** `Repetition::is_null`
  replaces the old string comparison against `""`.

### 2.3 Errors

`er7::Error` is mapped onto this crate's [`Hl7Error`](src/lib.rs) by a
`From` implementation, so the public error type is unchanged:

| `er7::Error` | `Hl7Error` |
|--------------|------------|
| `Empty` | `Empty` |
| `MissingHeader(name)` | `MissingMsh` |
| `BadHeader(detail)` | `BadMshHeader(detail)` |
| `BadPath(detail)` | `BadMshHeader(detail)` — unreachable; this crate never issues a path query |

## 3. Message-structure grouping (`src/structure.rs`)

### 3.1 Root element name

The root element name, and the key used to look up a message-structure
grammar, is derived from MSH-9 (`root_name` in `src/lib.rs`):

1. If MSH-9.3 (the structure ID) is present, it is used as-is.
2. Otherwise, from MSH-9.1 (message code) and MSH-9.2 (trigger event):
   - no code → `HL7Message`
   - code `ACK` → `ACK` (regardless of trigger)
   - code `ADT`, trigger one of `A01`/`A04`/`A08`/`A13` → `ADT_A01` (these
     trigger events share the ADT_A01 structure per the standard)
   - no trigger → the code alone
   - otherwise → `{code}_{trigger}`, e.g. `ORU_R01`

### 3.2 Grammars

A grammar is a sequence of `Item`s (`src/structure.rs`), each either a bare
segment or a named group of items; each item carries whether it is required
and whether it may repeat. Built-in grammars exist for:

| Structure  | Notes |
|------------|-------|
| `ACK`      | MSH, SFT*, MSA, ERR* |
| `ADT_A01`  | Also used for ADT^A04, ADT^A08, ADT^A13 (see §3.1) |
| `ORM_O01`  | Order detail supports the OBR choice only (see [Limitations](#6-limitations)) |
| `ORU_R01`  | Nested PATIENT_RESULT / PATIENT / ORDER_OBSERVATION / OBSERVATION / SPECIMEN groups |

Any other structure ID has no built-in grammar and always renders flat
(§3.3).

### 3.3 Matching and fallback to flat

Segments are matched against the root's grammar greedily, left to right. A
message groups successfully only if **every** segment is consumed and every
required item is satisfied. If the match fails for any reason — an
unexpected segment (e.g. a Z-segment appears where the grammar doesn't allow
it), a required segment missing, or the caller passed `Options { flat: true
}` (`--flat` on the CLI) — the message renders **flat**: all segments become
direct children of the root element, in their original order, with no group
elements at all. Flat rendering is always well-formed, lossless XML; it just
omits the group nesting.

## 4. XML element naming and rendering (`src/xml.rs`, `src/types.rs`)

### 4.1 Segments and fields

Each segment becomes a container element named after its ID (e.g. `<PID>`).
Each non-empty field becomes a child named `<SEG.n>` (1-based field index).
A field where every repetition is empty is omitted entirely, not rendered as
an empty element. Each repetition of a repeating field (`~`) becomes another
sibling element with the same name — repetition is not itself named or
counted in the XML.

### 4.2 Typed element names

Two lookup tables in `src/types.rs` drive typed naming:

- `segment_fields(seg)` — the ordered list of field data types for a known
  segment (MSH, EVN, PID, PV1, PV2, NK1, ORC, OBR, OBX, NTE, AL1, DG1, IN1,
  PR1, ROL, SFT, SPM, MSA, ERR, DSC, BLG, CTI, MRG, PD1, …).
- `composite_components(dt)` — the ordered list of component data types for
  a known composite type (CX, XPN, XCN, XAD, CE, CWE, EI, HD, TS, MSG, PT,
  VID, SN, …). A type absent from this table is treated as primitive (ST,
  ID, IS, NM, SI, DT, DTM, TX, FT, …) or as unknown.

Given a field's data type `DT`:

- If `DT` is a known composite: the field element's children are named
  `<DT.1>`, `<DT.2>`, … after `DT`'s components. A child that is itself a
  known composite type nests one further level the same way (e.g.
  `<CX.4><HD.1>…</HD.1></CX.4>`); subcomponents below that are treated as
  primitive.
- If `DT` is primitive, or unknown, or the field has exactly one component
  and one subcomponent: the field renders as a single text leaf.
- Otherwise (unknown structure — a segment or field not in the tables, with
  more than one component or subcomponent): positional **generic** names are
  used instead of type names — `<SEG.n.m>` per component, `<SEG.n.m.k>` per
  subcomponent. This is also what happens for every field of an unknown
  (including Z-) segment, since it has no entry in `segment_fields` at all.

### 4.3 OBX-5 variable typing

OBX-5's data type is not fixed by the segment table (it carries the
sentinel `VAR`); it is instead read from OBX-2 of the same segment
(uppercased). If OBX-2 names a known composite type, OBX-5's children use
that type's component names; otherwise OBX-5 falls back to the primitive /
generic rules above.

### 4.4 HL7 null and empty elements

A field, repetition, or component whose value is empty is either omitted
(field/repetition/component with *no* value) or rendered as a self-closing
element with no children and no text (the *explicit null* `""`) — never as
the literal text `""`.

### 4.5 Element name sanitizing

Every XML element name is passed through `xml::xml_name`: characters other
than ASCII letters, digits, `_`, `.`, `-` are stripped, and if the result
would not start with a letter or `_`, an `X` is prepended. This applies to
segment IDs (protecting against malformed/Z-segment IDs) and, transitively,
to every derived field/component name.

### 4.6 Document rendering

Output starts with `<?xml version="1.0" encoding="UTF-8"?>`, one newline,
then the root element with `xmlns="urn:hl7-org:v2xml"` and two-space
indentation per nesting level. Text content is XML-escaped for `&`, `<`,
`>` (not for quotes, which are never structurally significant in element
text). One converted message is one complete XML document; multiple
messages are never merged into one document.

## 5. Batch / multi-message input (`split_messages` in `src/lib.rs`)

Input may hold one message, several concatenated messages, or an HL7 batch
file:

- Input is normalized first (§2.1): BOM stripped, lines split on `\r`/`\n`,
  trimmed, empty lines dropped, rejoined with `\r`. Normalizing before
  splitting matters, because `er7` identifies a segment by its leading run
  of letters and digits, so an indented line would not be recognized as the
  `MSH` that starts a message.
- The normalized text is then split by `er7::split_messages` (`er7` spec
  §9, rule R21):
  - Batch envelope segments — `FHS`, `BHS`, `BTS`, `FTS` — are dropped. The
    name is matched exactly, so a longer local segment such as `BTSX` is
    kept. (Before 0.2.0 this crate reached the same answer by checking that
    the character after the three-letter code was not alphanumeric; the two
    rules agree on every input.)
  - A new message starts at each `MSH` line, or at the very first surviving
    line even if it is not `MSH` — that malformed message then fails with
    `Hl7Error::MissingMsh` when converted, rather than being silently
    dropped.
- `split_messages` returns owned `String`s, one per message, with segments
  joined by `\r`.

Each resulting message is converted independently; the CLI joins the
resulting XML documents with a blank line between them (§8).

## 6. Limitations

These are intentional scope boundaries, not defects:

- **Not a validator.** No XSD validation, cardinality checking, or HL7 table
  (vocabulary) checking is performed; the input is assumed to be reasonably
  well-formed HL7 v2.5.
- **Four built-in grammars.** Only `ACK`, `ADT_A01`, `ORM_O01`, and
  `ORU_R01` are grouped into message-structure elements; every other
  structure ID renders flat (§3.3), which is still well-formed, lossless
  XML.
- **ORM_O01 order detail** supports the common OBR choice only; messages
  using the RQD/RQ1/RXO/ODS/ODT detail-segment alternative render flat.
- **Formatting escape sequences** (`\.br\`, `\H\`, `\N\`, locally-defined
  `\Z...\`, etc.) are preserved as literal text, not mapped to `<escape/>`
  elements as some v2.xml producers do.
- **Data-type tables are scoped to common v2.5 messages.** Segments and
  composite types outside `src/types.rs`'s tables still convert (via the
  generic fallback in §4.2), just without type-derived names.
- **One dependency, by design.** This crate depends on [`er7`] for the
  encoding layer and on nothing else; `er7` itself has no dependencies, so
  the whole tree is two crates. Anything below the v2.5 dictionary belongs
  in `er7`, not here (§2).

## 7. References

- [`er7`](https://crates.io/crates/er7) — the ER7 encoding layer this crate
  is built on; its `spec/index.md` is normative for everything in §2
- [HL7 v2.xml encoding](https://www.hl7.eu/refactored/encoding02xml.html)
- [XML schemas for HL7 v2.5 and earlier (Australian Digital Health Agency)](https://implementer.digitalhealth.gov.au/standards/v2-xml-xml-schemas-for-hl7-version-2-5-and-earlier)
- [Microsoft BizTalk: HL7 2.X and 2.XML schemas](https://learn.microsoft.com/en-us/biztalk/adapters-and-accelerators/accelerator-hl7/hl7-2-x-and-2-xml-schemas)
- [InterSystems Healthcare HL7 XML](https://github.com/intersystems-ib/Healthcare-HL7-XML)
- [`hl7-v2-from-er7-into-json`](https://github.com/hl7-rust/hl7-v2-from-er7-into-json) —
  this crate's JSON sibling; §0 of its own `spec/index.md` states exactly
  where the two are meant to diverge
- [`hl7-v2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-v2-from-xml-into-er7)
  — reads this crate's v2.xml output back into ER7. Its `spec/index.md` §1.1
  documents the positional naming convention (§4.2 below) it depends on —
  changing that convention here without checking there breaks the round
  trip.

## 8. Command-line behavior (`src/main.rs`)

Documented here because it is spec-level (input/output contract), not an
implementation detail:

- `hl7_v2_from_er7_into_xml [OPTIONS] [FILE]` reads `FILE`, or stdin when `FILE` is
  omitted or `-`.
- `-o, --output <FILE>` writes to `FILE` instead of stdout.
- `--flat` forces flat rendering for every message in the input (§3.3).
- Input is split into messages per §5; each converts independently, and a
  conversion failure on any one message aborts the whole run (exit code 1,
  an error naming that message's 1-based position on stderr). Multiple
  output documents are joined with a blank line.
- Exit code 0 on success; 1 on any error (bad arguments, I/O failure, or a
  conversion error), with a message on stderr prefixed
  `hl7_v2_from_er7_into_xml: error:`.
