[index](../index.md) → §6 Ergonomics: Deref and From

# §6 Ergonomics: Deref and From

## 6.1 Rule S11

Every wrapper type in this crate implements:

- `Deref<Target = hl7_3::X>` and `DerefMut`, so `hl7_3::X`'s own fields
  and methods are reachable directly — `message.interaction_id`,
  `element.attribute("root")`, `act.id.push(...)` — without unwrapping
  `.0` first. `NullFlavor` is `Deref` only: it is an enum with nothing to
  mutate in place.
- `From<hl7_3::X> for X` and `From<X> for hl7_3::X`, so converting either
  direction is `.into()` at the call site.
- `Default`, for every struct wrapper, because every wrapped `hl7-3` struct
  has it — so a test fixture or a hand-built value starts from
  `Act::default()`.
- `Display` for `NullFlavor`, as its code.

`Message` additionally offers `parse`, a thin wrapper over
`hl7_3::message::parse`, so the crate's flagship path needs only this
crate's own type.

## 6.2 Why these are not part of the wire contract

`Deref`/`DerefMut`/`From`/`Default` are Rust-side ergonomics with no wire
representation. They are still public API, and removing one is a breaking
change under normal SemVer rules
([§8](../08-versioning-and-compatibility/index.md)), but they are tracked
separately from S10.

## 6.3 Why the orphan rule forces the wrapper pattern

`hl7_3::Message` is defined in `hl7-3`; `serde::Serialize` in `serde`.
Neither is local to this crate, so Rust's orphan rule forbids
`impl Serialize for hl7_3::Message` here. The newtype is what makes the
`impl` legal.

## 6.4 Nested values are cloned into wrappers on serialize

Serializing an `Act` clones each `Ii` in its `id` list into an `Ii`
wrapper to hand to `serialize_field`; serializing an `Element` clones each
child. This is the same simplicity trade-off `serde-er7` makes (its spec
§6.4): a v3 payload is kilobytes, and a second, lifetime-parameterized
borrowing family would double the crate's type surface to save it. If
profiling ever shows this mattering, the fix is additive.

## 6.5 `Strict<T>` follows the same convention

[§11](../11-strict-mode/index.md)'s `Strict<T>` carries `Deref`,
`DerefMut`, `Default`, and `From` both ways, plus a delegating
`Serialize`.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
