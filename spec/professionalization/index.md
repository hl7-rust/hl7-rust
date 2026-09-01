[hl7-rust](../../README.md) → spec → Professionalization

# Professionalization

This specification defines what "professional" means for this repository and
binds the maintainer as much as any contributor. The audience is healthcare
professionals and the engineers who serve them, worldwide, in production use;
the standing constraint is that a wrong claim in this domain has clinical
cost. Rationale and current execution state live in [`plan.md`](../../plan.md)
and [`tasks.md`](../../tasks.md); this file holds the rules.

## Rules

1. **Plans are files, and a checked box is a verified fact.** `plan.md` and
   `tasks.md` exist at the repository root. A `[x]` means the work was done
   and verified, with the evidence named — never that it is intended,
   assumed, or inherited from a sibling repository.
2. **The special files exist and stay accurate.** The canonical list is
   [`spec/special-files-for-public-repos/`](../special-files-for-public-repos/index.md).
   Every countable claim in those files (crate counts, test counts, coverage
   lists, "X is enabled/disabled") is measured before it is written and
   re-verified when cited.
3. **Self-declared gaps are promises.** A gap named in SECURITY.md,
   MAINTAINERS.md, or AI_STATEMENT.md ("no CI", "unsigned commits") is either
   closed or consciously accepted in `tasks.md` — and the declaring document
   is updated in the same change that closes it.
4. **CI enforces what documents claim.** Every check a document says this
   repository runs (tests, clippy, fmt, MSRV, trademark rules, doc gates)
   runs in CI on every push. A laptop-only check is a claim, not a guarantee.
5. **Trademark discipline.** The word marks used here — HL7® and FHIR® —
   belong to Health Level Seven International, and
   [`spec/hl7-trademarks-fair-use/`](../hl7-trademarks-fair-use/index.md)
   quotes the owner's rules verbatim: the ® registration mark follows the
   first use of each mark on every page, every page using a mark carries the
   prescribed disclaimer, and the Fast Healthcare Interoperability Resources
   are referred to as the "HL7® FHIR® standard". An automated checker
   enforces the rules, per rule 4, rather than leaving them to authorial
   memory. Where this project's naming goes *beyond* fair use — package
   names, the organization name, the domain — the written-permission
   question is tracked in [`plan.md`](../../plan.md) §Open decisions and
   [`RFC.md`](../../RFC.md) §11.
6. **Patient data is addressed in plain language.** `PHI.md` at the root
   states what the software does and does not do with patient data, for a
   reader who is a privacy officer, not a Rust programmer. It never claims
   compliance or certification.
7. **Conduct has a document and a path.** `CODE_OF_CONDUCT.md` at the root
   (Contributor Covenant 2.1 plus this family's claim-accuracy clause:
   overstating what the software does is a conduct matter, not only a bug).
8. **Harmonization runs through the family.** The sibling repositories
   (`hl7-rust`, `er7-rust`, `fhir-rust`, `snomed-rust`, `openehr-rust`)
   share these rules, the special-files list, and the six workstreams
   (governance; compliance — licensing and trademarks; security and supply
   chain; privacy and patient data; outreach; audit and harmonization).
   Conventions sync from the repository that owns the canonical copy rather
   than drifting independently.
9. **Outreach is gated.** No promotion while a rule above is unmet for the
   surface being promoted; `help/outreach/index.md` names the prerequisites.

## Status in this repository

Assessed 2026-08-26, rule by rule, with evidence paths. Open items cite the
[`tasks.md`](../../tasks.md) entry that tracks them.

1. **Met.** `plan.md` and `tasks.md` exist at the root, committed
   2026-08-26, and every `[x]` added since names its evidence.
2. **Met.** The special files exist and are substantive (the list is in
   `tasks.md` § Done). The local canonical list,
   `spec/special-files-for-public-repos/index.md`, was re-synced from the
   family version 2026-08-26 (tracked by the "Re-sync" item under Audit and
   harmonization) and that file's own Status section now states it is
   synchronized.
3. **Met as a practice, with gaps still open.** The gaps closed so far
   (`#![forbid(unsafe_code)]`, CI, dependency auditing via `cargo deny`,
   commit/tag signing 2026-08-27, and SBOM generation 2026-09-01) each
   updated the declaring documents in the closing change; the
   still-standing declared gap (no committed security-response window,
   a deliberate stance per `MAINTAINERS.md`, not an open task) is not
   tracked as an actionable item in `tasks.md` for that reason.
4. **Met.** `.github/workflows/ci.yml` (2026-08-26) runs the
   CONTRIBUTING.md gates — fmt, clippy, test, rustdoc, the 1.96 MSRV floor —
   and the rule-5 trademark check on every push and pull request. Its
   `sbom` job (2026-09-01) generates a CycloneDX document per crate on the
   same triggers. The `docs` job runs the link check and document-size
   budget (`bin/check-docs`), and `.github/workflows/security.yml` runs the
   dependency audit (`cargo deny`, per `deny.toml`) on push and a weekly
   cron.
5. **Partly met.** The prose rules are implemented, and since later on
   2026-08-26 `bin/check-trademarks` enforces them in CI over a deliberate,
   documented scope (markdown, crate-root rustdoc, the site's shared
   footer); the not-yet-covered remainder is recorded in the trademark
   spec's Assurance section. The beyond-fair-use permission request still
   has no recorded status — tracked under Compliance, and asked as
   `RFC.md` §11.
6. **Met.** `PHI.md` at the root, 2026-08-26, fronting
   [`spec/phi/index.md`](../phi/index.md); it claims no compliance or
   certification.
7. **Met.** `CODE_OF_CONDUCT.md` at the root, 2026-08-26, with the
   claim-accuracy clause and the reporting path in CONTRIBUTING.md §Conduct.
8. **Ongoing.** This file is itself an instance: adapted from the family's
   canonical template. The special-files list (rule 2) was re-synced
   2026-08-26 and is no longer a known divergence.
9. **Met.** `help/outreach/index.md` gates promotion on its prerequisites,
   and the Outreach section of `tasks.md` records the block as standing.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
