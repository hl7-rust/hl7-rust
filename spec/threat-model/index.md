[hl7-rust](../../README.md) → spec → Threat model

# Threat model

This document identifies the assets, trust boundaries, credible attackers,
controls, and residual risks for the HL7®-for-Rust workspace. It describes the
libraries and command-line tools shipped here, not a complete clinical system
that embeds them.

The model uses observable properties rather than compliance labels. It does not
claim HIPAA, GDPR, MDR, or any other certification. Patient-data behavior is
specified separately in [`spec/phi/index.md`](../phi/index.md); where the two
documents overlap, that document is authoritative about what content can leave
a message through an error or diagnostic.

## 1. Scope and assumptions

In scope:

- ER7, XML, JSON, dictionary, and XSD input supplied to a library or CLI;
- MLLP frames and byte streams supplied by a caller;
- SOAP envelopes supplied as text;
- parsed values, rendered output, acknowledgements, errors, and diagnostics;
- build dependencies, source history, and crates.io release artifacts.

The caller owns the process and operating-system boundary. The caller also owns
network listeners and clients, TLS, authentication, authorization, persistence,
timeouts, retries, rate limits, audit logs, and access control. The transport
crates accept text or a caller-provided `Read + Write` stream; they do not open
sockets or make HTTP requests themselves.

The model assumes Rust's safe-language guarantees and a non-compromised Rust
toolchain. It does not assume that an incoming message, a sender-declared
assigning authority, a dictionary, a schema, a file path, or a serialized value
is trustworthy merely because it is syntactically valid.

## 2. Assets and security goals

| Asset | Security goal |
|---|---|
| Clinical message content | Do not disclose it except through an API or output the caller requested and the documented error/diagnostic exceptions. |
| Message meaning | Do not turn malformed or ambiguous input into a different plausible clinical value. Reject it or apply a documented fallback. |
| Routing and correlation | Preserve the declared sender/receiver fields and correlate an acknowledgement with the correct control ID. |
| Availability | Malformed input must return an error rather than panic, abort, exhaust the stack, or grow an incomplete transport buffer without a bound. |
| Host boundary | Library code must not unexpectedly reach the network, environment, subprocesses, or message-bearing filesystem paths. |
| Release integrity | Make source and release provenance inspectable, and state where artifact-level guarantees do not exist. |

Correctness is a security property here. Returning the wrong patient's value
without an error can be more harmful than refusing a message.

## 3. Threat actors and trust boundaries

The credible input-side actor is a faulty or malicious remote system able to
send arbitrary bytes, including a validly framed message with adversarial
content. The same capabilities apply to a user or deployment supplying a vendor
dictionary, XSD set, JSON document, XML document, CLI input file, or standard
input. A report or fixture accidentally containing real patient data is also a
confidentiality incident even when nobody intended harm.

The supply-chain actor is someone able to compromise a dependency, a forge
account, a maintainer credential, a build environment, or a published artifact.

Data crosses these boundaries:

1. bytes or text enter a parser or converter;
2. a dictionary or schema assigns names and types to positions;
3. a caller-provided stream enters or leaves MLLP framing;
4. parsed values, rendered documents, errors, and diagnostics return to the
   caller;
5. a CLI reads a caller-selected file or standard input and writes a
   caller-selected file or standard output;
6. source and dependencies enter a build, and a package leaves it for crates.io.

## 4. Input-integrity threats and controls

### 4.1 Structural confusion

ER7 delimiters come from MSH-1/MSH-2 and duplicate delimiter roles are rejected.
MLLP strict framing is the default, truncated frames are not returned as
messages, and a framing error clears the buffer rather than guessing where the
next message starts. Acknowledgement helpers echo the received control ID and
swap the complete routing fields; the sender still has to compare the returned
control ID with the message it sent.

The JSON readers reject malformed strings, numbers, trailing data, and excessive
nesting. The XML reader requires matched elements and well-formed attributes.
It does not resolve namespaces: callers using documents where identical local
names from different namespaces have different authority must use a
namespace-aware parser instead. It also does not implement DTD entity
declarations; a `DOCTYPE` is skipped and unknown entities remain literal, so an
XML entity-expansion bomb is not expanded by this reader.

Dictionaries add names and validation rules but do not make a sender trusted.
An unmodelled field remains available positionally. SOAP assigning-authority
checks compare a self-asserted message value with an allowlist; they are input
validation, not cryptographic authentication. An authenticated channel or
equivalent caller-owned identity binding is required before treating that value
as the sender's identity.

### 4.2 Resource exhaustion

Recursive JSON and XML parsing stops after 256 container or element levels.
MLLP limits bytes buffered **without a complete frame** to 16 MiB by default,
and callers can select a smaller or larger interface-specific limit.

There is no universal byte-size, segment-count, field-count, or repetition-count
limit for a complete message, XML document, JSON document, dictionary, schema
set, or CLI input. Parsers and converters allocate in proportion to accepted
input, and the XML reader builds an owned tree. A complete MLLP frame can also
exceed the incomplete-frame limit when its terminator is already present in the
buffer. Applications exposed to untrusted or high-volume input must enforce
byte limits before parsing, configure read timeouts, cap concurrent work, and
apply rate limits at the boundary they own.

The project fuzzes the structured parsers named in
[`spec/rust-fuzz/index.md`](../rust-fuzz/index.md). Fuzzing reduces known parser
failures; it is not proof that every input is bounded or panic-free. The `er7`
dependency's parsing surface is not fuzzed in this repository.

### 4.3 Injection and output boundaries

XML writers escape markup characters, JSON writers escape strings, and ER7
mutation escapes structural delimiters when the value-taking API is used. APIs
that explicitly accept wire-format ER7 preserve its structural meaning; callers
must not pass untrusted plain text to those APIs as though it were already
encoded.

File paths accepted by CLIs and the XSD dictionary generator are authority the
caller deliberately supplies. The tools do not sanitize a path into a sandbox,
and output files follow the operating system's normal overwrite and permission
rules. Run them with an account and working directory appropriate to the files
they may read or replace.

## 5. Confidentiality threats and controls

Library code performs no logging or telemetry. Message-handling libraries do
not open files, sockets, environment variables, or subprocesses. The deliberate
filesystem exception is the XSD dictionary generator, which reads the schema
paths the caller names and does not process clinical messages.

Errors and validation diagnostics can contain message values. Formatting,
logging, panicking with, or serializing the whole error can therefore disclose
patient data. [`spec/phi/index.md`](../phi/index.md) names the exact variants and
the safe fields a caller can retain. Message buffers are not zeroed on drop and
may remain in swap, a core dump, allocator memory, terminal scrollback, or a
caller-created log.

MLLP supplies framing only. SOAP helpers supply envelope handling only. Neither
provides encryption, peer authentication, or authorization. Use a caller-owned
TLS or otherwise protected stream and apply deployment policy outside these
crates.

## 6. Supply-chain threats and controls

Every library and binary crate root forbids `unsafe` code. Runtime dependencies
are intentionally small; CI tests, lints, checks documentation, enforces the
MSRV and coverage floor, audits dependencies, and generates CycloneDX SBOMs.
Commits and tags made since 2026-08-27 are SSH-signed. The release runbook and
the bounds on agentic release decisions are in
[`spec/release-process/index.md`](../release-process/index.md).

Residual risks remain: crates.io artifacts have no project-provided signature or
reproducible-build attestation, SBOMs are CI artifacts rather than release
attachments, publishing uses a long-lived maintainer credential, and older
history is unsigned. A signed tag authenticates the tagged source history; it
does not prove that a downloaded `.crate` was built from that tag.

## 7. Deployment checklist

Before accepting untrusted traffic, a system embedding these crates should:

1. authenticate and encrypt the connection outside the crate;
2. enforce complete-input byte limits, timeouts, concurrency limits, and rate
   limits before parsing;
3. configure the MLLP incomplete-frame limit for the interface;
4. compare acknowledgement control IDs and persist before promising acceptance;
5. treat sender-declared authorities as claims until bound to an authenticated
   peer;
6. log paths and error categories, not whole errors or diagnostic details;
7. constrain CLI and schema-generator filesystem permissions;
8. pin and audit dependencies, and verify signed source tags when provenance
   matters.

## 8. Maintenance rule

This model changes in the same commit as any new input format, I/O capability,
ambient authority, dependency class, parser limit, transport guarantee, release
mechanism, or way message content can enter an error. A new mitigation needs a
test or CI evidence where it can be automated. A residual risk is removed only
when the code or process that closes it exists and has been verified.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
