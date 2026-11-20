# Specification: workspace test coverage

This document defines the reproducible workspace-wide test-coverage gate.
It measures whether tests execute shipped Rust source; it does not claim that
coverage alone proves correctness.

## 1. Required threshold

Workspace line coverage MUST be **100%** with every workspace feature
enabled. CI MUST fail when the measured value is below that threshold.

The first conforming measurement, on 2026-11-19, covered 8,529 of 8,964
lines: **95.15%**.

After the detached-node constructor and recursive `serde-hl7-v2` node
deserializer landed later that day, the same clean measurement covered 8,737
of 9,173 lines: **95.25%**.

After the workspace-wide backlog of untested error, fallback, CLI I/O, and
macro-generated paths was closed on 2026-11-20, the same clean measurement
covered 9,540 of 9,540 lines: **100.00%**.

## 2. Denominator

The denominator is every distinct instrumented line in every Rust source file
in the Cargo workspace, including libraries, binaries, and procedural macros.
Files below a `tests/`, `benches/`, or `examples/` directory are excluded by
path. Unit-test modules written inside a `src/` file remain part of that file
and therefore remain in the denominator.

Rust can compile the same physical source line into several compilation units,
particularly for generic code, procedural macros, and a library that has both
unit and integration tests. LCOV records for the same source path and line
number are therefore unioned: the line is covered when any instrumented copy
executes. This prevents compiler compilation strategy from changing the
denominator or treating one source line as several source lines.

The gate measures lines, not regions, functions, or branches. Those other
figures remain useful diagnostics but are not release thresholds.

## 3. Reproducible command

Use `cargo-llvm-cov` 0.9.1 with Rust 1.100.0. Start from a clean instrumented
target so profiles or objects produced under a different path mapping cannot
be merged into the report:

```sh
./bin/coverage
```

The script starts from a clean instrumented target, produces LCOV with
`cargo llvm-cov --workspace --all-features`, unions records by source path and
line number, and fails unless every distinct line has a nonzero execution
count.

An environment that sets a compiler cache wrapper MUST disable that wrapper
unless the wrapper is known to preserve coverage source paths. This workspace's
`kache` wrapper remaps cached source paths, so `bin/coverage` disables
`RUSTC_WRAPPER` for the clean measurement.

CI runs the same clean build and threshold on every push and pull request.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
