# HL7 for Rust

A Cargo workspace: parse, navigate, validate, modify, and render Health
Level Seven (HL7) messages in Rust. One crate per layer, one module per
standard.

```
er7                                    the ER7 encoding: delimiters,
                                       escapes, paths, byte-for-byte
                                       rendering, batch splitting
  |
hl7-v2                                 the HL7 v2 dictionary: releases
                                       2.1-2.9, data types, message
                                       structures; three parsing modes;
                                       mutation; validation
  |
hl7                                     the umbrella crate — hl7::v2,
                                       room for hl7::v3 and hl7::fhir
  |
  +-- hl7-v2-mllp                      transport: HL7 v2 over TCP (MLLP)
  +-- hl7-v2-soap                      transport: HL7 v2 over HTTP (SOAP)
  +-- hl7-v2-from-er7-into-json        format conversions
  +-- hl7-v2-from-er7-into-xml
  +-- hl7-v2-from-json-into-er7
  +-- hl7-v2-from-xml-into-er7
  +-- hl7-v2-from-xsd-into-json-dictionary   writes the dictionaries
  |                                          hl7-v2 reads, from HL7
  |                                          v2.xml XSDs
  +-- hl7-v2-xml-lite-helper            shared minimal XML reader
```

`er7` is [its own crate](https://crates.io/crates/er7), outside this
workspace. Everything else above is a member here.

## Crates

| crate | what it does |
|---|---|
| [`hl7`](hl7) | Umbrella crate. `hl7::v2` today; room for `hl7::v3`/`hl7::fhir`. |
| [`hl7-v2`](hl7-v2) | HL7 v2 itself: parse, navigate, validate, modify, render. Also a CLI (`hl7-v2`). |
| [`hl7-v2-derive`](hl7-v2-derive) | `#[derive(FromHl7)]` / `#[derive(ToHl7)]`, behind `hl7-v2`'s `derive` feature. |
| [`hl7-v2-mllp`](hl7-v2-mllp) | MLLP: HL7 v2 framed on a TCP stream. |
| [`hl7-v2-soap`](hl7-v2-soap) | HL7 v2 carried in a SOAP envelope over HTTP. |
| [`hl7-v2-from-er7-into-json`](hl7-v2-from-er7-into-json) | ER7 → typed JSON |
| [`hl7-v2-from-json-into-er7`](hl7-v2-from-json-into-er7) | typed JSON → ER7 |
| [`hl7-v2-from-er7-into-xml`](hl7-v2-from-er7-into-xml) | ER7 → v2.xml XML |
| [`hl7-v2-from-xml-into-er7`](hl7-v2-from-xml-into-er7) | v2.xml XML → ER7 |
| [`hl7-v2-from-xsd-into-json-dictionary`](hl7-v2-from-xsd-into-json-dictionary) | HL7 v2.xml XSDs → the JSON dictionary `hl7-v2` reads |
| [`hl7-v2-xml-lite-helper`](hl7-v2-xml-lite-helper) | Minimal XML reader shared by the v2.xml crates |

Each crate has its own `README.md` (user-facing tour) and, where behavior
is normative, a `spec/index.md` (single source of truth for that crate).

## Build

```sh
cargo build
cargo test
```

One `Cargo.lock` at the workspace root covers all members; a crate does not
carry its own.

## History

This workspace was assembled from what were previously separate
repositories (`hl7-rust/hl7`, `hl7-rust/hl7-v2`, `hl7-rust/hl7-v2-mllp`,
and so on), merged in with `git subtree` so each crate's commit history is
still walkable under its own directory.

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
