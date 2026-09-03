# Tasks — archive

Older, fully-resolved [`tasks.md`](tasks.md) entries, moved here verbatim
once `tasks.md` outgrew the 40 KB budget in
[`spec/docs-budget-and-links/index.md`](spec/docs-budget-and-links/index.md)
rule 1: "split it by topic ... or archive the older entries verbatim —
never meet the budget by deleting the record." `tasks.md` keeps a
one-paragraph summary and a pointer to each entry archived here; this file
is the full account, not a rewrite of it.

## Audit and harmonization

Three comprehensive accuracy sweeps, all under the "Audit and
harmonization" workstream in [`plan.md`](plan.md), moved here 2026-09-01
when adding the SBOM task pushed `tasks.md` over budget.

- [x] Comprehensive accuracy sweep across every documentation surface —
      done 2026-08-30, prompted by a plain "update, upgrade, harmonize,
      annotate, audit, fix" instruction. Five parallel read-only audits
      (`spec/`, `AGENTS.md`/`CLAUDE.md`, `README.md`/root docs, the
      website, `tasks.md`/`plan.md`), each required to verify every claim
      against real code/config rather than trust prose, found roughly 50
      concrete, independently-confirmed problems; three parallel fix
      passes (website, `spec/`, `AGENTS.md`/`CLAUDE.md`) plus direct edits
      here landed all of them. Highlights, not an exhaustive list — see
      `git log` for the full diffs:
      - **Three website code examples didn't compile**: a `&Options` where
        `convert_with_options` takes `Options` by value (3 pages), a
        `Dictionary::from_json_over` call with arguments in the wrong
        order, and an MLLP tutorial's `?` trying to propagate
        `ack::Error` into an `io::Result` function with no `From` impl for
        that conversion. Fixed by running the real CLI and checking real
        signatures, not by eyeballing the prose.
      - **A website page's own lede contradicted its own dynamic content**:
        `/spec/` said "ten" specs and crates in two places where its own
        rendered lists showed eleven and fourteen. `spec/dependabot`,
        `spec/free-open-source-funding`, `spec/rust-fuzz`,
        `spec/trusted-publishing` existed on disk but weren't linked from
        the page that's supposed to list every spec.
      - **Four website pages and the crate-dependency diagram directly
        contradicted the normative spec and the real `Cargo.toml` graph**:
        pages said raising MSRV "is a breaking change" where
        `spec/rust-msrv-n-minus-2/index.md` says the opposite ("routine
        and expected"); the architecture diagram drew edges (e.g.
        `hl7-2-soap` depending on `hl7-2`) that don't exist in any
        `Cargo.toml`, relabeled as a family/layer grouping instead of a
        literal dependency map.
      - **A "no filesystem access from library code" claim was false**:
        `hl7-2-from-xsd-into-json-dictionary/src/lib.rs` calls
        `std::fs::read_dir`/`read_to_string` directly to read XSD schema
        files — carved out as a named, explained exception instead of
        silently contradicted.
      - **Nine README/website install snippets pinned a version behind**:
        `hl7-2 = "0.2"` / `hl7-3 = "0.1"` where `Cargo.toml` says `0.3` /
        `0.2` — the same drift that motivated this whole sweep, just
        found by hand this time instead of by Dependabot.
      - **`hl7-2-xml-lite-helper` had five real consumers, every doc said
        three**: `hl7-3` and `hl7-3-soap` depend on it too (grepped every
        `Cargo.toml` to confirm), missing from its own `AGENTS.md`, two
        sibling crates' `AGENTS.md` files, and the crate's own
        verification build command — a change here could have passed its
        documented check while silently breaking two crates it doesn't
        name.
      - **`hl7/AGENTS.md` still described the crate as v2-only, `src/lib.rs`
        and nothing else, "about thirty lines"** — it re-exports `hl7::v3`
        too now and is 56 lines; corrected, and `hl7-3`'s own spec gained a
        section on struct mode (`FromElement`/`derive`) that had never
        been documented anywhere despite existing since near the crate's
        founding.
      - **`plan.md` (last touched 2026-08-26) flatly asserted "zero git
        tags, unsigned commits"** in two places — both false since
        2026-08-27 (70 tags backfilled; SSH signing verified on all three
        forges). `tasks.md`'s own log already had the correct history;
        `plan.md` just hadn't been re-read against it. Refreshed
        end-to-end rather than patched at just the flagged sentences: the
        "verified" date, crate/release counts, the MSRV N−3 mention (now
        N−2), the PHI section count, and the risk section's own benchmark
        watch-item note (see below).
      - **`BENCHMARKS.md` and `spec/benchmark/index.md` were stale against
        their own "current crates.io release" claim** — measured
        2026-08-26 against `hl7-2` 0.2.6, but `hl7-2` had since released
        twice more (0.2.7, then 0.3.0). Actually re-ran
        `cargo bench -p hl7-2` (not just relabeled the old numbers as
        historical) against 0.3.0 / `er7` 0.2.1, twice, to also check
        whether the previous doc's noted 13% run-to-run swing on
        `render/small` recurred — it didn't (about 1% this time) — and
        updated both files' tables, prose, and the website's
        `docs/benchmarks/` page consistently. `CITATION.cff` was also two
        releases stale (0.1.5 vs. the `hl7` crate's actual current 0.2.0)
        and `spec/phi/index.md` has 9 sections, not the "11" repeated in
        both `tasks.md` and `plan.md` — both fixed.
      - **`AI_STATEMENT.md` §12 still said "CI still runs no dependency
        audit"** after `cargo deny` landed in `security.yml` the same day
        that sentence was written by an earlier commit — corrected to
        name what's actually still true (CI gates no release; publishing
        is still a manual step).
      - Five crates' `AGENTS.md`/`CLAUDE.md` said "this repository" where
        the other nine said "this crate" — leftover phrasing from before
        the `git subtree` merges, not a deliberate distinction (one
        merged-in crate had already been updated, a sibling merged-in
        crate hadn't) — harmonized to "this crate" throughout.

      Every fix agent re-verified its own edits (`./bin/check-docs`,
      `./bin/check-trademarks`, and for the website, re-reading the
      changed lines) before reporting back; the full gate set below was
      run again after everything landed.
- [x] Second comprehensive accuracy sweep — done 2026-08-30, same day,
      after `hl7-skill/`, `hl7-rust-maintainer-skill/`, and root
      `llms.txt`/`llms.json` were added (per `spec/agent-skills/` and
      `spec/llms-json-and-llms-txt/`), prompted by the same plain
      "update, upgrade, harmonize, annotate, audit, fix" instruction
      repeated. Four parallel read-only audits (`spec/`,
      `AGENTS.md`/`CLAUDE.md`, `README.md`/root docs, the website), each
      required to verify every claim against real code/config, found
      eight concrete problems — fewer than the first sweep, since that
      one had already covered most of the surface, but two were live
      contradictions between governance documents, not just staleness:
      - **`SECURITY.md`'s Known gaps and `CODEOWNERS`' header comment
        both still said "commits and tags are not signed"**, flatly
        contradicting `MAINTAINERS.md` (updated 2026-08-28), which says
        signing landed 2026-08-27 with SSH verification on GitHub,
        GitLab, and Codeberg. `spec/professionalization/index.md` rule
        3's status line inherited the same staleness. This is the exact
        failure mode rule 3 exists to prevent — the declaring document
        wasn't updated in the change that closed the gap — caught only
        because this sweep re-reads every declared-gap document against
        each other, not just against `tasks.md`.
      - **`hl7-3/AGENTS.md` was never updated for struct mode**
        (`src/typed.rs`, the optional `hl7-3-derive` dependency), which
        landed 2026-08-19 — eleven days stale the entire time. Its own
        `spec/index.md` §8 documented the feature correctly throughout;
        only the agent playbook was wrong. `hl7-3-derive/AGENTS.md` was
        separately missing the trademark disclaimer footer every other
        crate's carries — not caught by `bin/check-trademarks`, since
        that file happens to use no bare word mark outside code spans.
      - **The 2026-08-30 benchmark re-measurement's own `er7` version was
        transcribed wrong**: `hl7-2` actually resolves `er7 0.2.1` in
        `Cargo.lock` (bumped in `aab39e7` the same day), but
        `BENCHMARKS.md`, `spec/benchmark/index.md`,
        `hl7-rust.github.io/docs/benchmarks/`, `plan.md`, and this file's
        own log entry above all said `0.1.3` — the version two unrelated
        crates (`hl7-2-from-json-into-er7`, `hl7-2-from-xml-into-er7`)
        pin, present in the same `Cargo.lock` under the same package
        name. Two spec agents found this independently. `RFC.md` §9
        separately still cited the *pre*-re-measurement tree-vs-path
        figures (1.50 ms / 4 µs) the same re-run had already superseded
        everywhere else (1.44 ms / 3.6 µs) — updated to match.
      - **Four crate-root rustdoc install snippets were one minor version
        behind**, in `src/lib.rs` doc comments rather than `README.md`
        (which the first sweep had already fixed, so this drift was
        specifically in the docs.rs-rendered copy, not the GitHub-
        rendered one): `hl7-2-derive` and `hl7-3-derive` each showed
        their target crate's previous version, and `hl7`/`hl7-2` each
        showed the other's previous version in the doc comment
        explaining how to skip the umbrella indirection.
      - **Both new skill pages' `frontmatter` code samples were
        truncated**: `/docs/agent-skill/` and `/docs/maintainer-skill/`
        each show a block captioned as that `SKILL.md`'s actual
        frontmatter, but both had silently dropped the description's
        final clause since the pages were first written — the
        trigger-phrase list on one, the disambiguating "see hl7-skill for
        that" parenthetical on the other, which is the sentence that
        actually distinguishes the two skills from each other.
      - Root `README.md`'s "Workspace documents" table never gained a row
        for `llms.txt`/`llms.json`, added in a later commit than the
        table itself.

      Every fix applied directly and re-verified: `cargo check` on the
      four crates whose `src/lib.rs` changed, `bin/check-docs` and
      `bin/check-trademarks` after every commit, and the website's
      `svelte-check` plus `vite build` (confirmed the escaped
      `MSH|^~\&|...` sample in the rewritten frontmatter renders
      correctly in the built HTML) before committing the website fixes.
- [x] Third comprehensive accuracy sweep — done 2026-08-30, same day
      again, same plain instruction repeated a third time. Five parallel
      read-only audits (governance/compliance, `spec/`,
      `AGENTS.md`/`CLAUDE.md`, root docs + website, skills + `llms.*`),
      each required to verify against real code/git/API state, found one
      structural problem and eight single-line content defects:
      - **Local `main` was 11 commits ahead of every remote** (GitHub,
        GitLab, Codeberg) the whole time the second sweep's fixes
        (`9437f7d`, `f35b494`) were reported "done" — so `SECURITY.md`/
        `CODEOWNERS` on all three public remotes still said "not signed,"
        contradicting the pushed `MAINTAINERS.md`, for four days.
        **Flagged to the maintainer, not pushed** — push is outward-facing
        and this sweep's mandate doesn't cover it on its own. New rule: a
        sweep isn't done until `git status` shows nothing ahead of remote.
      - `SECURITY.md`'s properties list and a second checklist item in
        `spec/phi/index.md` both still gave the flat "no filesystem access
        from library code" claim the first sweep had already carved an
        exception into everywhere else (`hl7-2-from-xsd-into-json-dictionary`
        reads XSDs by design) — two more places fixed to match.
      - `NEWS.md` alone still had the pre-remeasurement benchmark figures
        (1.50 ms/4 µs → 1.44 ms/3.6 µs) the second sweep says it fixed
        "everywhere else."
      - The website's `/spec/` page repeated the exact miscount the first
        sweep caught once already: "Fourteen today" plus a `<dl>` that
        hadn't grown for the two specs added 2026-08-30
        (`agent-skills`, `llms-json-and-llms-txt`) — now sixteen, both
        added; `svelte-check` (0 errors) and a `vite build` confirm the
        built `spec/index.html` renders the fix.
      - Root `AGENTS.md` undercounted crates with no pre-monorepo git
        history (named 3; commit messages show 6 — added
        `hl7-2-soap`, `hl7-2-xml-lite-helper`,
        `hl7-2-from-xsd-into-json-dictionary`).
      - `hl7-skill/SKILL.md`'s YAML frontmatter did not parse (unescaped
        `": "` in the plain-scalar `description`; PyYAML and js-yaml both
        rejected it) — the prior sweep's frontmatter fix touched only the
        website's rendered copy, never this source file. Reworded to drop
        the colon.
      - `hl7-rust-maintainer-skill/SKILL.md`'s checklist listed
        `cargo rustdoc --lib -- -W missing-docs` as a flat command, which
        fails from the repo root (virtual manifest) — pointed at `ci.yml`'s
        real per-crate `-p <crate>` loop instead.
      - `llms.txt`/`llms.json` (added after the first sweep's fix) had the
        same `hl7-2-xml-lite-helper` "three consumers" undercount already
        fixed in its `AGENTS.md` — corrected to all five.
      - `hl7-2-from-er7-into-json/spec/index.md` §8 claimed unconditionally
        that outputs "are joined with a blank line" — true in pretty mode,
        false in `--compact` (no trailing newline), confirmed by running
        the binary in both modes. Reworded to state the actual join and
        both outcomes.

      Verified before logging: `./bin/check-docs`/`./bin/check-trademarks`
      after every edit; the fixed frontmatter re-parsed with PyYAML and
      js-yaml; `llms.json` re-validated as JSON; the website change
      checked with the Svelte MCP autofixer, `svelte-check`, and a full
      `vite build`.

## Security and supply chain

Moved here 2026-09-02, when adding the AI release-authority task pushed
`tasks.md` over budget again.

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

## Security and supply chain, part 2

Moved here 2026-09-03, when the fourth comprehensive accuracy sweep's
summary entry pushed `tasks.md` over budget again.

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

## Audit and harmonization, part 2

- [x] Fourth comprehensive accuracy sweep — done 2026-09-03, prompted by
      the same "update, upgrade, harmonize, annotate, audit, fix"
      instruction, extended this time with an explicit checklist
      (spec/, AGENTS.md/CLAUDE.md size and accuracy, README/docs/examples/
      tutorials, llms.txt/llms.json, the two `*-skill` folders,
      `hl7-rust.github.io`, and plan/tasks accuracy and specificity).
      Run as six parallel read-only audits, one per named surface, each
      required to verify every claim against real code/config rather
      than trust prose (matching the first three sweeps' standard) —
      then five parallel fix passes grouped by non-overlapping file
      cluster, each given the verified findings to apply and told to
      re-verify with the real gate set afterward, not just re-assert.

      **35 distinct findings, one true code-vs-doc judgment call, zero
      false positives once independently re-checked.** Highlights:
      - **A real CLI behavior/doc mismatch**: `hl7-2/spec/index.md`,
        `hl7-2/src/main.rs`'s `USAGE`, and `hl7-2/README.md` all claimed
        `--strict` exits 2 on a validation failure. Traced the actual
        control flow: `--strict` turns a validation problem into a parse
        error, which propagates via `?` straight out of `run()` into
        `main()`'s generic `Err => ExitCode::from(1)` handler — so
        `--strict` failures exit **1**, not 2; only `--check` (without
        `--strict`) reaches the later `found_problems` path that exits 2
        while still rendering output. Fixed the three docs to state this
        precisely rather than change the binary's already-published,
        internally-consistent behavior — confirmed with a real
        `cargo run -p hl7-2 --bin hl7-v2 -- --help` after editing.
      - **Two subagents disagreed on the same claim** — whether
        `hl7-rust-maintainer-skill/SKILL.md`'s "raising MSRV is a
        breaking change" contradicts `spec/rust-msrv-n-minus-2/index.md`'s
        "not a breaking change to be avoided." Resolved directly: the
        spec's sentence was genuinely ambiguous (conflating "don't be
        reluctant to do it" with "not semver-breaking"), tripping up one
        careful reader and not another. Reworded to remove the ambiguity
        entirely rather than declare either subagent "right."
      - **A README example that doesn't reproduce**: `hl7-2/README.md`'s
        schema-mode snippet asserted `find("XPN.2") == "JOHN"` against a
        dictionary and segment that reappear later in the same file, run
        against the bundled `samples/vendor.hl7` — actually compiling and
        running it returned `"NORA"` (from `PID-5`, which precedes `ZAC`
        and also decodes as `XPN`). Fixed by making the snippet
        self-contained with its own minimal message rather than silently
        disagreeing with a sample introduced two sections later.
      - **A website page mismatch fixed by switching call sites, not just
        the matched variant**: `docs/patient-data`'s logging example
        matched `Error::BadValue` against `Message::get()`'s result, but
        `get()` can only ever return `Error::Path` — `BadValue` is
        struct-mode-only. Preserved the page's actual teaching point (the
        PHI-carrying `found` field) by switching the call to
        `message.decode::<Patient>()` instead, verified `decode()` is a
        real `Message` method with the right signature.
      - **The recurring `/spec/` page miscount, again**: "twelve" specs
        in the second `<dl>` where the real count is thirteen (19 total
        minus 6 crate specs) — the third sweep already fixed this exact
        failure mode once; it drifted back after `spec/release-process`
        and `spec/agent-skills` were added without the count being
        re-checked. Also found and fixed: `hl7-2-derive`/`hl7-3-derive`
        undercounted as depending on "syn and quote" (2) in six places
        across the website, when the real count is three
        (`proc-macro2`, `quote`, `syn`).
      - **A prior sweep's fix that didn't reach its own mirror**: the
        third sweep fixed `hl7-rust-maintainer-skill/SKILL.md`'s
        `cargo rustdoc --lib` command to the working `-p <crate>` form,
        but the website's `docs/maintainer-skill` page — which mirrors
        that file's checklist as a code sample — was never updated to
        match, and still had the broken flat command a reader could copy
        and watch fail.
      - **A false "byte-for-byte identical" claim**, in both root
        `AGENTS.md` and `hl7-rust-maintainer-skill/SKILL.md`: the 14
        crate `LICENSE.md` copies are identical to each other, but the
        root's is a longer, later-expanded superset (SPDX block,
        `LICENSES/` reference, trademark-scope section, added 2026-08-26)
        — the crate copies were never updated to match, so the claim was
        true of the crates but not of "everywhere." Fixed both files to
        say a new crate should copy an *existing crate's* `LICENSE.md`,
        not the root's.
      - A cluster of stale-since-founding claims: `hl7::v3` described as
        "not yet implemented" in three places (`hl7/README.md`,
        `hl7-2/README.md`, `hl7-2/spec/index.md`) though it has existed
        since the very first release; a consumer-count undercount for
        `hl7-2-xml-lite-helper` recurring in one spec file a prior sweep
        missed; two crates' specs falsely claiming a `fuzz/` workspace
        that doesn't exist for them; a misattributed origin commit and a
        wrong file count in `spec/schema-data-provenance/index.md`;
        `spec/docs-budget-and-links/index.md`'s own "largest document"
        claim gone stale now that `tasks.md` (not `hl7-2/spec/index.md`)
        is the biggest tracked file; a broken RFC.md table-of-contents
        anchor from a heading rename; an overbroad "no filesystem
        access" claim in `NEWS.md`'s press boilerplate missing the one
        named carve-out; a stale `make publish`-only claim in the
        maintainer skill after `make github-pages` landed; a `hl7-2-mllp`
        rustdoc warning from a doc-link only valid under a non-default
        feature, fixed at the source rather than merely noted; a "seven
        checks = exactly what CI runs" claim gone stale once the `sbom`
        job landed with no local pre-PR counterpart; a typo
        ("Monorope"); an imprecise PHI claim about `std::net` not
        accounting for one doc-example grep hit; and a misquote
        attributing invented exact wording to `spec/rust-msrv-n-minus-2/index.md`.

      Verified after every fix agent's pass, and again after all landed:
      `cargo test --workspace` (zero failures across every crate), `cargo
      clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo
      +1.96 check --workspace --all-targets`, `cargo rustdoc -p <crate>
      --lib -- -W missing-docs` looped over all 14 libs, `./bin/check-docs`,
      `./bin/check-trademarks`, and the website's `pnpm run check` (0
      errors) / `pnpm run build`. All clean. 35 files touched; none
      overlapped between the five concurrent fix agents (verified via
      `git status` after each landed).

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
