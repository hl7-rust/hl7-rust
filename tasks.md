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
- [x] Sign commits and tags going forward. **Tagging is done, 2026-08-27:**
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
      joel@joelparkerhenderson.com`), then were discarded. The first real
      signed commit landed and was pushed to all three remotes; checked
      against each host's own API rather than assumed. **GitHub: verified**
      (`verification.verified: true, reason: "valid"`). **GitLab:
      verified** (`verification_status: "verified"`, key titled
      `jph-code-signing`). **Codeberg: verified as of 2026-08-28** —
      re-checked directly against Codeberg's own API on a later commit
      (`f6a146d`) rather than assumed from the earlier registration gap:
      `verification.verified: true`, signer `joelparkerhenderson`, key
      fingerprint matching. All three remotes now verify the same key on
      the same commits. MAINTAINERS.md's tagging/signing paragraph was
      stale by one host at the moment this line was first written, and was
      corrected in the same commit — the sentence you might expect here
      promising a follow-up was itself the drift; fixed rather than left.
- [ ] Adopt Trusted Publishing for crates.io releases, in place of the
      long-lived API token, once it is production-ready across GitHub,
      GitLab, and Codeberg — the policy in
      `spec/trusted-publishing/index.md`. Checked 2026-08-28, so this is
      against current fact: GitHub Actions has been GA since July 2025;
      GitLab CI/CD support is GitLab.com-only beta (self-hosted GitLab
      unsupported); Codeberg/Forgejo has none on crates.io's side yet,
      though Forgejo has done OIDC token-issuance work on its own side.
      Adopting it for GitHub alone would leave the token this project is
      trying to retire long-lived regardless, since the workflow still has
      to exist for the other two mirrors — a partial win the stated policy
      declines. MAINTAINERS.md's publishing-identities section and RFC.md
      §8 both state this now; revisit when Codeberg lands.
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
- [x] Enable Dependabot, per `spec/dependabot/index.md` (found at
      `spec/spec/dependabot/index.md` — a doubled-up path, moved to the
      canonical `spec/<name>/index.md` location in the same change rather
      than left as a stray nested directory) — done 2026-08-29, both
      halves. Security updates: `dependabot_security_updates` was
      `disabled` (checked via the API before touching it, not assumed);
      enabled with `PUT /repos/hl7-rust/hl7-rust/automated-security-fixes`,
      re-checked afterward and now reports `"enabled"`. Vulnerability
      alerts were already on, which that endpoint needs as a prerequisite.
      Scheduled updates: `.github/dependabot.yml`, one entry per ecosystem
      this repository actually has — `cargo` at the root (covers all
      fourteen members through the one shared `Cargo.lock`),
      `github-actions` at the root, and `npm` under
      `hl7-rust.github.io` (GitHub's npm handler reads `pnpm-lock.yaml`
      natively; there is no separate "pnpm" ecosystem value). Weekly, to
      match the same best-effort cadence SECURITY.md and MAINTAINERS.md
      already state elsewhere rather than inventing a daily pace nobody is
      staffed to keep up with. This is a distinct capability from
      `cargo deny` above, not a duplicate of it: `deny` catches what is
      already in the lockfile today; Dependabot opens the PR when a new
      advisory or a new version appears after that. **First live PRs
      confirmed it works, and caught a real gap in the config within a
      day**: ten PRs opened across all three ecosystems; nine passed CI
      clean, and the tenth — `dtolnay/rust-toolchain` "bumped" from
      `1.96` to `1.100` — failed, because that pin is not a version to
      keep current, it *is* the MSRV floor
      `spec/rust-msrv-n-minus-2/index.md` states. Dependabot's
      github-actions ecosystem reads the tag the same way it would
      `checkout@4` → `checkout@7`; CI's `msrv` job correctly rejected the
      toolchain the policy never chose. Fixed with an `ignore:` rule on
      that one dependency name — raising the MSRV stays a deliberate,
      spec-driven change, never an automated PR — and the failing PR
      closed. The `@stable` pin in the other job was never at risk the
      same way; it names a channel, not a version, so Dependabot has
      nothing numeric to bump.
- [x] Merge the nine remaining PRs — done 2026-08-29. The four
      cargo/actions ones (`actions/checkout` v4→v7, `syn` 2.0.119→3.0.3,
      `criterion` 0.5.1→0.8.2, `er7` 0.1.1→0.1.3) really were covered by
      real CI — `clippy --all-targets`/`test` compile the derive crates
      against `syn` and the benches against `criterion` — so merged on
      that evidence, then re-verified with the full local gate set on the
      combined result before pushing to GitLab and Codeberg, which don't
      see GitHub PR merges on their own.

      **The five site PRs (TypeScript 5→6, Svelte, SvelteKit,
      lily-design-system, `vite-plugin-svelte` 5→6) were a different
      problem: `ci.yml` never runs `pnpm` anything, so their green
      checkmarks proved nothing about whether the site still builds** —
      confirmed by grepping the workflow file for `pnpm`/`svelte-check`/
      `vite build` and finding none. Verified each for real instead:
      fetched every branch into its own `git worktree`, ran
      `pnpm run check` (0 errors each) and `pnpm run build` (each wrote a
      complete `build/` output) before merging any of them. All five
      touch the same `package.json`/`pnpm-lock.yaml`, so each merge
      reopened a conflict on the next PR in line — `@dependabot rebase`
      requested, waited for `mergeable` to flip, re-verified the *rebased*
      branch (not just re-trusted the old one), merged, repeat, four
      times. After the tenth PR landed, ran `pnpm run check` and
      `pnpm run build` once more on the real merged `main` — not a
      worktree — since five bumps that each pass alone had never been
      tested together; both came back clean.

      **This is a real, standing gap, not just a one-time inconvenience**:
      `ci.yml` has no job that builds `hl7-rust.github.io`, so every future
      site-related Dependabot PR will keep showing a meaningless green
      check until one is added. Worth its own task — a `site` job running
      `pnpm install && pnpm run check && pnpm run build` — rather than
      relying on manual worktree verification each time.

- [x] Add a `site` job to `ci.yml` that runs
      `pnpm install && pnpm run check && pnpm run build` against
      `hl7-rust.github.io` on every push/PR — done 2026-08-29. Closes the
      gap found above: Dependabot's green checkmark on a site-related PR
      used to prove nothing, since no CI job ever touched `pnpm`.

      First attempt pinned `pnpm/action-setup@v4` to major version 11
      (matching local `pnpm --version`) with `node-version: lts/*`. That
      passed a local `pnpm install --frozen-lockfile` / `check` / `build`
      run, so it looked verified — but the local install was silently
      using a gitignored, untracked `hl7-rust.github.io/pnpm-workspace.yaml`
      left over on this machine, which grants `esbuild` permission to run
      its postinstall script. A real CI run (fresh checkout, no such file)
      failed at `pnpm install --frozen-lockfile` with
      `[ERR_PNPM_IGNORED_BUILDS]`: pnpm 10+ no longer reads the
      `pnpm.onlyBuiltDependencies` field in `package.json` that this repo
      actually relies on, and refuses to install non-interactively when a
      build script is silently ignored. The existing (separate, already
      working) `hl7-rust.github.io/.github/workflows/deploy.yml` — which
      runs in the sibling `hl7-rust.github.io` repository, triggered by
      `make publish` — had already solved this by pinning
      `pnpm/action-setup@v4` to version 9, which still honors that
      `package.json` field. Fixed the new `site` job to match: version 9,
      `node-version: 20` (mirroring `deploy.yml` exactly rather than
      inventing a second convention). Re-verified for real this time by
      moving the local `pnpm-workspace.yaml` aside, deleting `node_modules`,
      and running `npx pnpm@9.15.9 install --frozen-lockfile` / `check` /
      `build` from a clean state that matches what CI actually sees — all
      three passed — before trusting the fix and re-pushing.

      `pnpm install --frozen-lockfile` (kept from the first attempt) means
      a PR that edits `package.json` without updating `pnpm-lock.yaml`
      still fails loudly instead of silently resolving. Also fixed a
      stale claim this surfaced: `AGENTS.md`
      already had the corrected wording for the `rust-version.workspace =
      true` exception (see the MSRV entry above), but `CONTRIBUTING.md`
      still said flatly "No `[workspace.package]` inheritance... without
      discussion" with no exception noted — brought into agreement in the
      same commit, plus the site's own pre-PR gate commands added next to
      the Rust ones.
- [x] Close the "no SBOM" gap `SECURITY.md`, `MAINTAINERS.md`, `RFC.md` §8,
      and `plan.md`'s risk section all named — done 2026-09-01. Found while
      re-reading `plan.md` against `tasks.md`: the risk section called it
      "a standing confession" and `spec/professionalization/index.md` rule
      3 claimed it was "tracked in `tasks.md`", but no line here ever
      tracked it — the exact drift this file's audit sweeps exist to catch,
      just found this time by reading the declaring documents against each
      other instead of by a dedicated sweep.

      `fhir-rust`'s `fhir-ci.yml` is the family pattern (a `supply-chain`
      job named "SBOM" running `cargo cyclonedx --format json --all`,
      uploaded via `actions/upload-artifact`), adapted as a new `sbom` job
      in this workspace's own `ci.yml` rather than `security.yml` — `ci.yml`
      is this repo's analogue of `fhir-ci.yml` (build/lint gates), while
      `security.yml` is the analogue of `fhir-security.yml` (`cargo deny`
      only), so the job joins the file that already matches its sibling.
      Verified locally before wiring into CI: `cargo cyclonedx --format
      json --all` at the workspace root produced exactly fourteen
      `<crate>/<crate>.cdx.json` files, one per workspace member (matching
      `Cargo.toml`'s `[workspace] members` count, not the twenty crate
      roots — `cargo-cyclonedx` operates per-package, and the six
      `src/main.rs` binaries are extra targets inside existing packages,
      not separate packages), each valid CycloneDX 1.3 JSON with the
      expected `metadata.component.name`; deleted afterward rather than
      committed (`git status` clean).

      This closes *generation*, not distribution: nothing attaches the
      documents to a crates.io release, since publishing is still a manual,
      untagged act — `fhir-rust`'s docs describe its own SBOM as shipping
      "with the release," which is not true here, and every doc updated
      below says so explicitly rather than borrowing the stronger sibling
      claim. `SECURITY.md`'s known-gaps entry, `MAINTAINERS.md`'s "what is
      not here yet" list, `RFC.md` §8 (both the factual sentence and the
      "what a procurement reviewer might still want" list), `plan.md`
      (workstream 3 and the risk section), `spec/professionalization/index.md`
      (rules 3 and 4's status), and `hl7-rust-maintainer-skill/SKILL.md`'s
      "What CI actually gates" section were all updated in the same change
      — the same failure mode the second and third accuracy sweeps kept
      finding (a gap closed in code but not in every document that named
      it) checked for directly this time instead of waiting for the next
      sweep to catch it. `./bin/check-docs` and `./bin/check-trademarks`
      both still clean after the doc edits; the new `ci.yml` job's YAML
      parsed with `python3 -c "import yaml; yaml.safe_load(...)"` before
      committing since GitHub only validates it on push.

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
      the owner raises it again.
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

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
