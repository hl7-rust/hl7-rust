[hl7-rust](../../README.md) → spec → Release process and AI publishing authority

# Release process and AI publishing authority

Two things this document formalizes together, because the second only
makes sense against the first: **how a crates.io release is cut** (never
written down before this — practice existed only as a trail of `tasks.md`
log entries and `CHANGELOG.md`'s own scattered rules), and **when an
agentic tool may execute `cargo publish` on its own judgment**, adopted
2026-09-02 at the maintainer's direction.

## The release runbook

Every release, whoever executes it, follows the same steps. None of this
is new; it is what the four releases through 2026-08-29 actually did,
written down for the first time rather than reconstructed from git history
each time.

1. **Decide the version bump per crate.** [`CHANGELOG.md`](../../CHANGELOG.md)'s
   own rule: while a crate is `0.x`, a **minor** bump is the one allowed to
   break — including a raised MSRV, which is always breaking
   ([`spec/rust-msrv-n-minus-2/index.md`](../rust-msrv-n-minus-2/index.md)).
   A patch bump never carries a breaking change; if it would, it is a minor
   bump instead, full stop.
2. **Check every inter-crate version requirement a bump could break.** A
   `0.x` minor bump fails Cargo's caret-compatibility rule for any sibling
   crate's `path = ".."` dependency that named the old version — the fifth
   release needed twelve such requirements raised alongside the two crates
   whose own version changed. Verify with `cargo metadata` (or `cargo
   check --workspace`, which fails loudly on an unsatisfied requirement)
   before publishing, not after.
3. **Write the `CHANGELOG.md` entry first.** One dated heading, `### Added`
   / `### Changed` / `### Fixed` / `### Removed` as needed, each bullet
   naming what changed and why, per the format this file already follows.
   A release with no changelog entry is not a release this project
   recognizes as done, regardless of what crates.io shows.
4. **Run the full pre-PR gate set** —
   [`hl7-rust-maintainer-skill/SKILL.md`](../../hl7-rust-maintainer-skill/SKILL.md)'s
   "Before opening a pull request" list, all seven checks — against the
   exact commit being released, and confirm it is already pushed and green
   on CI (`.github/workflows/ci.yml` and `security.yml`), not merely green
   on a laptop. A release is never cut against unpushed or unverified work.
5. **`cargo publish` each affected crate**, in dependency order (a
   dependent cannot publish against a version its dependency has not
   published yet).
6. **`cargo package` each published crate and confirm the manifest carries
   the literal values it should** — a `rust-version` that reads
   `"1.96"`, not `rust-version.workspace = true`, is the check the fifth
   release's own verification used; the same principle applies to any
   value a workspace-level mechanism could otherwise leave unresolved in
   the published `Cargo.toml`.
7. **Tag and sign.** One annotated `<crate>-v<version>` tag per released
   crate, SSH-signed per [`MAINTAINERS.md`](../../MAINTAINERS.md)'s signing
   section, pushed to all three remotes.
8. **Record it.** A `tasks.md` entry or a `CHANGELOG.md` entry alone is not
   sufficient on its own for a release this document governs — both,
   because `CHANGELOG.md` is what a downstream `Cargo.lock` reader consults
   and `tasks.md` is what an evaluator of *this process* consults, and they
   answer different questions.

## Who may decide to release

Two paths, both bound by the runbook above and the preconditions below —
this section is about **who decides a release should happen**, not about
who is permitted to type the command once that decision is made.

- **The maintainer**, directing the action explicitly — unchanged, and
  the default for anything this section's next path does not cover.
- **An agentic AI tool** (Claude Code, per
  [`AI_STATEMENT.md`](../../AI_STATEMENT.md) §5), deciding on its own
  judgment that a specific, already-landed change warrants a release,
  **without the maintainer naming that release as a specific instruction
  first** — adopted 2026-09-02. This is the one exception
  [`AI_STATEMENT.md`](../../AI_STATEMENT.md) §6 otherwise states plainly:
  the decision of *what ships in a release* had been the maintainer's
  alone; it can now also be an agentic tool's, strictly within the scope
  below.

Concretely: an agent working in this repository may work through the
runbook's steps 1–4 above — the version bump, the inter-crate
requirement check, the `CHANGELOG.md` entry, the pre-PR gate set — decide
for itself that the release meets every one of them, and then carry out
step 5, `cargo publish`, itself. The maintainer no longer has to tick
every box personally before that command runs; the bounds in this
document are what stand in his place. Steps 6–8 (the manifest check, tag
and sign, and the record) still follow in the same action, per the
preconditions below — reaching step 5 is not the same as being done.

## Scope: what "on its own judgment" does and does not mean

- **Only inside a live, interactive session** — a terminal the maintainer
  started and is actively working in, on his own machine, the same
  environment [`MAINTAINERS.md`](../../MAINTAINERS.md) already documents
  as where the SSH key that signs commits and pushes lives. **Not** a
  scheduled, cron-triggered, or unattended `/loop`-style session with no
  maintainer present to see the action happen in real time. That is a
  distinct, larger step — "release automation" — which `MAINTAINERS.md`'s
  gap list and `tasks.md`'s conscious-acceptance entry both still list as
  open, unaffected by this policy.
- **No new credential.** The tool executes `cargo publish` using the
  crates.io API token already configured on the maintainer's own machine —
  the same one [`MAINTAINERS.md`](../../MAINTAINERS.md)'s
  publishing-identities table has always listed as "held by the
  maintainer, on his own machine." No separate crates.io account, owner
  invite, or token is issued to any tool. This is the same pattern already
  established for `git push`: the tool acts as the maintainer's hands on
  hardware and credentials that remain solely his.
- **Every precondition below still gates the action.** "On its own
  judgment" widens *who decides a release is warranted*; it does not
  relax *what has to be true before `cargo publish` runs*.

## Preconditions, regardless of who decided

`cargo publish` — whether the maintainer typed it or an agentic tool
executing under the scope above did — never runs unless all of these hold:

- The runbook's steps 1–4 above are complete: the version bump is
  semver-correct, every inter-crate requirement is verified satisfied, the
  `CHANGELOG.md` entry exists, and the commit being released is pushed and
  green on CI.
- The change is a real, already-landed fix, feature, or correction — never
  speculative ("might as well publish while we're here"). A release
  without a concrete reason a `CHANGELOG.md` bullet can name plainly is
  not warranted, whoever decides it.
- Steps 6–8 (the manifest sanity check, tagging and signing, and the
  `tasks.md`/`CHANGELOG.md` record) complete in the same action — a
  publish that skips the record it is supposed to leave is treated as
  unfinished, not merely under-documented.

## What stays exclusively the maintainer's call

Never delegated, regardless of session or judgment:

- **The first release of a brand-new crate.** Every release this policy
  covers is to a crate already on crates.io; adding a fifteenth crate is
  itself outside `plan.md`'s current non-goals in the first place.
- **A raised MSRV floor.** Already stated as "a deliberate, spec-driven
  change, never an automated PR" in
  [`spec/rust-msrv-n-minus-2/index.md`](../rust-msrv-n-minus-2/index.md);
  unaffected by this policy, restated here so the two documents cannot be
  read as disagreeing.
- **Anything touching the five-way license, the trademark posture, or a
  PHI-handling code path** ([`spec/phi/index.md`](../phi/index.md)) —
  these carry legal and clinical-safety weight beyond what a green CI run
  certifies, and need the maintainer's explicit review of that specific
  change before any release carries it.
- **Yanking a published version.** A separate, more consequential action
  than publishing a new one; not authorized by this document at all.

## Accountability does not change

[`AI_STATEMENT.md`](../../AI_STATEMENT.md) §4 is unaffected by this
policy: the maintainer is the sole named, accountable author of every
release, whatever tool typed the command. The crates.io owner account,
the git history's `Author:` field, and legal responsibility for what ships
all still terminate at the one person
[`MAINTAINERS.md`](../../MAINTAINERS.md) names. A tool executing a
publish under this policy is not a co-owner, not a signer, and not a
second accountable party — the same principle that already governs every
commit an agentic tool makes in this workspace, extended to this one
additional command rather than carved out as an exception to it.

## Where this is tracked

[`AI_STATEMENT.md`](../../AI_STATEMENT.md) §5, §6, and Annex A record the
policy change itself (version 2.0.0). `MAINTAINERS.md`'s
publishing-identities table and bus-factor sentence, `GOVERNANCE.md`'s
"who decides" section, and `tasks.md` all carry a pointer here rather than
restating the scope independently — this document is the source of truth
for what "on its own judgment" is bounded by, the same relationship every
other workspace-level policy spec has to the root documents that mention
it in passing.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
