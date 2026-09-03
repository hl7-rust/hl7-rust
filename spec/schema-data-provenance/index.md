[hl7-rust](../../README.md) → spec → Schema data provenance

# Where the bundled dictionary data came from

`hl7-2/schemas/v2.1.json` through `v2.9.json` are tables: which composite
data types HL7® v2 defines, what components each is built from, what data
type each field of a bundled segment holds, and the grammar of four message
structures. This traces where those tables came from, as far back as the
history actually goes, and states plainly where the trail ends.

This exists because `hl7-2/spec/index.md` §0 previously said only that
`schemas/v2.5.json` "was generated from" the conversion crates' own copies
— true, but one hop short of an actual origin. A reviewer checking supply
chain or licensing risk needs the whole chain, not the most recent link.

## Contents

- [The short answer](#the-short-answer)
- [The chain, oldest first](#the-chain-oldest-first)
- [Where the trail ends, and why that is stated rather than hidden](#where-the-trail-ends-and-why-that-is-stated-rather-than-hidden)
- [What this data is, legally](#what-this-data-is-legally)
- [Terms](#terms)
- [What would close the gap](#what-would-close-the-gap)

## The short answer

**No file from HL7® International is vendored anywhere in this
workspace — not an XSD, not a table, not a copy of the standard.** The
bundled tables were authored directly as Rust source and later as JSON,
in AI-directed sessions under the maintainer, encoding structural facts
about the v2.5 standard: which data type a field holds, which components a
composite type has. The earliest commit that introduces them cites no
source document, and none has since been found by walking the history
further. That gap is real and is not being papered over — see
[Where the trail ends](#where-the-trail-ends-and-why-that-is-stated-rather-than-hidden).

The XSD files under `hl7-2-from-xsd-into-json-dictionary/samples/example/`
are a different thing entirely and are not part of this question: they are
original, synthetic stand-ins written to exercise the converter in tests,
explicitly labeled as such in their own header comments, and contain no HL7®
content at all — see [Terms](#terms).

## The chain, oldest first

Traced by walking git history through the `git subtree` merges that built
this workspace, which preserved each former repository's full history as a
second parent of its merge commit — so the chain below is read from actual
commits, not reconstructed from memory.

1. **`6afe87f` — "Add HL7 v2.5 ER7 to v2.xml converter library and CLI".**
   The founding commit of what is now `hl7-2-from-er7-into-xml`, in a
   standalone repository before any workspace existed. Its `src/types.rs`
   and `src/structure.rs` introduce the composite-type and segment-field
   tables in the form they still substantially have. The commit message
   describes what the code does; it names no source for the table content.
2. **`cdcf9ba` — "Add HL7 v2.5 ER7 to JSON converter library and CLI".**
   The founding commit of what is now `hl7-2-from-er7-into-json`, in its own
   standalone repository. Its own message says plainly what it did: "JSON
   sibling of hl7-2-5-to-xml-using-rust" — the tables were copied from the
   XML converter above, not independently authored or sourced.
3. **The reverse crates**, `hl7-2-from-xml-into-er7` and
   `hl7-2-from-json-into-er7`, carry structural tables of their own for the
   same reason their own `spec/index.md` §1.1 gives: reversing a conversion
   this project already controls the output shape of does not need a v2.5
   data-type dictionary, only the position each element or key already
   encodes. They are not part of this chain.
4. **`e32e462` — "Add hl7-v2: the HL7 v2 dictionary layer, in three
   modes"**, the founding commit of what is now `hl7-2`, in its own
   standalone repository, dated 2026-08-17. This is where
   `hl7-2/schemas/v2.5.json` first appears, as `schemas/v2.5.json`. That
   history was brought into this monorepo by the `cc0b51e` subtree merge,
   and the later `3de94a0` (the workspace's `hl7-3` introduction commit)
   only renames the directory (`hl7-v2` to `hl7-2`), with zero content
   change to the file. §0 of `hl7-2`'s own spec states the
   relationship precisely: this file was generated from the conversion
   crates' own copies (link 1 and 2 above) and is table-for-table
   identical to them. The other ten bundled release files (§3.4 of that
   spec) are deltas layered over this one, authored the same way, in the
   same kind of session.

So: **`hl7-2-from-xml-into-er7`'s founding commit is the true root.**
Everything else bundled in this workspace is a copy, a JSON re-encoding, or
a stated delta of what that one commit introduced.

## Where the trail ends, and why that is stated rather than hidden

The founding commit's message says what the code does and who assisted —
"Co-Authored-By: Claude Fable 5" — and nothing about where the table
content came from. No citation, no reference to a standard document, no
mention of a prior crate or corpus.

That is a real gap, and this document does not fill it with a guess. Two
honest things can be said about it:

- **The content is not exotic.** The mapping "PID field 5 is XPN, XPN's
  fourth component is ST" is not private knowledge; it is republished,
  independently and without controversy, across essentially every open
  implementation of HL7® v2 — HAPI, Mirth, `hl7apy`, `python-hl7`, and the
  Rust crates this project's own [`COMPARISONS.md`](../../COMPARISONS.md)
  names. A structural fact about a public standard is not the kind of thing
  that has one traceable origin.
- **That is a fact about how widely the information is known, not a
  substitute for provenance.** It does not tell you whether the specific
  session that wrote `6afe87f` was reciting from general training, from a
  cached copy of a public reference table, or from something else. Nobody
  who worked on this project has a record of which. Saying otherwise would
  be inventing a chain of custody rather than reporting one.

## What this data is, legally

Not a legal opinion, and this project makes none — but the position taken
here follows the same reasoning `LICENSE.md` already states for the
standards generally, and is worth making explicit for this data
specifically:

**A fact about a standard's structure is not the copyrightable expression
of that standard.** That a message field is typed `XPN`, and that `XPN`'s
components are `FN, ST, ST, ST, ST, IS, ID, ID, DR, TS`, is a fact HL7®
International's specification states; the specification's prose, its exact
XSD files, and its typeset tables are HL7®'s copyrighted expression of that
fact. This project's tables assert the fact, in this project's own words
and its own JSON shape, and reproduce none of HL7®'s document text, layout,
or XSD markup.

That reasoning is the standard idea/expression basis most open
implementations of published standards rely on, and it is why HAPI,
`hl7apy`, and the rest can exist without a license from HL7® International
covering their internal tables. It is not a guarantee that a reviewer at
your organization will reach the same conclusion, and if your risk
tolerance requires HL7® membership or a licensed copy of the machine-readable
v2.xml schemas as the basis for any table your organization uses, that
tolerance is reasonable and this project's tables do not meet it — build
your own from your licensed copy, or use
`hl7-2-from-xsd-into-json-dictionary` to generate one, per
[Terms](#terms) below.

## Terms

- **No HL7® file is vendored.** No XSD, no PDF, no extracted table, no copy
  of the standard, anywhere in this workspace's history.
- **The bundled `schemas/*.json` files carry this project's own license**,
  the same five-way choice as the code: MIT, Apache-2.0, BSD-3-Clause,
  GPL-2.0-only, or GPL-3.0-only, at your option — see
  [`LICENSE.md`](../../LICENSE.md). They are project source, tracked and
  licensed the same as any other file here.
- **The sample XSDs under
  `hl7-2-from-xsd-into-json-dictionary/samples/example/`** are original test
  fixtures, written for this project, containing invented type and segment
  names sufficient to exercise the converter and nothing resembling HL7®'s
  actual published schema content. They carry the same project license and
  are unrelated to the provenance question this document answers.
- **A dictionary you generate yourself**, from your own licensed copy of
  HL7®'s v2.xml XSDs via `hl7-2-from-xsd-into-json-dictionary`, is your
  data under whatever terms your license with HL7® International gives you.
  This project supplies the generator; it makes no claim on what you feed
  it or what comes out.

## What would close the gap

Named so it is a task rather than a permanent shrug:

- **A second, independent source-checked pass**: build a dictionary from a
  licensed copy of the actual v2.5 v2.xml XSDs using
  `hl7-2-from-xsd-into-json-dictionary`, and diff it against
  `schemas/v2.5.json`. Agreement would be strong evidence the bundled table
  states the standard correctly; it would not, by itself, establish where
  the original commit's authors got it from.
- **Asking the maintainer** whether any reference material beyond an AI
  session's own training was used for the founding commit. This document
  was written from the git history alone; the maintainer has not yet been
  asked, and may not know or recall — the sessions that wrote `6afe87f` and
  `cdcf9ba` left no note of their sources, which is the gap this document
  reports rather than resolves.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
