# Contributing

Thanks for looking. There is more than one way to help here, and code is
only one of them.

| You have | Start with |
|---|---|
| Ten minutes and a real HL7® feed | [Tell us what your messages contain](#tell-us-what-your-messages-contain) — the single most useful thing anyone can do |
| An opinion about the design | [`RFC.md`](RFC.md), which lists what this project is actually unsure about |
| A bug | [What a good report contains](#what-a-good-report-contains) — and read [Never paste patient data](#never-paste-patient-data) first |
| Time and Rust | [Before you open a pull request](#before-you-open-a-pull-request) |
| Time and no Rust | [Ways to help that are not code](#ways-to-help-that-are-not-code) |
| Money | [Money](#money) |

This file is the short version — enough to help without reading anything
else. The long version is
[Support and contributing](https://hl7-rust.github.io/help/support/) on the
website, and the conventions for changing a particular crate are in that
crate's own `AGENTS.md`.

## Never paste patient data

HL7® messages are clinical records, and an issue tracker is public and
permanent. A message pasted into one cannot be unpasted.

Redact the values, keep the structure:

```
MSH|^~\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5
PID|1||REDACTED^^^ACME&1.2.3.4&ISO^MR||REDACTED^REDACTED||REDACTED|F
OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F
```

Structure is what reproduces a parsing bug: the delimiters, the field
positions, the repetition separators, the component depth. Names,
identifiers, dates of birth, and addresses are not. Replace them and the
report still works. If a bug genuinely depends on a specific byte sequence
in a value — an unusual escape, a non-ASCII character set — describe the
byte sequence rather than the record it appeared in.

The project's full position on this, including what these crates do and do
not do with the data you hand them, is [`spec/phi/index.md`](spec/phi/index.md).

## Where to file

- **Issues and pull requests**: <https://github.com/hl7-rust/hl7-rust>.
- **Security problems**: not a public issue. Use the private channels in
  [`SECURITY.md`](SECURITY.md).
- **ER7 encoding bugs** — delimiters, escapes, path syntax, byte-for-byte
  rendering — belong to <https://github.com/er7-rust/er7-rust>, which is a
  separate project this workspace depends on.
- **A crate's archived former repository** is not the place. Those still
  carry the old `hl7-v2*` names and are kept only so their history stays
  reachable.
- GitLab and Codeberg carry mirrors; issues live on GitHub.

## What a good report contains

1. **The crate and its version** — `cargo tree -p hl7-2` if you are not
   sure what resolved.
2. **A minimal redacted message that reproduces it.** One segment is often
   enough.
3. **What you expected and what you got.** `hl7-v2 --tree --paths` on the message
   is usually the clearest way to show both.
4. **The spec section, if you can find it.** Each crate's `spec/index.md`
   is numbered so it can be cited. "§4.2 says positional names are used
   here, but the output has …" turns a discussion into a fix.
5. **Your Rust version**, if it is a build failure.

The spec is the referee. If a crate's README, its rustdoc, or the website
disagrees with its `spec/index.md`, the spec is right and the other three
are the bug.

## Before you open a pull request

```sh
cargo test                                # unit and integration tests
cargo clippy --all-targets -- -D warnings # lint-clean
cargo fmt --check                         # formatting
cargo rustdoc --lib -- -W missing-docs    # every public item documented
cargo +1.95 check --workspace --all-targets   # the MSRV floor
```

The MSRV floor is current stable minus three releases, so the exact
toolchain in that last line moves; the rule is in
[`spec/rust-msrv-n-minus-3/index.md`](spec/rust-msrv-n-minus-3/index.md).

And the conventions that a reviewer will otherwise ask about:

- **Behavior changes go in the spec first.** The spec is the source of
  truth, so a code change that contradicts it is either a bug fix or an
  unstated spec change.
- **One `Cargo.lock`, at the workspace root.** Never one inside a member.
- **No `[workspace.package]` inheritance or shared
  `[workspace.dependencies]`** without discussion — either would touch
  every member's manifest at once.
- **The license boilerplate is byte-for-byte identical** in every crate.
  Don't invent different text for a new one.
- **A change spanning crates updates every affected crate's `AGENTS.md`
  and `spec/index.md`** in the same change.
- **The forward and reverse conversion crates are coupled.** A change to a
  forward crate's naming rules silently breaks its reverse crate's
  assumptions; run the round trip after touching either.
- **Raising the MSRV is a breaking change** and belongs in a release
  allowed to break, never in a patch.

## Adding dictionary coverage

The most useful code contribution, and the cheapest: filling a gap means
editing one JSON file under `hl7-2/schemas/` and adding a test.

Add coverage when a real message motivates it, rather than speculatively —
a table filled in from the standard with no message behind it is a table
nobody can check. If you have the message but not the Rust, stop here and
see [Tell us what your messages contain](#tell-us-what-your-messages-contain)
instead; the report is the hard part, and the JSON is not.

Before filing, check whether it is a gap at all. What each release does and
does not claim is stated precisely in
[`spec/conformance/index.md`](spec/conformance/index.md); an unmodelled
difference shows up as a positional name instead of a typed one, or as a
missing warning, never as a rejected message or a lost value.

## Performance changes

A change argued on performance needs a before-and-after from the benchmarks
in the same crate, produced the way
[`spec/benchmark/index.md`](spec/benchmark/index.md) describes:

```sh
git stash && cargo bench -p hl7-2 -- --save-baseline before
git stash pop && cargo bench -p hl7-2 -- --baseline before
```

Correctness wins over speed here every time. A faster parser that loses a
value, or that stops round-tripping byte for byte, is not faster; it is
broken.

## Fixing the website

The site is the `hl7-rust.github.io/` directory of this workspace — edit it
there, not in the published repository, which `make publish` force-pushes
over. Nothing on the site is normative: it summarizes the crates' READMEs
and specs. A correction there is a documentation fix; a correction to the
behavior underneath belongs against the crate.

## Tell us what your messages contain

The bound on this project is not effort, it is knowledge: it models 24
segments, 42 composite data types, and 4 message structures, and the gap
between that and your feed is invisible from here. Nobody can guess which
vendor sends which segment.

Ten minutes, no Rust, on a redacted sample:

```sh
hl7-v2 --check redacted-sample.hl7      # counts what the dictionary misses
hl7-v2 --tree --paths redacted-sample.hl7   # shows exactly which fields read positionally
```

Open an issue with the counts and a redacted example. `SegmentUnknown` and
`StructureUnknown` warnings are literally a to-do list, and each one closed
helps every other site sending the same shape. If you cannot share even a
redacted message, the segment and structure *names* on their own are still
worth having.

## Ways to help that are not code

- **Answer a question** on an issue. Someone who has debugged an interface
  before knows things the maintainer does not.
- **Fix the documentation.** If a guide misled you, that is a defect; the
  fix is usually two sentences, and you are the only person who will ever
  notice it.
- **Try the command-line tools** and say where they were awkward. They are
  meant to be usable by an integration analyst who writes no Rust, and that
  claim needs testing by such a person.
- **Review the claims.** Everything in
  [`spec/conformance/index.md`](spec/conformance/index.md),
  [`spec/phi/index.md`](spec/phi/index.md), and
  [`spec/benchmark/index.md`](spec/benchmark/index.md) is written to be
  checked rather than believed. Finding one that does not survive checking
  is a real contribution.
- **Run the benchmarks on your hardware** and post the numbers. One machine
  is one data point.
- **Tell us you are using it.** Even privately. It changes what gets
  prioritised, and right now the maintainer is guessing.
- **Answer the open questions** in [`RFC.md`](RFC.md).

## Money

This project is free, and it stays free under all five of its licences
whether or not anyone pays anything. There is no paid tier, no sponsor-only
feature, and no feature that unlocks with money.

If you want to fund the time anyway:
**<https://github.com/sponsors/joelparkerhenderson>**, one-off or recurring.
An Open Collective is not set up yet — this section will carry the link
the day it is, rather than one pointing at nothing.

What sponsorship buys, honestly: maintainer time, which mostly goes to
dictionary coverage and answering issues. What it does not buy: a support
contract, a response-time guarantee, a roadmap commitment, or influence
over what gets merged. [`MAINTAINERS.md`](MAINTAINERS.md) is candid that
the bus factor is one, and money does not change that number — if your
organisation depends on this, the mitigations there (pin a version, keep a
fork you can build) matter more than a donation does.

Sponsorship is never a condition of having a bug fixed. A report from
someone who has paid nothing is treated exactly like one from someone who
has.

## Licensing your contribution

Everything here is offered under MIT, Apache-2.0, BSD-3-Clause,
GPL-2.0-only, or GPL-3.0-only, at the user's option. A contribution is
offered on the same terms, so that the choice stays available to everyone
downstream. There is no CLA.

## Conduct

Be decent. Assume the person on the other end is working on a live clinical
interface and is short of time.

The full code is [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) — Contributor
Covenant 2.1, plus one addition this project takes as seriously as
harassment: do not overstate what the software does. Report conduct
problems privately to <joel@joelparkerhenderson.com>; that file is honest
about what a single-maintainer project can and cannot offer if the report
concerns the maintainer.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
