# serde-hl7-v3

> HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
>
> This project uses the HL7® name in its package names, its organization
> name, and its domain, which is beyond fair use; we are requesting
> permission from HL7® for that.

Serde support for [`hl7-3`](../hl7-3), the HL7 v3 foundation crate — so a
decoded interaction, a RIM class, a data type, or the raw XML element tree
beneath them can flow through JSON, YAML, or any other Serde data format,
and come back equal.

```rust
use serde_hl7_v3::Message;

let message = Message::parse(r#"
  <QUQI_IN000001UV01 xmlns="urn:hl7-org:v3">
    <id root="2.16.840.1.113883.19.5" extension="MSG00001"/>
    <controlActProcess classCode="CACT" moodCode="EVN">
      <code code="QUQI_TE000001UV01"/>
      <subject><observation classCode="OBS" moodCode="EVN"/></subject>
    </controlActProcess>
  </QUQI_IN000001UV01>"#)?;

let json = serde_json::to_string(&message)?;
let back: Message = serde_json::from_str(&json)?;

assert_eq!(back, message);
```

Most callers reach this crate as `serde_hl7::v3` through the
[`serde-hl7`](../serde-hl7) umbrella, the way `hl7-3` is reached as
`hl7::v3`. Depend on `serde-hl7-v3` directly when you want v3 alone.

```
hl7-2-xml-lite-helper    the XML reader
  |
hl7-3                 HL7 v3: RIM backbone classes, data types, the envelope
  |
  +-- serde-hl7-v3      this crate: Serde for every one of those values
```

This README is a tour. [`spec/index.md`](spec/index.md) is the normative
specification of every rule — the single source of truth this crate
implements against.

## Why this crate exists

HL7 v3's own serialization is XML, and `hl7-3` deliberately reads it
without writing it back — so once a message is decoded, the value has no
way out except code you write. `hl7-3` also has one runtime dependency and
no Serde support, on purpose. This crate is the bridge: two dependencies,
`serde` and `hl7-3`, and every `Serialize`/`Deserialize` impl written
against Serde's low-level traits — by one `macro_rules!` for the fourteen
object types, so their shapes and their strict-mode behaviour cannot
drift, and by hand for the one string type.

## Install

```sh
cargo add serde-hl7-v3
```

## The shape each type serializes as

Every type is an object whose keys are the `hl7-3` field names in
lowerCamelCase — which, wherever a field corresponds to an HL7 v3 XML
attribute or element, is that name: `classCode`, `codeSystem`,
`interactionId`. An `Option` is its value or `null`; a `Vec` is an array.

| `hl7-3` type | Wrapper | Keys |
|---|---|---|
| `Message` | `Message` | `id`, `creationTime`, `interactionId`, `sender`, `receiver`, `controlAct` |
| `ControlAct` | `ControlAct` | `classCode`, `moodCode`, `code`, `domain` |
| `xml::Element` | `Element` | `name`, `attributes`, `text`, `children` |
| `Ii` / `Cd` / `Ivl` / `Pq` / `Ed` | same names | `root`, `extension` / `code`, `codeSystem`, `displayName` / `value`, `low`, `high` / `value`, `unit` / `mediaType`, `representation`, `text` |
| `NullFlavor` | `NullFlavor` | a bare string: the code, `"NI"`, `"UNK"`, … any code is accepted |
| `rim::Act` | `Act` | `classCode`, `moodCode`, `id`, `code`, `statusCode`, `effectiveTime`, `text` |
| `rim::Entity` | `Entity` | `classCode`, `determinerCode`, `id`, `code`, `name` |
| `rim::Role` | `Role` | `classCode`, `id`, `code`, `statusCode`, `effectiveTime` |
| `rim::Participation` | `Participation` | `typeCode`, `time`, `functionCode` |
| `rim::ActRelationship` | `ActRelationship` | `typeCode`, `inversionInd` |
| `rim::RoleLink` | `RoleLink` | `typeCode` |

On the way back in, only the keys that name a thing are required —
`classCode`, `moodCode`, `typeCode`, `root`, `code`, an element's `name`
— and everything else defaults, matching `hl7-3`'s own rule that a missing
wrapper reads as absent rather than failing.

## What it does

- **Every public type**: the envelope, the six RIM classes, the six data
  types, and the `Element` tree — a Serde-enabled wrapper for each.
- **Round trip to an equal value**: every field is on the wire, so a value
  through any format and back compares equal. (Not an XML round trip:
  `hl7-3` has no XML writer, and its reader normalizes on the way in.)
- **HL7 v3's own names as keys**, so the JSON cross-references against the
  XML and the standard without a translation table.
- **Format-agnostic**: nothing in this crate mentions JSON, YAML, or any
  other format by name. `serde_json` appears only as a dev-dependency.
- **Ergonomic wrappers**: every wrapper implements `Deref`/`DerefMut` to
  its `hl7-3` type, `From` both ways, and `Default`.
- **Opt-in strict deserialization, nested to any depth**: `Strict<T>` for
  every object type rejects an unrecognized key wherever it appears —
  inside an identifier, a code, or the payload's element tree — where the
  plain type reads it as absent.

## What it deliberately does not do

It adds exactly one thing to `hl7-3`: Serde support for its existing
public types. It does not write XML, pick a wire format, validate codes
against a vocabulary domain, or give struct mode's `FromElement` types
Serde — your struct is yours to derive for.

## Documentation

| Where | What |
| ----- | ---- |
| [`spec/`](spec/index.md) | the normative specification — source of truth for behaviour |
| [`examples/`](examples/README.md) | runnable programs, one concept each |
| [`AGENTS.md`](AGENTS.md) | conventions and required checks for anyone, human or agent, changing this code |

Rendered API docs are at <https://docs.rs/serde-hl7-v3/>, or locally with
`cargo doc -p serde-hl7-v3 --no-deps --open`.

## Development

```sh
cargo test -p serde-hl7-v3                                   # unit, integration, and doc tests
cargo clippy -p serde-hl7-v3 --all-targets -- -D warnings    # lint-clean
cargo fmt --check                                            # formatting
cargo rustdoc -p serde-hl7-v3 --lib -- -W missing-docs       # every public item documented
cargo run -p serde-hl7-v3 --example round_trip_via_json      # try an example
```

Behavioural changes start in [`spec/`](spec/index.md), not in the code.

## License

Multi-licensed, so a downstream project can pick whichever fits: MIT,
Apache-2.0, BSD-3-Clause, GPL-2.0-only, or GPL-3.0-only. See
[LICENSE.md](LICENSE.md).

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
