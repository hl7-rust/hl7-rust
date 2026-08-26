# Trademarks

The consolidated trademark position for this project. The policy behind
it, the rules it imposes on every file here, and the check that enforces
them (`bin/check-trademarks`, running in CI) are in
[`spec/hl7-trademarks-fair-use/index.md`](spec/hl7-trademarks-fair-use/index.md).

## Notice

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.

That wording is prescribed by HL7 International and is carried verbatim at
the top of every README, in `LICENSE.md`, at the end of every other
document that uses the marks, and in the footer of every page of
<https://hl7-rust.github.io>.

## What this project uses, and how

| Mark | Owner | How it is used here |
| ---- | ----- | ------------------- |
| HL7® | Health Level Seven International | Descriptively, to say what these crates read and write: HL7® v2 and v3 messages. Also in names — see below. |
| FHIR® | Health Level Seven International | Descriptively, and only to say that this project does **not** implement the HL7® FHIR® standard |

No other HL7® word mark appears in this project.

The descriptive uses are fair use of a word mark under the terms HL7
International publishes at <https://www.hl7.org/legal/fairuse.cfm>: the
marks name the standards the software works with. The three fair-use
rules — ® after each mark's first use per page, the verbatim disclaimer,
and the "HL7® FHIR® standard" full form — are enforced by
`bin/check-trademarks` on every push.

## What goes beyond fair use, stated plainly

Unlike the sibling `er7-rust` project, whose names contain no HL7® word
mark, this project uses the HL7® name **in names it controls**:

- the crate names (`hl7`, `hl7-2`, `hl7-3`, and the other `hl7-*` crates),
- the GitHub organisation `hl7-rust`,
- the domain `hl7-rust.github.io`.

That is branding, not a fair-use reference, and HL7 International's
trademark guidance expects written consent for it. The README notice
therefore says permission is being requested. **Honest status, as of
2026-08-26:** no sent date and no outcome for that request is recorded
anywhere in this repository — recording it (and sending it, if it has not
been sent) is an open item in [`tasks.md`](tasks.md), and all promotion is
gated on it per [`help/outreach/index.md`](help/outreach/index.md). If HL7
International declines, the organisation, crate, and domain naming question
reopens.

## What this project does not do

- **No logo, badge, or brand element** of HL7 International appears here.
- **No claim of endorsement, affiliation, certification, accreditation, or
  conformity assessment** is made anywhere in this project. HL7
  International has not assessed this software;
  [`spec/conformance/index.md`](spec/conformance/index.md) opens by saying
  so, and [`NEWS.md`](NEWS.md) says the same thing in the form a reporter
  needs.
- **No implementation of the HL7® FHIR® standard** is claimed or provided;
  the FHIR® mark appears here only to state that boundary.

## Other marks

"Rust" and the Rust logo are trademarks of the Rust Foundation. This
project is written in Rust and is not affiliated with or endorsed by the
Rust Foundation.

Product names of the interface engines and libraries discussed in
[`COMPARISONS.md`](COMPARISONS.md) — Mirth Connect, Rhapsody, InterSystems,
Cloverleaf, HAPI, and others — are the marks of their respective owners,
and are used there nominatively, to identify the products being compared.

## If we have this wrong

Trademark owners are welcome to write to
<joel@joelparkerhenderson.com>. A correction here is treated the same way
as any other defect report: acted on, not argued with.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
