# Specification: HL7® v2 from XSD into JSON dictionary

This is the single source of truth for what this crate does and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Every rule below is exercised by a unit test (next to the code that
implements it, in that module's `#[cfg(test)]` block) or an integration test
(`tests/integration.rs`, against `samples/example/`). A change to this
document that isn't backed by a test, or a code change that isn't reflected
here, is a bug.

## 0. Relationship to the rest of the family

```
hl7-2-from-xsd-into-json-dictionary     schemas -> dictionary (this crate)
              |
              v
hl7-2       reads the dictionary (its spec/index.md §3)
              |
              +-- hl7-2-from-er7-into-xml     converts against it
              +-- ...
```

This crate runs at authoring time and writes a file. Nothing reads its
output at runtime except through `hl7_2::Dictionary::from_json`, so the
contract between the two is that format and nothing more: this crate does
not depend on `hl7-2` to build, and depends on it only to test that what
it writes loads (§7).

## 1. Scope

**In scope:** the HL7 v2.xml schema set — composite data types, segment
field lists with cardinality, and abstract message structures with their
segment groups.

**Out of scope:** validating a document against a schema, general XML Schema
(`xsd:restriction` facets, `xsd:choice`, `xsd:any`, imports, redefines),
resolving `<xsd:include>` beyond reading the base-file prefix out of it, and
anything about ER7. A construct not listed in §3 is skipped, never an error:
a schema that carries more than this crate models still converts, and what
it models is simply less.

## 2. Input

A **schema directory** holds:

| file | contributes |
|---|---|
| `<prefix>_types.xsd` | composite data types and their components |
| `<prefix>_fields.xsd` | every `SEG.n` element and the data type it carries |
| `<prefix>_segments.xsd` | each segment's field list, with cardinality |
| one file per structure | one abstract message structure each |

A file is a **structure schema** when its name ends in `.xsd` and does not
end in `_types.xsd`, `_fields.xsd`, or `_segments.xsd`. Its structure ID is
its filename without the extension (`ADT_A05.xsd` -> `ADT_A05`), and it must
declare a `<ID>.CONTENT` complexType or conversion fails.

`<prefix>` is read from the first structure schema that includes a
`<prefix>_segments.xsd` (`schema::included_prefix`), so the directory name
carries no meaning. A directory where no structure schema includes one is an
error: the base files cannot be found.

### 2.1 The XML this crate reads (`hl7-2-xml-lite-helper`, re-exported as `xml`)

Elements, attributes, and nesting are retained; character data, comments,
processing instructions, CDATA, and a `DOCTYPE` are skipped, because no
construct in §3 carries text. Attribute values decode the five predefined
entities and numeric character references.

Namespace prefixes are **not** resolved: `xsd:complexType`,
`xs:complexType`, and `complexType` are the same element. The names this
crate looks for exist only in XML Schema, so the local name is unambiguous
in any real schema.

## 3. What is read, and what it becomes

### 3.1 Cardinality

Two booleans, from every `<xsd:element>` in a sequence:

- `required` is `minOccurs >= 1`. **An absent `minOccurs` is 1**, per XML
  Schema, which is what makes `<xsd:element ref="MSH"/>` required.
- `repeats` is `maxOccurs` being `unbounded` or greater than 1. An absent
  `maxOccurs` is 1.

An unparseable `minOccurs` reads as required and an unparseable `maxOccurs`
as non-repeating — the stricter reading in both cases.

The exact upper bound is dropped: nothing downstream distinguishes "at most
10" from "unbounded", so both are `repeats: true`.

### 3.2 Data types (`types` — `schema::Types`)

A composite is an `<xsd:complexType name="XPN">` whose sequence refs
`XPN.1`...`XPN.n`. Each ref resolves to a data type by following its
declared type through `.CONTENT` wrappers:

```
XPN.1  -> type="XPN.1.CONTENT" -> complexContent extension base="FN" -> FN
HD.1   -> type="HD.1.CONTENT"  -> simpleContent  extension base="IS" -> IS
MSH.3  -> type="HD"                                                  -> HD
```

The walk stops at a type that has children of its own — a composite — and
guards against extensions that cycle. Both `complexContent` and
`simpleContent` extensions are followed: the first is how a component names
its own composite, the second is how a leaf names its primitive.

Only names without a dot are emitted as data types. An HL7 data type is a
short alphanumeric name (`XPN`, `CX`), never a dotted one, so the
`PID.CONTENT` and `XPN.1.CONTENT` entries that share the table are filtered
out. An element whose type cannot be resolved contributes the empty string,
which the dictionary format reads as unstated.

### 3.3 Segments (`segments` — `schema::segments`)

Every `<SEG>.CONTENT` complexType in the segments schema becomes one entry.
A field's position is **its field number**, taken from the ref's numeric
suffix, not its position in the sequence — so a schema that declares `PID.1`,
`PID.3` and `PID.5` produces a five-entry list whose second and fourth
entries are unstated.

A `.CONTENT` name that still contains a dot is not a segment and is skipped.
A segment whose sequence is empty is left out entirely: `Hxx.CONTENT` is
HL7's placeholder for an arbitrary Z-segment, and a dictionary that does not
mention a segment already means "unknown", which is what a wildcard is.

### 3.4 Structures (`structures` — `schema::structure`)

The root complexType is `<ID>.CONTENT`. Each ref in its sequence is a
**group** when the same file declares a `.CONTENT` complexType of that name,
and a **segment** otherwise; groups recurse, and a ref that would reopen a
group already being read is treated as a segment so a self-referential
schema cannot loop.

A group is named without its structure prefix — `ADT_A05.PROCEDURE` becomes
`PROCEDURE` — matching how the reading crates name group nodes: the
structure ID, a dot, then the group name.

## 4. What the schemas cannot say (`Options`)

Two things are not derivable from a schema directory and are supplied by the
caller:

- **`aliases`** — which trigger events arrive carried by another message's
  structure (`ADT_A28` -> `ADT_A05`). A directory holds `ADT_A05.xsd` but
  never says that an `ADT^A28` is one.
- **`inherits`** — a bundled release to layer the document over. Without it
  the document stands alone, which a full set of schemas justifies.

`name` is recorded in the description only; the reading crate takes a
dictionary's name as an argument. `version` defaults to the base-file prefix
with underscores as dots (`2_5_1` -> `2.5.1`) and can be overridden.
`structures` restricts conversion to named structures, and naming one the
directory has no schema for is an error rather than a silent omission.

## 5. Output (`src/dictionary.rs`)

`hl7-2`'s dictionary format, indented two spaces, with a trailing
newline, so a generated file and a hand-written one diff against each other.
Members are written in the order `version`, `description`, `inherits`,
`types`, `segments`, `aliases`, `structures`, and every map is in sorted key
order, so the output is deterministic: the same schemas always produce the
same bytes.

A field is written as a bare data type name when the schema said nothing
beyond the type, and as an object when it did — `{"type": "XTN", "repeats":
true}`. Absent flags are omitted rather than written false.

The `description` records the tool, the source directory, and the base
prefix, and says not to hand-edit. A generated dictionary is a build
artifact; the schemas are the source.

## 6. Dependencies

One: `hl7-2-xml-lite-helper`, the small, dependency-free XML reader shared
across five crates — this one, `hl7-2-soap`, `hl7-2-from-xml-into-er7`,
`hl7-3`, and `hl7-3-soap` (§2.1) — re-exported as `xml`. Writing a
dictionary needs a JSON writer (§5), which is small and
specific enough to the one shape involved to live here rather than pull in
a general-purpose one. `hl7-2` is a dev-dependency only, for §7 — it is
not needed to build this crate.

## 7. Testing

Beyond the per-module unit tests, `tests/integration.rs` converts
`samples/example/` — a small schema set shaped like a real one, exercising
primitive components, a nested composite, a skipped field number, an empty
placeholder segment, and a group — and then **loads the result with
`hl7_2::Dictionary::from_json`**, checking that the data types,
cardinality, and structure all survive. That test is the contract in §0: the
only thing that makes this crate's output correct is that the crate meant to
read it can.

## 8. Command-line behavior (`src/main.rs`)

```text
hl7-2-from-xsd-into-json-dictionary [OPTIONS] <DIRECTORY>
```

`-o/--output` writes to a file instead of stdout. `--name`, `--version-id`,
`--inherits`, `--alias CODE_TRIGGER=ID` (repeatable) and `--structure ID`
(repeatable) set the corresponding `Options`. `-h/--help` and `-V/--version`
print and exit successfully. Any error prints to stderr prefixed with the
program name and exits with a failure code; nothing is written to the output
file when conversion fails.

`--version-id` is spelled that way because `-V/--version` is the program's
own version, and the two would otherwise collide.

## 9. References

- HL7 v2.xml encoding syntax, and the published schema sets
- `hl7-2`, `spec/index.md` §3 — the dictionary format
- `hl7-2-from-er7-into-xml`, `spec/index.md` §4a — schema mode, which is
  what the cardinality in §3.1 is for
- W3C XML Schema Part 1, §3.9 (`minOccurs`/`maxOccurs` defaults)

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
