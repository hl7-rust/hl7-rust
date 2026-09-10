[index](../index.md) → §7 Testing strategy

# §7 Testing strategy

## 7.1 Rule coverage

Every rule in [the index](../index.md#rule-index) names the test that
enforces it. A rule with no test is a bug in this table.

| Rule | Test | File |
| ---- | ---- | ---- |
| S1 | `the_crate_has_exactly_two_runtime_dependencies` | `tests/integration.rs` |
| S2 | `no_format_crate_is_a_runtime_dependency` | `tests/integration.rs` |
| S3 | `keys_are_the_camel_case_field_names`, `act_serializes_every_key_always`, `serializes_four_keys_in_order` | `src/message.rs`, `src/rim.rs`, `src/element.rs` |
| S4 | `an_empty_message_is_all_nulls_and_reads_back`, `ivl_pq_and_ed_round_trip_and_default_to_empty`, `only_name_is_required` | `src/message.rs`, `src/vocabulary.rs`, `src/element.rs` |
| S5 | `required_codes_are_required`, `ii_round_trips_and_requires_root`, `control_act_requires_its_codes` | `src/rim.rs`, `src/vocabulary.rs`, `src/message.rs` |
| S6 | `round_trips_every_named_code`, `an_unrecognized_code_is_carried_not_rejected` | `src/null_flavor.rs` |
| S7 | `round_trips_a_tree`, `serializes_four_keys_in_order` | `src/element.rs` |
| S8 | `ignores_unknown_fields`, `ignores_unknown_fields_but_strict_does_not` | `src/element.rs`, `src/vocabulary.rs` |
| S9 | `only_name_is_required`, `required_codes_are_required`, `error_messages_still_name_the_missing_field` | `src/element.rs`, `src/rim.rs`, `tests/integration.rs` |
| S10 | `the_sample_message_serializes_to_the_documented_shape` and every shape test above | `tests/integration.rs` |
| S11 | `deref_reaches_the_inner_api`, `deref_and_from_work_both_ways`, `display_is_the_code` | `src/element.rs`, `src/message.rs`, `src/null_flavor.rs` |
| S12 | *by `cargo rustdoc --lib -- -W missing-docs`*, one of the four checks (§7.4) | — |
| S13 | `strict_reaches_every_nested_level`, `strict_rejects_an_unknown_field_at_any_depth`, `strict_reaches_into_ids_and_codes`, `strict_accepts_a_real_message_with_no_typos` | `src/message.rs`, `src/element.rs`, `src/rim.rs` |
| S14 | `round_trips_a_full_message`, `every_class_round_trips_from_default`, `every_type_round_trips_through_pretty_json` | `src/message.rs`, `src/rim.rs`, `tests/integration.rs` |
| S15 | `every_object_wrapper_is_written_by_the_macro` | `tests/integration.rs` |

The table is **checked by `cargo test`**, not only by review:
`every_rule_has_a_coverage_row` reads this file and the rule index and
fails if a rule is declared without a row here, or covered here without
being declared. `every_spec_section_is_indexed_and_present` does the same
for the section directories and [`index.md`](../index.md).

## 7.2 Four layers

1. **Per-module unit tests** — the shape and edge cases of that module's
   types in isolation: duplicate/missing/unknown keys, defaults, nested
   strictness.
2. **Doctests** — one realistic, runnable use of every public type, per
   rule S12.
3. **Integration tests** (`tests/integration.rs`) — black-box, through the
   public API only: a realistic interaction parsed from XML through every
   type in the crate, the guarantees in
   [§4](../04-round-trip-guarantee/index.md), the manifest checks, and the
   spec self-checks.
4. **Examples** (`examples/*.rs`) — built by `cargo clippy --all-targets`
   and run as part of manual verification; listed in `examples/README.md`.

## 7.3 What each layer is responsible for catching

| Failure mode | Caught by |
|--------------|-----------|
| Wrong shape for one type (§2 violated) | unit test for that module |
| Round trip loses a field (§4 violated) | integration test |
| Error message does not name the field (§5 violated) | unit test |
| A doc example no longer compiles or asserts stale output | doctest |
| Strictness stops at some nesting depth (§11 violated) | `strict_reaches_every_nested_level` |
| A rule with no test, or a test for no rule | `every_rule_has_a_coverage_row` |

## 7.4 Rule S12: doc coverage is enforced, not aspirational

`src/lib.rs` carries `#![warn(missing_docs)]`, and
`cargo rustdoc -p serde-hl7-v3 --lib -- -W missing-docs` is one of the
four required checks. CI runs the same command per crate.

## 7.5 Golden-fixture style

Where a test needs a literal document, prefer the shortest XML that
exercises the behaviour under test. `hl7-3` ships no sample files;
`tests/integration.rs` carries one realistic interaction of its own. Never
a real patient record: the workspace's `spec/phi/index.md` applies to test
fixtures too.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
