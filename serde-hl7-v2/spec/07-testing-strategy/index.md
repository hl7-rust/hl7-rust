[index](../index.md) → §7 Testing strategy

# §7 Testing strategy

## 7.1 Rule coverage

Every rule in [the index](../index.md#rule-index) names the test that
enforces it. A rule with no test is a bug in this table.

| Rule | Test | File |
| ---- | ---- | ---- |
| S1 | `the_crate_has_exactly_two_runtime_dependencies` | `tests/integration.rs` |
| S2 | `no_format_crate_is_a_runtime_dependency` | `tests/integration.rs` |
| S3 | `serializes_version_and_er7_only`, `round_trips_a_full_message_through_json`, `a_forced_version_survives_the_round_trip` | `src/message.rs` |
| S4 | `serializes_six_fields_on_every_node`, `names_paths_and_kinds_come_from_the_dictionary` | `src/node.rs` |
| S5 | `round_trips_a_diagnostic`, `round_trips_real_findings`, `rejects_a_missing_field` | `src/diagnostic.rs` |
| S6 | `round_trips_every_release`, `rejects_an_unknown_release` | `src/version.rs` |
| S7 | `node_kind_round_trips_every_variant`, `a_variant_is_written_as_a_string_not_an_index`, `rejects_an_unknown_variant` | `src/kind.rs` |
| S8 | `ignores_unknown_fields` | `src/message.rs`, `src/diagnostic.rs` |
| S9 | `rejects_a_message_missing_er7`, `rejects_a_missing_field`, `error_messages_still_name_the_missing_field` | `src/message.rs`, `src/diagnostic.rs`, `tests/integration.rs` |
| S10 | `round_trips_every_sample_from_the_hl7_2_crate`, `serializes_version_and_er7_only`, and every shape test above | `tests/integration.rs`, `src/message.rs` |
| S11 | `deref_reaches_the_inner_api`, `deref_and_display_reach_the_inner_node`, `deref_and_display_reach_the_inner_value` | `src/message.rs`, `src/node.rs`, `src/diagnostic.rs`, `src/version.rs` |
| S12 | *by `cargo rustdoc --lib -- -W missing-docs`*, one of the four checks (§7.4) | — |
| S13 | `strict_rejects_an_unknown_field`, `strict_still_requires_er7`, `strict_still_requires_every_field_the_plain_type_does`, `plain_deserialize_is_unaffected_by_strict_existing`, `strict_accepts_a_real_message_with_no_typos` | `src/message.rs`, `src/diagnostic.rs` |
| S14 | `node_has_no_deserialize_impl`, `the_tree_after_a_round_trip_serializes_identically` | `tests/integration.rs` |
| S15 | `seed_reads_through_the_callers_dictionary`, `seed_version_option_wins_over_the_wire`, `version_is_optional_and_falls_back_to_msh_12`, `seed_can_be_strict` | `src/message.rs` |

The table is **checked by `cargo test`**, not only by review:
`every_rule_has_a_coverage_row` reads this file and the rule index and
fails if a rule is declared without a row here, or covered here without
being declared. `every_spec_section_is_indexed_and_present` does the same
for the section directories and [`index.md`](../index.md).

## 7.2 Four layers

1. **Per-module unit tests** (`#[cfg(test)] mod tests` at the bottom of
   each `src/*.rs`) — the shape and edge cases of that one type in
   isolation: duplicate/missing/unknown keys, every enum variant, forced
   versions, the seed.
2. **Doctests** (the `Example:` section on every public type, per rule
   S12) — one realistic, runnable use of that item.
3. **Integration tests** (`tests/integration.rs`) — black-box, through the
   public API only, exercising real message shapes: `hl7-2`'s own
   `samples/*.hl7` files, read via `include_str!` from the sibling
   workspace member, plus the guarantees in
   [§4](../04-round-trip-guarantee/index.md) that only make sense end to
   end, plus the manifest and spec self-checks.
4. **Examples** (`examples/*.rs`) — not `#[test]`s, but built by
   `cargo clippy --all-targets` and run as part of manual verification;
   each is listed in `examples/README.md`.

## 7.3 What each layer is responsible for catching

| Failure mode | Caught by |
|--------------|-----------|
| Wrong shape for one type (§2 violated) | unit test for that module |
| Round trip loses the ER7 or the release (§4 violated) | integration test |
| Error message does not name the field (§5 violated) | unit test (`rejects_a_missing_...`) |
| A doc example no longer compiles or asserts stale output | doctest |
| A real sample message from `hl7-2` fails silently | integration test against `samples/` |
| A rule with no test, or a test for no rule | `every_rule_has_a_coverage_row` |

## 7.4 Rule S12: doc coverage is enforced, not aspirational

`src/lib.rs` carries `#![warn(missing_docs)]`, and
`cargo rustdoc -p serde-hl7-v2 --lib -- -W missing-docs` is one of the
four required checks. CI runs the same command per crate.

## 7.5 Golden-fixture style

Where a test needs a literal message, prefer the shortest text that
exercises the behaviour under test. Reach for `hl7-2`'s `samples/*.hl7`
specifically when the point of the test is "a real message, not one
written to order." Never a real patient record: the workspace's
`spec/phi/index.md` applies to test fixtures too.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
