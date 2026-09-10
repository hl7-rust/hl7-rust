# Examples

Runnable programs, one concept each. Run any of them with:

```sh
cargo run -p serde-hl7-v2 --example <name>
```

| Example | Shows |
| ------- | ----- |
| [`v2_round_trip_via_json`](v2_round_trip_via_json.rs) | ER7 → `Message` → JSON → `Message` → ER7, unchanged, with the release riding along — the crate's flagship path |
| [`v2_log_the_tree_as_json`](v2_log_the_tree_as_json.rs) | `Node` and `Diagnostic`: the dictionary-named tree and the validation findings as JSON |
| [`v2_catch_a_typo_with_strict`](v2_catch_a_typo_with_strict.rs) | `Strict<Message>` reporting a mistyped key by name, where the plain type silently falls back to MSH-12 |

See `../README.md` for the tour these examples are drawn from, and
`../spec/index.md` for the normative rules.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
