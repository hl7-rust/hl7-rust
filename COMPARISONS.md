# Comparisons

Interface engines, the mature libraries on other platforms, the other Rust
crates, and the pipe-splitting you were about to write yourself. When each
is the right answer, and when this project is the wrong one.

The reader-friendly version is
<https://hl7-rust.github.io/docs/comparison/>.

**No performance comparison is claimed here.** Nothing in this project has
been benchmarked against another library. Comparing fairly means matching
what each one actually does, and a parser that only splits on pipes is not
doing the same work as one that resolves a dictionary. So this document
compares *capability and shape*, which is checkable, rather than speed,
which would not be. Our own measured figures, with their method, are in
[`BENCHMARKS.md`](BENCHMARKS.md).

## First, which kind of thing do you need?

Most comparisons in this space go wrong by putting products from three
different categories in one table. The honest first question is what you
are building.

| If you need to… | You want | This project |
|---|---|---|
| *Run* interfaces: routes, retries, queues, monitoring, on-call | An interface engine | Not that. Useful alongside one. |
| Write an application that happens to speak HL7® v2 | A library | Yes |
| Do a one-off transformation at a shell prompt | A command-line tool | Yes — six binaries, no Rust required |

## Interface engines

[Open Integration Engine](https://github.com/OpenIntegrationEngine) — the
community fork of Mirth Connect, made after Mirth moved to a
commercial-only license at version 4.6 in 2025 — and its commercial
siblings are a different category of thing entirely. An engine gives you
channels, routing, a management UI, JavaScript transformers, persistence,
retry and alerting, and an operations story. It is a system you deploy and
run.

A library gives you a function call. If your problem is "forty interfaces,
three hospitals, and someone has to be paged when one stops", an engine is
the right answer and no amount of crate substitutes for it.

These crates are useful *alongside* an engine rather than instead of it:

- The service at the end of a channel, where you would otherwise be writing
  the v2 parsing again in whatever language that service is in.
- A shell-level check or transformation, using the command-line tools,
  without standing anything up.
- A dedicated high-volume path where a JVM per message and a channel round
  trip are more than the job needs.

## HAPI, and the mature libraries

[HAPI HL7v2](https://hapifhir.github.io/hapi-hl7v2/) is the reference
open-source HL7 v2 library, in Java, dual licensed under MPL 1.1 and
GPL 2.0. It has been maintained for two decades, ships a generated typed
model for every segment and message of every release, and has seen far more
real-world traffic than anything here. Its .NET port and the mature Python
libraries are in the same position.

**If your platform is the JVM, use HAPI.** That is not modesty: a
twenty-year-old library with complete release coverage and a large user
base is the lower-risk choice, and reimplementing it in a language you were
not otherwise using is a bad trade.

Where this project differs, stated as trade-offs rather than wins:

| | The mature libraries | Here |
|---|---|---|
| Release coverage | Complete generated model: every segment, every release | 24 segments, 42 types, 4 structures, extensible in JSON — [`spec/conformance/index.md`](spec/conformance/index.md) |
| Runtime | A JVM, a CLR, or a Python interpreter | A static binary: no runtime, no GC |
| Dependency tree | Substantial, and audited as such | One crate, itself dependency-free |
| Track record | Two decades of production traffic | Published in 2026. New. |
| License | MPL 1.1 or GPL 2.0, for HAPI | MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or GPL-3.0-only, at your option |

The license row decides some evaluations outright. A permissive option
matters if you are linking into a closed-source product; a copyleft option
matters if your organisation prefers one. Offering five is how this project
avoids having that conversation with anyone.

## The other Rust crates

If Rust is already your platform, the existing options are narrower than
they first appear. Figures from crates.io, checked 2026-08-26:

| Crate | Latest | Published | Downloads | Scope, as it describes itself |
|---|---|---|---|---|
| `hl7-mllp-codec` | 0.4.0 | 2022-07-22 | 25,755 | A Tokio codec for MLLP framing. Transport only, no v2 semantics. |
| `hl7-parser` | 0.3.0 | 2025-02-24 | 16,625 | Parses message structure; states that it does not validate correctness. |
| `rust-hl7` | 0.5.0 | 2021-09-08 | 14,777 | Parser and object builder; describes itself as experimental. |

Read those publication dates carefully rather than dismissively. A library
not released in three years may be dormant, or may simply be finished and
stable for what it does. `hl7-mllp-codec` in particular does one small
thing and does it well; if you are already on Tokio and want framing alone,
it remains a reasonable choice.

Those download counts are also the honest size of the Rust HL7 audience
today: low tens of thousands of pulls, accumulated over years.

What none of them offers is the whole span — a release dictionary,
validation, mutation and building, format conversion, and both transports,
maintained together. That gap is the reason this project exists.

## Splitting on pipes yourself

This is the real competition, and usually it is the incumbent: a hundred
lines somewhere in the codebase that split a segment on `|` and index the
result.

```rust
// The bug: this is not how HL7 works.
let fields: Vec<&str> = segment.split('|').collect();
let name = fields[5];
```

Each of the following is a production incident that code has already caused
somewhere:

- **The delimiters are declared by the message**, in MSH-1 and MSH-2. The
  usual set is a convention, not a guarantee, and a sender using different
  ones is parsed into nonsense rather than rejected.
- **Escape sequences** mean an ampersand or a backslash inside a value is
  not what it looks like. Splitting first and decoding later is the wrong
  order.
- **The explicit null is not an empty field.** One says "we have nothing
  here"; the other says "delete what you have on file". Collapsing the two
  writes wrong data into a patient record, silently.
- **Repetitions, components, and subcomponents** are four levels deep, and
  the naive version handles one.
- **MSH is off by one**, because MSH-1 is the field separator itself. Every
  hand-rolled parser meets this bug.

None of that is a reason to feel bad about the hundred lines; it is a
reason to replace them with something whose round trip is a test. If you
replace them with a different library than this one, the goal is still met.

## What this project offers

Stated without adjectives, so each one can be checked:

- **One dependency.** `hl7-2` depends on `er7`, which depends on nothing.
  That is the whole tree, which matters where dependency trees get audited.
- **No runtime.** A static binary, no JVM, no GC pause, and a command-line
  tool an integration analyst can use without writing any Rust.
- **Byte-for-byte round trip**, as a test rather than an aspiration.
- **The explicit null kept distinct** from an absent value, at every level.
- **A stated conformance position** — exactly which segments, types, and
  structures, and what happens outside them.
- **A stated position on patient data** — what these crates do with it, and
  where a value can escape into a log.
- **An MSRV of current stable minus three**, because hospital toolchains
  are approved on a cycle measured in quarters.
- **Five licenses, at your option**, so licensing is not a conversation.
- **A vendor dialect is one JSON file**, or generated from your own XSDs.

## When this project is the wrong answer

Read this before adopting, not after.

1. **You need the HL7® FHIR® standard.** Not implemented here, at all. The
   umbrella crate
   reserves a module path and nothing more.
2. **You need CDA**, or a substantial HL7 v3 implementation. The v3 crate
   is a foundation — six RIM classes, six data types, a generic envelope —
   and says so in its own first section.
3. **You are on the JVM, .NET, or Python already.** Use the mature library
   for your platform. Adding a language to a hospital's supported stack is
   a bigger decision than picking a parser.
4. **You need an operations story, not a function call.** Routing, retries,
   queues, monitoring, and an on-call runbook are an interface engine's
   job.

A fifth, softer one: this project is new, published in 2026, at 0.x, with a
single maintainer ([`MAINTAINERS.md`](MAINTAINERS.md) states the bus factor
plainly). If your risk posture needs a long production track record, it
does not have one yet. That is a fact about the calendar, and the only
honest thing to do is say so.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
