# PHI, privacy, and what this software does with patient data

**Plain-language answers for a privacy officer, a security reviewer, or anyone
filling in a vendor questionnaire.** Every HL7® message these crates touch is a
clinical record. The normative source for this page is
[`spec/phi/index.md`](spec/phi/index.md) — a description of observable
behaviour, written to be checked rather than believed; this page summarizes it
and links into it, so the depth and the how-to-verify steps live there.

Nothing here is legal advice or a compliance certification.

## The short answers

| Question | Answer |
| --- | --- |
| Does this software send data anywhere? | **No.** Library code opens no socket, ever — not conditionally, not on a feature flag. The MLLP crate frames bytes on a stream *you* supply. |
| Does it phone home, or collect telemetry or analytics? | **No.** No logging, no telemetry, no HTTP client, and no dependency that could provide any. |
| Does it embed or call an AI model? | **No.** These crates ship no AI and perform no inference — [`AI_STATEMENT.md`](AI_STATEMENT.md) §1. |
| Does it store PHI? | **No.** A message goes in as text, stays in memory as text, and comes back out when you ask. Nothing is written to disk, cached, or counted. |
| Does it write PHI to logs? | It writes no logs at all. **But error and diagnostic strings can quote a value from the message**, so *your* logging can — see below. |
| Does it read files, environment variables, or spawn processes? | Message-handling library code: no. The command-line tools read the file you name and nothing else. One non-message exception: the XSD-dictionary generator's library reads the XSD schema files you point it at — its entire purpose, and never a message. |
| Is anything encrypted? | **No.** MLLP is plaintext framing, as MLLP is specified; if the connection should be TLS, you supply the TLS stream. |
| Is memory zeroed after use? | **No.** Message text is freed normally, not overwritten; a core dump or swap file can contain message content. |
| Access control, audit trail, de-identification? | **None of the three.** These are parsing libraries; all of that belongs to the system you build around them. |
| Is there real patient data in this repository? | **No.** Every sample, fixture, and benchmark input is synthetic, and none may be added — including in issues and bug reports. |
| Is it a medical device? Is it HIPAA/GDPR certified? | **No, and no.** A library has no compliance posture; the system you build around it does. Every claim here is instead verifiable against the code. |
| Who do I contact? | [`SECURITY.md`](SECURITY.md) for anything sensitive; <joel@joelparkerhenderson.com> otherwise. |

## The one thing to know before you log an error

The usual way PHI leaves a well-behaved system is a log line, and that is the
one place these libraries can hand you a surprise: `Error::BadValue` carries
the offending text from the message verbatim, and a `ValueFormat` diagnostic
formats the value into its detail string. So `format!("{error}")`,
`.unwrap()`, and any logging call you make will carry a value if the error is
one of those kinds. This is deliberate — the value is often the only way to
debug a malformed feed — and it is documented rather than hidden.

The defence is on your side and it is short: match on the error, log the
`path` (which names a location, never a value), and drop the rest.
[`spec/phi/index.md` § Where a value can escape](spec/phi/index.md#where-a-value-can-escape)
has the exact variant-by-variant table and the code to copy.

## What is deliberately not defended against

Stated so you find it here rather than in an audit: no memory zeroing, no
constant-time operations, no access control, no encryption at rest or in
transit, no de-identification, no audit trail. The full list, with the
reasoning, is
[`spec/phi/index.md` § What is not defended against](spec/phi/index.md#what-is-not-defended-against).

## Verify it yourself

Each claim above is a property of the code, checkable in minutes: one runtime
dependency (`cargo tree -p hl7-2`), no `std::net`/`std::fs`/`std::env` in
message-handling library sources, no `log` or `tracing` in any manifest.
[`spec/phi/index.md` § If you are reviewing this for a deployment](spec/phi/index.md#if-you-are-reviewing-this-for-a-deployment)
is the five-step checklist.

If real patient data ever reaches an issue, a pull request, or a commit, say
so at <joel@joelparkerhenderson.com> and it will be handled as an incident:
removed, and the history rewritten if it landed in one.

## Trademarks

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
