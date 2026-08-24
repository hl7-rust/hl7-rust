<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-from-xml-into-er7');

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

  const er7 = `PID|1||241900||TEST^FOUAZ`;

  const cli = `# From a file to stdout
hl7-2-from-xml-into-er7 samples/orm_o01.xml

# From stdin, to a file
cat samples/oru_r01.xml | hl7-2-from-xml-into-er7 -o out.hl7

# Choose the segment terminator
hl7-2-from-xml-into-er7 --terminator crlf samples/orm_o01.xml`;

  const library = `let xml = r#"<ORM_O01 xmlns="urn:hl7-org:v2xml">
  <MSH>
    <MSH.1>|</MSH.1>
    <MSH.2>^~\\&amp;</MSH.2>
    <MSH.9><MSG.1>ORM</MSG.1><MSG.2>O01</MSG.2></MSH.9>
  </MSH>
  <ORM_O01.PATIENT>
    <PID><PID.5><XPN.1><FN.1>TEST</FN.1></XPN.1><XPN.2>FOUAZ</XPN.2></PID.5></PID>
  </ORM_O01.PATIENT>
</ORM_O01>"#;

let er7 = hl7_2_from_xml_into_er7::convert(xml)?;`;

  const parse = `use hl7_2_from_xml_into_er7::parse;

// When you want the full er7::Message — to query or edit it — rather than
// just its ER7 text.
let message = parse(xml)?;
assert_eq!(message.query("PID-5.1")?.as_deref(), Some("TEST"));`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    Convert HL7 v2.5 messages from the official HL7 <strong>v2.xml</strong> XML representation
    (<code>urn:hl7-org:v2xml</code>) back to the traditional pipe-delimited ER7 encoding, as a Rust
    library and command-line tool.
  </p>
  <p>
    It is the inverse of
    <a href="/crates/hl7-2-from-er7-into-xml/"><code>hl7-2-from-er7-into-xml</code></a>, and it reads
    back exactly what that crate writes.
  </p>

  <h2 id="no-dictionary">Why it needs no dictionary</h2>
  <Callout heading="The position is in the name">
    <p>
      The forward crate's element names always carry the field, component, or subcomponent
      <em>position</em> as the number after the name's last dot — whether or not the name in front of
      it is a recognized data type. <code>&lt;XPN.1&gt;</code> and <code>&lt;PID.5.1&gt;</code> both
      say “component 1”. So this crate reconstructs a message correctly with no HL7 v2.5 data-type
      dictionary at all, and its <code>spec/index.md</code> §1.1 says exactly that.
    </p>
  </Callout>
  <p>
    Which is why this crate is so much smaller than its forward counterpart: reversal is a purely
    structural, position-based rebuild. It depends on
    <a href="https://crates.io/crates/er7"><code>er7</code></a> for the encoding layer and, since
    0.5.0, on
    <a href="/crates/hl7-2-xml-lite-helper/"><code>hl7-2-xml-lite-helper</code></a> for reading the
    XML — it used to carry its own minimal XML reader.
  </p>

  <h2 id="example">What it produces</h2>
  <p>A v2.xml fragment such as:</p>
  <CodeSample language="xml" code={xml} />
  <p>converts back to:</p>
  <CodeSample language="er7" code={er7} />

  <h2 id="cli">Command line</h2>
  <CodeSample language="sh" code={cli} />
  <p>
    Message-structure group elements (<code>&lt;ORM_O01.PATIENT&gt;</code>,
    <code>&lt;ORU_R01.ORDER_OBSERVATION&gt;</code>, …) are flattened automatically; grouped and
    <code>--flat</code> input from the forward crate both reconstruct the same message.
  </p>
  <p>
    Namespace prefixes are ignored: a document that binds <code>urn:hl7-org:v2xml</code> to a prefix
    converts to exactly the same ER7 as one that makes it the default namespace, whatever prefix the
    sending system happened to pick.
  </p>

  <h2 id="library">Library</h2>
  <CodeSample language="rust" code={library} />
  <CodeSample language="rust" code={parse} />
  <p>
    <code>convert</code>, <code>convert_with_options</code>, and <code>parse</code> return
    <code>Result&lt;_, Hl7Error&gt;</code>; an <code>Err</code> only ever means the input is not
    well-formed XML, or the message has no usable <code>MSH</code>/<code>FHS</code>/<code>BHS</code>
    header. Everything else converts, falling back gracefully rather than failing.
  </p>
  <p>
    <code>convert_with_options</code> takes the segment terminator, as
    <code>er7::RenderOptions</code>, re-exported here as
    <code>hl7_2_from_xml_into_er7::er7::RenderOptions</code>.
  </p>

  <h2 id="does">What it does</h2>
  <ul>
    <li>
      <strong>XML reading</strong> for exactly the subset v2.xml uses: nested elements, text, the
      predefined entities and numeric character references, with attributes, comments, and the XML
      declaration recognized and skipped.
    </li>
    <li>
      <strong>Position-based reconstruction, no data-type dictionary</strong> — see above.
    </li>
    <li>
      <strong>Group flattening</strong>: message-structure group elements are recognized by their
      dotted name and flattened back into a plain segment sequence. No message-structure grammar is
      needed in this direction.
    </li>
    <li>
      <strong>Delimiter recovery</strong>: the header's <code>.1</code> and <code>.2</code> fields
      are reassembled into a synthetic header line and handed to
      <code>er7::Separators::from_header</code>, reusing <code>er7</code>'s own delimiter parsing.
    </li>
    <li>
      <strong>Faithful re-escaping</strong>: decoded leaf text is retokenized against
      <code>er7</code>'s own escape-sequence vocabulary, so delimiter characters are re-escaped while
      formatting sequences the forward crate never decoded (<code>\.br\</code>, <code>\H\</code>, …)
      are written back exactly as they were.
    </li>
    <li>
      <strong>HL7 null versus empty</strong>: an element whose text is <code>""</code> reconstructs
      as the explicit null (“delete this”); a self-closing or empty element reconstructs as an
      <em>empty</em> value (“nothing was sent”). The XML Encoding Rules give the two opposite
      meanings, so this crate keeps them apart — padding an XSD-shaped document with empty elements
      does not turn it into a message full of deletion markers.
    </li>
  </ul>

  <h2 id="limitations">Limitations</h2>
  <ul>
    <li>
      A field repetition that was present but entirely blank (not the explicit null) is dropped by
      the <em>forward</em> crate's own encoding and cannot be recovered here — a property of v2.xml as
      that crate writes it, not something this crate's reversal introduces.
    </li>
    <li>
      One v2.xml document converts to one ER7 message; there is no batch-file convention on the XML
      side to split.
    </li>
  </ul>
  <p>See <code>spec/index.md</code> §5 for the full list.</p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
