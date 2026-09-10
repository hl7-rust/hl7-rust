[index](../index.md) → §8 Versioning and compatibility

# §8 Versioning and compatibility

## 8.1 Rule S10: the wire shape is part of the public API

Standard SemVer governs this crate, under the workspace's release runbook
(`spec/release-process/index.md` at the root): while the crate is `0.x`,
a **minor** bump is the one allowed to break. What makes a Serde crate
slightly unusual is that its *wire shape* — [§2](../02-wire-shapes/index.md)'s
table — is as much a compatibility surface as its Rust API: a caller who
has stored `Message` JSON on disk, in a database, or in a queue is
depending on that shape, not merely on the Rust types that produce it.

Concretely, all of the following are breaking changes requiring a minor
bump (pre-1.0) and an update to §2 in the same change:

- Changing any type's shape (object ↔ string, added/removed/renamed key).
- Changing which keys are required vs. optional on deserialize.
- Changing an enum's wire representation from variant-name strings to
  anything else.
- Changing `Message` from ER7 text to a tree (this would also contradict
  [§2.3](../02-wire-shapes/index.md), so it should never happen without
  that section's argument being answered first).
- Adding a `Deserialize` impl for `Node` is *not* breaking — it is
  additive — but it changes S14 and needs the spec updated first
  ([§9](../09-roadmap-and-open-questions/index.md)).

## 8.2 What is not part of the compatibility surface

- Internal helper functions and private visitor types.
- The specific error *message* text for a given failure — the error
  *kind* (`missing_field("er7")` naming that field) is meaningful and
  stays stable; the exact wording a format renders is not this crate's.
- The text of `Diagnostic::detail`, which is `hl7-2`'s.
- Which methods are reached via `Deref` — `hl7-2`'s API can grow without
  a version bump here.
- The set of releases `Version` accepts, which is `hl7_2::version::ALL`:
  a new release in `hl7-2` becomes a valid wire value here automatically,
  and an older reader that lacks it rejects it with `invalid_value`, which
  is the right answer for a release it cannot read.

## 8.3 Following `hl7-2`'s own versions

This crate depends on `hl7-2` by path and version. A breaking change in
`hl7-2`'s public types — a new `Diagnostic` field, a renamed `Kind`
variant — requires updating this crate's wire shape to match and bumping
this crate accordingly, even if nothing in this crate's own code changed:
the wire shape is defined in terms of `hl7-2`'s types, so a break there is
a break here too. The inter-crate version requirement is checked at
release time per the root runbook's step 2.

## 8.4 Rust compatibility

| Item | Value |
| ---- | ----- |
| Edition | 2024 |
| MSRV | 1.96 — inherited from the workspace (`rust-version.workspace = true`) |
| `no_std` | not supported; the wrapped types own `String`s and an `Arc` |

The MSRV comes from the workspace-wide **N-2** policy,
[`spec/rust-msrv-n-minus-2/index.md`](../../../spec/rust-msrv-n-minus-2/index.md).
Two floors constrain this crate that do not constrain `hl7-2`: `serde`'s
own MSRV, and `hl7-2`'s. This crate's floor is the highest of the three;
if `serde` ever declares an MSRV above N-2, that becomes the real floor
and this section says so explicitly. Raising the MSRV is a breaking
change, so it lands in a minor bump.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
