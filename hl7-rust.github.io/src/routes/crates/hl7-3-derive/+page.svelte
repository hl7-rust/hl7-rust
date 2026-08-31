<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-3-derive');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'usage', label: 'Usage' },
    { id: 'attributes', label: 'The attributes' },
    { id: 'types', label: 'Field types' },
    { id: 'differences', label: 'Two deliberate differences from the v2 macro' },
    { id: 'related', label: 'Related crates' }
  ];

  const cargo = `hl7-3 = { version = "0.2", features = ["derive"] }`;

  const usage = `use hl7_3::FromElement;
use hl7_3::rim::Act;

#[derive(FromElement, Default)]
struct Observation {
    #[element("classCode")]          class_code: String,
    #[element("moodCode")]           mood_code: String,
    #[element(child = "note")]       note: Option<String>,
    #[element(nested = "component")] component: Act,        // its own FromElement
    #[element(raw)]                  raw: hl7_3::xml::Element, // the escape hatch
}

let element = hl7_3::xml::parse(xml)?;
let observation = Observation::from_element(&element);
assert_eq!(observation.class_code, "OBS");

// The one attribute no struct field models — same object, no second parse.
assert_eq!(observation.raw.attribute("negationInd"), Some("true"));`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <Callout heading="You do not depend on this crate directly">
    <p>
      <a href="/crates/hl7-3/"><code>hl7-3</code></a> re-exports the macro behind its
      <code>derive</code> feature. Note that the umbrella <a href="/crates/hl7/"><code>hl7</code></a>
      crate's own <code>derive</code> feature forwards to <code>hl7-2</code>'s only, so this macro is
      reached through <code>hl7-3</code> directly.
    </p>
  </Callout>
  <CodeSample language="toml" code={cargo} />
  <p>
    Keeping the macro in a crate of its own is what lets the default build of
    <code>hl7-3</code> keep exactly one dependency: <code>syn</code> and <code>quote</code> are
    compiled only for callers who ask for it.
  </p>

  <h2 id="usage">Usage</h2>
  <CodeSample language="rust" code={usage} />

  <h2 id="attributes">The attributes</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Attribute</th><th>Reads</th></tr>
      </thead>
      <tbody>
        <tr><td><code>#[element("classCode")]</code></td><td>The <code>classCode</code> attribute.</td></tr>
        <tr><td><code>#[element(child = "note")]</code></td><td>The <code>note</code> child's text.</td></tr>
        <tr>
          <td><code>#[element(nested = "component")]</code></td>
          <td>The <code>component</code> child, via the field type's own <code>FromElement</code>.</td>
        </tr>
        <tr>
          <td><code>#[element(raw)]</code></td>
          <td>The whole element, as <code>hl7_3::xml::Element</code>.</td>
        </tr>
        <tr><td>none</td><td><code>Default::default()</code></td></tr>
      </tbody>
    </table>
  </div>

  <h2 id="types">Field types</h2>
  <p>
    Field types convert through <code>hl7_3::typed::FromElementValue</code>:
    <code>String</code>, <code>bool</code>, and the integer and floating-point types, plus
    <code>Option&lt;T&gt;</code> of any of those.
  </p>
  <p>
    There is no <code>Vec&lt;T&gt;</code> support yet — a repeating child needs
    <code>element.children_named(...)</code> by hand for now.
  </p>

  <h2 id="differences">Two deliberate differences from the v2 macro</h2>
  <h3>No Result anywhere</h3>
  <p>
    Unlike <a href="/crates/hl7-2-derive/"><code>hl7-2-derive</code></a>'s
    <code>#[derive(FromHl7)]</code>, a missing attribute or child is not an error — it reads as that
    field's <code>Default</code>, the same degrade-don't-reject choice
    <a href="/crates/hl7-3/"><code>hl7-3</code></a>'s own <code>rim</code> types make. See that
    crate's <code>typed</code> module documentation for why.
  </p>

  <h3>No #[derive(ToElement)]</h3>
  <p>
    <code>hl7-3</code> has no XML-writing capability yet, so a write-direction macro would have
    nothing real to generate.
  </p>
  <p>
    This crate has no <code>spec/index.md</code> of its own; its behavior is documented in its README
    and in <code>hl7-3</code>'s <code>typed</code> module. A fuller walkthrough is in
    <a href="/guides/struct-mode/#v3">Struct mode and derive</a>.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
