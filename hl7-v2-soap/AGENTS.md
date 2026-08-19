# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

A Rust crate implementing HL7 v2 over SOAP: the SOAP 1.1 envelope, faults
and their HTTP statuses, the two ways HL7 v2 is carried in a body, the
response a receiver returns and how a sender reads one, and the WSDL that
describes the endpoint.

It is a **transport layer** of the `hl7-2` family, sitting beside
`hl7-v2-mllp` rather than above it:

```
er7                    the ER7 encoding
  |
hl7-2                 the HL7 v2 dictionary and message model
  |
  +-- hl7-v2-mllp      transport: HL7 v2 over TCP
  +-- hl7-v2-soap      transport: HL7 v2 over HTTP   (this crate)
  +-- hl7-v2-from-*    format conversions
```

Both `hl7-v2-mllp` and this crate answer the same question — how does a
message get from one system to another, and how does the receiver say what
became of it — for two different protocols. This crate does the protocol
and nothing else: no HTTP client, no HTTP server, no HL7 validation, no
format conversion.

See `README.md` for the user-facing pitch and `spec/index.md` for the exact,
normative rules — **`spec/index.md` is the single source of truth for
behavior.**

## Layout

```
src/lib.rs         parse(), the Error type, and the crate documentation.
src/envelope.rs    The SOAP envelope: Header (optional) and the single
                   business payload in Body.
src/message.rs     What the body carries: v2.xml payloads and ER7 wrapped
                   in an operation element, plus check() against what an
                   interface accepts.
src/fault.rs       Fault: code, reason, and the HTTP status that pairs
                   with it.
src/response.rs    Building a success reply and evaluating one a sender
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

- **Rust edition 2024.** One dependency: `hl7-v2-xml-lite-helper`,
  re-exported as `crate::xml` (see `spec/index.md` §2). It has no
  dependencies of its own, so the audit surface stays small. This crate
  depends on neither `er7` nor `hl7-2` — a SOAP envelope is XML, and
  reading one requires no HL7 knowledge beyond the names of a few elements.
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
  some stacks answer 200 and put the refusal in the body), then a `Status`
  element (`AA`, `CA`, `Success` accepted, any case), then acceptance.
  Don't reorder or shortcut this without updating the spec first.
- Every public item must have a doc comment; match the existing doc style
  — a one-paragraph summary of *what* and, where the *why* isn't obvious, a
  short rationale.
- Before finishing a change, run:
  ```sh
  cargo test
  cargo clippy --all-targets --all-features -- -D warnings
  cargo fmt --check
  cargo rustdoc --lib -- -W missing-docs
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
- **HL7 validation and format conversion.** Those belong to `hl7-2` and
  the `hl7-v2-from-*` crates.
- **SOAP 1.2, WS-Security, WS-Addressing, MTOM, and attachments.** None of
  them appear in the HL7 v2 interfaces this crate was built from, and
  guessing at them would be worse than not having them.
- **Importing the HL7 schemas into the WSDL.** The payload is declared
  `xsd:anyType` on purpose — the published HL7 schemas use relative
  `xsd:include` paths that break as soon as the WSDL is saved elsewhere.
  The structural check belongs on the server.
