# HL7® for Rust

> HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
>
> This project uses the HL7® name in its package names, its organization
> name, and its domain, which is beyond fair use; we are requesting
> permission from HL7® for that. The written-permission request was sent
> on 2026-08-25; the reply is pending.

A Cargo workspace: parse, navigate, validate, modify, and render Health
Level Seven (HL7) messages in Rust. One crate per layer, one module per
standard.

```
er7                                    the ER7 encoding: delimiters,
                                       escapes, paths, byte-for-byte
                                       rendering, batch splitting
  |
hl7-2                                 the HL7 v2 dictionary: releases
  |                                    2.1-2.9, data types, message
  |                                    structures; three parsing modes;
  |                                    mutation; validation
  |
  +-- hl7-2-mllp                      transport: HL7 v2 over TCP (MLLP)
  +-- hl7-2-soap                      transport: HL7 v2 over HTTP (SOAP)
  +-- hl7-2-from-er7-into-json        format conversions
  +-- hl7-2-from-er7-into-xml
  +-- hl7-2-from-json-into-er7
  +-- hl7-2-from-xml-into-er7
  +-- hl7-2-from-xsd-into-json-dictionary   writes the dictionaries
  |                                          hl7-2 reads, from HL7
  |                                          v2.xml XSDs
  +-- hl7-2-xml-lite-helper            shared minimal XML reader, also
                                        used directly by:
        |
        +-- hl7-3                      HL7 v3: RIM backbone classes,
              |                        coded values, the three-level
              |                        message envelope — a foundation,
              |                        not a full implementation
              +-- hl7-3-derive         #[derive(FromElement)] for hl7-3's
              |                        struct mode
              +-- hl7-3-soap           transport: HL7 v3 over HTTP (SOAP)
                                       — v3's own historically dominant
                                       transport

hl7                                     the umbrella crate — hl7::v2 and
                                       hl7::v3 today, room for hl7::fhir
```

`er7` is [its own crate](https://crates.io/crates/er7), outside this
workspace. Everything else above is a member here.

## Crates

| crate | what it does |
|---|---|
| [`hl7`](hl7) | Umbrella crate. `hl7::v2` and `hl7::v3` today; room for `hl7::fhir`. |
| [`hl7-2`](hl7-2) | HL7 v2 itself: parse, navigate, validate, modify, render. Also a CLI (`hl7-v2`). |
| [`hl7-3`](hl7-3) | HL7 v3: RIM backbone classes, coded values, the three-level message envelope. A foundation, not a full implementation. |
| [`hl7-2-derive`](hl7-2-derive) | `#[derive(FromHl7)]` / `#[derive(ToHl7)]`, behind `hl7-2`'s `derive` feature. |
| [`hl7-3-derive`](hl7-3-derive) | `#[derive(FromElement)]`, behind `hl7-3`'s `derive` feature. |
| [`hl7-2-mllp`](hl7-2-mllp) | MLLP: HL7 v2 framed on a TCP stream. |
| [`hl7-2-soap`](hl7-2-soap) | HL7 v2 carried in a SOAP envelope over HTTP. |
| [`hl7-3-soap`](hl7-3-soap) | HL7 v3 carried in a SOAP envelope over HTTP — v3's own dominant transport. |
| [`hl7-2-from-er7-into-json`](hl7-2-from-er7-into-json) | ER7 → typed JSON |
| [`hl7-2-from-json-into-er7`](hl7-2-from-json-into-er7) | typed JSON → ER7 |
| [`hl7-2-from-er7-into-xml`](hl7-2-from-er7-into-xml) | ER7 → v2.xml XML |
| [`hl7-2-from-xml-into-er7`](hl7-2-from-xml-into-er7) | v2.xml XML → ER7 |
| [`hl7-2-from-xsd-into-json-dictionary`](hl7-2-from-xsd-into-json-dictionary) | HL7 v2.xml XSDs → the JSON dictionary `hl7-2` reads |
| [`hl7-2-xml-lite-helper`](hl7-2-xml-lite-helper) | Minimal XML reader shared by the v2.xml crates and `hl7-3` |

Each crate has its own `README.md` (user-facing tour) and, where behavior
is normative, a `spec/index.md` (single source of truth for that crate).

## Build

```sh
cargo build
cargo test
```

One `Cargo.lock` at the workspace root covers all members; a crate does not
carry its own.

## Workspace documents

Each crate carries its own `README.md` and, where behavior is normative, a
`spec/index.md`. These sit above them and are true of the workspace rather
than of one member.

**Start here:**

| document | what it answers |
|---|---|
| [`INSTALL.md`](INSTALL.md) | How to install and use it, as a command line or as a library |
| [`CHANGELOG.md`](CHANGELOG.md) | What changed, change by change, with the crate versions that carried it |
| [`NEWS.md`](NEWS.md) | Announcements, project status, where updates appear, and press contacts |
| [`COMPARISONS.md`](COMPARISONS.md) | Interface engines, HAPI, the other Rust crates — and when this project is the wrong answer |
| [`BENCHMARKS.md`](BENCHMARKS.md) | Measured figures and the method that produced them |

**Before you adopt it, or review it:**

| document | what it settles |
|---|---|
| [`spec/conformance/index.md`](spec/conformance/index.md) | What "supports HL7 v2 releases 2.1-2.9" means, exactly — segments, types, and structures by name, and what happens outside them |
| [`spec/phi/index.md`](spec/phi/index.md) | What these crates do with protected health information, what they never do, and where a value can escape into a log |
| [`spec/benchmark/index.md`](spec/benchmark/index.md) | The rules that govern how performance figures are produced and published |
| [`spec/rust-msrv-n-minus-2/index.md`](spec/rust-msrv-n-minus-2/index.md) | The minimum supported Rust version policy every member pins to |
| [`spec/professionalization/index.md`](spec/professionalization/index.md) | What "professional" means here — the rules that bind the maintainer, and an honest status against each |
| [`spec/trusted-publishing/index.md`](spec/trusted-publishing/index.md) | Why crates.io releases still use a long-lived API token, and what has to be true before that changes |
| [`MAINTAINERS.md`](MAINTAINERS.md) | Who maintains this, what the bus factor is, and what happens if that person is unavailable |
| [`GOVERNANCE.md`](GOVERNANCE.md) | Who decides, what binds them, what is in scope, and how to become a maintainer |
| [`SECURITY.md`](SECURITY.md) | How to report a vulnerability, what counts as one, and the known gaps |
| [`AI_STATEMENT.md`](AI_STATEMENT.md) | How AI tools are used to build this, who is accountable, and the limits that survive it |
| [`LICENSE.md`](LICENSE.md) | The five-way license choice, its SPDX expression, and what it does not cover |

**If you want to contribute:** [`CONTRIBUTING.md`](CONTRIBUTING.md) — time,
code, a report from your own feed, or money, and never pasting patient data.

**If you have an opinion:** [`RFC.md`](RFC.md) — the twelve things this
project is genuinely unsure about, and what evidence would settle each.

**If you want to cite it:** `CITATION.cff` carries the metadata, including
an ORCID.

Run the benchmarks with `cargo bench`, or `cargo bench -p hl7-2` for the
five core operations.

## Website

[`hl7-rust.github.io/`](hl7-rust.github.io) is the source of
<https://hl7-rust.github.io> — documentation, guides, tutorials, examples, and
a reference page for each crate above. It is a SvelteKit site; see its own
[`README.md`](hl7-rust.github.io/README.md) to run it locally.

```sh
make publish
```

That is the only thing the `Makefile` here does. An organization GitHub Pages
site is only ever served from a repository named `hl7-rust.github.io`, so the
site cannot deploy from this workspace; `make publish` splits that directory
out of this history and pushes it to
[that repository](https://github.com/hl7-rust/hl7-rust.github.io), which
deploys it. Building and testing the crates stays with cargo.

## History

This workspace was assembled from what were previously separate
repositories (`hl7-rust/hl7`, `hl7-rust/hl7-v2`, `hl7-rust/hl7-v2-mllp`,
and so on), merged in with `git subtree` so each crate's commit history is
still walkable under its own directory. Every `hl7-v2*` crate was later
renamed to `hl7-2*` — `hl7-v2` itself because the name was already an
unrelated crate on crates.io, and the rest for consistency with it — but
the archived repos those directories came from still carry their original
`hl7-v2*` names.

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
