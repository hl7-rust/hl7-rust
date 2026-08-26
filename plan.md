# Plan — `hl7` Rust workspace

Goal: a production-grade, spec-driven Rust workspace for the HL7® v2 and v3
standards — 14 published crates covering messages, derives, MLLP, SOAP, and
the ER7/XML/JSON conversions — professionalized for its real audience:
healthcare professionals and the engineers who serve them, worldwide, in
settings where a wrong claim has clinical cost.

Method: **specification-driven development.** Behavior is written down in the
per-crate `spec/` directories before it is implemented, and repository-level
policy lives in the workspace `spec/` (7 policy documents, including
[`spec/hl7-trademarks-fair-use/`](spec/hl7-trademarks-fair-use/index.md) and
[`spec/phi/`](spec/phi/index.md)). Nothing in this file is a claim of what
works — that is what the crates' specs and tests are for. Day-to-day execution
items live in [`tasks.md`](tasks.md), where a `[x]` means verified, not
intended.

## Where the workspace stands (verified 2026-08-26)

All 14 crates are published to crates.io (latest release recorded in
`CHANGELOG.md`, 2026-08-26). 363 `#[test]` functions, fuzz targets in 3 of 14
crates, Criterion benches in 4. The root document set is nearly complete:
GOVERNANCE, SECURITY, LICENSE, CONTRIBUTING, MAINTAINERS, AI_STATEMENT, RFC,
CODEOWNERS, CITATION.cff, NEWS, COMPARISONS, BENCHMARKS, INSTALL, CHANGELOG,
and `.github/FUNDING.yml` all exist and are substantive. The honest part of
that document set is that it *names its own gaps* — unsigned commits, no
SBOM, no committed response window — rather than implying they are covered.
This plan exists to close those gaps deliberately instead of leaving them as
standing confessions.

## Workstreams — professionalization (2026-08 onward)

Six workstreams, shared with the sibling repositories (`er7-rust`,
`fhir-rust`, `snomed-rust`, `openehr-rust`) so the family converges on one
posture. Open items for each are in `tasks.md`.

1. **Governance.** GOVERNANCE.md, MAINTAINERS.md, and RFC.md exist and are
   candid about the single-maintainer model. The visible hole is conduct:
   `CONTRIBUTING.md` §Conduct is four lines with no reporting or enforcement
   path, and there is no `CODE_OF_CONDUCT.md`. The sibling `fhir-rust` has a
   Contributor Covenant 2.1 with a claim-accuracy clause to adapt.

2. **Compliance — licensing and trademarks.** The fair-use work is done for
   the word marks (® on first use per page, the disclaimer footer, per
   `spec/hl7-trademarks-fair-use/`), but three things remain: the org name,
   crate names, and domain use the HL7 mark beyond fair use, and the written
   permission the README says is being requested has no recorded outcome; and
   of the two provable compliance artifacts a legal review asks for, one is
   now present (`LICENSES/` with all five full license texts, 2026-08-26)
   and one is still missing (a provenance statement for the bundled
   `hl7-2/schemas/v2.*.json` table data). The trademark rules are checked by
   `bin/check-trademarks` in CI as of 2026-08-26.

3. **Security and supply chain.** SECURITY.md is substantive and honest. What
   it disclosed was the work list; two entries are now closed:
   `#![forbid(unsafe_code)]` is in all 20 crate roots, and CI exists as of
   2026-08-26 — `.github/workflows/ci.yml` runs the CONTRIBUTING.md gates
   (fmt, clippy, test, rustdoc, MSRV) on every push and pull request, and
   `.github/workflows/security.yml` runs `cargo deny` (advisories,
   licenses, bans, sources per `deny.toml`) on push plus a weekly cron.
   Still open: zero git tags, unsigned commits, no SBOM. CI was the
   keystone — every other check (trademarks, doc links, dependency audit)
   now has a place to run.

4. **Privacy and patient data.** `spec/phi/index.md` is a real PHI analysis
   (11 sections, grep-checkable claims, an honest "what is not defended
   against" list) but it is not surfaced where a hospital privacy officer will
   look. The family convention is a root `PHI.md`; promote or front the spec
   there.

5. **Outreach.** `help/outreach/index.md` (2026-08-25) is the campaign plan,
   and it gates itself correctly: issue templates and a stated response
   expectation landed 2026-08-26, so trademark resolution is the remaining
   unmet prerequisite — no promotion until it is answered or in flight.
   NEWS.md's press posture is ready.

6. **Audit and harmonization.** This repository has no findings register and
   no plan/tasks history — this file and `tasks.md` are the start. The family
   conventions to converge on: the canonical special-files list (the local
   `spec/special-files-for-public-repos/index.md` was re-synced from the
   `fhir-rust` copy 2026-08-26, typos fixed), MSRV N−3, `agents/`-style
   playbooks, and automated doc gates.

## Open decisions (awaiting a call, not code)

- **HL7 written permission.** The request's status (sent? answered?) is
  recorded nowhere. If HL7 declines, the org/crate/domain naming question
  reopens — which is why outreach stays gated on it.
- **Schema data provenance.** State where `hl7-2/schemas/v2.1–2.9.json` table
  content came from and under what terms, or replace it with content whose
  terms are stated. A hospital legal review asks this first.
- ~~**CI hosting shape.**~~ Decided 2026-08-26: one workflow at the root
  covering the whole workspace (`.github/workflows/ci.yml`, the `fhir-rust`
  pattern). Nothing here needs engine containers.

## Non-goals (for now)

- No interface-engine ambitions (routing, transformation pipelines, UIs).
- No new crates until the professionalization workstreams are closed; the
  workspace grows trust before it grows surface.

## Risks & watch items

- The document set's self-declared gaps are now promises. "No CI" fell
  2026-08-26, and "no dependency audit" fell the same day (`cargo deny` in
  `security.yml`); the remaining declared gaps (unsigned commits, no SBOM)
  are still standing confessions, and each release that ships while they
  stand makes SECURITY.md's candor look like a substitute for the fix
  rather than a commitment to it.
- BENCHMARKS.md was re-measured 2026-08-26 against the released `hl7-2`
  0.2.6, closing its version staleness — but the watch item stands: a
  claims-accuracy culture has to keep its numbers current or date-stamp
  them as historical every time a release moves past them.
- `CODEOWNERS` asserts the repo has no `.github/` directory; it has one. Small
  falsehoods in governance files are the failure mode this family exists to
  avoid.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
