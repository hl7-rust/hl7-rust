[hl7-rust](../../README.md) → spec → PHI

# Protected health information

Every message these crates touch is a clinical record. This document states
what the crates do with it, what they do not do with it, and where a value
can escape into somewhere you did not intend — so that a reviewer can check
the claims rather than take them on trust.

None of this is legal advice, and none of it is a compliance certification.
It is a description of observable behavior, and like every other document
under `spec/`, a change to the code that contradicts it is a bug.

## Contents

- [The short version](#the-short-version)
- [What the libraries do](#what-the-libraries-do)
- [What the libraries never do](#what-the-libraries-never-do)
- [Where a value can escape](#where-a-value-can-escape)
- [What is not defended against](#what-is-not-defended-against)
- [The command-line tools](#the-command-line-tools)
- [The transports](#the-transports)
- [This project's own data](#this-projects-own-data)
- [If you are reviewing this for a deployment](#if-you-are-reviewing-this-for-a-deployment)

## The short version

A message goes in as text, stays in memory as text, and comes back out when
you ask for it. Nothing is written to disk, sent over a network, logged,
counted, or cached. The libraries do not open files, read environment
variables, spawn processes, or open sockets — not conditionally, not on a
feature flag, not at all.

The one thing to know before you log an error: **error and diagnostic
messages can quote a value from the message.** See
[Where a value can escape](#where-a-value-can-escape).

## What the libraries do

- Hold the message text you passed in, and the parsed structure over it.
  `er7` stores text as sent and decodes on demand, which is what makes a
  byte-for-byte round trip possible — and it also means the original bytes
  stay in memory for the lifetime of the `Message`.
- Read the bundled release dictionaries, which are compiled into the binary
  with `include_str!` from `hl7-2/schemas/` and parsed on first use. No file
  is opened for them at run time.
- Read a dictionary you supply, from a string or from bytes you read
  yourself. The crate does not fetch it.
- Return values, trees, diagnostics, and rendered text to the caller.

## What the libraries never do

Verifiable by reading the manifests and grepping the sources:

| Not done | How to check |
|---|---|
| No logging or tracing | No `log`, `tracing`, or any logging facade in any `Cargo.toml` in the workspace |
| No telemetry, analytics, or phone-home | No HTTP client anywhere; no network dependency of any kind |
| No filesystem access from library code | `std::fs` and `File::open` appear in no library source, only in the CLI's `main.rs` |
| No environment variables | `std::env` appears in no library source, only in the CLI's argument parsing |
| No sockets opened | `std::net` appears in no library source; the MLLP crate is generic over a byte stream you supply |
| No subprocesses | `std::process` appears in no library source |
| No global or ambient state | Nothing is cached across calls except the lazily parsed bundled dictionaries, which contain no message data |
| No serialization framework | No `serde`; the JSON reader is hand-written and reads only dictionaries |

The whole runtime dependency surface of the workspace is `er7`, plus
`chrono` in `hl7-2-mllp` — optional, off by default, and used only to stamp
a generated acknowledgement with the wall clock. The `syn`, `quote`, and
`proc-macro2` crates appear in the two `*-derive` crates and run at compile
time only. `criterion` and `libfuzzer-sys` are development dependencies and
are never linked into a library or a binary you ship.

## Where a value can escape

This is the part that matters in practice, because the usual way PHI leaves
a well-behaved system is a log line.

**Errors that carry message content:**

| Variant | Carries |
|---|---|
| `Error::BadValue { path, expected, found }` | `found` is the offending text from the message, verbatim |
| `Error::BadMshHeader(String)` | The detail can quote part of the malformed header |
| `Error::Path(String)`, `Error::UnwritablePath(String)` | The path, which names a location rather than a value |
| `Error::Invalid(Vec<Diagnostic>)` | Every `Severity::Error` diagnostic, with the caveat below |

**Diagnostics that carry message content:** `Diagnostic::detail` for a
`Kind::ValueFormat` finding is formatted as `"{value:?} is not a valid
{data_type} value"` — the value is in the string. Every other diagnostic
kind reports a location and a description of the problem, not the content
at that location.

`Display` for both types reproduces those strings, so
`format!("{error}")`, `error.to_string()`, `{:?}`, `println!`, `panic!`,
`.unwrap()`, `.expect()`, and any logging call you make yourself will
carry a value if the error is one of the kinds above.

**A path is not a value, but it is not nothing.** `OBX[200]-5` says nothing
about a patient. `Diagnostic::path` and the path in an error are safe to log
in a way the detail is not.

**What to do about it.** If your logs are less trusted than your message
store — which is usual, because logs are shipped, aggregated, and retained
differently — match on the error and log the parts you want rather than
formatting the whole thing:

```rust
match message.get("PID-5.1") {
    Err(hl7_2::Error::BadValue { path, expected, .. }) => {
        // Deliberately drops `found`.
        eprintln!("{path}: expected {expected}");
    }
    Err(error) => eprintln!("{error}"),
    Ok(value) => { /* ... */ }
}
```

The same applies to `Message::validate`: filter on `severity` and `kind`,
log `path`, and treat `detail` as message content.

## What is not defended against

Stated plainly, because a security review will ask and a vague answer is
worse than a limitation:

- **Memory is not zeroed.** `String` and `Vec<u8>` holding message text are
  freed normally when a `Message` drops; the bytes are not overwritten
  first. A core dump, a swap file, or a heap inspection can contain message
  content. No crate here uses `zeroize` or a locked allocator, and adding
  one would not be meaningful while the caller also holds the original
  `&str`.
- **No constant-time anything.** Nothing here is a cryptographic operation,
  and no comparison is timing-hardened.
- **No access control.** These are parsing libraries. Who may read which
  message is entirely the caller's question.
- **No encryption, at rest or in transit.** MLLP is plaintext framing on
  whatever stream you give it; if that stream should be TLS, you supply the
  TLS stream.
- **No de-identification.** There is no scrub, redact, or anonymize
  function, and none is planned. Redaction is a policy decision about a
  particular data set, not a library default.
- **No audit trail.** Nothing records that a message was read. If your
  environment requires an access log, that is above this layer.

## The command-line tools

The `hl7-v2` binary, and the binaries the conversion crates ship, do what
you point them at and nothing else: read a named file or standard input,
write a named file or standard output, exit. No config file is searched
for, no environment variable is consulted, no history is kept, no temporary
file is written.

Two ordinary shell hazards are worth naming anyway, because they are how
PHI most often leaks from a command line and neither is something a program
can prevent:

- **Shell history.** A message passed as a command-line argument lands in
  `~/.zsh_history`. Pipe it or redirect from a file instead.
- **Terminal scrollback and screen sharing.** Output printed to a terminal
  persists in the scrollback buffer, and in whatever recording of the
  session exists.

## The transports

Both transport crates are deliberately narrower than they sound.

`hl7-2-mllp` implements MLLP framing over a `Transport` — a trait for any
byte stream. It does not open, bind, connect, or configure anything. The
caller passes a `TcpStream`, a TLS stream, a Unix socket, or a test buffer.
So the security properties of the connection are the caller's, and MLLP by
itself provides no confidentiality, integrity, or authentication. That is a
property of MLLP as specified, not a shortcut taken here.

`hl7-2-soap` builds and parses SOAP envelopes. It contains no HTTP client
and no HTTP server; its only dependency is the shared XML reader. You do
the HTTP with whatever your organization has approved.

The practical consequence for a deployment review: the network posture of a
system built on these crates is decided entirely by the code around them,
which is where a reviewer should look.

## This project's own data

- **Every sample, test fixture, and benchmark input in this repository is
  synthetic.** There is no real patient data anywhere in the history, and
  none may be added — including in an issue, a pull request, a test that
  reproduces a bug, or a benchmark corpus.
- **Benchmark inputs are either generated in code** (`hl7-2/benches/parse.rs`,
  and most of the conversion crates' `benches/convert.rs`) **or are this
  repository's own synthetic sample files**, compiled in with `include_str!`
  — `hl7-2-from-xml-into-er7` and `hl7-2-from-json-into-er7` read
  `samples/orm_o01.*` that way. Either way the input is in the repository
  and reviewable. There is no external corpus, no downloaded fixture, and
  no benchmark that reads whatever happens to be on the machine.
- **Reports must be redacted.** [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
  says how: keep the structure, replace the values. Structure is what
  reproduces a parsing bug; names and identifiers are not.
- **The website sets no cookies, runs no analytics, and loads no
  third-party script.** It is a static site on GitHub Pages; its stylesheets
  and fonts are served from the same origin. The only browser storage it
  uses is one `localStorage` key remembering a light or dark theme choice.

If real patient data does reach an issue, a pull request, or a commit, say
so at <joel@joelparkerhenderson.com> and it will be handled as an incident:
removed, and the history rewritten if it landed in one.

## If you are reviewing this for a deployment

The five things a review usually wants, with where to check each:

1. **Dependency surface.** `cargo tree -p hl7-2` — one crate, `er7`, itself
   dependency-free. Add `--all-features` to see what `derive` pulls in at
   compile time.
2. **No network or filesystem access from the library.** Grep the sources
   for `std::net`, `std::fs`, and `std::env`; they appear only in the CLI.
3. **No logging.** Grep the manifests for `log` and `tracing`.
4. **What can appear in an error.**
   [Where a value can escape](#where-a-value-can-escape) above, then
   `hl7-2`'s `src/lib.rs` `Error` enum and `src/validate.rs` `Diagnostic`.
5. **Licensing.** MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or
   GPL-3.0-only, at your option — the same five in every crate, byte for
   byte.

What this project cannot give you is a HIPAA, GDPR, or MDR attestation. A
library is not a covered entity and does not have a compliance posture; the
system you build around it does. What is offered instead is that every
claim on this page is a property of the code that you can verify yourself
in an afternoon.
