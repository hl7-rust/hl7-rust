# Specification: HL7 v3 over SOAP

This is the single source of truth for what this crate does and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Every rule below is exercised by a unit test (next to the code that
implements it, in that module's `#[cfg(test)]` block) or an integration test
(`tests/integration.rs`, against `samples/`). A change to this document that
isn't backed by a test, or a code change that isn't reflected here, is a
bug.

## 0. Relationship to the rest of the family

```
hl7-2-xml-lite-helper    the XML reader this crate reads through
  |
hl7-3                    RIM backbone classes, data types, message
  |                      envelope
  |
  +-- hl7-3-soap         transport: HL7 v3 over HTTP   (this crate)
```

Unlike `hl7-2`, HL7 v3 has no MLLP-equivalent transport crate — v3 was
designed with SOAP as its own historically dominant transport (NHS
England's Personal Demographics Service and IHE profiles built on v3 both
carry it this way), so there is no "the other one" for this crate to sit
beside.

It depends on neither `hl7-3` nor any RIM decoding, and that is deliberate:
a SOAP envelope is XML, and reading one requires no HL7 v3 knowledge
beyond the names of a few elements (§4). A caller that needs the payload
decoded reaches for `hl7-3` itself.

## 1. Scope

**In scope:** the SOAP 1.1 envelope, faults and their HTTP statuses, reading
an HL7 v3 payload's interaction ID, control ID, and assigning authority,
checking a payload against what an interface accepts, the real HL7 v3
acknowledgement (`MCCI_IN000002UV01`), and the WSDL that describes the
endpoint.

**Out of scope:** HTTP itself — no client, no server, no TLS, no retries.
RIM decoding and domain-payload interpretation, which belong to `hl7-3`.
SOAP 1.2, WS-Security, WS-Addressing, MTOM, and attachments: none of them
appear in the HL7 v3 interfaces this was built from, and guessing at them
would be worse than not having them. Validating a payload against HL7 v3's
published schemas — the WSDL declares the payload `xsd:anyType` on
purpose; see §7.

## 2. Reading XML (`hl7-2-xml-lite-helper`)

XML is read by the `hl7-2-xml-lite-helper` crate, re-exported here as
[`crate::xml`], and its `spec/index.md` §3 is authoritative for what it
does. Two of its rules matter enough to this crate to restate:

**Namespace prefixes are not resolved.** Elements are matched on their
local name, so `soapenv:Body`, `soap:Body`, `SOAP-ENV:Body` and `Body` are
the same element, and `PRPA_IN201305UV02` inside `xmlns="urn:hl7-org:v3"`
is read the same as one with no namespace declared at all. This is not
laxity for its own sake: the prefix and namespace declaration style is
chosen by whichever stack serialized the message, and a receiver that
insists on one rejects valid messages from the others.

**Whitespace-only text beside child elements is dropped**, because it is
indentation. Text in a leaf is kept as it arrived.

This crate uses the same shared reader `hl7-2-soap`,
`hl7-2-from-xml-into-er7`, and `hl7-2-from-xsd-into-json-dictionary` use —
see those crates' own specs for why it was extracted. `hl7-2-xml-lite-helper`
has no dependencies of its own, so the audit surface stays small.

## 3. The envelope (`src/envelope.rs`)

A document whose root is not `Envelope` is a `Client` fault, as is a
document that is not well-formed XML. `Header` is optional and returned as
it is.

**A body carries exactly one business payload.** Zero or more than one is a
`Client` fault. SOAP permits several; no HL7 interface uses several, and a
receiver that took the first would process something the sender did not
necessarily mean to send on its own.

## 4. What the body carries (`src/message.rs`)

One carriage, unlike `hl7-2-soap`'s two: HL7 v3 is XML natively, so the
payload is always a complete v3 message — the same transport wrapper,
control act wrapper, and domain payload `hl7-3::message::parse` reads,
root element named for the interaction (`PRPA_IN201305UV02` and so on).
There is no ER7-in-a-wrapper carriage to read; v3 has no ER7 form to wrap.

### 4.1 Reading a v3 payload

- **Interaction ID** is the payload element's local name.
- **Control ID** is the payload's `id` child's `extension` attribute — the
  v3 analogue of MSH-10, though v3 spells an identifier as `root` +
  `extension` (an `II`) rather than one string; this crate returns
  `extension` alone, since that is what a log line or a retry key wants.
- **Assigning authority** is the first value found at `id/@root` (the
  message's own identifier scheme), then
  `sender/device/id/@root` — the message's own claim first, then its
  sender's, which is the order an interface agreement usually states them
  in.

### 4.2 Checking a payload

`check` compares the interaction ID and the assigning authority against
what the interface accepts. An **empty list means no restriction**, which
is the only safe reading for a receiver that was configured with nothing.
An unaccepted interaction is a `Client.Validation` fault (400); an
unaccepted authority is `Client.Authorization` (403); a payload naming no
authority at all, when authorities are restricted, is `Client.Validation`,
because the message cannot be attributed to a system.

## 5. Faults (`src/fault.rs`)

A fault is a code, a reason, and an HTTP status. The pairing is the point:

| Constructor | Code | Status | Retry? |
|---|---|---|---|
| `client` | `Client` | 400 | no |
| `validation` | `Client.Validation` | 400 | no |
| `authorization` | `Client.Authorization` | 403 | no |
| `server` | `Server` | 500 | yes |
| `configuration` | `Server.Configuration` | 500 | yes |

`is_retryable` is `status >= 500`. A `Client` fault repeated is the same
fault; retrying it is how a poison message becomes an infinite loop.

Code and reason are escaped into the envelope, so neither can break the
document however they were built.

## 6. Responses (`src/response.rs`)

A receiver's success response is `MCCI_IN000002UV01`, HL7 v3's real
Master Information Composite / Control Act acknowledgement interaction —
not a shape this crate invented, unlike `hl7-2-soap`'s `AckResponse`
(which exists because HL7 v2 has no standard SOAP-carried ACK shape of its
own). It carries `acknowledgement/typeCode` and echoes the original
message's control ID in `targetMessage/id/@extension`.

A sender reads a response by these rules, in order:

1. A status outside 2xx is a rejection. The transport already said no.
2. A `Fault` element anywhere is a rejection, **even under HTTP 200** —
   some stacks answer 200 and put the refusal in the body, and believing
   the status there loses messages silently.
3. `acknowledgement/typeCode/@code` decides: `AA` and `CA` are accepted —
   the same two codes `hl7-2-soap` accepts from MSA-1, since v3's
   acknowledgement type codes and v2's ACK codes are drawn from the same
   conceptual vocabulary — anything else is a rejection naming the value.
4. Otherwise accepted. A bare 200 with an empty or unrecognised body is
   what a minimal endpoint returns, and treating that as failure would
   resend every message forever.

## 7. WSDL (`src/wsdl.rs`)

`for_address` returns the service description with `soap:address` set to
the address given. Serving it from the endpoint, with the address rebuilt
from the request that asked for it, means the document is always right
for the environment it came from — no hand-edited copy per environment.

**The payload is declared `xsd:anyType`**, not by importing HL7's own v3
schemas. Those schemas are large and cross-reference each other by
relative path, which breaks as soon as the WSDL is saved elsewhere — which
is exactly what happens when someone imports it into a client tool. The
structural check belongs on the server; the WSDL's job is to describe the
contract. The response message, unlike the request, references the real
`MCCI_IN000002UV01` element in the `urn:hl7-org:v3` namespace rather than
an `xsd:anyType` placeholder, since that shape is fixed by this crate, not
by the deployment.

The operation name and acknowledgement interaction name in the WSDL come
from the same constants the rest of the crate uses, so the published
contract cannot drift from what the code accepts.

## 8. References

- SOAP 1.1 (W3C Note, 2000): envelope, `Body`, `Fault`
- WSDL 1.1 (W3C Note, 2001)
- HL7 v3 message wrapper and `MCCI_IN000002UV01` acknowledgement shape
- `hl7-3`, `spec/index.md` — the RIM and message envelope this crate
  carries without decoding
- `hl7-2-soap`, `spec/index.md` — the v2 sibling this crate mirrors
