[index](../index.md) → §6 Ergonomics: Deref and From

# §6 Ergonomics: Deref and From

## 6.1 Rule S11

Every wrapper type in this crate (`Message`, `Node`, `NodeKind`,
`Diagnostic`, `Severity`, `DiagnosticKind`, `Version`) implements:

- `Deref<Target = hl7_2::X>` and, where the wrapped value is naturally
  mutable (`Message`, `Diagnostic`), `DerefMut`, so `hl7_2::X`'s own
  methods and fields are reachable directly — `message.get("PID-5.1")`,
  `message.tree()`, `node.find("PID")`, `diagnostic.path` — without
  unwrapping `.0` first.
- `From<hl7_2::X> for X` and `From<X> for hl7_2::X`, so converting either
  direction is `.into()` at the call site.
- `Display`, where it reads naturally: `Message` as its ER7, `Node` as its
  text, `Version` and the three enums as their wire string, `Diagnostic`
  as `hl7-2` formats it.

`Message` additionally offers `parse` and `parse_with_options`, thin
wrappers over `hl7_2`'s functions of the same names, so the crate's
flagship path needs only this crate's own type; and `seed` (S15).

## 6.2 Why these are not part of the wire contract

`Deref`/`DerefMut`/`From`/`Display` are Rust-side ergonomics with no wire
representation. They are still part of this crate's public API, and
removing one is a breaking change under normal SemVer rules
([§8](../08-versioning-and-compatibility/index.md)), but they are tracked
separately from S10 because a wire-shape change and an ergonomics change
call for different migration advice.

## 6.3 Why the orphan rule forces the wrapper pattern

`hl7_2::Message` is defined in `hl7-2`; `serde::Serialize` in `serde`.
Neither is local to this crate, so Rust's orphan rule forbids
`impl Serialize for hl7_2::Message` here. The newtype
(`pub struct Message(pub hl7_2::Message)`) is what makes the `impl` legal.
Every wrapper exists for this reason, not merely as a style preference.

## 6.4 Serializing a tree walks it once

`Node::serialize` delegates to a private borrowing wrapper over
`&hl7_2::Node`, so children are serialized by reference rather than by
cloning each subtree into an owning `Node` at every level — a tree of
`n` nodes is visited `n` times, not `n × depth`. The borrowing type is
private: one owning public wrapper per type keeps the API surface small,
and the performance concern is internal.

## 6.5 What `Message` does not derive

`hl7_2::Message` implements `Debug` and `Clone` but not `PartialEq` (it
holds an `Arc<Dictionary>`), so `Message` does not either. Tests compare
`to_er7()` and `version()`, which is what the round-trip guarantee is
stated in terms of ([§4](../04-round-trip-guarantee/index.md)).

## 6.6 `Strict<T>` follows the same convention

[§11](../11-strict-mode/index.md)'s `Strict<T>` carries `Deref`,
`DerefMut`, and `From` both ways, plus a delegating `Serialize`, for the
same reason: a caller reaching for `Strict<Message>` should not lose any
of the ergonomics a plain `Message` already has.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
