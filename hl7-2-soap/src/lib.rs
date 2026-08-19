//! HL7 v2 over SOAP: the envelope, faults, payload carriage, WSDL, and
//! response evaluation that carry HL7 v2 messages over HTTP.
//!
//! MLLP is how HL7 v2 usually moves, and `hl7-2-mllp` is that transport.
//! SOAP is the other one — the transport an estate ends up with when the
//! messages have to cross a boundary that speaks HTTP, or when the system
//! at the far end was built by a team who had a WSDL and no socket. This
//! crate is that transport, and it is deliberately the same shape as its
//! MLLP sibling: it does the protocol and nothing else.
//!
//! # What it does
//!
//! - [`parse`] a SOAP envelope and take the single payload out of its body
//! - [`Fault`]s, each carrying the HTTP status that belongs with it
//! - [`message`] — read a v2.xml payload, or ER7 wrapped in one, and check
//!   a payload against what the interface accepts
//! - [`response`] — build the reply, and read one as accepted or rejected
//! - [`wsdl`] — describe the endpoint to client tooling, at its real address
//!
//! # What it does not do
//!
//! No HTTP client and no HTTP server: this crate turns bytes into meaning
//! and back, and leaves the socket to whatever the caller already uses.
//! No HL7 validation and no format conversion either — `hl7-rust` and the
//! `hl7-2-from-*` crates own those, and a transport that also converted
//! formats would be two crates in a trench coat.
//!
//! # Receiving
//!
//! ```
//! use hl7_2_soap::{Fault, message, response};
//!
//! fn handle(request_body: &str) -> (u16, String) {
//!     match accept(request_body) {
//!         Ok(control_id) => (200, response::success(&control_id)),
//!         Err(fault) => (fault.status, fault.to_envelope()),
//!     }
//! }
//!
//! fn accept(request_body: &str) -> Result<String, Fault> {
//!     let envelope = hl7_2_soap::parse(request_body)?;
//!     let payload = envelope.payload()?;
//!     message::check(payload, &["ADT_A05".to_string()], &[])?;
//!     // ...validate and forward the payload here...
//!     Ok(message::control_id(payload).unwrap_or_default().to_string())
//! }
//!
//! let request = r#"<Envelope><Body><ADT_A05><MSH><MSH.10>9</MSH.10></MSH></ADT_A05></Body></Envelope>"#;
//! assert_eq!(handle(request).0, 200);
//!
//! let wrong = r#"<Envelope><Body><ADT_A39/></Body></Envelope>"#;
//! assert_eq!(handle(wrong).0, 400);
//! ```
//!
//! # Sending
//!
//! ```
//! use hl7_2_soap::{message, response::{self, Outcome}};
//!
//! let body = message::wrap_er7("MSH|^~\\&|APP||||1||ADT^A01|9|P|2.5");
//! // ...POST `body` with Content-Type: text/xml; charset=utf-8...
//! # let (status, reply) = (200, response::success("9"));
//! match response::evaluate(status, &reply) {
//!     Outcome::Accepted => {}
//!     Outcome::Rejected(reason) => panic!("not delivered: {reason}"),
//! }
//! ```
//!
//! See `spec/index.md` for the exact rules (source of truth).

#![warn(missing_docs, clippy::pedantic)]
// XML literals keep their `r#"..."#` delimiters even where no `"` currently
// forces them: these are documents, and adding a quoted attribute to one
// should not also mean changing its delimiter.
#![allow(clippy::needless_raw_string_hashes)]

pub mod envelope;
pub mod fault;
pub mod message;
pub mod response;
pub mod wsdl;

/// The XML reader this crate is built on, re-exported so callers can name
/// [`xml::Element`] and walk a payload themselves without adding their own
/// dependency.
///
/// `hl7-2-xml-lite-helper` has no dependencies of its own, and is shared with the other
/// crates in this family that read XML, so there is one parser to audit
/// rather than one per crate.
pub use hl7_2_xml_lite_helper as xml;

pub use envelope::{Envelope, parse, wrap_xml};
pub use fault::{Fault, SOAP_NS};
pub use response::Outcome;

/// The content type a SOAP 1.1 request and response are sent with.
///
/// SOAP 1.1 uses `text/xml`; SOAP 1.2 would use `application/soap+xml`.
/// This crate speaks 1.1, which is what the HL7 interfaces in the field
/// were written against.
pub const CONTENT_TYPE: &str = "text/xml; charset=utf-8";
