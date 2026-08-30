# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this repository.

## What this is

A Cargo workspace holding every crate in the HL7®-for-Rust project — see
`README.md` for the crate table and how they depend on each other. Most
crate directories are former standalone repositories, merged in with
`git subtree` so their commit history survived the move (`hl7-3`,
`hl7-3-derive`, and `hl7-3-soap` are the exceptions so far — born directly
in this workspace, no prior repo). Each crate has its own `README.md`, `AGENTS.md`,
`CLAUDE.md`, and `LICENSE.md`, and — for behavior that's normative rather
than incidental — a `spec/index.md` (the two `*-derive` crates, and `hl7`
itself, don't have one; see each one's own `AGENTS.md` for why), which
remains the single source of truth for that crate's behavior. **Read the
crate's own `AGENTS.md` before working in it**; this file only covers
workspace-wide concerns.

## Layout

```
Cargo.toml          The workspace: [workspace] members, nothing else.
<crate>/Cargo.toml   Each member's own package manifest — unchanged from
                     when it was a separate repo, except `repository` now
                     points here and there is no per-crate Cargo.lock.
```

There is one `Cargo.lock` at the workspace root, not one per crate. Don't
add one back inside a member directory.

Member crates still depend on each other by relative path
(`hl7-2-mllp/Cargo.toml`'s `hl7-2 = { path = "../hl7-2" }`, for
example) exactly as they did as sibling repositories — the flat,
one-directory-per-crate layout was kept specifically so those paths did
not need to change.

## Working with HL7 messages

[`hl7-skill/SKILL.md`](hl7-skill/SKILL.md) is an Agent Skill covering the
*use* of this workspace's crates — which one to reach for, CLI recipes for
parsing/converting/validating HL7 v2 messages, and the gotchas worth
knowing before relying on the output. This file and the rest of `AGENTS.md`
family cover working *on* the workspace instead; read whichever one
matches the task.

## Working conventions

- `cargo build` / `cargo test` from the workspace root builds and tests
  every member in one pass. Use `-p <crate>` to scope to one.
- Each crate keeps its own edition, feature set, and dependency list. The
  one deliberate exception is `rust-version`, below — everything else
  stays per-crate; there is still no shared `[workspace.dependencies]`
  table, and don't add one without discussion.
- **MSRV is current stable Rust minus two releases**, pinned once in the
  root `Cargo.toml`'s `[workspace.package]` and inherited by every member
  as `rust-version.workspace = true` — see
  [`spec/rust-msrv-n-minus-2/index.md`](spec/rust-msrv-n-minus-2/index.md),
  which also has the mechanical reason: fourteen crates that must always
  agree on one value are the case `[workspace.package]` inheritance exists
  for. Check a change against it with
  `cargo +1.96 check --workspace --all-targets`; raising the floor is a
  breaking change and belongs in a release allowed to break.
- **Workspace-wide claims live in `spec/`**, not in a crate: `conformance/`
  (what "supports 2.1-2.9" means, with the segment, type, and structure
  lists), `phi/` (what the crates do with patient data and where a value can
  escape into a log), `benchmark/` (how figures are measured and what the
  current ones are), `rust-msrv-n-minus-2/` (the MSRV policy). A change that
  widens dictionary coverage, adds a way for message content to reach an
  error string, or moves a published benchmark figure updates the
  corresponding one in the same change.
- Every crate, and the workspace root, has its own `LICENSE.md` — the same
  multi-license boilerplate byte-for-byte everywhere, matching each
  `Cargo.toml`'s `license` field. Keep new crates consistent with that;
  don't invent different license text for one crate.
- When a change spans crates (a shared type, a version bump one crate's
  `Cargo.toml` pins another to), update every affected crate's own
  `AGENTS.md`/`spec/index.md` in the same change, the same as when they
  were separate repos.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
