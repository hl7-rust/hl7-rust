# Tasks — `serde-hl7`, `serde-hl7-v2`, `serde-hl7-v3`

Execution checklist; rationale lives in [`plan.md`](plan.md). A `[x]` means
the work is **verified done**, not intended — check items off in the same
change that completes them, with the evidence named.

## Evaluation (verified 2026-09-10)

- [x] Confirmed no Serde code existed to extract: `grep -rIl serde` over
      the workspace excluding `target/` hit only `Cargo.lock`, `spec/phi`,
      four `AGENTS.md` files listing Serde under *don't*, and one website
      page. No member `Cargo.toml` named `serde`.
- [x] Read `serde-er7` in full (Cargo.toml, every `src/*.rs`, all eleven
      spec sections, tests, examples, README) to mirror its structure.
- [x] Checked crates.io: `serde-hl7`, `serde_hl7`, `serde-hl7-v2`, and
      `serde-hl7-v3` all returned 404 from
      `https://crates.io/api/v1/crates/<name>` on 2026-09-10.
- [x] Recorded the gate in root `plan.md` → Non-goals, and the
      maintainer's explicit override of it for these three crates.

## Build (done 2026-09-10)

- [x] Decided the names, the umbrella-with-features shape, and the
      sub-crate split — the maintainer's, in session.
- [x] Wrote `serde-hl7-v2`: `src/{lib,message,node,diagnostic,kind,version,strict}.rs`,
      `tests/integration.rs`, three examples, `spec/` with index and
      eleven sections, README, AGENTS.md, CLAUDE.md, LICENSE.md.
      41 unit + 12 integration + 10 doc tests pass.
- [x] Wrote `serde-hl7-v3`: `src/{lib,object,element,message,vocabulary,null_flavor,rim,strict}.rs`,
      `tests/integration.rs`, three examples, `spec/` with index and
      eleven sections, README, AGENTS.md, CLAUDE.md, LICENSE.md.
      32 unit + 9 integration + 17 doc tests pass.
- [x] Wrote `serde-hl7`: `Cargo.toml` with `v2`/`v3` features (both
      default), `src/lib.rs`, README, AGENTS.md, CLAUDE.md, LICENSE.md.
      Builds with `--no-default-features`, with each feature alone, and
      with both; the crate-level doctest passes.
- [x] All six examples run (`cargo run -p <crate> --example <name>`).
- [x] `cargo clippy --all-targets -- -D warnings` clean for all three with
      the pedantic group on; `cargo fmt --check` clean;
      `cargo rustdoc --lib -- -W missing-docs` clean for all three.
- [x] `bin/check-trademarks` and `bin/check-docs` pass with the new files
      staged.

## Workspace integration (done 2026-09-10)

- [x] Root `Cargo.toml` members and `.github/workflows/ci.yml`'s rustdoc
      loop list the three crates; "fourteen" → "seventeen" in both.
- [x] `spec/phi/index.md` "No serialization framework" row amended to name
      the three bridge crates as the opt-in exception.
- [x] Root `plan.md` non-goal reworded; crate counts updated in `plan.md`,
      `tasks.md`, `README.md`, `CHANGELOG.md`, `MAINTAINERS.md`, `RFC.md`,
      `CONTRIBUTING.md`, `AGENTS.md`, `AI_STATEMENT.md`, `SECURITY.md`,
      `spec/trusted-publishing/index.md`, `spec/release-process/index.md`.
- [x] Root `README.md` crate tree and table gained the three crates.
- [x] `llms.txt`, `llms.json`, and the website's `static/llms.json` gained
      the three crates.
- [x] Website: three entries in `src/lib/data/crates.ts`, three
      `src/routes/crates/<name>/` pages, crate counts updated in the
      spec, FAQ, architecture, and agent-skill pages and the navigation
      blurb; `pnpm run check` and `pnpm run build` pass.
- [x] `CHANGELOG.md` entry for the sixth release, dated 2026-09-10,
      written before publishing per the runbook's step 3.

## Release (the runbook in `spec/release-process/index.md`)

- [x] Step 1–2: first release, 0.1.0 for all three; no existing
      inter-crate requirement changes (`hl7-2` 0.3.0 and `hl7-3` 0.2.0 were
      already on crates.io). `cargo +1.96 check --workspace --all-targets`
      clean — done 2026-09-10.
- [x] Step 3: `CHANGELOG.md` entry "2026-09-10, sixth release" written
      before publishing, in commit `5e17a4e`.
- [x] Step 4: `cargo test`, `clippy --all-targets -D warnings`, `fmt
      --check`, `rustdoc -W missing-docs` per new crate, the MSRV check,
      `bin/check-trademarks`, `bin/check-docs`, and the site's `pnpm run
      check`/`build` all clean; commit `5e17a4e` pushed to GitHub and
      Codeberg; GitHub Actions run 34452665459 (`ci`) and 34452665554
      (`security`) green. **GitLab rejected the push as non-fast-forward:**
      its `main` holds `9d4d98d`, an earlier version of local `8c622b2`
      with a different tree (`hl7/Cargo.toml`, `hl7/README.md`), a
      divergence that predates this work; resolving it needs a force push
      the maintainer has to choose.
- [x] Step 5: `cargo publish` in dependency order — `serde-hl7-v2` 0.1.0,
      `serde-hl7-v3` 0.1.0, then `serde-hl7` 0.1.0 — all three reported
      "Published … at registry `crates-io`" on 2026-09-10.
- [x] Step 6: `cargo package` for each confirms a literal
      `rust-version = "1.96"`, the path dependencies stripped to versions,
      and the umbrella's `default = ["v2", "v3"]` in the published manifest.
- [x] Step 7: annotated, SSH-signed tags `serde-hl7-v2-v0.1.0`,
      `serde-hl7-v3-v0.1.0`, `serde-hl7-v0.1.0` on `5e17a4e`, verified with
      `git tag -v`, pushed to all three remotes.
- [x] Step 8: recorded here, in `CHANGELOG.md`'s "Released" line, and in
      root `tasks.md`.

## Later

- [ ] `Deserialize` for `serde-hl7-v2::Node`, if `hl7-2` grows a public
      `Node` constructor (v2 spec §9.1).
- [ ] Add `serde-hl7` to the "crate family" table in `serde-er7/README.md`
      in the sibling `er7-rust` repository.
- [ ] Wire-shape decision recorded for future readers: `serde-hl7-v2`'s
      `Node` shape is deliberately different from
      `hl7-2-from-er7-into-json`'s output (a faithful six-key node vs. a
      name-keyed nested object); cross-link the two READMEs when that
      crate's docs are next touched.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7. This project is an independent work.
