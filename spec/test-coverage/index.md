# Specification: workspace test coverage

This document defines the reproducible workspace-wide test-coverage gate.
It measures whether tests execute shipped Rust source; it does not claim that
coverage alone proves correctness.

## 1. Required threshold

Workspace line coverage MUST be at least **95%** with every workspace feature
enabled. CI MUST fail when the measured value is below that threshold.

The first conforming measurement, on 2026-11-19, covered 8,529 of 8,964
lines: **95.15%**.

After the detached-node constructor and recursive `serde-hl7-v2` node
deserializer landed later that day, the same clean measurement covered 8,737
of 9,173 lines: **95.25%**.

## 2. Denominator

The denominator is every instrumented Rust source file in the Cargo workspace,
including libraries, binaries, and procedural macros. Files below a `tests/`,
`benches/`, or `examples/` directory are excluded by path. Unit-test modules
written inside a `src/` file remain part of that file and therefore remain in
the denominator.

The gate measures lines, not regions, functions, or branches. Those other
figures remain useful diagnostics but are not release thresholds.

## 3. Reproducible command

Use `cargo-llvm-cov` 0.9.1 with Rust 1.100.0. Start from a clean instrumented
target so profiles or objects produced under a different path mapping cannot
be merged into the report:

```sh
cargo clean --target-dir target/llvm-cov-target
cargo llvm-cov --workspace --all-features --summary-only \
  --ignore-filename-regex '/(tests|benches|examples)/' \
  --fail-under-lines 95
```

An environment that sets a compiler cache wrapper MUST disable that wrapper
for both commands unless the wrapper is known to preserve coverage source
paths. This workspace's `kache` wrapper remaps cached source paths, so local
verification uses `env -u RUSTC_WRAPPER` before each command.

CI runs the same clean build and threshold on every push and pull request.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
