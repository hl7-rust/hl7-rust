# Specification: HL7® v2.5 ER7 → JSON conversion

This is the single source of truth for what this crate converts and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7_2_from_er7_into_json` as implemented. Every
rule below is exercised by a unit test (next to the code that implements
it, e.g. `src/types.rs`'s `#[cfg(test)]` module) or an integration test
(`tests/integration.rs`). A change to this document that isn't backed by a
test, or a code change that isn't reflected here, is a bug.

## 0. Relationship to `hl7-2-from-er7-into-xml`

This crate is the JSON sibling of `hl7-2-from-er7-into-xml`. Sections 1–3
below (ER7 parsing and message-structure grouping) describe identical
behavior between the two crates — same grammar, same delimiter rules, same
escape rules, same message-structure grammars — but that is no longer
literally shared code: ER7 parsing itself comes from the external `er7`
crate in both, but `hl7-2-from-er7-into-xml` reads its data types and
message structures from `hl7-2` as of its 0.5.0, while this crate still
carries its own `src/types.rs` copy of the v2.5 tables (see `hl7-2/spec/index.md`
§0). **Section 4 (element/key naming and rendering) is where the two
diverge in behavior**, because JSON and XML are different target formats
with different native capabilities (JSON has real arrays and a real null;
XML does not). Where this document says "unlike XML", that is the
deliberate difference; everything else is intentionally kept identical so
the two crates stay easy to compare and cross-reference.

This crate is **not a validator**, same as its sibling: no XSD/schema
validation, cardinality, or table checking is performed.

## 1. Scope

Convert one or more HL7 v2.5 messages, encoded in the traditional
pipe-delimited **ER7** ("Encoding Rule 7") syntax, into a JSON document.
There is no official "v2.json" HL7 standard equivalent to v2.xml; this
crate defines its own JSON mapping (§4), designed to preserve the same
information the v2.xml encoding preserves — including HL7's own typed
field/component naming — while using idiomatic JSON constructs (objects,
arrays, `null`, strings) instead of inventing an XML-shaped JSON wrapper.

## 2. ER7 parsing (the [`er7`] crate)

[`er7`]: https://crates.io/crates/er7

Identical to the sibling crate. Since 0.2.0 neither crate parses ER7
itself: the encoding layer — delimiters, the six-level value tree, escape
sequences, batch splitting — is the [`er7`] crate, whose own
`spec/index.md` is normative for all of it. See the sibling's
`spec/index.md` §2 for the fully worked rationale; behavior here is the
same.

The split is deliberate: ER7 is one small, stable encoding shared by every
HL7 v2 release, while the data-type tables and message structures below
(§3, §4) are specific to v2.5.

### 2.1 Input normalization (`normalize` in `src/lib.rs`)

Before parsing, input is tidied to the shape this crate has always
documented: BOM stripped; lines split on `\r`/`\n`; **each line trimmed**;
empty lines dropped; the rest rejoined with `\r` and handed to
`er7::parse`.

The trimming is this crate's, not `er7`'s. `er7` deliberately trims nothing
because it guarantees a byte-for-byte round trip and cannot know whether a
trailing space is data (`er7` spec §4.1, rule R16). This crate makes no
such promise — it renders JSON, where stray whitespace around a segment is
noise, and where an indented first line would otherwise become a
`MissingMsh` error rather than a converted document.

### 2.2 What `er7` guarantees, and this crate depends on

| Behavior | `er7` spec |
|----------|-----------|
| The first segment must be `MSH` (or `FHS`/`BHS`); nothing below the header can fail | §4.2, rules R5, R6 |
| The five delimiters are read from MSH-1/MSH-2, never hardcoded; omitted encoding characters fall back | §3.2, rules R1, R3 |
| A delimiter set that reuses one character for two roles is rejected | §3.3, rule R2 |
| MSH-1/MSH-2 (and FHS/BHS equivalents) are taken literally, never split or escape-decoded | §4.4.2, rule R8 |
| A field sent empty has no repetitions; empty positions below the field keep their places | §4.4.1, rule R7 |
| `\F\ \S\ \T\ \R\ \E\` and a well-formed `\Xhh..\` decode; every other sequence, and an unterminated escape character, is kept literally | §6.2, rule R13 |
| The explicit HL7 null `""` stays distinct from an empty value (§4.5) | §5.3, rules R10, R11 |

Two consequences worth stating, because they shape §4:

- **Decoding is on demand.** `er7` stores subcomponent text exactly as it
  arrived; this crate decodes it with `Subcomponent::value` at the point the
  text becomes JSON (`src/json.rs`), which is why the node builders take the
  message's `Separators`.
- **The explicit null is asked, not compared.** `Repetition::is_null`
  replaces the old string comparison against `""`.

### 2.3 Errors

`er7::Error` is mapped onto this crate's [`Hl7Error`](../src/lib.rs) by a
`From` implementation, so the public error type is unchanged:

| `er7::Error` | `Hl7Error` |
|--------------|------------|
| `Empty` | `Empty` |
| `MissingHeader(name)` | `MissingMsh` |
| `BadHeader(detail)` | `BadMshHeader(detail)` |
| `BadPath(detail)` | `BadMshHeader(detail)` — unreachable; this crate never issues a path query |

## 3. Message-structure grouping (`src/structure.rs`)

Identical to the sibling crate:

- The root key (and grammar lookup key) comes from MSH-9: MSH-9.3 verbatim
  when present, otherwise derived from MSH-9.1/9.2, with `ACK` collapsing
  every trigger to one structure and `ADT^A01/A04/A08/A13` all mapping to
  `ADT_A01`.
- Built-in grammars exist for `ACK`, `ADT_A01`, `ORM_O01` (OBR order-detail
  choice only), and `ORU_R01`. Any other structure ID, or a message whose
  segment sequence doesn't exactly fit its grammar (extra/missing/Z
  segments), falls back to **flat** rendering: every segment becomes a
  direct child of the root, in original order, with no group nesting.
  `Options { flat: true }` (`--flat`) forces this unconditionally.

## 4. JSON key naming and rendering (`src/json.rs`, `src/types.rs`)

### 4.1 Two-stage build

Conversion builds an intermediate [`Node`](../src/json.rs) tree — one node per
segment, field, repetition, and component, each carrying a *key* and either
child nodes or leaf text — using exactly the same typed-naming rules as the
XML sibling (§4.2 below). Then, and only then, the tree is turned into a
[`Value`](../src/json.rs) (JSON) tree, where **same-named sibling nodes
collapse into one JSON array** rather than becoming duplicate object keys.
This second stage has no XML equivalent — it's the reason repeating fields
and repeating groups render as real JSON arrays here instead of the
repeated-sibling-element trick XML uses.

### 4.2 Typed key names

A **segment's own key** is its segment ID with any `.` removed. This is not
cosmetic. The reverse sibling crate has no message-structure grammar and
does not need one, because it tells a group key from a segment key by the
`.` in a group's `{structure}.{group}` name (that crate's spec §3.1) — an
invariant that holds for every real segment ID. `er7` is deliberately
lenient about what it accepts as a segment ID, though, so a message can
carry `Z.1`; writing that out as the key `"Z.1"` would hand the reverse
crate something it must read as a group, and the segment and every value in
it would disappear without an error. Stripping the `.` keeps the invariant
true by construction rather than by hope.

Two lookup tables in `src/types.rs` (byte-for-byte shared with the sibling
crate) drive typed naming:

- `segment_fields(seg)` — the ordered field data types for a known segment
  (MSH, SFT, EVN, PID, PD1, NK1, PV1, PV2, ROL, DG1, PR1, ORC, OBR, OBX,
  NTE, AL1, IN1, MRG, MSA, ERR, DSC, BLG, CTI, SPM).
- `composite_components(dt)` — the ordered component data types for a known
  composite type (CX, XPN, XCN, XAD, CE, CWE, EI, HD, TS, MSG, PT, VID, SN,
  …). Anything absent is primitive (ST, ID, IS, NM, SI, DT, DTM, TX, FT,
  …) or unknown.

Given a field's data type `DT`:

- Known composite `DT`: the field's value is an object keyed `"DT.1"`,
  `"DT.2"`, … after `DT`'s components. A component that is itself a known
  composite type nests one further object level the same way (e.g.
  `"CX.4": {"HD.1": ..., "HD.2": ..., "HD.3": ...}`); subcomponents below
  that are treated as primitive.
- Primitive, unknown, or a field with exactly one component and one
  subcomponent: the field's value is a JSON **string** — the raw
  (escape-decoded) text, verbatim. Numeric-looking HL7 types (NM, SI, …)
  are still emitted as strings, never as JSON numbers: HL7 numeric text can
  carry leading zeros, explicit `+`, or trailing-zero decimal precision
  that a JSON number would silently normalize away, so this crate never
  performs that coercion.
- Otherwise (unknown structure — a segment or field outside the tables,
  with more than one component or subcomponent): positional **generic**
  keys are used instead of type names — `"SEG.n.m"` per component,
  `"SEG.n.m.k"` per subcomponent. This is also what happens for every field
  of an unknown (including Z-) segment.

### 4.3 Repetition and grouping become arrays

- A field with more than one repetition (`~`) produces a JSON **array**
  under the field's key, one array entry per repetition, each built per
  §4.2.
- A repeating segment or repeating group (per the structure grammar, §3)
  produces a JSON array under the segment ID or group key, one entry per
  occurrence — e.g. two `OBX` segments in one `OBSERVATION` group produce
  `"OBX": [ {...}, {...} ]`.
- A key that occurs exactly once — the common case — is **not** wrapped in
  a single-element array; its value is the object/string/null directly.
  Callers that need a uniform shape should treat a bare value and a
  one-element array as equivalent, or use a JSON path library that does.
- Sibling grouping is order-preserving: keys appear in first-occurrence
  order, matching the order segments/fields appeared in the source message.
- **A repeated key is grouped at its first occurrence**, which is the one
  place the mapping loses something a JSON object cannot hold. Segments
  `PID, ZZ1, ORC, PID` become `"PID": [ … , … ], "ZZ1": …, "ORC": …`: both
  `PID` entries keep their order relative to each other, but the two
  segments that sat *between* them no longer do. A JSON object cannot carry
  the same key twice, so there is nowhere else to put the second `PID`.
  Adjacent repeats — the common case, and every repeat inside a group the
  grammar knows — are unaffected.

### 4.4 OBX-5 variable typing

Identical rule to the XML sibling: OBX-5 has no fixed type; it's read from
OBX-2 of the same segment (uppercased). If OBX-2 names a known composite
type, OBX-5 is keyed by that type's components; otherwise it falls back to
the primitive/generic rules in §4.2.

### 4.5 HL7 null and absent fields

- The explicit HL7 null (`""`) becomes JSON **`null`** — not the two-
  character string `"\"\""`, and not an empty object or omitted key. This
  is the one place JSON expresses something XML cannot represent as
  directly (XML uses a self-closing empty element for the same case; see
  the XML sibling's spec §4.4).
- A field, repetition, or component with no value at all (`||`, not `""`)
  is **omitted** from its parent object entirely, not present as `null`
  and not present as an empty string. `null` always means "explicit HL7
  null"; a missing key always means "not sent".

### 4.6 String escaping

Field text is written as a JSON string per RFC 8259: `"`, `\`, and control
characters (`U+0000`–`U+001F`) are escaped; every other character,
including non-ASCII UTF-8, passes through unescaped. Unlike XML, `&`, `<`,
and `>` need **no** escaping in a JSON string — decoded ER7 text such as
`A&B` or `<200` appears in the output exactly as decoded.

### 4.7 Document shape

The whole document is one JSON object with exactly one top-level key: the
message's root name (§3), holding an object of that message's top-level
segments/groups, built the same way as any other container (§4.1–§4.3).
This mirrors the XML sibling's root element directly:

```json
{
  "ORM_O01": {
    "MSH": { "MSH.1": "|", "...": "..." },
    "ORM_O01.PATIENT": { "PID": { "...": "..." } },
    "ORM_O01.ORDER": { "ORC": { "...": "..." }, "...": "..." }
  }
}
```

Group keys carry the full official HL7 group name, prefixed by the message
structure ID (e.g. `ORM_O01.PATIENT`, not just `PATIENT`) — the same
convention the XML sibling uses for group element names, and the same
naming HL7's own chapter-2 structure tables use.

By default the document is rendered **pretty**: two-space indentation, one
key/value or array entry per line, a trailing newline at the end of the
document. `Options { compact: true }` (`--compact`) instead renders it as
one line with no insignificant whitespace. Both are exactly equivalent
JSON; pretty is the default because it matches this crate's sibling's
default (indented XML) and is far more readable for manual inspection.

## 5. Batch / multi-message input (`split_messages` in `src/lib.rs`)

Identical to the sibling crate (see its spec §5). Input is normalized
(§2.1) and then split by `er7::split_messages` (`er7` spec §9, rule R21):
batch envelope segments (`FHS`, `BHS`, `BTS`, `FTS`) are dropped, matched
by exact name so a longer local segment such as `BTSX` is kept; a new
message starts at each `MSH` line, or at the first surviving line even if
it is not `MSH`; segments are rejoined with `\r`.

Each resulting message converts to its own, independent JSON document
(never merged into one array or one object).

## 6. Limitations

Same scope boundaries as the XML sibling, restated for this crate:

- **Not a validator.** No JSON Schema equivalent, cardinality checking, or
  HL7 table (vocabulary) checking is performed.
- **Four built-in grammars** (`ACK`, `ADT_A01`, `ORM_O01`, `ORU_R01`);
  everything else renders flat — still valid JSON, just without group
  nesting. "Flat" is lossless for values but not for the *order* of
  non-adjacent repeats of one segment name: see §4.3. Where segment
  sequence carries meaning and the structure has no built-in grammar, the
  XML sibling keeps it and this mapping does not.
- **ORM_O01 order detail** supports the common OBR choice only; RQD/RQ1/
  RXO/ODS/ODT alternatives render flat.
- **Formatting escape sequences** (`\.br\`, `\H\`, `\N\`, locally-defined
  `\Z...\`, etc.) are preserved as literal text, not mapped to a dedicated
  JSON structure.
- **All scalars are strings** (§4.2) — this crate never emits a JSON
  number or boolean, by design, to avoid lossy numeric coercion of HL7
  text.
- **Single-vs-array ambiguity is inherent to the mapping** (§4.3): a
  consumer that always expects an array for a given key (because it *can*
  repeat) must normalize a bare value into a one-element array itself;
  this crate does not do so, to keep the common (non-repeating) case
  uncluttered.
- **One dependency, by design.** This crate depends on [`er7`] for the
  encoding layer and on nothing else; `er7` itself has no dependencies, so
  the whole tree is two crates. Anything below the v2.5 dictionary belongs
  in `er7`, not here (§2).

- **Development dependencies are separate.** `criterion` is compiled only
  for `cargo bench`, and `libfuzzer-sys` only for `cargo +nightly fuzz`
  in the separate `fuzz/` workspace. Neither is linked into the library
  or the binary, and neither reaches anyone who depends on this crate,
  so neither counts against the rule above.
## 7. References

- [`er7`](https://crates.io/crates/er7) — the ER7 encoding layer this crate
  is built on; its `spec/index.md` is normative for everything in §2
- [HL7 v2.xml encoding](https://www.hl7.eu/refactored/encoding02xml.html) —
  the XML encoding this crate's JSON mapping is modeled on (no equivalent
  official JSON encoding exists).
- [RFC 8259 — The JavaScript Object Notation (JSON) Data Interchange Format](https://www.rfc-editor.org/rfc/rfc8259)
- [XML schemas for HL7 v2.5 and earlier (Australian Digital Health Agency)](https://implementer.digitalhealth.gov.au/standards/v2-xml-xml-schemas-for-hl7-version-2-5-and-earlier)
- `hl7-2-from-er7-into-xml` — this crate's XML sibling; its `spec/index.md`
  documents §1–3 in full depth.
- [`hl7-2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7)
  — reads this crate's JSON output back into ER7. Its `spec/index.md` §1.1
  documents the positional key-naming convention (§4.2 above) it depends
  on — changing that convention here without checking there breaks the
  round trip.

## 8. Command-line behavior (`src/main.rs`)

- `hl7-2-from-er7-into-json [OPTIONS] [FILE]` reads `FILE`, or stdin when `FILE` is
  omitted or `-`. At most one input may be named: a second one, `-`
  included, is an error rather than a silent replacement of the first.
- `-o, --output <FILE>` writes to `FILE` instead of stdout.
- `--flat` forces flat rendering for every message in the input (§3).
- `--compact` emits compact JSON instead of the pretty-printed default
  (§4.7).
- Input is split into messages per §5; each converts independently, and a
  conversion failure on any one message aborts the whole run (exit code 1,
  an error naming that message's 1-based position on stderr). Multiple
  output documents are joined with a blank line (each is still its own,
  independent JSON document — the output as a whole is not itself one
  parseable JSON value when there is more than one message).
- Exit code 0 on success; 1 on any error (bad arguments, I/O failure, or a
  conversion error), with a message on stderr prefixed
  `hl7-2-from-er7-into-json: error:`.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
