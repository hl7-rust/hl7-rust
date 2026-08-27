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
- [ ] Sign commits and tags going forward. **Tagging is done, 2026-08-27:**
      the fourth release's tag convention (`<crate>-v<version>`, annotated)
      was backfilled onto the first three release commits too, so all 56
      tags (14 crates × 4 releases) exist and are pushed to all three
      remotes — `git show hl7-2-v0.2.7` and its three predecessors all
      resolve.
      **Signing is live, 2026-08-27:** `gpg.format ssh`,
      `commit.gpgsign`/`tag.gpgsign` both `true`, keyed to a dedicated
      passphrase-protected signing key (`~/.ssh/id.d/jph-code-signing=...`,
      created for exactly this rather than reusing a general-purpose
      identity key) — deliberately not the unattended automation key
      already used to push, since an unattended signature would not mean
      anything. A local `allowed_signers` file is wired via
      `gpg.ssh.allowedSignersFile` for local verification. The gate was
      exercised three times before the key was loaded — twice attempting a
      commit with no key in the agent (once against the identity key first
      proposed, again after switching to the dedicated key), and each time
      git refused rather than silently signing — and once for real: with
      the key loaded via `ssh-add --apple-use-keychain` (macOS's system
      `/usr/bin/ssh-add`, not the Homebrew one earlier on `PATH`, which
      lacks the flag), both a test commit and a test annotated tag produced
      a verified local signature (`Good "git" signature for
      joel@joelparkerhenderson.com`), then were discarded. Registering the
      public key as a **Signing Key** on GitHub, GitLab, and Codeberg
      (separate from the Authentication Key already present) is still the
      maintainer's step for the hosts to show a verified badge; local
      signing and verification no longer wait on it. MAINTAINERS.md's
      tagging/signing paragraph rewritten to describe this exact state.
- [x] Add dependency auditing (`cargo deny` covering advisories, licenses,
      bans, sources — the `fhir-rust` `fhir-security.yml` is the family
      pattern) on push plus a weekly cron — done 2026-08-26: `deny.toml`
      at the root (permissive-only dependency allowlist, empty advisory
      ignore list, wildcards denied with path-only dev-deps allowed) and
      `.github/workflows/security.yml`; `cargo deny --all-features check`
      green locally: "advisories ok, bans ok, licenses ok, sources ok".
      SECURITY.md's known-gaps entry closed in the same change.
- [x] Add `.github/ISSUE_TEMPLATE/` and a stated issue-response expectation —
      done 2026-08-26: bug report (never paste patient data), wrong claim
      (the report this repo values most), and a `config.yml` routing
      security reports privately and pointing at the expectation;
      MAINTAINERS.md now states it (read within a week, a target not a
      contract). `help/outreach/index.md`'s prerequisites updated — the
      trademark question is now its only unmet gate.

### Governance

- [x] **Add `CODE_OF_CONDUCT.md`** — done 2026-08-26: Contributor Covenant
      2.1 plus the claim-accuracy clause, adapted from `fhir-rust` with the
      sibling-specific references replaced (the clause now cites this repo's
      `spec/` culture; Scope names HL7® International's own spaces).
      CONTRIBUTING.md §Conduct now points to it and carries the private
      reporting path.
- [x] Fix `CODEOWNERS`'s stale header claiming the repo has no `.github/`
      directory — done 2026-08-26; the same pass reworded "commit
      signatures" to "committer identity", since MAINTAINERS.md says
      plainly that nothing is cryptographically signed.

### Compliance — licensing and trademarks

- [x] **Record the status of the HL7 written-permission request** (org name,
      crate names, domain — all beyond fair use per
      `spec/hl7-trademarks-fair-use/index.md`) — sent half done: the request
      was sent 2026-08-25 and that date is now recorded (README,
      `TRADEMARKS.md`, `plan.md`, the trademark spec, the outreach gate).
      The outcome is the remaining half, tracked by the next item.
- [ ] **Record the outcome of the HL7 written-permission request** when the
      reply arrives (pending as of 2026-08-26). Outreach stays gated on the
      answer, not on the sending, per `help/outreach/index.md`; if HL7
      declines, the org/crate/domain naming question reopens.
- [x] Restore an automated trademark checker — done 2026-08-26:
      `er7-rust/bin/check-trademarks` ported as `bin/check-trademarks`,
      running in CI (`.github/workflows/ci.yml`, trademarks job) and
      exiting 0 across the workspace after 28 in-scope fixes. Scope is
      deliberate and documented in the script header and the trademark
      spec's Assurance section: markdown, crate-root rustdoc, and the site
      footer are covered; per-route site source, non-root rustdoc,
      Cargo.toml descriptions, and --help output are recorded there as the
      not-yet-covered remainder (114 findings from the full-scope run).
      (The orphaned `__pycache__/tm.cpython-311.pyc` is gone and
      `__pycache__/` is now ignored; it was a stray from an ad-hoc helper
      script, not a checker worth restoring.)
- [x] State the provenance and terms of the bundled
      `hl7-2/schemas/v2.1–2.9.json` table data — done 2026-08-27:
      `spec/schema-data-provenance/index.md`. Traced through the `git
      subtree`-preserved history of the two former standalone repositories
      (one hop further than "sibling crates' copies," which is where
      `hl7-2/spec/index.md:43` had left it) to the founding commit
      `6afe87f`, which cites no source for the table content — stated as a
      real gap rather than filled with a guess. States the idea/expression
      legal reasoning, the terms (project license, no HL7® file vendored
      anywhere in the workspace), and what would close the sourcing gap.
      Linked from `hl7-2/spec/index.md` §0, the website `/spec/` page,
      RFC.md §12, and `plan.md`'s open-decisions list, all updated in the
      same change. The website `/spec/` page's own count of workspace specs
      was stale before this ("Four specs" naming only four of what were
      already ten `spec/` subdirectories); corrected to list five with
      pages and name the other five as not yet paged — a smaller, adjacent
      but real gap, not fully closed here.
- [x] Add `LICENSES/` with the full text of all five licenses in the SPDX
      expression (REUSE convention; `fhir-rust/LICENSES/` is the model) —
      done 2026-08-26: five files copied from the `fhir-rust` model,
      `LICENSE.md`'s table now links each local text alongside the URL.
- [x] Consolidate the trademark position into a root `TRADEMARKS.md`
      (the `er7-rust` file is the model: verbatim notice, mark-by-mark table,
      what is and is not claimed) — done 2026-08-26, adapted honestly for
      this repo's difference from `er7-rust`: the names here *do* use the
      HL7® mark beyond fair use, and the file states the pending-permission
      status plainly. Passes `bin/check-trademarks`.

### Privacy and patient data

- [x] Add root `PHI.md` fronting `spec/phi/index.md` for a privacy-officer
      reader — done 2026-08-26, in the `fhir-rust` Q&A-table shape. Every
      row was re-verified against the spec or the code; the verification
      caught and fixed one overbroad spec claim (`std::fs` does appear in
      one library source — the XSD-dictionary generator, whose purpose is
      reading the schema files you name; spec/phi table row corrected in
      the same commit).

### Outreach

- [ ] Blocked until the trademark item above is done (issue templates landed
      2026-08-26) — then execute `help/outreach/index.md`'s phase 1. Its
      stale note that the trademark notice "is not yet carried on the
      website" was corrected 2026-08-26 (site footer, crate rustdoc, and —
      later the same day — every Cargo.toml description carry it).
- [x] Add `version` and `date-released` to `CITATION.cff` — done
      2026-08-26: cites the umbrella `hl7` crate, 0.1.4, released
      2026-08-26 per CHANGELOG.md, with a comment saying the workspace
      versions independently.
- [ ] Add a DOI to `CITATION.cff` once a Zenodo deposit exists (needs the
      owner's Zenodo account).
- [x] Refresh BENCHMARKS.md against the released crate versions or mark its
      figures as historical (it cited `hl7-2` 0.2.3; released is 0.2.6) —
      done 2026-08-26: the Criterion suite was re-run on `hl7-2` 0.2.6
      (`cargo bench -p hl7-2`, same machine and toolchain), and
      BENCHMARKS.md, `spec/benchmark/index.md`, and the website page all
      carry the new figures. The re-run also caught and corrected a wrong
      claim: the old table said the figures were over `er7` 0.1.2, but
      `Cargo.lock` has pinned 0.1.1 in every revision — the measurement was
      of 0.1.1.

### Audit and harmonization

- [x] Re-sync `spec/special-files-for-public-repos/index.md` with the
      `fhir-rust` canonical version (it was missing CODE_OF_CONDUCT.md,
      PHI.md, LICENSES/, FUNDING.yml, and a status section, and carried two
      typos: "optimizaiton", "Prker") — done 2026-08-26: full canonical
      list plus a status section adapted to this repo, with those typos
      fixed (and a third, "summries"), and TRADEMARKS.md added to the list
      now that this repo carries one.
- [x] Adopt a repository-wide link check and document-size budget in CI, per
      the `snomed-rust` convention — done 2026-08-26: `bin/check-docs`
      ported, `spec/docs-budget-and-links/` adapted, `docs` job added to
      `ci.yml`. Its first run found nine broken links in a divergent
      `AI_STATEMENT.md` draft hiding in
      `spec/special-files-for-public-repos/`, resolved to a pointer at the
      root document (the `fhir-rust` precedent). Now green: 99 tracked
      documents, all within budget, zero broken relative links.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
