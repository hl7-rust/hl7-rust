# AI statement (pointer)

The AI statement has **one source**: [`AI_STATEMENT.md`](../../AI_STATEMENT.md)
at the workspace root, which is where its own header names its canonical
location.

A divergent full draft used to sit in this file — version 1.0.0, thirteen
kilobytes, the same fifteen-section-plus-two-annex structure the root
document carries at 1.1.0. Resolution, recorded here the way the
`fhir-rust` sibling recorded its identical situation: the root document is
newer, longer, and self-describes as canonical; the draft had no section
the root lacks; the draft also carried nine relative links that resolved
only from the repository root, which is what exposed it when
`bin/check-docs` first ran. Nothing from the draft needed rescuing, so
this pointer is all that remains.
