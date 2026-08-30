---
name: hl7-skill
description: Work with Health Level Seven (HL7®) v2 messages using the hl7-rust workspace — parse, query, validate, edit, and render ER7 (pipe-delimited); convert between ER7, v2.xml, and JSON; move messages over MLLP or SOAP; build a vendor dictionary from XSDs. Use when the task involves an HL7 v2 message, an ER7/pipe-delimited clinical message, an MSH/PID/OBX segment, an ADT/ORU/ORM message, or converting one of those to or from XML/JSON.
---

# HL7 v2 with hl7-rust

This skill covers the `hl7-rust` workspace: a Cargo workspace of small,
single-purpose crates for HL7 v2 (2.1–2.9). Prefer the command-line tools
below for one-off inspection and conversion; use the library crates when the
result needs to live inside a Rust program.

Every crate's `spec/index.md` is the single normative source of truth for
its behavior — this file is a map and a set of recipes, not the
specification. When a detail here and a crate's `spec/index.md` disagree,
the spec wins.

## Which crate or tool for which job

| Task | Tool |
|---|---|
| Look at, query, validate, edit, or re-render an ER7 message | `hl7-v2` CLI, or the `hl7-2` crate / `hl7::v2` |
| ER7 → typed JSON, or JSON → ER7 | `hl7-2-from-er7-into-json`, `hl7-2-from-json-into-er7` |
| ER7 → official v2.xml, or v2.xml → ER7 | `hl7-2-from-er7-into-xml`, `hl7-2-from-xml-into-er7` |
| Send/receive HL7 v2 over a raw TCP socket | `hl7-2-mllp` |
| Send/receive HL7 v2 over HTTP/SOAP | `hl7-2-soap` |
| Teach the parser a vendor's dialect from its XSDs | `hl7-2-from-xsd-into-json-dictionary` |
| HL7 v3 (RIM backbone, coded values, message envelope) | `hl7-3`, `hl7-3-soap` — a foundation, not a full v3 implementation |

None of these crates validates against a full conformance profile
(cardinality, table membership) — `hl7-v2 --check` / `--strict` catches
structural problems only. None of them is a transport in the network-stack
sense beyond framing (MLLP) or enveloping (SOAP): bring your own socket or
HTTP client.

## Install

```sh
cargo install hl7-2                              # binary: hl7-v2
cargo install hl7-2-from-er7-into-xml
cargo install hl7-2-from-xml-into-er7
cargo install hl7-2-from-er7-into-json
cargo install hl7-2-from-json-into-er7
cargo install hl7-2-from-xsd-into-json-dictionary

cargo add hl7            # library: hl7::v2 and hl7::v3
cargo add hl7-2-mllp
cargo add hl7-2-soap
```

## Recipes

**Look at an unfamiliar message, with its paths:**

```sh
hl7-v2 --paths samples/vendor.hl7
```

The bracketed path beside each value (`PID[1]-3[1].1`) is exactly what
`--query`, the library's `get`, and a `#[hl7(...)]` attribute all use to
read that same value — start here before writing any code against a new
message shape.

**Pull one field out of a batch of messages:**

```sh
cat inbox/*.hl7 | hl7-v2 --query PID-3
```

**Validate, with an exit status a script can branch on** (0 = ok, 1 = not a
message at all, 2 = validation failed):

```sh
hl7-v2 --check samples/adt_a01.hl7 || echo "rejected: $(hl7-v2 --check samples/adt_a01.hl7)"
```

**Edit a field and re-emit ER7:**

```sh
hl7-v2 --set 'PID-8=F' --er7 in.hl7 > out.hl7
```

**Convert to/from JSON or XML:**

```sh
hl7-2-from-er7-into-json samples/orm_o01.hl7        # ER7 -> typed JSON
hl7-2-from-er7-into-xml  samples/orm_o01.hl7         # ER7 -> official v2.xml
hl7-2-from-json-into-er7 samples/orm_o01.json        # typed JSON -> ER7
hl7-2-from-xml-into-er7  samples/orm_o01.xml         # v2.xml -> ER7

# Round trip as a smoke test
hl7-2-from-er7-into-xml in.hl7 | hl7-2-from-xml-into-er7 | diff - in.hl7
```

Both forward converters accept `--dictionary FILE` to convert against a
JSON dictionary (e.g. one built from a vendor's XSDs) instead of the bundled
HL7 v2.5 tables.

**Build a vendor dictionary from a directory of v2.xml XSDs:**

```sh
hl7-2-from-xsd-into-json-dictionary schemas/paris \
    --name paris \
    --alias ADT_A28=ADT_A05 \
    --inherits 2.5 \
    -o paris.json
```

**As a library — parse, query, and check:**

```rust
use hl7::v2;

let message = v2::parse(er7_text)?;
assert_eq!(message.structure_id(), "ORU_R01");
let patient_id = message.get("PID-5.1")?;
```

**MLLP (TCP) and SOAP (HTTP)** are library-only, no CLI: see
[`hl7-2-mllp`](../hl7-2-mllp/README.md) and
[`hl7-2-soap`](../hl7-2-soap/README.md) for the framing/enveloping API — each
handles only its own protocol layer, not the socket or HTTP stack itself.

## Things worth knowing before you rely on this

- **No field is ever a JSON number.** HL7 numeric text carries leading
  zeros, explicit signs, and trailing precision that a JSON number would
  silently destroy — every scalar converts as a string.
- **The explicit HL7 null (`""`) and an absent field are different things**
  and are preserved as different things across every conversion.
- **Nothing here is a validator against a conformance profile.** `hl7-v2
  --check`/`--strict` and the crates' own `Error`s catch structural
  problems (missing `MSH`, malformed input) only — not cardinality or table
  membership.
- **PHI handling:** every crate keeps a message as text in memory only —
  nothing is written to disk, logged, or sent over a network by the library
  itself. See [`spec/phi/index.md`](../spec/phi/index.md) for exactly what
  that does and does not cover, including where a value can still escape
  into a log via your own error handling.
- **Conformance claim:** see [`spec/conformance/index.md`](../spec/conformance/index.md)
  for exactly what "supports HL7 v2 releases 2.1–2.9" means — named
  segments, types, and structures, and what happens outside them.

## Where to go deeper

- [`README.md`](../README.md) — the full crate table and how the crates
  depend on each other.
- Each crate's own `README.md` (tour) and `spec/index.md` (normative
  specification, section by section).
- [`hl7-rust.github.io`](https://hl7-rust.github.io) — narrative guides,
  tutorials, and a `/docs/cli/` page with every flag of every binary.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
