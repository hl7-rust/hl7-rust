# Plan — `hl7` Rust workspace

Goal: a production-grade, spec-driven Rust workspace for the HL7® v2 and v3
standards — 14 published crates covering messages, derives, MLLP, SOAP, and
the ER7/XML/JSON conversions — professionalized for its real audience:
healthcare professionals and the engineers who serve them, worldwide, in
settings where a wrong claim has clinical cost.

Method: **specification-driven development.** Behavior is written down in the
per-crate `spec/` directories before it is implemented, and repository-level
policy lives in the workspace `spec/` (18 policy documents, including
[`spec/hl7-trademarks-fair-use/`](spec/hl7-trademarks-fair-use/index.md) and
[`spec/phi/`](spec/phi/index.md)). Nothing in this file is a claim of what
works — that is what the crates' specs and tests are for. Day-to-day execution
items live in [`tasks.md`](tasks.md), where a `[x]` means verified, not
intended.

## Where the workspace stands (verified 2026-09-01)

All 14 crates are published to crates.io (latest release recorded in
`CHANGELOG.md`, 2026-08-29 — the fifth release, an MSRV bump). 363 `#[test]` functions, fuzz
targets in 4 of 14 crates (`hl7-2`'s own dictionary reader joined the other
three 2026-09-01), Criterion benches in 6. The root document set is nearly
complete: GOVERNANCE, SECURITY, LICENSE, CONTRIBUTING, MAINTAINERS,
AI_STATEMENT, RFC, CODEOWNERS, CITATION.cff, NEWS, COMPARISONS, BENCHMARKS,
INSTALL, CHANGELOG, and `.github/FUNDING.yml` all exist and are substantive.
Two Agent Skills (`hl7-skill/`, `hl7-rust-maintainer-skill/`) and root
`llms.txt`/`llms.json` were added 2026-08-30, per `spec/agent-skills/`
and `spec/llms-json-and-llms-txt/`.
The honest part of that document set is that it *names its own gaps* —
originally unsigned commits, no tags, no SBOM, no committed response window
— rather than implying they are covered. This plan exists to close those
gaps deliberately instead of leaving them as standing confessions; three of
the four have since closed (see workstream 3 below), and this file is
re-verified against current state each time it's revisited rather than
trusted from its last edit date.

## Workstreams — professionalization (2026-08 onward)

Six workstreams, shared with the sibling repositories (`er7-rust`,
`fhir-rust`, `snomed-rust`, `openehr-rust`) so the family converges on one
posture. Open items for each are in `tasks.md`.

1. **Governance.** GOVERNANCE.md, MAINTAINERS.md, and RFC.md exist and are
   candid about the single-maintainer model. Conduct closed 2026-08-26:
   root `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1, adapted from
   `fhir-rust`, plus a claim-accuracy clause this project takes as
   seriously as harassment), with `CONTRIBUTING.md` §Conduct pointing to
   it and carrying the reporting path. This paragraph said the opposite —
   "there is no `CODE_OF_CONDUCT.md`" — ever since, stale from before this
   file's own three comprehensive sweeps (all 2026-08-30, four days after
   the file it was contradicting existed); none of them caught it because
   none re-read this workstream's own prose against the file it was
   talking about, only against `tasks.md`'s log, and this file's own
   summary was the thing that had drifted.

2. **Compliance — licensing and trademarks.** The fair-use work is done for
   the word marks (® on first use per page, the disclaimer footer, per
   `spec/hl7-trademarks-fair-use/`), but one thing remains: the org name,
   crate names, and domain use the HL7 mark beyond fair use, and the written
   permission request (sent 2026-08-25) has no answer yet. Both provable
   compliance artifacts a legal review asks for are now present:
   `LICENSES/` with all five full license texts (2026-08-26), and
   `spec/schema-data-provenance/index.md` for the bundled
   `hl7-2/schemas/v2.*.json` table data (2026-08-27), which traces the
   tables through the `git subtree`-preserved history of the two former
   standalone repositories to their founding commit and states plainly that
   the commit cites no source. The trademark rules are checked by
   `bin/check-trademarks` in CI as of 2026-08-26.

3. **Security and supply chain.** SECURITY.md is substantive and honest. What
   it disclosed was the work list; four entries are now closed:
   `#![forbid(unsafe_code)]` is in all 20 crate roots; CI exists as of
   2026-08-26 — `.github/workflows/ci.yml` runs the CONTRIBUTING.md gates
   (fmt, clippy, test, rustdoc, MSRV) on every push and pull request, plus
   a `site` job (added 2026-08-29) that actually builds
   `hl7-rust.github.io`, closing a gap where Dependabot's green checkmark
   on a site PR used to prove nothing; `.github/workflows/security.yml`
   runs `cargo deny` (advisories, licenses, bans, sources per `deny.toml`)
   on push plus a weekly cron; git tags exist for every crate release (70
   tags, `<crate>-v<version>`, backfilled 2026-08-27 and cut going
   forward); and commits/tags are SSH-signed as of 2026-08-27, verified
   `verification.verified: true` on GitHub, GitLab, and Codeberg alike;
   and a CycloneDX SBOM (one document per crate) generates in CI on every
   push and pull request as of 2026-09-01, though only as a workflow
   artifact — no release carries one. CI was the keystone — every other
   check (trademarks, doc links, dependency audit, the SBOM, the website
   itself) now has a place to run.

4. **Privacy and patient data.** `spec/phi/index.md` is a real PHI analysis
   (9 sections, grep-checkable claims, an honest "what is not defended
   against" list), and it is now fronted by a root `PHI.md` (2026-08-26) in
   the family's Q&A-table shape, for the hospital-privacy-officer reader
   who won't go looking under `spec/`.

5. **Outreach.** `help/outreach/index.md` (2026-08-25) is the campaign plan,
   and it gates itself correctly: issue templates and a stated response
   expectation landed 2026-08-26, and the trademark request was sent
   2026-08-25 — but the gate is on HL7's answer, not on the sending, so
   trademark resolution remains the unmet prerequisite: no promotion until
   the request is answered. NEWS.md's press posture is ready.

6. **Audit and harmonization.** This file and `tasks.md` are the ongoing
   findings register and plan/tasks history. Three comprehensive sweeps ran
   2026-08-30, all prompted by the same plain "update, upgrade,
   harmonize, annotate, audit, fix" instruction repeated — the second,
   after the two Agent Skills and `llms.txt`/`llms.json` were added,
   caught an unsigned-commits claim in `SECURITY.md`/`CODEOWNERS` left
   stale since signing landed 2026-08-27, `hl7-3/AGENTS.md` never updated
   for its 2026-08-19 struct-mode feature, and the benchmark
   re-measurement's own `er7` version transcribed wrong. The third caught
   something the first two didn't check for at all: local `main` was 11
   commits ahead of every remote the whole time, so the second sweep's own
   fixes never reached GitHub, GitLab, or Codeberg — plus a scatter of
   single-line drift (a false filesystem-access claim repeated in two
   files, stale benchmark figures in `NEWS.md` alone, the website's
   `/spec/` page undercounting again after two new specs landed, malformed
   YAML frontmatter in `hl7-skill/SKILL.md`, and `llms.txt`/`llms.json`
   inheriting a consumer-count bug already fixed elsewhere before those
   two files existed) — `tasks-archive.md` has the full account of all
   three, moved there 2026-09-01 when `tasks.md` outgrew its 40 KB budget.
   The family conventions to converge on: the canonical
   special-files list (the local
   `spec/special-files-for-public-repos/index.md` was re-synced from the
   `fhir-rust` copy 2026-08-26, typos fixed), MSRV N−2 (raised from N−3
   2026-08-29, per `spec/rust-msrv-n-minus-2/index.md`), `AGENTS.md`-style
   playbooks, and automated doc gates.

## Open decisions (awaiting a call, not code)

- **HL7 written permission.** The request was sent 2026-08-25; the reply is
  pending. If HL7 declines, the org/crate/domain naming question reopens —
  which is why outreach stays gated on the answer, not on the sending.
- ~~**Schema data provenance.**~~ Written 2026-08-27:
  `spec/schema-data-provenance/index.md` traces the tables through the
  `git subtree`-preserved history of the former standalone repositories to
  their founding commit, states that no HL7® file is vendored anywhere in
  this workspace, and states plainly that the founding commit itself cites
  no source. Whether that is *sufficient* for a given legal review is now
  RFC.md §12's question, not an undocumented gap.
- ~~**CI hosting shape.**~~ Decided 2026-08-26: one workflow at the root
  covering the whole workspace (`.github/workflows/ci.yml`, the `fhir-rust`
  pattern). Nothing here needs engine containers.

## Non-goals (for now)

- No interface-engine ambitions (routing, transformation pipelines, UIs).
- No new crates until the professionalization workstreams are closed; the
  workspace grows trust before it grows surface.

## Risks & watch items

- The document set's self-declared gaps are now promises. "No CI" fell
  2026-08-26, "no dependency audit" fell the same day (`cargo deny` in
  `security.yml`), "zero git tags" fell 2026-08-27 (70 tags, backfilled and
  ongoing), "unsigned commits" fell 2026-08-27 too (SSH signing, verified
  on all three forges), and "no SBOM" fell 2026-09-01 (a `sbom` job in
  `ci.yml` generates one CycloneDX document per crate on every push and
  pull request). That last one closes only the generation half, not the
  distribution half: nothing attaches these documents to a crates.io
  release, since publishing itself is still a manual, untagged act — a
  narrower claim than the sibling `fhir-rust`'s SBOM-per-release, stated
  that way rather than implied to be the same thing.
- BENCHMARKS.md's watch item made good on itself, once: it was re-measured
  2026-08-26 against `hl7-2` 0.2.6, but `hl7-2` released twice more since
  (0.2.7, then 0.3.0 on 2026-08-29) without a re-measurement, and the
  figures went stale — the exact failure mode the watch item predicted.
  Caught and re-measured 2026-08-30 against `hl7-2` 0.3.0 / `er7` 0.2.1
  (`tasks-archive.md` has the full account, moved there 2026-09-01). The
  watch item itself doesn't close:
  a claims-accuracy culture has to keep re-checking this every time a
  release moves past the measured version, not just once.
- ~~`CODEOWNERS` asserts the repo has no `.github/` directory; it has one.~~
  Fixed 2026-08-26. The general watch item stands: small falsehoods in
  governance files are the failure mode this family exists to avoid.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
