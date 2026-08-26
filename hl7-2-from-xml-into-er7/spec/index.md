# Specification: HL7® v2.xml → v2.5 ER7 conversion

This is the single source of truth for what this crate converts and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7_2_from_xml_into_er7` as implemented. Every rule
below is exercised by a unit test (next to the code that implements it) or an
integration test (`tests/integration.rs`). A change to this document that
isn't backed by a test, or a code change that isn't reflected here, is a bug.

## 1. Scope

Convert one HL7 v2.5 message, encoded in the official HL7 **v2.xml** XML
representation (namespace `urn:hl7-org:v2xml`), back to the traditional
pipe-delimited **ER7** ("Encoding Rule 7") syntax, as described in HL7's XML
encoding rules (see [References](#7-references)).

This crate is the inverse of the sibling
[`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml)
crate, and is intended primarily to read documents that one produced. It is
**not a validator**: it does not check cardinality, table values, or
data-type constraints, and it does not use or require an XSD.

### 1.1 No HL7 v2.5 dictionary — and why that's possible

Unlike its forward sibling, this crate carries no HL7 v2.5 data-type tables
at all. That sibling names every field element after either a known data
type (`<XPN.1>`) or, when the type is unknown, a bare position
(`<PID.5.1>`) — but in **both** cases, the number after the element name's
*last* `.` is always the 1-based position at that level: field number under
a segment, component number under a field repetition, subcomponent number
under a component (`hl7-2-from-er7-into-xml` spec §4.1–§4.2). Reading that
one number back out, at every level, is enough to rebuild the value tree
exactly — the data-type name that precedes it (`XPN`, `CX`, or nothing at
all) is decoration this crate never has to interpret. See §3.

## 2. XML parsing (`hl7-2-xml-lite-helper`)

Since 0.5.0 this crate does not read XML itself: the [`hl7-2-xml-lite-helper`]
crate does, re-exported here as [`crate::xml`], and its own `spec/index.md`
is normative for what it reads. Until 0.5.0 this crate carried its own
minimal, hand-written XML reader (`src/xml.rs`); the helper reads the same
subset a v2.xml document uses:

[`hl7-2-xml-lite-helper`]: https://crates.io/crates/hl7-2-xml-lite-helper

- One root element, arbitrarily nested child elements, and character data.
- The five predefined entities (`&amp; &lt; &gt; &quot; &apos;`) and numeric
  character references (`&#NN;`, `&#xHH;`); anything else that looks like an
  entity but isn't recognized is kept literally.
- Attributes are recognized and skipped — this crate has no use for `xmlns`
  or any other attribute.
- A `<?xml ... ?>` declaration, comments (`<!-- ... -->`), and a `DOCTYPE`
  are skipped wherever they appear before the root element; comments and
  `CDATA` sections are also accepted inside element content.
- **Mixed content is not supported**, because the source format never
  produces it: an element is read as a container (its child elements are
  kept, any text between them — pretty-printing whitespace — is discarded)
  whenever it has at least one child element, and as a leaf (its text is
  kept) otherwise. An element with neither text nor children is a leaf with
  no text — see §4.4.

Parsing produces a tree of [`xml::Element`] (helper spec §3.1): each element
carries a name plus either `children` or a (possibly empty) `text: String`,
mirroring the element the forward crate renders to.

Anything that isn't well-formed XML by the helper's rules — an unclosed
element, a mismatched close tag, a malformed attribute — is
`Hl7Error::Xml`, the only category of failure this crate reports that isn't
about what the XML means as HL7.

It is shared rather than owned because three crates in this family needed
the same XML subset and each had written it; `hl7-2-xml-lite-helper` has
no dependencies of its own, so the audit surface is unchanged from before
it existed.

### 2.1 Namespace prefixes

Every element name this crate reads is the element's **local** name: the
part after the first `:`, or the whole name when there is no `:` (helper
spec §3.2). The v2.xml namespace `urn:hl7-org:v2xml` may be bound to a
prefix rather than made the default, so the same message may arrive as
`<MSH><MSH.1>|</MSH.1></MSH>` from one serializer and
`<ns0:MSH><ns0:MSH.1>|</ns0:MSH.1></ns0:MSH>` from another. Which prefix a
serializer chose — `ns0`, `hl7`, `v2`, none — carries no information about
the message, so it is stripped before anything else looks at a name, at
every level:

- the segment/group test in §3.1,
- the `MSH`/`FHS`/`BHS` header test and the synthetic header line in §3.2,
- the trailing position number in §3.3,
- the segment ID written into the reconstructed `er7::Segment`, which is
  therefore always the bare `MSH`, `PID`, `ZDS`, … as ER7 requires.

Prefixes are stripped, not resolved: `xmlns` declarations are not
interpreted, so a document is read the same way whether or not it declares
the v2.xml namespace at all, and a document that used the same local names
under a *different* namespace would be read as v2.xml. That is the same
trade the helper makes (helper spec §3.2), and v2.xml documents do not mix
namespaces.

## 3. Reconstructing the value tree (`src/reconstruct.rs`)

### 3.1 Flattening groups back to a segment sequence

The forward crate nests segments into message-structure group elements for
the few structures it has a grammar for, e.g. `<ORM_O01.PATIENT>` or
`<ORU_R01.ORDER_OBSERVATION>` (`hl7-2-from-er7-into-xml` spec §3). This
crate does not know any message-structure grammar, and does not need one:
every group element is named `{message-structure}.{group}`, so a child
element is a segment when its local name (§2.1) contains no `.`, and a
group — to be recursed into, not kept — when it does. Real segment IDs never contain a
`.`, and every group name does, regardless of how deeply it is nested (the
`{message-structure}` prefix in a group's name is always the top-level one,
never accumulated per nesting level).

Flattening walks the whole tree this way and collects the segment elements
in document order. The result is exactly what the forward crate's own
`--flat` option would have produced, and a flat segment sequence is all ER7
needs — so this crate reconstructs the same message whether its input was
flat or grouped.

### 3.2 Recovering the delimiters

The first flattened segment must be `MSH`, `FHS`, or `BHS` — the same
requirement `er7::parse` places on ER7 text. Its `.1` and `.2` child
elements hold the field separator and encoding characters exactly as the
forward crate wrote them (see §4.1 below); this crate reassembles them into
a synthetic header line (e.g. `MSH|^~\&|`) and hands it to
[`er7::Separators::from_header`], reusing that crate's own delimiter
parsing rather than duplicating it.

### 3.3 Fields, repetitions, components, subcomponents

Within a segment element, each child's local name (§2.1) is `{segment}.n`;
`n` is the field number. Multiple children sharing the same `n` — sibling elements in
document order — are that field's repetitions, in that order. A field
number the segment element never mentions is absent (`Field::default()`),
not empty, matching how the forward crate omits an empty field entirely
rather than rendering an element for it.

The same rule applies one level down for components (children of one field
repetition, keyed by the number after their name's last `.`) and one level
further for subcomponents (children of one component). A **component**
number never repeats within one repetition — HL7 doesn't have repeating
components — so a duplicate is not a list: only its first occurrence is
kept.

At any of these levels, one element decides its own shape:

| Element has…              | Becomes                                          |
|----------------------------|---------------------------------------------------|
| child elements              | a container: recurse one level down                |
| text, no children           | a leaf holding that text (§3.4)                    |
| neither text nor children   | *empty* at that position — not the null (§4.4)     |

A gap between mentioned positions (e.g. components 1 and 3 present, 2 not)
is filled with an empty placeholder at that position — `Component::default()`
or `Subcomponent::default()` — which is exactly what the forward crate
itself omits when a value is blank rather than null.

### 3.3a The explicit null is text, an empty element is not

The [XML Encoding Rules][xml-encoding-rules] give the two opposite
meanings: "the occurrence of an empty element is treated as not existing to
keep backward compatibility with ER7", while the two double quote marks
`""` say the sender "ascertained that a data field has been deleted" and
the receiver should clear it. So:

- `<HD.2/>` and `<HD.2></HD.2>` reconstruct as an **empty** value — a
  position that exists and carries nothing.
- `<HD.2>""</HD.2>` reconstructs as the **explicit null** `""`, which is
  simply its text surviving §3.4 unchanged (`""` holds no delimiter and no
  escape character, so there is nothing to re-escape).

Before this rule, an empty element became the explicit null. That is the encoding
a schema-shaped document (`hl7-2-from-er7-into-xml` spec §4a rule 3) uses
to pad every declared-but-absent field and component, so a padded document
— the shape real senders validate against an XSD — read back as a message
whose absent fields had all become deletion instructions. The forward crate
was changed in the same commit to write the null as its literal text, so
the pair still round-trips both cases exactly.

[xml-encoding-rules]: https://www.hl7.eu/refactored/encoding02xml.html

### 3.4 Re-escaping leaf text

The forward crate's leaf text is not fully decoded: `er7`'s `unescape`
resolves only the five delimiter escape sequences (`\F\ \S\ \T\ \R\ \E\`)
and well-formed `\Xdd..\` hex data, and leaves every other escape sequence —
formatting commands (`\.br\`), highlighting (`\H\` / `\N\`), locally defined
sequences (`\Zdd..\`), character-set switches — exactly as written (`er7`
spec §5.2). So the text this crate reads back is a mix of literal delimiter
characters and untouched escape sequences, and turning it back into raw ER7
means telling those apart rather than escaping every character blindly.

This crate does that by retokenizing the text with `er7::escape::escapes`,
using the same separators: a run with no escape character in it is data and
is re-escaped for the delimiters and control characters it contains
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
The generic field-building rule in §3.3 cannot know that on its own, so
after building a header segment's fields generically, this crate overwrites
positions 1 and 2 with the segment's `.1`/`.2` text taken verbatim (decoded
from XML entities, but not escape-decoded or re-escaped).

### 4.2 Unrecognized segments and fields

Because reconstruction is purely positional (§1.1), a segment this crate
has never heard of — a Z-segment, or any segment the forward crate had no
data-type table for — reconstructs exactly like any other: its field
elements are still named `{segment}.n`, just with generic component/
subcomponent names below them (`{segment}.n.m`, `{segment}.n.m.k`) rather
than a data-type name, and the same last-`.`-is-the-position rule reads
them identically either way.

## 5. Fallback behavior and limitations

Fidelity degrades gracefully rather than failing, matching the forward
crate's own philosophy:

- **No dictionary, so no validation** of segment shape, data types,
  cardinality, or table values.
- **A segment named like a group cannot be told from one.** A segment
  element whose name contains a `.` is read as a group and flattened, so a
  segment the sender called `Z.1` contributes nothing. Nothing in the
  document distinguishes the two cases, and this crate has no
  message-structure grammar to consult (§3.1). The sibling forward crate
  never produces such a name — it strips `.` from segment IDs for exactly
  this reason — so this can only arise from another producer's output.
- **An element with no parseable trailing index** (should not arise from
  the forward crate's own output) is assigned the position right after the
  highest index already seen at that level, rather than being dropped.
- **A position above 10,000** is treated the same way, for the same
  reason and one more: reconstruction is dense, so position `n` costs `n`
  slots at that level, and a name is only text. `<PID.100000000>` — a
  hundred bytes of input — would otherwise ask for a hundred million
  fields, and a larger number for more memory than the machine has. Real
  segments run to tens of fields, so the cap is far above anything HL7
  defines and far below anything that hurts.
- **A subcomponent-level element with children** (deeper nesting than the
  forward crate ever produces) is read as empty rather than guessing at a
  value for it.
- **Blank (non-null) repetitions cannot be recovered.** The forward crate
  drops a field repetition that is present but entirely blank instead of
  rendering an element for it (`hl7-2-from-er7-into-xml` spec §4.1) — for
  example `A~~B` becomes only two elements, not three. Reconstruction from
  XML has no way to know a blank repetition was ever there, so a field's
  repetition *count* after a round trip through both crates may be lower
  than the original message's. This is a limitation of the forward crate's
  encoding, inherited here, not something this crate could recover.
- **A value whose text is literally two double quote marks** cannot be
  told apart from the explicit null, because ER7 itself does not
  distinguish them — `""` is the null wherever it appears.
- **One document per conversion.** Unlike ER7, which has an established
  batch-file convention, v2.xml as this crate reads it holds exactly one
  message per document; converting several means calling [`convert`]
  (or [`parse`]) once per document.

## 6. Errors

| `Hl7Error` variant | When |
|---------------------|------|
| `Xml(xml::Error)`    | the input is not well-formed XML by `hl7-2-xml-lite-helper`'s rules |
| `Empty`              | the document has no segment elements at all |
| `MissingMsh`         | the first segment element is not `MSH`, `FHS`, or `BHS` |
| `BadMshHeader(detail)` | the header's `.1`/`.2` fields don't declare a usable delimiter set |

Nothing else fails: an unrecognized segment, an unparseable position, or a
structure the forward crate never actually produces all reconstruct into
*something* rather than being rejected (§5).

## 7. References

- [`er7`](https://crates.io/crates/er7) — the ER7 encoding layer this crate
  writes onto; its `spec/index.md` is normative for delimiters, the value
  tree, escape sequences, and rendering.
- [`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml)
  — the forward crate this one inverts; its `spec/index.md` is normative for
  exactly how v2.xml elements are named and shaped.
- [`hl7-2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json)
  and [`hl7-2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7)
  — the JSON pair, applying the same positional-reconstruction idea (§1.1)
  to the typed JSON mapping instead of v2.xml.
- [HL7 v2.xml encoding](https://www.hl7.eu/refactored/encoding02xml.html)
- [XML schemas for HL7 v2.5 and earlier (Australian Digital Health Agency)](https://implementer.digitalhealth.gov.au/standards/v2-xml-xml-schemas-for-hl7-version-2-5-and-earlier)

## 8. Command-line behavior (`src/main.rs`)

Documented here because it is spec-level (input/output contract), not an
implementation detail:

- `hl7_2_from_xml_into_er7 [OPTIONS] [FILE]` reads `FILE`, or stdin when `FILE` is
  omitted or `-`. The input holds one v2.xml document. At most one input may
  be named: a second one, `-` included, is an error rather than a silent
  replacement of the first.
- `-o, --output <FILE>` writes to `FILE` instead of stdout.
- `-t, --terminator <cr|lf|crlf>` chooses the segment terminator; `cr` (a
  bare carriage return) is the default and the only terminator HL7 permits
  on the wire.
- `--trailing-terminator` ends the last segment with a terminator too,
  which the default does not (see `er7` spec §6.1 for why).
- Exit code 0 on success; 1 on any error (bad arguments, I/O failure, or a
  conversion error), with a message on stderr prefixed
  `hl7_2_from_xml_into_er7: error:`.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
