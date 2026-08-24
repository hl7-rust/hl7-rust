<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-from-xsd-into-json-dictionary');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'why', label: 'Why you would want one' },
    { id: 'input', label: 'What a schema directory looks like' },
    { id: 'use', label: 'Use' },
    { id: 'cannot-say', label: 'Two things the schemas cannot say' },
    { id: 'reading', label: 'Reading the result' },
    { id: 'library', label: 'As a library' },
    { id: 'dependencies', label: 'Dependencies' },
    { id: 'related', label: 'Related crates' }
  ];

  const layout = `schemas/paris/
  2_5_1_types.xsd      composite data types and their components
  2_5_1_fields.xsd     every SEG.n element and the data type it carries
  2_5_1_segments.xsd   each segment's field list, with cardinality
  ADT_A05.xsd          one abstract message structure each
  ADT_A39.xsd`;

  const simple = `hl7-2-from-xsd-into-json-dictionary schemas/paris -o paris.json`;

  const full = `hl7-2-from-xsd-into-json-dictionary schemas/paris \\
    --name paris \\
    --alias ADT_A28=ADT_A05 \\
    --alias ADT_A31=ADT_A05 \\
    --inherits 2.5 \\
    -o paris.json`;

  const reading = `let text = std::fs::read_to_string("paris.json")?;
let dictionary = hl7_2::Dictionary::from_json(&text, "paris")?;

assert_eq!(dictionary.field_type("PID", 3), Some("CX"));
assert!(dictionary.field_cardinality("PID", 3).repeats);`;

  const converter = `hl7-2-from-er7-into-xml --dictionary paris.json --schema-shape message.hl7`;

  const library = `use hl7_2_from_xsd_into_json_dictionary::{Options, convert_directory};

let document = convert_directory("schemas/paris".as_ref(), &Options::default())?;
std::fs::write("paris.json", document.to_json())?;`;

  const install = `cargo install hl7-2-from-xsd-into-json-dictionary`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    HL7 publishes v2.xml as schemas. <a href="/crates/hl7-2/"><code>hl7-2</code></a> reads a
    dictionary, because one dictionary format can serve every release and every local dialect from a
    single build. This crate is the bridge: it turns the schemas a site already has into the
    dictionary the crates already read.
  </p>
  <CodeSample language="sh" code={install} />

  <h2 id="why">Why you would want one</h2>
  <p>
    The dictionary it writes carries <strong>cardinality</strong> as well as data types — each
    field's <code>minOccurs</code> and <code>maxOccurs</code> become
    <code>required</code> and <code>repeats</code>.
  </p>
  <Callout heading="Cardinality is what makes schema mode possible">
    <p>
      It is what lets
      <a href="/crates/hl7-2-from-er7-into-xml/"><code>hl7-2-from-er7-into-xml</code></a> run in
      schema mode and emit a document that validates against the very schemas the dictionary was
      built from: required fields present even when empty, every declared component written, and no
      field split into more elements than the schema allows.
    </p>
  </Callout>

  <h2 id="input">What a schema directory looks like</h2>
  <p>
    Three base files that every message shares, plus one file per message structure:
  </p>
  <CodeSample language="text" code={layout} />
  <p>
    The <code>2_5_1</code> prefix is discovered from a structure file's
    <code>&lt;xsd:include&gt;</code>, so the directory can be named for the sending system rather than
    for the HL7 release.
  </p>

  <h2 id="use">Use</h2>
  <CodeSample language="sh" code={simple} />

  <h2 id="cannot-say">Two things the schemas cannot say</h2>
  <CodeSample language="sh" code={full} />
  <dl>
    <dt><code>--alias CODE_TRIGGER=STRUCTURE</code></dt>
    <dd>
      Which trigger events arrive carried by another message's structure. A directory holds
      <code>ADT_A05.xsd</code> but never says that an <code>ADT^A28</code> is one, so that mapping has
      to come from you.
    </dd>
    <dt><code>--inherits 2.5</code></dt>
    <dd>
      Layers the generated document over a bundled release instead of leaving it to stand alone.
      Without it, everything the schemas do not describe reads positionally.
    </dd>
    <dt><code>--name</code></dt>
    <dd>What the dictionary calls itself, which is what appears in diagnostics.</dd>
  </dl>

  <h2 id="reading">Reading the result</h2>
  <CodeSample language="rust" code={reading} />
  <p>Or hand it straight to a converter:</p>
  <CodeSample language="sh" code={converter} />

  <h2 id="library">As a library</h2>
  <CodeSample language="rust" code={library} />

  <h2 id="dependencies">Dependencies</h2>
  <p>
    One: <a href="/crates/hl7-2-xml-lite-helper/"><code>hl7-2-xml-lite-helper</code></a>, the small
    dependency-free XML reader shared with <code>hl7-2-soap</code> and
    <code>hl7-2-from-xml-into-er7</code>, re-exported here as <code>xml</code> so a caller can name
    <code>xml::Element</code> without adding its own dependency on it.
  </p>
  <p>
    Reading XSD needs an XML reader; writing a dictionary needs a JSON writer, which is small and
    specific enough to the one shape involved to live here rather than pull in a general-purpose one.
    <code>hl7-2</code> itself is a dev-dependency only, used to check that a generated dictionary
    loads — it is not needed to build this crate.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
