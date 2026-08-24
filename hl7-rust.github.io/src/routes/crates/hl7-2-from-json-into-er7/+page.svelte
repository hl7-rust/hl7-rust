<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-from-json-into-er7');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'no-dictionary', label: 'Why it needs no dictionary' },
    { id: 'example', label: 'What it produces' },
    { id: 'cli', label: 'Command line' },
    { id: 'library', label: 'Library' },
    { id: 'does', label: 'What it does' },
    { id: 'limitations', label: 'Limitations' },
    { id: 'related', label: 'Related crates' }
  ];

  const json = `{
  "PID.1": "1",
  "PID.3": { "CX.1": "241900" },
  "PID.5": { "XPN.1": { "FN.1": "TEST" }, "XPN.2": "FOUAZ" }
}`;

  const er7 = `PID|1||241900||TEST^FOUAZ`;

  const cli = `# From a file to stdout
hl7-2-from-json-into-er7 samples/orm_o01.json

# From stdin, to a file
cat samples/oru_r01.json | hl7-2-from-json-into-er7 -o out.hl7

# Choose the segment terminator
hl7-2-from-json-into-er7 --terminator crlf samples/orm_o01.json`;

  const library = `let json = r#"{
  "ORM_O01": {
    "MSH": { "MSH.1": "|", "MSH.2": "^~\\\\&", "MSH.9": {"MSG.1": "ORM", "MSG.2": "O01"} },
    "ORM_O01.PATIENT": {
      "PID": { "PID.5": { "XPN.1": {"FN.1": "TEST"}, "XPN.2": "FOUAZ" } }
    }
  }
}"#;

let er7 = hl7_2_from_json_into_er7::convert(json)?;`;

  const parse = `use hl7_2_from_json_into_er7::parse;

let message = parse(json)?;
assert_eq!(message.query("PID-5.1")?.as_deref(), Some("TEST"));`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    Convert HL7 v2.5 messages from the typed JSON representation that
    <a href="/crates/hl7-2-from-er7-into-json/"><code>hl7-2-from-er7-into-json</code></a> produces,
    back to the traditional pipe-delimited ER7 encoding, as a Rust library and command-line tool.
  </p>

  <h2 id="no-dictionary">Why it needs no dictionary</h2>
  <Callout heading="The position is in the key">
    <p>
      The forward crate's key names always carry the field, component, or subcomponent
      <em>position</em> as the number after the key's last dot, whether or not the name in front of it
      is a recognized data type. So reversal is a purely structural rebuild, and this crate's only
      dependency is the same <a href="https://crates.io/crates/er7"><code>er7</code></a> encoding
      layer the forward crate uses.
    </p>
  </Callout>

  <h2 id="example">What it produces</h2>
  <p>A JSON fragment such as:</p>
  <CodeSample language="json" code={json} />
  <p>converts back to:</p>
  <CodeSample language="er7" code={er7} />

  <h2 id="cli">Command line</h2>
  <CodeSample language="sh" code={cli} />
  <p>
    Message-structure group keys (<code>"ORM_O01.PATIENT"</code>,
    <code>"ORU_R01.ORDER_OBSERVATION"</code>, …) are flattened automatically, and a repeating
    field, segment, or group's JSON array un-arrays back into its repetitions or repeated segments —
    grouped and <code>--flat</code>, single-occurrence and repeated, all reconstruct the same
    message.
  </p>

  <h2 id="library">Library</h2>
  <CodeSample language="rust" code={library} />
  <CodeSample language="rust" code={parse} />
  <p>
    <code>convert</code>, <code>convert_with_options</code>, and <code>parse</code> return
    <code>Result&lt;_, Hl7Error&gt;</code>; an <code>Err</code> only ever means the input is not
    well-formed JSON, is not shaped like a converted message (a single-key object over an object of
    segments), or the message has no usable
    <code>MSH</code>/<code>FHS</code>/<code>BHS</code> header.
  </p>

  <h2 id="does">What it does</h2>
  <ul>
    <li>
      <strong>A minimal, dependency-free JSON reader</strong>: the full RFC 8259 grammar including
      <code>\uXXXX</code> surrogate pairs, with numbers and booleans tolerated (coerced to text) even
      though the forward crate never emits them.
    </li>
    <li><strong>Position-based reconstruction, no data-type dictionary</strong> — see above.</li>
    <li>
      <strong>Group flattening and array un-nesting</strong>: group keys are recognized by their
      dotted name and flattened back into a plain segment sequence, and a JSON array un-arrays into
      repeated occurrences. No message-structure grammar is needed in this direction.
    </li>
    <li>
      <strong>Delimiter recovery</strong>: the header's <code>.1</code> and <code>.2</code> fields
      are reassembled into a synthetic header line and handed to
      <code>er7::Separators::from_header</code>.
    </li>
    <li>
      <strong>Faithful re-escaping</strong>: decoded leaf text is retokenized against
      <code>er7</code>'s own escape-sequence vocabulary, so delimiter characters are re-escaped while
      formatting sequences the forward crate never decoded (<code>\.br\</code>, <code>\H\</code>, …)
      are written back exactly as they were.
    </li>
    <li>
      <strong>HL7 null</strong>: JSON <code>null</code> reconstructs as the explicit null
      <code>""</code> at that position.
    </li>
  </ul>

  <h2 id="limitations">Limitations</h2>
  <ul>
    <li>
      A field repetition that was present but entirely blank (not the explicit null) is dropped by
      the <em>forward</em> crate's own encoding and cannot be recovered here.
    </li>
    <li>
      One JSON document converts to one ER7 message; there is no batch-file convention on the JSON
      side to split.
    </li>
    <li>
      A segment name that repeated non-adjacently in the original message was already collapsed by
      the forward crate — a JSON object cannot carry the same key twice — so its original sequence
      cannot be restored here either.
    </li>
  </ul>
  <p>See <code>spec/index.md</code> §5 for the full list.</p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
