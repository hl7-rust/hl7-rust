# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this crate. `CLAUDE.md` is a pointer to this file — keep this one canonical
and don't fork the content between the two.

## What this is

The `#[derive(FromElement)]` macro for the
[`hl7-3`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-3) crate,
which re-exports it behind its optional `derive` feature. Nobody depends
on this crate directly. Sibling of `hl7-2-derive`, adapted to `hl7-3`'s
element-and-attribute shape instead of `hl7-2`'s path strings.

It exists as a separate crate for the same reason `hl7-2-derive` does:
`hl7-3` advertises exactly one runtime dependency, and a proc-macro crate
cannot be conditionally compiled inside it. `syn` and `quote` are compiled
only for callers who ask for the macro. **Do not add dependencies here,
and do not move anything into `hl7-3` that would make its default build
pull them in.**

The behavior this macro generates against is specified in `hl7-3`'s
`src/typed.rs` module documentation — there is no `spec/index.md` here or
in `hl7-3` for struct mode specifically (matching `hl7-2-derive`, which
also has none); the module doc comment and this file are the source of
truth for what the attributes mean and how field types convert.

## Layout

```
src/lib.rs        The derive: attribute parsing (Mapping) and the one
                  code generator (from_element).
tests/derive.rs   Tests, which compile real structs against the real
                  hl7-3 crate — the only way to test a proc macro.
```

`hl7-3` is a dev-dependency by path, which makes this a dev-dependency
cycle (`hl7-3` depends on this crate for the `derive` feature). Cargo
allows that; don't try to "fix" it by vendoring types. Unlike
`hl7-2-derive`'s equivalent dev-dependency, this one needs no `features =
[...]` — `hl7-3`'s `FromElement`/`FromElementValue` traits are not gated
behind the `derive` feature, only the macro re-export is (see
`hl7-3/src/lib.rs`).

## Working conventions

- **Rust edition 2024.** Dependencies: `proc-macro2`, `quote`, `syn` — and
  nothing else.
- Generated code names everything absolutely (`::hl7_3::typed::FromElement`,
  `::core::default::Default`) so it compiles in a module that has imported
  none of it, or has shadowed the names.
- **No `Result` in generated code.** `hl7-3`'s struct mode is total —
  `FromElement::from_element` returns `Self` directly, never
  `Result<Self, E>`, matching the "degrade, don't reject" choice
  `hl7-3::rim` already makes. Don't add fallibility here without changing
  that decision in `hl7-3` first, and updating both crates' docs together.
- The generated code names `hl7-3` through the path `crate_path` returns —
  `::hl7_3` by default, or whatever the struct's `#[element(crate = ...)]`
  says. Never hard-code `::hl7_3` in a `quote!` again: a caller who renamed
  the dependency has no such path, and generated code is not something they
  can edit. The same rule holds in `hl7-2-derive`.
- A malformed attribute must produce a `syn::Error` pointing at the
  offending tokens, not a panic and not a confusing type error a hundred
  lines later. Test the message.
- Every public item needs a doc comment.
- Before finishing a change, run — in this crate and in `hl7-3`, from the
  workspace root, because the macro and the traits move together:
  ```sh
  cargo test -p hl7-3-derive -p hl7-3 --all-features
  cargo clippy -p hl7-3-derive -p hl7-3 --all-targets --all-features -- -D warnings
  cargo fmt --check
  ```

## Non-goals (don't "fix" these without discussion)

- **`#[derive(ToElement)]` / any write direction.** `hl7-3` has no
  XML-serialization capability yet; a write-direction macro would have
  nothing real to generate. Add it only after `hl7-3` itself can render
  XML back out.
- **`Vec<T>` field support** (a repeating child, the way `hl7-2-derive`
  supports `Vec<T>` for a repeating v2 field). `hl7-3`'s `FromElementValue`
  doesn't have the equivalent of `hl7-2`'s `Message::repetitions` to build
  it on yet — add both together, not this crate alone.
- Attributes that encode RIM vocabulary knowledge (`ActClass`, `ActMood`,
  and the rest). The macro maps a field to an attribute or child; what a
  code *means* is out of scope for `hl7-3` itself right now (see its
  `spec/index.md` §6), so it is doubly out of scope here.
- `serde` compatibility or reusing `#[serde(...)]` attributes.
- Deriving for enums or tuple structs: neither has a single element shape
  or the field names the mapping is built on.
- Runtime behavior of any kind. This crate emits code; it does not parse
  XML, and it must not gain a dependency on anything that does.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
