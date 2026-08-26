# hl7-3-soap

> HL7® is the registered trademark of Health Level Seven International, and we are requesting permission to use it here. Use of the HL7 trademark does not constitute endorsement of this library by HL7.

HL7 v3 over SOAP: the envelope, faults, message carriage, WSDL, and
acknowledgement evaluation that carry Health Level Seven version 3
messages over HTTP.

Unlike v2 — where MLLP is the usual transport and SOAP is the exception —
SOAP *is* HL7 v3's own historically dominant transport: v3 was designed
alongside SOAP/WS-*, and real deployments (NHS England's Personal
Demographics Service, IHE profiles built on v3) carry it that way. This
crate is that transport, and it is deliberately the same shape as its
[`hl7-2-soap`](https://crates.io/crates/hl7-2-soap) cousin: it does the
protocol and nothing else.

## What it does

- Parse a SOAP envelope and take the single business payload out of its
  body — a complete HL7 v3 message, root element named for the interaction
- Faults, each carrying the HTTP status that belongs with it
- Read which interaction a payload is, its control ID, and its claimed
  assigning authority, and check a payload against what the interface
  accepts
- Build the real HL7 v3 acknowledgement (`MCCI_IN000002UV01`,
  `acknowledgement/typeCode`), and read one as accepted or rejected
- Serve a WSDL that describes the endpoint at the address it was reached on

## What it does not do

No HTTP client and no HTTP server: it turns bytes into meaning and back,
and leaves the socket to whatever you already use. No RIM decoding and no
domain-payload interpretation either — `hl7-3` owns those.

## Receiving

```rust
use hl7_3_soap::{Fault, message, response};

fn handle(request_body: &str) -> (u16, String) {
    match accept(request_body) {
        Ok(control_id) => (200, response::success(&control_id)),
        Err(fault) => (fault.status, fault.to_envelope()),
    }
}

fn accept(request_body: &str) -> Result<String, Fault> {
    let envelope = hl7_3_soap::parse(request_body)?;
    let payload = envelope.payload()?;
    message::check(payload, &["PRPA_IN201305UV02".to_string()], &[])?;
    // ...decode the payload with hl7-3, and forward it, here...
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}
```

## Sending

```rust
use hl7_3_soap::{envelope, response::{self, Outcome}};

let body = envelope::wrap_xml(r#"<PRPA_IN201305UV02><id extension="9"/></PRPA_IN201305UV02>"#);
// ...POST `body` with Content-Type: text/xml; charset=utf-8...
# let (status, reply) = (200, response::success("9"));
match response::evaluate(status, &reply) {
    Outcome::Accepted => {}
    Outcome::Rejected(reason) => eprintln!("not delivered: {reason}"),
}
```

## Three things it is opinionated about

**One payload per body.** SOAP permits several; no HL7 interface means
several. A body with none or many is a fault rather than a silent choice of
the first child.

**Prefixes do not matter.** The same envelope arrives as `soapenv:`,
`soap:`, `SOAP-ENV:` or unprefixed depending on which stack sent it.
Elements are matched on their local name, so all four are read alike.

**A fault carries its HTTP status.** `Client` is a 400 and must not be
retried; `Client.Authorization` is a 403; `Server` is a 500 and should be.
Getting that pairing wrong is how a poison message becomes an infinite
retry loop, or how a message that would have gone through a moment later is
dropped instead.

## Reading a response

`response::evaluate` reads the same three places `hl7-2-soap` does, in the
order that cannot be talked out of a rejection: a non-2xx status, then a
`Fault` element (even under HTTP 200, because some stacks answer 200 and
put the refusal in the body), then `acknowledgement/typeCode/@code`, then
acceptance. `AA` and `CA` are accepted, matching v2's MSA-1 — the two
standards draw the acceptance codes from the same conceptual vocabulary.

## Dependencies

One: `hl7-2-xml-lite-helper`, re-exported here as `xml`, shared with
`hl7-2-soap` and the other crates in the family that read XML. It has no
dependencies of its own, so the audit surface stays small. This crate
depends on neither `hl7-3` nor any RIM decoding — a SOAP envelope is XML,
and reading one requires no HL7 knowledge beyond the names of a few
elements.

## Specification

`spec/index.md` is the source of truth for what this crate does. If it and
the README disagree, the spec wins.

## License

Licensed under any of MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or
GPL-3.0-only, at your option.
