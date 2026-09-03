# News

Announcements, project status, where updates appear, and press contacts.

This is not the changelog. [`CHANGELOG.md`](CHANGELOG.md) records what
changed in the code, change by change; this file records what is worth
telling someone who is not reading diffs. The same material, written for
readers rather than for a repository, is at
<https://hl7-rust.github.io/news/>.

## Status at a glance

| | |
|---|---|
| First published | 2026-08-19 |
| Crates | 14 in this workspace, plus `er7` in its own repository |
| Maturity | `0.x`. New, and the API may still break in a minor bump. |
| Maintainers | One — [`MAINTAINERS.md`](MAINTAINERS.md) states the bus factor plainly |
| HL7® v2 coverage | Releases 2.1–2.9; 24 segments, 42 composite types, 4 structures, extensible in JSON — [`spec/conformance/index.md`](spec/conformance/index.md) |
| HL7 v3 | A foundation, not an implementation. No Clinical Document Architecture. |
| HL7® FHIR® standard | Not implemented |
| License | MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only |

## 2026-08-26 — What this project claims, and how to check it

Three documents published that answer what an evaluation actually asks,
rather than what a README usually says.

**A conformance statement with numbers in it.**
[`spec/conformance/index.md`](spec/conformance/index.md) lists the coverage
exactly — 24 segments, 42 composite data types, 4 message structures, every
one named — and states the principle that makes the number safe: *an
unmodelled segment, field, data type, structure, or release difference
costs you a name, never a value.* An unknown segment parses, reads
positionally, warns, and round-trips byte for byte.

**Benchmarks with their method.** [`BENCHMARKS.md`](BENCHMARKS.md)
publishes measured figures with confidence intervals, on a named machine
and toolchain, from benchmarks in the repository that anyone can run. Two
worth carrying away: parsing a small ADT costs about 3 µs, so parsing is
not your bottleneck; and reading two fields by path costs about 3.6 µs
against 1.44 ms to build the whole tree of the same message — nearly 400
times more, so use paths.

**A plain statement about patient data.**
[`spec/phi/index.md`](spec/phi/index.md): nothing is written to disk, sent
over a network, logged, counted, or cached, and each of those is a grep
away from being confirmed. The part that is not reassuring, and the reason
the document exists: error and diagnostic strings can quote a value from
the message, so logging a whole error can log a value from a clinical
record.

Also added: [`COMPARISONS.md`](COMPARISONS.md), `CONTRIBUTING.md`,
[`INSTALL.md`](INSTALL.md), [`MAINTAINERS.md`](MAINTAINERS.md),
[`AI_STATEMENT.md`](AI_STATEMENT.md), `CITATION.cff`, and this file.

## 2026-08-21 — Derive macros work through a renamed or re-exported crate

`#[hl7(crate = ...)]` lets `#[derive(FromHl7)]` and `#[derive(FromElement)]`
find `hl7-2` or `hl7-3` when it has been renamed in `Cargo.toml`, or is
reached through the `hl7` umbrella crate. The minimum supported Rust
version — current stable minus three releases — is now pinned in every
member's manifest.

## 2026-08-20 — A silent data-corruption fix in the conversion pairs

The forward and reverse v2.xml and JSON conversion crates disagreed about
naming, so a round trip could return a message that was not the one that
went in, without raising an error. This is the most serious defect the
project has had. The round trip is now a test rather than an assumption.
**If you are running a conversion crate published before 2026-08-20,
upgrade.**

## 2026-08-19 — First release of the workspace

Fourteen crates, previously separate repositories, assembled into one Cargo
workspace with their histories preserved; `hl7-3`, `hl7-3-derive`, and
`hl7-3-soap` added; every `hl7-v2*` crate renamed to `hl7-2*`.

## Where updates appear

| Channel | What arrives there |
|---|---|
| <https://hl7-rust.github.io/news/> | These announcements, written for readers |
| [crates.io](https://crates.io/crates/hl7) | Every release, per crate; the authoritative version list |
| [GitHub](https://github.com/hl7-rust/hl7-rust) | Commits, issues, and releases. *Watch → Releases only* is the low-volume subscription. |
| [`CHANGELOG.md`](CHANGELOG.md) | Change-by-change detail |

There is no mailing list and no social account. If one is added, it will be
announced here first.

## Funding

This project is free under all five of its licences whether or not anyone
pays anything, and stays that way. There is no paid tier and nothing that
unlocks with money.

**GitHub Sponsors is live:**
<https://github.com/sponsors/joelparkerhenderson>, one-off or recurring.
It funds maintainer time — mostly dictionary coverage and answering
issues — not a support contract or a response-time guarantee; see
[`CONTRIBUTING.md`](CONTRIBUTING.md)'s Money section for the full terms.

**An Open Collective is not set up yet.** Checked 2026-08-28: no
collective exists at any plausible slug. If one is created, it is
announced here, in [`CONTRIBUTING.md`](CONTRIBUTING.md), and in
[`.github/FUNDING.yml`](.github/FUNDING.yml) in the same change — not
before, since a funding link pointing at nothing real is worse than one
channel fewer.

## Press and media

**Contact:** Joel Parker Henderson, <joel@joelparkerhenderson.com>. Sole
maintainer, and the only person who can speak for the project. Please say
what you are writing and by when; a same-day reply is likely but not
promised, and [`MAINTAINERS.md`](MAINTAINERS.md) explains why nothing here
is promised.

**Available on request:** background on HL7 v2 and why a fifty-year-old
pipe-delimited standard is still the backbone of hospital integration;
commentary on memory safety in health IT; the reasoning behind any design
decision in the project, all of which is written down in the specs.

### Boilerplate, ready to quote

> HL7 for Rust is an open-source Cargo workspace of Rust crates that parse,
> navigate, validate, modify, and render Health Level Seven (HL7) messages.
> It covers HL7 v2 releases 2.1 through 2.9 in the ER7 pipe-delimited
> encoding, transports for MLLP over TCP and SOAP over HTTP, conversions
> between ER7 and both v2.xml and JSON, and a foundation for HL7 v3. It is
> multi-licensed under MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or
> GPL-3.0-only, at the user's option, and is maintained by Joel Parker
> Henderson. <https://hl7-rust.github.io>

### Facts a story might need, all checkable

- **Fourteen crates**, published on crates.io from 2026-08-19. The ER7
  encoding layer, `er7`, is a fifteenth in its own repository.
- **One runtime dependency.** `hl7-2` depends on `er7`, which depends on
  nothing.
- **No logging, telemetry, network access, or filesystem access** from
  message-handling library code — [`spec/phi/index.md`](spec/phi/index.md)
  names the greps that confirm it, and the one named exception: the
  XSD-to-dictionary generator's library reads the XSD schema files you
  point it at, because reading them is that crate's entire purpose.
- **Five licenses at the user's option**, chosen so that a proprietary
  vendor and a public-sector project can both adopt it without asking.
- **A minimum supported Rust version of current stable minus two**,
  chosen because hospital toolchains are approved on a cycle measured in
  quarters.
- **Benchmarks and their method are published**, including the project's
  own slowest operation.

### What this project will not say

Stated so nobody has to ask twice, and so no quote implies otherwise:

- **It is not certified or accredited by anyone.** HL7 International has
  not assessed it. No conformance testing body has assessed it. The
  conformance statement is a self-assessment, and says so in its own first
  lines.
- **It is not a medical device**, and it makes no clinical claim.
- **It has no production track record to cite.** It was first published in
  2026. Any story implying hospital deployments would be inventing them.
- **No benchmark comparison against another library exists**, so no
  "faster than X" claim can be sourced to this project.
- **No adoption or download figure will be offered as a success metric.**
  The numbers are public on crates.io and are small, as you would expect of
  a project this age.

### Trademark

HL7® and FHIR® are registered trademarks of HL7. We are requesting
permission to use it here. Use of the trademarks does not constitute
endorsement of this library by HL7.

That is the notice this project carries at the top of every README, and it
is the wording to quote. In plain terms for a story: this project
implements the published standards and is **independent of, not affiliated
with, and not endorsed by** HL7. Please carry that qualifier in any
coverage; a story implying otherwise would be wrong in a way that matters
to a standards body.

## Corrections

If something on this page — or anywhere in this repository — stops being
true, that is a defect and worth reporting the same way any other defect
is. Everything here is written so it can be checked rather than believed,
which only works if people check.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
