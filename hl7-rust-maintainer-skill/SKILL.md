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
- Every crate, and the workspace root, carries the **same license
  boilerplate byte-for-byte** (`LICENSE.md`), matching its `Cargo.toml`'s
  `license` field. Don't invent different text for a new crate.

## The rule that matters most: spec is source of truth

Each crate with normative behavior has a `spec/index.md` — the single
source of truth for what that crate does, numbered section by section.
**A code change that contradicts the spec is either a bug fix (fix the
code) or an unstated spec change (update the spec in the same commit).**
Never let the two drift.

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
cargo rustdoc -p <crate> --lib -- -W missing-docs   # per crate — root is a virtual manifest; loop over all 14 libs, per .github/workflows/ci.yml
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

All seven checks above (plus the site pair when relevant) are exactly
what `.github/workflows/ci.yml` runs — passing them locally is passing
CI, not a proxy for it.

## Conventions a reviewer will otherwise ask about

- **The forward and reverse conversion crates are coupled.** A change to
  a forward crate's naming rules (`hl7-2-from-er7-into-xml`,
  `-into-json`) silently breaks its reverse crate's assumptions
  (`hl7-2-from-xml-into-er7`, `-from-json-into-er7`) — run the round trip
  after touching either:
  `hl7-2-from-er7-into-xml in.hl7 | hl7-2-from-xml-into-er7 | diff - in.hl7`.
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

**Publishing the site**:

```sh
make publish   # the only thing the root Makefile does
```

## What CI actually gates

`.github/workflows/ci.yml`: `cargo fmt --check` / `clippy` / `test`
(one job), a full workspace `cargo check` pinned to the MSRV toolchain,
`bin/check-trademarks`, `bin/check-docs`, and the website's
`pnpm install --frozen-lockfile` → `check` → `build`.
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
