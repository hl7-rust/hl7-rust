//! Health Level Seven (HL7) version 3 (V3) for Rust — a foundation, not a
//! complete implementation.
//!
//! HL7 v3 replaced v2's pipe-delimited text and per-message-type segment
//! tables with one thing reused everywhere: the [Reference Information
//! Model](rim) (RIM), six backbone classes ([`rim::Act`], [`rim::Entity`],
//! [`rim::Role`], [`rim::Participation`], [`rim::ActRelationship`],
//! [`rim::RoleLink`]) that every domain payload — lab results, care
//! records, structured product labeling — is assembled from, serialized as
//! XML instead of ER7. It achieved little messaging adoption of its own
//! (implementers found the model-driven rigor expensive to work with) but
//! its RIM and three-level structure live on directly inside the Clinical
//! Document Architecture (CDA), which did succeed.
//!
//! ```
//! use hl7_3::message;
//!
//! let xml = r#"
//! <QUQI_IN000001UV01 xmlns="urn:hl7-org:v3">
//!   <id root="2.16.840.1.113883.19.5" extension="MSG00001"/>
//!   <interactionId root="2.16.840.1.113883.1.6" extension="QUQI_IN000001UV01"/>
//!   <controlActProcess classCode="CACT" moodCode="EVN">
//!     <code code="QUQI_TE000001UV01"/>
//!     <subject>
//!       <observation classCode="OBS" moodCode="EVN"/>
//!     </subject>
//!   </controlActProcess>
//! </QUQI_IN000001UV01>
//! "#;
//! let parsed = message::parse(xml)?;
//! assert_eq!(parsed.control_act.unwrap().domain.unwrap().local_name(), "observation");
//! # Ok::<(), hl7_3::Error>(())
//! ```
//!
//! ## What this crate is, and is not
//!
//! It is: the RIM backbone classes as Rust types ([`rim`]), the data types
//! RIM attributes are built from — identifiers, coded values, intervals,
//! quantities, encapsulated data, and the explicit-null mechanism any of
//! them can carry instead of a value ([`vocabulary`]) — and a reader for
//! the three-level message envelope every interaction shares ([`message`])
//! — transport wrapper, control act wrapper, domain payload.
//!
//! It is not: a validator against any of HL7 v3's vocabulary domains or
//! interaction schemas, a CDA document model, or a decoder for any
//! specific domain payload's internal shape (a lab result, a care record)
//! — those vary per interaction and are read with [`rim`] types by the
//! caller, the same way generic mode in
//! [`hl7-2`](https://crates.io/crates/hl7-2) hands back a tree rather than
//! a typed message. `spec/index.md` is the exact, current statement of
//! scope; where this comment and that document disagree, the document is
//! right.
//!
//! For a stable, long-lived interaction, [`typed::FromElement`] — derived
//! with `#[derive(FromElement)]` behind the `derive` feature
//! ([`hl7-3-derive`](https://crates.io/crates/hl7-3-derive)) — maps a
//! struct's fields onto an element's attributes and children once, the way
//! `hl7-2`'s struct mode does for v2 paths. See [`typed`] for the
//! attributes and an example.

#![warn(missing_docs, clippy::pedantic)]

pub mod message;
pub mod rim;
pub mod typed;
pub mod vocabulary;

pub use message::{ControlAct, Message};
pub use typed::{FromElement, FromElementValue};
pub use vocabulary::{Cd, Ed, Ii, Ivl, NullFlavor, Pq};

/// The `#[derive(FromElement)]` macro, re-exported so the `hl7-3-derive`
/// crate does not have to be named as a dependency. Requires the `derive`
/// feature. (Same name as the [`typed::FromElement`] trait, deliberately —
/// like `hl7-2`'s `FromHl7`, Rust keeps a derive macro and a trait of the
/// same name in separate namespaces, so `#[derive(FromElement)]` and `impl
/// FromElement` never conflict.)
#[cfg(feature = "derive")]
pub use hl7_3_derive::FromElement;

/// The XML reader this crate reads HL7 v3 messages through, re-exported so
/// callers can name [`xml::Element`] without adding their own dependency.
pub use hl7_2_xml_lite_helper as xml;

/// What can go wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input is not well-formed XML.
    Xml(hl7_2_xml_lite_helper::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Xml(error) => write!(f, "not well-formed XML: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<hl7_2_xml_lite_helper::Error> for Error {
    fn from(error: hl7_2_xml_lite_helper::Error) -> Error {
        Error::Xml(error)
    }
}
