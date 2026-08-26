# Tasks

Execution checklist; rationale and workstreams live in [`plan.md`](plan.md).
A `[x]` here means the work is **verified done**, not intended — check items
off in the same change that completes them, with the evidence named.

## Done (verified 2026-08-26, the state this file starts from)

- [x] All 14 crates published to crates.io; release recorded in
      `CHANGELOG.md` with per-crate versions.
- [x] Root document set exists and is substantive: GOVERNANCE.md, SECURITY.md,
      LICENSE.md, CONTRIBUTING.md, MAINTAINERS.md, AI_STATEMENT.md, RFC.md,
      CODEOWNERS, CITATION.cff, NEWS.md, COMPARISONS.md, BENCHMARKS.md,
      INSTALL.md, CHANGELOG.md, `.github/FUNDING.yml`.
- [x] Trademark fair-use rules implemented in prose per
      `spec/hl7-trademarks-fair-use/index.md`: ® after first use per page and
      the disclaimer footer in README.md, LICENSE.md, SECURITY.md,
      GOVERNANCE.md, CONTRIBUTING.md; site footer per CHANGELOG.
- [x] PHI analysis exists at `spec/phi/index.md` (11 sections, checkable
      claims, explicit non-defenses).
- [x] Outreach research exists at `help/outreach/index.md` (audiences,
      channels, 90-day sequence, prerequisites) and correctly gates promotion
      on trademark resolution and issue-response readiness.
- [x] BENCHMARKS.md is measured (Criterion, machine/toolchain/date recorded),
      not written.

## Next up

Grouped by `plan.md` workstream. Order within a group is priority order.

### Security and supply chain

- [x] **Stand up CI at the repository root** — done 2026-08-26:
      `.github/workflows/ci.yml`, two jobs (stable: `cargo fmt --check`,
      `cargo clippy --all-targets -- -D warnings`, `cargo test`,
      `cargo rustdoc --lib -- -W missing-docs` per crate; msrv:
      `cargo check --workspace --all-targets` on 1.95), YAML validated
      locally. SECURITY.md, MAINTAINERS.md, and AI_STATEMENT.md §7/§12
      updated in the same commit. The workflow has not yet been observed
      green on GitHub-hosted runners — that verification is pending the
      next push.
- [x] Add `#![forbid(unsafe_code)]` to all 14 crates — done, and to all
      **20** crate roots: the six `src/main.rs` binaries are separate crates
      and need their own attribute. Verified by negative test (adding
      `#[allow(unsafe_code)]` to `hl7-2` fails to compile with E0453,
      "overruled by previous forbid") and by `cargo test`, `clippy
      -D warnings`, `fmt --check`, and the 1.95 MSRV check all passing.
      SECURITY.md's claim and its "known gaps" entry updated in the same
      change.
- [ ] Tag releases and sign commits/tags going forward; record the change of
      posture in MAINTAINERS.md (which currently says nothing is signed).
- [ ] Add dependency auditing (`cargo deny` covering advisories, licenses,
      bans, sources — the `fhir-rust` `fhir-security.yml` is the family
      pattern) on push plus a weekly cron.
- [ ] Add `.github/ISSUE_TEMPLATE/` and a stated issue-response expectation —
      `help/outreach/index.md` lists both as prerequisites before any
      promotion.

### Governance

- [x] **Add `CODE_OF_CONDUCT.md`** — done 2026-08-26: Contributor Covenant
      2.1 plus the claim-accuracy clause, adapted from `fhir-rust` with the
      FHIR-specific references replaced (the clause now cites this repo's
      `spec/` culture; Scope names HL7® International's own spaces).
      CONTRIBUTING.md §Conduct now points to it and carries the private
      reporting path.
- [ ] Fix `CODEOWNERS`'s stale header claiming the repo has no `.github/`
      directory.

### Compliance — licensing and trademarks

- [ ] **Record the status of the HL7 written-permission request** (org name,
      crate names, domain — all beyond fair use per
      `spec/hl7-trademarks-fair-use/index.md`). If not yet sent, send it; the
      README's "we are requesting permission" needs a date and an outcome.
- [ ] Restore an automated trademark checker — port
      `er7-rust/bin/check-trademarks` or `fhir-rust/scripts/check-trademarks.sh`.
      (The orphaned `__pycache__/tm.cpython-311.pyc` is gone and
      `__pycache__/` is now ignored; it was a stray from an ad-hoc helper
      script, not a checker worth restoring.)
- [ ] State the provenance and terms of the bundled
      `hl7-2/schemas/v2.1–2.9.json` table data (currently traced only to
      sibling crates' copies at `hl7-2/spec/index.md:43`).
- [ ] Add `LICENSES/` with the full text of all five licenses in the SPDX
      expression (REUSE convention; `fhir-rust/LICENSES/` is the model).
- [ ] Consolidate the trademark position into a root `TRADEMARKS.md`
      (the `er7-rust` file is the model: verbatim notice, mark-by-mark table,
      what is and is not claimed).

### Privacy and patient data

- [ ] Add root `PHI.md` fronting `spec/phi/index.md` for a privacy-officer
      reader (the `fhir-rust` PHI.md Q&A-table shape is the model).

### Outreach

- [ ] Blocked until the trademark item and issue templates above are done —
      then execute `help/outreach/index.md`'s phase 1. Update its stale note
      that the trademark notice "is not yet carried on the website" (the site
      footer now carries it per CHANGELOG).
- [ ] Add `version`, `date-released`, and (once a Zenodo deposit exists) a DOI
      to `CITATION.cff`.
- [ ] Refresh BENCHMARKS.md against the released crate versions or mark its
      figures as historical (it cites `hl7-2` 0.2.3; released is 0.2.5).

### Audit and harmonization

- [ ] Re-sync `spec/special-files-for-public-repos/index.md` with the
      `fhir-rust` canonical version (it is missing CODE_OF_CONDUCT.md,
      PHI.md, LICENSES/, FUNDING.yml, and a status section, and carries two
      typos: "optimizaiton", "Prker").
- [ ] Adopt a repository-wide link check and document-size budget in CI, per
      the `snomed-rust` convention.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
