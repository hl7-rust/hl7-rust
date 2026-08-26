# Specification: HL7® v3 RIM backbone, coded values, and message envelope

This is the single source of truth for what this crate does and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7-3` (`hl7::v3` through the `hl7`
umbrella crate) as implemented. Every rule below is exercised by a unit
test, next to the code that implements it, in that module's
`#[cfg(test)]` block. A change to this document that isn't backed by a
test, or a code change that isn't reflected here, is a bug.

## 0. Relationship to the rest of the family

```
hl7-2-xml-lite-helper    the XML reader this crate reads through
  |
hl7-3                     this crate: RIM backbone classes, the data
                          types RIM attributes are built from, the
                          three-level message envelope
  |
hl7                       the umbrella crate — hl7::v3 re-exports this
```

Unlike the `hl7-2` family, HL7 v3 has no sibling transport or format
crates yet — no `hl7-3-mllp`, nothing analogous to
`hl7-2-from-er7-into-json`. HL7 v3 is XML natively, so there is no
encoding-layer crate underneath this one the way `er7` sits under
`hl7-2`; `hl7-2-xml-lite-helper` fills that role instead.

## 1. Scope — read this before filing anything as a bug

This crate is a **foundation**: the pieces every HL7 v3 interaction shares,
not a complete implementation of the standard. In scope:

- The six RIM backbone classes (§4): `Act`, `Entity`, `Role`,
  `Participation`, `ActRelationship`, `RoleLink`, as Rust structs with
  their most common attributes.
- Six data types (§3): `II` (Instance Identifier), `CD` (Concept
  Descriptor), `IVL` (Interval), `PQ` (Physical Quantity), `ED`
  (Encapsulated Data), and `NullFlavor`, the explicit-null mechanism any of
  them can carry instead of a value.
- The three-level message envelope (§5): transport wrapper, control act
  wrapper, domain payload — read generically, the same way `hl7-2`'s
  generic mode reads any v2 message without a dictionary.

Deliberately out of scope, for now:

- **The rest of HL7 v3's data type hierarchy.** `IVL`/`PQ` here are
  intentionally shallow — bounds and values are kept as raw text, not
  parsed into a structured time, interval arithmetic, or a checked numeric
  type. Generic `IVL<T>` (the real standard parameterizes the interval
  type over what it bounds), `RTO` (ratio), `BL`/`BN` (boolean), `SC`
  (string with a code), and the rest of the R2/ITS data types are not
  modeled at all.
- **Vocabulary domain validation.** A `CD.code` is read as whatever string
  the message used; this crate does not check it against `ActClass`,
  `ActMood`, `EntityClass`, `RoleClass`, `ParticipationType`,
  `ActRelationshipType`, or any of HL7 v3's dozens of other vocabulary
  domains. See §6.
- **Interaction-specific domain payload schemas.** `Message.control_act`'s
  `domain` field is a raw XML element; decoding it into a specific
  interaction's expected shape (a lab result, a care record) is the
  caller's job, using the RIM types this crate provides — this crate does
  not ship per-interaction schemas.
- **Clinical Document Architecture (CDA).** CDA reuses the RIM and a
  similar header/body split, but its document model (sections, entries,
  narrative blocks) is its own thing, not modeled here.
- **A `V3` equivalent of `hl7-2`'s CLI.** No binary ships with this crate.

## 2. The XML layer (the [`hl7-2-xml-lite-helper`] crate)

[`hl7-2-xml-lite-helper`]: https://crates.io/crates/hl7-2-xml-lite-helper

HL7 v3 is XML; this crate reads it through `hl7-2-xml-lite-helper`,
re-exported as [`xml`](../src/lib.rs) so callers can name `xml::Element`
without adding their own dependency. That crate's own rules apply
here — most importantly, namespace prefixes are matched by local name only
(`hl7:id` and `id` are the same element), and there is no schema
validation. See that crate's `spec/index.md` for the exact rules.

## 3. Data types (`src/vocabulary.rs`)

### 3.1 `II` — Instance Identifier

An `II` is a `root` (an OID or UUID naming an assigning authority or
identifier scheme) plus an optional `extension` (the identifier within
that scheme). Read from an element's `root`/`extension` attributes.
`Ii::from_element` returns `None` when the element has no `root` — an `II`
with no root names nothing, so there is nothing useful to construct.

### 3.2 `CD` — Concept Descriptor

A `CD` is a `code`, plus an optional `code_system` (the OID naming the
vocabulary domain `code` is drawn from) and an optional `display_name`
(human-readable text for `code`). Read from an element's
`code`/`codeSystem`/`displayName` attributes. `Cd::from_element` returns
`None` when the element has no `code`, for the same reason as `II`.

### 3.3 `IVL` — Interval

An `IVL` is either a single `value` (the element's own `value` attribute)
or a `low`/`high` pair (each read from a same-named child's `value`
attribute) — never both populated at once in a conforming message, though
`Ivl::from_element` does not enforce that; it reads whichever attributes
and children are present. `Ivl::default()` (all three `None`) is what a
missing or empty interval element reads as — this reader never fails.

### 3.4 `PQ` — Physical Quantity

A `PQ` is a `value` and a `unit`, both read straight from the element's
own attributes of the same names. Neither is parsed or validated — `value`
because HL7 v3 allows numeric forms this crate does not model (§1), `unit`
because UCUM validation is out of scope the same way vocabulary domains
are (§6).

### 3.5 `ED` — Encapsulated Data

An `ED` is a `media_type` and `representation` (both attributes) plus
`text` (the element's own text content, via `Element::text_opt`). Base64
or other encoded content is not decoded — `text` carries it exactly as
written, whatever `representation` says it is.

### 3.6 `NullFlavor` — the explicit-null mechanism

Any HL7 v3 attribute or element can carry `nullFlavor="..."` instead of a
value, stating *why* nothing is there rather than leaving a caller to
guess whether the sender simply omitted it. `NullFlavor::parse` recognizes
seven codes as named variants (`NI`, `UNK`, `ASKU`, `NASK`, `NAV`, `NA`,
`OTH`) and reads any other code into `NullFlavor::Unrecognized(String)` —
an open enum, not a validated one; see §6 for why this crate does not try
to enumerate HL7's real `NullFlavor` domain in full. `NullFlavor::as_code`
is the exact inverse of `NullFlavor::parse` for every input, including
`Unrecognized`. `NullFlavor::of(element)` reads the `nullFlavor` attribute
directly, independent of any other data type — call it on an element
whose `II`/`CD`/etc. came back with nothing to find out whether that was
because the sender said so.

## 4. The RIM backbone (`src/rim.rs`)

Six structs, each with a `from_element(&xml::Element) -> Self` reader that
never fails — a missing optional attribute or child reads as `None` or an
empty `Vec`, not an error, matching `hl7-2`'s "degrade, don't reject"
philosophy for generic reading.

| type | required attributes | optional children read |
|---|---|---|
| `Act` | `classCode`, `moodCode` | `id`* (0 or more), `code`, `statusCode`, `effectiveTime/@value`, `text` |
| `Entity` | `classCode` | `determinerCode` (attribute), `id`* , `code`, `name` |
| `Role` | `classCode` | `id`*, `code`, `statusCode`, `effectiveTime/@value` |
| `Participation` | `typeCode` | `time/@value`, `functionCode` |
| `ActRelationship` | `typeCode` | `inversionInd` (attribute, `"true"` → `Some(true)`) |
| `RoleLink` | `typeCode` | — |

\* every `id` child is read (`children_named`, not `child`) — a RIM
instance may carry more than one identifier from different assigning
authorities, and dropping all but the first would silently lose data a
caller may need to match against.

A required attribute (`classCode`, `moodCode`, `typeCode`) absent from the
element reads as an empty string, not an error — this crate does not
reject a nonconforming element outright; a caller checking conformance
reads an empty `class_code` as the signal.

## 5. The three-level message envelope (`src/message.rs`)

```
Message                          level 1: transport wrapper
├── id: II
├── creation_time: raw string
├── interaction_id: II            (root = catalog, extension = interaction)
├── sender / receiver: raw xml::Element
└── control_act: ControlAct       level 2 + 3
    ├── class_code, mood_code
    ├── code: CD                  the trigger event
    └── domain: raw xml::Element  level 3: the domain payload
```

### 5.1 Reading rule

[`message::parse`] takes the whole message's XML text and reads:

- `id` from the root element's `id` child (`II`).
- `creation_time` from the root's `creationTime` child's `value` attribute.
- `interaction_id` from the root's `interactionId` child — an `II`, not a
  `CD` (unlike most RIM-attribute codes, the interaction identifier is
  typed `II` in the standard: `root` names the interaction catalog, almost
  always `2.16.840.1.113883.1.6`, and `extension` names the specific
  interaction).
- `sender` / `receiver` from the root's `sender` / `receiver` child's
  **first child element** — the wrapper (`<sender typeCode="SND">...`)
  itself is not what a caller wants; what is inside it (typically a
  `device` or `telecom`-bearing element) is.
- `control_act` from the root's `controlActProcess` child:
  - `class_code` / `mood_code` from its `classCode` / `moodCode`
    attributes.
  - `code` from its `code` child (the trigger event).
  - `domain` from the **first child element of its `subject` child** — the
    common shape (`<subject><registrationEvent>...`), covering the
    single-subject case. A message with more than one `subject` (a batch
    of results in one control act) only yields the first; reading the rest
    needs the raw `xml::Element` tree via `hl7-2-xml-lite-helper::parse`
    directly, not this crate's `Message` shape.

### 5.2 Nothing here fails on a missing wrapper

Every field above is `Option`; a message missing `id`, `interactionId`,
`sender`, `receiver`, or `controlActProcess` entirely still parses, with
those fields `None`. The only error [`message::parse`] returns is
[`Error::Xml`], when the input is not well-formed XML at all. This mirrors
`hl7-2` generic mode: reading stays lenient, and a caller who needs to
know a required wrapper was present checks for `None` explicitly rather
than the library refusing the whole message.

## 6. Vocabulary domain validation — explicitly future work

`CD.code` is read verbatim. This crate does not ship the `ActClass`,
`ActMood`, `EntityClass`, `RoleClass`, `ParticipationType`,
`ActRelationshipType` (or any other) vocabulary domain's allowed value
list, and does not check a code against one. Adding that would mean
either bundling HL7's vocabulary tables (a maintenance burden this crate
does not yet take on) or making it pluggable (a caller supplies the table
for the domains they care about) — which approach, if either, is a design
decision for when a real caller needs it, not before.

`NullFlavor` (§3.6) is not an exception to this: it is not a validated
domain either. `NullFlavor::parse` never fails and never rejects a code —
the seven named variants are a convenience for the values common enough
to be worth matching on directly, and every other code, valid or not,
still round-trips through `Unrecognized(String)`. Nothing about
`NullFlavor` checks that a code is one HL7's real `NullFlavor` domain
actually defines.

## 7. Traceability

| rule | test |
|---|---|
| §3.1 `II` root/extension, and no-root case | `vocabulary::tests::ii_reads_root_and_extension`, `ii_extension_is_optional`, `ii_with_no_root_is_none` |
| §3.2 `CD` code/codeSystem/displayName, and no-code case | `vocabulary::tests::cd_reads_code_system_and_display_name`, `cd_with_no_code_is_none` |
| §3.3 `IVL` point value, low/high, and empty case | `vocabulary::tests::ivl_reads_a_point_value`, `ivl_reads_low_and_high`, `ivl_with_nothing_reads_all_none` |
| §3.4 `PQ` value and unit | `vocabulary::tests::pq_reads_value_and_unit` |
| §3.5 `ED` mediaType/representation/text, and text alone | `vocabulary::tests::ed_reads_media_type_representation_and_text`, `ed_with_no_attributes_still_reads_text` |
| §3.6 `NullFlavor` parse, `as_code` round trip, and reading the attribute | `vocabulary::tests::null_flavor_parses_the_seven_named_codes`, `null_flavor_parse_and_as_code_round_trip`, `null_flavor_of_reads_the_attribute` |
| §4 `Act` reads all attributes, and degrades when absent | `rim::tests::act_reads_class_mood_id_code_status_and_time`, `act_with_no_optional_children_still_reads` |
| §4 `Entity`, `Role`, `Participation`, `ActRelationship`, `RoleLink` | `rim::tests::entity_reads_class_determiner_and_name`, `role_reads_class_and_status`, `participation_reads_type_and_function`, `act_relationship_reads_type_and_inversion`, `role_link_reads_type` |
| §5.1 transport wrapper, control act wrapper, domain payload | `message::tests::reads_the_transport_wrapper`, `reads_the_control_act_wrapper_and_trigger_event`, `reads_the_domain_payload_as_a_raw_element` |
| §5.2 missing wrappers read as `None`, malformed XML is the only error | `message::tests::missing_wrappers_read_as_none_not_an_error`, `malformed_xml_is_an_error` |

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
