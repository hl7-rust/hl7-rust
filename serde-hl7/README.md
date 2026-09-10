# serde-hl7

> HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
>
> This project uses the HL7® name in its package names, its organization
> name, and its domain, which is beyond fair use; we are requesting
> permission from HL7® for that.

Serde support for HL7 messages in Rust, one module per standard — so a
parsed HL7 v2 or HL7 v3 value can flow through JSON, YAML, or any other
Serde data format.

```rust
use serde_hl7::v2;

let text = "MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\r\
            PID|1||241900||SMITH^JOHN";
let message = v2::Message::parse(text)?;

let json = serde_json::to_string(&message)?;
let back: v2::Message = serde_json::from_str(&json)?;
assert_eq!(back.to_er7(), text);
```

The [`hl7`](../hl7) crate organizes HL7 by standard — `hl7::v2` is
`hl7-2`, `hl7::v3` is `hl7-3` — because a "message" or a "code" in one
standard is a different thing in another. This crate is the same shape one
layer up:

```
serde-hl7                         this crate: re-exports one module per standard
  |
  +-- serde_hl7::v2 = serde-hl7-v2    Serde for hl7-2: the message, its
  |                                    dictionary-named tree, validation
  |                                    findings, and the release
  +-- serde_hl7::v3 = serde-hl7-v3    Serde for hl7-3: the envelope, the RIM
                                       backbone classes, the data types, and
                                       the XML element tree beneath them
```

Each module is behind a Cargo feature of the same name, both on by default:

```toml
[dependencies]
serde-hl7 = "0.1"                                                          # v2 and v3
serde-hl7 = { version = "0.1", default-features = false, features = ["v2"] }  # v2 only
```

Or depend on [`serde-hl7-v2`](../serde-hl7-v2) or
[`serde-hl7-v3`](../serde-hl7-v3) directly and skip the umbrella.

## Install

```sh
cargo add serde-hl7
```

## What is in each module

Each is its own crate, with its own normative spec, tests, and examples.
This README only points; the detail is there.

| Module | Crate | Wraps | Spec |
| ------ | ----- | ----- | ---- |
| `serde_hl7::v2` | [`serde-hl7-v2`](../serde-hl7-v2) | `Message` (as its ER7 text plus release), `Node`, `Diagnostic`, `Version`, the kind and severity enums, `Strict<T>` | [spec](../serde-hl7-v2/spec/index.md) |
| `serde_hl7::v3` | [`serde-hl7-v3`](../serde-hl7-v3) | `Message`, `ControlAct`, `Element`, `Ii`, `Cd`, `Ivl`, `Pq`, `Ed`, `NullFlavor`, the six RIM classes, `Strict<T>` | [spec](../serde-hl7-v3/spec/index.md) |

Both crates share the same rules: exactly two runtime dependencies each
(`serde` and the `hl7-*` crate wrapped), no format crate anywhere but in
tests, every impl hand-written against Serde's low-level traits, every
wrapper `Deref`-ing to the type it wraps, and an opt-in `Strict<T>` that
rejects unknown keys.

## Documentation

| Where | What |
| ----- | ---- |
| [`AGENTS.md`](AGENTS.md) | conventions for anyone, human or agent, changing this crate |
| [`serde-hl7-v2/spec/`](../serde-hl7-v2/spec/index.md) | the v2 module's normative specification |
| [`serde-hl7-v3/spec/`](../serde-hl7-v3/spec/index.md) | the v3 module's normative specification |

Rendered API docs are at <https://docs.rs/serde-hl7/>.

## License

Multi-licensed, so a downstream project can pick whichever fits: MIT,
Apache-2.0, BSD-3-Clause, GPL-2.0-only, or GPL-3.0-only. See
[LICENSE.md](LICENSE.md).

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
