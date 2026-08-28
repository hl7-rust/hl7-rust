# Maintainers and access continuity

This file is the roster, and the honest answer to the question a
procurement review asks about any software that will touch patient data:
*what happens if the person who can ship a fix is unavailable?*

It is deliberately not aspirational. Everything below describes the project
as it is on the day you read it in git history, not a structure the project
hopes to grow into.

## Roster

| Person | GitHub | Contact | Role | Since |
|---|---|---|---|---|
| Joel Parker Henderson | [@joelparkerhenderson](https://github.com/joelparkerhenderson) | <joel@joelparkerhenderson.com> | Maintainer (sole) | 2026-08-19 |

ORCID: <https://orcid.org/0009-0000-4681-282X>. The date is when this
workspace was assembled and first published; the crates it absorbed are
older, and their histories are still walkable under their own directories.

**The bus factor of this project is one.** There is exactly one person who
can accept a pull request, publish a release, or change a repository
setting. No second maintainer exists, no organisation stands behind the
project, and no legal entity is a party to it. The GitHub organisation
`hl7-rust` is an organisation in the GitHub sense only — it exists because
an organisation Pages site must be served from an org-owned repository, not
because there is a group behind it.

Everything else in this file follows from that sentence, and no wording
elsewhere in the repository should be read as softening it.

## Publishing identities and where they live

These are the credentials and configured identities that can put bytes in
front of a user. Naming them is the point: an inventory nobody has written
down is an inventory nobody can hand over.

| Identity | What it publishes | Held by | Recovery if the holder is unavailable |
|---|---|---|---|
| The GitHub organisation `hl7-rust` and its owner account | The repository, its issues, its settings | The maintainer's GitHub account, as sole owner | None. GitHub's account-recovery process is the only route, and it is between GitHub and the account holder. |
| A crates.io API token | All fourteen crates | The maintainer, on his own machine | The crates.io owner list is the recovery surface, and it is the maintainer's account. |
| An SSH key | Pushes to GitHub, and to the GitLab and Codeberg mirrors, which `origin` fans out to on one `git push` | The maintainer, on his own hardware | None; the key is not escrowed. A successor would use their own. |
| The same SSH key, via `make publish` | <https://hl7-rust.github.io> — the website, pushed by `git subtree split` into the `hl7-rust.github.io` repository, which deploys it | The maintainer | As above. Deliberately *not* a CI credential: a workflow doing this would need a token able to write another repository's workflow file, and GitHub refuses that. |

**The honest reading of that table:** every publishing identity terminates
at one person's GitHub account or one person's hardware. There is no
Trusted Publishing, no signing key escrow, and no second holder anywhere.
That is the residual risk, and it is stated rather than mitigated, because
no mitigation is available to a one-person project without a legal entity
behind it.

**Every release is tagged; commits and tags are signed from 2026-08-27
onward.** Since the fourth release (2026-08-26), each crate's release
commit carries an annotated `<crate>-v<version>` tag, backdated onto the
first three releases so the convention covers every version actually on
crates.io. A tag alone is not proof of anything by itself — anyone with
push access could retarget one — which is what signing adds.

This repository signs with SSH (`gpg.format ssh`, `commit.gpgsign` and
`tag.gpgsign` both `true`), keyed to a dedicated passphrase-protected
signing key created for exactly this — deliberately not the unattended
automation key already used to push, since a signature a script could
produce unattended would not mean anything. Local verification is wired
through `gpg.ssh.allowedSignersFile`, and both paths were exercised for
real, not merely configured: a commit and an annotated tag were each made,
verified (`Good "git" signature for joel@joelparkerhenderson.com`), and
discarded as a test before this paragraph was written.

**Commits and tags before 2026-08-27 are unsigned and stay that way** —
signing is not retroactive, and rewriting history to add it would cost more
than it is worth. Authorship for that period rests on the GitHub account
and the committer identity in the history, not on cryptography, and that
is said plainly because a reviewer will check.

The signing key is registered as a Signing Key on GitHub, GitLab, and
Codeberg, and all three show signed commits as verified via their own
APIs — not just locally.

## What is not here yet

Named rather than quietly omitted, because their absence is itself
information for an evaluation:

- **No committed response window for a security report.**
  [`SECURITY.md`](SECURITY.md) now exists and names the private channels,
  but it promises best effort rather than a deadline, because one person
  with no on-call rotation cannot meet a deadline. What it does give you is
  an escape hatch: no response in 14 days and you should publish.
- **No release automation.** Since 2026-08-26, `.github/workflows/ci.yml`
  runs the [`CONTRIBUTING.md`](CONTRIBUTING.md) gates — tests, lints,
  formatting, rustdoc, the MSRV floor — on every push and pull request. But
  no workflow publishes anything: a crates.io release is still a manual act
  from one laptop, by one person.
- **No release signing, no SBOM, no reproducible-build attestation.**

## Issue response expectation

**Issues are read within a week.** That is a best-effort target from one
person, not a contract, and it is the expectation the issue templates
(`.github/ISSUE_TEMPLATE/`) point at. A read is not a fix: triage says
what happens next, and [`CONTRIBUTING.md`](CONTRIBUTING.md) says which
kinds of report move fastest. Security reports do not go through issues at
all — [`SECURITY.md`](SECURITY.md) has the private channels, and its own
escalation path if nothing comes back within 14 days.

## If the maintainer is unavailable

There is no succession plan that a document can create. What exists
instead:

- **Nothing already published disappears.** Crates.io releases are
  immutable and cannot be unpublished, only yanked — which needs the owner
  anyway. A deployment already running is unaffected by maintainer
  availability, and these crates fetch nothing at run time, so nothing
  degrades on its own.
- **Nothing new ships.** No release, no fix, no dictionary coverage, no
  security patch.
- **The work is not lost.** The licence is five-way permissive-or-copyleft
  at your option, the history is public and mirrored on three hosts, every
  behavioral rule is written down in a numbered `spec/index.md`, and every
  claim the project makes about itself is in
  [`spec/conformance/index.md`](spec/conformance/index.md),
  [`spec/phi/index.md`](spec/phi/index.md), and
  [`spec/benchmark/index.md`](spec/benchmark/index.md). A fork is a
  complete and legitimate continuation, and the project's position is that
  it should be taken rather than waited on. The specs exist substantially
  so that a fork can be maintained by someone who never spoke to the
  author.

If you depend on this software in a clinical setting and that position is
not acceptable to you — it reasonably may not be — the mitigation is on
your side of the boundary: pin a version, keep a fork you can build, and
budget for maintaining it. That is a truthful answer, and more useful than
a continuity plan with nobody behind it.

## Adding a maintainer

The route is in [`GOVERNANCE.md`](GOVERNANCE.md), and it is deliberately
informal: sustained, reviewed contributions until the maintainer trusts
your judgement on changes you did not write, then a conversation you are
welcome to start. Dictionary coverage and answering other people's issues
are the two clearest paths — see [`CONTRIBUTING.md`](CONTRIBUTING.md).

When someone takes it, this file gains a row, `CODEOWNERS` gains their
identity on the areas they own, the table above gains a second holder
wherever the identity permits one, and `GOVERNANCE.md` gains a section on
how two people decide when they disagree. Those are the whole mechanism.
