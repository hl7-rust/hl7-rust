# HL7 v2 derive

Derive macros for [`hl7-v2`](https://crates.io/crates/hl7-v2): map a
struct's fields to HL7 v2 message paths once, in the type definition,
instead of writing the same accessor calls at every call site.

You do not depend on this crate directly. `hl7-v2` re-exports both macros
behind its `derive` feature:

```toml
hl7-v2 = { version = "0.2", features = ["derive"] }
```

Keeping the macros in a crate of their own is what lets the default build of
`hl7-v2` keep exactly one dependency: `syn` and `quote` are compiled only
for callers who ask for the macros.

## Usage

```rust
use hl7_v2::{FromHl7, ToHl7, Raw};

#[derive(FromHl7, ToHl7)]
struct Admission {
    #[hl7("PID-1")]      sequence: u32,
    #[hl7("PID-3")]      identifiers: Vec<String>,
    #[hl7("PID-5.1.1")]  family_name: String,
    #[hl7("PID-8")]      sex: Option<String>,
    #[hl7(nested)]       visit: Visit,   // its own FromHl7 / ToHl7
    #[hl7(raw)]          raw: Raw,       // the whole message, kept alongside
}

let admission: Admission = hl7_v2::parse(text)?.decode()?;
assert_eq!(admission.family_name, "EVERYWOMAN");

// The one vendor field no struct models — same object, no second parse.
assert_eq!(admission.raw.get("ZPD-1")?.as_deref(), Some("local"));
```

One attribute per field:

| attribute | on read | on write |
|---|---|---|
| `#[hl7("PID-5.1")]` | read the path | write the path |
| `#[hl7(nested)]` | the field's own `FromHl7` | the field's own `ToHl7` |
| `#[hl7(raw)]` | the whole message, as a `Raw` | skipped |
| none | `Default::default()` | skipped |

Field types convert through `hl7_v2::FromHl7Value` / `ToHl7Value`:
`String`, `bool`, the integer and floating-point types, and `Option<T>` and
`Vec<T>` of those — `Option` for a value that may be absent, `Vec` for one
that repeats. A plain type is required, and a path that names nothing is
`Error::MissingField`. Implement `hl7_v2::FromHl7Text` for a domain type of
your own and `Option` and `Vec` of it follow.

Writing needs the segments to exist already; build the message with
`hl7_v2::Builder` (whose `encode` method takes a `ToHl7`) or add them with
`Message::append_segment`.

## See also

- [`hl7-v2`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2) — the library these macros
  are for; [its `spec/index.md` §6](https://github.com/hl7-rust/hl7-rust/blob/main/hl7-v2/spec/index.md)
  is the normative description of struct mode
- [`er7`](https://github.com/hl7-rust/er7) — the ER7 encoding layer

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
