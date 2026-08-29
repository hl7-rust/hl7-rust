[hl7-rust](../../README.md) → spec → Rust fuzzing

# Rust fuzzing

Three crates have fuzz targets. All three keep them **outside** the main
Cargo workspace, on purpose, and this is the rule that says so and why.

## The rule

- A fuzz crate (`<crate>/fuzz/`) MUST declare its own empty `[workspace]`
  table in its `Cargo.toml`, making it a workspace root of one rather than
  a member of the root workspace at `Cargo.toml`.
- The root workspace's `members` list MUST NOT include any `<crate>/fuzz`
  path.
- Fuzzing MUST require only its own crate's toolchain, never the one
  [`spec/rust-msrv-n-minus-2/index.md`](../rust-msrv-n-minus-2/index.md)
  pins for the rest of the workspace.

## Why

`cargo-fuzz`'s runtime dependency, `libfuzzer-sys`, needs sanitizer-coverage
instrumentation flags that are nightly-only — there is no way to fuzz with
it on stable Rust. The MSRV policy is stable-only by definition: pre-release
channels are never the MSRV and no workspace target may require one.

Those two facts collide if a fuzz crate is a member of the main workspace.
`cargo check --workspace --all-targets` — the exact command
[`spec/rust-msrv-n-minus-2/index.md`](../rust-msrv-n-minus-2/index.md) runs
against the MSRV toolchain, and the exact command the `msrv` job in
[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) enforces —
would then need nightly to succeed on stable, which is precisely the
requirement the MSRV floor exists to rule out. Giving each fuzz crate its
own `[workspace]` keeps it invisible to every `--workspace` command run
against the root, so the MSRV floor holds without the fuzz crates ever
having to compile under it.

A fuzz crate's own `Cargo.toml` says the same thing in its own words,
verbatim across all three:

> Its own workspace: the fuzzers need `libfuzzer-sys` and a nightly
> toolchain, neither of which belongs in the workspace above.

## Where they are

| Crate | Fuzz targets |
|---|---|
| `hl7-2-xml-lite-helper` | `parse` |
| `hl7-2-from-xml-into-er7` | `convert`, `roundtrip` |
| `hl7-2-from-json-into-er7` | `convert`, `roundtrip` |

Five targets across three crates — the parsing surfaces that take
untrusted structured input from outside the workspace's own control:
hand-rolled XML and JSON readers, and the two conversions built on the XML
one. [`SECURITY.md`](../../SECURITY.md) names what is *not* yet covered —
the ER7 parsing surface in `er7` itself, and `hl7-2`'s dictionary reader —
as an open gap rather than an oversight to discover later.

## Running a fuzz target

Requires `cargo install cargo-fuzz` and a nightly toolchain, neither of
which this workspace's own build needs:

```sh
cd hl7-2-xml-lite-helper/fuzz
cargo +nightly fuzz run parse
```

Each fuzz `Cargo.toml` names its own crate's targets; `cargo fuzz list`
from inside a `fuzz/` directory lists them.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
