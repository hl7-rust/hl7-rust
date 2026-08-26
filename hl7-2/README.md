# HL7 v2

> HL7® and FHIR® are registered trademarks of HL7. We are requesting permission to use it here. Use of the trademarks does not constitute endorsement of this library by HL7.

Parse, navigate, validate, modify, and write Health Level Seven (HL7)
version 2 messages, as a Rust library and command-line tool — in three
modes that share one set of internals.

HL7 v2 is the format most healthcare data still moves in, and the hard part
is not the syntax. That is pipes and carets, and the
[`er7`](https://crates.io/crates/er7) crate this one is built on already
handles it. The hard part is knowing what the pipes and carets *mean* in
the release the sender speaks. This crate owns that: the per-release
data-type tables, the message structures, and the three ways to apply them.

```
er7                      the ER7 encoding: delimiters, escapes, paths,
                         byte-for-byte rendering, batch splitting
  |
hl7-2                   this crate: the HL7 v2 dictionary — releases
                         2.1-2.9, data types, message structures; three
                         modes; mutation; validation
  |
  +-- hl7-2-mllp                  transport (MLLP over TCP)
  +-- hl7-2-from-er7-into-json    format conversions
  +-- hl7-2-from-er7-into-xml
  +-- hl7-2-from-json-into-er7
  +-- hl7-2-from-xml-into-er7
```

**This crate is published standalone as `hl7-2`.** Most users get it
through the [`hl7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7) umbrella crate instead,
which re-exports it as `hl7::v2` — each HL7 standard gets a module of its
own there, leaving room for `hl7::v3` and `hl7::fhir`, because a "message",
a "segment", and a "code" mean different things in each, and one flat
namespace would only invite mixing them up. Depend on `hl7-2` directly only
if you specifically want v2 with no umbrella indirection. The command-line
tool is `hl7-2` either way.

This README is a tour. [`spec/index.md`](spec/index.md) is the normative,
section-by-section specification of every rule — the single source of truth
this crate implements against.

## Three modes

### Generic — for the vendor you have never seen

Parse anything into a navigable tree. Nothing is rejected, nothing is
dropped, and what the dictionary recognises gets a name from HL7's own
vocabulary.

```rust
let message = hl7_2::parse(text)?;
let tree = message.tree();

assert_eq!(tree.name(), "ORU_R01");
assert_eq!(tree.find("XPN.1").unwrap().text(), "EVERYWOMAN");

// Every node knows the path that reads it back.
let second = tree.find_all("OBX").nth(1).unwrap();
assert_eq!(second.path(), "OBX[2]");
assert_eq!(message.get("OBX[2]-5.2")?.as_deref(), Some("Rh positive"));
```

Segments are grouped into the message structure when they fit it —
`ORU_R01.PATIENT_RESULT.ORDER_OBSERVATION.OBSERVATION` — and read flat when
they do not.

### Schema-based — for the vendor whose format is not frozen

Write the shape as JSON, load it at runtime, and adding a field is a
configuration change rather than a release.

```rust
let dictionary = hl7_2::Dictionary::from_json(r#"{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}"#, "acme")?;

let options = hl7_2::Options::new().with_dictionary(std::sync::Arc::new(dictionary));
let message = hl7_2::parse_with_options(text, &options)?;

// The vendor's own segment now reads like any standard one.
assert_eq!(message.tree().find("XPN.2").unwrap().text(), "JOHN");
```

The same format describes the bundled releases, so a schema can inherit one
and state only its dialect.

### Struct-based — for the feed that does not change

```rust
use hl7_2::{FromHl7, Raw};

#[derive(FromHl7)]
struct Admission {
    #[hl7("PID-3.1")]  patient_id: String,
    #[hl7("PID-7.1")]  birth_date: Option<String>,
    #[hl7("PID-3")]    all_identifiers: Vec<String>,
    #[hl7(raw)]        raw: Raw,
}

let admission: Admission = hl7_2::parse(text)?.decode()?;
assert_eq!(admission.patient_id, "241900");

// The one vendor field no struct models — same object, no second parse.
assert_eq!(admission.raw.get("ZPD-1")?.as_deref(), Some("local"));
```

That last field is the point. Real feeds are stable until they are not, and
the usual choice at that moment is to re-parse the raw message or rewrite
the library. A `Raw` field keeps the whole parsed message beside the typed
data, so the fallback is a method call. Requires the `derive` feature:

```toml
hl7-2 = { version = "0.2", features = ["derive"] }
```

## Walkthrough: from a message you have never seen to a typed struct

The three modes are not three libraries to choose between. They are three
stages of the same job, and a real integration walks through them in order.

**Stage 1 — look at it.** A vendor sends a message and nobody knows what is
in it. Start with the tool, not with code:

```sh
$ hl7-v2 --paths samples/vendor.hl7
ADT_A01
  MSH  [MSH[1]]
    ...
  ZAC  [ZAC[1]]
    ZAC.1 = 7           [ZAC[1]-1[1]]
    ZAC.2               [ZAC[1]-2[1]]
      ZAC.2.1 = SMITH   [ZAC[1]-2[1].1]
      ZAC.2.2 = JOHN    [ZAC[1]-2[1].2]
    ZAC.3 = 20260814    [ZAC[1]-3[1]]
```

Everything standard is already named — `PID.5` broke into `XPN.1`, `XPN.2`
— and the vendor's own `ZAC` is there positionally, nothing lost. The
bracketed paths are not decoration: each one is what reads that value back.

```sh
$ hl7-v2 --query 'ZAC-2.1' samples/vendor.hl7
SMITH
JONES
```

**Stage 2 — write down what you learned.** `ZAC.2` is clearly a name. Say
so, in JSON, without touching the code:

```json
{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}
```

```sh
$ hl7-v2 --dictionary samples/acme.json --flat samples/vendor.hl7 | grep -A 2 'ZAC.2$'
    ZAC.2
      XPN.1 = SMITH
      XPN.2 = JOHN
```

`ZAC.2` now reads as an `XPN` like any standard name field — and when the
vendor adds `ZAC-4` next quarter it is one line in a file, not a release.

(Note the two vocabularies. `XPN.1` is a *node name*, which is what the tree
and the JSON and XML sibling crates call that component. `ZAC-2.1` is a
*path*, which is what `--query`, `get`, and `set` take. Names describe;
paths address.)

**Stage 3 — freeze what is stable.** Once the interface has held still long
enough to trust, move it into the type system and let the compiler carry it:

```rust
#[derive(FromHl7)]
struct AcmeAdmission {
    #[hl7("PID-3.1")]  patient_id: String,
    #[hl7("ZAC-2.1")]  clinician_family: String,
    #[hl7("ZAC-3")]    effective: Option<String>,
    #[hl7(raw)]        raw: Raw,
}
```

And keep the `raw` field, because stage 3 is never final — the day a message
arrives with something the struct does not model, you are back at stage 1 on
the same object, with no re-parse and no rewrite.

## Modify and build

A system that reads HL7 usually has to answer in it.

```rust
let mut message = hl7_2::parse(text)?;
message.set("PID-5.2", "EVELYN")?;      // escapes delimiters in the value
message.append_segment("NTE");
message.set("NTE[2]-3", "Amended.")?;
let er7 = message.to_er7();             // valid ER7, ready to send
```

```rust
let ack = hl7_2::builder::acknowledge(&message, "AA", "ACK00001", "20260814080100")
    .build_valid()?;
assert_eq!(ack.get("MSA-2")?.as_deref(), Some("MSG00042"));
```

An unmodified message writes back byte for byte — that guarantee is `er7`'s,
and this crate does not weaken it.

## Validate

Parsing stays lenient: unknown segments, unknown types, and structure
mismatches are never errors. Checking is a separate question with a
separate answer.

```rust
for diagnostic in message.validate() {
    println!("{diagnostic}");  // error: MSA[1]-4[1]: "many" is not a valid NM value
}
```

Diagnostics split by whose problem it is. **Errors** are the message
contradicting the dictionary it claims: a required segment missing, a
numeric field holding letters. **Warnings** are the dictionary not covering
the message: an unknown segment, a structure with no grammar yet. Strict
mode rejects the first and allows the second:

```rust
let options = hl7_2::Options::new().strict();
match hl7_2::parse_with_options(text, &options) {
    Err(hl7_2::Error::Invalid(diagnostics)) => { /* every error-level finding */ }
    Ok(message) => { /* conformant */ }
    Err(other) => { /* not a message at all */ }
}
```

A local Z-segment does not make a conformant message fail — most real
interfaces carry one.

## Versions

Releases 2.1 through 2.9, chosen from MSH-12 or forced with
`Options::with_version`. A release string this crate has no dictionary for
resolves to the nearest older one (`2.5.2` reads as `2.5.1`) rather than
failing.

v2.5 is the complete base dictionary; every other release is a delta of it
covering the differences this crate models today — MSH-9 without a
message-structure component before v2.3.1, the one-field `ERR` before v2.5,
`TS` withdrawn in favour of `DTM` from v2.7 — and inherits the rest. That
incompleteness is bounded by design: an unmodelled difference costs a typed
name, never a rejected message or a lost value. See
[`spec/index.md` §3.4](spec/index.md) for exactly what each release claims.

## Command line

```sh
# Look at a message you have never seen
hl7-v2 samples/oru_r01.hl7

# Pull out every result value
hl7-v2 --query OBX-5 samples/oru_r01.hl7

# Check it, with an exit status a shell can act on
hl7-v2 --check samples/adt_a01.hl7

# Read a vendor dialect
hl7-v2 --dictionary samples/acme.json samples/vendor.hl7

# Change something and write it back out
hl7-v2 --set 'PID-8=F' --er7 samples/orm_o01.hl7
```

`hl7-v2 --help` lists everything. Exit status is 0 on success, 1 on a usage
or parse error, and 2 when `--check` or `--strict` found something wrong.

## Dependencies

One: [`er7`](https://crates.io/crates/er7), which has none of its own. The
JSON reader that loads dictionaries is hand-written here for the same
reason the sibling crates hand-write their writers — in a domain where
dependency trees get audited, a two-crate tree is worth a few hundred
lines. Enabling the `derive` feature adds `hl7-2-derive`, and with it
`syn` and `quote`, for callers who want the macros.

## Install

```sh
cargo add hl7-2                            # library
cargo add hl7-2 --features derive          # library with the derive macros
cargo install hl7-2                        # command-line tool
```

Most users should instead depend on the [`hl7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7)
umbrella crate (`cargo add hl7`), which re-exports this crate as `hl7::v2`.

## See also

- [`spec/index.md`](spec/index.md) — the normative specification
- [`er7`](https://github.com/er7-rust/er7-rust) — the ER7 encoding layer
- [`hl7-2-derive`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-derive) — the derive macros
- [`hl7-2-mllp`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-mllp) — MLLP: sending and
  receiving these messages over TCP
- [`hl7-2-from-er7-into-json`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json),
  [`-into-xml`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml),
  [`hl7-2-from-json-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7),
  [`hl7-2-from-xml-into-er7`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-xml-into-er7)
  — format conversions

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
