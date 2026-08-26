# Special files for public repos

Special files that use top-level markdown:

- README.md
- LICENSE.md with SPDX license information
- CITATION.cff with ORCID citation for Joel Parker Henderson (joel@joelparkerhenderson.com) (see ~/git/assertables/assertiables/CITATION.md for template)
- NEWS.md with news, update information, press contacts, etc.
- COMPARISONS.md comparisons to relevant projects, context, etc.
- BENCHMARKS.md with any benchmarks, speed tests, optimization profiles, etc.
- INSTALL.md how to install and use any of the software
- CONTRIBUTING.md how a person can contribute their time, or update code, or donate money
- CODEOWNERS with joel@joelparkerhenderson.com
- MAINTAINERS.md with Joel Parker Henderson (joel@joelparkerhenderson.com) as sole maintainer (use this as template: https://github.com/rubentalstra/FerroEHR/blob/develop/MAINTAINERS.md)
- CHANGELOG.md with change log history summaries
- AI_STATEMENT.md (use this as template: https://github.com/rubentalstra/FerroEHR/blob/develop/AI_STATEMENT.md)
- GOVERNANCE.md how decisions are made, what binds them, how to disagree, how to become a maintainer
- SECURITY.md how to report a vulnerability, what is in scope, response windows, known open issues
- CODE_OF_CONDUCT.md Contributor Covenant 2.1, plus this project's claim-accuracy clause
- PHI.md what the software does and does not do with patient data, in plain language
- RFC.md the open questions this project wants answered, and what feedback helps
- TRADEMARKS.md the consolidated trademark position: verbatim notice, mark-by-mark table, what is and is not claimed
- LICENSES/ the full text of every licence the SPDX expression offers (REUSE convention)
- .github/FUNDING.yml the donation routes CONTRIBUTING.md points at

## Status in this repository

All of the above exist as of 2026-08-26. Two notes:

- **This list is synchronized from the `fhir-rust` sibling's canonical
  copy** (its `spec/special-files-for-public-repos/index.md`), which is the
  most complete in the family; this revision also fixes the typos the
  canonical copy still carries ("optimizaiton", "Prker", "summries") and
  adds `TRADEMARKS.md`, which this repository carries per the `er7-rust`
  model.
- **The HL7® trademark rules in
  [`spec/hl7-trademarks-fair-use/`](../hl7-trademarks-fair-use/index.md) are
  met by all of these files as of 2026-08-26.** They require `®` after the
  first use of each word mark on each page, plus the endorsement disclaimer
  wherever the marks appear; [`bin/check-trademarks`](../../bin/check-trademarks)
  verifies every markdown page in the workspace and runs in CI
  (`.github/workflows/ci.yml`). The issue templates under
  `.github/ISSUE_TEMPLATE/` deliberately avoid the marks entirely: their
  bodies inject into every filed issue, where a disclaimer footer does not
  belong.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
