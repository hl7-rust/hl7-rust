# HL7® v3

> HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
>
> This project uses the HL7® name in its package names, its organization
> name, and its domain, which is beyond fair use; we are requesting
> permission from HL7® for that.

The Reference Information Model (RIM) backbone classes, the data types
they're built from (identifiers, coded values, intervals, quantities,
encapsulated data, explicit null), and the three-level message envelope
for Health Level Seven (HL7) version 3 (V3) — a **foundation**, not a
complete implementation of the standard. See
[`spec/index.md`](spec/index.md) §1 for the exact, current scope.

## Why a foundation, not a full implementation

HL7 v3 replaced v2's flexible, custom-delimited text with one strict,
model-driven framework reused everywhere: the RIM, six backbone classes
(`Act`, `Entity`, `Role`, `ActRelationship`, `Participation`, `RoleLink`)
that every domain payload — lab results, care records, structured product
labeling — is assembled from, serialized as XML instead of ER7. That
rigor bought consistency at the cost of a steep learning curve, and V3
messaging itself saw limited adoption; what did succeed, and still runs
today, is the Clinical Document Architecture (CDA) and national registries
like NHS England's Personal Demographics Service, both built on the same
RIM and three-level structure this crate reads.

Full HL7 v3 fidelity — every vocabulary domain, every data type, CDA's own
document model — is a large, multi-year undertaking. This crate is the
part that is the same everywhere: the RIM types, and a reader for the
envelope every interaction shares. Building out a specific interaction (a
patient registration query, a lab result) on top of it is next.

## Use

```rust
use hl7_3::message;

let xml = r#"
<QUQI_IN000001UV01 xmlns="urn:hl7-org:v3">
  <id root="2.16.840.1.113883.19.5" extension="MSG00001"/>
  <creationTime value="20260101120000"/>
  <interactionId root="2.16.840.1.113883.1.6" extension="QUQI_IN000001UV01"/>
  <controlActProcess classCode="CACT" moodCode="EVN">
    <code code="QUQI_TE000001UV01"/>
    <subject>
      <observation classCode="OBS" moodCode="EVN">
        <id root="2.16.840.1.113883.19.5" extension="1"/>
        <code code="8302-2" codeSystem="2.16.840.1.113883.6.1" displayName="Height"/>
      </observation>
    </subject>
  </controlActProcess>
</QUQI_IN000001UV01>
"#;

let parsed = message::parse(xml)?;
assert_eq!(parsed.interaction_id.unwrap().extension.as_deref(), Some("QUQI_IN000001UV01"));

// Level 3, the domain payload, is a raw element — decode it with the RIM
// types yourself, matching what this interaction's schema says to expect.
let observation = parsed.control_act.unwrap().domain.unwrap();
let act = hl7_3::rim::Act::from_element(&observation);
assert_eq!(act.class_code, "OBS");
assert_eq!(act.code.unwrap().display_name.as_deref(), Some("Height"));
# Ok::<(), hl7_3::Error>(())
```

## The three levels

```
Message                    level 1 — transport: sender, receiver, id
└── ControlAct              level 2 — the real-world trigger event
    └── domain: xml::Element    level 3 — the interaction's own payload
```

Nothing here fails when a wrapper is missing — an absent `id`, `sender`,
or `controlActProcess` reads as `None`, the same lenient-by-default
reading `hl7-2`'s generic mode uses for v2 messages.

## The RIM backbone

```rust
use hl7_3::rim::Act;

let element = hl7_3::xml::parse(
    r#"<observation classCode="OBS" moodCode="EVN">
         <id root="2.16.840.1.113883.19.5" extension="1"/>
       </observation>"#,
)?;
let act = Act::from_element(&element);
assert_eq!(act.class_code, "OBS");
assert_eq!(act.mood_code, "EVN");
# Ok::<(), hl7_2_xml_lite_helper::Error>(())
```

`Entity`, `Role`, `Participation`, `ActRelationship`, and `RoleLink` all
work the same way — see [`spec/index.md`](spec/index.md) §4 for exactly
which attributes and children each reads.

## The other data types: intervals, quantities, encapsulated data, null

Beyond `II` and `CD`, four more of HL7 v3's data types are modeled — kept
as shallow as `CD` is (raw text, no parsing, no validation), but real:

```rust
use hl7_3::{Ed, Ivl, NullFlavor, Pq};

let range = hl7_3::xml::parse(
    r#"<effectiveTime><low value="20260101"/><high value="20261231"/></effectiveTime>"#,
)?;
let ivl = Ivl::from_element(&range); // IVL: an interval
assert_eq!(ivl.low.as_deref(), Some("20260101"));

let dose = hl7_3::xml::parse(r#"<doseQuantity value="5" unit="mg"/>"#)?;
let pq = Pq::from_element(&dose); // PQ: a quantity with a unit
assert_eq!(pq.unit.as_deref(), Some("mg"));

let note = hl7_3::xml::parse(r#"<text mediaType="text/plain">Reports pain.</text>"#)?;
let ed = Ed::from_element(&note); // ED: encapsulated content
assert_eq!(ed.text.as_deref(), Some("Reports pain."));

// NullFlavor: why a value is explicitly absent, not just missing.
let value = hl7_3::xml::parse(r#"<value nullFlavor="ASKU"/>"#)?;
assert_eq!(NullFlavor::of(&value), Some(NullFlavor::AskedButUnknown));
# Ok::<(), hl7_2_xml_lite_helper::Error>(())
```

See [`spec/index.md`](spec/index.md) §3 for exactly what each reads and
why `NullFlavor` is an open enum rather than a validated domain.

## Dependencies

One: [`hl7-2-xml-lite-helper`](https://crates.io/crates/hl7-2-xml-lite-helper),
the small dependency-free XML reader the `hl7-2`-family XML-facing crates
also use — HL7 v3 is XML natively, unlike v2's pipe-delimited ER7, so this
crate reads through the XML layer instead of `er7`.

## See also

- [`spec/index.md`](spec/index.md) — the normative specification
- [`hl7`](https://crates.io/crates/hl7) — the umbrella crate; this crate is
  `hl7::v3`
- [`hl7-2`](https://crates.io/crates/hl7-2) — HL7 v2, this crate's sibling
  standard
- [`hl7-2-xml-lite-helper`](https://crates.io/crates/hl7-2-xml-lite-helper) —
  the XML reader this crate is built on

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
