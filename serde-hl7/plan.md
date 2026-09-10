# Plan — `serde-hl7`, `serde-hl7-v2`, `serde-hl7-v3`

Three crates, decided and built 2026-09-10 at the maintainer's direction,
after the evaluation recorded at the bottom of this file. Execution items
and the release checklist live in [`tasks.md`](tasks.md).

## The decision

The question asked on 2026-09-10 was whether code should be *extracted*
from this workspace into a crate named `serde-hl7`, to make it clearer
that the code does serialization and deserialization. The evaluation
(below) found nothing to extract: no crate here used Serde. The maintainer
then changed the brief: **build a real `serde-hl7`, the way `serde-er7` was
built**, as an umbrella over two sub-crates, `serde-hl7-v2` and
`serde-hl7-v3`, with `v2`/`v3` features — and publish.

## What was built

| Crate | Wraps | Module | Runtime dependencies |
|---|---|---|---|
| [`serde-hl7-v2`](../serde-hl7-v2) | `hl7-2`: `Message`, `Node`, `Diagnostic`, `Version`, the three enums | `serde_hl7::v2` | `serde`, `hl7-2` |
| [`serde-hl7-v3`](../serde-hl7-v3) | `hl7-3`: `Message`, `ControlAct`, `Element`, `Ii`, `Cd`, `Ivl`, `Pq`, `Ed`, `NullFlavor`, six RIM classes | `serde_hl7::v3` | `serde`, `hl7-3` |
| [`serde-hl7`](.) | the two above, behind `v2` and `v3` features, both default | — | the two above, optional |

Each sub-crate follows `serde-er7` section for section: a `spec/` with an
S-numbered rule index and a §7.1 coverage table that `cargo test` checks
against it; hand-written impls against Serde's low-level traits, no
derive; two runtime dependencies, `serde_json` in dev only; a same-named
wrapper per type with `Deref`/`DerefMut`/`From`; `Strict<T>` for fixture
typos; runnable examples; README, AGENTS.md, CLAUDE.md, LICENSE.md.

## Design decisions, and why

- **Names.** `serde-hl7-v2`/`serde-hl7-v3` were the maintainer's choice.
  They break the workspace's `hl7-2`/`hl7-3` convention in favour of
  reading as `serde_hl7::v2`/`::v3`, which is the module path callers
  see. All four candidate names (`serde-hl7`, `serde_hl7`, `serde-hl7-v2`,
  `serde-hl7-v3`) were unclaimed on crates.io when checked.
- **v2's `Message` is ER7 text plus release, not a tree.** The tree shape
  belongs to `serde-er7` over `er7`'s types; a second copy would be two
  crates specifying one wire shape. `Node` is where v2's tree-shaped
  effort goes, because only `hl7-2` has the dictionary that names it.
  `serde-hl7-v2/spec/02-wire-shapes/index.md` §2.3.
- **v2's `Node` is `Serialize` only.** `hl7_2::Node` has no public
  constructor, and adding one is `hl7-2`'s decision. Rule S14, roadmap
  §9.1.
- **v2's `MessageSeed`.** Schema mode (a caller's own dictionary) cannot
  reach a stateless `Deserialize`; a `DeserializeSeed` carrying
  `hl7_2::Options` can. Rule S15.
- **v3 keys are lowerCamelCase.** Wherever a field corresponds to an HL7®
  v3 XML attribute or element, that is the XML name, so JSON and XML
  cross-reference without a table. Rule S3.
- **v3 uses one `macro_rules!` for its fourteen object types.** Fourteen
  hand-written visitors would drift; a local macro keeps the two-dependency
  rule and makes nested strictness uniform. Not a derive: no proc-macro
  dependency. Rule S15.
- **v3 promises a value round trip, not an XML one.** `hl7-3` has no XML
  writer and normalizes on the way in. Rule S14.
- **Two dependencies each, not three.** `serde-er7` was not added to
  `serde-hl7-v2` because the ER7 text shape needs no tree.

## Workspace consequences, handled in the same change

- `Cargo.toml` members: 14 → 17. CI's per-crate rustdoc loop likewise.
- `spec/phi/index.md`'s "no serialization framework" row now names the
  three opt-in bridge crates as the exception, and the fourteen existing
  crates' claim is unchanged.
- Root `plan.md`'s "no new crates until professionalization closes"
  non-goal was overridden by the maintainer for these three; the non-goal
  is reworded rather than deleted.
- `spec/release-process/index.md`'s "first release of a brand-new crate
  stays the maintainer's call" held: this first release was directed by
  the maintainer, in a live session, explicitly.
- Every root document that counted fourteen crates counts seventeen; the
  website's crate catalog, navigation, and `llms.txt`/`llms.json` gained
  three entries.

## Non-goals

- Extracting any existing module under a Serde name. Struct mode
  (`FromHl7`/`ToHl7`, `FromElement`) is not Serde and stays where it is.
- Adding `serde` to any of the fourteen existing crates.
- An XML writer, a `to_json_string`, Serde for `Dictionary`.

---

## Appendix: the evaluation (2026-09-10, before the decision)

Verified before any code was written:

- No Serde anywhere in the workspace: no member depended on `serde` or
  `serde_json`; `hl7-2/AGENTS.md` and `hl7-2-from-er7-into-json/AGENTS.md`
  listed pulling them in under *don't*; `spec/phi/index.md` recorded "no
  serialization framework" as a workspace property.
- `serde-er7` (`~/git/er7-rust/er7-rust/serde-er7`) was written new, not
  extracted: ~1,900 lines of `src/`, 50 tests, 11 spec sections, two
  runtime dependencies.
- The code a reader might call "serialization" — `hl7-2/src/typed.rs`
  struct mode and `hl7-2-derive`, `hl7-3/src/typed.rs` and `hl7-3-derive`,
  `hl7-2/src/json.rs`, the four `*-from-*-into-*` converters — is not
  Serde, and moving any of it under a Serde name would mislead.
- The recommendation was therefore: (now) say "serialize"/"deserialize"
  in `hl7-2`'s docs where struct mode already does it under other names,
  and (later, gated) build a real bridge crate. The maintainer chose to
  build now.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7. This project is an independent work.
