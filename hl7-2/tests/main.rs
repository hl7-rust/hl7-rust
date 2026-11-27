//! Integration tests, folded into one test binary.

// RCA-EXC-5: test code may use plain operators, `unwrap` and `expect`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::integer_division,
    clippy::default_numeric_fallback,
    reason = "RCA-EXC-5: test code may use plain operators, unwrap and expect"
)]

#[path = "main/integration.rs"]
mod integration;
#[path = "main/properties.rs"]
mod properties;
#[path = "main/scratch_nhs.rs"]
mod scratch_nhs;
#[path = "main/spec.rs"]
mod spec;
