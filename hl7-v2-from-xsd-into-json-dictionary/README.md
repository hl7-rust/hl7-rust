# hl7-v2-from-xsd-into-json-dictionary

Read a directory of HL7 version 2 XML Schema Definition (XSD) files — the
v2.xml encoding, as HL7 published it or as a vendor customised it — and write
the JSON dictionary the [`hl7-v2`](https://github.com/hl7-rust/hl7-v2)
crates read.

HL7 publishes v2.xml as schemas. `hl7-v2` reads a dictionary, because one
dictionary format can serve every release and every local dialect from a
single build. This crate is the bridge: it turns the schemas a site already
has into the dictionary the crates already read.

## Why you would want one

The dictionary it writes carries **cardinality** as well as data types —
each field's `minOccurs` and `maxOccurs` become `required` and `repeats`.
That is what lets `hl7-v2-from-er7-into-xml` run in schema mode and emit a
document that validates against the very schemas the dictionary was built
from: required fields present even when empty, every declared component
written, and no field split into more elements than the schema allows.

## Install

```sh
cargo install hl7-v2-from-xsd-into-json-dictionary
```

## Use

A schema directory holds three base files that every message shares, plus one
file per message structure:

```text
schemas/paris/
  2_5_1_types.xsd      composite data types and their components
  2_5_1_fields.xsd     every SEG.n element and the data type it carries
  2_5_1_segments.xsd   each segment's field list, with cardinality
  ADT_A05.xsd          one abstract message structure each
  ADT_A39.xsd
```

The `2_5_1` prefix is discovered from a structure file's `<xsd:include>`, so
the directory can be named for the sending system rather than for the HL7
release.

```sh
hl7-v2-from-xsd-into-json-dictionary schemas/paris -o paris.json
```

Two things the schemas cannot tell you, so you pass them in:

```sh
hl7-v2-from-xsd-into-json-dictionary schemas/paris \
    --name paris \
    --alias ADT_A28=ADT_A05 \
    --alias ADT_A31=ADT_A05 \
    -o paris.json
```

`--alias` says which trigger events arrive carried by another message's
structure. A directory holds `ADT_A05.xsd` but never says that an `ADT^A28`
is one, so that mapping has to come from you. `--inherits 2.5` layers the
document over a bundled release instead of standing alone.

Then read it:

```rust
let text = std::fs::read_to_string("paris.json")?;
let dictionary = hl7_v2::Dictionary::from_json(&text, "paris")?;
assert_eq!(dictionary.field_type("PID", 3), Some("CX"));
assert!(dictionary.field_cardinality("PID", 3).repeats);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Or hand it straight to a converter:

```sh
hl7-v2-from-er7-into-xml --dictionary paris.json --schema-shape message.hl7
```

## As a library

```rust
use hl7_v2_from_xsd_into_json_dictionary::{Options, convert_directory};

let document = convert_directory("schemas/paris".as_ref(), &Options::default())?;
std::fs::write("paris.json", document.to_json())?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Dependencies

None. Reading XSD needs an XML reader and writing a dictionary needs a JSON
writer; both are small enough, and specific enough to the shapes involved,
to live here rather than pull a general-purpose parser into a stack that
healthcare integration code gets audited for.

## Specification

`spec/index.md` is the source of truth for what this crate does. If it and
the README disagree, the spec wins.

## License

Licensed under any of MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or
GPL-3.0-only, at your option.
