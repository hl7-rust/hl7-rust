[index](../index.md) → §4 The round-trip guarantee

# §4 The round-trip guarantee

## 4.1 Rule S14: the guarantee

For any value `v` of any wrapper type in this crate, and any Serde data
format `F` capable of representing arbitrary UTF-8 strings, booleans,
`Option`, sequences, and string-keyed maps:

```text
v  --serialize with F-->  bytes  --deserialize with F-->  v'
```

then `v' == v`. Every field of every `hl7-3` type this crate wraps is
public plain data, and every one of them is on the wire (S3), so nothing
is derived or dropped in either direction. The integration tests state
this for a full message parsed from XML, for every RIM class, and for an
`Element` tree with attributes, text, and children at several depths.

## 4.2 What is not promised: XML

HL7® v3's own serialization is XML, and a natural question is whether
`Element` → JSON → `Element` → XML reproduces the original document. This
crate does not promise that, for two reasons that are `hl7-3`'s, not this
crate's:

- `hl7-3` has no XML writer (its spec §1), so there is no "back to XML"
  step for this crate to be transparent to.
- `hl7-3`'s reader normalizes on the way in — whitespace-only text beside
  children is dropped, namespace prefixes are kept but not resolved,
  entity references are decoded (`hl7-2-xml-lite-helper`'s spec §3) — so
  the document is already not byte-recoverable from the `Element` before
  this crate sees it.

What *is* promised is that the `Element` value this crate is given is the
`Element` value that comes back: same name, same attributes, same text,
same children, recursively.

## 4.3 What is not promised: the envelope's XML shape

`hl7-3` reads a message into six fields and keeps only the *first* element
under `<sender>`, `<receiver>`, and `<subject>` (its spec §5.1). A message
value therefore does not contain everything the document did, and this
crate serializes the value, not the document. A caller who needs the whole
document keeps the `Element` from `hl7_3::xml::parse` and serializes that
instead — which round-trips per §4.1.

## 4.4 The distinction that must survive

HL7 v3 draws its version of the absent/null distinction with
`nullFlavor` (`hl7-3`'s spec §3.6). This crate does not collapse it: an
`Option<String>` that is `None` is `null` on the wire, one that is
`Some("")` is `""`, and a `NullFlavor` — read by the caller off an
element — carries its code exactly, unknown codes included (S6).
`Element::attributes` carries a `nullFlavor` attribute like any other,
so the raw form survives too.

## 4.5 Testing this guarantee

Every fixture in `tests/integration.rs` follows the same shape: build or
parse a value, serialize, deserialize, assert equality. New fixtures added
for a bug fix should follow the same pattern rather than asserting on the
JSON's own shape, which is already covered by
[§2](../02-wire-shapes/index.md) and the per-module unit tests.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
