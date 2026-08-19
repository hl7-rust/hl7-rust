//! The reply a receiver sends, and how a sender reads one.
//!
//! MLLP has an unambiguous answer to "did that get through?": the ACK code
//! in MSA-1. SOAP has three places to look and no agreement between
//! implementations about which is authoritative — the HTTP status, a SOAP
//! `Fault`, and whatever the interface put in the body. This module reads
//! all three, in the order that cannot be talked out of a rejection.

use crate::fault::SOAP_NS;
use crate::xml;

/// The response element a receiver returns on success.
pub const ACK_ELEMENT: &str = "AckResponse";

/// HL7 acknowledgement codes that mean accepted.
const ACCEPTED: [&str; 2] = ["AA", "CA"];

/// What a response says happened to the message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The receiver has the message.
    Accepted,
    /// The receiver does not, and says why.
    Rejected(String),
}

impl Outcome {
    /// Whether the message got through.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        matches!(self, Outcome::Accepted)
    }
}

/// Build the success response for a received message, in the default
/// target namespace ([`crate::wsdl::TARGET_NS`]).
#[must_use]
pub fn success(control_id: &str) -> String {
    success_in(control_id, crate::wsdl::TARGET_NS)
}

/// Build the success response in a target namespace of your own.
///
/// It has to match the one the WSDL advertises ([`crate::wsdl::for_address_in`]),
/// because that is what the client was generated against.
#[must_use]
pub fn success_in(control_id: &str, target_namespace: &str) -> String {
    format!(
        concat!(
            r#"<soapenv:Envelope xmlns:soapenv="{}" xmlns:tns="{}">"#,
            "<soapenv:Body>",
            "<tns:{}>",
            "<Status>Success</Status>",
            "<MessageControlId>{}</MessageControlId>",
            "</tns:{}>",
            "</soapenv:Body>",
            "</soapenv:Envelope>",
        ),
        SOAP_NS,
        xml::escape(target_namespace),
        ACK_ELEMENT,
        xml::escape(control_id),
        ACK_ELEMENT,
    )
}

/// Decide what an HTTP response says about the message.
///
/// The order is the point:
///
/// 1. **A non-success status is a rejection.** Whatever the body says, the
///    transport already said no.
/// 2. **A `Fault` element is a rejection**, even under HTTP 200. Some
///    stacks answer 200 and put the refusal in the body; believing the
///    status there would lose messages silently.
/// 3. **A `Status` element decides**, when the interface returns one:
///    `AA`/`CA` accepted, `Success` accepted, anything else rejected.
/// 4. **Otherwise accepted.** A bare 200 with an empty or unrecognised body
///    is what a minimal endpoint returns, and treating that as failure
///    would resend every message forever.
#[must_use]
pub fn evaluate(status: u16, body: &str) -> Outcome {
    if !(200..300).contains(&status) {
        return Outcome::Rejected(format!("HTTP {status}"));
    }

    let Ok(root) = xml::parse(body) else {
        // Unparseable under a success status: nothing here contradicts the
        // transport, and rule 4 applies.
        return Outcome::Accepted;
    };

    if let Some(fault) = root.find("Fault") {
        let code = fault
            .child("faultcode")
            .map(|element| element.text.trim())
            .unwrap_or_default();
        let reason = fault
            .child("faultstring")
            .map(|element| element.text.trim())
            .unwrap_or_default();
        return Outcome::Rejected(match (code.is_empty(), reason.is_empty()) {
            (true, true) => "SOAP fault".to_string(),
            (true, false) => reason.to_string(),
            (false, true) => code.to_string(),
            (false, false) => format!("{code}: {reason}"),
        });
    }

    if let Some(element) = root.find("Status") {
        let value = element.text.trim();
        if ACCEPTED.contains(&value) || value.eq_ignore_ascii_case("Success") {
            return Outcome::Accepted;
        }
        return Outcome::Rejected(format!("Status {value}"));
    }

    Outcome::Accepted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_success_response_names_the_message_it_answers() {
        let body = success("CTRL7");
        let root = xml::parse(&body).unwrap();
        assert_eq!(root.find("Status").unwrap().text, "Success");
        assert_eq!(root.find("MessageControlId").unwrap().text, "CTRL7");
        assert_eq!(evaluate(200, &body), Outcome::Accepted);
    }

    #[test]
    fn a_control_id_cannot_break_out_of_the_response() {
        let body = success("a<b>&c");
        assert!(body.contains("a&lt;b&gt;&amp;c"));
        assert_eq!(
            xml::parse(&body)
                .unwrap()
                .find("MessageControlId")
                .unwrap()
                .text,
            "a<b>&c"
        );
    }

    #[test]
    fn a_failing_status_is_a_rejection_whatever_the_body_says() {
        assert_eq!(
            evaluate(500, &success("1")),
            Outcome::Rejected("HTTP 500".into())
        );
        assert!(!evaluate(404, "").is_accepted());
    }

    #[test]
    fn a_fault_under_http_200_is_still_a_rejection() {
        let body = crate::Fault::validation("bad structure").to_envelope();
        assert_eq!(
            evaluate(200, &body),
            Outcome::Rejected("Client.Validation: bad structure".into())
        );
    }

    #[test]
    fn an_hl7_acknowledgement_code_decides_when_one_is_returned() {
        let with_status = |value: &str| {
            format!("<Envelope><Body><Ack><Status>{value}</Status></Ack></Body></Envelope>")
        };
        assert!(evaluate(200, &with_status("AA")).is_accepted());
        assert!(evaluate(200, &with_status("CA")).is_accepted());
        assert!(!evaluate(200, &with_status("AE")).is_accepted());
        assert!(!evaluate(200, &with_status("AR")).is_accepted());
        assert_eq!(
            evaluate(200, &with_status("AE")),
            Outcome::Rejected("Status AE".into())
        );
    }

    #[test]
    fn a_minimal_endpoint_that_says_nothing_is_believed() {
        assert!(evaluate(200, "").is_accepted());
        assert!(evaluate(202, "OK").is_accepted());
        assert!(evaluate(200, "<Envelope><Body/></Envelope>").is_accepted());
    }
}
