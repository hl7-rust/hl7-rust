[hl7-rust](../../README.md) → spec → Dependabot

# Dependabot

Two independent Dependabot features, both enabled.

## Security updates

`dependabot_security_updates` is enabled at the repository level (via
GitHub's API, since it is a repository setting rather than a file).
Vulnerability alerts were already on, which that setting requires as a
prerequisite. This is what opens a PR automatically when a dependency
already in use gets a new advisory.

## Scheduled update PRs

[`.github/dependabot.yml`](../../.github/dependabot.yml) requests routine,
non-security version bumps on a weekly schedule, one entry per package
ecosystem this repository actually has:

| Ecosystem | Directory | Covers |
| --- | --- | --- |
| `cargo` | `/` | All fourteen workspace members, which share one root `Cargo.lock` |
| `github-actions` | `/` | The actions pinned in `.github/workflows/*.yml` |
| `npm` | `/hl7-rust.github.io` | The website; GitHub's `npm` ecosystem handler reads `pnpm-lock.yaml` natively — there is no separate `pnpm` value |

Weekly, not daily: [`SECURITY.md`](../../SECURITY.md) and
[`MAINTAINERS.md`](../../MAINTAINERS.md) are both explicit that one person
reads issues on a best-effort, roughly weekly cadence with no committed
response window. A daily PR cadence would just accumulate unread against
that same person; weekly matches the pace that already governs everything
else here.

## The `github-actions` `ignore:` rule

The `github-actions` entry ignores `dtolnay/rust-toolchain`. That pin is
not a routine action version — it is the MSRV floor
[`spec/rust-msrv-n-minus-2/index.md`](../rust-msrv-n-minus-2/index.md)
states, currently `1.96`, referenced in the `msrv` CI job. Dependabot
cannot tell that pin apart from an ordinary action version like
`actions/checkout@4`, so left unignored it proposes bumping the toolchain
forward the same way it would bump any other action — and CI's `msrv` job
correctly fails that PR by design, since the toolchain a routine bump
proposes is not one the MSRV policy chose. That policy's own Maintenance
section spells out raising the MSRV as a deliberate, multi-step change a
maintainer makes by hand (bump `rust-version`, bump the CI pin, verify with
the older toolchain) — not something an automated PR should ever do, so
the pin is excluded rather than left to fail weekly. The `dtolnay/rust-toolchain@stable` pin used elsewhere is a
channel name, not a version, so it was never at risk the same way and
needs no `ignore:` entry.

## Status

Both halves are live, confirmed by real PRs rather than by configuration
alone: the first scheduled run opened ten PRs across all three ecosystems,
nine passed CI clean, and the tenth was the `dtolnay/rust-toolchain` bump
described above, caught by the `msrv` job before the `ignore:` rule was
added.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
