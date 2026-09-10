[index](../index.md) → §10 Glossary

# §10 Glossary

Terms specific to this document; for HL7® v3 terms themselves (RIM, act,
entity, role, participation, II, CD, nullFlavor, the three-level envelope),
see the `hl7-3` spec.

**Wire shape**
: The JSON-shaped (or any-Serde-format-shaped) representation a type's
  `Serialize` implementation produces and its `Deserialize` implementation
  accepts. Specified per type in [§2](../02-wire-shapes/index.md).

**Format-agnostic**
: Implemented against `serde::Serializer`/`Deserializer`, the trait-level
  abstraction, rather than any one format's concrete API. See
  [§3](../03-dependencies-and-format-agnosticism/index.md).

**Round trip**
: Serialize a value with some Serde format, deserialize back, and get an
  equal value. Specified in [§4](../04-round-trip-guarantee/index.md).
  Not to be confused with an XML round trip, which this crate does not
  promise.

**Wrapper (or wrapper type)**
: A newtype such as `pub struct Act(pub hl7_3::rim::Act)`, local to this
  crate, that exists so that a foreign trait (`Serialize`) can be
  implemented on a foreign type without violating Rust's orphan rule. See
  [§6.3](../06-ergonomics/index.md).

**Mode**
: The one-word declaration in an `object_wrapper!` field list — `req`,
  `opt`, `dflt`, `reqw`, `optw`, `vecw` — that fixes how a field
  serializes, what it reads as when absent, and whether strictness is
  carried into it. See rule S15 and `src/object.rs`.

**Seed**
: A `serde::de::DeserializeSeed` — a deserializer entry point that carries
  state the stateless `Deserialize` trait cannot. This crate's three seeds
  carry the strict flag into a nested value, an `Option` of one, or a
  `Vec` of them. See [§11.2](../11-strict-mode/index.md).

**S-numbered rule**
: A normative rule in this specification, numbered to line up with
  `serde-hl7-v2`'s and `serde-er7`'s where the rule is the same. See the
  [rule index](../index.md#rule-index).

**Strict mode**
: Deserializing through [`Strict<T>`](../11-strict-mode/index.md) rather
  than `T` directly, so an unrecognized key is a `serde::de::Error`
  instead of being ignored. Opt-in, per call; nests to any depth.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
