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

## From Audits of 2026-11-25 (moved 2026-11-26)

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
- [x] **`serde-hl7-v2` dependency contract disagreed with its manifest**
      (2026-11-26) — `Cargo.toml` requires `hl7-2` 0.4.0 (for
      `Diagnostic::redacted_path`, rule S16), while spec §3.1 and
      `the_crate_has_exactly_two_runtime_dependencies` still said 0.3.3, so
      `bin/test-coverage` failed. Spec, manifest comment, and test now say
      0.4.0 and give both reasons (`redacted_path`, `Node::new`).
- [x] **Reclassify the eleventh release's dependent bumps (release blocker)**
      (2026-11-26: public APIs audited; `hl7-2-mllp` 0.3.0,
      `hl7-2-from-er7-into-xml` 0.8.0, `serde-hl7-v2` 0.2.0, `hl7` 0.3.0, and
      `serde-hl7` 0.2.0 (re-exports `serde-hl7-v2`) take minor bumps;
      `hl7-2-from-xsd-into-json-dictionary` exposes no `hl7_2` type and stays
      0.2.2; `serde-hl7-v3` 0.1.2 joins the release for the `visit_seq` fix.
      CHANGELOG table and rationale, `spec/release-process/index.md` step 1,
      README/site/doc install snippets updated. **Still open before publishing:**
      the standalone-consumer build against `cargo package` archives.)
      — the CHANGELOG plans patch bumps for crates that raise their
      `hl7-2` requirement 0.3 → 0.4 (`hl7-2-mllp`, `hl7-2-soap`,
      `hl7-2-from-er7-into-json`, `hl7-2-from-er7-into-xml`,
      `hl7-2-from-xsd-into-json-dictionary`, `serde-hl7-v2`, `hl7`). Most
      expose `hl7_2` types in public signatures (`hl7-2-mllp`'s
      `acknowledge_message(&v2::Message) -> v2::Message`, for example), so
      a downstream crate pinned to `hl7-2` 0.3 stops compiling on a
      `cargo update` ([Cargo's version-incompatibility hazards](https://doc.rust-lang.org/cargo/reference/resolver.html#version-incompatibility-hazards)).
      Audit each crate's public API for re-exported or exposed `hl7_2`
      types; give every such crate a minor bump (0.x → 0.(x+1)); update the
      CHANGELOG table and `spec/release-process/index.md` with the rule;
      before publishing, build a standalone consumer against the
      `cargo package` archives that passes a `Message` across two crates.
- [x] **Serde bridges cannot read back non-self-describing formats**
      (2026-11-26: `visit_seq` on `Message`, `Node`, `Diagnostic`, and the
      `serde-hl7-v3` macro; `serde-hl7-v2` rule S17, `serde-hl7-v3` §4.1;
      `postcard` dev-dependency (cargo-deny clean); tests
      `round_trips_through_a_non_self_describing_format`,
      `a_whole_message_round_trips_through_a_non_self_describing_format`, and
      every v3 JSON round trip now also goes through postcard. Each fails with
      `visit_seq` disabled.) —
      `serde-hl7-v2`'s `Message`, `Diagnostic`, and `Node` visitors
      implement `visit_map` only (`src/message.rs`, `src/diagnostic.rs`,
      `src/node.rs`), and `serde-hl7-v3`'s generated struct visitors
      likewise (`src/object.rs`). `bincode` and similar formats call
      `visit_seq` for structs, so serialization succeeds and
      deserialization fails. Implement `visit_seq` in positional field
      order ([serde.rs/deserialize-struct](https://serde.rs/deserialize-struct.html)),
      state the guarantee in each spec, and add a non-self-describing
      round-trip test beside the JSON ones (a dev-dependency only, per S2).
- [x] **`RedactedDiagnostic`'s `Debug` exposes the detail**
      (2026-11-26: hand-written `Debug` over the redacted projection, new
      `Display` = `Diagnostic::redacted`; S16 and `spec/phi/index.md` updated;
      test `redacted_diagnostics_format_without_message_text`. The `pub` field
      and `Deref` stay as deliberate access to the full finding; `hl7-2`'s
      `redacted()` returns a `String`, so it has no such issue.) —
      `serde-hl7-v2/src/diagnostic.rs` has
      `#[derive(Debug)] pub struct RedactedDiagnostic(pub hl7_2::Diagnostic)`,
      so `{:?}` and `tracing`'s `?field` print the unredacted detail and
      segment name that `Serialize` withholds. The public tuple field leaks
      the same way. Implement `Debug` (and `Display`, if offered) from the same
      redacted projection `Serialize` uses; consider making the field private;
      check `hl7-2`'s own `Diagnostic::redacted()` return type for the
      same issue; test every logging path with synthetic markers. Update
      `spec/phi/index.md`.
- [x] **SOAP acknowledgement evidence is not bound to one element**
      (2026-11-26: both crates read code and ID from exactly one carrier inside
      `Body`; specs §6.1; test `evidence_comes_from_one_acknowledgement_in_the_body`
      in each crate.) —
      `hl7-2-soap/src/response.rs` `evidence()` takes `Status` and
      `MessageControlId` from two independent `root.find` calls, so a body
      with an `AA` in one element and the expected control ID in another
      (or in a header, or in an echoed payload) is accepted by
      `evaluate_correlated`. Read both from the same acknowledgement element
      inside the SOAP `Body`, and reject several conflicting acknowledgements.
      Check `hl7-3-soap` for the same pattern. Tests: split elements, an
      ID in the header only, two acknowledgements that disagree.
- [x] **`evaluate_correlated` returns remote fault text**
      (2026-11-26: fixed reasons in both crates; specs §6.1 and
      `spec/phi/index.md`; test `correlated_reasons_never_quote_the_peer` in
      each crate.) — it returns
      `evaluate`'s `Outcome::Rejected` unchanged, and that carries
      `faultcode: faultstring` from the peer, despite the doc promise that
      the reason "never quotes the message's own text." A fault string often
      echoes payload content (PHI). Return fixed reasons from the correlated
      API in both SOAP crates (keep the remote text available through an
      explicit, documented accessor if needed); test with a synthetic marker
      in `faultstring`; update the PHI inventory.
- [x] **Generated ACKs for v2.1–2.3 carry `MSH-9.3`**
      (2026-11-26: `Builder::message_type` writes MSH-9.3 only when the
      dictionary's MSH-9 type has a third component; `hl7-2` spec §7;
      tests `writes_the_structure_component_only_where_the_release_has_one`
      (`hl7-2`, fails with the fix reverted) and
      `a_framed_reply_is_valid_in_every_release` (`hl7-2-mllp`).) —
      `hl7-2/src/builder.rs` `message_type` always sets `MSH-9.3` from
      `structure_id`, but the message-structure component first appears in
      v2.3.1; the library's own validator flags every 2.1/2.2/2.3
      acknowledgement `hl7-2-mllp::ack` builds. Make the builder write
      only the components the release defines (spec §8 of `hl7-2`), and
      add tests that frame and re-validate an ACK for every release 2.1–2.9
      (the existing `answers_in_the_release_the_sender_spoke` checks only
      MSH-12).
- [x] **Repair the pre-PR checklist** in `CONTRIBUTING.md` (2026-11-25; CI's
      rustdoc loop left as is pending the disk-use check): replace the
      `cargo llvm-cov --fail-under-lines 95` recipe with `./bin/test-coverage`
      (spec and CI require 100% of distinct instrumented lines), and replace
      `cargo rustdoc --lib -- -W missing-docs` (exits 101 at a virtual
      manifest) with
      `RUSTDOCFLAGS="-W missing-docs -D warnings" cargo doc --locked --workspace --all-features --no-deps --lib`.
      Check CI runner disk use before retiring the per-crate cleanup loop.
- [x] **Close the checked-arithmetic gap** (done 2026-11-26: all 17 crates
      inherit `[workspace.lints]`; the four invariant `expect`s became
      `Result`s at the maintainer's choice — `Builder::new`/`with_dictionary`,
      `builder::acknowledge`, `Version::dictionary`, breaking, in the
      unpublished `hl7-2` 0.4.0; `Hl7Error::Dictionary` added; RCA-ENF-5 grep
      clean outside fuzz harnesses; coverage 10,479/10,479. Originally 2026-11-25: CI Clippy now runs
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

      **Progress 2026-11-26.** Root `[workspace.lints.clippy]` now holds the
      full RCA-ENF-1 set at `deny`. Opted in (`[lints] workspace = true`),
      clean on all targets: `hl7-3`, `hl7`, `serde-hl7`, `hl7-2-soap`,
      `hl7-3-soap`, `hl7-2-derive`, `hl7-3-derive`, `serde-hl7-v2`,
      `serde-hl7-v3`, and (same day, second pass) `hl7-2-from-xml-into-er7` and
      `hl7-2-from-xsd-into-json-dictionary` — the latter also gained a declared
      10,000 bound on segment field numbers (spec §3.3), since the field list
      was sized from the XSD's own number. Integration-test and bench roots carry one RCA-EXC-5 `allow`
      with a reason; examples were fixed instead (they teach callers). A
      temporary `x + 1` in `serde-hl7-v2` failed the CI/CONTRIBUTING/VS Code
      Clippy command (same command everywhere). Still to convert, each with an
      empty `[lints]` table and a comment until then (production findings
      from the 2026-11-26 survey): `hl7-2` (~95). `hl7-2` sixth pass, 2026-11-26:
      `typed`, `generic`, `structure`, `message`, `dictionary`, `version`,
      `builder` and `validate` are converted (date/time checks now read digits
      with checked arithmetic and split with `split_at_checked`/`rsplit_once`;
      sparse dictionary positions are bounded by `dictionary::MAX_POSITION`,
      spec §3.2; `Version` picks its bundled file by a constant-index `match`).
      Seventh pass: `json.rs` (suffix cursor, `decode_utf16` pairs, checked hex),
      `main.rs` (indent string instead of a depth count), and the test and bench
      roots are converted too; the two pretty-printers in the er7→XML and
      er7→JSON converters now pass an indent string as well, so they no longer
      need an arithmetic exception. **All that stands between `hl7-2` and
      `[lints] workspace = true` is four `expect`s on internal invariants (`Builder::new`/`with_dictionary`
      parsing their own header, `acknowledge`'s MSH header, and
      `Version::dictionary` parsing a bundled file) that no RCA exception
      covers.** Removing them needs a decision: make those APIs return
      `Result` (breaking, but `hl7-2` 0.4.0 is not yet published), or add a
      spec exception for compile-time-embedded data proven by a test.
      Counts include `#[cfg(test)]` modules in `src/`.
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
