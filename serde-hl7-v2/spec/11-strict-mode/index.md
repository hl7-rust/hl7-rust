[index](../index.md) → §11 Strict mode

# §11 Strict mode: an opt-in `deny_unknown_fields`

## 11.1 What this section covers

`serde-er7` added `Strict<T>` after shipping, to catch typos in
hand-written fixtures (its spec §11). This crate ships with it from the
start, for the same reason: `Message` has one optional key,
`"version"`, and a typo on it (`"verison"`) is exactly the case S8's
tolerance hides — the key is silently ignored, the release silently falls
back to MSH-12, and no error of any kind says so.

## 11.2 Rule S13: `Strict<T>` rejects unknown fields; `T` alone still does not

`Strict<T>` is a wrapper type, `pub struct Strict<T>(pub T)`, implementing
`Deserialize` for `T` in `{Message, Diagnostic}` — the two object-shaped
types this crate can deserialize. Where the ordinary `T::deserialize`
ignores a key it does not recognize (S8), `Strict::<T>::deserialize`
reports it with `serde::de::Error::unknown_field`, naming the key and the
field names it could have been.

`Version`, `NodeKind`, `Severity`, and `DiagnosticKind` are bare strings
that already reject an unrecognized value either way, so there is no
`Strict` form for them. `Node` has no `Deserialize` at all (S14).
`Message`'s and `Diagnostic`'s nested values are all strings of those
kinds, so — unlike `serde-er7`'s `Message`, which nests objects three
deep — there is no nested-object strictness to carry here; the seeded
path has `MessageSeed::strict` for the same effect (S15).

## 11.3 This does not change what `T::deserialize` accepts

**S8 is unchanged.** `serde_json::from_str::<Message>(json)` behaves as
before — tolerant, ignoring unknown keys — because `Strict<T>` is a
distinct, additive type with its own `Deserialize` impl, not a flag on
the existing one. [§2](../02-wire-shapes/index.md) is unchanged:
`Strict<T>` accepts and requires exactly what `T` does, plus the one
additional rejection.

## 11.4 Ergonomics

`Strict<T>` follows [§6](../06-ergonomics/index.md) (rule S11): `Deref`,
`DerefMut`, and `From` both ways, plus a `Serialize` impl delegating to
`T`'s own.

## 11.5 Why not a global flag

A thread-local or ambient flag, set before calling the ordinary
`T::deserialize`, was rejected for the reason `serde-er7` gives: a caller
who forgets to unset it leaves every later, unrelated `T::deserialize` in
the process silently strict. `Strict<T>` is requested at the one call site
that wants it and nowhere else.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
