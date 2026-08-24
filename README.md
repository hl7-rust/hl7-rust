# HL7 for Rust

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
