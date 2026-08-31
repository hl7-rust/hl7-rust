<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'four-crates', label: 'Four crates, two directions, two formats' },
    { id: 'xml', label: 'ER7 into v2.xml' },
    { id: 'json', label: 'ER7 into JSON' },
    { id: 'differences', label: 'Where XML and JSON deliberately diverge' },
    { id: 'reverse', label: 'Back to ER7' },
    { id: 'library', label: 'Using them as libraries' },
    { id: 'batches', label: 'Batches' },
    { id: 'fallbacks', label: 'Fallbacks, and what is lossy' },
    { id: 'not-validators', label: 'None of them is a validator' }
  ];

  const er7 = `PID|1||241900||TEST^FOUAZ`;

  const xml = `<PID>
  <PID.1>1</PID.1>
  <PID.3>
    <CX.1>241900</CX.1>
  </PID.3>
  <PID.5>
    <XPN.1>
      <FN.1>TEST</FN.1>
    </XPN.1>
    <XPN.2>FOUAZ</XPN.2>
  </PID.5>
</PID>`;

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

  const xmlCli = `hl7-2-from-er7-into-xml samples/orm_o01.hl7
cat samples/oru_r01.hl7 | hl7-2-from-er7-into-xml -o out.xml
hl7-2-from-er7-into-xml --flat samples/orm_o01.hl7
hl7-2-from-er7-into-xml --dictionary my-dialect.json samples/orm_o01.hl7
hl7-2-from-er7-into-xml --dictionary my-dialect.json --schema-shape samples/orm_o01.hl7`;

  const jsonCli = `hl7-2-from-er7-into-json samples/orm_o01.hl7
hl7-2-from-er7-into-json --compact samples/orm_o01.hl7
hl7-2-from-er7-into-json --flat samples/orm_o01.hl7`;

  const reverseCli = `hl7-2-from-xml-into-er7 samples/orm_o01.xml
hl7-2-from-json-into-er7 samples/orm_o01.json
hl7-2-from-xml-into-er7 --terminator crlf samples/orm_o01.xml`;

  const libXml = `let er7 = "MSH|^~\\\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\\r\\
           PID|1||241900||MEDIANO^FOUAZ\\r\\
           ORC|NW|ORD1";

let xml = hl7_2_from_er7_into_xml::convert(er7)?;
let json = hl7_2_from_er7_into_json::convert(er7)?;`;

  const libOptions = `use hl7_2_from_er7_into_json::{Options, convert_with_options};

let json = convert_with_options(er7, Options { flat: true, compact: true })?;`;

  const libReverse = `let er7 = hl7_2_from_xml_into_er7::convert(xml)?;
let er7 = hl7_2_from_json_into_er7::convert(json)?;

// Or take the full er7::Message, to query or edit before rendering:
let message = hl7_2_from_xml_into_er7::parse(xml)?;
assert_eq!(message.query("PID-5.1")?.as_deref(), Some("TEST"));`;

  const batch = `use hl7_2_from_er7_into_xml::{convert, split_messages};

let batch = "MSH|^~\\\\&|A||||1||ACK|1|P|2.5\\rMSA|AA|1\\r\\
             MSH|^~\\\\&|B||||2||ACK|2|P|2.5\\rMSA|AA|2";

for message in split_messages(batch) {
    match convert(&message) {
        Ok(xml) => println!("{xml}"),
        Err(e) => eprintln!("skipping malformed message: {e}"),
    }
}`;

  const roundTrip = `hl7-2-from-er7-into-xml samples/orm_o01.hl7 \\
  | hl7-2-from-xml-into-er7`;
</script>

<DocPage
  lede="ER7 to the official v2.xml XML representation or to a typed JSON mapping, and back again. Four small crates, each with a library API and a command-line binary."
  {contents}
>
  <h2 id="four-crates">Four crates, two directions, two formats</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Crate</th><th>Direction</th><th>Needs a dictionary?</th></tr>
      </thead>
      <tbody>
        <tr>
          <td><a href="/crates/hl7-2-from-er7-into-xml/"><code>hl7-2-from-er7-into-xml</code></a></td>
          <td>ER7 → v2.xml</td>
          <td>Yes — to name elements by data type</td>
        </tr>
        <tr>
          <td><a href="/crates/hl7-2-from-xml-into-er7/"><code>hl7-2-from-xml-into-er7</code></a></td>
          <td>v2.xml → ER7</td>
          <td>No</td>
        </tr>
        <tr>
          <td><a href="/crates/hl7-2-from-er7-into-json/"><code>hl7-2-from-er7-into-json</code></a></td>
          <td>ER7 → typed JSON</td>
          <td>Yes</td>
        </tr>
        <tr>
          <td><a href="/crates/hl7-2-from-json-into-er7/"><code>hl7-2-from-json-into-er7</code></a></td>
          <td>typed JSON → ER7</td>
          <td>No</td>
        </tr>
      </tbody>
    </table>
  </div>
  <Callout heading="Why the reverse crates need no dictionary">
    <p>
      Every element or key name a forward crate writes carries its own <em>position</em>, as the
      number after the name's last dot — whether the name in front of it is a recognized data type
      (<code>XPN.1</code>) or a positional fallback (<code>PID.5.1</code>). So reversing is a purely
      structural rebuild, and each reverse crate's <code>spec/index.md</code> §1.1 says exactly
      that. It is the single most useful fact about these four crates.
    </p>
  </Callout>

  <h2 id="xml">ER7 into v2.xml</h2>
  <p>This ER7 fragment:</p>
  <CodeSample language="er7" code={er7} />
  <p>
    becomes the official v2.xml structure (namespace <code>urn:hl7-org:v2xml</code>), with
    components named after their HL7 v2.5 data types:
  </p>
  <CodeSample language="xml" code={xml} />
  <CodeSample language="sh" caption="From the command line" code={xmlCli} />
  <p>
    <code>--dictionary</code> reads a JSON dictionary in place of the bundled v2.5 tables, and
    <code>--schema-shape</code> lets that dictionary decide the document's shape rather than only
    its names. Both are covered in <a href="/guides/dictionaries/">Vendor dictionaries</a>.
  </p>

  <h2 id="json">ER7 into JSON</h2>
  <p>The same fragment, in the JSON mapping:</p>
  <CodeSample language="json" code={json} />
  <CodeSample language="sh" code={jsonCli} />
  <p>
    There is no official “v2.json” standard to target, so this crate defines its own mapping —
    designed to preserve everything v2.xml preserves while using idiomatic JSON instead of XML's
    constructs. It shares the ER7 parser, the data-type tables, and the message-structure grammars
    with its XML sibling; only the renderer differs.
  </p>

  <h2 id="differences">Where XML and JSON deliberately diverge</h2>
  <p>
    The two forward specs are kept consistent with each other except where the target format forces
    a difference. Each one's §0 states exactly where. The differences that will actually affect
    your consumers:
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Situation</th><th>v2.xml</th><th>JSON</th></tr>
      </thead>
      <tbody>
        <tr>
          <td>A repeating field</td>
          <td>Repeated sibling elements of the same name</td>
          <td>A real array under one key</td>
        </tr>
        <tr>
          <td>The explicit HL7 null <code>""</code></td>
          <td><code>&lt;PID.2&gt;""&lt;/PID.2&gt;</code>, keeping the literal text</td>
          <td><code>null</code></td>
        </tr>
        <tr>
          <td>A field with no value at all</td>
          <td>An empty element, or absent</td>
          <td>The key is omitted</td>
        </tr>
        <tr>
          <td>A numeric-looking value</td>
          <td>Text</td>
          <td>A JSON string, never a number</td>
        </tr>
      </tbody>
    </table>
  </div>
  <p>
    That last row matters more than it looks. HL7 numeric text carries leading zeros, explicit
    signs, and trailing precision, and a JSON number would silently destroy all three. Every scalar
    is a string.
  </p>
  <Callout type="warning" heading="A JSON key is a bare value once and an array only when repeated">
    <p>
      A key that <em>can</em> repeat is written as a bare value when it occurs once, and as an array
      only when it occurs more than once. Consumers that need a uniform shape have to normalize
      that themselves — check whether the value is an array before iterating it.
    </p>
  </Callout>

  <h2 id="reverse">Back to ER7</h2>
  <CodeSample language="sh" code={reverseCli} />
  <p>
    Message-structure group elements and keys are flattened automatically, and a repeating field's
    JSON array un-arrays back into its repetitions — so grouped and <code>--flat</code>,
    single-occurrence and repeated, all reconstruct the same message. Namespace prefixes are
    ignored, so a document that binds <code>urn:hl7-org:v2xml</code> to any prefix converts
    identically.
  </p>
  <p>The two compose into a round trip you can run from a shell:</p>
  <CodeSample language="sh" code={roundTrip} />
  <p>
    The output is the original ER7 message, canonicalized. That is a good smoke test after changing
    either crate's naming rules, since a drift in one breaks the other's assumptions. See
    <a href="/tutorials/round-trip/">A lossless round trip</a>.
  </p>

  <h2 id="library">Using them as libraries</h2>
  <CodeSample language="rust" code={libXml} />
  <CodeSample language="rust" caption="With options" code={libOptions} />
  <CodeSample language="rust" caption="And in reverse" code={libReverse} />
  <p>
    The reverse crates' <code>parse</code> gives you the full <code>er7::Message</code> — useful
    when the point of the conversion is to query or edit, not to render text you immediately
    re-parse.
  </p>

  <h2 id="batches">Batches</h2>
  <p>
    Input on the ER7 side may hold one message, several, or an HL7 batch file; FHS/BHS envelopes are
    dropped, and each message becomes one independent output document.
  </p>
  <CodeSample language="rust" code={batch} />
  <p>
    There is no batch convention on the XML or JSON side, so one document converts to one ER7
    message in the reverse direction.
  </p>

  <h2 id="fallbacks">Fallbacks, and what is lossy</h2>
  <p>Fidelity degrades instead of failing:</p>
  <ul>
    <li>
      A message whose segments do not fit its declared structure — because it carries Z-segments,
      or uses a structure with no built-in grammar — renders <strong>flat</strong> under the root.
      Still well-formed, still lossless for values.
    </li>
    <li>
      Fields of unknown segments, and fields beyond the built-in tables, use positional generic
      names: <code>&lt;ZDS.1&gt;</code>, <code>"ZDS.1.1"</code>, and so on.
    </li>
    <li>
      Formatting escape sequences such as <code>\.br\</code> are preserved as literal text rather
      than mapped to a dedicated construct.
    </li>
  </ul>
  <p>The genuinely lossy cases, both of which are properties of the forward encoding:</p>
  <ul>
    <li>
      A field repetition that was present but <em>entirely blank</em> (not the explicit null) is
      dropped by the forward crate's own encoding and cannot be recovered on the way back.
    </li>
    <li>
      In JSON only: a segment name that repeats <em>non-adjacently</em> is grouped at its first
      occurrence, so segments that sat between the two occurrences lose their place in the
      sequence — a JSON object cannot carry the same key twice. Adjacent repeats, and repeats
      inside a group the grammar knows, are unaffected.
    </li>
  </ul>
  <p>
    Grammars are included for <code>ACK</code>, <code>ADT_A01</code> (also used by
    <code>ADT^A04</code>, <code>A08</code>, and <code>A13</code>), <code>ORM_O01</code>, and
    <code>ORU_R01</code>. Everything else renders flat.
  </p>

  <h2 id="not-validators">None of them is a validator</h2>
  <p>
    No schema validation, no cardinality checking, no table checking. The input is assumed to be
    sensible HL7 v2.5. An <code>Err</code> from <code>convert</code> only ever means the message has
    no usable header — empty input, a missing <code>MSH</code>, or a malformed one; or on the
    reverse side, input that is not well-formed XML or JSON.
  </p>
  <p>
    If you need conformance checking, run <a href="/guides/validating/"><code>hl7-2</code></a>
    first and convert what passes.
  </p>
</DocPage>
