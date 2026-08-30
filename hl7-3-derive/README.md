# HL7® v3 derive

> HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
>
> This project uses the HL7® name in its package names, its organization
> name, and its domain, which is beyond fair use; we are requesting
> permission from HL7® for that.

Derive macro for [`hl7-3`](https://crates.io/crates/hl7-3): map a struct's
fields to XML element attributes and children once, in the type
definition, instead of writing the same accessor calls at every call site.

You do not depend on this crate directly. `hl7-3` re-exports the macro
behind its `derive` feature:

```toml
hl7-3 = { version = "0.2", features = ["derive"] }
```

Keeping the macro in a crate of its own is what lets the default build of
`hl7-3` keep exactly one dependency: `syn` and `quote` are compiled only
for callers who ask for the macro.

## Usage

```rust
use hl7_3::FromElement;
use hl7_3::rim::Act;

#[derive(FromElement, Default)]
struct Observation {
    #[element("classCode")]          class_code: String,
    #[element("moodCode")]           mood_code: String,
    #[element(child = "note")]       note: Option<String>,
    #[element(nested = "component")] component: Act,        // its own FromElement
    #[element(raw)]                  raw: hl7_3::xml::Element, // the escape hatch
}

let element = hl7_3::xml::parse(xml)?;
let observation = Observation::from_element(&element);
assert_eq!(observation.class_code, "OBS");

// The one attribute no struct field models — same object, no second parse.
assert_eq!(observation.raw.attribute("negationInd"), Some("true"));
```

One attribute per field:

| attribute | reads |
|---|---|
| `#[element("classCode")]` | the `classCode` attribute |
| `#[element(child = "note")]` | the `note` child's text |
| `#[element(nested = "component")]` | the `component` child, via the field type's own `FromElement` |
| `#[element(raw)]` | the whole element, as `hl7_3::xml::Element` |
| none | `Default::default()` |

Field types convert through `hl7_3::typed::FromElementValue`: `String`,
`bool`, and the integer and floating-point types, plus `Option<T>` of any
of those. There is no `Vec<T>` support yet — a repeating child needs
`element.children_named(...)` by hand for now.

**No `Result` anywhere.** Unlike `hl7-2-derive`'s `#[derive(FromHl7)]`, a
missing attribute or child is not an error — it reads as that field's
`Default`, the same "degrade, don't reject" choice `hl7-3`'s own `rim`
types make. See `hl7-3`'s `typed` module documentation for why.

**No `#[derive(ToElement)]`.** `hl7-3` has no XML-writing capability yet, so
a write-direction macro would have nothing real to generate.

## See also

- [`hl7-3`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-3) — the
  library this macro is for; its `typed` module documents struct mode
- [`hl7-2-xml-lite-helper`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-xml-lite-helper) —
  the XML reader `hl7-3` (and this macro's generated code) reads through

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
