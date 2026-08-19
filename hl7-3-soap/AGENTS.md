# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

A Rust crate implementing HL7 v3 over SOAP: the SOAP 1.1 envelope, faults
and their HTTP statuses, reading what a v3 payload claims (interaction,
control ID, assigning authority), the real HL7 v3 acknowledgement, and the
WSDL that describes the endpoint. Sibling of `hl7-2-soap`, adapted to
HL7 v3's shape instead of v2's.

It sits beside `hl7-3`, not above it:

```
hl7-2-xml-lite-helper    the XML reader this crate reads through
  |
hl7-3                    RIM backbone classes, data types, message
  |                      envelope
  |
  +-- hl7-3-soap         transport: HL7 v3 over HTTP   (this crate)
```

Unlike `hl7-2`, there is no MLLP-equivalent sibling here to sit beside —
SOAP is HL7 v3's own historically dominant transport, not the alternative
one. This crate does the protocol and nothing else: no HTTP client, no
HTTP server, no RIM decoding, no domain-payload interpretation.

See `README.md` for the user-facing pitch and `spec/index.md` for the
exact, normative rules — **`spec/index.md` is the single source of truth
for behavior.**

## Layout

```
src/lib.rs         parse(), the Error/Fault re-exports, and the crate
                   documentation.
src/envelope.rs    The SOAP envelope: Header (optional) and the single
                   business payload in Body.
src/message.rs     What the body carries: a complete HL7 v3 message, plus
                   interaction_id/control_id/assigning_authority readers
                   and check() against what an interface accepts.
src/fault.rs       Fault: code, reason, and the HTTP status that pairs
                   with it.
src/response.rs    Building the real HL7 v3 acknowledgement
                   (MCCI_IN000002UV01) and evaluating one a sender
                   receives (Outcome::Accepted / Rejected).
src/wsdl.rs        for_address(): the service description served from the
                   endpoint, with soap:address rebuilt from the request.
tests/integration.rs  Black-box tests, against samples/.
samples/           Example payloads used by the integration tests.
spec/index.md      Normative specification (source of truth).
```

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block;
anything crossing module boundaries goes in `tests/integration.rs`.

## Working conventions

- **Rust edition 2024.** One dependency: `hl7-2-xml-lite-helper`,
  re-exported as `crate::xml` (see `spec/index.md` §2). It has no
  dependencies of its own, so the audit surface stays small. This crate
  depends on neither `hl7-3` nor any RIM decoding — a SOAP envelope is
  XML, and reading one requires no HL7 v3 knowledge beyond the names of a
  few elements.
- **Namespace prefixes are not resolved.** Elements are matched on their
  local name, so `soapenv:Body`, `soap:Body`, `SOAP-ENV:Body` and `Body`
  are the same element. This is deliberate, not laxity: the prefix is
  chosen by whichever stack serialized the envelope.
- **One payload per body.** Zero or more than one business payload in a
  `Body` is a `Client` fault — never silently take the first.
- **A fault carries its HTTP status, and the pairing is the point.**
  `Client` (400) and `Client.Authorization` (403) must not be retried;
  `Server` (500) and `Server.Configuration` (500) should be. Getting this
  wrong turns a poison message into an infinite retry loop, or drops a
  message that would have gone through a moment later.
- **A sender reads a response in a fixed order** (`spec/index.md` §6): a
  non-2xx status, then a `Fault` element anywhere (even under HTTP 200 —
  some stacks answer 200 and put the refusal in the body), then
  `acknowledgement/typeCode/@code` (`AA`, `CA` accepted), then acceptance.
  Don't reorder or shortcut this without updating the spec first.
- **The acknowledgement shape is real HL7 v3** (`MCCI_IN000002UV01`), not
  invented — unlike `hl7-2-soap`'s `AckResponse`, which exists precisely
  because v2 has no standard SOAP-carried ACK shape. Don't rename its
  element or attribute names without checking they still match what real
  HL7 v3 interfaces expect.
- Every public item must have a doc comment; match the existing doc style
  — a one-paragraph summary of *what* and, where the *why* isn't obvious, a
  short rationale.
- Before finishing a change, run:
  ```sh
  cargo test -p hl7-3-soap
  cargo clippy -p hl7-3-soap --all-targets -- -D warnings
  cargo fmt --check
  ```

## Making a spec-affecting change

1. **Update `spec/index.md` first** so it states the new behavior precisely.
2. Implement it, matching the module boundaries above.
3. Add or update the tests that pin it.
4. Update `README.md` only if the user-facing summary or examples change.
5. Run the checks above.

## Non-goals (don't "fix" these without discussion)

- **HTTP itself.** No client, no server, no TLS, no retries. This crate
  turns bytes into meaning and back; the socket is the caller's.
- **RIM decoding and domain-payload interpretation.** Those belong to
  `hl7-3`.
- **SOAP 1.2, WS-Security, WS-Addressing, MTOM, and attachments.** None of
  them appear in the HL7 v3 interfaces this crate was built from, and
  guessing at them would be worse than not having them.
- **Importing HL7's real v3 schemas into the WSDL.** The request payload is
  declared `xsd:anyType` on purpose — those schemas cross-reference each
  other by relative path and break as soon as the WSDL is saved elsewhere.
  The structural check belongs on the server.
- **Vocabulary domain validation** on `typeCode`, or any other coded
  attribute this crate reads. `hl7-3` itself stays off HL7's full
  vocabulary-domain tables for the same reason (hallucination risk on
  hand-typed code lists); this crate inherits that restraint.
