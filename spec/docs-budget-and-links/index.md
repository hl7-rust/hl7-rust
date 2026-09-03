[hl7-rust](../../README.md) → spec → Documentation budget and link integrity

# Documentation budget and link integrity

This policy binds every markdown document git tracks in this repository,
adopted 2026-08-26 from the `snomed-rust` sibling's convention so the
family converges on one posture. Its first run here earned its keep
immediately: it exposed nine broken relative links in a divergent
`AI_STATEMENT.md` draft that had sat unnoticed in
`spec/special-files-for-public-repos/`, now resolved to a pointer at the
root document.

## Rules

1. **Every tracked markdown document is at most 40 KB** (40,960 bytes).
   The budget exists for readers and for agents: a document that must be
   read in one sitting, or loaded into one context, has to have a ceiling.
   The largest document here today is `tasks.md` at 40,917 bytes — 43
   bytes under the 40,960-byte limit — so the budget is already binding
   close to the edge, not with room to spare. When a document
   outgrows it, split it by topic with any rule numbers kept in one file,
   or archive the older entries verbatim — never meet the budget by
   deleting the record.
2. **Every relative link in a tracked markdown document resolves** to a
   file or directory that exists in the repository. External URLs
   (`http:`, `https:`, `mailto:`) are out of scope — availability of other
   people's servers is not this repository's claim to make. `#fragment`
   anchors are stripped before the path is checked, so a wrong fragment on
   a correct path is **not** caught; that is a stated limitation, not an
   oversight.
3. **`bin/check-docs` enforces rules 1 and 2 and runs in CI** (the `docs`
   job in `.github/workflows/ci.yml`) on every push and pull request, per
   rule 4 of [`spec/professionalization/index.md`](../professionalization/index.md):
   a laptop-only check is a claim, not a guarantee. The checker scans the
   files `git ls-files` reports, so untracked and generated trees are out
   of scope by construction; symlinked documents are skipped, since their
   target is scanned once already.

## What the checker deliberately does not do

- It does not fetch external URLs (rule 2's scope).
- It does not verify `#fragment` anchors (rule 2's stated limitation).
- It does not lint prose, headings, or style — `bin/check-trademarks`
  covers the one prose rule that is enforced, and
  [`spec/serial-comma/`](../serial-comma/index.md) is a convention
  reviewers apply, not a machine gate.
- Links inside code fences and inline code spans are masked before
  scanning, the same way `bin/check-trademarks` masks them: an example of
  a broken link is not a broken link.
