[index](../index.md) → §5 Error handling

# §5 Error handling

## 5.1 Two different kinds of failure

- **XML-level failure** — the input is not well-formed XML. This is
  `hl7_3::Error`, entirely `hl7-3`'s own concern, and `Message::parse`
  forwards it unchanged.
- **Serde-level failure** — the *Serde* input does not match the shape in
  [§2](../02-wire-shapes/index.md): a required key is missing, an array
  was expected where a string appeared. This is reported through the
  format's own error type via the standard `serde::de::Error`
  constructors.

Unlike `serde-hl7-v2`, whose `Message` re-parses ER7 on the way in, no
`Deserialize` here re-parses anything: every value is rebuilt field by
field from the wire, so an `hl7_3::Error` never surfaces from a
`Deserialize` call.

## 5.2 Rule S8: unknown fields are ignored, not rejected

Every generated `visit_map` matches known keys and routes everything else
to `serde::de::IgnoredAny`. This extends `hl7-3`'s own tolerance (its spec
§5.2) into the Serde layer: a producer that has added a field this crate
does not yet know about should not break a consumer that only needs the
fields it already understands. `T::deserialize` always behaves as if
`#[serde(deny_unknown_fields)]` is absent; the opt-in alternative is
[§11](../11-strict-mode/index.md)'s `Strict<T>`.

## 5.3 Rule S9: a missing required field names itself

`missing_field("classCode")`, `missing_field("root")`, and so on — every
required key (S5) uses `serde::de::Error::missing_field`, which every
Serde format renders with the field's own name.

## 5.4 A duplicate key is also an error

`serde::de::Error::duplicate_field` fires if the same key appears twice in
one object, so a fixture with `"code"` twice does not silently take one of
them.

## 5.5 `NullFlavor` does not fail on a string

Rule S6: any string is a valid `NullFlavor`. Only a non-string is an
error. This is the one place this crate is *more* tolerant than a
derive-generated enum would be, deliberately, to match `hl7-3`.

## 5.6 No panics

No implementation in this crate panics on malformed *input* of either
kind. The pedantic clippy group and `#![warn(missing_docs)]`, both
required by the checks in [the index](../index.md#required-checks), are
the mechanical backstop.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
