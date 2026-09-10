[index](../index.md) → §11 Strict mode

# §11 Strict mode: an opt-in `deny_unknown_fields`

## 11.1 What this section covers

Almost every key in this crate is optional (S4). That is right for reading
what a sender sent, and it is exactly the situation in which a typo in a
hand-written fixture is silent: `"extention"` for `"extension"` is an
unknown key, ignored, and the identifier simply has no extension, with no
error of any kind. `Strict<T>` is the opt-in that reports it.

## 11.2 Rule S13: `Strict<T>` rejects unknown fields, at any depth

`Strict<T>` is a wrapper type, `pub struct Strict<T>(pub T)`, implementing
`Deserialize` for every object-shaped `T` in this crate — `Message`,
`ControlAct`, `Element`, the five data-type structs, and the six RIM
classes. Where the ordinary `T::deserialize` ignores a key it does not
recognize (S8), `Strict::<T>::deserialize` reports it with
`serde::de::Error::unknown_field`, naming the key and listing the keys it
could have been.

`NullFlavor` is a bare string that accepts any code by design (S6), so
there is no `Strict<NullFlavor>`; nothing about it changes between the two
modes.

**Strictness nests.** `Strict<Message>` rejects an unknown key not only on
the message object but inside its `Ii` identifiers, inside `controlAct`,
inside that act's `Cd`, and inside every `Element` of the `sender`,
`receiver`, and `domain` trees, however deep. Internally every object
wrapper implements a crate-private `StrictDeserialize` trait with a
`strict` flag, and a parent reads each nested value through one of three
`DeserializeSeed` types — for a value, an `Option` of one, or a `Vec` of
them — that carry the flag down, rather than through the nested type's
own always-tolerant `Deserialize` impl. Because every wrapper is written
by the same macro (S15), no type can be left out of this by accident.

What strictness does **not** check is the contents of
`Element::attributes`: attribute names are the document's data, and this
crate has no way to know what attributes an element should have.
`Strict<Element>` checks the element object's own four keys and leaves the
attribute map alone.

## 11.3 This does not change what `T::deserialize` accepts

**S8 is unchanged.** `serde_json::from_str::<Message>(json)` behaves as
before — tolerant, ignoring unknown keys — because `Strict<T>` is a
distinct, additive type with its own `Deserialize` impl, not a flag on
the existing one. [§2](../02-wire-shapes/index.md) is unchanged:
`Strict<T>` accepts and requires exactly what `T` does, plus the one
additional rejection.

## 11.4 Ergonomics

`Strict<T>` follows [§6](../06-ergonomics/index.md) (rule S11): `Deref`,
`DerefMut`, `Default`, and `From` both ways — the concrete
`From<Strict<T>> for T` impls are written by the macro alongside each
type, since the orphan rule forbids a blanket one — plus a `Serialize`
impl delegating to `T`'s own.

## 11.5 Why not a global flag

A thread-local or ambient flag was rejected for the reason `serde-er7`
gives (its spec §11.5): a caller who forgets to unset it leaves every
later, unrelated `T::deserialize` in the process silently strict.
`Strict<T>` is requested at the one call site that wants it and nowhere
else, which is what an ordinary Rust type parameter is for.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
