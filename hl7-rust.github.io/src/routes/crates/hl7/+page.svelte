<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'features', label: 'Cargo features' },
    { id: 'use', label: 'Use' },
    { id: 'modules', label: 'The modules' },
    { id: 'why-empty-root', label: 'Why the root is empty' },
    { id: 'when-not', label: 'When not to use it' },
    { id: 'related', label: 'Related crates' }
  ];

  const source = `// hl7/src/lib.rs, in full, minus the documentation
pub use hl7_2 as v2;
pub use hl7_3 as v3;`;

  const use = `use hl7::v2;

let message = v2::parse("MSH|^~\\\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\\r\\
                         PID|1||241900||SMITH^JOHN")?;

assert_eq!(message.structure_id(), "ORU_R01");
assert_eq!(message.get("PID-5.1")?.as_deref(), Some("SMITH"));`;

  const both = `use hl7::{v2, v3};

// v2: delimited text, addressed by path.
let admission = v2::parse(er7_text)?;
let patient_id = admission.get("PID-3.1")?;

// v3: XML, walked by element name.
let interaction = v3::message::parse(xml_text)?;
let control_act = interaction.control_act;`;

  const install = `cargo add hl7
cargo add hl7 --features derive   # pulls in hl7-2's derive macros`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <CodeSample language="rust" code={source} />
  <p>
    That is the entire crate. It exists so that a caller can depend on “HL7 for Rust” rather than on
    a specific standard, and so that <code>hl7::fhir</code> has somewhere to land when that standard
    is implemented.
  </p>

  <h2 id="features">Cargo features</h2>
  <CodeSample language="sh" code={install} />
  <Callout heading="derive forwards to hl7-2 only">
    <p>
      This crate's <code>derive</code> feature enables <code>hl7-2</code>'s, giving you
      <code>#[derive(FromHl7)]</code> and <code>#[derive(ToHl7)]</code>. It does
      <em>not</em> enable <code>hl7-3</code>'s <code>#[derive(FromElement)]</code> — for that,
      depend on <a href="/crates/hl7-3/"><code>hl7-3</code></a> directly with its own
      <code>derive</code> feature.
    </p>
  </Callout>

  <h2 id="use">Use</h2>
  <CodeSample language="rust" code={use} />
  <p>And when a system speaks both standards:</p>
  <CodeSample language="rust" code={both} />

  <h2 id="modules">The modules</h2>
  <dl>
    <dt><code>hl7::v2</code> — the <a href="/crates/hl7-2/"><code>hl7-2</code></a> crate</dt>
    <dd>
      HL7 v2, releases 2.1 through 2.9. The format most healthcare data still moves in. Parse,
      navigate, validate, modify, and render, in three modes.
    </dd>
    <dt><code>hl7::v3</code> — the <a href="/crates/hl7-3/"><code>hl7-3</code></a> crate</dt>
    <dd>
      HL7 v3: the Reference Information Model, and the three-level message envelope built from it. A
      foundation, not a full implementation — read that crate's own scope section first.
    </dd>
    <dt><code>hl7::fhir</code> — not yet</dt>
    <dd>Deliberately unclaimed, so the name is free when that standard is implemented.</dd>
  </dl>

  <h2 id="why-empty-root">Why the root is empty</h2>
  <p>
    HL7 is not one standard but a family of them, and they have little in common beyond the name and
    the problem: v2 is delimited text, v3 is XML, FHIR is resources over HTTP.
  </p>
  <p>
    A “message”, a “segment”, and a “code” all mean something different in each. Flattening them
    into one namespace would only invite mixing them up — in a domain where mixing them up means a
    clinical record. So nothing lives at the root, and the module you want is the version your
    senders speak.
  </p>

  <h2 id="when-not">When not to use it</h2>
  <p>
    Depend on <a href="/crates/hl7-2/"><code>hl7-2</code></a> or
    <a href="/crates/hl7-3/"><code>hl7-3</code></a> directly if you specifically want one standard
    with no umbrella indirection. There is no functional difference — this crate is a re-export —
    but a direct dependency states your intent in the manifest and keeps one crate out of your
    dependency tree.
  </p>
  <p>
    This crate has no <code>spec/index.md</code> of its own: a thin re-export has nothing normative
    to state. The specifications that govern it are
    <a href="/spec/"><code>hl7-2</code>'s and <code>hl7-3</code>'s</a>.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
