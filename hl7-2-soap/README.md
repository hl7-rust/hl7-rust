# hl7-2-soap

> HL7® is the registered trademark of Health Level Seven International, and we are requesting permission to use it here. Use of the HL7 trademark does not constitute endorsement of this library by HL7.

HL7 v2 over SOAP: the envelope, faults, payload carriage, WSDL, and response
evaluation that carry Health Level Seven version 2 messages over HTTP.

MLLP is how HL7 v2 usually moves, and
[`hl7-2-mllp`](https://crates.io/crates/hl7-2-mllp) is that transport.
SOAP is the other one — what an estate ends up with when the messages have
to cross a boundary that speaks HTTP, or when the system at the far end was
built by a team who had a WSDL and no socket. This crate is that transport,
and it is deliberately the same shape as its MLLP sibling: it does the
protocol and nothing else.

## What it does

- Parse a SOAP envelope and take the single business payload out of its body
- Faults, each carrying the HTTP status that belongs with it
- Read a v2.xml payload, or ER7 wrapped in one, and check a payload against
  what the interface accepts
- Build the reply, and read one as accepted or rejected
- Serve a WSDL that describes the endpoint at the address it was reached on

## What it does not do

No HTTP client and no HTTP server: it turns bytes into meaning and back, and
leaves the socket to whatever you already use. No HL7 validation and no
format conversion either — `hl7-2` and the `hl7-2-from-*` crates own
those.

## Receiving

```rust
use hl7_2_soap::{Fault, message, response};

fn handle(request_body: &str) -> (u16, String) {
    match accept(request_body) {
        Ok(control_id) => (200, response::success(&control_id)),
        Err(fault) => (fault.status, fault.to_envelope()),
    }
}

fn accept(request_body: &str) -> Result<String, Fault> {
    let envelope = hl7_2_soap::parse(request_body)?;
    let payload = envelope.payload()?;
    message::check(payload, &["ADT_A05".to_string()], &[])?;
    // ...validate and forward the payload here...
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}
```

## Sending

```rust
use hl7_2_soap::{message, response::{self, Outcome}};

let body = message::wrap_er7("MSH|^~\\&|APP||||1||ADT^A01|9|P|2.5");
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

There is no equivalent of MSA-1 in SOAP — there are three places to look
and no agreement about which wins. `response::evaluate` reads all three, in
the order that cannot be talked out of a rejection: a non-2xx status, then
a `Fault` element (even under HTTP 200, because some stacks answer 200 and
put the refusal in the body), then a `Status` element, then acceptance.

`Status` is accepted for `AA`, `CA`, and `Success`. Both conventions are in
the field — an implementation that echoes the HL7 acknowledgement code, and
one that writes a word — and a sender that knows only one will retry
forever against an endpoint that speaks the other. The crate that this one
was generalised from had exactly that split between its own receiver and
its own sender.

## Dependencies

One: `hl7-2-xml-lite-helper`, re-exported here as `xml`. It reads the
envelopes this crate carries, matching elements on their local name so
`soapenv:Body`, `soap:Body`, `SOAP-ENV:Body` and `Body` are all read alike.

It is shared rather than owned because three crates in this family needed
the same XML subset and each had written its own copy: this crate,
`hl7-2-from-xml-into-er7`, and `hl7-2-from-xsd-into-json-dictionary`. One
reader that keeps both text and attributes replaced all three. The helper
has no dependencies of its own, so the audit surface is unchanged from
before it existed, and there is one parser to read instead of three.

## Specification

`spec/index.md` is the source of truth for what this crate does. If it and
the README disagree, the spec wins.

## License

Licensed under any of MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or
GPL-3.0-only, at your option.
