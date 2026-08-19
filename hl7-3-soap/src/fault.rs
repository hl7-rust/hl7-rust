//! SOAP faults, and the HTTP status that goes with them.
//!
//! A fault is how a SOAP endpoint says no. It carries a code the caller can
//! branch on and a sentence a human can read, and — because SOAP rides on
//! HTTP — a status, which is what a load balancer, a retry policy and a
//! dashboard actually look at.
//!
//! The pairing matters more than it looks. A rejection the sender must fix
//! is a 400 and must not be retried; a rejection the sender is not
//! permitted to make is a 403; a failure on the receiving side is a 500 and
//! *should* be retried. Getting that wrong turns a poison message into an
//! infinite loop, or loses a message that would have gone through a moment
//! later.

use crate::xml;
use std::fmt;

/// The SOAP 1.1 envelope namespace.
pub const SOAP_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";

/// A refusal to process a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The fault code, e.g. `Client.Validation`. SOAP 1.1 spells the two
    /// top-level codes `Client` and `Server`; a dotted suffix narrows one.
    pub code: String,
    /// What went wrong, in a sentence, for a person reading a log.
    pub reason: String,
    /// The HTTP status to answer with.
    pub status: u16,
}

impl Fault {
    /// A fault with every part stated.
    #[must_use]
    pub fn new(code: impl Into<String>, reason: impl Into<String>, status: u16) -> Fault {
        Fault {
            code: code.into(),
            reason: reason.into(),
            status,
        }
    }

    /// The request is wrong and will be wrong again: `Client`, HTTP 400.
    #[must_use]
    pub fn client(reason: impl Into<String>) -> Fault {
        Fault::new("Client", reason, 400)
    }

    /// The request is well formed but does not satisfy a rule:
    /// `Client.Validation`, HTTP 400.
    #[must_use]
    pub fn validation(reason: impl Into<String>) -> Fault {
        Fault::new("Client.Validation", reason, 400)
    }

    /// The sender is not permitted: `Client.Authorization`, HTTP 403.
    #[must_use]
    pub fn authorization(reason: impl Into<String>) -> Fault {
        Fault::new("Client.Authorization", reason, 403)
    }

    /// Something failed on this side: `Server`, HTTP 500. The only kind
    /// worth retrying.
    #[must_use]
    pub fn server(reason: impl Into<String>) -> Fault {
        Fault::new("Server", reason, 500)
    }

    /// This endpoint is misconfigured: `Server.Configuration`, HTTP 500.
    #[must_use]
    pub fn configuration(reason: impl Into<String>) -> Fault {
        Fault::new("Server.Configuration", reason, 500)
    }

    /// Whether the sender should try again. True only for a `Server` fault:
    /// a `Client` fault repeated is the same fault.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        self.status >= 500
    }

    /// The fault as a SOAP 1.1 envelope, ready to be the response body.
    #[must_use]
    pub fn to_envelope(&self) -> String {
        format!(
            concat!(
                r#"<soapenv:Envelope xmlns:soapenv="{}">"#,
                "<soapenv:Body>",
                "<soapenv:Fault>",
                "<faultcode>{}</faultcode>",
                "<faultstring>{}</faultstring>",
                "</soapenv:Fault>",
                "</soapenv:Body>",
                "</soapenv:Envelope>",
            ),
            SOAP_NS,
            xml::escape(&self.code),
            xml::escape(&self.reason),
        )
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}): {}", self.code, self.status, self.reason)
    }
}

impl std::error::Error for Fault {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_carries_the_status_that_matches_it() {
        assert_eq!(Fault::client("x").status, 400);
        assert_eq!(Fault::validation("x").status, 400);
        assert_eq!(Fault::authorization("x").status, 403);
        assert_eq!(Fault::server("x").status, 500);
        assert_eq!(Fault::configuration("x").status, 500);
    }

    #[test]
    fn only_a_server_fault_is_worth_retrying() {
        assert!(!Fault::client("x").is_retryable());
        assert!(!Fault::authorization("x").is_retryable());
        assert!(Fault::server("x").is_retryable());
    }

    #[test]
    fn a_reason_cannot_break_out_of_the_envelope() {
        let fault = Fault::client(r#"bad <tag> & "quote""#);
        let envelope = fault.to_envelope();
        assert!(envelope.contains("bad &lt;tag&gt; &amp; &quot;quote&quot;"));
        // Still one well-formed document.
        let root = xml::parse(&envelope).unwrap();
        assert_eq!(root.local_name(), "Envelope");
    }

    #[test]
    fn reads_back_as_the_fault_it_describes() {
        let envelope = Fault::validation("no good").to_envelope();
        let root = xml::parse(&envelope).unwrap();
        let fault = root.find("Fault").unwrap();
        assert_eq!(fault.child("faultcode").unwrap().text, "Client.Validation");
        assert_eq!(fault.child("faultstring").unwrap().text, "no good");
    }
}
