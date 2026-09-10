//! Serde support for Health Level Seven (HL7®) messages in Rust.
//!
//! The `hl7` crate organizes HL7 by standard, one module each — `hl7::v2`
//! is the `hl7-2` crate, `hl7::v3` is `hl7-3` — because a "message" or a
//! "code" in one standard is a different thing in another. This crate is
//! the same shape one layer up: Serde support for each standard's types,
//! one module each, so a parsed message can flow through `serde_json`,
//! `serde_yaml`, `bincode`, or any other Serde data format.
//!
//! - [`v2`] — the `serde-hl7-v2` crate: `Serialize`/`Deserialize` wrappers
//!   for `hl7-2`'s message, generic-mode tree, validation diagnostics, and
//!   release. Behind the `v2` feature, on by default.
//! - [`v3`] — the `serde-hl7-v3` crate: wrappers for `hl7-3`'s message
//!   envelope, RIM backbone classes, data types, and the XML element tree
//!   beneath them. Behind the `v3` feature, on by default.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use serde_hl7::v2;
//!
//! let text = "MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\r\
//!             PID|1||241900||SMITH^JOHN";
//! let message = v2::Message::parse(text)?;
//!
//! let json = serde_json::to_string(&message)?;
//! let back: v2::Message = serde_json::from_str(&json)?;
//! assert_eq!(back.to_er7(), text);
//! # Ok(())
//! # }
//! ```
//!
//! Each module is its own crate underneath, so a caller who wants only one
//! standard can depend on that crate directly instead of this umbrella —
//! or keep this crate and turn the other off:
//!
//! ```toml
//! [dependencies]
//! serde-hl7 = { version = "0.1", default-features = false, features = ["v2"] }
//! ```
//!
//! Nothing lives at the root but the modules. Every wrapper is hand-written
//! against Serde's low-level traits in the per-standard crate, with exactly
//! two runtime dependencies each — `serde` and the `hl7-*` crate it wraps —
//! and no format crate anywhere but in tests. See each module's own
//! documentation for the wire shape of every type, and `serde-hl7-v2`'s
//! and `serde-hl7-v3`'s `spec/index.md` for the normative rules.
//!
//! # Trademarks
//!
//! HL7®, and FHIR® are the registered trademarks of Health Level Seven
//! International and their use of these trademarks does not constitute an
//! endorsement by HL7.

// No `unsafe` anywhere in this crate, enforced rather than merely true:
// `forbid` cannot be lifted by an `allow` further down, so this is a
// property a reviewer can rely on without reading the sources. See
// SECURITY.md.
#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

#[cfg(feature = "v2")]
pub use serde_hl7_v2 as v2;

#[cfg(feature = "v3")]
pub use serde_hl7_v3 as v3;
