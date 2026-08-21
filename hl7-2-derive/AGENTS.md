# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this repository. `CLAUDE.md` is a pointer to this file — keep this one
canonical and don't fork the content between the two.

## What this is

The `#[derive(FromHl7)]` and `#[derive(ToHl7)]` macros for the
[`hl7-2`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2) crate, which re-exports them
behind its optional `derive` feature. Nobody depends on this crate directly.

It exists as a separate crate for one reason: `hl7-2` advertises exactly
one runtime dependency, and a proc-macro crate cannot be conditionally
compiled inside it. `syn` and `quote` are compiled only for callers who ask
for the macros. **Do not add dependencies here, and do not move anything
into `hl7-2` that would make its default build pull them in.**

The behavior these macros generate against is specified in the `hl7-2`
crate's `spec/index.md` §6 — that document is the source of truth for what
the attributes mean and how field types convert. This crate's job is only to
write the code a caller would otherwise write by hand.

## Layout

```
src/lib.rs        Both derives: attribute parsing (Mapping) and the two
                  code generators (from_hl7, to_hl7).
tests/derive.rs   Tests, which compile real structs against the real
                  hl7-2 crate — the only way to test a proc macro.
```

`hl7-2` is a dev-dependency by path, which makes this a dev-dependency
cycle (`hl7-2` depends on this crate for the `derive` feature). Cargo
allows that; don't try to "fix" it by vendoring types.

## Working conventions

- **Rust edition 2024.** Dependencies: `proc-macro2`, `quote`, `syn` — and
  nothing else.
- Generated code names everything absolutely (`::hl7_2::FromHl7Value`,
  `::core::result::Result`) so it compiles in a module that has imported
  none of it, or has shadowed the names.
- The generated code names `hl7-2` through the path `crate_path` returns —
  `::hl7_2` by default, or whatever the struct's `#[hl7(crate = ...)]` says.
  Never hard-code `::hl7_2` in a `quote!` again: a caller who renamed the
  dependency has no such path, and generated code is not something they can
  edit. The same rule holds in `hl7-3-derive`.
- A malformed attribute must produce a `syn::Error` pointing at the
  offending tokens, not a panic and not a confusing type error a hundred
  lines later. Test the message.
- Every public item needs a doc comment.
- Before finishing a change, run — in this crate and in `hl7-2`, from the
  workspace root, because the macros and the traits move together:
  ```sh
  cargo test -p hl7-2-derive -p hl7-2
  cargo clippy -p hl7-2-derive -p hl7-2 --all-targets -- -D warnings
  cargo fmt --check
  ```

## Non-goals (don't "fix" these without discussion)

- Attributes that encode HL7 knowledge (segment tables, data types,
  cardinality). The macro maps a field to a path; what a path *means* is the
  dictionary's job, in `hl7-2`. Keep this crate ignorant of HL7.
- `serde` compatibility or reusing `#[serde(...)]` attributes.
- Deriving for enums or tuple structs: neither has a single message shape or
  the field names the mapping is built on.
- Runtime behavior of any kind. This crate emits code; it does not parse
  messages, and it must not gain a dependency on anything that does.
