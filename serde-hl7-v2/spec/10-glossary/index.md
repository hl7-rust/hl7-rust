[index](../index.md) → §10 Glossary

# §10 Glossary

Terms specific to this document; for HL7® v2 terms themselves (segment,
field, dictionary, release, the explicit null), see the `hl7-2` spec, and
for ER7 terms the `er7` spec's own glossary.

**Wire shape**
: The JSON-shaped (or any-Serde-format-shaped) representation a type's
  `Serialize` implementation produces and its `Deserialize` implementation
  accepts. Specified per type in [§2](../02-wire-shapes/index.md).

**Format-agnostic**
: Implemented against `serde::Serializer`/`Deserializer`, the trait-level
  abstraction, rather than any one format's concrete API. See
  [§3](../03-dependencies-and-format-agnosticism/index.md).

**Round trip**
: Parse ER7 text, serialize with some Serde format, deserialize back — and
  get a message with the same ER7 text and the same release out. Specified
  in [§4](../04-round-trip-guarantee/index.md).

**Wrapper (or wrapper type)**
: A newtype such as `pub struct Message(pub hl7_2::Message)`, local to this
  crate, that exists so that a foreign trait (`Serialize`) can be
  implemented on a foreign type without violating Rust's orphan rule. See
  [§6.3](../06-ergonomics/index.md).

**Seed**
: A `serde::de::DeserializeSeed` — a deserializer entry point that carries
  state the stateless `Deserialize` trait cannot. `Message::seed` carries
  `hl7_2::Options`. See rule S15.

**Release, as read**
: The `hl7_2::Version` a message was parsed under — MSH-12 resolved
  through `Version::nearest`, or whatever `Options::version` forced. This,
  not MSH-12's literal text, is what `"version"` carries (S3).

**S-numbered rule**
: A normative rule in this specification, numbered to line up with
  `serde-er7`'s where the rule is the same. See the
  [rule index](../index.md#rule-index).

**Strict mode**
: Deserializing through [`Strict<T>`](../11-strict-mode/index.md) rather
  than `T` directly, so an unrecognized key is a `serde::de::Error`
  instead of being ignored. Opt-in, per call.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
