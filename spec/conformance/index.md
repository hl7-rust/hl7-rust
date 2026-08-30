[hl7-rust](../../README.md) → spec → Conformance

# Conformance

What "supports HL7® v2 releases 2.1 through 2.9" means here, stated
precisely enough to evaluate against — including everything it does not
mean.

This exists because "supports HL7 v2" is a claim every HL7 library makes
and almost none define. The useful question is not whether a library
supports v2; it is *which* segments, *which* structures, and what happens
to the ones it has never heard of. Those answers are below.

**This is not a certification, and no certifying body has assessed this
project.** HL7 International certifies people, not libraries, and where
product conformance testing exists it is against a specific implementation
guide and a specific test suite. What follows is a self-assessment whose
every line is checkable against the code and the tests.

## Contents

- [The one-paragraph answer](#the-one-paragraph-answer)
- [The governing principle: degrade, never reject](#the-governing-principle-degrade-never-reject)
- [HL7 v2: releases](#hl7-v2-releases)
- [HL7 v2: the dictionary, exactly](#hl7-v2-the-dictionary-exactly)
- [HL7 v2: encoding](#hl7-v2-encoding)
- [HL7 v2: what validation checks](#hl7-v2-what-validation-checks)
- [HL7 v2: what is out of scope](#hl7-v2-what-is-out-of-scope)
- [Transports](#transports)
- [Format conversions](#format-conversions)
- [HL7 v3](#hl7-v3)
- [The HL7® FHIR® standard](#the-hl7-fhir-standard)
- [How to evaluate this yourself](#how-to-evaluate-this-yourself)
- [How the gaps get filled](#how-the-gaps-get-filled)

## The one-paragraph answer

`hl7-2` reads and writes **any** syntactically well-formed ER7 message from
**any** release, losing nothing and rejecting nothing. What varies by
release, and what the dictionary bounds, is how much of the message it can
*name*: which fields it knows the data type of, which segments it can check
against a structure, and which values it can validate. It ships a complete
v2.5 dictionary of 24 segments, 42 composite data types, and 4 message
structures, and models the other thirteen releases as deltas of it. A
message using a segment or structure outside that set parses completely and
reads positionally.

If your evaluation criterion is "does it handle our feed without losing
data", the answer is yes for any ER7 input. If it is "does it know what
every field in our feed means", the answer is bounded by the table below,
and extending it is one JSON file.

## The governing principle: degrade, never reject

Every conformance gap in this project resolves the same way, and this is
the design decision the rest of the document elaborates:

**An unmodelled segment, field, data type, structure, or release
difference costs you a *name*, never a *value*.** The field reads with a
positional generic name instead of its data type; the message still parses,
every value is still reachable by path, and rendering still returns the
original bytes.

Only four things fail a call: a message with no usable MSH header, a path
that is not a path, a dictionary that will not load, and — in struct mode —
a value that does not fit the Rust type you asked for. Everything below the
header degrades and is reported by `validate()` if you want to know.

This is why the incompleteness below is bounded rather than dangerous. A
library that rejected what it did not recognize would turn every gap into a
dropped clinical message.

## HL7 v2: releases

All fourteen published releases resolve: **2.1, 2.2, 2.3, 2.3.1, 2.4, 2.5,
2.5.1, 2.6, 2.7, 2.7.1, 2.8, 2.8.1, 2.8.2, 2.9.**

- **v2.5 is the complete base dictionary.** Every other release is a delta
  of it, stating the differences this project models and inheriting the
  rest.
- **2.7.1, 2.8.1, and 2.8.2 have no dictionary file of their own** and
  resolve to their base release.
- A release string with no dictionary resolves to the nearest *older*
  known release: `2.5.2` reads as `2.5.1`, `3.0` reads as `2.9`. A version
  older than 2.1, unreadable, or absent falls back to v2.5 and raises a
  validation warning.
- `Options::version` overrides MSH-12 outright, for the common case of a
  sender that mislabels what it actually sends.

**What each release actually claims**, from `hl7-2`'s `spec/index.md` §3.4:

| Release | Modelled differences from v2.5 |
|---|---|
| 2.1, 2.2, 2.3 | MSH-9 has no message-structure component; MSH-12 is a plain `ID`; `ERR` is the one-field form; no `SFT`, no `SPM`; the pre-2.5 `ACK` structure |
| 2.3.1 | `ERR` one-field form; no `SFT`, no `SPM`; pre-2.5 `ACK`. MSH-9.3 and the `VID` composite arrived here, so both are inherited |
| 2.4 | `ERR` one-field form; no `SFT`, no `SPM`; pre-2.5 `ACK` |
| 2.5 | The complete base |
| 2.5.1, 2.6 | Nothing this project models changed |
| 2.7, 2.8, 2.9 | `TS` is withdrawn in favour of the primitive `DTM`, so a `TS`-typed field holds a scalar timestamp rather than a `value^precision` pair |

Where a release differs from v2.5 in a way not listed, it currently reads as
v2.5 would. That is stated rather than hidden: three of those differences
change how a message *reads* rather than only what a field is called, and
they are the three worth remembering — the `MSH-9` structure component
before 2.3.1, the one-field `ERR` before 2.5, and `TS` becoming `DTM` from
2.7.

## HL7 v2: the dictionary, exactly

The v2.5 base, in full, so there is nothing to guess at.

**24 segments:** `AL1` `BLG` `CTI` `DG1` `DSC` `ERR` `EVN` `IN1` `MRG`
`MSA` `MSH` `NK1` `NTE` `OBR` `OBX` `ORC` `PD1` `PID` `PR1` `PV1` `PV2`
`ROL` `SFT` `SPM`

**42 composite data types:** `AD` `AUI` `CCD` `CE` `CNE` `CNN` `CP` `CQ`
`CWE` `CX` `DLD` `DLN` `DR` `ED` `EI` `EIP` `ELD` `ERL` `FC` `FN` `HD`
`JCC` `MO` `MOC` `MSG` `NDL` `PL` `PRL` `PT` `RI` `RP` `SAD` `SN` `SPS`
`TQ` `TS` `VID` `XAD` `XCN` `XON` `XPN` `XTN`

**4 message structures:** `ACK`, `ADT_A01`, `ORM_O01`, `ORU_R01`, with
`ADT_A04`, `ADT_A08`, and `ADT_A13` aliased onto `ADT_A01`.

That is the honest number. HL7 v2.5 defines well over a hundred segments
and around eighty message structures; this dictionary covers the ones that
carry the overwhelming majority of real interface traffic — admissions,
orders, results, and their acknowledgements — and nothing else.

**What happens outside that set**, which is the part that decides whether
the number matters to you:

| Input | Result |
|---|---|
| A segment not in the 24 | Parses; fields read positionally; `SegmentUnknown` warning |
| A `Z`-segment | Parses; no warning at all — local extensions are nobody's business but the site's |
| A field past the end of a known segment's definition | Parses; reads positionally; `FieldUnknown` warning |
| A component past the end of a known type | Parses; reads positionally; `ComponentUnknown` warning |
| A message structure not in the 4 | Parses; reads as a flat segment list; `StructureUnknown` warning |
| A data type not in the 42 | Treated as primitive: the value is a scalar |

In every row the message parses, every value is reachable by path, and the
round trip is byte for byte.

**Extending it is deliberately cheap.** A dictionary is JSON. Add segments,
types, structures, or a whole vendor dialect at run time with schema mode,
or generate one from a site's own HL7 v2.xml XSD files with
`hl7-2-from-xsd-into-json-dictionary`. A dialect can inherit a bundled
release and state only its differences.

## HL7 v2: encoding

Encoding conformance is `er7`'s, and it is the layer where this project
makes its strongest claims:

- **Delimiters come from the message.** MSH-1 and MSH-2 declare them;
  `|^~\&` is a default, not an assumption. A sender using different
  separators is read correctly.
- **Escape sequences** are decoded on demand and re-encoded on the way out.
- **The explicit null `""` is distinct from an absent value**, at every
  level, and stays distinct through parsing, mutation, and rendering. This
  is the single most commonly botched detail in HL7 tooling, and it is the
  difference between "the patient has no middle name" and "delete the
  middle name we have on file".
- **Byte-for-byte round trip**: a message parsed and not modified renders
  back identically, after documented input normalization (a leading BOM is
  dropped, line endings are normalized to `\r`, blank lines are dropped,
  lines are trimmed).
- **Batches** (`FHS`/`BHS`) split into their constituent messages.
- **Paths** — `PID-5.1`, `OBX[2]-5[1].1.2` — with repetition, component,
  and subcomponent addressing.

## HL7 v2: what validation checks

`Message::validate` never fails and never modifies. It reports.

**Errors — the message contradicts the dictionary it claims:** MSH-9.1 or
MSH-10 empty; a required segment or group absent; segments that do not fit
the structure; an `SI`, `NM`, `DT`, `TM`, or `DTM` value that is not one.

**Warnings — the dictionary does not cover the message:** MSH-12 empty or
naming an unmodelled release; no grammar for the structure; standard
segments fit but Z-segments do not; an unknown segment, field, or
component.

**Deliberately not checked:** `ST`, `TX`, `ID`, `IS`, and every other type
whose validity is defined by an HL7 table or a length limit. Those are not
modelled, so the crate says nothing rather than guessing. Empty values and
explicit nulls are not format-checked.

`Options::strict` turns any error-severity finding into a parse failure.
Warnings never fail a parse: a coverage gap in this project is not the
sender's mistake.

## HL7 v2: what is out of scope

Named explicitly, because an evaluator needs the gaps more than the
features:

- **Vocabulary and code tables.** HL7 tables 0001–0396, LOINC, SNOMED CT,
  ICD, and every other terminology. A coded value is read as the string it
  is. Nothing checks that `F` is a valid result status or that a LOINC code
  exists.
- **Conformance profiles.** HL7 v2 conformance profiles and message
  profiles (the XML kind) are not read, applied, or generated. Validation
  is against the dictionary, not against a profile.
- **Field length limits.** Not modelled, so never enforced.
- **Usage codes.** R/RE/O/C/X optionality beyond the required/repeats pair
  the dictionary carries. Conditional usage rules are not modelled.
- **Exact repetition upper bounds.** "At most 10" and "unbounded" both read
  as "repeats".
- **Implementation guides.** No US Core, no national extension, no IHE
  profile, no jurisdiction-specific rule set.
- **Clinical semantics.** Nothing here knows that an `A08` should update a
  patient rather than create one.

## Transports

**MLLP** (`hl7-2-mllp`) implements the Minimal Lower Layer Protocol
framing — start block, end block, carriage return — including frames
accumulated from a stream that respects no message boundary, and generation
of the acknowledgement HL7 expects. Out of scope: TLS (compose it; the
transport takes any byte stream), async runtimes, connection pooling, retry
and backoff, and message persistence. It opens no sockets itself.

**SOAP** (`hl7-2-soap`, `hl7-3-soap`) implements the **SOAP 1.1** envelope,
faults and their HTTP statuses, the two ways HL7 v2 is carried in a body,
the response a receiver returns, and a WSDL describing the endpoint. Out of
scope, and stated as a deliberate choice rather than an omission: HTTP
itself — no client, no server, no TLS, no retries — plus SOAP 1.2,
WS-Security, WS-Addressing, MTOM, and attachments, none of which appeared
in the interfaces this was built from.

## Format conversions

Four crates convert ER7 to and from HL7 **v2.xml** and a typed JSON, in both
directions. What matters for conformance:

- The naming used by the tree, the JSON keys, and the XML elements is
  deliberately identical, so a path, a key, and an element read the same.
- `hl7-2-from-er7-into-xml` reads its data types and structures from
  `hl7-2` as of its 0.5.0. The other three still carry their own copies of
  the v2.5 tables, generated from the same source and table-for-table
  identical — they move across one at a time.
- The round trip ER7 → XML → ER7 is a test, not an aspiration.

## HL7 v3

`hl7-3` is **a foundation, not an implementation**, and says so in its own
first section. In scope: six RIM backbone classes (`Act`, `Entity`, `Role`,
`Participation`, `ActRelationship`, `RoleLink`), six data types (`II`,
`CD`, `IVL`, `PQ`, `ED`, and `NullFlavor`), and the three-level message
envelope read generically.

Out of scope: the rest of the v3 data type hierarchy (`IVL`/`PQ` are
shallow — bounds stay raw text; generic `IVL<T>`, `RTO`, `BL`/`BN`, `SC` and
the R2/ITS types are not modelled at all), vocabulary domain validation,
per-interaction domain payload schemas, and **the Clinical Document
Architecture**, which reuses the RIM but is its own document model and is
not implemented here.

Anyone evaluating this project for a v3 or Clinical Document Architecture
workload should read that as a no for the document architecture, and as
"a starting point you will extend" for v3.

## The HL7® FHIR® standard

**Not implemented.** `hl7/Cargo.toml`'s description names `hl7::fhir` as
"room for" once that standard is implemented, but no module by that name is
declared anywhere in source — `hl7/src/lib.rs` re-exports only `v2` and
`v3`. There is no HL7® FHIR® standard code in this workspace. If you need
the HL7® FHIR® standard today, this project is not it.

## How to evaluate this yourself

Do not take the above on trust; it takes about an hour to check.

1. **Run your own messages through the CLI.** `hl7-v2 --tree --paths` on a
   redacted sample prints every node, its value, and its path. A field that
   reads as `PID.34` rather than by its data type is a dictionary gap, and
   you will see exactly which ones you have.
2. **Check the round trip.** Parse and render one of your messages and
   diff it against the input. It should be byte-identical.
3. **Run `validate` over a day of redacted traffic** and count the
   warnings by kind. `SegmentUnknown` and `StructureUnknown` tell you the
   size of the gap between this dictionary and your feed.
4. **Read the numbered specs.** Each crate's `spec/index.md` is the source
   of truth and is numbered so it can be cited. Every rule in them is
   backed by a test; if a claim here and the code disagree, that is a bug
   and worth filing.
5. **Decide whether the gap is work.** A vendor dialect is one JSON file,
   or a generated dictionary from your own XSDs. That is usually an
   afternoon, not a project.

## How the gaps get filled

By real messages, not by transcription. The rule this project follows is
that coverage is added when a message motivates it — a table filled in from
the standard with nothing behind it is a table nobody can check and no test
can defend.

So the fastest way to widen the dictionary is to report a redacted message
that reads positionally when it should not. See
[`CONTRIBUTING.md`](../../CONTRIBUTING.md), and redact it the way
[`spec/phi/index.md`](../phi/index.md) describes.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
