<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-derive');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'usage', label: 'Usage' },
    { id: 'attributes', label: 'The attributes' },
    { id: 'types', label: 'Field types' },
    { id: 'writing', label: 'Writing' },
    { id: 'related', label: 'Related crates' }
  ];

  const cargo = `hl7-2 = { version = "0.2", features = ["derive"] }`;

  const usage = `use hl7_2::{FromHl7, ToHl7, Raw};

#[derive(FromHl7, ToHl7)]
struct Admission {
    #[hl7("PID-1")]      sequence: u32,
    #[hl7("PID-3")]      identifiers: Vec<String>,
    #[hl7("PID-5.1.1")]  family_name: String,
    #[hl7("PID-8")]      sex: Option<String>,
    #[hl7(nested)]       visit: Visit,   // its own FromHl7 / ToHl7
    #[hl7(raw)]          raw: Raw,       // the whole message, kept alongside
}

let admission: Admission = hl7_2::parse(text)?.decode()?;
assert_eq!(admission.family_name, "EVERYWOMAN");

// The one vendor field no struct models — same object, no second parse.
assert_eq!(admission.raw.get("ZPD-1")?.as_deref(), Some("local"));`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <Callout heading="You do not depend on this crate directly">
    <p>
      <a href="/crates/hl7-2/"><code>hl7-2</code></a> re-exports both macros behind its
      <code>derive</code> feature.
    </p>
  </Callout>
  <CodeSample language="toml" code={cargo} />
  <p>
    Keeping the macros in a crate of their own is what lets the default build of
    <code>hl7-2</code> keep exactly one dependency: <code>syn</code> and <code>quote</code> are
    compiled only for callers who ask for the macros.
  </p>

  <h2 id="usage">Usage</h2>
  <CodeSample language="rust" code={usage} />

  <h2 id="attributes">The attributes</h2>
  <p>One attribute per field.</p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Attribute</th><th>On read</th><th>On write</th></tr>
      </thead>
      <tbody>
        <tr><td><code>#[hl7("PID-5.1")]</code></td><td>Read the path.</td><td>Write the path.</td></tr>
        <tr>
          <td><code>#[hl7(nested)]</code></td>
          <td>The field's own <code>FromHl7</code>.</td>
          <td>The field's own <code>ToHl7</code>.</td>
        </tr>
        <tr>
          <td><code>#[hl7(raw)]</code></td>
          <td>The whole message, as a <code>Raw</code>.</td>
          <td>Skipped.</td>
        </tr>
        <tr><td>none</td><td><code>Default::default()</code></td><td>Skipped.</td></tr>
      </tbody>
    </table>
  </div>

  <h2 id="types">Field types</h2>
  <p>
    Field types convert through <code>hl7_2::FromHl7Value</code> and
    <code>ToHl7Value</code>: <code>String</code>, <code>bool</code>, the integer and floating-point
    types, and <code>Option&lt;T&gt;</code> and <code>Vec&lt;T&gt;</code> of those —
    <code>Option</code> for a value that may be absent, <code>Vec</code> for one that repeats.
  </p>
  <p>
    A plain type is required, and a path that names nothing is <code>Error::MissingField</code>.
    Implement <code>hl7_2::FromHl7Text</code> for a domain type of your own and
    <code>Option</code> and <code>Vec</code> of it follow.
  </p>

  <h2 id="writing">Writing</h2>
  <p>
    Writing needs the segments to exist already. Build the message with
    <code>hl7_2::Builder</code> — whose <code>encode</code> method takes a <code>ToHl7</code> — or add
    them with <code>Message::append_segment</code>.
  </p>
  <p>
    This crate has no <code>spec/index.md</code> of its own: the normative description of struct mode
    is <a href="/spec/"><code>hl7-2</code>'s <code>spec/index.md</code> §6</a>. A fuller walkthrough
    is in <a href="/guides/struct-mode/">Struct mode and derive</a>.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
