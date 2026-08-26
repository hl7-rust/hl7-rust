[hl7-rust](../../README.md) → help → Outreach

# Promoting HL7® for Rust to professionals

Research, as of 2026-08-25, into where the people who would use these
crates actually gather, what reaches them, and in what order to try. This
is a reference document, not a commitment: nothing here is normative for
the code.

The short version: there are three separate audiences with almost no
overlap, one unusually good timing hook, and one channel — trade press —
that is far less useful than it looks.

## Contents

- [The three audiences](#the-three-audiences)
- [The timing hook](#the-timing-hook)
- [What we are actually promoting](#what-we-are-actually-promoting)
- [Prerequisites before any promotion](#prerequisites-before-any-promotion)
- [Channel: the Rust community](#channel-the-rust-community)
- [Channel: healthcare integration practitioners](#channel-healthcare-integration-practitioners)
- [Channel: HL7 International itself](#channel-hl7-international-itself)
- [Channel: conferences and talks](#channel-conferences-and-talks)
- [Channel: email](#channel-email)
- [Channel: press and analysts](#channel-press-and-analysts)
- [Channel: foundations](#channel-foundations)
- [Trademark, first](#trademark-first)
- [A ninety-day sequence](#a-ninety-day-sequence)
- [What not to do](#what-not-to-do)
- [Measurement, and today's baseline](#measurement-and-todays-baseline)
- [Sources](#sources)

## The three audiences

They want different things, read different places, and are persuaded by
different sentences. Writing one announcement for all three produces
something that lands with none of them.

**1. Healthcare integration engineers and interface analysts.** The people
who keep ADT, ORU, and SIU feeds running between an EHR and everything
else. They are the primary users. They mostly do not read Rust news, they
are pragmatic about tooling, and they judge a library by whether it round
trips a real vendor message without mangling it. They are found in the
Mirth and Open Integration Engine communities, on vendor developer
communities, and on the HL7 Zulip.

**2. Rust developers building in health tech.** A smaller group, but the
ones who will file good issues, send patches, and pull these crates into
something. They are found in the usual Rust places, and they are reachable
by a well-written technical post in a way the first group is not.

**3. Engineering leadership at device makers and health-tech companies.**
They do not adopt a crate; they approve a direction. Their interest is
memory safety, supply chain, licensing, and maintenance risk. They are
reached through conference talks, trade press, and the safety-critical
Rust conversation — not through a crates.io release.

## The timing hook

In March 2025 NextGen Healthcare moved Mirth Connect to a commercial-only
license as of version 4.6; 4.5.2 was the last release under MPL-2.0. Within
about a week, veterans of that community forked 4.5.2 as
[Open Integration Engine](https://github.com/OpenIntegrationEngine) (OIE),
a vendor-neutral MPL-2.0 project with public governance, its own
[docs site](https://docs.openintegrationengine.org/), and a Discord.

That matters here more than any marketing idea in this document. The single
largest population of open-source HL7 v2 practitioners spent 2025 and 2026
re-evaluating what their interface tooling is built on and who controls it.
A permissively multi-licensed, dependency-light, actively maintained HL7 v2
stack arrives into an audience that is already asking the question. The
pitch to them is not "Rust is fast"; it is "here is another piece of
tooling you can own outright."

The corollary is that OIE and the Mirth forums are the *first* healthcare
channel to approach, not the fifth — and that the approach should be a
contribution to their conversation (an MLLP interop note, a conformance
comparison, an offer to test against their sample corpus) rather than an
announcement in their space.

## What we are actually promoting

Assets that exist today:

- Fifteen published crates on crates.io, plus `er7` outside the workspace.
- A CLI (`hl7-v2`), which is the single easiest thing for a non-Rust
  integration engineer to try — no toolchain, no dependencies, pipe a
  message in.
- <https://hl7-rust.github.io>: docs, guides, tutorials, examples, and a
  reference page per crate.
- An unusual scope claim: releases 2.1 through 2.9, three parsing modes,
  MLLP and SOAP transports, ER7 ↔ JSON ↔ XML conversion, dictionaries
  generated from the official v2.xml XSDs, and an HL7 v3 foundation.

Six assets this document originally listed as missing were built on
2026-08-26, because they are what each audience asks for first:

| Asset | Where it landed | Who asks for it |
|---|---|---|
| A news route | `/news/`, with a first post | Everyone — somewhere to publish that is not someone else's platform |
| A comparison page | `/docs/comparison/` | Integration engineers, evaluators |
| Benchmarks with a method | `spec/benchmark/index.md`, `hl7-2/benches/parse.rs`, `/docs/benchmarks/` | Rust readers, and anyone skeptical of a performance claim |
| A PHI position | `spec/phi/index.md`, `/docs/patient-data/` | Anyone in a regulated environment |
| `CONTRIBUTING.md` | Workspace root | Would-be contributors, work group co-chairs |
| A conformance statement | `spec/conformance/index.md`, `/docs/conformance/` | HL7 people, who ask this immediately and are unimpressed by hand-waving |

Three of them are the ones that matter for the pitch, and each is
deliberately unflattering in at least one place — a conformance statement
that publishes "24 segments", a benchmark page that names its own slowest
operation, and a comparison page with four cases where the reader should
choose something else. That is the point: an evaluator who finds a
limitation stated before they found it themselves reads the rest of the
claims differently.

Still missing, and worth building next:

| Missing | Who asks for it |
|---|---|
| A memory measurement in the benchmarks | Anyone sizing a deployment; the current figures are time-only, and the benchmark spec says so |
| A fair cross-library comparison, with method | Every evaluation; hard to do honestly, which is why it does not exist |
| An announcement list | The one channel not mediated by someone else's platform |

## Prerequisites before any promotion

A launch that succeeds and then strands people is worse than no launch.
Before pointing anyone at this:

- Every crate's docs.rs build is green, and the front page of each crate's
  docs answers "what is this and what do I type first" in the first screen.
- The umbrella `hl7` crate reads as current at a glance. The name was
  claimed in 2019 by an unrelated "An HL7 2.x parser" at 0.0.1 and 0.0.2,
  and this project's 0.1.0 and 0.1.1 landed in August 2026 — so the
  crates.io page shows a 2019 creation date beside a 0.1.x version, which
  a skimming evaluator can read as seven years of neglect. The version
  history and the README have to say otherwise, loudly.
- A `CHANGELOG.md` per crate, or one for the workspace.
- ~~Issue templates, and a stated response expectation.~~ Done 2026-08-26:
  `.github/ISSUE_TEMPLATE/` (bug report, wrong claim, and a config pointing
  security reports at the private channel), and `MAINTAINERS.md` states the
  read-within-a-week expectation. The fastest way to lose the integration
  audience is an unanswered issue about a vendor dialect.
- The trademark question is answered or in flight — see
  [Trademark, first](#trademark-first). It gates the rest.
- The MSRV policy is already documented
  ([`spec/rust-msrv-n-minus-3`](../../spec/rust-msrv-n-minus-3/index.md)) and is
  genuinely a selling point to this audience — hospital toolchains move in
  quarters. Say so explicitly in promotional copy.

## Channel: the Rust community

Cheapest, fastest, and the right place to start, because the failure modes
are recoverable and the feedback improves the pitch before it reaches
healthcare.

**This Week in Rust.** Accepts both self-suggestions and community
nominations for *Crate of the Week*, and runs a *Call for Participation*
section where a project can list good first issues. Both are worth doing;
the second is repeatable and the better long-term play, since it recruits
rather than merely announces. Suggestions go through the
[this-week-in-rust](https://github.com/rust-lang/this-week-in-rust)
repository.

**r/rust.** Tolerant of project posts that read as engineering write-ups
and hostile to posts that read as advertising. The post that works is
"what I learned implementing the HL7 v2 dictionary from the official XSDs"
with the crate as the footnote — not "announcing hl7-2".

**users.rust-lang.org**, in the *Show and tell* category. Low traffic, high
signal, permanent.

**Hacker News**, as a `Show HN`. Neutral title, no hype and no exclamation
points, no solicited upvotes, and one calm first comment from the author
within a few minutes giving context: who you are, the specific problem,
and what is technically interesting. Earlier in the week does better. HN's
health-IT readership is real and includes exactly audience 3.

**Lobsters**, if an invite is available — it is invite-only, and posting
your own work without participating first goes badly.

**Curated lists.** Submissions are cheap, permanent, and how people find
things years later:

- [awesome-rust](https://github.com/rust-unofficial/awesome-rust)
- [kakoni/awesome-healthcare](https://github.com/kakoni/awesome-healthcare)
  — already has an HL7 v2 section
- [fhir-fuel/awesome-FHIR®](https://github.com/fhir-fuel/awesome-FHIR)
- [jcfr/awesome-health](https://github.com/jcfr/awesome-health)

**GitHub topics.** Free discovery: `hl7`, `hl7v2`, `hl7-v2`, `mllp`,
`healthcare-interoperability`, `health-informatics`, `ehr`, `rust`.

**Positioning against prior art.** The existing Rust options are thin and
mostly dormant, which is a fair and checkable claim rather than a swipe:

| crate | latest | last published | downloads |
|---|---|---|---|
| `hl7-mllp-codec` | 0.4.0 | 2022-07-22 | 25,755 |
| `hl7-parser` | 0.3.0 | 2025-02-24 | 16,625 |
| `rust-hl7` | 0.5.0 | 2021-09-08 | 14,777 |
| `fhirbolt` | 0.4.0 | 2023-05-17 | 12,613 |

Those download counts are also the honest size of the addressable Rust HL7
audience today: low tens of thousands of pulls, accumulated over years.
Expect the Rust channel to produce credibility and contributors, not
volume.

## Channel: healthcare integration practitioners

Where the actual users are. Slower, and it rewards showing up repeatedly
rather than announcing once.

**Open Integration Engine.** GitHub org
[OpenIntegrationEngine](https://github.com/OpenIntegrationEngine), docs at
[docs.openintegrationengine.org](https://docs.openintegrationengine.org/),
and an active Discord. The highest-value target in this document. Best
first move: participate on an interop question, not a launch post.

**Mirth Community forums** ([forums.mirthproject.io](https://forums.mirthproject.io/)).
Thousands of members and years of archived threads on channel
configuration, transformation, and performance. Answering HL7 v2 encoding
questions there — the ER7 escaping and delimiter corner cases these crates
already handle correctly — builds standing that an announcement cannot.

**InterSystems Developer Community**
([community.intersystems.com](https://community.intersystems.com/tags/hl7)).
A large working population of v2 practitioners around IRIS for Health and
Ensemble. Vendor-run, so the etiquette is contribute-first, and a competing
product pitch will not land — but a language-agnostic technical article on
v2 encoding will.

**Reddit.** `r/healthIT` for practitioners, and the health informatics
subreddits. Treat as a listening channel first: what people ask about is
better documentation input than promotion output.

**LinkedIn.** Genuinely the healthcare-IT professional network, unlike in
most software niches. Interface analysts, integration leads, and HL7
consultants are all there and post about tooling. Practical use: the
author's own posts, plus HL7 and healthcare-interoperability groups.
Consultancies specializing in Mirth and HL7 integration are visible there
and are a plausible early adopter segment — they build bespoke tooling and
own their stack.

## Channel: HL7 International itself

The highest-credibility channel and the slowest. Membership in the HL7
Working Group is open to anyone willing to volunteer.

**chat.fhir.org.** Note the consolidation: `chat.hl7.org` has been retired
and all HL7 community chat now lives at
[chat.fhir.org](https://chat.fhir.org), Zulip, despite the FHIR-specific
name. It has v2 and implementer streams. Read the
[community expectations](https://confluence.hl7.org/spaces/FHIR/pages/76158463/Chat.fhir.org+Community+Expectations)
before posting; this community is unforgiving about broadcast messages and
about `@all` on large streams.

**Work groups.** The two that matter for this project:

- [Infrastructure and Messaging (InM)](https://www.hl7.org/special/committees/inm/index.cfm)
  — the infrastructure that lets systems exchange v2 content.
- [Version 2 Management Group (V2MG)](https://confluence.hl7.org/spaces/V2MG/overview)
  — quality criteria for the v2 products, with Conformance, TSMG, and InM.

Joining a call and being a useful implementer voice is worth more than any
number of posts. Implementers who find genuine ambiguities in the standard
are welcome there, and this project's dictionary generation from the
official XSDs is exactly the kind of work that surfaces them.

**Confluence** ([confluence.hl7.org](https://confluence.hl7.org/)) hosts
work group pages and sample v2 message sets — the sample corpus is both a
test asset and a credibility asset if conformance results are published.

## Channel: conferences and talks

A talk is the single highest-leverage artifact: it reaches audience 3,
produces a recording that works for months, and converts into press more
reliably than a pitch does.

**Healthcare side.**

- HL7 Working Group Meetings, three times a year, with connectathons. The
  40th Annual Plenary and WGM is 19–25 September 2026 in Rockville, MD;
  then 15–21 May 2027 in Denver, CO; then the 41st Annual, 18–24 September
  2027 in Dallas, TX.
- HL7 FHIR DevDays — Amsterdam, June 2027; the program is still forming, so
  the submission window is open in a way the 2026 one is not.
- HIMSS, for audience 3 specifically. Expensive, and the right use is
  hallway conversations and the interoperability showcase, not a booth.

**Rust side.** As of today, the 2026 CFPs have closed:

- RustConf 2026 — 8–11 September 2026, Montréal and online. CFP closed
  16 February 2026; attendable, not speakable.
- EuroRust 2026 — 14–17 October 2026, Barcelona and online.
- RustWeek 2026 — Utrecht; CFP closed 31 December 2025.
- Rust Nation UK — London, February; 2026 CFP closed.

So the realistic target is the 2027 cycle, which means having a talk-shaped
story ready by late 2026: "HL7 v2 in Rust, and what a fifty-year-old
pipe-delimited standard teaches you about parser design" is a better
proposal to a Rust audience than a library tour.

**Podcasts.** [corrode.dev](https://corrode.dev/)'s *Rust in Production*
interviews teams shipping Rust in real industries and has an obvious gap
where healthcare should be. A single episode reaches audiences 2 and 3 at
once and is far easier to obtain than press coverage.

## Channel: email

Direct email works here precisely because the audience is small and
identifiable. Volume email does not, and would damage standing in
communities that are small enough to notice.

Worth a personal, individually written message:

- Maintainers of adjacent open-source projects — OIE, HL7 test-tooling
  projects, FHIR converters — offering interop or a specific contribution.
- HL7 work group co-chairs, once there is a concrete implementer finding to
  bring, not before.
- Integration consultancies that build custom HL7 tooling.
- Academic health informatics programs, where a permissively licensed,
  readable reference implementation is teaching material.
- Authors of prior Rust HL7 crates — a courteous note about consolidation
  is both decent practice and how ecosystems avoid duplicated effort.

Structure that works: one sentence on who you are, one on the specific
thing you noticed about *their* work, one on what you built, one concrete
offer, and a link. Under 150 words. No attachments, no deck, no follow-up
sequence.

An announcement list on the site is worth adding — a low-volume release
list is the one channel not mediated by someone else's platform. Anything
resembling a purchased or scraped list is out of scope and off the table.

## Channel: press and analysts

Set expectations honestly: **trade press does not cover libraries.** The
outlets below cover funding, regulation, vendor moves, breaches, and
deployments at named institutions. A crate release is none of those, and
pitching one as news burns the contact.

The realistic paths, in order of likelihood:

1. **A contributed or bylined article** on memory safety in health IT, or
   on what the Mirth license change means for open-source interoperability
   tooling. Several outlets take contributed pieces from practitioners.
2. **Being the expert quoted** in someone else's story about
   interoperability tooling or open-source health infrastructure — which
   follows from a conference talk, not from an email.
3. **An actual news event**: a named health system or device maker in
   production, a foundation adopting the project, or a certification
   result. Then it is a story.

Outlets and who covers this beat:

- [HIStalk](https://histalk2.com/) — the one the industry actually reads
  daily; opinionated, and covers open source more readily than the others.
- [Healthcare IT News](https://www.healthcareitnews.com/topics/interoperability)
  (HIMSS) — a standing interoperability topic section.
- [Fierce Healthcare](https://www.fiercehealthcare.com/) — Heather Landi
  covers digital health and health IT.
- Health Data Management, and the health-IT trade press generally.
- ARPR maintains a [health-IT reporters list](https://arpr.com/blog/health-it-reporters/);
  [Muck Rack](https://muckrack.com/media-outlet/healthcareitnews) has
  current bylines and contacts per outlet.

Rust-side press is easier and more receptive: the Rust Foundation's
channels, InfoQ's Rust coverage, and language-community newsletters will
take a genuine "Rust in a new domain" story.

## Channel: foundations

Two live opportunities, both about durability rather than reach — a
foundation home answers the "what if the author stops" question that every
serious healthcare evaluator asks:

- [Linux Foundation Public Health](https://www.lfph.io/) — builds and
  sustains open-source software for digital health.
- The **Open Health Stack Software Foundation**, which the Linux Foundation
  announced its intent to launch on 9 July 2026, with more than twenty
  supporting organizations and a stated focus on health data built on the
  HL7® FHIR® standard. New foundations recruit projects; being early is easier than
  being late.

Related and relevant to audience 3: the Rust Foundation's
[Safety-Critical Rust Consortium](https://rustfoundation.org/media/announcing-the-safety-critical-rust-consortium/)
(AdaCore, Arm, Ferrous Systems, OxidOS, Woven by Toyota, and others), and
the Rust Project's January 2026 post
[What does it take to ship Rust in safety-critical?](https://blog.rust-lang.org/2026/01/14/what-does-it-take-to-ship-rust-in-safety-critical/).
Health software is adjacent to that conversation and largely absent from
it, which is an opening for a talk or an article.

## Trademark, first

Handle this before any of the above. Promotion raises visibility, and
visibility is what turns a latent trademark question into an active one.

HL7 International's
[Guide to Using HL7 Trademarks](https://www.hl7.org/legal/trademarks.cfm)
and its
[trademark guidelines](https://info.hl7.org/hl7-trademark-guidelines)
state that using HL7 trademarks to brand a product or service, as part of
the product name, without express written consent is prohibited, and that
the marks may not be abbreviated or combined with other words without
explicit written permission. The
[FHIR Trademark Policy](https://confluence.hl7.org/display/FHIR/FHIR+Trademark+Policy)
is similarly explicit that product names and domain URLs go past fair use
and need a product-use license.

This project is `hl7-rust`, publishing crates named `hl7`, `hl7-2`, and
`hl7-3`, from `hl7-rust.github.io`. That is the mark in the organization
name, in the package names, and in the domain. Whether that is permitted as
descriptive use, needs a license, or needs a rename is a question for HL7
rather than one to guess at — and the plain reading of the published
guidance is that written consent is expected.

The productive move is to ask early and in good faith: a short note
describing a free, permissively multi-licensed, open-source implementation
of the published standard, asking what naming and attribution they want.
Standards bodies generally want implementations to exist, and a
conversation the project starts reads very differently from one their
counsel starts. HL7 runs a
[trademark licensing site](http://www.hl7.org/community-use/index.cfm) and
a
[product trademark application](http://www.hl7.org/about/product.trademark.application.cfm)
for exactly this.

Carrying a notice is worth doing either way, because it is cheap, correct
regardless of the answer, and exactly what a reviewer at a hospital or a
device maker looks for. The one this project adopted, at the top of every
README and in `LICENSE.md`, is:

> HL7® and FHIR® are registered trademarks of HL7. We are requesting
> permission to use it here. Use of the trademarks does not constitute
> endorsement of this library by HL7.

Two gaps in it are worth knowing rather than discovering later. It says
nothing about *certification*, which a reviewer also asks about — that is
answered in [`spec/conformance/index.md`](../../spec/conformance/index.md),
whose first lines state that no certifying body has assessed this project.
And its coverage has widened since this document was first written: as of
2026-08-26 the disclaimer is carried in the website footer (every rendered
page) and in every crate root's rustdoc, which is what docs.rs renders —
not only in the READMEs and the licence. The `Cargo.toml` `description`
strings remain outside the checked scope, recorded as such in
[`spec/hl7-trademarks-fair-use/index.md`](../../spec/hl7-trademarks-fair-use/index.md).

## A ninety-day sequence

Ordered so that each phase's feedback improves the next, and so the
irreversible channels come last.

**Days 1–14, fix the prerequisites.** Six of these are done as of
2026-08-26 — see the table in
[What we are actually promoting](#what-we-are-actually-promoting). What
remains is the list in
[Prerequisites](#prerequisites-before-any-promotion): docs.rs polish, the
`hl7` crate's stale-looking crates.io page, and changelogs (issue templates
and the response expectation landed 2026-08-26). Send the trademark note to
HL7 — it is the only item with an
external dependency, so it starts first and runs in the background while
everything else proceeds. Nothing else external.

**Days 15–30, quiet seeding.** Curated-list PRs, GitHub topics, docs.rs
polish, `users.rust-lang.org` Show and tell, This Week in Rust *Call for
Participation* with two or three genuinely approachable issues. Start
reading the OIE Discord and the Mirth forums without posting.

**Days 31–50, the Rust community properly.** Publish the first technical
article on the project's own site — the XSD-to-dictionary pipeline, or ER7
escaping corner cases, both of which are real and interesting. Then r/rust
and Show HN pointing at the article. Nominate for Crate of the Week. Absorb
the feedback and fix what it exposes before going further.

**Days 51–75, healthcare practitioners.** Begin contributing answers on the
Mirth forums and in the OIE Discord. Publish a conformance or round-trip
comparison against a public sample corpus. Post the same article, rewritten
for a non-Rust audience, on LinkedIn. Individually written emails to
adjacent maintainers.

**Days 76–90, standards body and durability.** Join a chat.fhir.org v2
stream and introduce the project once, briefly, in the right place. Attend
an InM or V2MG call. Draft a 2027 conference proposal. Open a conversation
with LFPH or OHS-SF. Only now approach a podcast, and only then consider a
contributed article pitch to a trade outlet.

## What not to do

- **Do not announce the same text in five places on one day.** Every
  community here can see the others.
- **Do not post any real PHI**, in examples, in issues, in benchmarks, or
  in a bug report from a user. Have a stated policy and a scrubbing note in
  the issue template before inviting bug reports.
- **Do not claim conformance, certification, or compliance** that has not
  been demonstrated. This audience checks, and one overstatement ends the
  project's credibility with the people who matter most.
- **Resolve the trademark question before promoting, not after.** It is
  the one item here that could stop everything else — see
  [Trademark, first](#trademark-first).
- **Do not pitch a library as news** to a trade reporter.
- **Do not disparage Mirth, HAPI, or the existing Rust crates.** The people
  who maintain them are the audience, and much of the community has years
  invested in them.
- **Do not promise support you cannot deliver.** Better to state plainly
  that this is a small project with a defined scope than to imply a vendor
  relationship.

## Measurement, and today's baseline

crates.io figures for 2026-08-25, so later movement is attributable:

| crate | version | created | total downloads | recent |
|---|---|---|---|---|
| `hl7` | 0.1.1 | 2019-05-09 | 3,799 | 51 |
| `er7` | 0.1.2 | 2026-08-15 | 160 | 160 |
| `hl7-2` | 0.2.3 | 2026-08-19 | 99 | 99 |
| `hl7-3` | 0.1.3 | 2026-08-19 | 72 | 72 |

Worth tracking, in rough order of how much they mean:

1. Issues and pull requests from people who are not the author — the only
   metric that indicates the project has become somebody else's.
2. Inbound questions that reveal real use ("our vendor sends this, does
   your parser…").
3. Reverse dependencies on crates.io.
4. crates.io recent downloads, against the baseline above and against the
   prior-art numbers.
5. Site traffic and referral sources, to learn which channels carried.
6. GitHub stars, last, as the weakest signal of the set.

## Sources

Research links behind the above, retrieved 2026-08-25.

HL7 International:
[events](https://hl7.org/events/),
[Work Group Meetings](https://www.hl7.org/events/workgroupmeetings.cfm),
[FHIR DevDays](https://www.hl7.org/events/fhir-devdays.cfm),
[Infrastructure and Messaging](http://www.hl7.org/special/committees/inm/index.cfm),
[V2 Management Group](https://confluence.hl7.org/spaces/V2MG/overview),
[Confluence](https://confluence.hl7.org/),
[chat.hl7.org retirement notice](https://chat.hl7.org/),
[chat.fhir.org community expectations](https://confluence.hl7.org/spaces/FHIR/pages/76158463/Chat.fhir.org+Community+Expectations).

Practitioner communities:
[Open Integration Engine](https://github.com/OpenIntegrationEngine),
[OIE docs](https://docs.openintegrationengine.org/),
[OIE announcement on the Mirth forums](https://forums.mirthproject.io/forum/mirth-connect/development/186147-announcing-open-integration-engine),
[Mirth Connect history and license change](https://en.wikipedia.org/wiki/Mirth_Connect),
[InterSystems Developer Community, HL7 tag](https://community.intersystems.com/tags/hl7).

Rust community:
[This Week in Rust](https://this-week-in-rust.org/),
[RustConf 2026 CFP](https://rustfoundation.org/media/the-rustconf-2026-call-for-proposals-is-open/),
[EuroRust 2026](https://eurorust.eu/),
[RustWeek 2026 CFP](https://2026.rustweek.org/cfp/),
[Rust Nation UK](https://www.rustnationuk.com/),
[Rust conferences 2026, corrode.dev](https://corrode.dev/blog/rust-conferences-2026/),
[awesome-healthcare](https://github.com/kakoni/awesome-healthcare),
[awesome-FHIR](https://github.com/fhir-fuel/awesome-FHIR),
[awesome-health](https://github.com/jcfr/awesome-health).

Safety-critical and foundations:
[Safety-Critical Rust Consortium](https://rustfoundation.org/media/announcing-the-safety-critical-rust-consortium/),
[What does it take to ship Rust in safety-critical?](https://blog.rust-lang.org/2026/01/14/what-does-it-take-to-ship-rust-in-safety-critical/),
[Linux Foundation Public Health](https://www.lfph.io/),
[Open Health Stack Software Foundation announcement](https://www.linuxfoundation.org/press/linux-foundation-announces-intent-to-launch-open-health-stack-software-foundation-to-advance-open-source-digital-health-innovation).

Trademark:
[Guide to Using HL7 Trademarks](https://www.hl7.org/legal/trademarks.cfm),
[HL7 trademark guidelines](https://info.hl7.org/hl7-trademark-guidelines),
[FHIR Trademark Policy](https://confluence.hl7.org/display/FHIR/FHIR+Trademark+Policy),
[HL7 trademark licensing](http://www.hl7.org/community-use/index.cfm),
[HL7 brand style guide](https://brandguide.hl7.org/identity/).

Press:
[HIStalk](https://histalk2.com/),
[Healthcare IT News interoperability](https://www.healthcareitnews.com/topics/interoperability),
[Fierce Healthcare](https://www.fiercehealthcare.com/),
[ARPR health IT reporters](https://arpr.com/blog/health-it-reporters/),
[Muck Rack, Healthcare IT News](https://muckrack.com/media-outlet/healthcareitnews),
[Hacker News posting guide](https://syften.com/blog/hacker-news-marketing/).

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
