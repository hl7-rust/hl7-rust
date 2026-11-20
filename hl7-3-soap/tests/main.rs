//! Both directions end to end: a receiver answering a request, and a
//! sender reading the answer.

use hl7_3_soap::response::Outcome;
use hl7_3_soap::{Fault, message, response, wsdl};

const REQUEST: &str = include_str!("../samples/request-prpa-in201305uv02.xml");

/// A receiver, written the way the crate documentation says to write one.
fn receive(body: &str, allowed: &[String], authorities: &[String]) -> (u16, String) {
    match accept(body, allowed, authorities) {
        Ok(control_id) => (200, response::success(&control_id)),
        Err(fault) => (fault.status, fault.to_envelope()),
    }
}

fn accept(body: &str, allowed: &[String], authorities: &[String]) -> Result<String, Fault> {
    let envelope = hl7_3_soap::parse(body)?;
    let payload = envelope.payload()?;
    message::check(payload, allowed, authorities)?;
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}

fn allowed() -> Vec<String> {
    vec!["PRPA_IN201305UV02".to_string()]
}

fn authorities() -> Vec<String> {
    vec!["2.16.840.1.113883.19.5".to_string()]
}

#[test]
fn accepts_a_real_v3_request_and_answers_with_its_control_id() {
    let (status, body) = receive(REQUEST, &allowed(), &authorities());
    assert_eq!(status, 200);
    assert!(body.contains(r#"<typeCode code="AA"/>"#), "{body}");
    assert!(body.contains("202505052323300000000000"), "{body}");
    // And a sender reading that answer sees the message through.
    assert_eq!(response::evaluate(status, &body), Outcome::Accepted);
}

#[test]
fn an_interaction_the_interface_does_not_take_is_a_400() {
    let (status, body) = receive(REQUEST, &["PRPA_IN201306UV02".to_string()], &authorities());
    assert_eq!(status, 400);
    assert!(body.contains("Unsupported HL7 v3 interaction"), "{body}");
    assert!(!response::evaluate(status, &body).is_accepted());
}

#[test]
fn a_system_the_interface_does_not_know_is_a_403() {
    let (status, body) = receive(REQUEST, &allowed(), &["9.9.9".to_string()]);
    assert_eq!(status, 403);
    assert!(body.contains("is not authorised"), "{body}");
}

#[test]
fn a_request_that_is_not_a_soap_envelope_is_a_400() {
    for bad in [
        "",
        "not xml",
        "<html><body>hello</body></html>",
        "<Envelope><Body/></Envelope>",
        "<Envelope><Body><A/><B/></Body></Envelope>",
    ] {
        let (status, _) = receive(bad, &[], &[]);
        assert_eq!(status, 400, "{bad:?} should be rejected as a client error");
    }
}

#[test]
fn the_sender_side_round_trips_a_wrapped_message() {
    let message = r#"<PRPA_IN201305UV02><id root="2.16.840.1.113883.19.5" extension="CTRL1"/></PRPA_IN201305UV02>"#;
    let body = hl7_3_soap::wrap_xml(message);

    let envelope = hl7_3_soap::parse(&body).unwrap();
    let payload = envelope.payload().unwrap();
    assert_eq!(message::control_id(payload), Some("CTRL1"));

    // And what the sender makes of a successful reply.
    assert_eq!(
        response::evaluate(200, &response::success("CTRL1")),
        Outcome::Accepted
    );
}

#[test]
fn a_sender_believes_a_fault_over_the_http_status() {
    let fault = Fault::validation("payload schema validation failed").to_envelope();
    match response::evaluate(200, &fault) {
        Outcome::Rejected(reason) => assert!(reason.contains("schema validation"), "{reason}"),
        other => panic!("a fault is not an acceptance: {other:?}"),
    }
}

#[test]
fn the_wsdl_describes_the_endpoint_that_served_it() {
    let document = wsdl::for_address("https://hub.example.nhs.uk/soap");
    assert!(document.contains(r#"location="https://hub.example.nhs.uk/soap""#));
    // It parses, which is the least a client tool needs of it.
    assert!(hl7_3_soap::xml::parse(&document).is_ok());
}

#[test]
fn every_fault_carries_a_status_a_sender_can_act_on() {
    // The distinction that keeps a poison message out of an infinite loop.
    assert!(!Fault::validation("x").is_retryable());
    assert!(!Fault::authorization("x").is_retryable());
    assert!(Fault::server("x").is_retryable());
    assert!(Fault::configuration("x").is_retryable());
}

#[test]
fn a_deployment_can_serve_its_own_target_namespace() {
    // Clients are generated against whatever namespace the WSDL advertised,
    // so a service with existing clients has to keep serving that one.
    let namespace = "urn:example:hl7-v3-soap-server";
    let wsdl = wsdl::for_address_in("https://hub.example.nhs.uk/soap", namespace);
    assert!(
        wsdl.contains(&format!(r#"targetNamespace="{namespace}""#)),
        "{wsdl}"
    );
    assert!(
        !wsdl.contains(wsdl::TARGET_NS),
        "the default must not leak in"
    );
}
