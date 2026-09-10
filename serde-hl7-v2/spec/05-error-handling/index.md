[index](../index.md) → §5 Error handling

# §5 Error handling

## 5.1 Two different kinds of failure

This crate sits between two error domains that must not be conflated:

- **HL7®-level failure** — the ER7 text has no usable MSH header, or (under
  `Options::strict`) fails validation. This is `hl7_2::Error`, entirely
  `hl7-2`'s own concern (its spec §10). `Message::parse` and
  `Message::parse_with_options` forward it unchanged.
- **Serde-level failure** — the *Serde* input does not match the shape in
  [§2](../02-wire-shapes/index.md): a required key is missing, a string
  was expected where an object appeared, a version string names no known
  release. This is reported through the format's own error type
  (`D::Error`/`S::Error`), via the standard `serde::de::Error`
  constructors.

The one place the two meet is `Message::deserialize` (S3): the ER7 text on
the wire is parsed, and if `hl7_2::parse_with_options` rejects it, that
`hl7_2::Error` is surfaced as a Serde error through
`serde::de::Error::custom`, carrying `hl7-2`'s own message text. A
`Deserialize` call therefore never returns an `hl7_2::Error` as such, but
its text is not lost.

## 5.2 Rule S8: unknown fields are ignored, not rejected

Every `visit_map` implementation in this crate matches known keys and
routes everything else to `serde::de::IgnoredAny`, exactly the pattern
serde's own manual-implementation guide shows. This is a direct extension
of `hl7-2`'s own fallback-first principle (its `AGENTS.md`: reading never
fails below the MSH header) into the Serde layer — a producer that has
added a field this crate does not yet know about should not break a
consumer that only needs the fields it already understands.

`T::deserialize` always behaves as if `#[serde(deny_unknown_fields)]` is
absent, for every wrapper type. What exists instead is an opt-in
alternative *type*, not a flag on this one: [§11](../11-strict-mode/index.md)'s
`Strict<T>`.

## 5.3 Rule S9: a missing required field names itself

`missing_field("er7")`, `missing_field("severity")`, and so on — every
required key uses `serde::de::Error::missing_field`, which every Serde
format renders with the field's own name. A caller debugging a hand-written
JSON fixture gets "missing field `er7`," not a generic "invalid input."

## 5.4 A duplicate key is also an error

`serde::de::Error::duplicate_field` fires if the same key appears twice in
one object. This matches the general Serde convention for hand-written
struct visitors and stops a fixture with `"er7"` twice from silently
taking one of them.

## 5.5 An unknown enum string is an error

`Version` (S6) reports an unrecognized release with `invalid_value`,
naming the string; `NodeKind`, `Severity`, and `DiagnosticKind` (S7)
report an unrecognized variant with `unknown_variant`, naming the string
and listing the accepted names. None of the four silently maps an unknown
string to a default.

## 5.6 No panics

No implementation in this crate panics on malformed *input* of either
kind. Handling a caller's malformed data is always a `Result`, never an
`unwrap`/`expect`/`panic!`. The pedantic clippy group and
`#![warn(missing_docs)]`, both required by the checks in
[the index](../index.md#required-checks), are the mechanical backstop.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
