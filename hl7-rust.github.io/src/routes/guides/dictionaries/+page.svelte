<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'why', label: 'Why a dictionary is a file' },
    { id: 'smallest', label: 'The smallest useful dictionary' },
    { id: 'format', label: 'The format, in full' },
    { id: 'lists-objects', label: 'Lists replace, objects patch' },
    { id: 'cardinality', label: 'Cardinality, and why it matters' },
    { id: 'inheritance', label: 'Inheritance' },
    { id: 'loading', label: 'Loading one' },
    { id: 'from-xsd', label: 'Generating one from XSDs' },
    { id: 'schema-shape', label: 'Schema shape, in the XML converter' }
  ];

  const smallest = `{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}`;

  const format = `{
  "version": "2.5",
  "description": "Acme Labs, interface 4",
  "inherits": "2.5",

  "types": {
    "XPN": ["FN", "ST", "ST", "ST", "ST", "IS", "ID", "ID", "DR", "TS"]
  },

  "segments": {
    "PID": ["SI", "CX", "CX", "CX", "XPN"],
    "NK1": ["SI", { "type": "XPN", "repeats": true }],
    "MSH": { "12": "ID" }
  },

  "aliases": { "ADT_A04": "ADT_A01" },

  "structures": {
    "ACK": [
      { "segment": "MSH", "required": true },
      { "segment": "MSA", "required": true },
      { "segment": "ERR", "repeats": true },
      { "group": "PATIENT", "repeats": true, "items": ["PID", "PV1"] }
    ]
  }
}`;

  const forms = `// A list states the whole thing and REPLACES what was inherited.
"segments": { "PID": ["SI", "CX", "CX", "CX", "XPN"] }

// An object states individual 1-based positions and leaves the rest
// inherited. This is how a release delta restates MSH-12 without
// restating the whole of MSH.
"segments": { "MSH": { "12": "ID" } }`;

  const cardinality = `// A bare type name, and the same declaration with cardinality:
"NK1": ["SI", "XPN"]
"NK1": ["SI", { "type": "XPN", "repeats": true }]

// Read it back:
assert_eq!(dictionary.field_type("PID", 3), Some("CX"));
assert!(dictionary.field_cardinality("PID", 3).repeats);`;

  const inheritance = `// Layer over a bundled release.
{ "inherits": "2.5", "segments": { "ZAC": ["SI", "XPN", "DT"] } }

// Remove something the base defines.
{ "inherits": "2.5", "segments": { "SFT": null } }

// Or layer over a dictionary you already hold, ignoring "inherits":
let derived = hl7_2::Dictionary::from_json_over(&text, "acme", base)?;`;

  const loading = `use std::sync::Arc;

let text = std::fs::read_to_string("acme.json")?;
let dictionary = hl7_2::Dictionary::from_json(&text, "acme")?;

let options = hl7_2::Options::new().with_dictionary(Arc::new(dictionary));
let message = hl7_2::parse_with_options(text_of_message, &options)?;

// The vendor's own segment now reads like any standard one.
assert_eq!(message.tree().find("XPN.2").unwrap().text(), "JOHN");`;

  const cli = `hl7-v2 --dictionary acme.json --flat vendor.hl7`;

  const layout = `schemas/paris/
  2_5_1_types.xsd      composite data types and their components
  2_5_1_fields.xsd     every SEG.n element and the data type it carries
  2_5_1_segments.xsd   each segment's field list, with cardinality
  ADT_A05.xsd          one abstract message structure each
  ADT_A39.xsd`;

  const generate = `hl7-2-from-xsd-into-json-dictionary schemas/paris \\
    --name paris \\
    --alias ADT_A28=ADT_A05 \\
    --alias ADT_A31=ADT_A05 \\
    --inherits 2.5 \\
    -o paris.json`;

  const asLibrary = `use hl7_2_from_xsd_into_json_dictionary::{Options, convert_directory};

let document = convert_directory("schemas/paris".as_ref(), &Options::default())?;
std::fs::write("paris.json", document.to_json())?;`;

  const schemaShape = `# Name what a field IS, using the vendor's own types
hl7-2-from-er7-into-xml --dictionary paris.json message.hl7

# Let the dictionary decide what the document CONTAINS
hl7-2-from-er7-into-xml --dictionary paris.json --schema-shape message.hl7`;
</script>

<DocPage
  lede="The dictionary is what turns positions into meaning. It is also a JSON file you can write — which is how a vendor's Z-segments become named fields without a release of your software."
  {contents}
>
  <h2 id="why">Why a dictionary is a file</h2>
  <p>
    Every real HL7 v2 interface is a dialect. The sender adds a segment nobody standardised, or
    uses a field past the end of the published segment, or means something specific by a component
    the standard leaves open. If handling that requires editing and shipping your parser, then a
    business decision made on Tuesday becomes an engineering release on Friday.
  </p>
  <p>
    So the dictionary — the per-release knowledge of which data type each field carries and what
    each composite type contains — is expressed as data. The bundled releases are written in
    exactly the same format as anything you write, which means a vendor schema can inherit a
    standard release and state only what it changes.
  </p>

  <h2 id="smallest">The smallest useful dictionary</h2>
  <p>This is complete and valid. It says: everything from v2.5, plus one local segment.</p>
  <CodeSample language="json" caption="acme.json" code={smallest} />
  <p>
    <code>ZAC</code>'s three fields now carry a sequence id, a person name, and a date. The second
    of those will decompose into <code>XPN.1</code>, <code>XPN.2</code> and the rest, exactly as
    <code>PID-5</code> does, because <code>XPN</code> is a composite the base release already
    describes.
  </p>

  <h2 id="format">The format, in full</h2>
  <CodeSample language="json" code={format} />
  <dl>
    <dt><code>types</code></dt>
    <dd>
      Maps a composite data type to its component data types, in order. A type absent from
      <code>types</code> is primitive — or unknown, which behaves the same way: the value is
      treated as a scalar.
    </dd>
    <dt><code>segments</code></dt>
    <dd>
      Maps a segment name to its field data types, index 0 being field 1. The sentinel
      <code>"VAR"</code> marks a field whose type another field names — in practice only
      <code>OBX-5</code>, whose type is declared in <code>OBX-2</code>.
    </dd>
    <dt><code>aliases</code></dt>
    <dd>
      Maps <code>CODE_TRIGGER</code> to the structure that carries it, which is how
      <code>ADT^A08</code> resolves to <code>ADT_A01</code>.
    </dd>
    <dt><code>structures</code></dt>
    <dd>
      Maps a message structure id to its grammar. An item is an object with <code>segment</code>,
      or with <code>group</code> plus <code>items</code>; each takes optional
      <code>required</code> and <code>repeats</code>, both defaulting to false. A bare string is
      shorthand for an optional, non-repeating segment.
    </dd>
  </dl>

  <h2 id="lists-objects">Lists replace, objects patch</h2>
  <p>
    This is the one subtlety in the format, and it is worth reading twice — the two forms mean
    different things.
  </p>
  <CodeSample language="json" code={forms} />
  <p>
    A position the object form leaves unstated, and that no inherited list covers, reads as
    <em>unknown</em> — not as some default. Unknown is fine: the value still converts, positionally.
  </p>

  <h2 id="cardinality">Cardinality, and why it matters</h2>
  <p>
    Either form may write a position as a bare data type name, or as an object that also states
    cardinality. The two are the same declaration when the object states nothing else.
  </p>
  <CodeSample language="json" code={cardinality} />
  <p>
    <code>required</code> and <code>repeats</code> both default to false. They exist because a
    dictionary generated from XML Schema knows a field's <code>minOccurs</code> and
    <code>maxOccurs</code>, and both change what a faithful conversion emits: a
    <strong>required</strong> field is written even when the message leaves it empty, so the
    position stays visible to a validator; and a field that does <strong>not</strong> repeat keeps
    its repetition separator as ordinary text instead of being split into several elements.
  </p>
  <Callout heading="The exact upper bound is not modelled">
    <p>
      “At most 10” and “unbounded” both read as <code>"repeats": true</code>, because nothing
      downstream distinguishes them. Cardinality is kept for <code>segments</code> only: composite
      components neither repeat nor are individually required, so an object form under
      <code>types</code> states a type and nothing more.
    </p>
  </Callout>

  <h2 id="inheritance">Inheritance</h2>
  <CodeSample language="json" code={inheritance} />
  <p>
    <code>"inherits": "2.5"</code> starts from that bundled release and layers the document over
    it: a listed entry replaces, <code>null</code> removes, and anything unmentioned is inherited.
    A schema that inherits nothing describes the world by itself, and everything it omits reads
    positionally.
  </p>

  <h2 id="loading">Loading one</h2>
  <CodeSample language="rust" code={loading} />
  <p>The command-line tool takes the same file:</p>
  <CodeSample language="sh" code={cli} />
  <p>
    Schema mode is generic mode with a dictionary you supplied — everything about the tree, the
    paths, the fallbacks, and the grouping works identically. Nothing else in your code changes.
  </p>

  <h2 id="from-xsd">Generating one from XSDs</h2>
  <p>
    Many sites already have HL7 v2.xml XML Schemas — as HL7 published them, or as a vendor
    customised them. <a href="/crates/hl7-2-from-xsd-into-json-dictionary/"
      ><code>hl7-2-from-xsd-into-json-dictionary</code></a
    > turns a directory of them into the dictionary these crates read, and captures cardinality
    along the way.
  </p>
  <CodeSample language="text" caption="What a schema directory looks like" code={layout} />
  <p>
    The <code>2_5_1</code> prefix is discovered from a structure file's
    <code>&lt;xsd:include&gt;</code>, so the directory can be named for the sending system rather
    than for the HL7 release.
  </p>
  <CodeSample language="sh" code={generate} />
  <p>
    Two things the schemas cannot tell you, so you pass them in.
    <code>--alias</code> says which trigger events arrive carried by another message's structure: a
    directory holds <code>ADT_A05.xsd</code> but never says that an <code>ADT^A28</code> is one.
    <code>--inherits</code> layers the generated document over a bundled release instead of leaving
    it to stand alone.
  </p>
  <CodeSample language="rust" caption="Or as a library" code={asLibrary} />

  <h2 id="schema-shape">Schema shape, in the XML converter</h2>
  <p>
    A generated dictionary carries cardinality, and that unlocks a second mode in
    <a href="/crates/hl7-2-from-er7-into-xml/"><code>hl7-2-from-er7-into-xml</code></a>.
  </p>
  <CodeSample language="sh" code={schemaShape} />
  <p>
    Without <code>--schema-shape</code>, the dictionary only names what a field <em>is</em>. With
    it, the dictionary decides what the document <em>contains</em>: required fields are written
    even when empty, fields that cannot repeat keep their repetition separator as text, and no
    field the dictionary does not declare is written at all.
  </p>
  <p>
    That is what lets the converter emit a document which validates against the very schemas the
    dictionary was built from — which is usually the reason someone wanted v2.xml in the first
    place.
  </p>
  <p>
    For a start-to-finish walkthrough of discovering a dialect and writing it down, see
    <a href="/tutorials/vendor-dialect/">Taming a vendor dialect</a>.
  </p>
</DocPage>
