<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-3');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'why-foundation', label: 'Why a foundation, not a full implementation' },
    { id: 'use', label: 'Use' },
    { id: 'levels', label: 'The three levels' },
    { id: 'rim', label: 'The RIM backbone' },
    { id: 'data-types', label: 'The other data types' },
    { id: 'derive', label: 'Struct mode' },
    { id: 'dependencies', label: 'Dependencies' },
    { id: 'not-yet', label: 'What is not here yet' },
    { id: 'related', label: 'Related crates' }
  ];

  const use = `use hl7_3::message;

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
assert_eq!(act.code.unwrap().display_name.as_deref(), Some("Height"));`;

  const levels = `Message                       level 1 — transport: sender, receiver, id
└── ControlAct                 level 2 — the real-world trigger event
    └── domain: xml::Element    level 3 — the interaction's own payload`;

  const rim = `use hl7_3::rim::Act;

let element = hl7_3::xml::parse(
    r#"<observation classCode="OBS" moodCode="EVN">
         <id root="2.16.840.1.113883.19.5" extension="1"/>
       </observation>"#,
)?;

let act = Act::from_element(&element);
assert_eq!(act.class_code, "OBS");
assert_eq!(act.mood_code, "EVN");`;

  const types = `use hl7_3::{Ed, Ivl, NullFlavor, Pq};

let range = hl7_3::xml::parse(
    r#"<effectiveTime><low value="20260101"/><high value="20261231"/></effectiveTime>"#,
)?;
let ivl = Ivl::from_element(&range);           // IVL: an interval
assert_eq!(ivl.low.as_deref(), Some("20260101"));

let dose = hl7_3::xml::parse(r#"<doseQuantity value="5" unit="mg"/>"#)?;
let pq = Pq::from_element(&dose);              // PQ: a quantity with a unit
assert_eq!(pq.unit.as_deref(), Some("mg"));

let note = hl7_3::xml::parse(r#"<text mediaType="text/plain">Reports pain.</text>"#)?;
let ed = Ed::from_element(&note);              // ED: encapsulated content
assert_eq!(ed.text.as_deref(), Some("Reports pain."));

// NullFlavor: why a value is explicitly absent, not just missing.
let value = hl7_3::xml::parse(r#"<value nullFlavor="ASKU"/>"#)?;
assert_eq!(NullFlavor::of(&value), Some(NullFlavor::AskedButUnknown));`;

  const derive = `use hl7_3::FromElement;

#[derive(FromElement, Default)]
struct Observation {
    #[element("classCode")]    class_code: String,
    #[element(child = "note")] note: Option<String>,
    #[element(raw)]            raw: hl7_3::xml::Element,
}

let observation = Observation::from_element(&hl7_3::xml::parse(xml)?);   // no Result`;

  const install = `cargo add hl7-3
cargo add hl7-3 --features derive   # adds #[derive(FromElement)]`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <CodeSample language="sh" code={install} />
  <Callout type="warning" heading="Read spec/index.md §1 before filing anything as a bug">
    <p>
      That section is titled “Scope — read this before filing anything as a bug”, and it means it.
      This crate implements the part of HL7 v3 that is the same in every domain. Anything beyond
      that — a specific interaction, a vocabulary domain, CDA's document model — is not here yet, by
      design rather than by oversight.
    </p>
  </Callout>

  <h2 id="why-foundation">Why a foundation, not a full implementation</h2>
  <p>
    HL7 v3 replaced v2's flexible, custom-delimited text with one strict, model-driven framework
    reused everywhere: the RIM, six backbone classes (<code>Act</code>, <code>Entity</code>,
    <code>Role</code>, <code>ActRelationship</code>, <code>Participation</code>,
    <code>RoleLink</code>) that every domain payload — lab results, care records, structured product
    labeling — is assembled from, serialized as XML instead of ER7.
  </p>
  <p>
    That rigor bought consistency at the cost of a steep learning curve, and v3 messaging itself saw
    limited adoption. What did succeed, and still runs today, is the Clinical Document Architecture
    and national registries such as NHS England's Personal Demographics Service — both built on the
    same RIM and three-level structure this crate reads.
  </p>
  <p>
    Full v3 fidelity is a large, multi-year undertaking. This crate is the part that is the same
    everywhere: the RIM types, and a reader for the envelope every interaction shares. Building out a
    specific interaction on top of it is next.
  </p>

  <h2 id="use">Use</h2>
  <CodeSample language="rust" code={use} />

  <h2 id="levels">The three levels</h2>
  <CodeSample language="text" code={levels} />
  <p>
    Nothing here fails when a wrapper is missing — an absent <code>id</code>, <code>sender</code>,
    or <code>controlActProcess</code> reads as <code>None</code>, the same lenient-by-default reading
    <a href="/crates/hl7-2/"><code>hl7-2</code></a>'s generic mode uses for v2 messages.
  </p>

  <h2 id="rim">The RIM backbone</h2>
  <CodeSample language="rust" code={rim} />
  <p>
    <code>Entity</code>, <code>Role</code>, <code>Participation</code>,
    <code>ActRelationship</code>, and <code>RoleLink</code> all work the same way — see
    <code>spec/index.md</code> §4 for exactly which attributes and children each reads.
  </p>

  <h2 id="data-types">The other data types: intervals, quantities, encapsulated data, null</h2>
  <p>
    Beyond <code>II</code> and <code>CD</code>, four more of HL7 v3's data types are modeled — kept
    as shallow as <code>CD</code> is (raw text, no parsing, no validation), but real:
  </p>
  <CodeSample language="rust" code={types} />
  <p>
    See <code>spec/index.md</code> §3 for exactly what each reads, and why
    <code>NullFlavor</code> is an open enum rather than a validated domain.
  </p>

  <h2 id="derive">Struct mode</h2>
  <CodeSample language="rust" code={derive} />
  <p>
    Behind the <code>derive</code> feature, from
    <a href="/crates/hl7-3-derive/"><code>hl7-3-derive</code></a>. Note there is no
    <code>Result</code>: a missing attribute or child reads as that field's <code>Default</code>,
    matching this crate's own degrade-don't-reject choice. Full reference:
    <a href="/guides/struct-mode/#v3">Struct mode and derive</a>.
  </p>

  <h2 id="dependencies">Dependencies</h2>
  <p>
    One: <a href="/crates/hl7-2-xml-lite-helper/"><code>hl7-2-xml-lite-helper</code></a>, the small
    dependency-free XML reader the <code>hl7-2</code>-family XML-facing crates also use. HL7 v3 is
    XML natively, unlike v2's pipe-delimited ER7, so this crate reads through the XML layer instead
    of <code>er7</code>.
  </p>

  <h2 id="not-yet">What is not here yet</h2>
  <ul>
    <li>
      <strong>Writing.</strong> This crate reads; there is no XML-writing capability, which is also
      why there is no <code>#[derive(ToElement)]</code>.
    </li>
    <li>
      <strong>Vocabulary domain validation.</strong> Explicitly marked as future work in
      <code>spec/index.md</code> §6.
    </li>
    <li>
      <strong>Any specific interaction.</strong> Level 3 is handed back as a raw element for you to
      decode with the RIM types.
    </li>
    <li>
      <strong>CDA's document model.</strong> CDA is built on the same RIM, so the pieces here apply
      — but the document model itself is not implemented.
    </li>
  </ul>

  <RelatedCrates slugs={crate.related} />
</DocPage>
