# HL7 v2 from ER7 into XML

Convert Health Level Seven (HL7) version 2.5 messages from the traditional
pipe-delimited Encoding Rules version 7 (ER7) encoding to the official HL7
**v2.xml** XML representation (`urn:hl7-org:v2xml`), as a Rust library and
command-line tool.

The ER7 encoding itself comes from the [`er7`](https://crates.io/crates/er7)
crate. This crate is the layer above it: the HL7 v2.5 data-type tables that
name XML elements, the message-structure grammars that group segments, and
the XML renderer. Since 0.5.0 those tables and grammars come from the
[`hl7-2`](https://crates.io/crates/hl7-2) dictionary rather than being
hand-written here, which is also what lets a caller pass `--dictionary` to
convert against a vendor's own XML Schema instead of the bundled v2.5
release (see `spec/index.md` §2 and §4a).

This is one of four sibling crates in the `hl7-rust` family — see
[Related crates](#related-crates) below. Its JSON counterpart is
[`hl7-v2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-er7-into-json)
(same parser, same data-type tables, same grammars, rendered as JSON
instead); its inverse is
[`hl7-v2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-xml-into-er7),
which reads this crate's own v2.xml output back into ER7.

This README is a tour. [`spec/index.md`](spec/index.md) is the normative,
section-by-section specification of every conversion rule — the single
source of truth this crate implements against; consult it for exact
behavior (delimiter resolution, typed naming, grouping, fallbacks, CLI
contract).

An ER7 fragment such as:

```
PID|1||241900||TEST^FOUAZ
```

converts to the v2.xml structure, with components named after their HL7 v2.5
data types:

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

## Usage

### Command line

```sh
# From a file to stdout
cargo run -- samples/orm_o01.hl7

# From stdin, to a file
cat samples/oru_r01.hl7 | cargo run -- -o out.xml

# Disable message-structure grouping
cargo run -- --flat samples/orm_o01.hl7

# Convert against a dictionary built from XSDs instead of the bundled v2.5 tables
cargo run -- --dictionary my-dialect.json samples/orm_o01.hl7

# Let that dictionary decide the document's exact shape (required fields,
# repeatability, declared components) rather than treating it as a table
cargo run -- --dictionary my-dialect.json --schema-shape samples/orm_o01.hl7
```

`--dictionary <FILE>` reads a JSON dictionary — such as one built by
`hl7-v2-from-xsd-into-json-dictionary` from a directory of XSDs — in place
of the bundled HL7 v2.5 tables. `--schema-shape` changes how that
dictionary is read: instead of only naming what a field *is*, it decides
what the document *contains* — required fields are written even when
empty, fields that can't repeat keep their repetition separator as text,
and no field the dictionary doesn't declare is written (`spec/index.md`
§4a).

Input may hold one message, several messages, or an HL7 batch file (FHS/BHS
envelopes are dropped); each message becomes one XML document.

### Library

```rust
let er7 = "MSH|^~\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\r\
           PID|1||241900||MEDIANO^FOUAZ\r\
           ORC|NW|ORD1";
let xml = hl7_v2_from_er7_into_xml::convert(er7)?;
```

See also `convert_with_options` (e.g. `Options { flat: true }`) and
`split_messages` for batch input:

```rust
use hl7_v2_from_er7_into_xml::{convert, split_messages};

let batch = "MSH|^~\\&|A||||1||ACK|1|P|2.5\rMSA|AA|1\r\
             MSH|^~\\&|B||||2||ACK|2|P|2.5\rMSA|AA|2";
for message in split_messages(batch) {
    match convert(&message) {
        Ok(xml) => println!("{xml}"),
        Err(e) => eprintln!("skipping malformed message: {e}"),
    }
}
```

`convert`/`convert_with_options` return `Result<String, hl7_v2_from_er7_into_xml::Hl7Error>`;
an `Err` only ever means the message has no usable MSH header (empty input,
missing MSH, or a malformed MSH header) — everything below that always
converts, falling back to generic names or a flat layout rather than
failing (`spec/index.md` §6).

## What it does

- **ER7 parsing** at every level: segments, fields, repetitions (`~`),
  components (`^`), and subcomponents (`&`).
- **Dynamic delimiters**: the separator set is read from MSH-1/MSH-2 rather
  than hardcoded, and MSH-1/MSH-2 are emitted literally per the standard.
- **Escape sequences**: `\F\ \S\ \T\ \R\ \E\` decode to the delimiter
  characters and `\Xhh..\` decodes hex bytes, before XML escaping.
  Unrecognized sequences (formatting commands such as `\.br\`) are kept
  literally.
- **Typed element names**: built-in HL7 v2.5 tables map each field of the
  common segments (MSH, SFT, EVN, PID, PD1, NK1, PV1, PV2, ROL, DG1, PR1,
  ORC, OBR, OBX, NTE, AL1, IN1, MRG, MSA, ERR, DSC, BLG, CTI, SPM) to its
  data type, and each composite type (CX, XPN, XCN, XAD, CE, CWE, EI, HD,
  TS, ...) to its component types — producing `<PID.5><XPN.1><FN.1>` style
  nesting. Anything outside these tables (Z-segments, uncommon types) still
  converts, using positional generic names instead (`spec/index.md` §4.2).
- **OBX-5 variable typing**: the value type declared in OBX-2 (CE, CX, SN,
  ...) names the OBX-5 components.
- **HL7 null**: the explicit null `""` becomes an empty element.
- **Message-structure groups**: for known structures the segments are nested
  into their official groups, e.g. `<ORM_O01.PATIENT>` or
  `<ORU_R01.ORDER_OBSERVATION>`. Grammars are included for ACK, ADT_A01
  (also used by ADT^A04/A08/A13), ORM_O01, and ORU_R01. The root element
  name comes from MSH-9.3 when present, otherwise from MSH-9.1/9.2.

## Fallback behavior

Fidelity degrades gracefully instead of failing:

- A message whose segment sequence does not fit its declared structure
  (e.g. it contains Z-segments, or uses a structure without a built-in
  grammar) renders with all segments **flat** under the root element.
- Fields of unknown segments — and segment fields beyond the built-in
  tables — use positional generic names: `<ZDS.1>`, `<ZDS.1.1>`, and so on.

## Limitations

- Not a validator: no XSD validation, cardinality, or table checking is
  performed; the input is assumed to be sensible HL7 v2.5.
- Only the four message structures listed above are grouped; everything
  else renders flat (which is still well-formed, lossless XML).
- ORM_O01 order detail supports the common OBR choice; RQD/RQ1/RXO/ODS/ODT
  detail segments cause a flat rendering.
- Formatting escape sequences are preserved as literal text rather than
  mapped to `<escape/>` elements.

## Documentation

- [`spec/index.md`](spec/index.md) — the normative specification (source of
  truth for behavior).
- `cargo doc --no-deps --open` — rustdoc for the library API, including a
  runnable example in the crate-level docs (`src/lib.rs`).
- [`AGENTS.md`](AGENTS.md) — conventions and required checks for anyone
  (human or agent) changing this code; `CLAUDE.md` points here too.
- [`samples/`](samples/) — example ER7 input files (`orm_o01.hl7`,
  `oru_r01.hl7`) used by the commands above and by manual testing.

## Development

```sh
cargo test                                # unit + integration tests, incl. a golden ORM_O01 document
cargo clippy --all-targets -- -D warnings # lint-clean
cargo fmt --check                         # formatting
cargo rustdoc --lib -- -W missing-docs    # every public item is documented
cargo run -- samples/orm_o01.hl7
```

## Related crates

Four small crates, all built on the same [`er7`](https://crates.io/crates/er7)
encoding layer, cover ER7's two directions against both target formats:

| Crate | Direction |
|---|---|
| **`hl7-v2-from-er7-into-xml`** (this crate) | ER7 → v2.xml XML |
| [`hl7-v2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-xml-into-er7) | v2.xml XML → ER7 |
| [`hl7-v2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-er7-into-json) | ER7 → typed JSON |
| [`hl7-v2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-json-into-er7) | typed JSON → ER7 |

## Round trip with the reverse crate

Because [`hl7-v2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-xml-into-er7)
reads back exactly what this crate writes, the two compose into a lossless
round trip you can run from the shell — both crates live in this workspace,
so no `cd` is needed:

```sh
cargo run -- samples/orm_o01.hl7 \
  | cargo run -p hl7-v2-from-xml-into-er7 --
```

The output is the original ER7 message, canonicalized (see
[`spec/index.md`](spec/index.md) §2.1) — a good smoke test after changing
either crate's naming rules, since a drift in one breaks the other's
assumptions (`AGENTS.md` explains why).

## References

- [HL7 v2.xml encoding](https://www.hl7.eu/refactored/encoding02xml.html)
- [XML schemas for HL7 v2.5 and earlier (Australian Digital Health Agency)](https://implementer.digitalhealth.gov.au/standards/v2-xml-xml-schemas-for-hl7-version-2-5-and-earlier)
- [Microsoft BizTalk: HL7 2.X and 2.XML schemas](https://learn.microsoft.com/en-us/biztalk/adapters-and-accelerators/accelerator-hl7/hl7-2-x-and-2-xml-schemas)
- [InterSystems Healthcare HL7 XML](https://github.com/intersystems-ib/Healthcare-HL7-XML)
- [`hl7-v2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-er7-into-json) — this crate's JSON sibling
- [`hl7-v2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-v2-from-xml-into-er7) — reads this crate's v2.xml output back into ER7
