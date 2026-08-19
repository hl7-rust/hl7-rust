//! Health Level Seven (HL7) for Rust.
//!
//! HL7 is not one standard but a family of them, and they have little in
//! common beyond the name and the problem: v2 is delimited text, v3 is XML,
//! FHIR is resources over HTTP. So this crate is organized by standard,
//! one module each, and the module you want is the version your senders
//! speak:
//!
//! - [`v2`] — HL7 v2, releases 2.1 through 2.9. The format most healthcare
//!   data still moves in.
//!
//! ```
//! use hl7::v2;
//!
//! let message = v2::parse("MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\r\
//!                          PID|1||241900||SMITH^JOHN")?;
//! assert_eq!(message.structure_id(), "ORU_R01");
//! assert_eq!(message.get("PID-5.1")?.as_deref(), Some("SMITH"));
//! # Ok::<(), v2::Error>(())
//! ```
//!
//! Each module is its own crate underneath — `hl7::v2` is the `hl7-v2`
//! crate, re-exported here — so a caller who wants only one standard can
//! depend on that crate directly instead of this umbrella:
//!
//! ```toml
//! [dependencies]
//! hl7-v2 = "0.2"
//! ```
//!
//! Nothing lives at the root but the modules. A name that means one thing
//! in v2 means another in FHIR — a "message", a "segment", a "code" — and
//! flattening them into one namespace would only invite mixing them up.
//!
//! See [`v2`] for everything HL7 v2, and `hl7-v2`'s `spec/index.md` for the
//! normative specification of it.

#![warn(missing_docs)]

pub use hl7_v2 as v2;
