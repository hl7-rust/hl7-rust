# serde-hl7-v3 specification

This is the normative specification for `serde-hl7-v3` — the source of
truth for behaviour. Where the rustdoc, README, or any other document
disagrees with this one, this one is right; where this document and a test
disagree, that is a bug in whichever one is wrong, and the fix updates both
together.

This crate is a companion to [`hl7-3`](../../hl7-3/spec/index.md) and its
own spec (hereafter "the `hl7-3` spec"), and a sibling of
[`serde-hl7-v2`](../../serde-hl7-v2/spec/index.md) and of
[`serde-er7`](https://github.com/er7-rust/er7-rust/tree/main/serde-er7),
whose specifications it follows section for section. This document does
not restate anything the `hl7-3` spec already settles — the RIM, the data
types, the three-level envelope, what a missing wrapper means — it only
specifies the one thing this crate adds: Serde support for the public types
`hl7-3` already defines.

## Sections

| § | Title | Covers |
|---|-------|--------|
| [1](01-purpose-and-scope/index.md) | Purpose and scope | What this crate is, why it is a separate crate, what it does not do |
| [2](02-wire-shapes/index.md) | Wire shapes | The exact `Serialize`/`Deserialize` shape for every wrapped type |
| [3](03-dependencies-and-format-agnosticism/index.md) | Dependencies and format-agnosticism | Why exactly two dependencies, and why no format is named in the library code |
| [4](04-round-trip-guarantee/index.md) | The round-trip guarantee | What survives a Serde round trip, and what is deliberately not promised |
| [5](05-error-handling/index.md) | Error handling | How malformed input is reported, and by what mechanism |
| [6](06-ergonomics/index.md) | Ergonomics: Deref and From | The non-normative conveniences layered over the wrapper types |
| [7](07-testing-strategy/index.md) | Testing strategy | Unit, doc, and integration tests, and what each layer is responsible for catching |
| [8](08-versioning-and-compatibility/index.md) | Versioning and compatibility | SemVer commitments, the wire-shape table as a compatibility surface, the N-2 Rust MSRV |
| [9](09-roadmap-and-open-questions/index.md) | Roadmap and open questions | What is deliberately deferred, and why |
| [10](10-glossary/index.md) | Glossary | Terms this document uses that are specific to this crate |
| [11](11-strict-mode/index.md) | Strict mode | The opt-in `Strict<T>` wrapper that rejects unknown fields, nested to any depth |

## Rule index

Every normative rule below carries an ID (`S1`, `S2`, ...) so it can be
cited from code, tests, and commit messages without restating it. The
numbering lines up with `serde-hl7-v2`'s and `serde-er7`'s where the rule
is the same one (S1, S2, S8–S13).

| ID | One-line statement | Section |
|----|---------------------|---------|
| S1 | Exactly two runtime dependencies: `serde` and `hl7-3` | [§3](03-dependencies-and-format-agnosticism/index.md) |
| S2 | No format-specific crate is a runtime dependency | [§3](03-dependencies-and-format-agnosticism/index.md) |
| S3 | Every struct wrapper serializes as an object whose keys are the `hl7-3` field names in lowerCamelCase, every key always present, in declaration order | [§2](02-wire-shapes/index.md) |
| S4 | An `Option` field serializes as its value or `null` and reads `None` when absent; a `Vec` field serializes as an array and reads empty when absent; `Element`'s `attributes` and `text` read empty when absent | [§2](02-wire-shapes/index.md) |
| S5 | A non-`Option`, non-`Vec` field (`root`, `code`, `classCode`, `moodCode`, `typeCode`, `name`) is required on deserialize | [§2](02-wire-shapes/index.md), [§5](05-error-handling/index.md) |
| S6 | `NullFlavor` serializes as its code, a bare string, and deserializes any string — an unknown code is `Unrecognized`, never an error | [§2](02-wire-shapes/index.md) |
| S7 | `Element` serializes as `{"name", "attributes", "text", "children"}`, attributes as an object in name order, children as an array of elements | [§2](02-wire-shapes/index.md) |
| S8 | Deserializing an object ignores unknown fields rather than rejecting them | [§5](05-error-handling/index.md) |
| S9 | A missing required field is a `missing_field` error naming that field | [§5](05-error-handling/index.md) |
| S10 | The wire shape in [§2](02-wire-shapes/index.md) is part of this crate's public API and its SemVer contract | [§8](08-versioning-and-compatibility/index.md) |
| S11 | Every wrapper type implements `Deref`, `DerefMut` (except `NullFlavor`), and `From` both ways | [§6](06-ergonomics/index.md) |
| S12 | Every public item carries a doc comment; `cargo rustdoc --lib -- -W missing-docs` stays clean | [§7](07-testing-strategy/index.md) |
| S13 | `Strict<T>`, for every object-shaped `T`, rejects an unknown field at any depth instead of ignoring it; `T::deserialize` alone stays tolerant (S8) | [§11](11-strict-mode/index.md) |
| S14 | Every type round-trips to an equal value through any Serde format; no XML round trip is promised | [§4](04-round-trip-guarantee/index.md) |
| S15 | All fourteen object wrappers are written by one `macro_rules!` in `src/object.rs`, so their shape rules and strictness cannot drift between types | [§2](02-wire-shapes/index.md), [§11](11-strict-mode/index.md) |

## Which goal wins when two conflict

In order:

1. **Correctness against the `hl7-3` spec.** If a wire shape would make a
   round trip lose something `hl7-3` itself keeps — an attribute on a raw
   element, the difference between an absent `Option` and an empty
   string, an unrecognized `nullFlavor` code — that shape is wrong.
2. **Format-agnosticism.** A choice that only reads well in one format is
   wrong even if it never causes a round-trip failure.
3. **HL7® v3's own names.** Where a field corresponds to an XML attribute or
   element, its key is that attribute's or element's name (`classCode`,
   `codeSystem`), so a reader can cross-reference the JSON against the
   XML and the standard without a translation table.
4. **Readability of the wire shape.** Prefer the shape a human would choose
   reading the JSON cold.
5. **Everything else** — code brevity, symmetry between types, and so on.

## Required checks

Before finishing any change, from the workspace root:

```sh
cargo test -p serde-hl7-v3                                   # unit, integration, and doc tests
cargo clippy -p serde-hl7-v3 --all-targets -- -D warnings    # lint-clean, pedantic group on
cargo fmt --check                                            # formatting
cargo rustdoc -p serde-hl7-v3 --lib -- -W missing-docs       # every public item documented
```

All four are clean on `main` and must stay that way.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
