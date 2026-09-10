[index](../index.md) → §8 Versioning and compatibility

# §8 Versioning and compatibility

## 8.1 Rule S10: the wire shape is part of the public API

Standard SemVer governs this crate, under the workspace's release runbook
(`spec/release-process/index.md` at the root): while the crate is `0.x`,
a **minor** bump is the one allowed to break. A caller who has stored
`Message` JSON is depending on [§2](../02-wire-shapes/index.md)'s table,
not merely on the Rust types that produce it.

Concretely, all of the following are breaking changes requiring a minor
bump (pre-1.0) and an update to §2 in the same change:

- Renaming, adding, or removing any key — including switching from
  camelCase to snake_case, or from the field's name to the XML element's
  (`controlAct` → `controlActProcess`).
- Changing which keys are required vs. optional on deserialize.
- Making `NullFlavor` reject an unknown code.
- Omitting `null` or `[]` values rather than serializing every key.

## 8.2 What is not part of the compatibility surface

- Internal helper types: the macro, the seed types, `ObjectVisitor`.
- The specific error *message* text for a given failure — the error *kind*
  (`missing_field("root")` naming that field) stays stable; the exact
  wording a format renders is not this crate's.
- Which methods are reached via `Deref`.

## 8.3 Following `hl7-3`'s own versions

This crate depends on `hl7-3` by path and version. A new field on an
`hl7-3` type is a new key here (additive for readers, since S8 ignores
unknown keys, but a shape change under S10 and so a minor bump); a renamed
or removed field is a break here too. The inter-crate version requirement
is checked at release time per the root runbook's step 2.

## 8.4 Rust compatibility

| Item | Value |
| ---- | ----- |
| Edition | 2024 |
| MSRV | 1.96 — inherited from the workspace (`rust-version.workspace = true`) |
| `no_std` | not supported; the wrapped types own `String`s and a `BTreeMap` |

The MSRV comes from the workspace-wide **N-2** policy,
[`spec/rust-msrv-n-minus-2/index.md`](../../../spec/rust-msrv-n-minus-2/index.md).
This crate's floor is the highest of its own, `serde`'s, and `hl7-3`'s;
raising it is a breaking change and lands in a minor bump.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
