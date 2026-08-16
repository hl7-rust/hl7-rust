# JSON to HL7 2.5 using Rust

Convert HL7 version 2.5 messages from the typed JSON representation the
sibling
[`hl7-v2-from-er7-into-json`](https://github.com/hl7-rust/hl7-v2-from-er7-into-json)
crate produces back to the traditional pipe-delimited **ER7** encoding, as
a Rust library and command-line tool.

This is the inverse of that crate, and depends only on the same
[`er7`](https://crates.io/crates/er7) encoding layer it does — no HL7 v2.5
data-type dictionary is needed to reverse the conversion, because the
forward crate's key names always carry the field/component/subcomponent
*position* as the number after the key's last `.`, whether or not the name
in front of it is a recognized data type. See
[`spec/index.md`](spec/index.md) for exactly how that works and its
limits — it is the normative specification this crate implements.

A JSON fragment such as:

```json
{
  "PID.1": "1",
  "PID.3": { "CX.1": "241900" },
  "PID.5": { "XPN.1": { "FN.1": "TEST" }, "XPN.2": "FOUAZ" }
}
```

converts back to:

```
PID|1||241900||TEST^FOUAZ
```

## Usage

### Command line

```sh
# From a file to stdout
cargo run -- samples/orm_o01.json

# From stdin, to a file
cat samples/oru_r01.json | cargo run -- -o out.hl7

# Choose the segment terminator
cargo run -- --terminator crlf samples/orm_o01.json
```

Message-structure group keys (`"ORM_O01.PATIENT"`,
`"ORU_R01.ORDER_OBSERVATION"`, …) are flattened automatically, and a
repeating field/segment/group's JSON array un-arrays back into its
repetitions or repeated segments — grouped and `--flat`, single-occurrence
and repeated, all reconstruct the same message.

### Library

```rust
let json = r#"{
  "ORM_O01": {
    "MSH": { "MSH.1": "|", "MSH.2": "^~\\&", "MSH.9": {"MSG.1": "ORM", "MSG.2": "O01"} },
    "ORM_O01.PATIENT": {
      "PID": { "PID.5": { "XPN.1": {"FN.1": "TEST"}, "XPN.2": "FOUAZ" } }
    }
  }
}"#;
let er7 = hl7_v2_from_json_into_er7::convert(json)?;
```

See also `convert_with_options` for the segment terminator
(`er7::RenderOptions`, re-exported as `hl7_v2_from_json_into_er7::er7::RenderOptions`),
and `parse` when the caller wants the full `er7::Message` — to query or
edit it — rather than just its ER7 text:

```rust
use hl7_v2_from_json_into_er7::parse;

let message = parse(json)?;
assert_eq!(message.query("PID-5.1")?.as_deref(), Some("TEST"));
```

`convert`/`convert_with_options`/`parse` return
`Result<_, hl7_v2_from_json_into_er7::Hl7Error>`; an `Err` only ever means the input
isn't well-formed JSON, isn't shaped like a converted message (a single-key
object over an object of segments), or the message has no usable
`MSH`/`FHS`/`BHS` header — everything else converts, falling back
gracefully rather than failing (`spec/index.md` §5).

## What it does

- **A minimal, dependency-free JSON reader** (`src/json.rs`), the full RFC
  8259 grammar including `\uXXXX` surrogate pairs, with numbers and
  booleans tolerated (coerced to text) even though the forward crate never
  emits them.
- **Position-based reconstruction, no data-type dictionary** (`src/reconstruct.rs`):
  every field, component, and subcomponent key's *position* comes from the
  number after its key's last `.`, so this crate reconstructs a message
  correctly whether the forward crate rendered typed keys (`"XPN.1"`) or
  fell back to generic ones (`"PID.5.1"`).
- **Group flattening and array un-nesting**: message-structure group keys
  are recognized by their dotted name and flattened back into a plain
  segment sequence, and a JSON array (repeating field, segment, or group)
  un-arrays back into repeated occurrences — no message-structure grammar
  is needed in this direction.
- **Delimiter recovery**: the header's `.1`/`.2` fields are reassembled into
  a synthetic header line and handed to `er7::Separators::from_header`,
  reusing `er7`'s own delimiter parsing.
- **Faithful re-escaping**: decoded leaf text is retokenized against `er7`'s
  own escape-sequence vocabulary, so delimiter characters are re-escaped
  while formatting sequences the forward crate never decoded (`\.br\`,
  `\H\`, …) are written back exactly as they were.
- **HL7 null**: JSON `null` reconstructs as the explicit null `""` at that
  position.

## Limitations

See [`spec/index.md`](spec/index.md) §5 for the full list; the two worth
knowing up front:

- A field repetition that was present but entirely blank (not the explicit
  null) is dropped by the *forward* crate's own encoding and cannot be
  recovered here — this is a property of that crate's JSON mapping, not
  something this crate's reversal introduces.
- One JSON document converts to one ER7 message; there is no batch-file
  convention on the JSON side to split.

## Documentation

- [`spec/index.md`](spec/index.md) — the normative specification (source of
  truth for behavior).
- `cargo doc --no-deps --open` — rustdoc for the library API.
- [`AGENTS.md`](AGENTS.md) — conventions and required checks for anyone
  (human or agent) changing this code; `CLAUDE.md` points here too.
- [`samples/`](samples/) — example JSON input files, including the exact
  documents the sibling crate's own golden tests produce.

## Development

```sh
cargo test                                # unit + integration tests, incl. round trips through real samples
cargo clippy --all-targets -- -D warnings # lint-clean
cargo fmt --check                         # formatting
cargo rustdoc --lib -- -W missing-docs    # every public item is documented
cargo run -- samples/orm_o01.json
```

## References

- [`er7`](https://crates.io/crates/er7) — the ER7 encoding layer this crate
  writes onto
- [`hl7-v2-from-er7-into-json`](https://github.com/hl7-rust/hl7-v2-from-er7-into-json)
  — the forward crate this one inverts
- [`hl7-v2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-v2-from-xml-into-er7)
  — the XML sibling of this crate
- [RFC 8259 — The JavaScript Object Notation (JSON) Data Interchange Format](https://www.rfc-editor.org/rfc/rfc8259)
