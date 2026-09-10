<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('serde-hl7');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'features', label: 'Cargo features' },
    { id: 'use', label: 'Use' },
    { id: 'modules', label: 'The modules' },
    { id: 'shared-rules', label: 'What both modules share' },
    { id: 'related', label: 'Related crates' }
  ];

  const source = `// serde-hl7/src/lib.rs, in full, minus the documentation
#[cfg(feature = "v2")]
pub use serde_hl7_v2 as v2;

#[cfg(feature = "v3")]
pub use serde_hl7_v3 as v3;`;

  const use = `use serde_hl7::v2;

let text = "MSH|^~\\\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\\r\\
            PID|1||241900||SMITH^JOHN";
let message = v2::Message::parse(text)?;

let json = serde_json::to_string(&message)?;
let back: v2::Message = serde_json::from_str(&json)?;
assert_eq!(back.to_er7(), text);`;

  const install = `cargo add serde-hl7                                          # v2 and v3
cargo add serde-hl7 --no-default-features --features v2      # v2 only`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <CodeSample language="rust" code={source} />
  <p>
    That is the entire crate. The <code>hl7</code> crate organizes HL7 by standard because a
    “message” or a “code” in one standard is a different thing in another; this crate is the same
    shape one layer up, so a caller can depend on “Serde for HL7” and reach the standard they need
    by module.
  </p>

  <h2 id="features">Cargo features</h2>
  <CodeSample language="sh" code={install} />
  <p>
    <code>v2</code> and <code>v3</code> are both on by default, mirroring <code>hl7</code>, which
    always carries both modules. Turn one off to skip the other's dependency tree entirely; a
    build with neither is an empty crate, which is allowed and useless.
  </p>

  <h2 id="use">Use</h2>
  <CodeSample language="rust" code={use} />
  <p>
    Any Serde format works the same way — <code>serde_json</code> is used here only because JSON is
    the easiest format to read on a page. Neither this crate nor the modules beneath it name a
    format in their runtime dependencies.
  </p>

  <h2 id="modules">The modules</h2>
  <p>
    <a href="/crates/serde-hl7-v2/"><code>serde_hl7::v2</code></a> wraps <code>hl7-2</code>: the
    message (as its ER7 text plus the release it was read as), the dictionary-named tree,
    validation findings, and the release. <a href="/crates/serde-hl7-v3/"><code>serde_hl7::v3</code></a>
    wraps <code>hl7-3</code>: the three-level envelope, the six RIM backbone classes, the data
    types, and the raw XML element tree. Each is its own crate with its own normative spec.
  </p>

  <h2 id="shared-rules">What both modules share</h2>
  <p>
    Exactly two runtime dependencies each, <code>serde</code> and the <code>hl7-*</code> crate
    wrapped. Every <code>Serialize</code>/<code>Deserialize</code> impl hand-written against Serde's
    low-level traits, no derive, no proc-macro crate. A same-named wrapper per type that
    <code>Deref</code>s to the type it wraps, so the wrapped crate's whole API is reachable on the
    wrapper. Unknown keys ignored by default, and an opt-in <code>Strict&lt;T&gt;</code> that
    rejects them — the pattern <code>serde-er7</code> established over <code>er7</code>.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
