# HL7®

> HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
>
> This project uses the HL7® name in its package names, its organization
> name, and its domain, which is beyond fair use; we are requesting
> permission from HL7® for that.

Health Level Seven (HL7) for Rust, organized by standard: one module per
release family, so a "message", a "segment", or a "code" in one standard is
never confused with the same word in another.

```
hl7                      this crate: re-exports one module per standard
  |
  +-- hl7::v2 = hl7-2    HL7 v2, releases 2.1-2.9 — parse, navigate,
                          validate, modify, and render, in three modes:
                          generic, schema-based, and struct-based
```

Today that's just `hl7::v2`. Room is left for `hl7::v3` and `hl7::fhir` as
those standards get implemented, each its own crate underneath, re-exported
here the same way.

## Install

```sh
cargo add hl7
cargo add hl7 --features derive   # pulls in hl7-2's derive macros
```

Depend on [`hl7-2`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2) directly instead if
you specifically want v2 with no umbrella indirection — this crate is
nothing but a thin re-export (`pub use hl7_2 as v2;`) plus room for future
standards.

## Use

```rust
use hl7::v2;

let message = v2::parse("MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\r\
                          PID|1||241900||SMITH^JOHN")?;
assert_eq!(message.get("PID-5.1")?.as_deref(), Some("SMITH"));
```

See [`hl7-2`'s README](https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2/README.md) for the
full tour of what `hl7::v2` can do, and its
[`spec/index.md`](https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2/spec/index.md)
for the normative specification.

## See also

- [`hl7-2`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2) — the HL7 v2 implementation
  this crate re-exports
- [`hl7-2-derive`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-derive) — the derive
  macros behind the `derive` feature
- [`hl7-2-mllp`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-mllp) — MLLP: sending
  and receiving HL7 v2 messages over TCP
- [`hl7-2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json),
  [`-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml),
  [`hl7-2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7),
  [`hl7-2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-xml-into-er7)
  — format conversions

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
