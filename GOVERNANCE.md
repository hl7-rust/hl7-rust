# Governance

How decisions get made in HL7® for Rust, who makes them, and how that
changes.

The short version: **one person decides.** This document exists not to
dress that up as a structure, but to make it predictable — so a contributor
knows what will happen to their pull request, and a reviewer knows what
they are depending on.

## Who decides

Joel Parker Henderson, sole maintainer. [`MAINTAINERS.md`](MAINTAINERS.md)
is the roster and is candid that the bus factor is one: one person can
merge, one can change a setting, and one person's credential and authority
stand behind every release — since 2026-09-02, an agentic tool he directs
may execute a release on his own judgment, within
[`spec/release-process/index.md`](spec/release-process/index.md)'s bounds,
but the decision to grant that scope, and everything the tool is bounded
by, is still this one person's.

There is no steering committee, no vote, no technical board, and no legal
entity. Where other projects would say "the maintainers decide by
consensus", here it is one person, and pretending otherwise would mislead
exactly the people — hospital and vendor reviewers — most likely to read
this file.

## What the decision-maker is bound by

Being sole maintainer is not the same as being unconstrained. Four things
bind a decision, and they bind the maintainer as much as anyone:

1. **The specs are the source of truth.** Each crate's `spec/index.md` is
   numbered so it can be cited. Behaviour changes go in the spec *first*; a
   code change contradicting the spec is a bug in one of the two, never a
   silent redefinition. This is the constraint that matters most, because
   it means a decision has to be written down before it can ship.
2. **Every rule is backed by a test.** A spec claim with no test is a
   defect in the spec.
3. **Coverage grows from real messages**, never from transcribing the
   standard. A table with no message behind it is a table nobody can check.
4. **Published claims stay true.**
   [`spec/conformance/index.md`](spec/conformance/index.md),
   [`spec/phi/index.md`](spec/phi/index.md), and
   [`spec/benchmark/index.md`](spec/benchmark/index.md) are written to be
   checked rather than believed. A change that makes one of them false must
   change the document in the same commit.

A maintainer who ignores these is wrong in a way anyone can demonstrate,
which is the only accountability a one-person project can offer.

## How a decision gets made

- **Small and reversible** — a bug fix, a doc correction, a dictionary
  entry backed by a real message: just do it. No discussion needed, and
  waiting for one wastes everybody's time.
- **Affects published behaviour** — an API change, a rename, a new error
  variant: it goes in the spec, with the reasoning, in the same change that
  ships it. The commit message carries the *why*; the spec carries the
  *what*.
- **Changes what the project is** — a new crate, a dropped crate, a
  dependency, a licence change, a raised MSRV: an issue first, open for
  comment, before code exists. [`RFC.md`](RFC.md) lists the ones currently
  open.
- **Contested** — the maintainer decides, states why in public, and the
  reasoning is on the tracker where the next person can find and challenge
  it. "Because I said so" is not an acceptable recorded reason.

Where decisions are recorded, in order of authority: the crate's
`spec/index.md`, then [`CHANGELOG.md`](CHANGELOG.md), then the issue
tracker. A decision that exists only in someone's head, or only in a tool
session, is not a decision this project made.

## Scope: what belongs here

The project is HL7® message handling in Rust, one crate per layer, one
module per standard.

**In scope:** parsing, navigating, validating, modifying, and rendering
messages; the dictionaries that give fields meaning; transports that carry
messages; conversions between the encodings HL7® itself defines.

**Out of scope**, and each for a reason rather than by accident:

- **Clinical semantics.** Nothing here knows what an `A08` should do to a
  patient record. That belongs to the system using these crates.
- **Terminology.** LOINC, SNOMED CT, ICD, and the HL7® tables are large,
  separately licensed, and a different problem.
- **An interface engine.** Routing, retries, queues, monitoring, and
  on-call are a product, not a library. See
  [`COMPARISONS.md`](COMPARISONS.md).
- **Anything requiring a runtime.** No async runtime, no framework, no
  service.

Whether the HL7® v3 foundation and the HL7® FHIR® standard belong in scope
is genuinely undecided — [`RFC.md`](RFC.md) §5 and §6.

Two neighbours are deliberately *not* governed here.
[`er7`](https://github.com/er7-rust/er7-rust) is a separate project with
its own repository, and encoding-level decisions are made there. The HL7®
standards themselves belong to Health Level Seven International; this
project implements them and has no say in them, which is also why
[`spec/hl7-trademarks-fair-use/index.md`](spec/hl7-trademarks-fair-use/index.md)
exists.

## Contributions

Anyone may contribute; see [`CONTRIBUTING.md`](CONTRIBUTING.md), which
covers time, code, reports from a real feed, and money. No contributor
licence agreement, no copyright assignment. A contribution is offered under
the same five licences as everything else, so the choice stays available
downstream.

**A pull request is accepted when** it passes the gates in
[`CONTRIBUTING.md`](CONTRIBUTING.md), it is consistent with the crate's
spec (or changes the spec deliberately, in the same commit), and the
maintainer understands it well enough to maintain it after you leave. That
last clause is the one that most often blocks otherwise good work, and it
is stated so that it is not a surprise.

**A pull request may be declined** for being out of scope, for adding a
dependency the project does not want, or for being a change the maintainer
cannot commit to maintaining. A decline comes with a reason on the tracker,
and "not now" is a legitimate reason as long as it is said out loud.

## Becoming a maintainer

The route is open, and deliberately informal because a formal ladder for a
project with one person on it would be theatre.

It starts with sustained, reviewed contributions — enough that the
maintainer trusts your judgement on changes you did not write, which is
the actual job. Dictionary coverage and answering other people's issues are
the two clearest paths, because both demonstrate judgement rather than only
output. Then it is a conversation, and you may start it.

When someone takes it: [`MAINTAINERS.md`](MAINTAINERS.md) gains a row,
`CODEOWNERS` gains their identity on the areas they own, the publishing
identities gain a second holder wherever they permit one, and this document
gains a section on how two people decide when they disagree — because at
that point the question is real and today it is not.

## Disagreement

Argue on the tracker, in public. The specs are the referee: a claim that
the code contradicts its spec, or that a spec contradicts the standard, is
settleable by reading. A claim that a decision is *unwise* is not settleable
that way, and the maintainer decides, in public, with reasons.

If a decision is one you cannot live with, the licence is five-way and the
history is public and mirrored on three hosts. A fork is a legitimate
outcome and not a hostile one; the specs exist substantially so that a fork
is maintainable by someone who never spoke to the author.

## If the maintainer stops

[`MAINTAINERS.md`](MAINTAINERS.md) covers this in full, and does not
pretend a document can create a succession plan. In short: what is
published stays published, nothing new ships, and a fork is the intended
continuation rather than a last resort.

Security reports have their own escape hatch, which does not depend on
anyone's availability: if a private report gets no response within 14 days,
[`SECURITY.md`](SECURITY.md) already tells you to publish.

## Changing this document

Like anything else here: a pull request, with the reasoning. Until a second
maintainer exists, the maintainer decides — including about this file,
which is exactly the kind of circularity a one-person project cannot
escape, and is better stated than hidden.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
