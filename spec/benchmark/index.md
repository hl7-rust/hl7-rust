[hl7-rust](../../README.md) → spec → Benchmark

# Benchmarks

How performance is measured here, what the numbers mean, and what they are
not evidence of. A number without its method is a marketing claim, so the
method comes first and the figures come last.

## Contents

- [The rules](#the-rules)
- [What is measured, and why those five](#what-is-measured-and-why-those-five)
- [The inputs](#the-inputs)
- [Running them](#running-them)
- [Comparing a change](#comparing-a-change)
- [Measured figures](#measured-figures)
- [Reading those figures](#reading-those-figures)
- [What these numbers are not](#what-these-numbers-are-not)

## The rules

1. **Every published figure names its machine, its toolchain, and its
   date.** A number without those three is not reproducible and is not
   published.
2. **Every benchmark is in the repository and runnable by anyone**, with
   one command, on inputs that are also in the repository.
3. **Inputs are synthetic.** No real patient data, ever, for any reason —
   see [`spec/phi/index.md`](../phi/index.md).
4. **Criterion, defaults, no tuning.** Whatever Criterion's warm-up and
   sample count are, unchanged, so the numbers are comparable to anyone
   else's Criterion run.
5. **The published figure is the middle of Criterion's three**, its point
   estimate. The outer two are its confidence interval and are reported
   alongside rather than quietly dropped.
6. **A performance claim in a pull request carries a before-and-after** from
   the same machine in the same sitting. Cross-machine comparison is not
   evidence.
7. **Correctness outranks speed.** A faster parser that loses a value, or
   that stops round-tripping byte for byte, is not faster.

## What is measured, and why those five

`hl7-2/benches/parse.rs` measures five operations separately rather than
one end-to-end figure, because they scale differently and a given interface
pays for only some of them:

| Group | The operation | Paid |
|---|---|---|
| `parse` | Text in, `Message` out | Once per message |
| `get` | One field read by path | Once per field an integration cares about |
| `tree` | The whole generic tree, every value named | Only when something walks the message |
| `validate` | The message against its dictionary | Only when validation is asked for |
| `render` | `Message` back to ER7 text | Once per message written out |

One combined "messages per second" would hide which of those an interface
is actually paying for. A feed that reads two fields and forwards the
message pays `parse` plus two `get`s; a converter pays `parse` plus `tree`;
a strict receiver pays `parse` plus `validate`. Those are different costs
and they deserve different numbers.

The conversion crates each have their own `benches/convert.rs` measuring
one conversion end to end, which is the right shape for them, because a
conversion *is* the operation.

## The inputs

Two, at the two sizes that matter:

- **small** — a four-segment `ADT^A08` (MSH, EVN, PID, PV1), **177 bytes**.
  The shape most interfaces move in bulk, and the one where per-message
  overhead dominates.
- **large** — an `ORU^R01` carrying 200 observations as OBR/OBX/NTE
  triples, **29,104 bytes**. The shape that decides whether a parser keeps
  up with a day's traffic, and where per-segment cost dominates.

Both are built in code at the top of the benchmark file, so the input is
readable next to the measurement and cannot drift.

Neither is a claim about a representative message mix. There isn't one:
site traffic varies more between two hospitals than between these two
sizes. They are chosen to bracket the range, not to average it.

## Running them

```sh
cargo bench -p hl7-2                     # the five groups above
cargo bench                              # every crate that has benchmarks
cargo bench -p hl7-2 -- parse            # one group
```

Benchmarks build under the `bench` profile, which is `release` plus debug
assertions off. Running them under `--profile dev` measures the debug build
and is meaningless.

## Comparing a change

The only comparison that means anything is against the same machine, in the
same sitting, minutes apart:

```sh
git stash
cargo bench -p hl7-2 -- --save-baseline before
git stash pop
cargo bench -p hl7-2 -- --baseline before
```

Criterion prints the change and whether it considers it significant. Treat
anything under about 5% on a laptop as noise: thermal state, other
processes, and allocator behavior move numbers by that much between runs
that changed nothing.

## Measured figures

**Machine:** Apple M4 Max, 128 GB, macOS 26.6.1, arm64.
**Toolchain:** rustc 1.98.0 (88d9e12ae 2026-08-18), release profile.
**Date:** 2026-08-26. **Crate:** `hl7-2` 0.2.3, `er7` 0.1.2.
**Method:** `cargo bench -p hl7-2`, Criterion defaults, machine otherwise
idle.

| Group | Input | Time | Throughput |
|---|---|---|---|
| `parse` | small, 177 B | 3.00 µs | 56.3 MiB/s |
| `parse` | large, 29,104 B | 381 µs | 72.8 MiB/s |
| `get` | small, `PID-5.1` | 110 ns | — |
| `get` | large, `OBX[200]-5` | 1.94 µs | — |
| `tree` | small | 13.4 µs | — |
| `tree` | large | 1.50 ms | — |
| `validate` | small | 6.95 µs | — |
| `validate` | large | 633 µs | — |
| `render` | small, 177 B | 365 ns | 463 MiB/s |
| `render` | large, 29,104 B | 29.4 µs | 944 MiB/s |

Criterion's confidence intervals for the same run, lower and upper:

| Group | Input | Interval |
|---|---|---|
| `parse` | small | 2.97 – 3.04 µs |
| `parse` | large | 376 – 388 µs |
| `get` | small | 108 – 112 ns |
| `get` | large | 1.91 – 1.97 µs |
| `tree` | small | 13.3 – 13.6 µs |
| `tree` | large | 1.49 – 1.52 ms |
| `validate` | small | 6.87 – 7.06 µs |
| `validate` | large | 628 – 640 µs |
| `render` | small | 360 – 372 ns |
| `render` | large | 28.9 – 30.1 µs |

## Reading those figures

Four things in that table are worth saying out loud, including the
unflattering one:

**Parsing a small message costs about 3 µs**, so a single core parses on the
order of 300,000 small ADTs a second. For essentially every real HL7
interface, parsing is not the bottleneck — the network, the database, and
the downstream system are.

**Rendering is roughly eight times cheaper than parsing** (365 ns against
3.00 µs on the small message), which is what "stored as sent, decoded on
demand" buys: writing back out is mostly copying bytes that were never
transformed.

**`get` is the cheap path and `tree` is the expensive one.** Reading two
fields from the large message costs about 4 µs; building its whole tree
costs 1.5 ms — nearly 400 times more. An integration that wants a handful
of fields should use paths and not walk the tree. This is the single most
useful thing in this document for someone writing against these crates.

**`tree` on the large message is the slowest thing here, and it is slower
than parsing the same message four times over.** It allocates a named node
for every value in a 600-segment message, so the cost is real work rather
than waste — but it has had no optimization attention, and it is the
obvious place to look first if a profile points here. Stating that is more
useful than omitting the row.

## What these numbers are not

- **Not a comparison against another library.** Nothing here has been
  benchmarked against HAPI, Mirth, Open Integration Engine, or another Rust
  crate. Doing that fairly means matching what each one actually does — and
  a parser that only splits on pipes is not doing the same work as one that
  resolves a dictionary — so until that comparison is built and published
  with its method, no claim is made. See
  [`spec/conformance/index.md`](../conformance/index.md) for what this
  project claims to do, which is the honest basis for any later comparison.
- **Not a guarantee.** They are one run, on one machine, on one day, on two
  synthetic messages.
- **Not a throughput figure for a system.** These measure a library call.
  MLLP framing, TCP, acknowledgement round trips, disk, and the database
  behind the interface are all outside them, and in a real deployment they
  are usually what sets the ceiling.
- **Not a memory measurement.** Nothing here reports allocations or peak
  resident size. That is a gap, and a contribution that adds it would be
  welcome.
