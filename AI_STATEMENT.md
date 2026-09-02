# AI statement

| | |
|---|---|
| Version | 2.0.0 |
| Effective date | 2026-09-02 |
| Status | Active |
| Author and owner | Joel Parker Henderson, maintainer |
| Canonical location | `AI_STATEMENT.md` at the workspace root |
| License | The same five-way choice as the rest of the project — [`LICENSE.md`](LICENSE.md) |
| Review | At every release that changes the practice described here, and on any trigger in §13 |

**Abstract.** This document discloses how artificial-intelligence tools are
used to develop HL7® for Rust, an open-source workspace of Rust crates for
Health Level Seven messages. It states what the tools do and do not touch,
who is accountable, which controls bound the work and how each is enforced,
the licensing and data posture, the rules for contributors, the uses that
are prohibited, and the limitations that survive all of it. It is a
self-declaration by the maintainer, written for evaluators and regulated
adopters performing supplier due diligence, and it changes in the same
commit that changes the practice it describes.

The key words **shall**, **should**, and **may** are used as ISO/IEC
Directives Part 2 defines them: requirement, recommendation, permission.

## 1. Scope

This document covers the use of AI tools in developing everything in this
workspace: crate code, the release dictionaries and the generator that
writes them, tests, fuzz targets, benchmarks, the website, the
specifications under each crate's `spec/`, and this document itself. Since
2026-09-02 it also covers *releasing* it — see §5's `autonomous` row and
[`spec/release-process/index.md`](spec/release-process/index.md).

It does not cover an AI system in the product, because there is none:
**these crates ship no AI.** No model is trained, embedded, or called at
run time. Nothing here performs inference, and nothing here reaches a
network at all — a property stated and made checkable in
[`spec/phi/index.md`](spec/phi/index.md). AI is used to *build* the
software, in the same sense a compiler and a linter are used to build it.

## 2. Which frameworks apply, and which do not

Stated plainly, because borrowed authority is worse than none.

- **The EU AI Act imposes no obligation on this project.** The Act binds
  providers and deployers of AI *systems*. This workspace is not one: it
  ships no model and performs no inference. Content-marking duties bind an
  AI tool's provider, not the tool's user. This document is voluntary.
- **These crates are not a medical device.** They are parsing and encoding
  libraries with no clinical purpose and no clinical claim. A downstream
  integrator who gives *their* product a medical purpose may bring that
  product into scope; that classification is theirs to make, and this
  document exists partly so they can answer their own supplier questions.
- **No standard is claimed as conformity.** No certification exists, no
  audit has occurred, and the words "certified", "audited", and "validated"
  appear in this document only in this sentence, to say they do not apply.
  The same disclaimer governs
  [`spec/conformance/index.md`](spec/conformance/index.md), which is a
  self-assessment of HL7 coverage and says so in its own first lines.

## 3. Terms

This document reuses the W3C AI Content Disclosure vocabulary rather than
inventing one: **none** (entirely human-authored), **ai-assisted**
(human-authored; AI edited, refined, or filled in boilerplate),
**ai-generated** (AI-generated with human prompting and review),
**autonomous** (AI-generated without meaningful human oversight). An
**agentic tool** is one that plans and executes multi-step work — editing
files, running builds and tests — under a human's direction, as opposed to
inline completion.

## 4. Accountability

One named human — the maintainer, listed in
[`MAINTAINERS.md`](MAINTAINERS.md) — is the author of and accountable for
every change in this workspace, whatever tool produced the bytes. A tool
**shall not** be named as the author of, or a signer of, anything here,
because a tool cannot be responsible for accuracy, integrity, or
originality, and responsibility that cannot be borne cannot be assigned.
There is no AI-issued sign-off of any kind.

The commit trailers described in §10 record *participation*, not
authorship. The `Author:` field of every commit in this history is the
maintainer.

## 5. Where AI is used, and at what level

The tooling is agentic AI coding assistance — Claude Code, by Anthropic,
in sessions the maintainer directs and reviews. The repository carries
`AGENTS.md` at the workspace root and in every one of the fourteen crates,
each with a `CLAUDE.md` beside it pointing at it: those files are the
standing instructions given to the tools, they are committed, and they are
readable by anyone evaluating this claim.

Levels below use the §3 vocabulary. Deliberately, no percentage appears
anywhere in this document: no defensible method exists for measuring one.

| Activity | Level | Notes |
|---|---|---|
| Crate code | ai-generated | Written in directed sessions against the HL7 standards and each crate's own spec; reviewed and committed by the maintainer |
| Tests, fuzz targets, benchmarks | ai-generated | Held to the same authority as the code they exercise; §7 governs what happens when one fails |
| The `spec/index.md` documents | ai-generated | The normative layer. A rule in a spec is only there because a test backs it |
| Documentation, the website, and this statement | ai-generated | Held to the repository's own prose rules |
| Which HL7 behaviors to model, what a standard's silence means | none | Decided by the maintainer |
| Deciding *that* a specific, already-landed change warrants a crates.io release, and executing `cargo publish` for it | autonomous | The one deliberate exception to the row above — adopted 2026-09-02, at the maintainer's direction, and strictly bounded by [`spec/release-process/index.md`](spec/release-process/index.md): only inside a live interactive session on the maintainer's own machine, never a scheduled or unattended one; only using the crates.io credential already configured there, never a separate one; and only when every precondition that spec states holds (pushed and CI-green, a real landed fix, semver and inter-crate requirements verified, a `CHANGELOG.md` entry, a `tasks.md` record). §4's accountability is unchanged by this row |
| Accepting a contribution from someone else | none | Prohibited use; see §11 |

**autonomous** now appears in exactly the one row above, adopted
2026-09-02 — see §6.

## 6. Human oversight

The maintainer directs the work, reads the result, and commits every
change; nothing lands on its own authority. Until 2026-09-02, no commit or
release was automated at all — this document said so flatly, and that
carve-out is now the only exception: within the bounds
[`spec/release-process/index.md`](spec/release-process/index.md) states, an
agentic tool may decide a specific change warrants a release without the
maintainer naming that release as a specific instruction first, and
execute it. That is deliberately narrower than "automated" in the
scheduled-workflow sense — it still requires a human-started, live
session, the same environment every other agentic action in this
workspace already runs in — and it is the one row in §5 where the
decision is not the maintainer's alone. Every other decision with
consequences — what a coverage gap means, whether an incompleteness is
acceptable, what a released version *claims* rather than merely *whether
it ships* — remains the maintainer's. A decision that exists only inside a
tool session, outside what §5's autonomous row and
[`spec/release-process/index.md`](spec/release-process/index.md) bound, is
still not a decision this project made.

## 7. Quality controls, and what each one proves

AI-produced work is not a shortcut around engineering process. Every
change, whoever or whatever wrote it, passes the same gates:

```sh
cargo test                                # unit, integration, and doc tests
cargo clippy --all-targets -- -D warnings # lint-clean, pedantic harmonized
cargo fmt --check                         # formatting
cargo rustdoc --lib -- -W missing-docs    # every public item documented
cargo +1.96 check --workspace --all-targets   # the MSRV floor
```

- **Spec authority.** Each crate's `spec/index.md` is the single source of
  truth for its behavior, numbered so it can be cited. Every rule in one is
  backed by a test. A change to behavior goes in the spec first, and a code
  change that contradicts the spec is a bug in one of the two. This is the
  control that catches a plausible-but-wrong implementation regardless of
  who wrote it, because the tool cannot quietly redefine what correct means.
- **Round-trip assertions.** A message parsed and not modified must render
  back byte for byte, and ER7 → XML → ER7 is a test. The project's most
  serious defect to date — silent data corruption in the conversion pairs,
  fixed 2026-08-20 — was exactly the kind a confident-looking
  implementation produces, and the round trip is the assertion that now
  catches it.
- **Fuzz targets**, in the four crates whose parsing surface takes
  untrusted structured input — `hl7-2-xml-lite-helper`,
  `hl7-2-from-xml-into-er7`, `hl7-2-from-json-into-er7`, and — since
  2026-09-01 — `hl7-2` itself, for its JSON dictionary reader. Four of
  fourteen is not coverage; it is the surfaces where malformed input is the
  realistic threat, and the rest is an open gap rather than a decision.
- **Benchmarks with a published method**
  ([`spec/benchmark/index.md`](spec/benchmark/index.md)), so a performance
  claim is a measurement rather than an impression.
- **Tests and expectations shall not be weakened to make a build pass.**
  That is a standing hard rule, for humans and tools alike.

What these controls do **not** prove is §12. Since 2026-08-26 these gates
are enforced by machine: `.github/workflows/ci.yml` runs all five on every
push and pull request. Before that date they were run on a laptop by one
person, and the history carries that period.

## 8. Licensing and provenance of AI output

The project is multi-licensed — MIT, Apache-2.0, BSD-3-Clause,
GPL-2.0-only, or GPL-3.0-only, at the user's option. The position taken
here follows the Apache Software Foundation's and LLVM's published
reasoning rather than wishful shortcuts: an AI tool's output does not
launder anyone's copyright, the full provenance of generated text is
generally not knowable, and prompting alone is not treated as authorship.

In practice: contributions of substantially copied third-party material are
refused however they were produced; generated code is held to the same
originality expectations as human code, under the same review; and if
identifiable third-party material is found in the tree, it is removed or
licensed properly, exactly as it would be for a human-introduced copy. The
tools are used under terms that do not restrict the output's use under
these licenses.

The HL7 standards themselves are not this project's to license, and are not
reproduced here: the crates implement the standards, and
[`LICENSE.md`](LICENSE.md) states that boundary.

## 9. Data

**No patient data, no personally identifiable health information, and no
customer data exists anywhere in this project** — not in the repository,
not in test fixtures, not in benchmark inputs, not in telemetry (there is
none), and therefore not in any prompt. Every sample and fixture is
synthetic. This is a structural property a reader can check against the
tree, not a promise about tool behavior, and
[`spec/phi/index.md`](spec/phi/index.md) states it as a rule that binds
future changes too.

Vendor-side data handling is governed by the tool vendor's terms; this
document deliberately makes no claim on the vendor's behalf, because such
claims go stale silently.

## 10. Rules for contributors

Contributors **may** use AI tools. A contribution with **ai-generated**
content per §3 **should** say so in the pull-request description: which
tool, and what it did.

**This project records tool participation in commit trailers**, in the
form `Co-Authored-By: Claude <model> <noreply@anthropic.com>`, and the
history carries them. That is a deliberate choice and worth naming, because
it is not universal — some projects require such trailers and others forbid
them, and there is no ecosystem-wide agreement. The reasoning here is that
the trailer is a per-commit fact, cheap to record and impossible to
reconstruct later, while this document is the standing disclosure that
explains what the trailer means. §4 governs how to read it: the trailer
records participation, and the `Author:` field records accountability.

A contributor remains responsible for their submission in full, under the
same [`CONTRIBUTING.md`](CONTRIBUTING.md) bar as any other work: understood,
explained on request, tested, and honest.

## 11. Prohibited uses

In this project, AI **shall not**: commit or merge anything on its own
authority; decide whether to accept a contribution from someone else; sign
anything; decide what a standard means where the standard is silent, or
what a release claims; or weaken a test, an expectation, a spec rule, or a
gate to make something pass.

One more, specific to this domain: AI **shall not** be used to widen
dictionary coverage speculatively from the standard's tables. Coverage is
added when a real message motivates it, because a table transcribed with no
message behind it is a table nobody can check and no test can defend. That
rule predates this document and is in
[`CONTRIBUTING.md`](CONTRIBUTING.md) and each crate's spec.

## 12. Limitations and residual risks

This section exists because a disclosure without one is marketing.

- **The gates prove what they test, not correctness.** The test suite
  demonstrates the behaviors it covers. Coverage is real and ratchets
  upward, and it is still a boundary.
- **CI is young.** The §7 gates run in CI on every push and pull request
  as of 2026-08-26 (`.github/workflows/ci.yml`), and a dependency audit
  (`cargo deny`, per `deny.toml`) followed the same day in
  `.github/workflows/security.yml`. Everything before 2026-08-26 depended
  on one person remembering to run these gates by hand, and CI still gates
  no release: nothing publishes to crates.io on a green check by itself.
  Since 2026-09-02 the human typing `cargo publish` is no longer the only
  path — an agentic tool may, within
  [`spec/release-process/index.md`](spec/release-process/index.md)'s
  bounds — but every precondition that spec states still has to hold
  first, and CI passing is only one of them.
- **Review depth is one person's.**
  [`MAINTAINERS.md`](MAINTAINERS.md) says the bus factor is one. "The
  maintainer understands and can explain every committed change" is the
  honest claim; "every line was independently re-derived" would not be.
  That claim is about the *content* of a change, made in a directed
  session per §5's other rows, and is unaffected by an agentic tool
  deciding the *timing* of a release under
  [`spec/release-process/index.md`](spec/release-process/index.md) — the
  code being released was still reviewed and committed by the maintainer
  before the release decision was ever made.
- **Conformance is self-assessed and incomplete by design.**
  [`spec/conformance/index.md`](spec/conformance/index.md) publishes the
  bound — 24 segments, 42 types, 4 structures — rather than implying
  completeness. AI-assisted development makes it easy to produce more
  surface than one person can verify, and publishing the bound is the
  counterweight.
- **Retroactivity.** Commits predating this statement carry the trailers
  described in §10 but no other disclosure marker. This document describes
  the practice, not a per-commit audit trail, and no such trail is claimed.
- **Provenance uncertainty survives.** Whether any generated fragment
  echoes unlicensed training material is not fully knowable with current
  tools. §8 states the handling, not a guarantee.
- **The legal ground is unsettled.** Copyright in AI output is an open
  question in most jurisdictions. This document records positions, and
  positions may have to change.
- **This is a self-declaration.** No third party has audited it. The
  checkable artifacts — the specs, the tests, the trailers, the published
  benchmark method — are the counterweight: they can disagree with this
  document, and if they do, the document is wrong.

## 13. Review and change

This statement is reviewed at every release that changes the practice
described here, and revised off-cycle when any of these fires: the tooling
changes materially, a tool vendor's terms change in a way §8 or §9 relies
on, a binding rule emerges that touches this use, or a claim in this
document stops being true. The change lands as a commit like everything
else, and the version and the change log in Annex A update in the same
commit.

## 14. Reporting

A suspected provenance, licensing, or quality problem in this repository —
including a claim in this document that does not survive checking — is a
report this project wants. Open an issue and cite this file; for anything
security-sensitive, use the private channels in
[`SECURITY.md`](SECURITY.md), which is also candid about what is and is not
promised about response. The handling commitment is the same as for any defect:
answered, and never silently absorbed.

## 15. References

**Normative for this project** — the documents that bind the practice
described here: [`LICENSE.md`](LICENSE.md); each crate's `spec/index.md`;
[`spec/conformance/index.md`](spec/conformance/index.md),
[`spec/phi/index.md`](spec/phi/index.md),
[`spec/benchmark/index.md`](spec/benchmark/index.md),
[`spec/rust-msrv-n-minus-2/index.md`](spec/rust-msrv-n-minus-2/index.md);
the workspace and per-crate `AGENTS.md`;
[`CONTRIBUTING.md`](CONTRIBUTING.md); [`MAINTAINERS.md`](MAINTAINERS.md);
[`GOVERNANCE.md`](GOVERNANCE.md); [`SECURITY.md`](SECURITY.md).

**Informative** — the sources this document's structure and positions draw
on: the W3C AI Content Disclosure vocabulary; the ISO/IEC Directives
Part 2 verbal forms; the Apache Software Foundation's and LLVM's
generative-tooling positions; the Linux Foundation's generative-AI policy;
the practice of the FerroEHR project, whose AI statement is the structural
model for this one.

## Annex A. Change log

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-08-26 | First issue. |
| 1.1.0 | 2026-08-26 | CI stood up (`.github/workflows/ci.yml`); §7 and §12 updated: the quality gates are now machine-enforced on every push and pull request. |
| 1.2.0 | 2026-09-01 | §7's fuzz-targets row updated: a fourth crate, `hl7-2` itself (its JSON dictionary reader), gained a fuzz target, closing the gap `SECURITY.md` and this file both named. The header table's "Review" row corrected from "§12" to "§13" — the trigger list it points at has always lived in §13, not §12. |
| 2.0.0 | 2026-09-02 | Major, not minor: §5's table gained an `autonomous` row for the first time since 1.0.0 — the maintainer authorized an agentic tool to decide, on its own judgment, that a specific already-landed change warrants a crates.io release and to execute `cargo publish` for it, strictly bounded by the new [`spec/release-process/index.md`](spec/release-process/index.md). §5, §6, and §12 updated to state the exception precisely rather than let "no commit or release is automated" stand as a claim this version makes false. §4's accountability model is explicitly unchanged. |

## Annex B. Machine-readable summary

Levels per the W3C AI Content Disclosure vocabulary (§3); the prose above
is authoritative where the two could ever disagree.

```yaml
ai-statement:
  version: 2.0.0
  last-updated: 2026-09-02
  vocabulary: w3c-ai-content-disclosure
  disclosure-default: ai-generated
  tools:
    - name: Claude Code
      provider: Anthropic
  processes:
    design: ai-assisted
    implementation: ai-generated
    testing: ai-generated
    documentation: ai-generated
    review: none
    standards-adjudication: none
    release-scope: none
    release-execution: autonomous
  commit-trailers: true
  ships-ai-system: false
  autonomous-use: release-execution
  autonomous-use-scope: spec/release-process/index.md
```

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
