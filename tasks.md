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
- [x] PHI analysis exists at `spec/phi/index.md` (9 sections, checkable
      claims, explicit non-defenses).
- [x] Outreach research exists at `help/outreach/index.md` (audiences,
      channels, 90-day sequence, prerequisites) and correctly gates promotion
      on trademark resolution and issue-response readiness.
- [x] BENCHMARKS.md is measured (Criterion, machine/toolchain/date recorded),
      not written.

## Next up

Grouped by `plan.md` workstream. Order within a group is priority order.

### Security and supply chain

- [ ] Adopt Trusted Publishing for crates.io releases, in place of the
      long-lived API token, once it is production-ready across GitHub,
      GitLab, and Codeberg — the policy in
      `spec/trusted-publishing/index.md`. Re-checked 2026-11-20 (previously
      2026-09-01), so this is against current fact, not a stale one: GitHub
      Actions has been GA since July 2025; GitLab CI/CD support is still
      GitLab.com-only beta (self-hosted GitLab unsupported, per the Rust
      blog's 2026-01-21 development update); Codeberg/Forgejo still has no
      support on crates.io's side, and the wait got no closer this check —
      Forgejo's own tracking issue
      (`codeberg.org/forgejo/forgejo/issues/9939`) was closed 2026-03-20 as
      "not actionable," a maintainer stating plainly that trust has to be
      established from crates.io's side first, which hasn't happened; the
      "Forgejo has done OIDC token-issuance work on its own side" framing
      this line carried before is now stale and dropped rather than
      repeated. Adopting it for GitHub alone would leave the token this
      project is trying to retire long-lived regardless, since the
      workflow still has to exist for the other two mirrors — a partial
      win the stated policy declines. MAINTAINERS.md's
      publishing-identities section and RFC.md §8 both state this now;
      revisit when Codeberg lands.
- [x] Consciously accept, here, the declared gaps closing the SBOM and
      fuzzing items above left standing — `spec/professionalization/index.md`
      rule 3 requires every gap `SECURITY.md` or `MAINTAINERS.md` names be
      "closed or consciously accepted in `tasks.md`," and naming them in a
      spec-file status paragraph (which rule 3 itself now does, dated
      2026-09-01) is not the same as this file doing the accepting. Doing
      that now rather than leaving the rule's own audit trail with a
      hole one level deeper than the one this session already closed
      twice today:
      - **`er7`'s unfuzzed ER7 parsing surface.** Out of this repository's
        control — `er7` is a dependency this workspace does not publish,
        so there is no code here to add a fuzz target to. Would close if
        `er7` grew its own, or if this project ever vendored the parsing
        layer, neither planned.
      - **No release signing (of the crates.io artifact), no
        reproducible-build attestation.** No mechanism identified.
        `cargo publish` has no first-party signing story as of this
        writing; revisit if one appears (Sigstore-for-crates and similar
        efforts exist but are not something a single-maintainer project
        can adopt unilaterally — it needs crates.io's side to exist first,
        the same shape of wait as the Trusted Publishing item above).
      - **No published threat model — closed 2026-11-19.**
        [`spec/threat-model/index.md`](spec/threat-model/index.md) now names
        the assets, trust boundaries, attackers, controls, and residual risks.
        It records that recursive JSON/XML depth and incomplete
        MLLP buffering are bounded while complete inputs have no universal
        byte limit, and gives the deployment controls that therefore remain
        the caller's responsibility. `SECURITY.md` and the public repository
        maps link to it in the same change.
      - **No release automation (`MAINTAINERS.md`), in the CI-workflow
        sense.** A crates.io release is still `cargo publish` from the
        maintainer's own laptop for all fourteen crates — 2026-09-02's
        `spec/release-process/index.md` widened *who on that laptop* may
        decide to run it (an agentic tool, within its bounds), not *where*
        it runs from; a scheduled or CI-triggered publish is still exactly
        this gap. Automating that (a tag-triggered workflow) is real
        engineering, not a documentation fix, and per
        `spec/trusted-publishing/index.md`'s policy it would have to carry
        Trusted Publishing's OIDC flow rather than a workflow-embedded
        long-lived token, so it is blocked on the same Codeberg wait as
        that item, not merely deferred alongside it.
      - **No committed security-response window (`MAINTAINERS.md`).**
        The one gap on this list that is not awaiting a mechanism: a
        deliberate stance, stated where it is declared — one person, no
        on-call rotation, a 14-day escape hatch to publish instead of a
        deadline nobody could keep. Revisit only if the maintainer model
        changes (a second maintainer, per `GOVERNANCE.md`'s succession
        section).

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
- [x] **Authorize an agentic tool to decide, on its own judgment, that a
      change warrants a crates.io release and execute `cargo publish`** —
      done 2026-09-02, at the maintainer's explicit direction. Before this,
      `AI_STATEMENT.md` §5 put "what ships in a release" at level `none`
      and §6 said flatly "no commit or release is automated"; this is the
      first time `autonomous` (per the document's own §3 vocabulary) has
      applied to anything in this workspace.

      New `spec/release-process/index.md` does two things at once: it
      formalizes the release runbook four prior releases followed ad hoc
      (semver-per-crate, the inter-crate version-requirement check the
      fifth release needed on twelve dependencies, `CHANGELOG.md`-entry-
      first, tag-and-sign, `cargo package` as a manifest sanity check —
      never written down before, reconstructed from `tasks.md`'s own log),
      and states the new authorization's bounds on top of it: only inside
      a live interactive session on the maintainer's own machine, never
      scheduled or unattended; only the credential already configured
      there; every precondition (pushed and CI-green, a real landed
      change, semver and inter-crate requirements verified, a
      `CHANGELOG.md` entry, a `tasks.md` record) gates the action
      regardless of who decided it; and a closed list stays exclusively
      the maintainer's — a new crate's first release, an MSRV bump,
      anything touching license/trademark/PHI-handling code, or yanking a
      version.

      Scope was asked directly, not assumed; the maintainer chose the
      broader of two offered options ("on its own judgment," not "only
      when told to publish this specifically"), named `autonomous` as
      such rather than softened.

      Updated in the same change, cross-referencing the new spec
      rather than restating its bounds independently:
      `AI_STATEMENT.md` (§1, §5's table split into two rows plus a new
      `autonomous` one, §6, §12 twice, Annex A, Annex B — version bumped
      **1.2.0 → 2.0.0**, major rather than minor, because this is the
      first practice change to cross a vocabulary tier the document has
      had since 1.0.0, not a factual correction like the two prior minor
      bumps), `MAINTAINERS.md` (the bus-factor sentence, the crates.io
      token's table row, and the "no release automation" gap reworded to
      distinguish CI-workflow automation, still absent, from this),
      `GOVERNANCE.md` ("who decides"), `RFC.md` §8, and `SECURITY.md`
      (a new bullet under Known gaps naming exactly what widened and what
      did not, for a supply-chain reviewer specifically).

      Verified: `./bin/check-docs`/`./bin/check-trademarks` clean across
      the new spec file and every edited document; Annex B's YAML
      re-parsed with PyYAML after `release-decisions` split into
      `release-scope`/`release-execution`, and `autonomous-use` renamed to
      the one scoped activity plus a pointer to the bounding spec.
- [x] **Make `AI_STATEMENT.md` §4/§10 precise about the `Co-Authored-By:`
      trailer** — done 2026-09-02, same day, flagged directly: the two
      sections risked being read as "a tool is never named anywhere,"
      when actual practice (every commit in this history) literally uses
      git's own co-authorship trailer convention to name the model. Fixed
      to say so plainly — the trailer names the tool, deliberately; the
      `Author:`/`Committer:` fields and any signature stay the
      maintainer's alone, and that narrower claim is the one that
      actually matters. Also documented the `Claude-Session:` trailer
      (added to practice 2026-08-30, never described here until now) and
      that the disclosed model name varies by session (`Sonnet 5`,
      `Opus 5`, `Fable 5` all appear in history). `CONTRIBUTING.md` gained
      a "Using AI tools" section stating the same thing — missing
      entirely before this, though `fhir-rust`'s copy already has one.
      `AI_STATEMENT.md` version bumped **2.0.0 → 2.1.0** (a clarifying
      fix, not a new practice, so minor rather than major again).
      Verified: `./bin/check-docs`/`./bin/check-trademarks` clean; Annex
      B's YAML re-parsed after the version bump; `git log --format='%an/%cn'`
      re-checked before writing the accountability claim rather than
      assumed — it also surfaced Dependabot's automated commits as the
      one other non-maintainer actor in this history, out of this
      document's scope and named as such rather than silently excluded.
      Freed room for this entry by moving the "Sign commits and tags"
      entry above to `tasks-archive.md`, its own budget pressure from
      today's earlier entries.

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
- [ ] Add a DOI to `CITATION.cff` once a Zenodo deposit exists.
      **Deliberately deferred, 2026-09-02** — the owner's decision, not a
      stalled blocker: the deposit needs the owner's own account, the same
      shape of call as Open Collective below, not happening right now by
      choice. Revisit only when the owner raises it again.

### Funding

Per `spec/free-open-source-funding/index.md`.

- [x] Set up GitHub Sponsors — already live (re-verified 2026-08-28:
      `github.com/sponsors/joelparkerhenderson` returns a real sponsor
      page, not a "not accepting sponsors" state).
- [ ] Set up Open Collective. **Deliberately deferred, 2026-08-28** — the
      owner's decision, not a stalled blocker: creating a collective and
      choosing a fiscal host is an identity/financial step nobody should do
      but the owner, and it is not happening right now by choice. Checked
      before deferring, so the deferral is informed rather than an excuse:
      no collective exists yet at any plausible slug —
      `opencollective.com/{hl7-rust,hl7rust,hl7-for-rust,hl7-2,er7-rust,
      joel-parker-henderson,joelparkerhenderson}` all resolve to the site's
      generic search shell (`<title>Search</title>`), not a real
      collective. `.github/FUNDING.yml`, `CONTRIBUTING.md`, and `NEWS.md`
      say so plainly rather than pointing at a slug that does not exist —
      the last thing a funding page should be is broken. Revisit only when
      the owner raises it again. **Reaffirmed 2026-09-02** alongside the
      Zenodo deferral above.
- [x] Add `.github/FUNDING.yml` — already present (`github:
      joelparkerhenderson`), and now comments that an `open_collective:`
      line will be added once the item above is unblocked.
- [x] Update `CONTRIBUTING.md`'s Money section to match — done 2026-08-28,
      names GitHub Sponsors as live and Open Collective as not yet set up
      rather than staying silent about the gap.
- [x] Update `NEWS.md` to match — done 2026-08-28, adds a `## Funding`
      section with the same two facts.
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

- [x] **Preserve JSON scalar values or report a conversion error**
      (2026-11-19). Public converter regressions failed on a lone surrogate
      and a bare minus before the fix. Valid cases cover CJK, Arabic, a
      decomposed accent, a supplementary character, a literal replacement
      character, and exact decimal and exponent spellings.
      [Converter integration tests](hl7-2-from-json-into-er7/tests/main.rs)
- [x] **Restore the local compiler compatibility check** (2026-11-19).
      The Make target now invokes the existing script and accepts installed
      patch releases. On Apple Silicon, the target ran the workspace suite
      and compile-checked all four fuzz workspaces on Rust 1.98.1 and
      1.99.1. [MSRV verification](spec/rust-msrv-n-minus-2/index.md#local-verification)
- [x] **Return complete routing identities in acknowledgements** (2026-11-19).
      The framed regression first failed on a missing universal ID. It now
      verifies all four composite fields, an empty namespace, nondefault
      separators, escaped data, and the exact echoed control ID.
      [MLLP integration tests](hl7-2-mllp/tests/main.rs)
- [x] **Demonstrate a legacy-encoded MLLP exchange** (2026-11-19).
      The runnable example verified byte preservation, the UTF-8 helper's
      `NotText` error, and the exact Latin-1 reply frame with MSH-18 set
      to `8859/1`. [Runnable example and test](hl7-2-mllp/examples/latin1_ack.rs)
- [x] **Establish cross-platform evidence** (2026-11-19). The
      default-feature interoperability jobs passed file/stdin conversion,
      CR/CRLF rendering, and loopback MLLP on Rust 1.100.0. The runners
      were `macos-26-arm64` with host
      `aarch64-apple-darwin`, and Microsoft Windows Server 2025
      image `windows-2025-vs2026` with host
      `x86_64-pc-windows-msvc`.
      [GitHub Actions run](https://github.com/hl7-rust/hl7-rust/actions/runs/44510248558)

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
- [x] Raise MSRV from N-3 to N-2 (`spec/rust-msrv-n-minus-2/index.md`,
      committed by the owner as `e1352d0`) — implemented 2026-08-29: current
      stable is 1.98.0, so the floor moves to 1.96, installed and verified
      locally (`cargo +1.96 check --workspace --all-targets` — clean, no
      code needed to change). The mechanism changed along with the number:
      `rust-version` now lives once, in the root `Cargo.toml`'s
      `[workspace.package]`, and all fourteen members inherit it via
      `rust-version.workspace = true` instead of each declaring "1.95"
      individually — the spec's own stated reason (`cargo metadata`
      confirms all fourteen resolve to 1.96). That is new use of
      `[workspace.package]` inheritance, which `AGENTS.md`'s working
      conventions previously said not to add without discussion; corrected
      there to describe the narrow exception rather than left contradicting
      the code the moment this landed. Propagated everywhere the old value
      or the old spec path was live prose rather than a historical record —
      fourteen `Cargo.toml`s, `.github/workflows/ci.yml`, `AGENTS.md`,
      `README.md`, `RFC.md`, `INSTALL.md`, `CONTRIBUTING.md`,
      `AI_STATEMENT.md`, `NEWS.md`, `COMPARISONS.md`,
      `help/outreach/index.md`, `spec/professionalization/index.md`, and
      seven website pages (typechecked and built clean). Left alone,
      deliberately: mentions inside dated `CHANGELOG.md`, `NEWS.md`, and
      `tasks.md` entries that record what was true *at the time* — rewriting
      those would falsify history, not fix staleness. **Released
      2026-08-29, fifth release, as a minor bump on all fourteen** — caught
      before publishing, not after: the first attempt patch-bumped, which
      directly contradicted `CHANGELOG.md`'s own standing rule that a
      raised MSRV is always breaking and never lands in a patch. Reverted
      and redone as minor bumps, which meant twelve inter-crate version
      requirements needed raising too (a `0.x` minor bump fails Cargo's
      caret compatibility, unlike a patch), all twelve re-verified
      satisfied before publishing. `cargo package` was used to confirm the
      published manifest actually carries a literal `rust-version = "1.96"`
      rather than the workspace reference, which would mean nothing outside
      this repository. The BrokenPipe test fix, declined for its own
      release two turns earlier as not worth one alone, shipped bundled
      into this one instead.
- [x] Comprehensive accuracy sweep across every documentation surface —
      done 2026-08-30, prompted by a plain "update, upgrade, harmonize,
      annotate, audit, fix" instruction. Five parallel read-only audits
      found roughly 50 concrete, independently-confirmed problems (broken
      website code examples, contradicted specs, stale version pins, an
      undercounted consumer list, a stale re-measured benchmark); three
      parallel fix passes plus direct edits here closed all of them. Full
      account moved to [`tasks-archive.md`](tasks-archive.md) 2026-09-01
      when it pushed this file over the 40 KB budget.
- [x] Second comprehensive accuracy sweep — done 2026-08-30, same day,
      after the two Agent Skills and `llms.txt`/`llms.json` were added.
      Four parallel read-only audits found eight concrete problems,
      including two live contradictions between governance documents
      (`SECURITY.md`/`CODEOWNERS` still saying commits were unsigned after
      signing had landed; `hl7-3/AGENTS.md` never updated for struct mode)
      and a wrong `er7` version transcribed into the benchmark
      re-measurement. Full account moved to
      [`tasks-archive.md`](tasks-archive.md) 2026-09-01.
- [x] Third comprehensive accuracy sweep — done 2026-08-30, same day
      again. Found local `main` was 11 commits ahead of every remote the
      whole time the second sweep's fixes were reported "done" — flagged
      to the maintainer rather than pushed, and a new rule added: a sweep
      isn't done until `git status` shows nothing ahead of remote — plus
      eight single-line content defects (repeated stale claims, a
      malformed YAML frontmatter, a website spec-count undercount
      recurring after being fixed once already). Full account moved to
      [`tasks-archive.md`](tasks-archive.md) 2026-09-01.
- [x] Fourth comprehensive accuracy sweep — done 2026-09-03, run as six
      parallel read-only audits (spec/, AGENTS.md/CLAUDE.md, README/root
      docs, the website, llms.txt/llms.json + agent skills, plan/tasks)
      against an explicit checklist, then five parallel fix passes.
      **35 findings, one real code-vs-doc judgment call** (a CLI
      `--strict` exit-code claim was wrong in three places; traced the
      actual control flow and fixed the docs, not the already-published
      binary), **zero false positives**. Highlights: the `/spec/` page's
      recurring miscount drifted back after two new specs landed
      (a prior sweep had fixed the same failure mode once already); a
      README worked example that didn't reproduce when actually run
      against its own later sample; a prior sweep's `SKILL.md` fix that
      never reached its website mirror; a false "byte-for-byte identical"
      `LICENSE.md` claim in two files; `hl7::v3` still called "not yet
      implemented" in three places despite existing since the first
      release. Full account moved to
      [`tasks-archive.md`](tasks-archive.md) 2026-09-03. Verified: full
      gate set clean (`cargo test --workspace`, `clippy`, `fmt`, MSRV
      check, `rustdoc` per crate, `check-docs`, `check-trademarks`,
      the site's `pnpm run check`/`build`).

### Serde support

- [x] **Evaluate extracting code into a `serde-hl7` crate** — done
      2026-09-10. Finding: nothing to extract; no crate used Serde, and the
      code a reader might call serialization (struct mode, the converters)
      is not Serde. Recorded in [`serde-hl7/plan.md`](serde-hl7/plan.md).
- [x] **Build `serde-hl7`, `serde-hl7-v2`, `serde-hl7-v3`** — done
      2026-09-10 at the maintainer's direction, overriding the "no new
      crates" non-goal for these three. Each sub-crate mirrors `serde-er7`:
      an eleven-section `spec/` whose rule index and §7.1 coverage table
      are checked by `cargo test`, hand-written impls, two runtime
      dependencies, `Strict<T>`, examples. 41+12+10 tests in v2, 32+9+17
      in v3, the umbrella's doctest; clippy pedantic, fmt, rustdoc,
      `check-trademarks`, `check-docs` all clean. Checklist in
      [`serde-hl7/tasks.md`](serde-hl7/tasks.md).
- [x] **First release of the three** — the maintainer's call
      (`spec/release-process/index.md`), made explicitly 2026-09-10, and
      done the same day: `serde-hl7-v2` 0.1.0, `serde-hl7-v3` 0.1.0,
      `serde-hl7` 0.1.0 published in that order from commit `5e17a4e`
      (CI run 34452665459 green), packaged manifests checked for a literal
      `rust-version = "1.96"`, three signed `<crate>-v0.1.0` tags pushed.
      Runbook steps with evidence in
      [`serde-hl7/tasks.md`](serde-hl7/tasks.md). One loose end, not
      caused by this release: the GitLab mirror's `main` had diverged
      earlier (an amended commit) and rejected the push; tags landed
      there, the branch did not.
- [x] **Node round-trip patch release** — 2026-11-19: `hl7-2` 0.3.3 and
      `serde-hl7-v2` 0.1.2; evidence in
      [`serde-hl7/tasks.md`](serde-hl7/tasks.md).
- [x] **Tag `hl7` 0.2.1 and 0.2.2** — found 2026-09-10: both were
      published 2026-09-09 (crates.io 11:32 and 11:35 UTC) from the
      "Improve name distinction" change with no `CHANGELOG.md` entry and no
      tags. Entry written 2026-09-10 ("2026-09-09, sixth release"); tags
      cut 2026-09-13 — `hl7-v0.2.1` on `8c622b2`, `hl7-v0.2.2` on
      `dc80f52`, SSH-signed, pushed to all three remotes — after
      downloading both `.crate` files from static.crates.io and confirming
      `Cargo.toml`, `src/lib.rs`, and `README.md` byte-identical to each
      commit's.

### Audits of 2026-11-25

Distilled from three audits of revision `249fd38` (VS Code engineering,
clinical interoperability, and a generic quality-improvement list); the audit
files themselves were removed after this was written. Every item is open.
Reproduce each finding before fixing it; the observed results were measured
on the audited revision.

#### Clinical interoperability (high priority)

- [x] **Validate primitive OBX-5 by OBX-2** (2026-11-25: spec §4.4, fix in
      `validate.rs`, test `checks_obx_5_against_a_primitive_named_by_obx_2`;
      the CLI now exits 1 on the audit's example. `CHANGELOG.md` entry written; `./bin/test-coverage`
      9,878/9,878 lines, 100%.) — `hl7-v2 --strict --check` exits 0
      with `NOT_NUMERIC` in an OBX-5 whose OBX-2 is `NM`: the validator
      resolves the type through the composite-type lookup, which has no entry
      for primitives. Resolve OBX-2 independently of composite expansion;
      test valid decimal, invalid second repetition, coded result, `SN`,
      explicit null, and local type; require a `ValueFormat` finding at the
      exact repetition. Update `hl7-2/spec/index.md` §§4.4 and 8 first.
- [x] **Calendar and clock validity for DT/DTM/TM** (2026-11-25: spec §8,
      `validate.rs`, test
      `rejects_impossible_dates_and_times_and_orphan_fractions`; changelog written;
      coverage 100% as above) — `20250230` and the
      fractional year `2025.1` pass validation (digits are counted, not
      interpreted). Define validity in `hl7-2/spec/index.md` §8 (leap days,
      precision, fractional seconds, offsets), keep year-only and month-only
      at supplied precision, and test 29 vs 30 February.
- [x] **Typed JSON loses OBX/NTE association** (2026-11-25: spec §3.1, `preserves_segment_order`,
      `--require-order`, two tests, changelog; coverage 9,911/9,911, 100%. Default output is
      unchanged, so existing consumers keep their wire shape.) — with `--flat`, or with a
      trailing local `ZXX` segment under default options, the reverse
      conversion reorders to OBX A, OBX B, note A, note B. Add a conversion
      option that reports or rejects unrepresentable interleaving before
      emitting JSON; test reconstructed observation–comment pairs for
      grouped, flat, and trailing-local-segment cases. Spec:
      `hl7-2-from-er7-into-json/spec/index.md` §§3–4.
- [ ] **Fix the lab example that teaches what `ok` means** — the quickstart
      and `hl7-2/samples/oru_r01.hl7` put total cholesterol and Rh type under
      one lipid-panel order, with `D^Rh positive^LN` presenting `D` as a
      LOINC answer. Use a second lipid observation or give the blood-bank
      observation its own order with a reviewed answer code; place each NTE
      next to the observation it qualifies; regenerate the quickstart output
      from the fixture; assert order/observation association and code/system
      pairs in the fixture test. Needs review by a laboratory informatician.
- [x] **Acknowledgement evidence in SOAP evaluators** (2026-11-25: both crates,
      spec §6.1, `evaluate_correlated`/`evidence`, tests incl. matching reply,
      different message, empty/unparseable 200, fault under 200, failing
      status; the lost-reply redelivery case is the receiver's and stays an
      interface test; coverage 10,064/10,064, 100%) — `hl7-2-soap` and
      `hl7-3-soap` map any 2xx with an empty, unparseable, or unrecognized
      body to `Accepted`, and take no expected message ID. Add a
      policy-aware evaluator beside the compatibility API (transport success
      vs correlated application acknowledgement; expose `CA` vs `AA`
      evidence). Test matching reply, reply to a different message, empty
      200, fault under 200, and lost reply followed by redelivery.
- [x] **Diagnostics can carry message text** (2026-11-25: `spec/phi/index.md`
      inventory corrected; `Diagnostic::redacted()` and `Kind::as_str` added in `hl7-2`,
      test `redacted_diagnostics_never_carry_message_text`; CLI `--redact`
      added with a test;
      `RedactedDiagnostic` in `serde-hl7-v2` (S16) added with tests; coverage
      10,131/10,131, 100%; changelog written. Remaining:
      bump `serde-hl7-v2`'s `hl7-2` requirement at release time) — `spec/phi/index.md` names only
      `ValueFormat`, but `StructureUnknown` interpolates a message-derived
      structure ID (MSH-9) and `SegmentUnknown` builds its path from the
      input segment name; the Serde wrapper serializes both. Correct the
      PHI inventory and add an operational formatter (fixed kind, severity,
      numeric positions, allowlisted segment labels). Test with synthetic
      markers in a value, a header, and an unknown segment name; none may
      appear in the collected output.
- [ ] **Qualify one clinical interface end to end** — start with a lab-result
      feed and a patient-identity feed: A40 merge (MRG-1/PID-3, same local
      number from two authorities), A08 omit/null/replace, preliminary →
      final → corrected result, local codes/units/`SN` comparators. Note
      the built-in dictionary gives a structure warning and exit 0 for
      synthetic `ADT^A40^ADT_A39`. Expected outcomes agreed with a domain
      specialist first; pin guide, dictionary, and terminology versions; run
      the relevant NIST suite if the feed adopts that profile. Synthetic
      fixtures only.
- [ ] **Independent dictionary comparison** for the entries a pilot uses
      (version-matched, independently sourced); publish checked subset and
      discrepancies per `spec/schema-data-provenance/index.md`.

#### Contributor checks and editor support

- [x] **Repair the pre-PR checklist** in `CONTRIBUTING.md` (2026-11-25; CI's
      rustdoc loop left as is pending the disk-use check): replace the
      `cargo llvm-cov --fail-under-lines 95` recipe with `./bin/test-coverage`
      (spec and CI require 100% of distinct instrumented lines), and replace
      `cargo rustdoc --lib -- -W missing-docs` (exits 101 at a virtual
      manifest) with
      `RUSTDOCFLAGS="-W missing-docs -D warnings" cargo doc --locked --workspace --all-features --no-deps --lib`.
      Check CI runner disk use before retiring the per-crate cleanup loop.
- [ ] **Close the checked-arithmetic gap** (2026-11-25: CI Clippy now runs
      `--workspace --all-targets --all-features`, clean locally; the
      `[workspace.lints]` table and module migration remain) — `spec/rust-checked-arithmetic`
      RCA-ENF-1 calls for workspace lint levels and an all-target,
      all-feature Clippy run; the root manifest has no `[workspace.lints]`
      table and CI's Clippy omits `--all-features`. Adopt
      `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
      in CI. Then migrate module by module (RCA-ENF-3): enabling
      `clippy::arithmetic_side_effects` on `hl7-2` lib yields 67 diagnostics;
      review each, test boundaries, and prove a temporary violation fails
      both CI and the local task.
- [x] **VS Code path through the parser** (2026-11-25: `.vscode/` files, a
      `.gitignore` exception for them, `CONTRIBUTING.md` walkthrough,
      `hl7-2/AGENTS.md` paths fixed; the first test and the CLI invocation
      ran from the terminal. Still unverified: fresh-profile F5 and CodeLens
      in the editor, Problems-panel click-through, a Windows launch entry.) — add `.vscode/extensions.json`
      (rust-analyzer), a Clippy `tasks.json`, a CodeLLDB `launch.json` for
      `hl7-v2 --query ZAC-2.1 hl7-2/samples/vendor.hl7` with
      `CARGO_PROFILE_DEV_DEBUG=2` in `cargo.env` (the dev profile's
      `line-tables-only` hides locals), and a short walkthrough naming the
      first test
      (`cargo test --locked -p hl7-2 --test main integration::a_release_difference_changes_how_a_field_reads -- --exact`).
      Update `hl7-2/AGENTS.md`, which still cites `tests/spec.rs` (now
      `tests/main/`). Remaining acceptance: fresh-profile F5 and CodeLens
      run, Problems-panel click-through, Windows debugger adapter.
- [x] **Correct the dictionary ownership map** (2026-11-25; §0 and AGENTS.md
      corrected, forward-JSON comparison still open) — `hl7-2/spec/index.md` §0
      says three converter crates carry copies of the v2.5 tables; only the
      forward JSON converter has its own type lookup, while both reverse
      converters use the numeric key/element suffix. Fix §0 and the crate
      guide together; keep the reverse round-trip goldens; compare forward
      JSON naming against the parser dictionary before proposing any
      consolidation.

#### Generic quality list (condensed; skip anything already done)

The third audit predates current state in places (it cites 95.25% coverage;
coverage is 100%, and fuzzing, Dependabot, and the threat model already
exist). Items not already covered above or elsewhere in this file:

- [ ] Property-based round-trip tests (proptest) for the v2 parser.
- [ ] Mutation testing (`cargo-mutants`) baseline and score tracking.
- [ ] Snapshot tests for message rendering; cross-version compatibility tests.
- [ ] Performance regression tests and memory-usage benchmarks (record in
      `spec/benchmark/`).
- [ ] `cargo-audit`/`cargo-deny` and SBOM generation (`cargo cyclonedx`) in
      CI/release; `security.txt`; security review checklist for new features.
- [ ] Website: in-browser "paste a message, see the parse" demo, glossary,
      "which crate do I need?" decision tree, expanded FAQ.
- [ ] README badges (CI, crates.io, docs.rs, license) on each crate.
- [ ] Maintainer onboarding checklist and ADRs for major design choices.
- [ ] Async MLLP and WebAssembly exploration (low priority).
- Promotion items (outreach, curated lists, Show HN, etc.) stay gated on
  trademark resolution per `help/outreach/index.md`; not re-listed here.

## Work in progress (handoff 2026-11-20)

- [x] Coverage backlog complete: 9,540/9,540 lines (100%); CI requires 100%.
- No local code work remains. External/deferred: Trusted Publishing,
      HL7 reply/outreach, Zenodo DOI, and Open Collective (details above).

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
