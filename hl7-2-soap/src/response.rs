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
            .map_or_default(|element| element.text.trim());
        let reason = fault
            .child("faultstring")
            .map_or_default(|element| element.text.trim());
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

/// Which positive acknowledgement a receiver gave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckCode {
    /// `AA`: the application accepted the message.
    Application,
    /// `CA`: the receiver committed to it (enhanced mode) but has not
    /// necessarily processed it.
    Commit,
    /// The word `Success`, which [`success`] writes.
    Success,
}

/// What a response positively says about one message, apart from whether the
/// transport succeeded.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Evidence {
    /// The positive acknowledgement found, if any. A negative code, a fault
    /// or an absent `Status` is `None`; use [`evaluate`] for the reason.
    pub code: Option<AckCode>,
    /// The message control ID the response says it answers, if it names one.
    pub control_id: Option<String>,
}

/// Read the acknowledgement evidence out of a response body.
///
/// Unlike [`evaluate`], an empty or unrecognised body is *not* accepted
/// here: it yields no evidence.
#[must_use]
pub fn evidence(body: &str) -> Evidence {
    let Ok(root) = xml::parse(body) else {
        return Evidence::default();
    };
    let code = root
        .find("Status")
        .and_then(|element| match element.text.trim() {
            "AA" => Some(AckCode::Application),
            "CA" => Some(AckCode::Commit),
            value if value.eq_ignore_ascii_case("Success") => Some(AckCode::Success),
            _ => None,
        });
    let control_id = root
        .find("MessageControlId")
        .map(|element| element.text.trim().to_string())
        .filter(|id| !id.is_empty());
    Evidence { code, control_id }
}

/// Decide whether a response is a *correlated* acknowledgement of the
/// message whose control ID is `expected`.
///
/// [`evaluate`] believes a bare 2xx, which suits a sender that only needs
/// the transport to have worked. A sender that clears an outbound queue on
/// the answer needs more, so this accepts only when all of these hold:
///
/// - [`evaluate`] accepts the response (no failing status, no fault);
/// - the body carries a positive acknowledgement ([`evidence`]); and
/// - it names `expected` as the message it answers.
///
/// Anything else is a rejection with a reason that never quotes the
/// message's own text.
#[must_use]
pub fn evaluate_correlated(status: u16, body: &str, expected: &str) -> Outcome {
    let outcome = evaluate(status, body);
    if !outcome.is_accepted() {
        return outcome;
    }
    let found = evidence(body);
    if found.code.is_none() {
        return Outcome::Rejected("no positive acknowledgement in the response".to_string());
    }
    match found.control_id.as_deref() {
        None => Outcome::Rejected("the acknowledgement does not name a message".to_string()),
        Some(id) if id == expected => Outcome::Accepted,
        Some(_) => Outcome::Rejected("the acknowledgement answers a different message".to_string()),
    }
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
        assert_eq!(
            evaluate(200, "<Envelope><Body><Fault/></Body></Envelope>"),
            Outcome::Rejected("SOAP fault".into())
        );
        assert_eq!(
            evaluate(
                200,
                "<Envelope><Body><Fault><faultstring>reason</faultstring></Fault></Body></Envelope>",
            ),
            Outcome::Rejected("reason".into())
        );
        assert_eq!(
            evaluate(
                200,
                "<Envelope><Body><Fault><faultcode>Code</faultcode></Fault></Body></Envelope>",
            ),
            Outcome::Rejected("Code".into())
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

#[cfg(test)]
mod correlated_tests {
    use super::*;

    #[test]
    fn only_a_positive_acknowledgement_naming_the_message_is_accepted() {
        assert_eq!(
            evaluate_correlated(200, &success("M1"), "M1"),
            Outcome::Accepted
        );
        let with = |status: &str, id: &str| {
            format!(
                "<Envelope><Body><Ack><Status>{status}</Status>\
                 <MessageControlId>{id}</MessageControlId></Ack></Body></Envelope>"
            )
        };
        assert_eq!(
            evaluate_correlated(200, &with("AA", "M1"), "M1"),
            Outcome::Accepted
        );
        assert_eq!(
            evaluate_correlated(200, &with("CA", "M1"), "M1"),
            Outcome::Accepted
        );
        // A reply to a different message must not retire this one, and the
        // reason must not quote either control ID.
        let other = evaluate_correlated(200, &with("AA", "OTHER"), "M1");
        assert_eq!(
            other,
            Outcome::Rejected("the acknowledgement answers a different message".into())
        );
        assert_eq!(
            evaluate_correlated(200, &with("AA", ""), "M1"),
            Outcome::Rejected("the acknowledgement does not name a message".into())
        );
        assert!(!evaluate_correlated(200, &with("AE", "M1"), "M1").is_accepted());
    }

    #[test]
    fn silence_a_fault_and_a_failing_status_are_not_correlated_acknowledgements() {
        let none = Outcome::Rejected("no positive acknowledgement in the response".into());
        assert_eq!(evaluate_correlated(200, "", "M1"), none);
        assert_eq!(
            evaluate_correlated(200, "<Envelope><Body/></Envelope>", "M1"),
            none
        );
        assert_eq!(evaluate_correlated(200, "not xml <", "M1"), none);
        // The compatibility evaluator still believes the empty 200.
        assert!(evaluate(200, "").is_accepted());
        let fault = crate::Fault::validation("bad").to_envelope();
        assert!(!evaluate_correlated(200, &fault, "M1").is_accepted());
        assert!(!evaluate_correlated(503, &success("M1"), "M1").is_accepted());
    }

    #[test]
    fn evidence_distinguishes_commit_from_application_acceptance() {
        let code = |status: &str| {
            evidence(&format!(
                "<Envelope><Body><Status>{status}</Status></Body></Envelope>"
            ))
            .code
        };
        assert_eq!(code("AA"), Some(AckCode::Application));
        assert_eq!(code("CA"), Some(AckCode::Commit));
        assert_eq!(code("success"), Some(AckCode::Success));
        assert_eq!(code("AR"), None);
        assert_eq!(evidence("").control_id, None);
    }
}
