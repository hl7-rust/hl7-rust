# HL7 2.5 to XML using Rust

Convert HL7 version 2.5 messages from the traditional pipe-delimited ER7
encoding to the official HL7 **v2.xml** XML representation
(`urn:hl7-org:v2xml`), as a Rust library and command-line tool with zero
dependencies.

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
```

Input may hold one message, several messages, or an HL7 batch file (FHS/BHS
envelopes are dropped); each message becomes one XML document.

### Library

```rust
let er7 = "MSH|^~\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\r\
           PID|1||241900||MEDIANO^FOUAZ\r\
           ORC|NW|ORD1";
let xml = hl7_2_5_to_xml::convert(er7)?;
```

See also `convert_with_options` (e.g. `Options { flat: true }`) and
`split_messages` for batch input:

```rust
use hl7_2_5_to_xml::{convert, split_messages};

let batch = "MSH|^~\\&|A||||1||ACK|1|P|2.5\rMSA|AA|1\r\
             MSH|^~\\&|B||||2||ACK|2|P|2.5\rMSA|AA|2";
for message in split_messages(batch) {
    match convert(&message) {
        Ok(xml) => println!("{xml}"),
        Err(e) => eprintln!("skipping malformed message: {e}"),
    }
}
```

`convert`/`convert_with_options` return `Result<String, hl7_2_5_to_xml::Hl7Error>`;
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

## References

- [HL7 v2.xml encoding](https://www.hl7.eu/refactored/encoding02xml.html)
- [XML schemas for HL7 v2.5 and earlier (Australian Digital Health Agency)](https://implementer.digitalhealth.gov.au/standards/v2-xml-xml-schemas-for-hl7-version-2-5-and-earlier)
- [Microsoft BizTalk: HL7 2.X and 2.XML schemas](https://learn.microsoft.com/en-us/biztalk/adapters-and-accelerators/accelerator-hl7/hl7-2-x-and-2-xml-schemas)
- [InterSystems Healthcare HL7 XML](https://github.com/intersystems-ib/Healthcare-HL7-XML)
