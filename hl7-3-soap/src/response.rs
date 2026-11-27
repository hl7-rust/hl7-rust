//! The reply a receiver sends, and how a sender reads one.
//!
//! HL7 v3 has an unambiguous answer to "did that get through?", the same
//! way v2's MSA-1 does: `acknowledgement/typeCode` inside an
//! `MCCI_IN000002UV01` (Master Information Composite / Control Act
//! acknowledgement interaction) — the real HL7 v3 acknowledgement message,
//! not a convention this crate invented. SOAP still leaves three places to
//! look and no agreement between implementations about which is
//! authoritative, so this module reads all three, in the order that cannot
//! be talked out of a rejection: the HTTP status, then a SOAP `Fault`, then
//! the acknowledgement's own `typeCode`.

use crate::xml;

/// The interaction a receiver answers with on success.
pub const ACK_INTERACTION: &str = "MCCI_IN000002UV01";

/// HL7's acknowledgement type codes that mean accepted — `AA` (application
/// accept) and `CA` (enhanced-mode commit accept). Shared with v2's MSA-1,
/// which draws from the same conceptual vocabulary.
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

/// Build the success acknowledgement for a received message.
///
/// `control_id` is the extension of the `id` the original message carried
/// ([`crate::message::control_id`]) — `targetMessage/id` echoes it back so
/// the sender can match the acknowledgement to the message it answers, the
/// same purpose MSA-2 serves in v2.
#[must_use]
pub fn success(control_id: &str) -> String {
    format!(
        concat!(
            r#"<soapenv:Envelope xmlns:soapenv="{}">"#,
            "<soapenv:Body>",
            r#"<{} xmlns="urn:hl7-org:v3">"#,
            "<acknowledgement>",
            r#"<typeCode code="AA"/>"#,
            "<targetMessage>",
            r#"<id extension="{}"/>"#,
            "</targetMessage>",
            "</acknowledgement>",
            "</{}>",
            "</soapenv:Body>",
            "</soapenv:Envelope>",
        ),
        crate::fault::SOAP_NS,
        ACK_INTERACTION,
        xml::escape(control_id),
        ACK_INTERACTION,
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
/// 3. **`acknowledgement/typeCode/@code` decides**, when the interface
///    returns one: `AA`/`CA` accepted, anything else rejected.
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

    if let Some(acknowledgement) = root.find("acknowledgement") {
        let code = acknowledgement
            .child("typeCode")
            .and_then(|type_code| type_code.attribute("code"))
            .unwrap_or_default();
        if ACCEPTED.contains(&code) {
            return Outcome::Accepted;
        }
        return Outcome::Rejected(format!("typeCode {code}"));
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
}

/// What a response positively says about one message, apart from whether the
/// transport succeeded.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Evidence {
    /// The positive `acknowledgement/typeCode/@code` found, if any. A
    /// negative code, a fault or an absent acknowledgement is `None`; use
    /// [`evaluate`] for the reason.
    pub code: Option<AckCode>,
    /// The `targetMessage/id/@extension` the acknowledgement says it
    /// answers, if it names one.
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
    // Exactly one `acknowledgement` inside the SOAP `Body`: the code and the
    // target message must come from the same one, and several would leave it
    // open which of them answers the message.
    let scope = root.find("Body").unwrap_or(&root);
    let mut found = Vec::new();
    acknowledgements(scope, &mut found);
    let [acknowledgement] = found.as_slice() else {
        return Evidence::default();
    };
    let code = match acknowledgement
        .child("typeCode")
        .and_then(|type_code| type_code.attribute("code"))
    {
        Some("AA") => Some(AckCode::Application),
        Some("CA") => Some(AckCode::Commit),
        _ => None,
    };
    let control_id = acknowledgement
        .child("targetMessage")
        .and_then(|target| target.child("id"))
        .and_then(|id| id.attribute("extension"))
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string);
    Evidence { code, control_id }
}

/// Every `acknowledgement` element under `element`, itself included.
fn acknowledgements<'a>(element: &'a xml::Element, found: &mut Vec<&'a xml::Element>) {
    if element.local_name() == "acknowledgement" {
        found.push(element);
    }
    for child in &element.children {
        acknowledgements(child, found);
    }
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
/// - its `targetMessage/id` names `expected` as the message it answers.
///
/// Anything else is a rejection with a reason that never quotes the
/// message's own text.
#[must_use]
pub fn evaluate_correlated(status: u16, body: &str, expected: &str) -> Outcome {
    if !(200..300).contains(&status) {
        return Outcome::Rejected(format!("HTTP {status}"));
    }
    // `evaluate`'s reasons quote the peer's fault and status text, which can
    // echo the message; this API's reasons are fixed strings.
    if xml::parse(body).is_ok_and(|root| root.find("Fault").is_some()) {
        return Outcome::Rejected("SOAP fault".to_string());
    }
    if !evaluate(status, body).is_accepted() {
        return Outcome::Rejected("negative acknowledgement in the response".to_string());
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
        assert_eq!(root.find("typeCode").unwrap().attribute("code"), Some("AA"));
        assert_eq!(
            root.find("targetMessage")
                .unwrap()
                .child("id")
                .unwrap()
                .attribute("extension"),
            Some("CTRL7")
        );
        assert_eq!(evaluate(200, &body), Outcome::Accepted);
    }

    #[test]
    fn a_control_id_cannot_break_out_of_the_response() {
        let body = success("a<b>&c");
        assert!(body.contains("a&lt;b&gt;&amp;c"));
        assert_eq!(
            xml::parse(&body)
                .unwrap()
                .find("targetMessage")
                .unwrap()
                .child("id")
                .unwrap()
                .attribute("extension"),
            Some("a<b>&c")
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
        let body = crate::Fault::validation("bad interaction").to_envelope();
        assert_eq!(
            evaluate(200, &body),
            Outcome::Rejected("Client.Validation: bad interaction".into())
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
    fn an_hl7_acknowledgement_type_code_decides_when_one_is_returned() {
        let with_code = |code: &str| {
            format!(
                "<Envelope><Body><MCCI_IN000002UV01><acknowledgement>\
                 <typeCode code=\"{code}\"/></acknowledgement></MCCI_IN000002UV01></Body></Envelope>"
            )
        };
        assert!(evaluate(200, &with_code("AA")).is_accepted());
        assert!(evaluate(200, &with_code("CA")).is_accepted());
        assert!(!evaluate(200, &with_code("AE")).is_accepted());
        assert!(!evaluate(200, &with_code("AR")).is_accepted());
        assert_eq!(
            evaluate(200, &with_code("AE")),
            Outcome::Rejected("typeCode AE".into())
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

    fn ack(code: &str, id: &str) -> String {
        format!(
            "<Envelope><Body><MCCI_IN000002UV01><acknowledgement><typeCode code=\"{code}\"/>\
             <targetMessage><id extension=\"{id}\"/></targetMessage></acknowledgement>\
             </MCCI_IN000002UV01></Body></Envelope>"
        )
    }

    #[test]
    fn only_a_positive_acknowledgement_naming_the_message_is_accepted() {
        assert_eq!(
            evaluate_correlated(200, &success("M1"), "M1"),
            Outcome::Accepted
        );
        assert_eq!(
            evaluate_correlated(200, &ack("CA", "M1"), "M1"),
            Outcome::Accepted
        );
        assert_eq!(
            evaluate_correlated(200, &ack("AA", "OTHER"), "M1"),
            Outcome::Rejected("the acknowledgement answers a different message".into())
        );
        assert_eq!(
            evaluate_correlated(200, &ack("AA", ""), "M1"),
            Outcome::Rejected("the acknowledgement does not name a message".into())
        );
        assert!(!evaluate_correlated(200, &ack("AE", "M1"), "M1").is_accepted());
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
        assert!(evaluate(200, "").is_accepted());
        let fault = crate::Fault::validation("bad").to_envelope();
        assert!(!evaluate_correlated(200, &fault, "M1").is_accepted());
        assert!(!evaluate_correlated(503, &success("M1"), "M1").is_accepted());
    }

    #[test]
    fn evidence_distinguishes_commit_from_application_acceptance() {
        assert_eq!(evidence(&ack("AA", "M1")).code, Some(AckCode::Application));
        assert_eq!(evidence(&ack("CA", "M1")).code, Some(AckCode::Commit));
        assert_eq!(evidence(&ack("AR", "M1")).code, None);
        assert_eq!(evidence(&ack("AA", "M1")).control_id.as_deref(), Some("M1"));
        assert_eq!(evidence("").control_id, None);
    }

    #[test]
    fn evidence_comes_from_one_acknowledgement_in_the_body() {
        let none = Outcome::Rejected("no positive acknowledgement in the response".into());
        let one = "<acknowledgement><typeCode code=\"AA\"/>\
                   <targetMessage><id extension=\"M1\"/></targetMessage></acknowledgement>";
        let other = "<acknowledgement><typeCode code=\"AA\"/>\
                     <targetMessage><id extension=\"M2\"/></targetMessage></acknowledgement>";
        // Two acknowledgements, so neither is the answer.
        let two = format!(
            "<Envelope><Body><MCCI_IN000002UV01>{one}{other}</MCCI_IN000002UV01></Body></Envelope>"
        );
        assert_eq!(evaluate_correlated(200, &two, "M1"), none);
        // An acknowledgement outside the body is no evidence.
        let outside = format!("<Envelope><Header>{one}</Header><Body/></Envelope>");
        assert_eq!(evaluate_correlated(200, &outside, "M1"), none);
        assert_eq!(evidence(one).control_id.as_deref(), Some("M1"));
    }

    #[test]
    fn correlated_reasons_never_quote_the_peer() {
        let fault = crate::Fault::validation("PATIENTMARK is wrong").to_envelope();
        assert!(
            matches!(evaluate(200, &fault), Outcome::Rejected(reason) if reason.contains("MARK"))
        );
        assert_eq!(
            evaluate_correlated(200, &fault, "M1"),
            Outcome::Rejected("SOAP fault".into())
        );
        assert_eq!(
            evaluate_correlated(200, &ack("MARK", "M1"), "M1"),
            Outcome::Rejected("negative acknowledgement in the response".into())
        );
        assert_eq!(
            evaluate_correlated(503, "", "M1"),
            Outcome::Rejected("HTTP 503".into())
        );
    }
}
