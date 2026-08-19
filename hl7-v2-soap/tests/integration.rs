//! Both directions end to end: a receiver answering a request, and a
//! sender reading the answer.

use hl7_v2_soap::response::Outcome;
use hl7_v2_soap::{Fault, message, response, wsdl};

const REQUEST: &str = include_str!("../samples/request-v2xml.xml");

/// A receiver, written the way the crate documentation says to write one.
fn receive(body: &str, allowed: &[String], authorities: &[String]) -> (u16, String) {
    match accept(body, allowed, authorities) {
        Ok(control_id) => (200, response::success(&control_id)),
        Err(fault) => (fault.status, fault.to_envelope()),
    }
}

fn accept(body: &str, allowed: &[String], authorities: &[String]) -> Result<String, Fault> {
    let envelope = hl7_v2_soap::parse(body)?;
    let payload = envelope.payload()?;
    message::check(payload, allowed, authorities)?;
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}

fn allowed() -> Vec<String> {
    vec!["ADT_A05".to_string()]
}

fn authorities() -> Vec<String> {
    vec!["252".to_string()]
}

#[test]
fn accepts_a_real_v2xml_request_and_answers_with_its_control_id() {
    let (status, body) = receive(REQUEST, &allowed(), &authorities());
    assert_eq!(status, 200);
    assert!(body.contains("<Status>Success</Status>"), "{body}");
    assert!(body.contains("202505052323300000000000"), "{body}");
    // And a sender reading that answer sees the message through.
    assert_eq!(response::evaluate(status, &body), Outcome::Accepted);
}

#[test]
fn a_structure_the_interface_does_not_take_is_a_400() {
    let (status, body) = receive(REQUEST, &["ADT_A39".to_string()], &authorities());
    assert_eq!(status, 400);
    assert!(body.contains("Unsupported HL7 message structure"), "{body}");
    assert!(!response::evaluate(status, &body).is_accepted());
}

#[test]
fn a_system_the_interface_does_not_know_is_a_403() {
    let (status, body) = receive(REQUEST, &allowed(), &["999".to_string()]);
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
fn the_sender_side_round_trips_an_er7_message() {
    let er7 = "MSH|^~\\&|APP|FAC|RECV|RECVFAC|20250101||ADT^A01|CTRL1|P|2.5\rPID|||123^^^NHS^NH\r";
    let body = message::wrap_er7(er7);

    // What a receiver on the far side would read back out. The trailing
    // segment terminator does not survive: see `message::er7`.
    let envelope = hl7_v2_soap::parse(&body).unwrap();
    let payload = envelope.payload().unwrap();
    assert_eq!(message::er7(payload), Some(er7.trim_end_matches('\r')));

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
    assert!(hl7_v2_soap::xml::parse(&document).is_ok());
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
    // so a service with existing clients has to keep serving that one — and
    // the success response has to agree with it.
    let namespace = "urn:example:hl7-soap-server";
    let wsdl = wsdl::for_address_in("https://hub.example.nhs.uk/soap", namespace);
    assert!(
        wsdl.contains(&format!(r#"targetNamespace="{namespace}""#)),
        "{wsdl}"
    );
    assert!(
        !wsdl.contains(wsdl::TARGET_NS),
        "the default must not leak in"
    );

    let reply = response::success_in("CTRL1", namespace);
    assert!(
        reply.contains(&format!(r#"xmlns:tns="{namespace}""#)),
        "{reply}"
    );
    assert_eq!(response::evaluate(200, &reply), Outcome::Accepted);
}
