# Contributing

Thanks for looking. This file is the short version — enough to file a good
report or land a small change without reading anything else. The long
version is [Support and contributing](https://hl7-rust.github.io/help/support/)
on the website, and the conventions for changing a particular crate are in
that crate's own `AGENTS.md`.

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

## The most useful contribution

Dictionary coverage. Filling a gap means editing one JSON file under
`hl7-2/schemas/` and adding a test, and it is the thing most likely to help
somebody else's interface.

Add coverage when a real message motivates it, rather than speculatively —
a table filled in from the standard with no message behind it is a table
nobody can check.

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

## Licensing your contribution

Everything here is offered under MIT, Apache-2.0, BSD-3-Clause,
GPL-2.0-only, or GPL-3.0-only, at the user's option. A contribution is
offered on the same terms, so that the choice stays available to everyone
downstream. There is no CLA.

## Conduct

Be decent. Assume the person on the other end is working on a live clinical
interface and is short of time.

Contact: <joel@joelparkerhenderson.com>.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
