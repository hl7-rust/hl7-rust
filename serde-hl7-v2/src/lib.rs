//! # serde-hl7-v2
//!
//! Serde support for [`hl7_2`], the HL7® v2 dictionary layer: wrap a parsed
//! [`hl7_2::Message`] in [`Message`], its generic-mode tree in [`Node`], or a
//! validation finding in [`Diagnostic`], and it can flow through
//! `serde_json`, `serde_yaml`, `bincode`, or any other Serde data format —
//! for storing a message in a document database, logging its
//! dictionary-named tree as structured JSON, returning validation results
//! from a web API, or testing against a literal.
//!
//! [`hl7_2`] has one runtime dependency, `er7`, and no Serde support of its
//! own, deliberately: adding `serde` there would cost every user of `hl7-2`
//! a dependency they may not want. This crate is the bridge instead — two
//! dependencies, `serde` and `hl7-2`, and nothing else. Every impl is
//! written by hand against the low-level `Serializer`/`Deserializer`/
//! `Visitor` traits, following the pattern [serde's own
//! documentation](https://docs.rs/serde/latest/serde/) walks through for a
//! manual implementation — no `#[derive(Serialize)]` anywhere, because the
//! types wrapped here are foreign and the shapes below are chosen, not
//! derived.
//!
//! Most callers get this crate through the `serde-hl7` umbrella, which
//! re-exports it as `serde_hl7::v2`, the same way `hl7` re-exports
//! `hl7-2` as `hl7::v2`.
//!
//! # Trademarks
//!
//! HL7®, and FHIR® are the registered trademarks of Health Level Seven
//! International and their use of these trademarks does not constitute an
//! endorsement by HL7.
//!
//! # Example
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use serde_hl7_v2::Message;
//!
//! let text = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260815120000||ORU^R01|MSG9|P|2.5\r\
//!             PID|1||12345^^^ACME^MR||SMITH^JOHN^Q||19800101|M\r\
//!             OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|||||F";
//!
//! let message = Message::parse(text)?;
//!
//! // Out to JSON, and back — this crate never mentions JSON itself; any
//! // Serde format works the same way.
//! let json = serde_json::to_string(&message)?;
//! let back: Message = serde_json::from_str(&json)?;
//!
//! assert_eq!(back.to_er7(), text);
//! assert_eq!(back.version(), message.version());
//! # Ok(())
//! # }
//! ```
//!
//! # The shape each type serializes as
//!
//! | `hl7-2` type | Wrapper | Serializes as | Deserializes |
//! |---|---|---|---|
//! | [`hl7_2::Message`] | [`Message`] | object: `{"version": "2.5", "er7": "MSH\|^~\\&\|…"}` | yes — re-parsed through `hl7_2`, so the dictionary is resolved, not shipped |
//! | [`hl7_2::Node`] | [`Node`] | object: `{"name", "path", "kind", "text", "null", "children": [...]}` | no — `hl7_2` builds trees only from a parsed message |
//! | [`hl7_2::generic::Kind`] | [`NodeKind`] | one of the strings `"Group"`, `"Segment"`, `"Field"`, `"Component"`, `"Subcomponent"` | yes |
//! | [`hl7_2::Diagnostic`] | [`Diagnostic`] | object: `{"severity", "kind", "path", "detail"}` | yes |
//! | [`hl7_2::Severity`] | [`Severity`] | `"Error"` or `"Warning"` | yes |
//! | [`hl7_2::validate::Kind`] | [`DiagnosticKind`] | the variant name, e.g. `"SegmentMissing"` | yes |
//! | [`hl7_2::Version`] | [`Version`] | the release as MSH-12 spells it: `"2.5.1"` | yes |
//!
//! The message itself travels as its ER7 text plus the release it was read
//! as, not as a tree: `hl7-2` is the dictionary layer, not the encoding
//! layer, and the encoding layer's tree already has a Serde crate of its
//! own — `serde-er7`, over [`hl7_2::er7`]'s types, reachable from
//! [`hl7_2::Message::raw`]. What this crate adds on top is the dictionary-aware
//! view: [`Node`], with `PID.5` and `XPN.1` for names, and [`Diagnostic`],
//! with what validation found.
//!
//! Deserializing an object-shaped type ignores a key it does not recognize,
//! by default. A caller who wants a typo in a hand-written JSON fixture
//! reported instead deserializes into [`Strict<Message>`](Strict) or
//! [`Strict<Diagnostic>`](Strict), which rejects an unknown key rather than
//! ignoring it.
//!
//! A message read through a caller's own dictionary — `hl7-2`'s schema mode
//! — deserializes through [`Message::seed`], which carries the
//! [`hl7_2::Options`] the plain `Deserialize` impl has no way to be given.
//!
//! # What is deliberately not here
//!
//! This crate adds exactly one thing to [`hl7_2`]: Serde support for its
//! existing public types. It does not add a wire format (`serde_json` is a
//! dev-dependency only), a tree shape for the ER7 encoding (that is
//! `serde-er7`'s), or Serde support for [`hl7_2::Dictionary`] — dictionaries
//! are JSON already, in the shape `hl7_2::Dictionary::from_json` reads, and
//! a second shape would be a second thing to keep in step.
//!
//! # Documentation
//!
//! `spec/index.md` in the repository is this crate's specification — what
//! each wrapper type must serialize as and why. Runnable programs are in
//! `examples/`.

// No `unsafe` anywhere in this crate, enforced rather than merely true:
// `forbid` cannot be lifted by an `allow` further down, so this is a
// property a reviewer can rely on without reading the sources. See
// SECURITY.md.
#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

mod diagnostic;
mod kind;
mod message;
mod node;
mod strict;
mod version;

pub use diagnostic::Diagnostic;
pub use kind::{DiagnosticKind, NodeKind, Severity};
pub use message::{Message, MessageSeed};
pub use node::Node;
pub use strict::Strict;
pub use version::Version;

// Re-exported so a caller can name `hl7_2::Options`, `hl7_2::Error`, and
// the rest — and `hl7_2::er7` beneath them — without adding their own
// dependency, the same convenience `hl7-2` itself extends for `er7`.
pub use hl7_2;
