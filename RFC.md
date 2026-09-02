# Request for comments

What this project does not know, and where an outside opinion changes the
answer.

Most open-source projects ask for feedback in the abstract and get none,
because "feedback welcome" tells nobody what would actually help. This file
is the specific version: the questions that are genuinely open, what kind
of evidence would settle each, and which decisions are closed so nobody
spends an afternoon relitigating them.

Answer any of it by opening an issue at
<https://github.com/hl7-rust/hl7-rust/issues> and citing the section number.
A one-line answer from someone who runs a real interface is worth more than
a long one from someone reasoning from the standard, and if the only thing
you can say is "we send that segment too", that is still an answer.

## Contents

- [What would help most](#what-would-help-most)
- [1. Coverage: what do real feeds actually contain?](#1-coverage-what-do-real-feeds-actually-contain)
- [2. Is "degrade, never reject" right?](#2-is-degrade-never-reject-right)
- [3. Should errors be able to quote patient data?](#3-should-errors-be-able-to-quote-patient-data)
- [4. Vendor dialects: is JSON the right shape?](#4-vendor-dialects-is-json-the-right-shape)
- [5. Is the HL7® v3 foundation worth keeping?](#5-is-the-hl7-v3-foundation-worth-keeping)
- [6. Should the HL7® FHIR® standard be implemented here at all?](#6-should-the-hl7-fhir-standard-be-implemented-here-at-all)
- [7. Does five-way licensing help or confuse?](#7-does-five-way-licensing-help-or-confuse)
- [8. What would make a one-maintainer project adoptable?](#8-what-would-make-a-one-maintainer-project-adoptable)
- [9. Is the tree worth optimising?](#9-is-the-tree-worth-optimising)
- [10. What is missing that nobody has mentioned?](#10-what-is-missing-that-nobody-has-mentioned)
- [11. If HL7® declines naming permission, what should the project be called?](#11-if-hl7-declines-naming-permission-what-should-the-project-be-called)
- [12. What does a legal review need to know about the dictionary data?](#12-what-does-a-legal-review-need-to-know-about-the-dictionary-data)
- [Decided, and not looking for comment](#decided-and-not-looking-for-comment)
- [How feedback gets handled](#how-feedback-gets-handled)

## What would help most

If you read only one section, read §1. Everything else here is a design
question that can wait; §1 is the one where the project is structurally
blind and an outsider can see what the maintainer cannot.

## 1. Coverage: what do real feeds actually contain?

**The question.** Which segments, message structures, and vendor quirks
does your interface send that this project does not model?

**Why it is open.** The bundled dictionary covers 24 segments, 42 composite
data types, and 4 message structures — admissions, orders, results, and
acknowledgements. HL7® v2.5 defines well over a hundred segments and around
eighty structures. Coverage is deliberately added only when a real message
motivates it, because a table transcribed from the standard with no message
behind it is a table nobody can check and no test can defend. That policy
is sound and it has one consequence: **without reports, the dictionary
cannot grow correctly.** No amount of maintainer effort substitutes for
knowing what a Meditech feed in a Dutch hospital actually puts in PV2.

**What would settle it.** Warning counts by kind from `hl7-v2 --check` over
a day of your traffic, and a redacted example of anything that reads
positionally. Segment and structure names alone, with no message at all,
are still useful.

**What this is not.** Not a request for your data. See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for redaction, and
[`spec/phi/index.md`](spec/phi/index.md) for the project's full position.

## 2. Is "degrade, never reject" right?

**The question.** An unmodelled segment, field, type, or structure costs
you a *name*, never a *value*: it parses, reads positionally, raises a
warning, and still round-trips byte for byte. Only a missing MSH header, a
malformed path, an unloadable dictionary, or a struct-mode type mismatch
fails a call.

**Why it is open.** It is the right default for a routing or archiving
system, where dropping a clinical message is the worst outcome. It may be
wrong for a system that must not act on data it did not understand, where
failing loudly is safer than proceeding with a positional name. `strict`
mode exists, but it keys off validation severity rather than off "did I
recognise this at all".

**What would settle it.** A description of a real integration where the
current behaviour is dangerous, or where `strict` is too blunt. Concretely:
would a mode that fails on `SegmentUnknown` be something you would turn on?

## 3. Should errors be able to quote patient data?

**The question.** `Error::BadValue` carries the offending text verbatim,
and a `ValueFormat` diagnostic formats the value into its detail string. So
`format!("{error}")` can put a value from a clinical record into a log.
This is documented rather than fixed.

**Why it is open.** Three options, none obviously right. Keep it, because
the value is often the only way to debug a malformed feed. Remove it, and
lose that. Or split `Display` from a redacting `Display`-like method, which
means an API decision about which one is the default — and defaults are
what people actually get.

**What would settle it.** How your organisation treats logs relative to the
message store, and whether you would rather have a redacted default with an
opt-in to the value, or the reverse.

## 4. Vendor dialects: is JSON the right shape?

**The question.** A dialect is a JSON document that can inherit a bundled
release and state only its differences, loaded at run time. There is also a
generator that writes one from a site's HL7® v2.xml XSD files.

**Why it is open.** Nobody outside the project has reported writing one, so
the ergonomics are untested by anyone who does not already know how the
dictionary works. It is plausible that people want a Rust builder API, or a
simpler format, or that the XSD generator is the only path anyone will ever
use.

**What would settle it.** Try it against a dialect you actually have, and
say where it fought you. A failed attempt is a better report than a
successful one.

## 5. Is the HL7® v3 foundation worth keeping?

**The question.** `hl7-3` implements six RIM backbone classes, six data
types, and the three-level envelope, read generically. It says in its own
first section that it is a foundation, not an implementation. No support
for the Clinical Document Architecture.

**Why it is open.** HL7® v3 is a small and shrinking share of new
integration work. The foundation is either a useful starting point for the
sites still on v3 and national registries that mandate it, or it is a crate
nobody uses that costs maintenance and dilutes the project's focus.
Deprecating it would be honest if it is the latter.

**What would settle it.** Whether anyone is using it, and for what. Silence
here will eventually be read as an answer.

## 6. Should the HL7® FHIR® standard be implemented here at all?

**The question.** Not implemented. The umbrella crate reserves the module
path and nothing more.

**Why it is open.** There are existing Rust efforts, the standard is very
large, and a half-built implementation is worse than none in a domain where
people check conformance. The alternative view is that a v2-and-v3 project
that cannot touch the standard most new work targets is a project with a
ceiling.

**What would settle it.** If you need it here rather than from a dedicated
crate, say why — particularly if the reason is v2-to-FHIR conversion, which
would sit naturally next to the existing conversion crates and is a much
smaller thing to build than the whole standard.

## 7. Does five-way licensing help or confuse?

**The question.** Every crate is MIT, Apache-2.0, BSD-3-Clause,
GPL-2.0-only, or GPL-3.0-only, at your option, so that a proprietary vendor
and a public-sector project can both adopt without asking.

**Why it is open.** The intent is to remove a conversation. The risk is
that it creates one: a legal reviewer who has never seen a five-way `OR`
may treat unfamiliar as suspicious, where a plain `MIT OR Apache-2.0` would
have passed without comment.

**What would settle it.** If you took this through a legal or procurement
review, what happened? A review that stalled on the licence expression is
exactly the evidence needed, and nobody would otherwise report it.

## 8. What would make a one-maintainer project adoptable?

**The question.** [`MAINTAINERS.md`](MAINTAINERS.md) states the bus factor
is one, names every publishing identity, and says there is no release
signing. (It used to say no CI and no SBOM too; a root workflow has run
the [`CONTRIBUTING.md`](CONTRIBUTING.md) gates on every push since
2026-08-26, and a `sbom` job has generated a CycloneDX document per crate
in CI since 2026-09-01 — a CI artifact, not something a crates.io release
itself carries.) crates.io releases still publish with a long-lived API
token rather than Trusted Publishing —
[`spec/trusted-publishing/index.md`](spec/trusted-publishing/index.md)
says why: it is available for GitHub Actions, but GitLab support is
GitLab.com-only beta and Codeberg has none yet, so this project is waiting
for all three rather than adopting it for GitHub alone.
[`SECURITY.md`](SECURITY.md) adds a published policy but still promises
best effort rather than a response window. Since 2026-09-02, one more fact
belongs on this list for a procurement reviewer: an agentic tool the
maintainer directs may decide, on its own judgment, that a change warrants
a release and execute `cargo publish` for it, bounded by
[`spec/release-process/index.md`](spec/release-process/index.md) —
[`AI_STATEMENT.md`](AI_STATEMENT.md) §5 states it as the one `autonomous`
row the document carries.

**Why it is open.** Those gaps are disclosed rather than fixed, and the
disclosure is deliberate. But which of them actually blocks adoption is a
guess. CI is the maintainer's instinct; a procurement reviewer might say
signed releases, an SBOM tied to the released artifact rather than only to
a CI run, a stated response window, a second maintainer, or the
long-lived crates.io token specifically, and nothing else.

**What would settle it.** If this project failed your supplier review, what
failed it? Naming the specific checklist item is more useful than a general
impression, and "we did not even evaluate it because X" is the most useful
answer of all.

## 9. Is the tree worth optimising?

**The question.** Building the generic tree of a 200-observation message
costs about 1.44 ms, against about 3.6 µs to read two fields by path —
nearly 400 times more. It has had no optimisation attention.

**Why it is open.** The guidance is to use paths, and for most integrations
that is enough, which is why the number has been published rather than
chased. But conversion crates walk the whole message by necessity, and if
anyone is running the tree in a hot path, the priority changes.

**What would settle it.** Whether you build the tree per message in
production, and what your message sizes are.

## 10. What is missing that nobody has mentioned?

**The question.** The literal one: what did you expect to find and not
find?

**Why it is open.** Known gaps are easy to list — memory measurement in the
benchmarks, a fair cross-library comparison, CI. Unknown ones are the
expensive kind, and they show up as somebody quietly choosing a different
library and never saying why.

**What would settle it.** If you evaluated this project and chose something
else, that is the single most valuable issue anyone could file, and it will
be read as information rather than as criticism.

## 11. If HL7® declines naming permission, what should the project be called?

**The question.** The project's package names, organization name, and
domain all use the HL7® mark, which is branding rather than fair use, and
[`spec/hl7-trademarks-fair-use/index.md`](spec/hl7-trademarks-fair-use/index.md)
says written consent is expected for that. The README states permission is
being requested; [`plan.md`](plan.md) §Open decisions records that the
request's status — sent, answered — is written down nowhere, which is
itself the first thing to fix.

**Why it is open.** If HL7® grants permission, nothing moves. If it
declines, fourteen published crate names, a GitHub organization, and a
Pages domain all reopen at once — and renaming published crates is the
kind of change that strands existing users, so the fallback should be
chosen before it is needed, not during a takedown letter.

**What would settle it.** Two things: experience — if your project used a
standards body's mark in its name and asked, what happened? — and
preference: as a user, would you rather see a rename land early and once,
or only if forced?

## 12. Is the dictionary data's provenance statement good enough?

**The question.** The bundled release dictionaries,
`hl7-2/schemas/v2.1.json` through `v2.9.json`, describe HL7® v2 segments,
data types, and message structures.
[`spec/schema-data-provenance/index.md`](spec/schema-data-provenance/index.md)
now traces them as far back through git history as the trail goes — to a
founding commit that cites no source — and states plainly that the trail
ends there. No HL7® file is vendored anywhere in this workspace, and the
document gives the idea/expression reasoning most open HL7® implementations
rely on for why that matters.

**Why it is open.** Writing the document closed the "where does this claim
come from" gap; it did not close the "is this legally sufficient" question,
because that answer belongs to whoever is reviewing it, not to the project
describing itself. The document says outright that a reviewer requiring
HL7® membership or a licensed XSD as the basis for any table will not be
satisfied by it, and offers the XSD-generator path as the alternative for
that reviewer.

**What would settle it.** If this project has been through your legal or
procurement review: did the provenance document hold up, what did the
review ask for beyond it, and what would have made the answer complete on
the first pass?

## Decided, and not looking for comment

These are settled, with the reasoning written down. Comments are welcome if
you have evidence the reasoning is *wrong*, but not simply that you would
have chosen differently.

| Decision | Where the reasoning is |
|---|---|
| MSRV is current stable minus two releases | [`spec/rust-msrv-n-minus-2/index.md`](spec/rust-msrv-n-minus-2/index.md) |
| The spec is the source of truth; behaviour changes go there first | each crate's `spec/index.md` |
| Coverage grows from real messages, never from transcribing the standard | §1 above, and [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| Byte-for-byte round trip, and the explicit null kept distinct | [`spec/conformance/index.md`](spec/conformance/index.md) |
| No AI ships in these crates; AI is used to build them | [`AI_STATEMENT.md`](AI_STATEMENT.md) |
| Benchmarks publish their method, including the slowest operation | [`spec/benchmark/index.md`](spec/benchmark/index.md) |
| One dependency, no runtime, no logging, no telemetry | [`spec/phi/index.md`](spec/phi/index.md) |
| English prose uses the serial comma | [`spec/serial-comma/index.md`](spec/serial-comma/index.md) |
| CI is one root workflow over the whole workspace, running the CONTRIBUTING.md gates | [`plan.md`](plan.md) §Open decisions, decided 2026-08-26 |

## How feedback gets handled

Plainly, because a project that asks for comment and then absorbs it
silently teaches people not to bother:

- **It gets answered on the tracker**, in public, where the reasoning is
  reachable by the next person with the same question.
- **A comment that changes a decision changes a document too.** The specs
  are the source of truth, so a design change that is not written down did
  not happen.
- **Disagreement is fine and stays on the tracker.** Where this file says
  something is decided, that is a statement about the current reasoning,
  not about who is allowed to question it.
- **No response-time promise.** [`MAINTAINERS.md`](MAINTAINERS.md) explains
  why: one person, no on-call. Silence means nobody has got to it, never
  that the question was unwelcome.
- **Credit where it is due.** A report that changes the code gets named in
  the changelog entry, unless you would rather it did not.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
