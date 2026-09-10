[index](../index.md) → §9 Roadmap and open questions

# §9 Roadmap and open questions

The crate-level `plan.md`/`tasks.md` convention this workspace uses at
its root is not repeated per crate; the evaluation that led to this crate
lives in [`../../serde-hl7/plan.md`](../../../serde-hl7/plan.md) and
[`../../serde-hl7/tasks.md`](../../../serde-hl7/tasks.md), and what is
still open lives here.

## 9.1 Deliberately deferred

- **`Deserialize` for `Node`** (S14). Needs a public constructor on
  `hl7_2::Node`, which is `hl7-2`'s decision: a node whose name and path
  can disagree with any message is a new kind of value for that crate.
  If `hl7-2` adds one, this crate adds the impl — an additive change
  ([§8.1](../08-versioning-and-compatibility/index.md)) — and S14 is
  retired the way `serde-er7`'s §9.2 retired its open question.
- **Serde for `Dictionary`.** Deferred indefinitely for the reason in
  [§1.4](../01-purpose-and-scope/index.md): the JSON dictionary format
  already exists and is the schema-mode contract.
- **A tree shape for the message itself.** Not deferred — declined, per
  [§2.3](../02-wire-shapes/index.md). Recorded here so the question is
  not reopened by accident.
- **Additional format crates as dev-dependencies**, to demonstrate more
  formats in examples. A nice-to-have; JSON demonstrates every shape in
  [§2](../02-wire-shapes/index.md) without ambiguity.

## 9.2 Open questions

- **Should `Node` omit `"children": []` at a leaf?** S4 says no, for
  schema stability. If real consumers find the size cost matters — a
  200-observation ORU has thousands of leaves — the alternative is a
  second, compact shape behind a distinct wrapper, not a change to this
  one.
- **Should `Message` carry the dictionary's *name*?** `hl7_2::Dictionary`
  has a name (`"2.5"`, or a caller's `"acme"`). Carrying it would let a
  receiver notice that the sender read through a dictionary the receiver
  does not have, and fail loudly rather than silently reading through the
  bundled one. Deferred until a real interface shows the need; it would be
  an additive optional key.

## 9.3 Process

A change here follows the same order `serde-er7` and `hl7-2` use: update
the matching section of this `spec/` first, then the code, then the tests
and the [§7.1](../07-testing-strategy/index.md) table, then `README.md`
if the change is user-facing.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
