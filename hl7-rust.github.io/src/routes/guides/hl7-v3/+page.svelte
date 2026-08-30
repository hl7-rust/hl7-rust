<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'scope', label: 'Read the scope first' },
    { id: 'why-foundation', label: 'Why a foundation, not a full implementation' },
    { id: 'envelope', label: 'The three-level envelope' },
    { id: 'rim', label: 'The RIM backbone' },
    { id: 'data-types', label: 'The data types' },
    { id: 'null-flavor', label: 'NullFlavor: why a value is absent' },
    { id: 'derive', label: 'Struct mode' },
    { id: 'transport', label: 'Transport' },
    { id: 'v2-differences', label: 'What is different from v2, in practice' }
  ];

  const envelope = `use hl7_3::message;

let parsed = message::parse(xml)?;
assert_eq!(
    parsed.interaction_id.unwrap().extension.as_deref(),
    Some("QUQI_IN000001UV01"),
);

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

// IVL: an interval.
let range = hl7_3::xml::parse(
    r#"<effectiveTime><low value="20260101"/><high value="20261231"/></effectiveTime>"#,
)?;
assert_eq!(Ivl::from_element(&range).low.as_deref(), Some("20260101"));

// PQ: a quantity with a unit.
let dose = hl7_3::xml::parse(r#"<doseQuantity value="5" unit="mg"/>"#)?;
assert_eq!(Pq::from_element(&dose).unit.as_deref(), Some("mg"));

// ED: encapsulated content.
let note = hl7_3::xml::parse(r#"<text mediaType="text/plain">Reports pain.</text>"#)?;
assert_eq!(Ed::from_element(&note).text.as_deref(), Some("Reports pain."));`;

  const nullFlavor = `// NullFlavor: why a value is explicitly absent, not just missing.
let value = hl7_3::xml::parse(r#"<value nullFlavor="ASKU"/>"#)?;
assert_eq!(NullFlavor::of(&value), Some(NullFlavor::AskedButUnknown));`;

  const derive = `use hl7_3::FromElement;

#[derive(FromElement, Default)]
struct Observation {
    #[element("classCode")]          class_code: String,
    #[element("moodCode")]           mood_code: String,
    #[element(child = "note")]       note: Option<String>,
    #[element(raw)]                  raw: hl7_3::xml::Element,
}

let observation = Observation::from_element(&hl7_3::xml::parse(xml)?);
assert_eq!(observation.class_code, "OBS");

// The one attribute no struct field models — same object, no second parse.
assert_eq!(observation.raw.attribute("negationInd"), Some("true"));`;

  const cargo = `hl7-3 = { version = "0.2", features = ["derive"] }`;
</script>

<DocPage
  title="HL7 v3"
  lede="The Reference Information Model backbone, the data types it is built from, and the three-level message envelope every v3 interaction shares — the part of v3 that is the same everywhere."
  {contents}
>
  <h2 id="scope">Read the scope first</h2>
  <Callout type="warning" heading="hl7-3 is a foundation, not a complete implementation">
    <p>
      Its <code>spec/index.md</code> §1 is titled “Scope — read this before filing anything as a
      bug”, and that is not a joke. Full HL7 v3 fidelity means every vocabulary domain, every data
      type, and CDA's own document model: a large, multi-year undertaking. What this crate
      implements is the RIM types and a reader for the envelope every interaction shares. Building
      out a specific interaction on top of that is next.
    </p>
  </Callout>

  <h2 id="why-foundation">Why a foundation, not a full implementation</h2>
  <p>
    HL7 v3 replaced v2's flexible, custom-delimited text with one strict, model-driven framework
    reused everywhere: the Reference Information Model, six backbone classes
    (<code>Act</code>, <code>Entity</code>, <code>Role</code>, <code>ActRelationship</code>,
    <code>Participation</code>, <code>RoleLink</code>) that every domain payload — lab results, care
    records, structured product labeling — is assembled from, serialized as XML instead of ER7.
  </p>
  <p>
    That rigor bought consistency at the cost of a steep learning curve, and v3 messaging itself saw
    limited adoption. What did succeed, and still runs today, is the Clinical Document Architecture
    and national registries such as NHS England's Personal Demographics Service — both built on the
    same RIM and three-level structure this crate reads.
  </p>
  <p>
    So the part that is worth implementing first is the part that is the same everywhere. That is
    what is here.
  </p>

  <h2 id="envelope">The three-level envelope</h2>
  <CodeSample language="text" code={levels} label="Every v3 interaction has this shape" />
  <CodeSample language="rust" code={envelope} />
  <p>
    Nothing here fails when a wrapper is missing — an absent <code>id</code>, <code>sender</code>,
    or <code>controlActProcess</code> reads as <code>None</code>, the same lenient-by-default
    reading <a href="/guides/parsing/#generic"><code>hl7-2</code>'s generic mode</a> uses for v2
    messages.
  </p>
  <p>
    Level 3 is deliberately left as a raw element. The domain payload is whatever that interaction's
    schema says, and pretending otherwise would mean claiming coverage the crate does not have.
    Decode it with the RIM types yourself.
  </p>

  <h2 id="rim">The RIM backbone</h2>
  <CodeSample language="rust" code={rim} />
  <p>
    <code>Entity</code>, <code>Role</code>, <code>Participation</code>,
    <code>ActRelationship</code>, and <code>RoleLink</code> all work the same way. The crate's
    <code>spec/index.md</code> §4 states exactly which attributes and children each one reads —
    worth checking before assuming a field is there.
  </p>

  <h2 id="data-types">The data types</h2>
  <p>
    Beyond <code>II</code> (an instance identifier) and <code>CD</code> (a coded value), four more
    of HL7 v3's data types are modeled — kept as shallow as <code>CD</code> is, with raw text, no
    parsing, and no validation, but real:
  </p>
  <CodeSample language="rust" code={types} />
  <p>
    Shallow is the deliberate choice. Parsing an <code>IVL</code>'s bounds into dates would mean
    committing to one timestamp interpretation, and v3 timestamps carry precision and timezone
    conventions that vary by deployment. Text preserves what arrived.
  </p>

  <h2 id="null-flavor">NullFlavor: why a value is absent</h2>
  <CodeSample language="rust" code={nullFlavor} />
  <p>
    v3 goes further than v2's <a href="/docs/concepts/#null">explicit null</a>: it says
    <em>why</em> a value is absent. <code>ASKU</code> — asked but unknown — is a different clinical
    fact from <code>NAV</code>, temporarily unavailable, or <code>MSK</code>, masked for
    confidentiality.
  </p>
  <p>
    <code>NullFlavor</code> is an <em>open</em> enum rather than a validated domain, so a flavour
    this crate has never seen still reads rather than failing. The crate's
    <code>spec/index.md</code> §3 explains why, and §6 marks vocabulary-domain validation explicitly
    as future work.
  </p>

  <h2 id="derive">Struct mode</h2>
  <CodeSample language="toml" code={cargo} />
  <CodeSample language="rust" code={derive} />
  <p>
    Note there is no <code>Result</code> anywhere: a missing attribute or child reads as that
    field's <code>Default</code>. That matches the degrade-don't-reject choice the
    <code>rim</code> types make, and it is the main way this macro differs from its v2 cousin.
    There is also no <code>Vec&lt;T&gt;</code> support yet — a repeating child needs
    <code>element.children_named(...)</code> by hand — and no
    <code>#[derive(ToElement)]</code>, because <code>hl7-3</code> has no XML-writing capability for
    one to generate against.
  </p>
  <p>Full attribute reference: <a href="/guides/struct-mode/#v3">Struct mode and derive</a>.</p>

  <h2 id="transport">Transport</h2>
  <p>
    <a href="/crates/hl7-3-soap/"><code>hl7-3-soap</code></a> carries v3 over HTTP, which is v3's own
    historically dominant transport rather than an alternative to something else. It does not depend
    on <code>hl7-3</code>: a SOAP envelope is XML, and routing one requires no RIM knowledge. See
    <a href="/guides/soap/">SOAP over HTTP</a>.
  </p>

  <h2 id="v2-differences">What is different from v2, in practice</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th></th><th>HL7 v2</th><th>HL7 v3</th></tr>
      </thead>
      <tbody>
        <tr><td>Encoding</td><td>Delimited text (ER7), or v2.xml</td><td>XML, natively</td></tr>
        <tr><td>Underlying layer</td><td><code>er7</code></td><td><code>hl7-2-xml-lite-helper</code></td></tr>
        <tr>
          <td>Addressing a value</td>
          <td>A path — <code>PID-5.1</code></td>
          <td>Element and attribute names, walked</td>
        </tr>
        <tr><td>Versioning</td><td>By release, 2.1–2.9</td><td>By interaction</td></tr>
        <tr><td>Usual transport</td><td>MLLP over TCP</td><td>SOAP over HTTP</td></tr>
        <tr><td>Absence</td><td>Empty, or the explicit null</td><td>Absent, or a <code>NullFlavor</code></td></tr>
        <tr><td>Writing</td><td>Full: set, build, render</td><td>Not yet — reading only</td></tr>
      </tbody>
    </table>
  </div>
  <p>
    The last row is the one to plan around. <code>hl7-3</code> reads; it does not yet write. If you
    need to emit v3 XML today, you will be building the document yourself.
  </p>
</DocPage>
