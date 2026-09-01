[hl7-rust](../../README.md) → spec → Trusted Publishing

# Trusted Publishing

Trusted Publishing is a secure way to publish Rust crates from a CI/CD
platform without a manually managed API token. It uses OpenID Connect
(OIDC) to let crates.io verify that a workflow is running from the
repository it claims to be, then hands that workflow a short-lived token
scoped to one publish, instead of a long-lived credential sitting in
repository secrets indefinitely.

This project's current publishing credential is exactly the long-lived
kind Trusted Publishing exists to retire: a crates.io API token, held on
the maintainer's own machine, valid for all fourteen crates until revoked
— see [`MAINTAINERS.md`](../../MAINTAINERS.md)'s publishing-identities
table. Adopting Trusted Publishing means that token goes away in favor of
a workflow that proves its identity per run.

## The policy

Adopt Trusted Publishing once it is production-ready across every code
forge this project mirrors to (GitHub.com, GitLab.com, Codeberg.org) and
every destination it publishes to (crates.io today; npm for
`hl7-rust.github.io` if that ever needs it), not piecemeal per forge. This
project pushes to all three remotes on one `git push` and treats them as
equally canonical, so adopting Trusted Publishing for one forge alone
would still leave the API token this project is trying to retire
long-lived regardless — a partial win the policy declines in favor of
retiring the token once, for good.

## Where each forge stands

Re-checked 2026-09-01 (previously 2026-08-28), so this is against current
fact rather than an old assumption — and the re-check found the Codeberg
wait got no closer, not further along as the previous check's wording
might have implied:

| Forge | Status |
| --- | --- |
| GitHub Actions | Generally available on crates.io's side since July 2025. |
| GitLab CI/CD | Beta, and GitLab.com-only; self-hosted GitLab is not supported (confirmed against the Rust blog's 2026-01-21 crates.io development update). |
| Codeberg / Forgejo | No support on crates.io's side. Forgejo's own tracking issue for it (`codeberg.org/forgejo/forgejo/issues/9939`) was closed 2026-03-20 as "not actionable" — a Forgejo maintainer stated the trust relationship has to be established from crates.io's side first, and it hasn't been, so there is nothing here for Forgejo to build against yet, let alone for this project to adopt. |

GitHub Actions alone clearing the bar is what makes adopting Trusted
Publishing "for GitHub only" a live temptation and, per the policy above,
a declined one: this project mirrors equally to GitLab and Codeberg, and
a credential that still has to exist for two of three remotes is not
retired.

## Where this is tracked

[`MAINTAINERS.md`](../../MAINTAINERS.md)'s publishing-identities section
and [`RFC.md`](../../RFC.md) §8 both state the current wait against these
same facts. `tasks.md` carries the open item and is the place to check for
the latest status; revisit this spec when Codeberg support lands on
crates.io's side.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
