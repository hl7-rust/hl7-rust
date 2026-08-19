# Specification: HL7 v2 over SOAP

This is the single source of truth for what this crate does and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Every rule below is exercised by a unit test (next to the code that
implements it, in that module's `#[cfg(test)]` block) or an integration test
(`tests/integration.rs`, against `samples/`). A change to this document that
isn't backed by a test, or a code change that isn't reflected here, is a bug.

## 0. Relationship to the rest of the family

```
er7                    the ER7 encoding
  |
hl7-v2                 the HL7 v2 dictionary and message model
  |
  +-- hl7-v2-mllp      transport: HL7 v2 over TCP
  +-- hl7-v2-soap      transport: HL7 v2 over HTTP   (this crate)
  +-- hl7-v2-from-*    format conversions
```

This crate sits beside `hl7-v2-mllp`, not above it. Both answer the same
question — how does a message get from one system to another, and how does
the receiver say what became of it — for two different answers.

It depends on neither `er7` nor `hl7-v2`, and that is deliberate: a SOAP
envelope is XML, and reading one requires no HL7 knowledge beyond the names
of a few elements (§4). A caller that needs the message parsed reaches for
those crates itself.

## 1. Scope

**In scope:** the SOAP 1.1 envelope, faults and their HTTP statuses, the two
ways HL7 v2 is carried in a body, the response a receiver returns and how a
sender reads one, and the WSDL that describes the endpoint.

**Out of scope:** HTTP itself — no client, no server, no TLS, no retries.
HL7 validation and format conversion, which belong to `hl7-v2` and the
`hl7-v2-from-*` crates. SOAP 1.2, WS-Security, WS-Addressing, MTOM, and
attachments: none of them appear in the HL7 v2 interfaces this was built
from, and guessing at them would be worse than not having them.

## 2. Reading XML (`hl7-v2-xml-lite-helper`)

XML is read by the `hl7-v2-xml-lite-helper` crate, re-exported here as [`crate::xml`], and
its `spec/index.md` §3 is authoritative for what it does. Two of its rules
matter enough to this crate to restate:

**Namespace prefixes are not resolved.** Elements are matched on their local
name, so `soapenv:Body`, `soap:Body`, `SOAP-ENV:Body` and `Body` are the
same element. This is not laxity for its own sake: the prefix is chosen by
whichever stack serialized the envelope, and a receiver that insists on one
rejects valid messages from the others.

**Whitespace-only text beside child elements is dropped**, because it is
indentation. Text in a leaf is kept as it arrived.

That reader is shared rather than owned because three crates in this family
needed the same subset and each had written it: `hl7-v2-from-xml-into-er7`
kept text and dropped attributes, `hl7-v2-from-xsd-into-json-dictionary`
kept attributes and dropped text, and this crate wanted text again. A reader
that keeps both replaced all three. `hl7-v2-xml-lite-helper` has no dependencies of its
own, so the audit surface is unchanged and there is one parser to read
instead of three.

## 3. The envelope (`src/envelope.rs`)

A document whose root is not `Envelope` is a `Client` fault, as is a
document that is not well-formed XML. `Header` is optional and returned as
it is.

**A body carries exactly one business payload.** Zero or more than one is a
`Client` fault. SOAP permits several; no HL7 interface uses several, and a
receiver that took the first would process something the sender did not
necessarily mean to send on its own.

## 4. What the body carries (`src/message.rs`)

Two carriages, and an interface picks one:

- **v2.xml.** The payload element is the message structure
  (`<ADT_A05>...`), and its contents are addressable as XML. This is the
  shape a receiver can validate against the published HL7 schemas.
- **ER7 in a wrapper.** The payload is an operation element
  (`SendHL7Message`) with a text child (`hl7Message`) carrying the escaped
  pipe-delimited message. A bare element whose own text begins `MSH` is also
  accepted, because senders differ on whether they nest it.

Escaping is not optional on the second: ER7 uses `&` as its subcomponent
separator, and an unescaped message closes the document early.

**ER7 text is returned trimmed.** XML does not preserve the difference
between a sender's trailing segment terminator and a serializer's
indentation — both are whitespace in a text node — so a reader cannot honour
one without inventing the other. Trimming is safe because an ER7 parser
accepts a final segment with or without its terminator.

### 4.1 Reading a v2.xml payload

- **Structure ID** is the payload element's local name.
- **Control ID** is `MSH/MSH.10`.
- **Assigning authority** is the first value found at `MSH/MSH.3/HD.1`,
  then `MSH/MSH.4/HD.1`, then `PID/PID.3/CX.4/HD.1` — sending application,
  sending facility, then the authority on the patient identifier, which is
  the order an interface agreement usually states them in.

A path walks **every** element at each step, not only the first, because a
repeating field puts several `PID.3` elements side by side and the value may
be under any of them.

### 4.2 Checking a payload

`check` compares the structure ID and the assigning authority against what
the interface accepts. An **empty list means no restriction**, which is the
only safe reading for a receiver that was configured with nothing. An
unaccepted structure is a `Client.Validation` fault (400); an unaccepted
authority is `Client.Authorization` (403); a payload naming no authority at
all, when authorities are restricted, is `Client.Validation`, because the
message cannot be attributed to a system.

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

A receiver's success response carries `Status` of `Success` and the message
control ID it is answering.

A sender reads a response by these rules, in order:

1. A status outside 2xx is a rejection. The transport already said no.
2. A `Fault` element anywhere is a rejection, **even under HTTP 200** —
   some stacks answer 200 and put the refusal in the body, and believing
   the status there loses messages silently.
3. A `Status` element decides: `AA`, `CA` and `Success` (any case) are
   accepted; anything else is a rejection naming the value.
4. Otherwise accepted. A bare 200 with an empty or unrecognised body is what
   a minimal endpoint returns, and treating that as failure would resend
   every message forever.

Rule 3 accepts **both** conventions on purpose. An implementation that
echoes the HL7 acknowledgement code and one that writes a word are both in
the field, and a sender that knows only one retries forever against an
endpoint that speaks the other. The Python services this crate was
generalised from had exactly that split: the receiver answered `Success`
and the sender accepted only `AA`/`CA`.

## 7. WSDL (`src/wsdl.rs`)

`for_address` returns the service description with `soap:address` set to the
address given. Serving it from the endpoint, with the address rebuilt from
the request that asked for it, means the document is always right for the
environment it came from — no hand-edited copy per environment.

**The payload is declared `xsd:anyType`**, not by importing the HL7 schemas.
Those schemas use relative `xsd:include` paths, which break as soon as the
WSDL is saved elsewhere — which is exactly what happens when someone imports
it into a client tool. The structural check belongs on the server; the
WSDL's job is to describe the contract.

The operation and response element names in the WSDL come from the same
constants the rest of the crate uses, so the published contract cannot drift
from what the code accepts.

## 8. References

- SOAP 1.1 (W3C Note, 2000): envelope, `Body`, `Fault`
- WSDL 1.1 (W3C Note, 2001)
- HL7 v2.xml encoding syntax
- `hl7-v2-mllp`, `spec/index.md` — the other transport
