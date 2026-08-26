# Security policy

How to report a vulnerability in HL7® for Rust, what counts as one, and
what you can honestly expect back.

Read the last part first if you are performing a supplier review: this is a
single-maintainer project, and this document says so rather than implying a
response capacity that does not exist.
[`MAINTAINERS.md`](MAINTAINERS.md) has the full picture.

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Two private channels, either is fine:

1. **GitHub private vulnerability reporting** — the "Report a
   vulnerability" button under the Security tab of
   <https://github.com/hl7-rust/hl7-rust>. Preferred, because it keeps the
   report, the discussion, and the eventual advisory in one place.
2. **Email** — <joel@joelparkerhenderson.com>. Put "security" in the
   subject line.

**Never include real patient data in a report**, including in a
reproducing message. Redact the values and keep the structure; structure is
what reproduces a parsing bug, and names and identifiers are not.
[`CONTRIBUTING.md`](CONTRIBUTING.md) shows the redaction, and
[`spec/phi/index.md`](spec/phi/index.md) is the project's full position. A
report that arrives with real data in it becomes an incident of its own.

Useful report contents: the crate and version, a redacted input that
triggers it, what happens, and what you expected. If you have a view on
severity, say so, but do not spend effort on a CVSS score — the discussion
will reach the same place faster.

## What you can expect

Stated as commitments only where a commitment is real.

| | |
|---|---|
| Acknowledgement | Best effort, usually within a few days. **No committed window.** |
| Fix | Best effort, prioritised over everything else in the project. |
| Credit | You are named in the advisory and the changelog unless you ask not to be. |
| Embargo | Held until a fixed version is published, or 90 days from the report, whichever comes first. |

**There is no service-level agreement, and there cannot be one.** One
person maintains this project, there is no on-call rotation, and no
organisation stands behind it. Inventing a 48-hour response promise that
nobody is staffed to meet would be worse than this paragraph.

**If you get no response within 14 days**, escalate: send a second message
saying you intend to publish, then publish on your own timetable. You do
not need permission, and a maintainer who has gone quiet is not a reason
for a real vulnerability to stay unreported. That escalation path is
deliberate — it means a report is never trapped by one person's
availability.

## Supported versions

**Only the newest release of each crate is supported.** A fix ships as a
new patch or minor release; there are no backports and no long-term support
branches. Every crate is `0.x`, and [`CHANGELOG.md`](CHANGELOG.md) records
what changed.

If you are pinned to an older version, the upgrade path is the fix. If that
is not viable for you, the licence lets you patch your own copy, and
[`MAINTAINERS.md`](MAINTAINERS.md) recommends keeping a fork you can build
for exactly this reason.

## In scope

These are libraries that parse untrusted input — messages arriving from
another organisation's system — so the interesting failures are input
handling:

- **A panic, abort, or unbounded resource use on malformed input.** A
  parser that crashes on a hostile message is a denial of service against
  whatever interface embeds it. This is the most likely real finding here,
  and the one most worth your time.
- **Stack exhaustion from deeply nested input.** The dictionary reader
  bounds nesting at 256 levels for exactly this reason; a way around that
  bound is a vulnerability.
- **Memory unsafety of any kind**, though see below on `unsafe`.
- **Incorrect parsing that crosses a trust boundary** — input that makes a
  message read as a *different, valid* message. In a clinical context,
  silently returning the wrong patient's value is more serious than
  crashing, and it is in scope even though it looks like a correctness bug.
- **Escaping a boundary the documentation claims**: the library reaching
  the filesystem, the network, or an environment variable, contrary to
  [`spec/phi/index.md`](spec/phi/index.md).
- **A dictionary or schema file that causes any of the above** when loaded.

## Not a vulnerability

Named explicitly so that a scanner report or a review does not spend time
here, and so the boundary is honest rather than convenient:

- **Error and diagnostic strings can quote a value from the message.**
  `Error::BadValue` carries the offending text verbatim, and a
  `ValueFormat` diagnostic formats the value into its detail. This is
  documented, deliberate, and the reason
  [`spec/phi/index.md`](spec/phi/index.md) exists. Logging a whole error
  can put patient data in a log — that is a property of your logging, and
  the document shows how to avoid it. It becomes a vulnerability only if a
  value appears somewhere the documentation says it will not.
- **MLLP provides no confidentiality, integrity, or authentication.** That
  is MLLP as specified. `hl7-2-mllp` frames bytes on a stream you supply;
  if the stream should be TLS, you supply the TLS stream.
- **An unmodelled segment, field, or structure reading positionally.** That
  is the documented behaviour: coverage gaps cost a name, never a value.
  See [`spec/conformance/index.md`](spec/conformance/index.md).
- **Memory holding message text is not zeroed on drop.** Documented, and
  not currently defended against.
- **No encryption, no access control, no audit trail, no
  de-identification.** These are parsing libraries; all four belong to the
  system around them.
- **A missing HL7® feature.** File it as an issue.

## Security properties you can rely on

Each one is checkable in a few minutes, which is the point:

- **No `unsafe` code, enforced by the compiler.** Every crate root —
  fourteen libraries and six binaries — carries
  `#![forbid(unsafe_code)]`. `forbid` outranks any `allow` further down, so
  a crate here cannot regain the ability to write `unsafe` without someone
  deleting that line in a diff you can see.
- **No network access, no filesystem access, no environment variables, no
  subprocesses** from library code. `std::net`, `std::fs`, `std::env`, and
  `std::process` appear only in the command-line binaries.
- **No logging, telemetry, or analytics**, and no dependency that could
  provide any.
- **One runtime dependency.** `hl7-2` depends on `er7`, which depends on
  nothing. `cargo tree` is short enough to read.
- **Bounded dictionary nesting**, at 256 levels, so a malformed dictionary
  is an error rather than a stack overflow.
- **Fuzzing** on the parsing surfaces that take untrusted structured input:
  five targets across `hl7-2-xml-lite-helper`, `hl7-2-from-xml-into-er7`,
  and `hl7-2-from-json-into-er7`.

## Known gaps

A security policy that lists no gaps is a security policy nobody checked.

- **Fuzzing covers 3 of 14 crates.** The ER7 parsing surface in `er7`
  itself, and `hl7-2`'s dictionary reader, are not fuzzed here.
- **CI is new, and narrow.** Since 2026-08-26,
  `.github/workflows/ci.yml` runs the [`CONTRIBUTING.md`](CONTRIBUTING.md)
  gates — tests, clippy, formatting, the rustdoc pass, and the MSRV floor —
  on every push and pull request. It runs no dependency audit and gates no
  release, and before that date every check depended on one person's
  laptop.
- **No release signing, no SBOM, no reproducible-build attestation.**
  Commits and tags are not signed either.
- **No dependency-audit automation.** With one dependency this is a small
  surface, but `cargo audit` is not run on a schedule.
- **No published threat model.**

If one of these is what blocks your adoption, say so —
[`RFC.md`](RFC.md) §8 asks exactly that question, because which gap matters
most is currently a guess.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
