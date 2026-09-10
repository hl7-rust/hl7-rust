[index](../index.md) → §2 Wire shapes

# §2 Wire shapes

This is the normative table: what each wrapper type must serialize as, and
must accept when deserializing. A change to any shape here is a breaking
change (see [§8](../08-versioning-and-compatibility/index.md), rule S10).

## 2.1 The table

| Type | Shape | Example | Deserializes |
|------|-------|---------|--------------|
| `Message` | object, fields `"version"`, `"er7"` | `{"version": "2.5", "er7": "MSH\|^~\\&\|LAB\rPID\|1"}` | yes (S3) |
| `Node` | object, fields `"name"`, `"path"`, `"kind"`, `"text"`, `"null"`, `"children"` | `{"name": "PID.5", "path": "PID[1]-5[1]", "kind": "Field", "text": "SMITH^JOHN", "null": false, "children": [...]}` | no (S14) |
| `NodeKind` | one of the strings `"Group"`, `"Segment"`, `"Field"`, `"Component"`, `"Subcomponent"` | `"Field"` | yes |
| `Diagnostic` | object, fields `"severity"`, `"kind"`, `"path"`, `"detail"` | `{"severity": "Error", "kind": "ValueFormat", "path": "OBX[1]-5[1]", "detail": "..."}` | yes (S5) |
| `Severity` | `"Error"` or `"Warning"` | `"Warning"` | yes |
| `DiagnosticKind` | the variant name: `"Header"`, `"StructureUnknown"`, `"StructureMismatch"`, `"SegmentMissing"`, `"SegmentUnknown"`, `"FieldUnknown"`, `"ComponentUnknown"`, `"ValueFormat"` | `"SegmentMissing"` | yes |
| `Version` | the release as MSH-12.1 spells it | `"2.5.1"` | yes (S6) |

## 2.2 Rules

- **S3**: `Message` serializes as its release (`hl7_2::Message::version`,
  through `Version`) and its ER7 text (`hl7_2::Message::to_er7`).
  Deserializing reads `"er7"` and parses it with
  `hl7_2::parse_with_options`, the version from `"version"` pinned
  through `Options::version`, so the value that comes back has resolved
  its dictionary exactly as a freshly parsed message would. The dictionary
  is never on the wire: it is derived state, large, and — when it is a
  caller's own — not something a JSON document should be able to swap
  under a receiver. `"version"` is the release the message was *read as*,
  which `Options::version` or `Version::nearest` may have made different
  from what MSH-12 literally says; carrying it is what makes
  `Message::version()` survive the round trip.

- **S4**: `Node` serializes every one of its six accessors, always, at
  every node — `"children": []` at a leaf and `"null": false` almost
  everywhere included. A fixed schema is worth more to a consumer in a
  typed language, or writing a JSON-path query, than the bytes omitting
  the defaults would save. `"kind"` goes through `NodeKind` (S7);
  `"path"` is the `er7` path, empty at the root.

- **S5**: `Diagnostic` serializes its four public fields under their own
  names, `severity` and `kind` through `Severity` and `DiagnosticKind`
  (S7). All four are required on deserialize: a finding with no severity
  or no kind is not a finding.

- **S6**: `Version` serializes through `hl7_2::Version::as_str` and
  deserializes through `hl7_2::Version::parse`. Only a known release is
  accepted, spelled exactly (`Version::parse` trims surrounding
  whitespace, nothing more); a string such as `"2.5.2"` is an
  `invalid_value` error naming it. The tolerant nearest-release resolution
  `hl7_2::parse` applies to MSH-12 belongs to parsing, not to a field
  that says which release a message *was* read as.

- **S7**: `NodeKind`, `Severity`, and `DiagnosticKind` serialize as their
  Rust variant identifier via `serialize_str`, not
  `serialize_unit_variant` — a value that reads the same in every format,
  at the cost of the compact index a binary format could otherwise use
  (the same trade `serde-er7` makes for `Terminator`). An unrecognized
  string is an `unknown_variant` error listing the accepted names.

- **S14**: `Node` has no `Deserialize` impl. `hl7_2::Node` has no public
  constructor — `hl7-2` builds trees only from a parsed message, so that a
  node's name, path, and text can never disagree with the message they
  describe. Inventing a constructor here would mean inventing one in
  `hl7-2`, which is that crate's decision to make
  ([§9](../09-roadmap-and-open-questions/index.md)). The way back to a
  tree is `Message` → `.tree()`.

- **S15**: the plain `Message::deserialize` has no way to be handed
  `hl7_2::Options`, so it reads through the bundled dictionary for the
  wire's release. `Message::seed(&options)` is a `DeserializeSeed` that
  reads under the caller's options instead — a custom dictionary (schema
  mode), strict validation, or a forced release. When `options.version` is
  `Some`, it wins over the wire's `"version"`, exactly as it wins over
  MSH-12 in `hl7_2::parse_with_options`; when `None`, the wire's version
  is used; when the wire has no `"version"` either, MSH-12 decides, as in
  plain `hl7_2::parse`. `MessageSeed::strict` gives the seeded path the
  same unknown-key rejection `Strict<Message>` gives the plain one.

## 2.3 Why text, not a tree

The obvious shape for a message is a tree, and that shape exists — in
`serde-er7`, over `er7`'s types, which is where segments, fields, and
subcomponents are defined. `hl7-2` adds a dictionary to that tree, not a
different tree. Serializing `hl7_2::Message` as a second copy of the same
segment/field/repetition/component nesting would mean two crates
specifying, testing, and versioning one wire shape for one set of bytes.

ER7 text is the one form every HL7® v2 system already agrees on. It is
lossless by definition, it round-trips through `er7` byte for byte, it is
compact, and a caller who wants the structural tree has it in one call on
either side of the wire: `serde_er7::Message(message.raw().clone())`.
What only this crate can offer — because only `hl7-2` has the dictionary
— is `Node`, and that is where this crate's tree-shaped effort goes.

## 2.4 Deserializing: what is required and what is optional

- `Message` requires `"er7"`; `"version"` is optional (S15).
- `Diagnostic` requires all four of `"severity"`, `"kind"`, `"path"`,
  `"detail"`.
- Every object ignores keys it does not recognize (S8), so a value
  produced by a newer version of this crate that has grown an additional
  field can still be read by an older one, as long as no currently-required
  key changes shape. A caller who wants the opposite opts into it per call
  with [`Strict<T>`](../11-strict-mode/index.md) (S13).

## 2.5 Why not derive

`#[derive(Serialize)]` cannot be placed on a foreign type, and a derive on
the newtype wrapper would produce a one-field wrapper object, not the
shapes above. More to the point, none of the shapes here *is* the Rust
layout: `Message` serializes two things it computes, not its fields;
`Node` serializes accessors over private fields; the enums are strings.
Every implementation is therefore hand-written against the `Serializer`/
`Deserializer` traits directly, following serde's own manual-implementation
guide.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
