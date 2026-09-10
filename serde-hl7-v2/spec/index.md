# serde-hl7-v2 specification

This is the normative specification for `serde-hl7-v2` — the source of
truth for behaviour. Where the rustdoc, README, or any other document
disagrees with this one, this one is right; where this document and a test
disagree, that is a bug in whichever one is wrong, and the fix updates both
together.

This crate is a companion to [`hl7-2`](../../hl7-2/spec/index.md) and its
own spec (hereafter "the `hl7-2` spec"), and a sibling of
[`serde-er7`](https://github.com/er7-rust/er7-rust/tree/main/serde-er7),
whose specification it follows section for section. This document does not
restate anything the `hl7-2` spec already settles — the dictionary, the
three modes, validation, the meaning of a version — it only specifies the
one thing this crate adds: Serde support for the public types `hl7-2`
already defines.

## Sections

| § | Title | Covers |
|---|-------|--------|
| [1](01-purpose-and-scope/index.md) | Purpose and scope | What this crate is, why it is a separate crate, what it does not do |
| [2](02-wire-shapes/index.md) | Wire shapes | The exact `Serialize`/`Deserialize` shape for every wrapped type |
| [3](03-dependencies-and-format-agnosticism/index.md) | Dependencies and format-agnosticism | Why exactly two dependencies, and why no format is named in the library code |
| [4](04-round-trip-guarantee/index.md) | The round-trip guarantee | What survives a Serde round trip, and what deliberately does not |
| [5](05-error-handling/index.md) | Error handling | How malformed input is reported, and by what mechanism |
| [6](06-ergonomics/index.md) | Ergonomics: Deref and From | The non-normative conveniences layered over the wrapper types |
| [7](07-testing-strategy/index.md) | Testing strategy | Unit, doc, and integration tests, and what each layer is responsible for catching |
| [8](08-versioning-and-compatibility/index.md) | Versioning and compatibility | SemVer commitments, the wire-shape table as a compatibility surface, the N-2 Rust MSRV |
| [9](09-roadmap-and-open-questions/index.md) | Roadmap and open questions | What is deliberately deferred, and why |
| [10](10-glossary/index.md) | Glossary | Terms this document uses that are specific to this crate |
| [11](11-strict-mode/index.md) | Strict mode | The opt-in `Strict<T>` wrapper that rejects unknown fields |

## Rule index

Every normative rule below carries an ID (`S1`, `S2`, ...) so it can be
cited from code, tests, and commit messages without restating it. The
numbering deliberately lines up with `serde-er7`'s where the rule is the
same one (S1, S2, S7–S13), so the two crates can be discussed together.

| ID | One-line statement | Section |
|----|---------------------|---------|
| S1 | Exactly two runtime dependencies: `serde` and `hl7-2` | [§3](03-dependencies-and-format-agnosticism/index.md) |
| S2 | No format-specific crate is a runtime dependency | [§3](03-dependencies-and-format-agnosticism/index.md) |
| S3 | `Message` serializes as `{"version", "er7"}` — the release it was read as, and its ER7 text — and deserializes by parsing that text again through `hl7-2` with the version pinned | [§2](02-wire-shapes/index.md), [§4](04-round-trip-guarantee/index.md) |
| S4 | `Node` serializes as an object with exactly six keys, always all present: `name`, `path`, `kind`, `text`, `null`, `children` | [§2](02-wire-shapes/index.md) |
| S5 | `Diagnostic` serializes as an object with four keys, `severity`, `kind`, `path`, `detail`, all required on deserialize | [§2](02-wire-shapes/index.md) |
| S6 | `Version` serializes as the release string MSH-12 spells; a release `hl7-2` does not know is a deserialize error | [§2](02-wire-shapes/index.md) |
| S7 | A C-like enum (`NodeKind`, `Severity`, `DiagnosticKind`) serializes as its variant name, as a string, via `serialize_str` | [§2](02-wire-shapes/index.md) |
| S8 | Deserializing an object ignores unknown fields rather than rejecting them | [§5](05-error-handling/index.md) |
| S9 | A missing required field is a `missing_field` error naming that field | [§5](05-error-handling/index.md) |
| S10 | The wire shape in [§2](02-wire-shapes/index.md) is part of this crate's public API and its SemVer contract | [§8](08-versioning-and-compatibility/index.md) |
| S11 | Every wrapper type implements `Deref` (and `DerefMut` where the wrapped value is mutable) and `From` both ways | [§6](06-ergonomics/index.md) |
| S12 | Every public item carries a doc comment; `cargo rustdoc --lib -- -W missing-docs` stays clean | [§7](07-testing-strategy/index.md) |
| S13 | `Strict<T>` (`T` in `Message`, `Diagnostic`) rejects an unknown field instead of ignoring it; `T::deserialize` alone stays tolerant (S8) | [§11](11-strict-mode/index.md) |
| S14 | `Node` is `Serialize` only; the way back to a tree is through `Message` | [§2](02-wire-shapes/index.md), [§4](04-round-trip-guarantee/index.md) |
| S15 | `Message::seed(&options)` deserializes under the caller's `hl7_2::Options`; an `Options::version` set there wins over the wire's `"version"`, and `"version"` is otherwise optional, falling back to MSH-12 | [§2](02-wire-shapes/index.md), [§4](04-round-trip-guarantee/index.md) |

## Which goal wins when two conflict

In order:

1. **Correctness against the `hl7-2` spec.** If a wire shape would make a
   round trip lose something `hl7-2` itself preserves — the ER7 bytes, the
   release a message was read as, the absent/empty/null distinction in a
   tree — that shape is wrong, no matter how much more convenient the
   alternative reads in JSON.
2. **Format-agnosticism.** A choice that only reads well in one format is
   wrong even if it never causes a round-trip failure.
3. **One shape per fact.** The ER7 tree already has a Serde shape, in
   `serde-er7`; this crate does not define a second one for the same bytes
   ([§2.3](02-wire-shapes/index.md)).
4. **Readability of the wire shape.** Prefer the shape a human would choose
   reading the JSON cold.
5. **Everything else** — code brevity, symmetry between types, and so on.

## Required checks

Before finishing any change, from the workspace root:

```sh
cargo test -p serde-hl7-v2                                   # unit, integration, and doc tests
cargo clippy -p serde-hl7-v2 --all-targets -- -D warnings    # lint-clean, pedantic group on
cargo fmt --check                                            # formatting
cargo rustdoc -p serde-hl7-v2 --lib -- -W missing-docs       # every public item documented
```

All four are clean on `main` and must stay that way.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
