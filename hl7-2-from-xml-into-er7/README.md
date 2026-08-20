# HL7 v2 from XML into ER7

Convert Health Level Seven (HL7) version 2.5 messages from the official HL7
**v2.xml** XML representation (`urn:hl7-org:v2xml`) back to the traditional
pipe-delimited Encoding Rules version 7 (ER7) encoding, as a Rust library
and command-line tool.

This is one of four sibling crates in the `hl7-rust` family — see
[Related crates](#related-crates) below. It is the inverse of the sibling
[`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml)
crate. No HL7 v2.5 data-type dictionary is needed to reverse the
conversion, because the forward crate's element names always carry the
field/component/subcomponent *position* as the number after the name's
last `.`, whether or not the name in front of it is a recognized data
type — so this crate depends on [`er7`](https://crates.io/crates/er7) for
the encoding layer and, since 0.5.0, on `hl7-2-xml-lite-helper` for
reading the XML itself (it used to carry its own minimal XML reader; see
`src/lib.rs`'s crate docs for the change). See [`spec/index.md`](spec/index.md)
for exactly how reconstruction works and its limits — it is the normative
specification this crate implements.

A v2.xml fragment such as:

```xml
<PID>
  <PID.1>1</PID.1>
  <PID.3>
    <CX.1>241900</CX.1>
  </PID.3>
  <PID.5>
    <XPN.1>
      <FN.1>TEST</FN.1>
    </XPN.1>
    <XPN.2>FOUAZ</XPN.2>
  </PID.5>
</PID>
```

converts back to:

```
PID|1||241900||TEST^FOUAZ
```

## Usage

### Command line

```sh
# From a file to stdout
cargo run -- samples/orm_o01.xml

# From stdin, to a file
cat samples/oru_r01.xml | cargo run -- -o out.hl7

# Choose the segment terminator
cargo run -- --terminator crlf samples/orm_o01.xml
```

Message-structure group elements (`<ORM_O01.PATIENT>`,
`<ORU_R01.ORDER_OBSERVATION>`, …) are flattened automatically; grouped and
`--flat` input from the forward crate both reconstruct the same message.

Namespace prefixes are ignored: a document that binds `urn:hl7-org:v2xml`
to a prefix (`<ns0:MSH><ns0:MSH.1>|</ns0:MSH.1>…`) converts to exactly the
same ER7 as one that makes it the default namespace, whatever prefix the
sending system happened to pick (`spec/index.md` §2.1).

### Library

```rust
let xml = r#"<ORM_O01 xmlns="urn:hl7-org:v2xml">
  <MSH>
    <MSH.1>|</MSH.1>
    <MSH.2>^~\&amp;</MSH.2>
    <MSH.9><MSG.1>ORM</MSG.1><MSG.2>O01</MSG.2></MSH.9>
  </MSH>
  <ORM_O01.PATIENT>
    <PID><PID.5><XPN.1><FN.1>TEST</FN.1></XPN.1><XPN.2>FOUAZ</XPN.2></PID.5></PID>
  </ORM_O01.PATIENT>
</ORM_O01>"#;
let er7 = hl7_2_from_xml_into_er7::convert(xml)?;
```

See also `convert_with_options` for the segment terminator
(`er7::RenderOptions`, re-exported as `hl7_2_from_xml_into_er7::er7::RenderOptions`),
and `parse` when the caller wants the full `er7::Message` — to query or
edit it — rather than just its ER7 text:

```rust
use hl7_2_from_xml_into_er7::parse;

let message = parse(xml)?;
assert_eq!(message.query("PID-5.1")?.as_deref(), Some("TEST"));
```

`convert`/`convert_with_options`/`parse` return
`Result<_, hl7_2_from_xml_into_er7::Hl7Error>`; an `Err` only ever means the input
isn't well-formed XML, or the message has no usable `MSH`/`FHS`/`BHS`
header — everything else converts, falling back gracefully rather than
failing (`spec/index.md` §5).

## What it does

- **XML reading via `hl7-2-xml-lite-helper`** (re-exported as `xml`), for
  exactly the subset v2.xml uses: nested elements, text, the predefined
  entities and numeric character references, with attributes, comments, and
  the XML declaration recognized and skipped. The helper has no
  dependencies of its own, and is shared with two other crates in this
  family that need the same subset (see the helper's own `spec/index.md`).
- **Position-based reconstruction, no data-type dictionary** (`src/reconstruct.rs`):
  every field, component, and subcomponent element's *position* comes from
  the number after its name's last `.`, so this crate reconstructs a
  message correctly whether the forward crate rendered typed names
  (`<XPN.1>`) or fell back to generic ones (`<PID.5.1>`).
- **Group flattening**: message-structure group elements are recognized by
  their dotted name and flattened back into a plain segment sequence — no
  message-structure grammar is needed in this direction.
- **Delimiter recovery**: the header's `.1`/`.2` fields are reassembled into
  a synthetic header line and handed to `er7::Separators::from_header`,
  reusing `er7`'s own delimiter parsing.
- **Faithful re-escaping**: decoded leaf text is retokenized against `er7`'s
  own escape-sequence vocabulary, so delimiter characters are re-escaped
  while formatting sequences the forward crate never decoded (`\.br\`,
  `\H\`, …) are written back exactly as they were.
- **HL7 null vs empty**: an element whose text is `""` reconstructs as the
  explicit null ("delete this"); a self-closing or empty element
  reconstructs as an *empty* value ("nothing was sent"). The XML Encoding
  Rules give the two opposite meanings, so this crate keeps them apart —
  padding an XSD-shaped document with empty elements does not turn it into
  a message full of deletion markers.

## Limitations

See [`spec/index.md`](spec/index.md) §5 for the full list; the two worth
knowing up front:

- A field repetition that was present but entirely blank (not the explicit
  null) is dropped by the *forward* crate's own encoding and cannot be
  recovered here — this is a property of v2.xml as that crate writes it,
  not something this crate's reversal introduces.
- One v2.xml document converts to one ER7 message; there is no batch-file
  convention on the XML side to split.

## Documentation

- [`spec/index.md`](spec/index.md) — the normative specification (source of
  truth for behavior).
- `cargo doc --no-deps --open` — rustdoc for the library API.
- [`AGENTS.md`](AGENTS.md) — conventions and required checks for anyone
  (human or agent) changing this code; `CLAUDE.md` points here too.
- [`samples/`](samples/) — example v2.xml input files, including the exact
  documents the sibling crate's own golden tests produce.

## Development

```sh
cargo test                                # unit + integration tests, incl. round trips through real samples
cargo clippy --all-targets -- -D warnings # lint-clean
cargo fmt --check                         # formatting
cargo rustdoc --lib -- -W missing-docs    # every public item is documented
cargo run -- samples/orm_o01.xml
```

## Related crates

Four small crates, all built on the same [`er7`](https://crates.io/crates/er7)
encoding layer, cover ER7's two directions against both target formats:

| Crate | Direction |
|---|---|
| [`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml) | ER7 → v2.xml XML |
| **`hl7-2-from-xml-into-er7`** (this crate) | v2.xml XML → ER7 |
| [`hl7-2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json) | ER7 → typed JSON |
| [`hl7-2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7) | typed JSON → ER7 |

## References

- [`er7`](https://crates.io/crates/er7) — the ER7 encoding layer this crate
  writes onto
- [`hl7-2-from-er7-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml)
  — the forward crate this one inverts
- [`hl7-2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json)
  and [`hl7-2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7)
  — the JSON pair, doing the same job for the JSON mapping
- [HL7 v2.xml encoding](https://www.hl7.eu/refactored/encoding02xml.html)
- [XML schemas for HL7 v2.5 and earlier (Australian Digital Health Agency)](https://implementer.digitalhealth.gov.au/standards/v2-xml-xml-schemas-for-hl7-version-2-5-and-earlier)
