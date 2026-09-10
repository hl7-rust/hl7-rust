//! # serde-hl7-v3
//!
//! Serde support for [`hl7_3`], the HL7® v3 foundation crate: wrap a parsed
//! [`hl7_3::Message`] in [`Message`], a RIM class in [`Act`] or [`Entity`],
//! a data type in [`Ii`] or [`Cd`], or a raw XML element in [`Element`],
//! and it can flow through `serde_json`, `serde_yaml`, `bincode`, or any
//! other Serde data format — for storing a decoded interaction in a
//! document database, logging it as structured JSON, returning it from a
//! web API, or testing against a literal.
//!
//! [`hl7_3`] has one runtime dependency, the XML reader it parses through,
//! and no Serde support of its own, deliberately: adding `serde` there
//! would cost every user of `hl7-3` a dependency they may not want. This
//! crate is the bridge instead — two dependencies, `serde` and `hl7-3`, and
//! nothing else. Every impl is written by hand against the low-level
//! `Serializer`/`Deserializer`/`Visitor` traits, following the pattern
//! [serde's own documentation](https://docs.rs/serde/latest/serde/) walks
//! through for a manual implementation — no `#[derive(Serialize)]`
//! anywhere, because the types wrapped here are foreign and the shapes
//! below are chosen, not derived.
//!
//! Most callers get this crate through the `serde-hl7` umbrella, which
//! re-exports it as `serde_hl7::v3`, the same way `hl7` re-exports
//! `hl7-3` as `hl7::v3`.
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
//! use serde_hl7_v3::Message;
//!
//! let xml = r#"
//! <QUQI_IN000001UV01 xmlns="urn:hl7-org:v3">
//!   <id root="2.16.840.1.113883.19.5" extension="MSG00001"/>
//!   <interactionId root="2.16.840.1.113883.1.6" extension="QUQI_IN000001UV01"/>
//!   <controlActProcess classCode="CACT" moodCode="EVN">
//!     <code code="QUQI_TE000001UV01"/>
//!     <subject><observation classCode="OBS" moodCode="EVN"/></subject>
//!   </controlActProcess>
//! </QUQI_IN000001UV01>"#;
//!
//! let message = Message::parse(xml)?;
//!
//! // Out to JSON, and back — this crate never mentions JSON itself; any
//! // Serde format works the same way.
//! let json = serde_json::to_string(&message)?;
//! let back: Message = serde_json::from_str(&json)?;
//!
//! assert_eq!(back, message);
//! assert_eq!(back.interaction_id.as_ref().unwrap().extension.as_deref(), Some("QUQI_IN000001UV01"));
//! # Ok(())
//! # }
//! ```
//!
//! # The shape each type serializes as
//!
//! Every type is an object whose keys are the `hl7-3` field names in
//! lowerCamelCase — which, for every field that corresponds to an HL7 v3
//! XML attribute or element, is that attribute's or element's own name:
//! `classCode`, `moodCode`, `codeSystem`, `effectiveTime`, `interactionId`.
//! An `Option` field serializes as `null` when absent; a `Vec` field as an
//! array, `[]` when empty.
//!
//! | `hl7-3` type | Wrapper | Keys |
//! |---|---|---|
//! | [`hl7_3::Message`] | [`Message`] | `id`, `creationTime`, `interactionId`, `sender`, `receiver`, `controlAct` |
//! | [`hl7_3::ControlAct`] | [`ControlAct`] | `classCode`, `moodCode`, `code`, `domain` |
//! | [`hl7_3::xml::Element`] | [`Element`] | `name`, `attributes` (an object), `text`, `children` |
//! | [`hl7_3::Ii`] | [`Ii`] | `root`, `extension` |
//! | [`hl7_3::Cd`] | [`Cd`] | `code`, `codeSystem`, `displayName` |
//! | [`hl7_3::Ivl`] | [`Ivl`] | `value`, `low`, `high` |
//! | [`hl7_3::Pq`] | [`Pq`] | `value`, `unit` |
//! | [`hl7_3::Ed`] | [`Ed`] | `mediaType`, `representation`, `text` |
//! | [`hl7_3::NullFlavor`] | [`NullFlavor`] | a bare string: the code, `"NI"`, `"UNK"`, … |
//! | [`hl7_3::rim::Act`] | [`Act`] | `classCode`, `moodCode`, `id`, `code`, `statusCode`, `effectiveTime`, `text` |
//! | [`hl7_3::rim::Entity`] | [`Entity`] | `classCode`, `determinerCode`, `id`, `code`, `name` |
//! | [`hl7_3::rim::Role`] | [`Role`] | `classCode`, `id`, `code`, `statusCode`, `effectiveTime` |
//! | [`hl7_3::rim::Participation`] | [`Participation`] | `typeCode`, `time`, `functionCode` |
//! | [`hl7_3::rim::ActRelationship`] | [`ActRelationship`] | `typeCode`, `inversionInd` |
//! | [`hl7_3::rim::RoleLink`] | [`RoleLink`] | `typeCode` |
//!
//! Deserializing any object ignores a key it does not recognize, by
//! default. A caller who wants a typo in a hand-written JSON fixture
//! reported instead deserializes into [`Strict<T>`](Strict), which rejects
//! an unknown key — at the top level or nested anywhere beneath — rather
//! than ignoring it.
//!
//! # What round-trips
//!
//! A value through any Serde format and back is equal to the value that
//! went in — every field of every type here is public, so nothing is lost
//! in either direction. What this crate does not promise is XML: `hl7-3`
//! reads a message into its three-level envelope and keeps the payload as
//! a raw [`Element`], and it has no XML writer, so there is no
//! "back to the original document" here to guarantee. The `Element` tree
//! itself round-trips exactly, attributes and children included.
//!
//! # What is deliberately not here
//!
//! This crate adds exactly one thing to [`hl7_3`]: Serde support for its
//! existing public types. It does not add a wire format (`serde_json` is a
//! dev-dependency only), an XML writer, or Serde support for the
//! struct-mode trait [`hl7_3::FromElement`] — a caller's own struct is
//! theirs to derive Serde for.
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

#[macro_use]
mod object;

mod element;
mod message;
mod null_flavor;
mod rim;
mod strict;
mod vocabulary;

pub use element::Element;
pub use message::{ControlAct, Message};
pub use null_flavor::NullFlavor;
pub use rim::{Act, ActRelationship, Entity, Participation, Role, RoleLink};
pub use strict::Strict;
pub use vocabulary::{Cd, Ed, Ii, Ivl, Pq};

// Re-exported so a caller can name `hl7_3::Error`, `hl7_3::xml::Element`,
// and the rest without adding their own dependency, the same convenience
// `hl7-3` itself extends for its XML reader.
pub use hl7_3;
