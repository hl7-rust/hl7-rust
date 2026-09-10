---
name: hl7-rust-maintainer-skill
description: Technical, implementation-level guidance for maintainers and contributors changing code, specs, or docs inside the hl7-rust workspace itself — repo layout, the spec-first rule, the exact pre-PR checklist, adding dictionary coverage, benchmarking a performance claim, fixing the website, and what CI actually gates. Use when the task modifies this repository's own crates, specs, docs, or website (not when the task is merely using the published crates to process an HL7® message — see hl7-skill for that).
---

# Maintaining hl7-rust

Implementation-level conventions for working *on* this workspace, as
opposed to *with* it. If the task is understanding or converting an HL7
message, that's [`hl7-skill`](../hl7-skill/SKILL.md) instead. Everything
here distills `AGENTS.md`, `CONTRIBUTING.md`, and the files under `spec/`
— those remain authoritative; this file is the fast path, not a
replacement.

## Repo layout

A Cargo workspace, one directory per crate, flat (no nesting). Most crate
directories are former standalone repositories merged in with
`git subtree`, so their commit history is still walkable under their own
directory.

```
Cargo.toml          [workspace] members, nothing else — see below
<crate>/Cargo.toml   each member's own manifest, otherwise unchanged from
                     when it was a separate repo
```

Seventeen members, grouped by role (the root `README.md` has the full
table and the dependency graph):

```
hl7                                    umbrella: hl7::v2 + hl7::v3
hl7-2, hl7-3                           the two parsers
hl7-2-derive, hl7-3-derive             proc-macro companions (behind a feature)
hl7-2-mllp, hl7-2-soap, hl7-3-soap     transports
hl7-2-from-er7-into-json, -into-xml,   the four ER7 conversions, forward and
hl7-2-from-json-into-er7, -xml-into-er7  reverse (each is a lib + a bin)
hl7-2-from-xsd-into-json-dictionary    dictionary tooling (lib + bin)
hl7-2-xml-lite-helper                  the XML reader hl7-3, both SOAP crates,
                                       and the XML-reading conversions share
serde-hl7, serde-hl7-v2, serde-hl7-v3  the Serde bridge: umbrella + one per standard
```

- **One `Cargo.lock`, at the workspace root.** Never add one inside a
  member.
- **Members depend on each other by relative path**
  (`hl7-2-mllp/Cargo.toml`'s `hl7-2 = { path = "../hl7-2" }`), exactly as
  they did as sibling repos — that's *why* the layout stays flat.
- **No shared `[workspace.dependencies]`**, and no `[workspace.package]`
  inheritance beyond `rust-version`, without discussion first — either
  would touch every member's manifest in one change.
- `rust-version` is the one deliberate exception: pinned once in the root
  `[workspace.package]` and inherited by every member as
  `rust-version.workspace = true`. MSRV is **current stable minus two
  releases** — see
  [`spec/rust-msrv-n-minus-2/index.md`](../spec/rust-msrv-n-minus-2/index.md).
  Raising it is a breaking change, never a patch.
- Every crate's `LICENSE.md` is **byte-for-byte identical to every other
  crate's**, matching its `Cargo.toml`'s `license` field. The workspace
  root's `LICENSE.md` is a longer superset (SPDX block, `LICENSES/`
  reference, per-file SPDX-marking section, trademark-scope section) —
  a contributor adding a new crate should copy an existing *crate's*
  `LICENSE.md`, not the root's.

## The rule that matters most: spec is source of truth

Each crate with normative behavior has a `spec/index.md` — the single
source of truth for what that crate does, numbered section by section.
Thirteen of the seventeen have one (`ls */spec/index.md`); the two
`*-derive` crates and the two umbrellas, `hl7` and `serde-hl7`, don't,
and each one's own `AGENTS.md` says why. **A code change that contradicts
the spec is either a bug fix (fix the code) or an unstated spec change
(update the spec in the same commit).** Never let the two drift.

Two layouts are sanctioned, and a new crate picks one:

- **One file**, `spec/index.md`, with numbered `##` sections — what the
  eleven `hl7-*` crates use.
- **Numbered section directories**, which `serde-hl7-v2` and
  `serde-hl7-v3` use, mirroring `serde-er7` in the sibling `er7-rust`
  workspace: `spec/index.md` holds a section table, an S-numbered rule
  index (`S1`…`S15`, citable from tests and commits), the
  which-goal-wins list, and the required checks; each section is its own
  `spec/NN-slug/index.md` (`01-purpose-and-scope` … `11-strict-mode`).
  §7.1 is a coverage table naming the test behind every rule, and
  `cargo test -p <crate>` **checks it**: `every_rule_has_a_coverage_row`
  fails on a rule with no row or a row with no rule, and
  `every_spec_section_is_indexed_and_present` fails on a section directory
  the index doesn't list (or vice versa). So in those crates a spec change
  is: edit `spec/NN-*/index.md`, then the rule index, then the §7.1 row,
  then the code and tests — in that order.

Workspace-wide claims — not one crate's alone — live under the root
[`spec/`](../spec/) instead: what "supports HL7 v2 2.1–2.9" means
(`spec/conformance/`), what happens to patient data (`spec/phi/`), how
benchmark figures are produced (`spec/benchmark/`), the MSRV policy
(`spec/rust-msrv-n-minus-2/`), and more narrowly-scoped process specs
alongside them. **A change spanning crates — a shared type, a widened
dictionary, a new way for message content to reach an error string, a
moved benchmark figure — updates the corresponding spec and every
affected crate's own `AGENTS.md` in the same change.**

## Before opening a pull request

```sh
cargo test                                    # unit and integration tests
cargo clippy --all-targets -- -D warnings     # lint-clean
cargo fmt --check                             # formatting
cargo rustdoc -p <crate> --lib -- -W missing-docs   # per crate — root is a virtual manifest; loop over all 17 libs, per .github/workflows/ci.yml
cargo +1.96 check --workspace --all-targets   # the MSRV floor (moves with the policy)
./bin/check-trademarks                        # HL7®/FHIR®/CDA® fair-use rules, T1–T3
./bin/check-docs                              # doc size budget + relative-link integrity
```

Touched `hl7-rust.github.io/`? Run its own two gates from inside that
directory too:

```sh
pnpm run check   # svelte-kit sync && svelte-check
pnpm run build   # vite build
```

The seven local checks above (plus the site pair when relevant) map to
five of `.github/workflows/ci.yml`'s six jobs: `checks` covers fmt,
clippy, test, and rustdoc in one job; `msrv`, `trademarks`, and `docs`
each match one local check; `site` matches the site pair. The sixth CI
job, `sbom`, generates a CycloneDX SBOM with `cargo-cyclonedx` — it isn't
something a contributor is expected to run before every PR, so there's no
local step for it above.

## Conventions a reviewer will otherwise ask about

- **The forward and reverse conversion crates are coupled.** A change to
  a forward crate's naming rules (`hl7-2-from-er7-into-xml`,
  `-into-json`) silently breaks its reverse crate's assumptions
  (`hl7-2-from-xml-into-er7`, `-from-json-into-er7`) — run the round trip
  after touching either:
  `hl7-2-from-er7-into-xml in.hl7 | hl7-2-from-xml-into-er7 | diff - in.hl7`.
- **Dependencies are per-crate and deliberately few.** `deny.toml`
  (run weekly by `security.yml`) is the audit trail, so read its comments
  before adding one anywhere. The three Serde bridge crates are the
  strictest case: `serde-hl7-v2` and `serde-hl7-v3` each have **exactly
  two runtime dependencies** — `serde` and the `hl7-2`/`hl7-3` crate it
  wraps, with `default-features = false` — and a manifest-reading test
  (`the_crate_has_exactly_two_runtime_dependencies`) fails on a third.
  `serde_json` is a **dev-dependency only**, for tests, doctests, and
  examples; no format crate is ever named in `src/`. Every impl there is
  hand-written against Serde's low-level traits — `macro_rules!` is fine,
  a `serde_derive`/proc-macro dependency is not. And no other crate in
  the workspace depends on `serde` at all
  ([`spec/phi/index.md`](../spec/phi/index.md), "No serialization
  framework") — keep it that way.
- **Two umbrellas, same shape.** `serde-hl7` is to `serde-hl7-v2`/`-v3`
  what `hl7` is to `hl7-2`/`hl7-3`: re-exports only (`serde_hl7::v2`,
  `serde_hl7::v3`, each behind a same-named feature, both on by default),
  no code of its own, no `spec/` directory. A behavior fix never lands in
  an umbrella; a new public item in a wrapped crate is visible through
  the umbrella with no change there.
- **PHI care.** Never paste real patient data into an issue, a commit, a
  test fixture, or a prompt to an AI tool — synthesize instead. See
  [`spec/phi/index.md`](../spec/phi/index.md) for what the libraries
  themselves do and don't do with message content, including where a
  value can escape into a log via error handling you write.
- **Trademark fair use.** Every markdown page, every crate root's
  rustdoc, and every publishable `Cargo.toml` description that uses a
  word mark — HL7®, FHIR® (as in the "HL7® FHIR® standard"), or CDA® —
  needs the ® immediately after that mark's *first* use on the page, plus
  the verbatim disclaimer somewhere on it. `bin/check-trademarks`
  enforces this; see
  [`spec/hl7-trademarks-fair-use/index.md`](../spec/hl7-trademarks-fair-use/index.md)
  for the exact rules (T1–T3) and current scope.

## Recipes

**Adding dictionary coverage** — the cheapest, most useful contribution:
edit one JSON file under `hl7-2/schemas/` and add a test. Add coverage
because a real message motivates it, not speculatively — a table filled
in from the standard with no message behind it is a table nobody can
check. Confirm it's actually a gap first:
[`spec/conformance/index.md`](../spec/conformance/index.md) states
precisely what each release does and doesn't claim; an unmodelled
difference shows up as a positional name instead of a typed one, never as
a rejected message.

**A performance change** needs a before-and-after from the crate's own
benchmarks, per [`spec/benchmark/index.md`](../spec/benchmark/index.md):

```sh
git stash && cargo bench -p hl7-2 -- --save-baseline before
git stash pop && cargo bench -p hl7-2 -- --baseline before
```

Correctness wins over speed every time — a faster parser that loses a
value, or stops round-tripping byte for byte, is not faster; it's broken.

**Fixing the website**: edit `hl7-rust.github.io/` in this workspace, not
the published `hl7-rust/hl7-rust.github.io` repo, which `make publish`
force-pushes over from here. Nothing on the site is normative — it
summarizes crate READMEs and specs, so a wording fix there is a docs fix,
and a behavior fix belongs against the crate instead.

**Publishing the site**: the root `Makefile` has two publish targets, both
`git subtree split`-ing `hl7-rust.github.io/` out and pushing it to the
`hl7-rust.github.io` repository — plus their `*-remote` helper targets
that add the needed git remote on a fresh clone.

```sh
make publish        # forced push — use for the first publish, or a history rewrite
make github-pages   # non-forced `git subtree push` — prefer this once history is established
```

**Cutting a crates.io release** follows the runbook in
[`spec/release-process/index.md`](../spec/release-process/index.md): the
semver rule (a `0.x` minor bump is the only one allowed to break, MSRV
raises included), the inter-crate version-requirement check a bump can
break, a `CHANGELOG.md` entry before anything publishes, `cargo package`
as a manifest sanity check, then tag and sign. That same spec states the
bounds under which an agentic tool may decide, on its own judgment, that a
release is warranted and execute it — not something to invoke casually
just because this recipe exists next to it. The first release of a
brand-new crate, a raised MSRV floor, and anything touching the license
or trademark posture stay the maintainer's call, never delegated.

## What CI actually gates

`.github/workflows/ci.yml`: `cargo fmt --check` / `clippy` / `test`
(one job), a full workspace `cargo check` pinned to the MSRV toolchain,
`bin/check-trademarks`, `bin/check-docs`, a CycloneDX SBOM generated per
crate (`sbom` job, uploaded as a workflow artifact — not attached to a
release), and the website's `pnpm install --frozen-lockfile` → `check` →
`build`.
`.github/workflows/security.yml` runs `cargo deny --all-features check`
against [`deny.toml`](../deny.toml) weekly and on demand — the dependency
tree is small and audited on purpose; see that file's own comments before
adding a dependency.

## Where the rest of the detail lives

- [`AGENTS.md`](../AGENTS.md) — the canonical version of the workspace
  conventions above; each crate has its own that adds crate-specific
  detail. Read the crate's own before working in it.
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) — the full contributor guide
  this file distills, including how to file a report and how to
  contribute without writing code.
- [`MAINTAINERS.md`](../MAINTAINERS.md) / [`GOVERNANCE.md`](../GOVERNANCE.md)
  — who maintains this, the bus factor, and who decides what.
- [`spec/`](../spec/) — every workspace-wide and per-crate normative
  claim, one directory per topic.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
