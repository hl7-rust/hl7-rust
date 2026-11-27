[hl7-rust](../../README.md) → spec → Mutation testing

# Mutation testing

Line coverage says a test *ran* a line; it does not say a test would notice
if the line were wrong. Mutation testing asks the second question directly:
[cargo-mutants](https://mutants.rs/) rewrites one expression at a time (`+`
to `-`, a guard to `true`, a function body to `Default::default()`) and
runs the crate's tests. A mutant the tests fail on is **caught**; one they
pass with is **missed**, and every missed mutant names a behaviour no test
pins down.

## Contents

- [The rules](#the-rules)
- [Running it](#running-it)
- [The score](#the-score)
- [The record](#the-record)
- [Accepted mutants](#accepted-mutants)

## The rules

1. **A crate is mutated against its own tests only**
   (`test_workspace = false` in `.cargo/mutants.toml`), **with every feature
   on** (`--all-features`, as the coverage gate builds). A mutant that only
   another crate's tests catch is a gap in this crate's suite, because this
   crate is published and tested on its own; and code behind `clock` or
   `noncompliance` is otherwise never compiled, so its mutants could not be
   caught or missed, only ignored.
2. **Production code only.** `tests/`, `benches/`, `examples/` and the
   fuzz workspaces are excluded, matching the coverage denominator
   ([`spec/test-coverage/index.md`](../test-coverage/index.md)).
3. **A weekly check, not a pull-request gate.** A full run takes about half
   an hour, so `.github/workflows/mutation.yml` runs it weekly and on demand,
   not on every pull request. It fails if more mutants survive than are
   accepted below (`ACCEPTED_MISSED`, now 8). The score is recorded here, with
   its machine, toolchain and date, and a change that lowers a crate's score
   should say why.
4. **A missed mutant is closed by a test, or accepted here with a reason.**
   An equivalent mutant (one no input can tell apart from the original) is
   accepted by name in this file rather than silenced in the code.

## Running it

```sh
make test-mutants              # every crate, results in target/mutants/<crate>/
bin/test-mutants hl7-2-mllp    # one crate
```

`bin/test-mutants` runs `cargo mutants -p <crate> -j 4` per crate with the
lean `mutants` build profile from the root `Cargo.toml`, then prints one
summary line per crate. `mutants.out/missed.txt` in each crate's output
directory lists what survived.

## The score

**score = caught ÷ (caught + missed)**. Timeouts (a mutant that makes a test
loop past cargo-mutants' limit) and unviable mutants (ones that do not
compile) are reported but left out of the score: a timeout is detected,
though crudely, and an unviable mutant tests nothing.

## The record

Apple M4 Max (16 cores), macOS, `rustc 1.100.0`, cargo-mutants 27.1.0,
`-j 4`. `hl7` and `serde-hl7` re-export other crates and have no mutants
of their own.

**First run, 2026-11-26** (default features): 1,247 caught, 81 missed,
**93.9%**; `hl7-2-mllp` 88.0%, `hl7-2-from-er7-into-xml` 89.2%,
`serde-hl7-v3` 90.0% were lowest. About 30 minutes for the workspace.

**After closing the survivors, 2026-11-27** (all features): the 81 became 8,
every one accepted below.

| Crate | Caught | Missed | Timeout | Unviable | Score |
|---|---:|---:|---:|---:|---:|
| `hl7-2` | 413 | 6 | 6 | 77 | 98.6% |
| `hl7-2-from-er7-into-json` | 172 | 0 | 2 | 15 | 100.0% |
| `hl7-2-from-xsd-into-json-dictionary` | 120 | 0 | 0 | 3 | 100.0% |
| `hl7-2-from-json-into-er7` | 110 | 0 | 1 | 15 | 100.0% |
| `hl7-2-mllp` | 90 | 1 | 4 | 12 | 98.9% |
| `hl7-2-xml-lite-helper` | 86 | 0 | 2 | 2 | 100.0% |
| `hl7-2-from-er7-into-xml` | 73 | 1 | 0 | 11 | 98.6% |
| `hl7-2-soap` | 65 | 0 | 0 | 8 | 100.0% |
| `hl7-3-soap` | 55 | 0 | 0 | 8 | 100.0% |
| `hl7-2-from-xml-into-er7` | 50 | 0 | 0 | 7 | 100.0% |
| `hl7-3` | 47 | 0 | 0 | 5 | 100.0% |
| `serde-hl7-v2` | 35 | 0 | 0 | 57 | 100.0% |
| `serde-hl7-v3` | 5 | 0 | 5 | 20 | 100.0% |
| `hl7-3-derive` | 1 | 0 | 0 | 8 | 100.0% |
| `hl7-2-derive` | 0 | 0 | 0 | 11 | — |
| **workspace** | **1,322** | **8** | **20** | **259** | **99.4%** |

The unviable counts are mutants that do not compile (for example
`Default::default()` for a type with no default), and are most of
`serde-hl7-v2`. A timeout is a mutant that sends a test into a loop; it is
detected, if crudely.

## Accepted mutants

These survive because no input can tell them from the original, or because
only the other feature configuration can. Each is accepted by name here
rather than silenced in the code (rule 4).

- `hl7-2/src/validate.rs:372`, `:382` and `:416` — `||` for `&&` in the
  guards that skip an empty or explicitly null repetition, component or
  subcomponent. Checking a null value as text yields the empty string, which
  every data type accepts, so the guard only saves work.
- `hl7-2/src/validate.rs:283` — `<` for `<=` in `structure`. When a message has
  no local Z-segments its standard segments *are* all its segments, so a
  structure that did not fit the whole message cannot fit them either, and
  both readings report the same error.
- `hl7-2/src/validate.rs:591` — `!time.is_empty()` replaced by `true` in
  `split_offset`: a value that is only an offset (`-0500`) is rejected by
  either path, one for its length and the other for its sign.
- `hl7-2/src/main.rs:158` — `>` for `>=` on the option length: a lone `-` is
  matched by the arm above it first.
- `hl7-2-from-er7-into-xml/src/xml.rs:155` — `>` for `>=` on a field's
  repetition count: with exactly one repetition the joined text is the
  repetition itself.
- `hl7-2-mllp/src/framer.rs:55` — `Tolerance::lenient` returning
  `Default::default()`. With the `noncompliance` feature on (the all-features
  run) the default *is* lenient, so the two are equal; the build without that
  feature distinguishes them, and `strict_and_lenient_are_what_they_say_whatever_the_features`
  covers it there.
