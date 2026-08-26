# Specification: HL7® v2 parsing, navigation, modification, and validation

This is the single source of truth for what this crate does and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7-2` (reached as `hl7_2::...` directly,
or `hl7::v2::...` through the `hl7` umbrella crate) as implemented. Every rule below
is exercised by a unit test (next to the code that implements it, in that
module's `#[cfg(test)]` block) or an integration test
(`tests/integration.rs`, `../hl7-2-derive/tests/derive.rs`). A change to
this document that isn't backed by a test, or a code change that isn't
reflected here, is a bug.

## 0. Relationship to the rest of the family

Four layers, each owning exactly one thing:

```
er7                      the ER7 encoding: delimiters, escapes, paths,
                         byte-for-byte rendering, batch splitting
  |
hl7-2                   this crate: the HL7 v2 dictionary — releases
                         2.1-2.9, data types, message structures; three
                         parsing modes; mutation; validation
  |
  +-- hl7-2-mllp                  transport (MLLP over TCP)
  +-- hl7-2-from-er7-into-json    format conversions
  +-- hl7-2-from-er7-into-xml
  +-- hl7-2-from-json-into-er7
  +-- hl7-2-from-xml-into-er7

hl7-2-from-xsd-into-json-dictionary     writes the dictionaries this
                                         crate reads, from HL7 v2.xml XSDs
```

Anything about *how ER7 is written* belongs in `er7`, not here (§2).
Anything about *what a segment or field means* belongs here.
`hl7-2-from-er7-into-xml` reads its data types and message structures from
this crate as of its 0.5.0; the other three conversion crates still carry
their own copies of the v2.5 tables. This crate's `schemas/v2.5.json` was
generated from those copies and is table-for-table identical to them, which
is what lets each of them move across in turn.

A dictionary need not be hand-written or bundled:
`hl7-2-from-xsd-into-json-dictionary` generates one from a directory of HL7
v2.xml schemas, which is how a site that has schemas rather than
dictionaries gets a dialect this crate can read (§3.2.1).

Node naming (§4.2) is deliberately identical to those crates' key and
element naming, so a path through this crate's tree, a key in the JSON
crate's output, and an element in the XML crate's output all read the same.

## 1. Scope

The crate is published standalone as `hl7-2`, and its library is named
`hl7_2`. Most users instead reach it through the `hl7` umbrella crate,
whose library re-exports it as `hl7::v2` — one module per HL7 standard,
leaving `hl7::v3` and `hl7::fhir` free for later. Sources live under `src/`.
The command-line tool is `hl7-v2`.

Read one or more HL7 v2 messages in the pipe-delimited **ER7** encoding
and, through a dictionary for the release the sender speaks:

- **Generic mode** (§4) — a navigable tree of everything in the message,
  named by data type where the dictionary knows one and positionally where
  it does not.
- **Schema mode** (§5) — the same, through a dictionary the caller wrote
  and loaded at runtime.
- **Struct mode** (§6) — a caller's own Rust types, filled from paths, with
  the whole message still reachable from the same object.
- **Mutation and building** (§7) — change a parsed message, or build one
  from nothing, and write valid ER7 back out.
- **Validation** (§8) — check a message against its dictionary and report;
  optionally refuse.

Out of scope: transport, HL7 vocabulary tables, conformance profiles, and
conversion to JSON or XML. Those have owners of their own —
`hl7-2-mllp` frames messages for a network, and the four conversion
crates translate formats — which is why this crate can be about meaning
alone.

## 2. The ER7 layer (the [`er7`] crate)

[`er7`]: https://crates.io/crates/er7

This crate does not parse ER7. It reads and writes messages through `er7`,
and inherits its guarantees:

1. **Delimiters come from the message.** MSH-1 and MSH-2 declare them; the
   standard `|^~\&` is a default, not an assumption.
2. **Text is stored as sent** and decoded on demand, which is what makes a
   byte-for-byte round trip possible.
3. **The explicit null `""` is distinct from an absent value**, at every
   level, and stays distinct through this crate (§4.3, §7.2).
4. **Paths** (`PID-5.1`, `OBX[2]-5[1].1.2`) are `er7::Path`; this crate
   adds no path syntax of its own.
5. **Round trip**: a message parsed and not modified writes back byte for
   byte, after the normalization in §2.1.

### 2.1 Input normalization (`normalize` in `src/lib.rs`)

Before parsing, input is tidied: a leading byte-order mark is dropped,
lines are split on `\r` or `\n` (or both), each line is trimmed, blank
lines are dropped, and the rest are rejoined with `\r`. `er7` deliberately
trims nothing; this crate does, because a message indented for readability
would otherwise be a missing-header error rather than a message.

### 2.2 Errors

Only a message with no usable MSH header fails to parse: `Error::Empty`,
`Error::MissingMsh`, `Error::BadMshHeader`. Everything below the header
degrades (§4.2, §8) rather than failing. `er7::Error::BadPath` surfaces as
`Error::Path` from the call that used the path, not from parsing.

## 3. The dictionary (`src/dictionary.rs`, `schemas/`)

A **dictionary** is what a release, or a vendor dialect, says about
segments, data types, and message structures. It is JSON, read by the
hand-written reader in `src/json.rs`, and the same format serves the
bundled releases and schema mode.

That reader accepts objects and arrays nested at most **256** deep and
reports an ordinary `Error` past that. Reading is recursive, so nesting
depth is stack depth: without a limit a few kilobytes of `[[[[…` overflow
the stack and abort the process, and a dictionary is a file loaded from
somewhere — a vendor, a deployment's configuration directory — so a
malformed one must produce an error a caller can handle, not a crash it
cannot. A real dictionary nests a handful of levels.

### 3.1 Format

```json
{
  "version": "2.5",
  "description": "...",
  "inherits": "2.5",
  "types":      { "XPN": ["FN", "ST", "ST", "ST", "ST", "IS", "ID", "ID", "DR", "TS"] },
  "segments":   { "PID": ["SI", "CX", "CX", "CX", "XPN"],
                  "NK1": ["SI", {"type": "XPN", "repeats": true}],
                  "MSH": { "12": "ID" } },
  "aliases":    { "ADT_A04": "ADT_A01" },
  "structures": { "ACK": [
                    {"segment": "MSH", "required": true},
                    {"segment": "MSA", "required": true},
                    {"segment": "ERR", "repeats": true},
                    {"group": "PATIENT", "repeats": true, "items": ["PID", "PV1"]}
                  ] }
}
```

- `types` maps a composite data type to its component data types, in order.
  A type absent from `types` is primitive (or unknown, which behaves the
  same: treat the value as a scalar).
- `segments` maps a segment name to its field data types, index 0 being
  field 1. The sentinel `"VAR"` marks a field whose type another field
  names — only OBX-5, whose type is in OBX-2 (§4.4).
- `aliases` maps `CODE_TRIGGER` to the structure that carries it, which is
  how `ADT^A08` resolves to `ADT_A01` (§4.1).
- `structures` maps a message structure ID to its grammar. An item is an
  object with `segment` or with `group` + `items`, each with optional
  `required` and `repeats` (both default false); a bare string is the
  shorthand for an optional, non-repeating segment.

### 3.2 Two forms for a type list

A **list** (`["SI", "CX"]`) states the whole thing and replaces what was
inherited. An **object** (`{"12": "ID"}`) states individual 1-based
positions and leaves the rest inherited — which is how a release delta
restates MSH-12 without restating MSH. A position the object leaves unstated
and that no inherited list covers reads as unknown, not as some default.

### 3.2.1 Two forms for a position

Either form may write a position as a bare data type name (`"XPN"`) or as an
object that also states cardinality (`{"type": "XPN", "repeats": true}`).
The two are the same declaration when the object states nothing else.

`required` and `repeats` both default to false, and `Dictionary::field_cardinality`
reports them for any segment and field. They exist because a dictionary
generated from XML Schema knows a field's `minOccurs` and `maxOccurs`, and
both change what a faithful conversion emits: a **required** field is written
even when the message leaves it empty, so the position stays visible to a
validator, and a field that does **not** repeat keeps its repetition
separator as ordinary text instead of being split into several elements
(`hl7-2-from-er7-into-xml`, its `spec/index.md` §4).

The exact upper bound is not modelled — "at most 10" and "unbounded" both
read as `"repeats": true`, because nothing downstream distinguishes them.
Cardinality is kept for `segments` only: composite components neither repeat
nor are individually required, so an object form under `types` states a type
and nothing more.

The `hl7-2-from-xsd-into-json-dictionary` crate generates this form from a
directory of HL7 v2.xml XSD files, which is how a site with schemas rather
than dictionaries gets one.

### 3.3 Inheritance

`"inherits": "2.5"` starts from that bundled release and layers the
document over it: a listed entry replaces, `null` removes, anything
unmentioned is inherited. `Dictionary::from_json_over` layers over a
dictionary the caller already has instead, ignoring `inherits`.

### 3.4 Bundled releases and their coverage

`Version` covers the fourteen published releases 2.1 … 2.9.
`schemas/v2.*.json` are embedded at compile time and parsed on first use.
Releases without a file of their own (2.7.1, 2.8.1, 2.8.2) resolve to their
base release's file.

**v2.5 is the complete base**: its tables are the ones the sibling
conversion crates ship, covering the common segments, composite data types,
and the four structures ACK, ADT_A01, ORM_O01, ORU_R01.

**Every other release is a delta of v2.5 covering the differences this
crate models today, and inherits the rest.** What is modelled:

| release | modelled differences from v2.5 |
|---|---|
| 2.1, 2.2, 2.3 | MSH-9 has no message-structure component; MSH-12 is a plain `ID`; `ERR` is the one-field form; no `SFT`, no `SPM`; the pre-2.5 `ACK` structure |
| 2.3.1 | `ERR` one-field form; no `SFT`, no `SPM`; pre-2.5 `ACK`. MSH-9.3 and the `VID` composite arrived here, so both are inherited |
| 2.4 | `ERR` one-field form; no `SFT`, no `SPM`; pre-2.5 `ACK` |
| 2.5.1, 2.6 | nothing this crate models changed |
| 2.7, 2.8, 2.9 | `TS` is withdrawn in favour of the primitive `DTM`, so a `TS`-typed field holds a scalar timestamp rather than a `value^precision` pair |

This is honest incompleteness, not a claim of full per-release fidelity:
where a release differs from v2.5 in a way not listed above, this crate
currently reads it as v2.5 would. The consequences are bounded by design —
an unmodelled difference shows up as a positional name instead of a typed
one (§4.2), or as a missing warning in §8, never as a rejected message or a
lost value. Filling a gap means editing one JSON file and adding a test;
new coverage should be added when a real message motivates it rather than
speculatively.

### 3.5 Choosing a release

`Options::version`, when set, wins outright. Otherwise MSH-12.1 is resolved
by `Version::nearest`: an exact match if there is one, otherwise the newest
known release no newer than what was declared (so `2.5.2` reads as `2.5.1`
and `3.0` as `2.9`). A version older than 2.1, an unreadable one, or an
absent MSH-12 falls back to the default, v2.5, and §8 reports a warning.

## 4. Generic mode (`src/generic.rs`, `src/structure.rs`)

`Message::tree` returns a `Node` tree. Nothing in the message is dropped
and nothing is an error.

### 4.1 The root and message structure

The root node is named the **message structure ID**: MSH-9.3 when the
sender supplied one, otherwise resolved from MSH-9.1 and MSH-9.2 through
the dictionary — an alias (§3.1), then a structure named `CODE_TRIGGER`,
then one named `CODE` (which is how `ACK^A01` reaches `ACK`), then
`CODE_TRIGGER` unresolved. A message with no MSH-9 at all is `HL7Message`.

The structure ID is read from the message on each call, so a message whose
header was changed reports what it now says it is.

### 4.2 Node names

| level | dictionary knows the type | it does not |
|---|---|---|
| group | `ORU_R01.ORDER_OBSERVATION` | — |
| segment | `PID` | `ZPD` |
| field | `PID.5` | `ZPD.2` |
| component | `XPN.1` (the field's type) | `ZPD.2.1` |
| subcomponent | `FN.1` (the component's type) | `ZPD.2.1.1` |

A field whose type the dictionary knows expands into components named after
that type; a component whose own type is composite expands into
subcomponents named after *its* type. Otherwise names are positional, and
a value with no internal structure stays a leaf.

Every node also carries the `er7` path that locates it —
`PID[1]-5[1].1.2` — which is what makes a node found by exploring
immediately readable and writable by path.

**Names and paths are two different vocabularies and are not
interchangeable.** A name (`XPN.1`) says what a node *is*, in HL7's own
data-type vocabulary, and is what `Node::child` and `Node::find` match on.
A path (`PID-5.1`) says *where* a node is, and is what `Message::get`,
`Message::set`, and the CLI's `--query` take. `XPN.1` as a path would name
field 1 of a segment called `XPN`, which no message has.

### 4.3 What survives

- **Repetitions** are separate sibling nodes with the same name: `A~B~C` in
  PID-3 is three `PID.3` nodes, at paths `PID[1]-3[1]`, `[2]`, `[3]`.
  Reading them by path takes `Message::repetitions`, not
  `Message::get_all`: a path naming a whole field is one value to `er7` —
  the field's text, repetition separators and all — because that is what
  the field *is*, so `get_all("PID-3")` gives `["241900~99~7"]` where
  `repetitions("PID-3")` gives the three. Paths that already name a
  repetition or a component behave identically in both.
- **Empty fields and repetitions** produce no node.
- **The explicit null `""`** produces a node with `is_null()` true, so
  "clear this value" stays distinguishable from "nothing to say".
- **`Node::text`** is the decoded text of the node and everything beneath
  it, delimiters intact, so a field reads as `SMITH^JOHN` whether or not
  the dictionary knew what an `XPN` is.

### 4.4 OBX-5

OBX-5's data type is whatever OBX-2 names. If OBX-2 names a composite the
dictionary knows, OBX-5 expands with that type's component names; otherwise
OBX-5 is a scalar.

### 4.5 Message-structure grouping

When the dictionary has a grammar for the structure ID, a greedy
recursive-descent matcher arranges the segments into its groups; group
nodes are named `STRUCTURE.GROUP`.

Matching is **all-or-nothing**: unless every segment is consumed by the
grammar, the tree is flat instead. A partial match would have to guess
where an unexpected segment belongs, and a wrong guess is worse than no
grouping. `Message::tree_with_options(false)` asks for a flat tree
outright, and `Message::layout` exposes the match itself.

## 5. Schema mode (`Dictionary::from_json`, `Options::with_dictionary`)

Schema mode is generic mode with a dictionary the caller supplies:
everything in §4 applies, using the caller's segment tables, data types,
aliases, and structures. A schema is JSON in the §3 format, loaded at
runtime, so adding a field the business just invented is a configuration
change rather than a release.

A schema that inherits a bundled release gets the standard segments for
free and states only its dialect. A schema that inherits nothing describes
the world by itself, and everything it omits reads positionally (§4.2).

## 6. Struct mode (`src/typed.rs`, `hl7-2-derive`)

`FromHl7` reads a caller's type from a message; `ToHl7` writes one back.
`#[derive(FromHl7)]` and `#[derive(ToHl7)]` (feature `derive`) generate
both from one attribute per field:

| attribute | on read | on write |
|---|---|---|
| `#[hl7("PID-5.1")]` | read the path via `FromHl7Value` | write the path via `ToHl7Value` |
| `#[hl7(nested)]` | the field's own `FromHl7` | the field's own `ToHl7` |
| `#[hl7(raw)]` | a `Raw` holding the whole message | skipped |
| none | `Default::default()` | skipped |

One attribute is written on the **struct** rather than a field:
`#[hl7(crate = ...)]`, taking a path either bare (`crate = hl7`) or quoted
(`crate = "::vendor::hl7_2"`). The generated code names this crate
absolutely, as `::hl7_2`, so that a derived type compiles wherever it is
defined without its author importing anything; a caller who renames the
dependency (`hl7 = { package = "hl7-2" }`) has no `::hl7_2` for the macro to
reach, and cannot patch generated code from their side. The attribute is how
they say where it went. It defaults to `::hl7_2`, which is what almost every
caller wants.

### 6.1 Value conversion

`FromHl7Value` is implemented for `String`, `bool`, the integer and
floating-point types, and `Option<T>` and `Vec<T>` of those:

- a plain type is **required**: a path that names nothing is
  `Error::MissingField`, because a non-optional field is a promise;
- `Option<T>` reads absent, empty, and the explicit null as `None`;
- `Vec<T>` reads through `Message::repetitions`, so `#[hl7("PID-3")]` gives
  one entry per repetition rather than one string containing all of them;
- `bool` reads `Y`/`N` (and `1`/`0`, `true`/`false`) and writes `Y`/`N`;
- a value that does not fit its Rust type is `Error::BadValue`, naming the
  path.

`FromHl7Text` is the extension point for a caller's own scalar type; once
implemented, `Option<T>` and `Vec<T>` of it follow.

### 6.2 Multi-modal access

A `#[hl7(raw)]` field of type `Raw` holds the parsed message alongside the
typed data, so the escape hatch is a method call on the object the caller
already has — no second parse, no rewriting the library — which is the
whole point of having three modes rather than three libraries.
`Message::raw` is the same escape hatch one level down, reaching the
`er7::Message` itself.

## 7. Mutation and building (`src/message.rs`, `src/builder.rs`)

### 7.1 Writing values

`Message::set(path, value)` writes data: delimiters inside `value` are
escaped, so `SMITH^JOHN` becomes one component containing a literal caret.
`Message::set_er7(path, text)` writes text that is already encoded, so the
same string becomes two components. Both create whatever the path names and
the message lacks — fields, repetitions, components, subcomponents — but
**not segments**: a write to a segment that is not there is
`Error::NoSuchSegment`, because inventing one silently would put it in the
wrong place.

Writing at a level replaces everything beneath it.

### 7.2 Clearing versus nulling

`Message::set_null(path)` writes the HL7 explicit null `""` — "clear this
value". `Message::clear(path)` empties it, as if never sent. Clearing what
is already absent does nothing and succeeds: it does not create the empty
field it would then be emptying, which is what keeps writing an
`Option::None` in struct mode from filling a message with empty components.

Reading the difference back needs a reader that keeps it. `get` and
`get_all` return *decoded* text, and decoded, `""` is the empty string, so
a nulled field and an empty one both read as `Some("")`. The tree (§4.3) is
where they stay apart: the null is a node with `is_null()` true, and an
empty field is no node at all. `raw().query_path_raw()` is the other way,
one level down. This is a property of decoding rather than a gap — but it
is worth knowing before a caller reads `Some("")` as "nothing was sent".

### 7.3 Segments

`append_segment`, `insert_segment`, `remove_segment`, `remove_segments`.
The MSH header is never removed: it carries the delimiters, so a message
without it is not a message.

### 7.4 Building

`Builder::new(version)` starts from a well-formed MSH for that release
(standard delimiters, processing ID `P`, MSH-12 set) and everything else
empty. Failures are collected and reported by `build`, so a chain of calls
stays a chain; `build_valid` additionally refuses a message that fails §8.

The builder takes no timestamp from the clock and invents no control ID:
both are the caller's, because a message that made up its own would be
untraceable and untestable. `builder::acknowledge` builds an `ACK` for a
received message, echoing its control ID into MSA-2 and swapping sender and
receiver.

## 8. Validation (`src/validate.rs`)

`Message::validate` checks a message against its dictionary and returns
diagnostics. It never fails and never changes the message.

**`Severity::Error` — the message contradicts the dictionary it claims:**

| kind | when |
|---|---|
| `Header` | MSH-9.1 or MSH-10 is empty |
| `SegmentMissing` | a segment or group the structure requires is absent |
| `StructureMismatch` | the segments do not fit the structure |
| `ValueFormat` | an `SI`, `NM`, `DT`, `TM`, or `DTM` value is not one |

**`Severity::Warning` — the dictionary does not cover the message:**

| kind | when |
|---|---|
| `Header` | MSH-12 is empty or names an unmodelled release |
| `StructureUnknown` | no grammar for this structure ID |
| `StructureMismatch` | the standard segments fit, but local Z-segments do not (§8.1) |
| `SegmentUnknown` | a segment the dictionary does not define |
| `FieldUnknown` | a field past the end of the segment's definition |
| `ComponentUnknown` | a component past the end of the data type's definition |

Only the value formats with a machine-checkable shape are checked. `ST`,
`TX`, `ID`, `IS` and the rest are constrained by HL7 tables and lengths,
which this crate does not model, so it says nothing about them rather than
guessing. Empty values and explicit nulls are not checked.

### 8.1 Z-segments

A segment whose name begins with `Z` is a local extension that the standard
says nothing about, so neither does this crate: no `SegmentUnknown` for it.
And when a message fails its structure *only* because of Z-segments — the
standard segments fit on their own — the mismatch is a warning, not an
error. Most real interfaces carry one, and rejecting them all would make
strict mode useless.

### 8.2 Strict mode

`Options::strict` runs the same check at parse time and turns any
`Severity::Error` into `Error::Invalid`, carrying the diagnostics. Warnings
never fail a parse: a coverage gap in this crate is not the sender's error.

## 9. Batches and multiple messages (`split_messages` in `src/lib.rs`)

`split_messages` splits input into one string per MSH segment. Batch
envelope segments (FHS, BHS, BTS, FTS) are dropped; each message is then
independent, with its own delimiters and its own release.

## 10. Errors

Every failure is one `Error` variant. The list is short on purpose: reading
is lenient (§2.2, §4.2, §8), so most of what can be *wrong* with a message
is reported rather than raised.

| variant | raised by | when |
|---|---|---|
| `Empty` | parsing | the input held no segments |
| `MissingMsh` | parsing | the first segment is not MSH, so no delimiters were declared |
| `BadMshHeader` | parsing | MSH declared a delimiter set that cannot be used |
| `Path` | any call taking a path | the path is not a path (`PID-0`, an empty segment name) |
| `NoSuchSegment` | `set`, `set_er7`, `set_null` | the path names a segment the message does not have (§7.1) |
| `UnwritablePath` | `set`, `set_er7`, `set_null` | the path names no field, so there is nothing to write |
| `MissingField` | struct mode | a non-optional field's path names nothing (§6.1) |
| `BadValue` | struct mode | a value is present but does not fit the Rust type (§6.1) |
| `Dictionary` | loading a dictionary | the JSON is malformed, or a member is the wrong shape (§3.1) |
| `Invalid` | strict parsing, `build_valid` | validation found `Severity::Error` diagnostics (§8.2) |

`Error::Dictionary` carries a `dictionary::Error` naming the position in the
document (`segments.PID[0]`) and, for a syntax error, the byte offset;
`Error::Invalid` carries every error-level diagnostic, not just the first.
Notably absent: nothing about an unknown segment, an unknown data type, an
unmodelled release, or a message that does not fit its structure — all four
are ordinary readings with a diagnostic attached.

## 11. Limitations

- **Per-release coverage is incremental** (§3.4). v2.5 is complete; other
  releases model the listed differences and inherit the rest.
- **Only four message structures are bundled** (ACK, ADT_A01, ORM_O01,
  ORU_R01). Anything else reads flat, with a `StructureUnknown` warning,
  until a grammar is added to `schemas/` or supplied by a schema (§5).
- **Not a conformance validator.** No HL7 tables, no lengths, no
  conformance profiles, no cardinality beyond what a structure states.
- **Grouping is all-or-nothing** (§4.5), including for messages carrying
  Z-segments.
- **No transport.** MLLP framing is `hl7-2-mllp`'s; files and queues are
  the caller's.
- **`Node::text` decodes escape sequences** that stand for characters;
  formatting escapes (`\.br\`, `\H\`) are left as sent, as `er7` leaves
  them.

## 12. Command line (`src/main.rs`)

```
hl7-v2 [OPTIONS] [FILE]
```

Reads FILE, or standard input when FILE is absent or `-`. Input may hold
one message, several, or a batch; each is handled separately.

Output — the first one given wins, and the default is `--tree`:

| option | prints |
|---|---|
| `-t, --tree` | the message as an indented tree (§4) |
| `-q, --query PATH` | every value at PATH, one per line |
| `-c, --check` | validation diagnostics (§8), or `ok` |
| `-e, --er7` | the message back as ER7 |

Options: `-s, --set PATH=VALUE` (repeatable) and `-n, --null PATH` edit
before printing; `-v, --hl7-version VER` forces a release; `-d,
--dictionary FILE` reads through a schema (§5); `-f, --flat` suppresses
grouping; `-p, --paths` shows each node's path; `-S, --strict` fails on a
validation error; `-o, --output FILE` writes to a file.

Successive trees are separated by a blank line; the value and ER7 outputs
are not, so they can be piped into another tool.

Exit status: 0 on success, 1 on a usage or parse error, 2 when `--check` or
`--strict` found something wrong with the message.

## 13. Traceability

Every section above is pinned by at least one test. This table is the
contract that keeps this document honest: a rule with no test is a rule
nobody is holding, and a test whose name no longer describes what it checks
is a rule that has quietly changed.

Unit tests live in their module's `#[cfg(test)] mod tests`; integration
tests in `tests/integration.rs`; derive tests in
`../hl7-2-derive/tests/derive.rs`.

| § | rule | test |
|---|---|---|
| 2.1 | input normalization | `tests::normalizes_before_parsing` |
| 2.2 | only a bad header fails | `tests::maps_er7_errors_onto_this_crates_type` |
| 3.1 | dictionary format, structure shorthand | `dictionary::tests::reads_structures_including_the_string_shorthand` |
| 3.2 | list replaces, object overrides positions | `dictionary::tests::a_sparse_delta_restates_one_position_and_keeps_the_rest` |
| 3.3 | `inherits`: replace, remove, inherit | `dictionary::tests::a_delta_adds_removes_and_inherits`, `dictionary::tests::layering_over_an_explicit_base_ignores_inherits` |
| 3.4 | every bundled release loads and reads | `version::tests::bundled_dictionaries_all_load`, `integration::every_bundled_release_reads_a_message_that_declares_it`, `integration::a_release_difference_changes_how_a_field_reads` |
| 3.5 | release choice and nearest-older fallback | `version::tests::falls_back_to_the_nearest_older_release`, `message::tests::defaults_the_version_when_msh_12_is_missing_or_odd`, `tests::forcing_a_version_overrides_the_header` |
| 4.1 | structure ID, aliases, MSH-9.3 | `message::tests::reads_version_and_structure_off_the_header` |
| 4.2 | node names, typed and positional | `generic::tests::names_known_types_after_the_type_and_the_rest_positionally` |
| 4.2 | every node carries its path | `generic::tests::every_node_carries_the_path_that_reads_it_back` |
| 4.3 | repetitions are siblings; `repetitions` vs `get_all` | `generic::tests::repetitions_are_separate_siblings`, and the doc test on `Message::repetitions` |
| 4.3 | the explicit null survives | `generic::tests::the_explicit_null_survives` |
| 4.4 | OBX-5 typed by OBX-2 | `generic::tests::obx_5_takes_its_type_from_obx_2`, `dictionary::tests::resolves_obx_5_through_obx_2` |
| 4.5 | grouping, and its all-or-nothing fallback | `structure::tests::*`, `generic::tests::groups_nest_under_the_structure_id`, `message::tests::falls_back_to_a_flat_tree_when_the_structure_does_not_fit` |
| 5 | schema mode end to end | `integration::schema_mode_teaches_the_parser_one_vendors_dialect` |
| 6 | the four attributes | `derive::reads_each_annotated_field_from_its_path`, `derive::nests_structs_and_keeps_the_raw_message` |
| 6.1 | required, optional, repeating, bad values | `typed::tests::reads_scalars_repetitions_and_absences`, `typed::tests::a_required_field_that_is_absent_is_an_error`, `typed::tests::a_value_of_the_wrong_shape_names_the_path`, `typed::tests::booleans_read_the_spellings_senders_use` |
| 6.2 | the raw escape hatch | `integration::struct_mode_keeps_the_generic_escape_hatch_on_the_same_object` |
| 7.1 | set escapes, `set_er7` does not; creation | `message::tests::set_escapes_and_set_er7_does_not`, `message::tests::writes_values_creating_what_is_missing`, `integration::escaped_text_survives_the_whole_trip` |
| 7.2 | clearing versus nulling | `message::tests::distinguishes_clearing_from_nulling` |
| 7.3 | segment add and remove; MSH is kept | `message::tests::adds_and_removes_segments` |
| 7.4 | building, and `acknowledge` | `builder::tests::*`, `integration::building_a_reply_to_a_message`, `integration::a_builder_makes_a_message_from_nothing_that_parses_back` |
| 8 | every diagnostic kind | `validate::tests::*` |
| 8.1 | Z-segment leniency | `validate::tests::unknown_segments_and_fields_are_warnings_not_errors`, `integration::the_samples_parse_and_report_what_they_should` |
| 8.2 | strict mode | `tests::strict_mode_turns_diagnostics_into_a_failure`, `integration::strict_mode_is_the_difference_between_reporting_and_refusing` |
| 9 | batch splitting | `tests::splits_batches_into_messages`, `integration::a_batch_file_becomes_one_message_each` |
| 10 | error variants | `message::tests::reads_values_by_path`, `message::tests::writes_values_creating_what_is_missing`, `dictionary::tests::reports_where_a_malformed_dictionary_is_wrong` |
| 2, 7 | round trip after parse and after edit | `message::tests::round_trips_an_unmodified_message`, `integration::reading_modifying_and_writing_round_trips` |
| 12 | the CLI contract | `integration::the_cli_*` |

## 14. References

- HL7 v2 standards: <https://www.hl7.org/implement/standards/>
- `er7` crate (the encoding layer): <https://crates.io/crates/er7>
- `hl7-2-derive` (the macros): <https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-derive>
- `hl7-2-mllp` (MLLP transport): <https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-mllp>
- Sibling conversion crates: <https://github.com/hl7-rust>

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
