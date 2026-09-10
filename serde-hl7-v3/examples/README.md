# Examples

Runnable programs, one concept each. Run any of them with:

```sh
cargo run -p serde-hl7-v3 --example <name>
```

| Example | Shows |
| ------- | ----- |
| [`round_trip_via_json`](round_trip_via_json.rs) | XML → `Message` → JSON → `Message`, equal — the crate's flagship path |
| [`decode_the_payload_as_rim`](decode_the_payload_as_rim.rs) | Reading the domain payload into a RIM class with `hl7-3`, then serializing the typed `Role`/`Act` |
| [`catch_a_typo_with_strict`](catch_a_typo_with_strict.rs) | `Strict<Message>` reporting a mistyped key nested deep in the payload's element tree, where the plain type reads it as absent |

See `../README.md` for the tour these examples are drawn from, and
`../spec/index.md` for the normative rules.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
