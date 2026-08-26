# Changelog

Notable changes to this workspace, newest first.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
loosely, with one deliberate departure: **this workspace has no single
version number.** Fourteen crates version independently on crates.io, so
each entry below is dated and lists the crate versions that carried it.
`cargo` resolves per crate; a date here is what a person needs to line up
"the release where X changed" with what is in their `Cargo.lock`.

Every crate follows [Semantic Versioning](https://semver.org/). While a
crate is `0.x`, a minor bump is the one allowed to break — including a
raised minimum supported Rust version, which is always a breaking change
and never lands in a patch
([`spec/rust-msrv-n-minus-3/index.md`](spec/rust-msrv-n-minus-3/index.md)).

## 2026-08-26

Documentation and policy only — no crate behavior changed. Released
anyway, and only for one reason: the trademark notice belongs on the
crates.io page of every crate, and a README reaches crates.io only through
a release. Every crate took a patch bump; no dependency requirement needed
editing, because each is a caret range a patch satisfies.

### Added

- **A conformance statement**
  ([`spec/conformance/index.md`](spec/conformance/index.md)): what
  "supports HL7® v2 releases 2.1 through 2.9" means, with the 24 segments,
  42 composite data types, and 4 message structures listed by name, what
  happens to input outside that set, and the out-of-scope list.
- **A PHI statement** ([`spec/phi/index.md`](spec/phi/index.md)): what
  these crates do with patient data, what they never do, and where a value
  can escape — `Error::BadValue` carries the offending text verbatim, and a
  `ValueFormat` diagnostic formats the value into its detail string.
- **Benchmarks for `hl7-2` itself** (`hl7-2/benches/parse.rs`): five
  Criterion groups covering parse, path read, tree, validate, and render.
  Previously only the conversion crates had benchmarks. Method and measured
  figures in [`spec/benchmark/index.md`](spec/benchmark/index.md) and
  [`BENCHMARKS.md`](BENCHMARKS.md).
- **`CONTRIBUTING.md`**, leading with redaction and never pasting patient
  data.
- **[`COMPARISONS.md`](COMPARISONS.md)**: interface engines, HAPI, the
  other Rust crates, and hand-rolled pipe splitting — including four cases
  where you should choose something else.
- **[`INSTALL.md`](INSTALL.md)**, **[`NEWS.md`](NEWS.md)**,
  **[`MAINTAINERS.md`](MAINTAINERS.md)**,
  **[`AI_STATEMENT.md`](AI_STATEMENT.md)**, **`CITATION.cff`** (with ORCID),
  **`CODEOWNERS`**, and this file.
- **Website**: `/docs/conformance/`, `/docs/comparison/`,
  `/docs/benchmarks/`, `/docs/patient-data/`, and a `/news/` section.
- **`help/outreach/index.md`**: research on reaching HL7 and Rust
  professionals, including the trademark question that gates promotional
  use of the name.

### Changed

- `LICENSE.md` now leads with the `SPDX-License-Identifier` expression, and
  states what the license does not cover: the HL7 standards themselves, and
  the "HL7" trademark.
- `hl7-2` gained a `[dev-dependencies]` entry for Criterion. Development
  only; it is never linked into the library or the binary, so the runtime
  dependency surface is unchanged.
- **Every README carries the HL7 trademark notice** directly under its
  title, which is what crates.io renders. `LICENSE.md` and `NEWS.md` carry
  the same wording.

### Released

`hl7` 0.1.2 · `hl7-2` 0.2.4 · `hl7-3` 0.1.4 · `hl7-2-derive` 0.1.4 ·
`hl7-3-derive` 0.1.2 · `hl7-2-mllp` 0.1.4 · `hl7-2-soap` 0.1.2 ·
`hl7-3-soap` 0.1.2 · `hl7-2-xml-lite-helper` 0.1.2 ·
`hl7-2-from-er7-into-xml` 0.6.1 · `hl7-2-from-xml-into-er7` 0.6.1 ·
`hl7-2-from-er7-into-json` 0.4.3 · `hl7-2-from-json-into-er7` 0.4.3 ·
`hl7-2-from-xsd-into-json-dictionary` 0.1.2

## 2026-08-21

### Added

- `#[hl7(crate = ...)]` on the derive macros, so `#[derive(FromHl7)]` and
  `#[derive(FromElement)]` work when `hl7-2` or `hl7-3` has been renamed in
  the consuming crate's `Cargo.toml`, or is reached through the `hl7`
  umbrella crate.

### Changed

- The MSRV policy — current stable minus three releases — is pinned as
  `rust-version` in every member's `Cargo.toml`. Crates with no other
  change were released solely to carry the pin, so that `cargo` reports a
  clear "requires rustc 1.x" rather than an error from the middle of a
  build.

### Documentation

- Stated that `get` cannot distinguish the explicit null from an empty
  field, and which calls can.

### Released

`hl7` 0.1.1 · `hl7-2` 0.2.3 · `hl7-3` 0.1.3 · `hl7-2-derive` 0.1.3 ·
`hl7-3-derive` 0.1.1 · `hl7-2-mllp` 0.1.3 · `hl7-2-soap` 0.1.1 ·
`hl7-3-soap` 0.1.1 · `hl7-2-from-xsd-into-json-dictionary` 0.1.1

## 2026-08-20

### Fixed

- **Silent data corruption in the v2.xml and JSON conversion pairs.** The
  forward and reverse crates disagreed about naming, so a round trip could
  return a message that was not the one that went in — without an error.
  This is the most serious defect the project has had, and the round trip
  is now a test rather than an assumption.
- The command line reports a second input file instead of silently reading
  standard input and ignoring it.

### Added

- Fuzz targets, and Criterion benchmarks for the conversion crates.
- The N-3 MSRV policy, written down.

### Released

`hl7-2` 0.2.2 · `hl7-2-from-er7-into-xml` 0.6.0 ·
`hl7-2-from-xml-into-er7` 0.6.0 · `hl7-2-from-er7-into-json` 0.4.2 ·
`hl7-2-from-json-into-er7` 0.4.2 · `hl7-2-xml-lite-helper` 0.1.1

The two v2.xml conversion crates took a minor bump rather than a patch
because the naming fix changed output.

## 2026-08-19

The first release of the workspace as a workspace.

### Added

- **`hl7-3`**: the HL7 v3 foundation — six RIM backbone classes, the `II`,
  `CD`, `IVL`, `PQ`, and `ED` data types, `NullFlavor`, and the three-level
  message envelope. A foundation, not a full implementation, and its own
  §1 says so.
- **`hl7-3-derive`**: `#[derive(FromElement)]` for `hl7-3`'s struct mode.
- **`hl7-3-soap`**: HL7 v3 over SOAP, sibling of `hl7-2-soap` — v3's own
  historically dominant transport.
- **`hl7-2-soap`**, **`hl7-2-xml-lite-helper`**, and
  **`hl7-2-from-xsd-into-json-dictionary`**, born in this workspace with no
  prior repository.

### Changed

- **Every `hl7-v2*` crate was renamed to `hl7-2*`.** `hl7-v2` itself
  because that name already belonged to an unrelated crate on crates.io,
  and the rest for consistency with it. The archived repositories those
  directories came from still carry their original `hl7-v2*` names.
- `clippy::pedantic` harmonized across the workspace.
- Documentation audited and corrected across every crate.

### Infrastructure

- The workspace was assembled from what were previously separate
  repositories, merged with `git subtree` so each crate's commit history is
  still walkable under its own directory. One `Cargo.lock` at the root
  covers all members.

### Released

`hl7` 0.1.0 · `hl7-2` 0.2.0, 0.2.1 · `hl7-3` 0.1.0, 0.1.1, 0.1.2 ·
`hl7-2-derive` 0.1.2 · `hl7-3-derive` 0.1.0 · `hl7-2-mllp` 0.1.1, 0.1.2 ·
`hl7-2-soap` 0.1.0 · `hl7-3-soap` 0.1.0 ·
`hl7-2-from-er7-into-xml` 0.5.0 · `hl7-2-from-xml-into-er7` 0.5.0 ·
`hl7-2-from-er7-into-json` 0.4.1 · `hl7-2-from-json-into-er7` 0.4.1 ·
`hl7-2-xml-lite-helper` 0.1.0 ·
`hl7-2-from-xsd-into-json-dictionary` 0.1.0

## Before the workspace

Each crate directory was its own repository, and its history is still
reachable: `git log <crate>/` reaches back before this workspace existed.
`hl7-3`, `hl7-3-derive`, and `hl7-3-soap` are the exceptions — they were
born here.

Two pieces of history worth knowing when reading old versions on crates.io:

- **The `hl7` crate name was claimed in 2019** by an unrelated "An HL7 2.x
  parser" at 0.0.1 and 0.0.2. This project's first release under that name
  is 0.1.0, on 2026-08-19. A crates.io page showing a 2019 creation date
  beside a 0.1.x version is that history, not neglect.
- **`er7` is a separate project**, in its own repository and its own
  organization: <https://github.com/er7-rust/er7-rust>. Encoding-level
  changes are recorded there, not here.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
