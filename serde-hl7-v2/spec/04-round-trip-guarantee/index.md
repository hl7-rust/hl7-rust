[index](../index.md) → §4 The round-trip guarantee

# §4 The round-trip guarantee

## 4.1 The guarantee, for `Message`

For any ER7 text `t` that `hl7_2::parse` accepts, any `Options` `o`, and
any Serde data format `F` capable of representing arbitrary UTF-8 strings:

```text
Message::parse_with_options(t, &o)?  --serialize with F-->  bytes
bytes  --deserialize with F-->  Message m'
```

then `m'.to_er7()` equals whatever `hl7_2::parse_with_options(t, &o)?.to_er7()`
would already produce — canonical input round-trips byte for byte, and
non-canonical terminators are normalized once, at the first parse, exactly
as in plain `hl7-2` — and `m'.version()` equals the original's
`version()`. This is `er7`'s own round-trip guarantee (its spec rule R16),
carried through `hl7-2` and then through a Serde format, with the release
added.

When `o` names a caller's own dictionary, the equality of `version()` and
`to_er7()` still holds through the plain `Deserialize`, but `m'` reads
through the *bundled* dictionary for that release — the wire does not carry
a dictionary (S3). Deserializing through `Message::seed(&o)` instead makes
`m'` read through `o`'s dictionary too (S15), at which point
`m'.tree()` equals the original's `tree()` as well.

## 4.2 What makes it possible

Rule S3: the message travels as ER7 text, which `er7` writes and re-reads
losslessly, plus the one piece of state `hl7-2` adds at parse time that
the text alone cannot reconstruct — the release it was read as, which
`Options::version` may have forced away from MSH-12 and which
`Version::nearest` may have resolved from a release `hl7-2` does not
know. Everything else `hl7_2::Message` holds is derived from those two by
`hl7_2::parse_with_options`, so re-running it is the round trip.

## 4.3 What does not round-trip, and why that is out of scope

- **`Node`** (S14). A tree serializes but does not deserialize; it is a
  view of a message, and `hl7_2::Node` has no public constructor. The
  round trip for a tree is `Message` → any format → `Message` →
  `.tree()`, and `tests/integration.rs`'s
  `the_tree_after_a_round_trip_serializes_identically` pins down that
  the tree's own serialization is byte-identical on both sides.
- **A caller's dictionary through the plain `Deserialize`**, as §4.1
  says. The seed exists for exactly this.
- **`Options::strict`.** Whether the original was parsed strictly is not
  on the wire; strictness is a parse-time policy, not message state, and
  `Message::seed` is where a receiver states its own.

## 4.4 The distinction that must survive alongside the bytes

`er7`'s rules R10/R11 require absent, empty, and the explicit null (`""`)
to stay three different answers, because collapsing them is a
patient-safety bug. Because the message travels as its ER7 text, that
distinction survives trivially on the `Message` path. On the `Node` path
it is carried explicitly: an absent field has no node, an empty one is not
emitted either (`hl7-2`'s tree skips empty fields, its spec §4), and the
explicit null is a node with `"null": true` (S4). The integration test
`the_explicit_null_is_visible_in_the_tree` pins this down.

## 4.5 `Diagnostic`

`Diagnostic` round-trips to an equal value: all four fields are public,
plain data, and required (S5), so `Vec<Diagnostic>` through any format
and back compares equal. Nothing is derived on the way in.

## 4.6 Testing this guarantee

Every fixture in `tests/integration.rs` follows the same shape: parse,
serialize, deserialize, assert the ER7 text and the version are
unchanged, and — for the sample files — that the tree serializes
identically on both sides. New fixtures added for a bug fix should follow
the same pattern rather than asserting on the JSON's own shape, which is
already covered by [§2](../02-wire-shapes/index.md) and the per-module
unit tests.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
