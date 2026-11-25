# Tasks archive, November 2026

Completed entries moved verbatim out of [`tasks.md`](tasks.md) on 2026-11-25 to keep it inside its size budget. Earlier entries are in [`tasks-archive.md`](tasks-archive.md).

## From Security and supply chain

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
- [x] Sign commits and tags going forward — done 2026-08-27. All 56 tags
      (14 crates x 4 releases, `<crate>-v<version>`, annotated) exist and
      are pushed; SSH signing (`gpg.format ssh`, a dedicated
      passphrase-protected key, deliberately not the unattended automation
      key) verified `true` against GitHub's, GitLab's, and Codeberg's own
      APIs directly, not assumed. Full account moved to
      [`tasks-archive.md`](tasks-archive.md) 2026-09-02.

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
- [x] Enable Dependabot, per `spec/dependabot/index.md` — done 2026-08-29:
      security updates and scheduled updates (cargo/github-actions/npm,
      weekly) both live. First live PRs caught a real config gap within a
      day (`dtolnay/rust-toolchain` "bumped" past the MSRV pin) — fixed
      with an `ignore:` rule. Full account moved to
      [`tasks-archive.md`](tasks-archive.md) 2026-09-03.
- [x] Merge the nine remaining PRs, and add a `site` job to `ci.yml` —
      done 2026-08-29. The four cargo/actions PRs merged on real CI
      coverage; the five site PRs exposed that `ci.yml` never ran `pnpm`
      anything, so their green checkmarks proved nothing — verified each
      by hand in a worktree, then closed the gap for good with a `site`
      job (`pnpm install && pnpm run check && pnpm run build`, pinned to
      match the working `deploy.yml` convention after a first attempt
      failed on a gitignored local file masking a real CI failure). Full
      account moved to [`tasks-archive.md`](tasks-archive.md) 2026-09-03.
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
      committing since GitHub only validates it on push. Same caveat this
      file gave the original CI stand-up: the `sbom` job has not yet been
      observed green on a GitHub-hosted runner — local verification ran
      the same `cargo cyclonedx` command, not the job's `actions/checkout`
      → `dtolnay/rust-toolchain` → `Swatinem/rust-cache` →
      `actions/upload-artifact` sequence, and the runner's disk/network
      behave differently — pending the next push.
- [x] Close the "hl7-2's dictionary reader" half of `SECURITY.md`'s
      "Fuzzing covers 3 of 14 crates" known gap — done 2026-09-01. Found
      the same way as the SBOM item above, by checking a self-declared gap
      against `tasks.md` rather than waiting for a sweep:
      `spec/professionalization/index.md` rule 3 requires every gap named
      in `SECURITY.md` to be "closed or consciously accepted in
      `tasks.md`," and this one was neither — `spec/rust-fuzz/index.md`
      named it as deliberate scope, but only there, not here.

      New `hl7-2/fuzz/` (its own `[workspace]`, matching all three
      existing fuzz crates' `Cargo.toml` boilerplate verbatim, per
      `spec/rust-fuzz/index.md`'s rule) with one target, `dictionary`,
      against `Dictionary::from_json` — schema mode's public entry point
      for a vendor-supplied dialect file, and the exact untrusted
      structured input the gap named. Asserts loading is total (any bytes
      produce a `Dictionary` or an `Error`, never a panic), that
      `structures` nesting stays within `src/json.rs`'s own `MAX_DEPTH`
      (256) — a claim `SECURITY.md` already made ("bounded dictionary
      nesting") and this target is the first thing to actually check
      rather than assume — and that the three "list the names, then look
      one up" accessor pairs agree with each other
      (`segment_names`/`segment_fields`,
      `type_names`/`is_composite`+`composite_components`,
      `structure_ids`/`structure`).

      Verified locally: `cargo +nightly fuzz build dictionary` compiled
      clean, then `cargo +nightly fuzz run dictionary --
      -max_total_time=30` ran ~6 million executions with zero crashes
      (`peak_rss_mb: 954`). No seed corpus committed — `corpus/`,
      `artifacts/`, `target/`, and `fuzz/Cargo.lock` are gitignored inside
      every fuzz crate here, this one included, matching all three
      existing ones, none of which ship a seed corpus either.

      `SECURITY.md`, `AI_STATEMENT.md` (§7's fuzz-targets row, version
      bumped 1.1.0 → 1.2.0 per its own §13 rule that a stale claim is a
      revision trigger — which also caught and fixed a separate stale
      cross-reference in the same file, the header table's "Review" row
      pointing at "§12" where the trigger list has always lived in §13),
      `spec/rust-fuzz/index.md`, `plan.md`'s opening state, and the
      website's `/spec/` page (still said "three crates" after the first
      sweep already corrected a different miscount on the same page) were
      all updated in the same change; `hl7-2/AGENTS.md` gained the
      "Fuzzing" section every other fuzzed crate's `AGENTS.md` already
      carries. `er7` itself — a dependency, not a crate this repository
      publishes — is the one piece of `SECURITY.md`'s original gap left
      standing, reworded to say so plainly rather than imply it is next.

      Verified: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
      `cargo fmt --check`, `cargo +1.96 check --workspace --all-targets`
      (confirms `hl7-2/fuzz`'s own `[workspace]` keeps it invisible to the
      MSRV floor, as the rule promises), `cargo rustdoc -p hl7-2 --lib --
      -W missing-docs`, `./bin/check-docs`, `./bin/check-trademarks`, and
      the website's `pnpm run check` / `pnpm run build` all clean.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
