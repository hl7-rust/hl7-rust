# Benchmarks

Measured figures for the five things `hl7-2` is asked to do, and the method
that produced them. A number without its method is a marketing claim, so
both are here.

The normative version of this document — the rules that govern how figures
are produced and published — is
[`spec/benchmark/index.md`](spec/benchmark/index.md). The reader-friendly
version is <https://hl7-rust.github.io/docs/benchmarks/>. This file is the
summary a person lands on from the repository root.

## The figures

**Machine:** Apple M4 Max, 128 GB, macOS 26.6.1, arm64.
**Toolchain:** rustc 1.98.0 (88d9e12ae 2026-08-18), release profile.
**Date:** 2026-08-30.
**Crates:** `hl7-2` 0.3.0 over `er7` 0.2.1 (the version `Cargo.lock`
pins) — the current crates.io release, so the figures are the code you
would install today.
**Method:** `cargo bench -p hl7-2`, Criterion defaults, machine otherwise
idle. Time is Criterion's point estimate; the interval is its confidence
interval, reported rather than quietly dropped.

| Group | Input | Time | Interval | Throughput |
|---|---|---|---|---|
| `parse` | small, 177 B | 2.81 µs | 2.79 – 2.82 µs | 60.1 MiB/s |
| `parse` | large, 29,104 B | 373 µs | 371 – 376 µs | 74.3 MiB/s |
| `get` | small, `PID-5.1` | 79.2 ns | 78.4 – 80.1 ns | — |
| `get` | large, `OBX[200]-5` | 1.80 µs | 1.77 – 1.83 µs | — |
| `tree` | small | 13.0 µs | 12.8 – 13.1 µs | — |
| `tree` | large | 1.44 ms | 1.44 – 1.45 ms | — |
| `validate` | small | 6.40 µs | 6.38 – 6.42 µs | — |
| `validate` | large | 602 µs | 598 – 607 µs | — |
| `render` | small, 177 B | 367 ns | 365 – 369 ns | 460 MiB/s |
| `render` | large, 29,104 B | 28.5 µs | 28.3 – 28.7 µs | 974 MiB/s |

Measured with two consecutive full-suite runs on this same machine,
minutes apart, to check for the kind of swing the previous measurement
(2026-08-26) reported: this time `render/small` agreed within about 1%
across the two runs (363 ns and 367 ns, overlapping intervals), not the
13% swing seen before. That doesn't mean the earlier swing was wrong —
it means run-to-run noise on a laptop is itself not constant, and
nanosecond-scale rows should still be read as a scale, not a guaranteed
point.

The conversion crates carry their own `benches/convert.rs`, measuring one
conversion end to end — the right shape for them, because a conversion
*is* the operation. Run everything with `cargo bench` at the workspace
root.

## What the numbers say

**Parsing is not your bottleneck.** A small ADT parses in about 2.8 µs, so
one core parses on the order of 350,000 a second. For essentially every
real HL7® interface the network, the database, and the downstream system
decide the throughput. Choosing a library on parse speed is optimising the
wrong number.

**Rendering is seven to eight times cheaper than parsing** — about 0.37 µs
against 2.81 µs on the same message. That is what "stored as sent, decoded
on demand" buys: writing back out is mostly copying bytes that were never
transformed, which is also why the round trip comes back byte for byte.

**Use paths, not the tree.** Reading two fields from the large message
costs about 3.6 µs. Building its whole tree costs 1.44 ms — nearly 400
times more. An integration that wants a handful of fields should use
paths and never materialize the tree. This is the most useful line in
this document.

**The tree on a large message is the slowest thing here**, slower than
parsing that message four times over. It allocates a named node for every
value in a 600-segment message, so the cost is real work rather than waste
— but it has had no optimization attention, and it is the first place to
look if a profile points this way. Stating that is more useful than
omitting the row.

## The inputs

Two, at the two sizes that matter, both built in code at the top of
`hl7-2/benches/parse.rs` so the input is readable next to the measurement
and cannot drift:

- **small — 177 bytes.** A four-segment `ADT^A08`. The shape most
  interfaces move in bulk, where per-message overhead dominates.
- **large — 29,104 bytes.** An `ORU^R01` carrying 200 observations as
  OBR/OBX/NTE triples. The shape that decides whether a parser keeps up
  with a day's traffic, where per-segment cost dominates.

Neither is a claim about a representative message mix. There isn't one:
site traffic varies more between two hospitals than between these two
sizes. They bracket the range rather than averaging it.

Every input is synthetic. No real patient data, ever, for any reason —
[`spec/phi/index.md`](spec/phi/index.md).

## Running them

```sh
cargo bench                       # every crate that has benchmarks
cargo bench -p hl7-2              # the five groups above
cargo bench -p hl7-2 -- parse     # one group
```

Benchmarks build under the `bench` profile: release, with debug assertions
off. Running them under the dev profile measures the debug build and is
meaningless.

## Comparing a change

The only comparison that means anything is the same machine, minutes apart:

```sh
git stash
cargo bench -p hl7-2 -- --save-baseline before
git stash pop
cargo bench -p hl7-2 -- --baseline before
```

Criterion prints the change and whether it considers it significant. Treat
anything under about 5% on a laptop as noise — thermal state, other
processes, and allocator behavior move numbers by that much between runs
that changed nothing.

A pull request argued on performance carries a before-and-after produced
this way. Correctness outranks speed: a faster parser that loses a value,
or that stops round-tripping byte for byte, is not faster.

## What these are not

- **Not a comparison against another library.** Nothing here has been
  benchmarked against HAPI, Mirth, Open Integration Engine, or another Rust
  crate. Comparing fairly means matching what each one actually does, and a
  parser that only splits on pipes is not doing the same work as one that
  resolves a dictionary. Until that comparison exists with its method
  published, no claim is made. The honest, capability-based version is
  [`COMPARISONS.md`](COMPARISONS.md).
- **Not a guarantee.** One run, one machine, one day, two synthetic
  messages.
- **Not a throughput figure for a system.** These measure a library call.
  MLLP framing, TCP, acknowledgement round trips, disk, and the database
  behind the interface are outside them, and usually they set the ceiling.
- **Not a memory measurement.** Nothing here reports allocations or peak
  resident size. That is a gap, and a contribution adding it would be
  welcome.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
