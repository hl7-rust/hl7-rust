[index](../index.md) → §9 Roadmap and open questions

# §9 Roadmap and open questions

The evaluation that led to this crate lives in
[`../../serde-hl7/plan.md`](../../../serde-hl7/plan.md) and
[`../../serde-hl7/tasks.md`](../../../serde-hl7/tasks.md); what is still
open lives here.

## 9.1 Deliberately deferred

- **Following `hl7-3`'s growth.** `hl7-3` describes itself as a
  foundation, not a full implementation; as it models more of the data
  type hierarchy or more of the envelope, each new public type gets a
  wrapper here, written with `object_wrapper!` (S15), in the same release
  that depends on the new `hl7-3`.
- **Borrowing (zero-clone) wrapper types** — see
  [§6.4](../06-ergonomics/index.md). Deferred until a real workload shows
  the clone cost mattering.
- **Additional format crates as dev-dependencies**, to demonstrate more
  formats in examples. JSON demonstrates every shape in
  [§2](../02-wire-shapes/index.md) without ambiguity.

## 9.2 Open questions

- **Should `Element::attributes` be strict-checked against anything?** No
  — attribute names are the document's data, not this crate's keys, so
  `Strict<Element>` checks the four keys of the element object and leaves
  the attribute map alone. Recorded here because the distinction surprises
  people (`examples/catch_a_typo_with_strict.rs` shows it).
- **Should `Message` carry the root element's tag** — the interaction's
  wire name? `hl7-3` does not currently expose it as a field (its
  `message.rs` says so), so there is nothing to serialize. If `hl7-3` adds
  it, it is one more `opt String` line here.

## 9.3 Process

A change here follows the same order `serde-er7`, `serde-hl7-v2`, and
`hl7-3` use: update the matching section of this `spec/` first, then the
code, then the tests and the [§7.1](../07-testing-strategy/index.md)
table, then `README.md` if the change is user-facing.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
