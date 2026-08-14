# Specification: HL7 v2.5 ER7 → v2.xml conversion

This is the single source of truth for what this crate converts and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7_2_5_to_xml` as implemented. Every rule
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

## 2. ER7 parsing (`src/er7.rs`)

### 2.1 Message and segment splitting

- Input is split into lines on `\r` or `\n`; each line is trimmed of
  surrounding whitespace; empty lines are dropped. A leading UTF-8 BOM
  (`\u{FEFF}`) is stripped first.
- The first non-empty line MUST start with `MSH`, or parsing fails with
  `Hl7Error::MissingMsh`. Input with no non-empty lines fails with
  `Hl7Error::Empty`.
- Every remaining line becomes one `Segment`, identified by its first three
  characters (the segment ID, e.g. `PID`).

### 2.2 Delimiters

The five ER7 delimiter characters are **read from the message itself**,
never hardcoded, per `Separators` (`src/er7.rs`):

| Delimiter    | Source              | Default | Role                        |
|--------------|----------------------|:-------:|------------------------------|
| field        | MSH, byte 4 (`MSH` + this char) | `\|` | separates fields in a segment |
| component    | MSH-2, position 1   | `^`     | separates components in a repetition |
| repetition   | MSH-2, position 2   | `~`     | separates repetitions in a field |
| escape       | MSH-2, position 3   | `\`     | introduces/closes an escape sequence |
| subcomponent | MSH-2, position 4   | `&`     | separates subcomponents in a component |

If the field separator is missing or is alphanumeric, parsing fails with
`Hl7Error::BadMshHeader`. If MSH-2 supplies fewer than four encoding
characters, the missing ones fall back to their default.

### 2.3 Field/repetition/component/subcomponent structure

Each segment line splits into fields on the field separator; each field
splits into repetitions on the repetition separator; each repetition splits
into components on the component separator; each component splits into
subcomponents on the subcomponent separator. Every subcomponent string is
then escape-decoded (§2.5).

**Exception — MSH-1 and MSH-2 (and the equivalent FHS-1/2, BHS-1/2 batch
envelope fields):** these are taken as one literal, pre-decoded value each,
never split on any delimiter and never escape-decoded, because MSH-1 *is*
the field separator and MSH-2 *is* the encoding-characters string.

### 2.4 HL7 null

The explicit HL7 null, the two-character literal `""`, marks a field whose
value is deliberately empty (as opposed to simply not sent). It is
distinguished from an ordinary empty field and renders as an empty XML
element (`<PID.2/>`) rather than as the literal text `""` or as an omitted
field. See §4.4.

### 2.5 Escape sequences

Within subcomponent text, `\X\` sequences (where `\` is the message's escape
character) decode as:

| Sequence | Decodes to |
|----------|------------|
| `\F\`    | the field separator |
| `\S\`    | the component separator |
| `\T\`    | the subcomponent separator |
| `\R\`    | the repetition separator |
| `\E\`    | the escape character itself |
| `\Xhh..\`| the bytes given by the hexadecimal digits `hh..`, interpreted as UTF-8 (lossy) |

Any other sequence — including formatting/highlighting commands such as
`\.br\` or `\H\`, and locally-defined `\Z...\` sequences — is **kept
literally**, escape characters included. An unterminated escape character
(no matching closing `\`) is likewise kept literally. This crate does not
map formatting escapes to `<escape/>` elements (see [Limitations](#6-limitations)).

Decoded text is XML-escaped afterward at render time (§4), so a decoded `&`,
`<`, or `>` is safe in the output.

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

- A leading BOM is stripped; lines are split on `\r`/`\n`, trimmed, and
  empty lines dropped, as in §2.1.
- Batch envelope segments — `FHS`, `BHS`, `BTS`, `FTS` — are dropped,
  **unless** the character immediately after the 3-letter code is
  alphanumeric (which would mean it's actually a different, longer segment
  ID that happens to start with those three letters, not a real envelope
  segment).
- A new message starts at each line beginning with `MSH` (or at the very
  first surviving line, even if not `MSH` — that malformed message will
  then fail with `Hl7Error::MissingMsh` when converted).
- Lines belonging to one message are rejoined with `\r`.

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

## 7. References

- [HL7 v2.xml encoding](https://www.hl7.eu/refactored/encoding02xml.html)
- [XML schemas for HL7 v2.5 and earlier (Australian Digital Health Agency)](https://implementer.digitalhealth.gov.au/standards/v2-xml-xml-schemas-for-hl7-version-2-5-and-earlier)
- [Microsoft BizTalk: HL7 2.X and 2.XML schemas](https://learn.microsoft.com/en-us/biztalk/adapters-and-accelerators/accelerator-hl7/hl7-2-x-and-2-xml-schemas)
- [InterSystems Healthcare HL7 XML](https://github.com/intersystems-ib/Healthcare-HL7-XML)

## 8. Command-line behavior (`src/main.rs`)

Documented here because it is spec-level (input/output contract), not an
implementation detail:

- `hl7_2_5_to_xml [OPTIONS] [FILE]` reads `FILE`, or stdin when `FILE` is
  omitted or `-`.
- `-o, --output <FILE>` writes to `FILE` instead of stdout.
- `--flat` forces flat rendering for every message in the input (§3.3).
- Input is split into messages per §5; each converts independently, and a
  conversion failure on any one message aborts the whole run (exit code 1,
  an error naming that message's 1-based position on stderr). Multiple
  output documents are joined with a blank line.
- Exit code 0 on success; 1 on any error (bad arguments, I/O failure, or a
  conversion error), with a message on stderr prefixed
  `hl7_2_5_to_xml: error:`.
