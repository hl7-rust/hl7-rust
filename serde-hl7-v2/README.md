# serde-hl7-v2

> HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
>
> This project uses the HL7® name in its package names, its organization
> name, and its domain, which is beyond fair use; we are requesting
> permission from HL7® for that.

Serde support for [`hl7-2`](../hl7-2), the HL7 v2 dictionary layer — so a
parsed message, its dictionary-named tree, or its validation findings can
flow through JSON, YAML, or any other Serde data format, and the message
can come back out unchanged.

```rust
use serde_hl7_v2::Message;

let text = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260815120000||ORU^R01|MSG9|P|2.5\r\
            PID|1||12345^^^ACME^MR||SMITH^JOHN^Q||19800101|M";

let message = Message::parse(text)?;

let json = serde_json::to_string(&message)?;
let back: Message = serde_json::from_str(&json)?;

assert_eq!(back.to_er7(), text);
assert_eq!(back.version(), message.version());
```

Most callers reach this crate as `serde_hl7::v2` through the
[`serde-hl7`](../serde-hl7) umbrella, the way `hl7-2` is reached as
`hl7::v2`. Depend on `serde-hl7-v2` directly when you want v2 alone.

```
er7                    the ER7 encoding — serde-er7 gives its tree Serde support
  |
hl7-2               HL7 v2 itself: the dictionary, three modes, validation
  |
  +-- serde-hl7-v2    this crate: Serde for hl7-2's message, tree, and findings
```

This README is a tour. [`spec/index.md`](spec/index.md) is the normative
specification of every rule — the single source of truth this crate
implements against.

## Why this crate exists

`hl7-2` has one runtime dependency, `er7`, and no Serde support of its
own, on purpose — in a domain where dependency trees get audited, adding
`serde` there would cost every user of `hl7-2` a dependency they may not
want. This crate is the bridge instead: two dependencies, `serde` and
`hl7-2`, and nothing else. Every `Serialize`/`Deserialize` impl is written
by hand against the low-level trait methods — the same pattern
[serde's own documentation](https://docs.rs/serde/latest/serde/) walks
through — because the types are foreign and the shapes below are chosen,
not derived.

## Install

```sh
cargo add serde-hl7-v2
```

## The shape each type serializes as

| `hl7-2` type | Wrapper | Serializes as | Deserializes |
|---|---|---|---|
| `Message` | `Message` | `{"version": "2.5", "er7": "MSH\|^~\\&\|…"}` | yes: the text is parsed again, the version pinned, the dictionary resolved |
| `Node` | `Node` | `{"name": "PID.5", "path": "PID[1]-5[1]", "kind": "Field", "text": "SMITH^JOHN", "null": false, "children": [...]}` | no: a tree is a view of a message; the message is what round-trips |
| `generic::Kind` | `NodeKind` | `"Group"`, `"Segment"`, `"Field"`, `"Component"`, `"Subcomponent"` | yes |
| `Diagnostic` | `Diagnostic` | `{"severity": "Error", "kind": "ValueFormat", "path": "OBX[1]-5[1]", "detail": "…"}` | yes |
| `Severity` | `Severity` | `"Error"` or `"Warning"` | yes |
| `validate::Kind` | `DiagnosticKind` | the variant name, e.g. `"SegmentMissing"` | yes |
| `Version` | `Version` | `"2.5.1"`, as MSH-12 spells it | yes |

**Why the message is text, not a tree.** The segment/field/component tree
already has a Serde crate — [`serde-er7`](https://crates.io/crates/serde-er7),
over the `er7` types `hl7-2` exposes as `Message::raw()`. Repeating it
here would be two crates specifying one wire shape for one set of bytes.
ER7 text is the form every HL7 v2 system already agrees on, it round-trips
exactly, and what only this crate can add — because only `hl7-2` has the
dictionary — is the tree with `PID.5` and `XPN.1` for names, which is
`Node`.

## What it does

- **The message, with its release**: `Message` carries the ER7 text and
  the HL7 release it was read as — which `Options::version` may have
  forced, or MSH-12's `2.5.2` may have resolved to `2.5.1` — so
  `message.version()` survives the round trip along with the bytes.
- **The dictionary-named tree**: `Node` serializes `hl7_2::Message::tree()`
  as one object per node, six keys always present, for logs, document
  stores, and queries that want `PID.5` rather than field five.
- **Validation findings**: `Diagnostic` round-trips, so an API can return
  what `validate()` found and the other side can read it back.
- **Schema mode through Serde**: `Message::seed(&options)` is a
  `DeserializeSeed` that reads a message back through *your* dictionary,
  the way `parse_with_options` does.
- **Format-agnostic**: nothing in this crate mentions JSON, YAML, or any
  other format by name. `serde_json` appears only as a dev-dependency.
- **Ergonomic wrappers**: every wrapper implements `Deref` to its
  `hl7-2` type (`DerefMut` too for `Message` and `Diagnostic`, the two
  that are mutable) and `From` both ways, so `message.get("PID-5.1")`,
  `message.tree()`, and the rest work directly on the wrapper.
- **Opt-in strict deserialization**: `Strict<Message>` and
  `Strict<Diagnostic>` reject an unrecognized key instead of ignoring it —
  for catching a typo in a hand-written JSON fixture.

## What it deliberately does not do

It adds exactly one thing to `hl7-2`: Serde support for its existing
public types. It does not pick a wire format, define a second tree shape
for ER7, serialize dictionaries (they are already JSON, in the shape
`Dictionary::from_json` reads), or give struct mode's `FromHl7` types
Serde — your struct is yours to derive for.

## Documentation

| Where | What |
| ----- | ---- |
| [`spec/`](spec/index.md) | the normative specification — source of truth for behaviour |
| [`examples/`](examples/README.md) | runnable programs, one concept each |
| [`AGENTS.md`](AGENTS.md) | conventions and required checks for anyone, human or agent, changing this code |

Rendered API docs are at <https://docs.rs/serde-hl7-v2/>, or locally with
`cargo doc -p serde-hl7-v2 --no-deps --open`.

## Development

```sh
cargo test -p serde-hl7-v2                                   # unit, integration, and doc tests
cargo clippy -p serde-hl7-v2 --all-targets -- -D warnings    # lint-clean
cargo fmt --check                                            # formatting
cargo rustdoc -p serde-hl7-v2 --lib -- -W missing-docs       # every public item documented
cargo run -p serde-hl7-v2 --example v2_round_trip_via_json      # try an example
```

Behavioural changes start in [`spec/`](spec/index.md), not in the code.

## License

Multi-licensed, so a downstream project can pick whichever fits: MIT,
Apache-2.0, BSD-3-Clause, GPL-2.0-only, or GPL-3.0-only. See
[LICENSE.md](LICENSE.md).

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
