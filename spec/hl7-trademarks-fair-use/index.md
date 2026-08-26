[hl7-rust](../../README.md) → spec → HL7® trademarks fair use

# HL7® trademarks fair use

HL7®'s own fair-use rules, quoted verbatim below, are what this project
follows. They are reproduced rather than paraphrased because the disclaimer
wording is prescribed and a paraphrase would not satisfy it.

## What this project does about it

- **The disclaimer**, verbatim, appears at the top of every `README.md` and
  in `LICENSE.md`, at the end of every other document that uses the marks,
  and in the website footer, which puts it on every page of
  <https://hl7-rust.github.io>.
- **The ® symbol** follows the first use of each word mark — HL7®, FHIR®,
  CDA® — on every page.
  On the website that is the brand in the site header, which is why the
  header reads "HL7® Rust".
- **The Fast Healthcare Interoperability Resources** are referred to as the
  "HL7® FHIR® standard" in headings and other places of prominence.
- **Beyond fair use:** this project also uses the HL7® name *in* its package
  names, its organization name, and its domain. That is branding, not a
  fair-use reference, and HL7®'s separate trademark guidance expects written
  consent for it — so the README notice states that permission is being
  requested. See [`help/outreach/index.md`](../../help/outreach/index.md).

## The rules, as HL7® publishes them

> Fair Use of HL7 Word Marks: Anyone may use HL7 word marks in fair use ways. Examples of acceptable fair uses of HL7 word mark are provided at http://www.hl7.org/legal/fairuse.cfm. When using HL7 word marks (e.g., "HL7", "FHIR", "CDA", etc.) for fair use:
>
> Always include the trademark registration mark® after the first use of word marks each page
>
> Include the following disclaimer on the webpages, material and other locations where such marks are used: "HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7."
>
> Please refer to the Fast Healthcare Interoperability Resources as the "HL7® FHIR® standard". When referencing the HL7® FHIR® standard in a website, document, presentation, or otherwise in a place of prominence, refer to it as the "HL7® FHIR® standard". In subsequent uses, please refer to it as the "HL7® FHIR® standard" or "HL7® FHIR®", using the ® symbol as often as is practical, at least once on each page of printed matter, generally in connection with the first or dominant usage.

## Assurance

Assurance: create automatic tests to verify this works.

**Done 2026-08-26, with a deliberate scope.** [`bin/check-trademarks`](../../bin/check-trademarks),
ported from the `er7-rust` sibling, verifies three rules per page — T1, the
® after each mark's first use; T2, the disclaimer verbatim; T3, the
"HL7® FHIR® standard" full form on pages that refer to that standard — and
runs in CI (`.github/workflows/ci.yml`). Code spans, fenced blocks, link
targets, and URLs are masked first, so `hl7-2` and `chat.fhir.org` are
correctly not treated as uses of a mark.

**Covered today:** every markdown page in the workspace, every crate
root's rustdoc (`src/lib.rs`), and the website's shared layout footer —
which is what puts the disclaimer on every rendered page of
<https://hl7-rust.github.io>.

**Not yet covered**, recorded here so widening the scope stays a visible
task rather than a silent gap: per-route website source under
`hl7-rust.github.io/src`, rustdoc in non-root `.rs` files, `Cargo.toml`
`description` strings, and the six binaries' `--help` output. A full-scope
run on 2026-08-26 found 142 problems; the 28 inside today's scope were
fixed, and the remaining 114 sit in these deferred surfaces. Widening is a
matter of re-enabling the corresponding sections of the script and fixing
what they report.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
