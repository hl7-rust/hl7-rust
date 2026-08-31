<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-from-er7-into-json');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'example', label: 'What it produces' },
    { id: 'cli', label: 'Command line' },
    { id: 'library', label: 'Library' },
    { id: 'does', label: 'What it does' },
    { id: 'fallbacks', label: 'Fallback behavior' },
    { id: 'limitations', label: 'Limitations' },
    { id: 'related', label: 'Related crates' }
  ];

  const er7 = `PID|1||241900||TEST^FOUAZ`;

  const json = `{
  "PID": {
    "PID.1": "1",
    "PID.3": {
      "CX.1": "241900"
    },
    "PID.5": {
      "XPN.1": {
        "FN.1": "TEST"
      },
      "XPN.2": "FOUAZ"
    }
  }
}`;

  const cli = `# From a file to stdout
hl7-2-from-er7-into-json samples/orm_o01.hl7

# From stdin, to a file
cat samples/oru_r01.hl7 | hl7-2-from-er7-into-json -o out.json

# Disable message-structure grouping
hl7-2-from-er7-into-json --flat samples/orm_o01.hl7

# Compact (single-line) JSON instead of pretty-printed
hl7-2-from-er7-into-json --compact samples/orm_o01.hl7`;

  const library = `let er7 = "MSH|^~\\\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\\r\\
           PID|1||241900||MEDIANO^FOUAZ\\r\\
           ORC|NW|ORD1";

let json = hl7_2_from_er7_into_json::convert(er7)?;`;

  const options = `use hl7_2_from_er7_into_json::{Options, convert_with_options};

let json = convert_with_options(er7, Options { flat: true, compact: true })?;`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    Convert HL7 v2.5 messages from ER7 to a typed <strong>JSON</strong> representation, as a Rust
    library and command-line tool.
  </p>
  <p>
    Its XML counterpart is
    <a href="/crates/hl7-2-from-er7-into-xml/"><code>hl7-2-from-er7-into-xml</code></a>: same ER7
    parser, same HL7 v2.5 data-type tables, same message-structure grammars — rendered as JSON
    instead of the official v2.xml XML.
  </p>
  <Callout heading="There is no official “v2.json”">
    <p>
      So this crate defines its own JSON mapping, designed to preserve everything v2.xml preserves
      while using idiomatic JSON — real arrays, real <code>null</code> — instead of XML's constructs.
      Its <code>spec/index.md</code> §0 states exactly where it is meant to diverge from the XML
      sibling, and where it is meant to agree.
    </p>
  </Callout>

  <h2 id="example">What it produces</h2>
  <CodeSample language="er7" code={er7} />
  <p>converts to:</p>
  <CodeSample language="json" code={json} />

  <h2 id="cli">Command line</h2>
  <CodeSample language="sh" code={cli} />
  <p>
    Input may hold one message, several messages, or an HL7 batch file (FHS/BHS envelopes are
    dropped); each message becomes one independent JSON document.
  </p>

  <h2 id="library">Library</h2>
  <CodeSample language="rust" code={library} />
  <CodeSample language="rust" code={options} />
  <p>
    <code>convert</code> and <code>convert_with_options</code> return
    <code>Result&lt;String, Hl7Error&gt;</code>; an <code>Err</code> only ever means the message has
    no usable MSH header. Everything below that always converts.
  </p>

  <h2 id="does">What it does</h2>
  <ul>
    <li>
      <strong>ER7 parsing</strong> at every level, and <strong>dynamic delimiters</strong> read from
      <code>MSH-1</code> and <code>MSH-2</code> rather than hardcoded.
    </li>
    <li>
      <strong>Escape sequences</strong>: <code>\F\ \S\ \T\ \R\ \E\</code> decode to the delimiter
      characters and <code>\Xhh..\</code> decodes hex bytes. Unlike XML, decoded text needs no
      further escaping for <code>&amp;</code>, <code>&lt;</code>, <code>&gt;</code> in JSON.
    </li>
    <li>
      <strong>Typed key names</strong> from the same tables the XML sibling uses — producing
      <code>"PID.5": &#123;"XPN.1": &#123;"FN.1": "TEST"&#125;&#125;</code> style nesting.
    </li>
    <li>
      <strong>Repetition and repeating groups become JSON arrays</strong>: a repeating field,
      repeating segment, or repeating group collapses same-named siblings into one array under one
      key — real JSON structure, not XML's repeated-sibling-tag trick.
    </li>
    <li>
      <strong>OBX-5 variable typing</strong>: the value type declared in <code>OBX-2</code> names the
      <code>OBX-5</code> keys.
    </li>
    <li>
      <strong>HL7 null</strong>: the explicit null <code>""</code> becomes JSON
      <code>null</code>; a field with no value at all is simply omitted.
    </li>
    <li>
      <strong>All scalars are JSON strings</strong>, never numbers or booleans — HL7 numeric text
      (leading zeros, explicit signs, trailing precision) is never silently coerced.
    </li>
    <li>
      <strong>Message-structure groups</strong> for ACK, ADT_A01 (also used by ADT^A04/A08/A13),
      ORM_O01, and ORU_R01. The root key comes from <code>MSH-9.3</code> when present, otherwise from
      <code>MSH-9.1</code> and <code>MSH-9.2</code>.
    </li>
  </ul>

  <h2 id="fallbacks">Fallback behavior</h2>
  <ul>
    <li>
      A message whose segment sequence does not fit its declared structure renders with all segments
      <strong>flat</strong> under the root key.
    </li>
    <li>
      Fields of unknown segments, and fields beyond the built-in tables, use positional generic keys:
      <code>"ZDS.1"</code>, <code>"ZDS.1.1"</code>, and so on.
    </li>
  </ul>

  <h2 id="limitations">Limitations</h2>
  <Callout type="warning" heading="Not a validator">
    <p>
      No schema validation, cardinality, or table checking is performed; the input is assumed to be
      sensible HL7 v2.5.
    </p>
  </Callout>
  <ul>
    <li>Only the four message structures listed above are grouped; everything else renders flat.</li>
    <li>
      <strong>A segment name that repeats non-adjacently</strong> is grouped at its first occurrence,
      so segments that sat between the two occurrences lose their place in the sequence — a JSON
      object cannot carry the same key twice. Adjacent repeats, and repeats inside a group the
      grammar knows, are unaffected. This is the one place the JSON mapping loses information the XML
      mapping keeps.
    </li>
    <li>
      ORM_O01 order detail supports the common OBR choice; RQD/RQ1/RXO/ODS/ODT detail segments cause
      a flat rendering.
    </li>
    <li>Formatting escape sequences are preserved as literal text.</li>
    <li>
      <strong>A key that can repeat is a bare value when it occurs once</strong>, and an array only
      when it occurs more than once — consumers that need a uniform shape must normalize that
      themselves.
    </li>
  </ul>

  <RelatedCrates slugs={crate.related} />
</DocPage>
