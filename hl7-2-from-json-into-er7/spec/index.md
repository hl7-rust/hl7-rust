# Specification: typed JSON → HL7 v2.5 ER7 conversion

This is the single source of truth for what this crate converts and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7_2_from_json_into_er7` as implemented. Every
rule below is exercised by a unit test (next to the code that implements it)
or an integration test (`tests/integration.rs`). A change to this document
that isn't backed by a test, or a code change that isn't reflected here, is
a bug.

## 1. Scope

Convert one HL7 v2.5 message, encoded in the typed JSON representation the
sibling
[`hl7-2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json)
crate defines (its own `spec/index.md` §4), back to the traditional
pipe-delimited **ER7** ("Encoding Rule 7") syntax.

This crate is the inverse of that forward crate, and is intended primarily
to read documents that one produced. It is **not a validator**: it does not
check cardinality, table values, or data-type constraints, and there is no
JSON Schema it validates against.

### 1.1 No HL7 v2.5 dictionary — and why that's possible

Unlike its forward sibling, this crate carries no HL7 v2.5 data-type tables
at all. That sibling names every field key after either a known data type
(`"XPN.1"`) or, when the type is unknown, a bare position (`"PID.5.1"`) —
but in **both** cases, the number after the key's *last* `.` is always the
1-based position at that level: field number under a segment, component
number under a field repetition, subcomponent number under a component
(forward crate spec §4.2–§4.3). Reading that one number back out, at every
level, is enough to rebuild the value tree exactly — the data-type name
that precedes it (`XPN`, `CX`, or nothing at all) is decoration this crate
never has to interpret. This is exactly the same insight the sibling
[`hl7-2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-xml-into-er7)
crate applies to v2.xml; see §3.

## 2. JSON parsing (`src/json.rs`)

This crate includes its own minimal JSON reader (RFC 8259) rather than
depending on one, matching its sibling crates' one-dependency policy. It
reads the full JSON grammar — objects (keys keep first-appearance order),
arrays, strings (with all standard escapes, including `\uXXXX` surrogate
pairs), numbers, `true`/`false`, and `null` — even though the forward crate
never emits a number or a boolean itself (that crate's spec §4.2, §6): the
input may be hand-edited, and a document with an unexpected scalar should
still convert rather than fail.

A JSON value read where this crate expects a string (§4's leaf rule) is
coerced:

| JSON value | Read as |
|------------|---------|
| string     | its text, re-escaped (§4) |
| number     | its literal source text (`"3.5e2"`, not a parsed value) |
| `true` / `false` | `"1"` / `"0"` |
| object, array, `null` | the explicit HL7 null (§4.4) |

Anything that isn't well-formed JSON by this reader's rules — an unterminated
string, a missing comma, trailing data after the top-level value — is
`Hl7Error::Json`, the only category of failure this crate reports that
isn't about what the JSON means as HL7.

### 2.1 Nesting limit

Objects and arrays may nest at most **256** deep; past that, reading stops
with `Hl7Error::Json`.

Reading is recursive, so nesting depth is stack depth. Without a limit, a
few kilobytes of `[[[[…` overflow the stack and abort the process — a crash
the caller cannot catch, from a document another system sent them, which is
the one failure mode a reader must not have. An error is recoverable; an
abort is not.

The limit is not a judgement about documents. A converted message nests six
levels at the outside — structure, group, segment, field, component,
subcomponent — so 256 is far above anything this crate is for and far below
anything that threatens the stack.

## 3. Reconstructing the value tree (`src/reconstruct.rs`)

### 3.1 Unwrapping the document and flattening groups

The forward crate's document is one JSON object with exactly one top-level
key — the message structure ID — whose value holds the message's segments
and groups (forward crate spec §4.7). This crate requires that shape:
`Hl7Error::Empty` if the document isn't a single-key object over an object.

Within that inner object, the forward crate nests segments into
message-structure group entries for the few structures it has a grammar
for, e.g. `"ORM_O01.PATIENT"` or `"ORU_R01.ORDER_OBSERVATION"` (forward
crate spec §3). This crate does not know any message-structure grammar,
and does not need one: every group key is `{message-structure}.{group}`,
so an entry is a group, to be recursed into rather than kept, exactly when
its key contains a `.` — real segment IDs never do, and every group key
does, regardless of how deeply it is nested (the `{message-structure}`
prefix is always the top-level one, never accumulated per nesting level).

Flattening walks the whole tree this way — including through a *repeated*
group, which is a JSON array of group occurrences (§3.3) — and collects
the segment entries in document order. The result is exactly what the
forward crate's own `--flat` option would have produced, and a flat
segment sequence is all ER7 needs — so this crate reconstructs the same
message whether its input was flat or grouped.

### 3.2 Recovering the delimiters

The first flattened segment must be `MSH`, `FHS`, or `BHS` — the same
requirement `er7::parse` places on ER7 text. Its `.1` and `.2` fields hold
the field separator and encoding characters exactly as the forward crate
wrote them (see §4.1 below); this crate reassembles them into a synthetic
header line (e.g. `MSH|^~\&|`) and hands it to
[`er7::Separators::from_header`], reusing that crate's own delimiter
parsing rather than duplicating it.

### 3.3 Repetition: JSON arrays instead of repeated siblings

Unlike XML — which has no native way to say "this happened twice" other
than repeating the element — JSON has real arrays, and the forward crate
uses them: a key that occurred more than once collapses into one JSON
array holding one entry per occurrence, in order; a key that occurred
exactly once is **not** wrapped in a one-element array (forward crate spec
§4.3). This applies at every level a name can repeat: a segment or group
(`"OBX": [...]`), and a field (`"PID.3": [...]`).

This crate reads that back through one small rule, applied wherever it
needs "every occurrence of this key": an array's items are the occurrences,
in order; anything else is the sole occurrence. A component or
subcomponent position never legitimately repeats (HL7 has no such thing),
so if one somehow arrives as an array — not something the forward crate
produces — only its first item is used.

### 3.4 Fields, repetitions, components, subcomponents

Within a segment object, each entry's key is `{segment}.n`; `n` is the
field number, and that key's occurrences (§3.3) are the field's
repetitions, in order. A field number the segment object never mentions is
absent (`Field::default()`), not empty, matching how the forward crate
omits a field entirely rather than emitting a key with an empty value for
it.

The same rule applies one level down for components (entries of one field
repetition's object, keyed by the number after their key's last `.`) and
one level further for subcomponents (entries of one component's object).

At any of these levels, one value decides its own shape:

| Value            | Becomes                                            |
|-------------------|-----------------------------------------------------|
| an object          | a container: recurse one level down (§3.4)          |
| a string (or number/bool, coerced per §2) | a leaf holding that text (§3.5) |
| `null`             | the explicit HL7 null `""` at that position (§4.4)  |

A gap between mentioned positions (e.g. components 1 and 3 present, 2 not)
is filled with an empty placeholder at that position — `Component::default()`
or `Subcomponent::default()` — which is exactly what the forward crate
itself omits when a value is blank rather than null (that crate's spec
§4.5).

### 3.5 Re-escaping leaf text

The forward crate's leaf string is not fully decoded: `er7`'s `unescape`
resolves only the five delimiter escape sequences (`\F\ \S\ \T\ \R\ \E\`)
and well-formed `\Xdd..\` hex data, and leaves every other escape sequence —
formatting commands (`\.br\`), highlighting (`\H\` / `\N\`), locally
defined sequences (`\Zdd..\`), character-set switches — exactly as written
(`er7` spec §5.2). So the text this crate reads back is a mix of literal
delimiter characters and untouched escape sequences, and turning it back
into raw ER7 means telling those apart rather than escaping every
character blindly.

This crate does that by retokenizing the text with `er7::escape::escapes`,
using the same separators: a run with no escape character in it is data
and is re-escaped for the delimiters and control characters it contains
(`er7::escape::escape`); every other token is already valid ER7 and is
written back unchanged. This is not a general solution — a literal escape
character in the original data that happens to look like a formatting
sequence cannot be told apart from a real one, but that ambiguity is
inherent to ER7 itself and not something this crate's reversal introduces.

## 4. Segment- and field-level special cases

### 4.1 The header's delimiter fields

A header segment's field 1 (the field separator) and field 2 (the encoding
characters) are stored literally in `er7`'s value tree, never escaped —
they *are* the delimiters, not data encoded with them (`er7` spec §3.4).
The generic field-building rule in §3.4 cannot know that on its own, so
after building a header segment's fields generically, this crate
overwrites positions 1 and 2 with the segment's `.1`/`.2` string taken
verbatim (not escape-decoded or re-escaped).

### 4.2 Unrecognized segments and fields

Because reconstruction is purely positional (§1.1), a segment this crate
has never heard of — a Z-segment, or any segment the forward crate had no
data-type table for — reconstructs exactly like any other: its field keys
are still `{segment}.n`, just with generic component/subcomponent keys
below them (`{segment}.n.m`, `{segment}.n.m.k`) rather than a data-type
name, and the same last-`.`-is-the-position rule reads them identically
either way.

### 4.3 Duplicate object keys

JSON technically permits an object to repeat a key; the forward crate
never does (it uses an array instead, §3.3). If hand-edited input has a
duplicate key at what would be the same field/component/subcomponent
position, only the first is used — silently, matching this crate's
fallback-first philosophy (§5) rather than treating it as an error.

## 5. Fallback behavior and limitations

Fidelity degrades gracefully rather than failing, matching the forward
crate's own philosophy:

- **No dictionary, so no validation** of segment shape, data types,
  cardinality, or table values.
- **A segment named like a group cannot be told from one.** A segment
  key whose name contains a `.` is read as a group and flattened, so a
  segment the sender called `Z.1` contributes nothing. Nothing in the
  document distinguishes the two cases, and this crate has no
  message-structure grammar to consult (§3.1). The sibling forward crate
  never produces such a name — it strips `.` from segment IDs for exactly
  this reason — so this can only arise from another producer's output.
- **A key with no parseable trailing index** (should not arise from the
  forward crate's own output) is assigned the position right after the
  highest index already seen at that level, rather than being dropped.
- **A position above 10,000** is treated the same way, for the same
  reason and one more: reconstruction is dense, so position `n` costs `n`
  slots at that level, and a key is only text. `"PID.100000000"` — a
  hundred bytes of input — would otherwise ask for a hundred million
  fields, and a larger number for more memory than the machine has. Real
  segments run to tens of fields, so the cap is far above anything HL7
  defines and far below anything that hurts.
- **A number or boolean scalar** (never emitted by the forward crate, but
  valid JSON) is coerced to text per §2 rather than rejected.
- **A subcomponent-level value that is itself an object** (deeper nesting
  than the forward crate ever produces) is read as the explicit null
  rather than losing the value silently.
- **Blank (non-null) repetitions cannot be recovered.** The forward crate
  omits a field repetition that is present but entirely blank instead of
  including an entry for it (that crate's spec §4.5) — for example `A~~B`
  becomes a two-element array, not three. Reconstruction from JSON has no
  way to know a blank repetition was ever there, so a field's repetition
  *count* after a round trip through both crates may be lower than the
  original message's. This is a limitation of the forward crate's
  encoding, inherited here, not something this crate could recover.
- **A component- or subcomponent-level explicit null inside an otherwise
  populated composite** does not round-trip distinctly from a component
  that was simply blank, for the same reason: the forward crate does not
  mark that case differently in its output. Only a field-level (whole
  repetition) or leaf-level explicit null is unambiguous, and both are
  handled exactly (§3.4's table, §4.4).
- **One document per conversion.** Unlike ER7, which has an established
  batch-file convention, JSON as this crate reads it holds exactly one
  message per document; converting several means calling [`convert`]
  (or [`parse`]) once per document.

## 6. Errors

| `Hl7Error` variant | When |
|---------------------|------|
| `Json(JsonError)`    | the input is not well-formed JSON by `src/json.rs`'s rules |
| `Empty`              | the document isn't a single-key object over an object of segments, or that object has no segment entries at all |
| `MissingMsh`         | the first segment entry is not `MSH`, `FHS`, or `BHS` |
| `BadMshHeader(detail)` | the header's `.1`/`.2` fields don't declare a usable delimiter set |

Nothing else fails: an unrecognized segment, an unparseable position, or a
structure the forward crate never actually produces all reconstruct into
*something* rather than being rejected (§5).

## 7. References

- [`er7`](https://crates.io/crates/er7) — the ER7 encoding layer this crate
  writes onto; its `spec/index.md` is normative for delimiters, the value
  tree, escape sequences, and rendering.
- [`hl7-2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json)
  — the forward crate this one inverts; its `spec/index.md` is normative
  for exactly how JSON keys are named and values shaped.
- [`hl7-2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-xml-into-er7)
  — the XML sibling of this crate; same positional-reconstruction idea,
  applied to v2.xml instead.
- [`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml)
  — the forward crate `hl7-2-from-xml-into-er7` inverts; not a direct
  dependency of this crate, but the fourth member of the family.
- [RFC 8259 — The JavaScript Object Notation (JSON) Data Interchange Format](https://www.rfc-editor.org/rfc/rfc8259)

## 8. Command-line behavior (`src/main.rs`)

Documented here because it is spec-level (input/output contract), not an
implementation detail:

- `hl7_2_from_json_into_er7 [OPTIONS] [FILE]` reads `FILE`, or stdin when `FILE` is
  omitted or `-`. The input holds one converted-JSON document. At most one
  input may be named: a second one, `-` included, is an error rather than a
  silent replacement of the first.
- `-o, --output <FILE>` writes to `FILE` instead of stdout.
- `-t, --terminator <cr|lf|crlf>` chooses the segment terminator; `cr` (a
  bare carriage return) is the default and the only terminator HL7 permits
  on the wire.
- `--trailing-terminator` ends the last segment with a terminator too,
  which the default does not (see `er7` spec §6.1 for why).
- Exit code 0 on success; 1 on any error (bad arguments, I/O failure, or a
  conversion error), with a message on stderr prefixed
  `hl7_2_from_json_into_er7: error:`.
