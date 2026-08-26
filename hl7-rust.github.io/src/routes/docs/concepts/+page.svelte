<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'family', label: 'HL7 is a family, not a standard' },
    { id: 'er7', label: 'ER7: the pipes and carets' },
    { id: 'anatomy', label: 'The anatomy of a v2 message' },
    { id: 'names-paths', label: 'Names describe, paths address' },
    { id: 'dictionary', label: 'The dictionary' },
    { id: 'structures', label: 'Message structures and groups' },
    { id: 'null', label: 'The HL7 null is not an empty field' },
    { id: 'escapes', label: 'Escape sequences' },
    { id: 'v2xml', label: 'v2.xml, and the JSON mapping' },
    { id: 'v3', label: 'HL7 v3: the RIM and the envelope' },
    { id: 'transports', label: 'Transports: MLLP and SOAP' },
    { id: 'z', label: 'Z-segments, and degrading rather than rejecting' }
  ];

  const raw = `MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5`;

  const anatomy = `PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E
 ^  ^  ^                                       ^
 |  |  |                                       |
 |  |  |                                       PID-5, an XPN (person name)
 |  |  PID-3, a CX (identifier), whose 4th component
 |  |  is itself an HD with subcomponents split by &
 |  PID-2, not sent
 PID-1, the set ID

Field       PID-3            separated by |
Repetition  PID-3[2]         separated by ~
Component   PID-3.4          separated by ^
Subcomponent PID-3.4.2       separated by &`;

  const paths = `message.get("PID-5.1")        // segment PID, field 5, component 1
message.get("OBX[2]-5.2")     // the second OBX segment
message.get("PID-3[2].1")     // the second repetition of PID-3
message.get("PID-3.4.2")      // down to a subcomponent

// The tree speaks the other vocabulary:
tree.find("XPN.1")            // "the family-name component of a person name"
tree.find_all("OBX")          // every OBX node
node.path()                   // the path that reads this node back`;

  const dictionary = `{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}`;

  const nulls = `PID|1||""||SMITH
      ^  ^
      |  the explicit HL7 null: "delete the value you have"
      not sent: "I am not telling you anything about this field"`;

  const nullFormats = `ER7    ""                  empty field
XML    <PID.2>""</PID.2>   <PID.2/>  or absent
JSON   null                key omitted`;

  const escapes = `\\F\\   |     the field separator, as a value
\\S\\   ^     component separator
\\T\\   &     subcomponent separator
\\R\\   ~     repetition separator
\\E\\   \\     the escape character itself
\\X0A\\ hex   an arbitrary byte

\\.br\\       a formatting command — kept literally, not decoded`;

  const v3 = `Message                       level 1 — transport: sender, receiver, id
└── ControlAct                 level 2 — the real-world trigger event
    └── domain: xml::Element    level 3 — the interaction's own payload`;

  const mllp = `<VT> message <FS><CR>
0x0B          0x1C 0x0D`;
</script>

<DocPage
  title="Concepts"
  lede="The vocabulary the rest of this site assumes. If you are new to HL7 this is the page to read first; if you are not, skim it for the two or three places where these crates make a specific choice."
  {contents}
>
  <h2 id="family">HL7 is a family, not a standard</h2>
  <p>
    “HL7” names an organization and several unrelated standards it published. They have little in
    common beyond the name and the problem:
  </p>
  <dl>
    <dt>HL7 v2</dt>
    <dd>
      Delimited text. Releases 2.1 through 2.9, first published in the late 1980s, and still the
      format most healthcare data actually moves in. Flexible to the point of being negotiable —
      which is why a “v2 parser” that does not know which release it is reading is not much of a
      parser. Handled by <a href="/crates/hl7-2/"><code>hl7-2</code></a>.
    </dd>
    <dt>HL7 v3</dt>
    <dd>
      XML, generated from one strict object model. It replaced v2's flexibility with rigor, and
      bought consistency at the cost of a steep learning curve. V3 messaging saw limited adoption;
      what did succeed, and still runs, is the Clinical Document Architecture and national
      registries built on the same model. Handled by
      <a href="/crates/hl7-3/"><code>hl7-3</code></a>.
    </dd>
    <dt>The HL7® FHIR® standard</dt>
    <dd>
      Resources over HTTP, the current direction of travel. Not implemented in this workspace —
      but the umbrella crate deliberately leaves <code>hl7::fhir</code> free for it.
    </dd>
  </dl>
  <p>
    That is why the umbrella crate has one module per standard and nothing at its root. A
    “message”, a “segment”, and a “code” all mean different things in each, and one flat namespace
    would only invite mixing them up.
  </p>

  <h2 id="er7">ER7: the pipes and carets</h2>
  <p>
    ER7 — Encoding Rules version 7 — is v2's traditional wire encoding: the pipe-delimited text
    everybody pictures when they hear “HL7”. The delimiters are not fixed by the standard. They
    are declared by each message, in its own first line:
  </p>
  <CodeSample language="er7" code={raw} />
  <p>
    <code>MSH-1</code> is the character immediately after <code>MSH</code> — here <code>|</code>,
    the field separator. <code>MSH-2</code> is the next four characters — <code>^~\&amp;</code> —
    which are the component, repetition, escape, and subcomponent separators in that order. A
    parser that hardcodes those five characters will misread any sender that chose differently,
    and senders do.
  </p>
  <p>
    In this workspace the ER7 layer is its own crate,
    <a href="https://crates.io/crates/er7"><code>er7</code></a>, outside this repository and with
    no dependencies of its own. It owns delimiters, escapes, paths, byte-for-byte rendering, and
    batch splitting — the syntax. Everything above it owns the meaning.
  </p>

  <h2 id="anatomy">The anatomy of a v2 message</h2>
  <p>
    A message is a sequence of segments, one per line, each named by three characters. A segment is
    a sequence of fields. A field may repeat, may split into components, and a component may split
    into subcomponents. Four levels, four separators.
  </p>
  <CodeSample language="text" code={anatomy} label="How a PID segment decomposes" />

  <h2 id="names-paths">Names describe, paths address</h2>
  <p>
    This is the distinction worth internalising early, because both vocabularies appear in the same
    output and they are not interchangeable.
  </p>
  <ul>
    <li>
      A <strong>path</strong> is an address: <code>PID-5.1</code> means segment
      <code>PID</code>, field 5, component 1. Paths are what <code>get</code>, <code>set</code>,
      and the CLI's <code>--query</code> take, and they are what the derive macros' attributes
      hold.
    </li>
    <li>
      A <strong>name</strong> is a description: <code>XPN.1</code> means the family-name component
      of an extended person name. Names are what the tree, the XML elements, and the JSON keys use,
      and they come from the dictionary.
    </li>
  </ul>
  <CodeSample language="rust" code={paths} />

  <h2 id="dictionary">The dictionary</h2>
  <p>
    The dictionary is the per-release knowledge that turns positions into meaning: which data type
    each field of each segment carries, which components each composite type has, and what the
    message structures look like. v2.5 is the complete base; every other release from 2.1 to 2.9 is
    a delta of it, covering the differences these crates model today, and inheriting the rest.
  </p>
  <p>
    The release is chosen from <code>MSH-12</code>, or forced with
    <code>Options::with_version</code>. A release string with no dictionary resolves to the nearest
    older one — <code>2.5.2</code> reads as <code>2.5.1</code> — rather than failing.
  </p>
  <p>
    A dictionary is also a file you can write. That is the whole of schema mode: state a vendor's
    dialect as JSON, inherit a bundled release, and adding a field becomes a configuration change
    rather than a release of your software.
  </p>
  <CodeSample language="json" caption="A complete vendor dictionary" code={dictionary} />
  <p>
    You can also generate one from a site's own XML Schemas, with
    <a href="/crates/hl7-2-from-xsd-into-json-dictionary/"
      ><code>hl7-2-from-xsd-into-json-dictionary</code></a
    >, which additionally captures cardinality. See
    <a href="/guides/dictionaries/">Vendor dictionaries</a>.
  </p>

  <h2 id="structures">Message structures and groups</h2>
  <p>
    <code>MSH-9</code> says what the message is: a message code (<code>ORU</code>), a trigger event
    (<code>R01</code>), and, from v2.3.1, a message-structure id (<code>ORU_R01</code>). The
    structure is a grammar over segments, and it says which segments group together — an
    <code>ORU_R01</code> is a patient result containing an order observation containing
    observations.
  </p>
  <p>
    When the segments fit the grammar, these crates nest them:
    <code>ORU_R01.PATIENT_RESULT.ORDER_OBSERVATION.OBSERVATION</code>. When they do not, the
    message reads flat instead. It is never an error either way — see
    <a href="#z">degrading rather than rejecting</a>, below.
  </p>

  <h2 id="null">The HL7 null is not an empty field</h2>
  <p>
    HL7 v2 distinguishes “I am not telling you anything about this field” from “delete the value
    you currently hold”. The second is the explicit null, written as two double quotes.
  </p>
  <CodeSample language="er7" code={nulls} />
  <p>
    In an update message the difference is the difference between leaving a patient's address alone
    and erasing it, so every crate here keeps the two apart, in every format:
  </p>
  <CodeSample language="text" code={nullFormats} label="The HL7 null across encodings" />
  <Callout type="warning" heading="One thing get cannot tell you">
    <p>
      <code>Message::get</code> returns the text at a path. Both an explicit null and an empty field
      come back as an empty string, so <code>get</code> alone cannot distinguish them. When that
      distinction matters — and in an update message it usually does — read the node from the tree
      rather than the value from the path.
    </p>
  </Callout>

  <h2 id="escapes">Escape sequences</h2>
  <p>
    A value that needs to contain a delimiter escapes it. The escape character is whichever
    <code>MSH-2</code> declared, conventionally a backslash.
  </p>
  <CodeSample language="text" code={escapes} label="The ER7 escape vocabulary" />
  <p>
    The five delimiter escapes and <code>\X..\</code> hex escapes are decoded on the way in and
    re-escaped on the way out. Formatting commands such as <code>\.br\</code> are
    <em>not</em> decoded — they are kept as literal text — because there is no faithful
    representation of a line-break instruction in an XML element or a JSON string, and inventing
    one would lose information the round trip is supposed to preserve.
  </p>

  <h2 id="v2xml">v2.xml, and the JSON mapping</h2>
  <p>
    HL7 also publishes an official XML encoding of v2, namespace
    <code>urn:hl7-org:v2xml</code>. Every field, component, and subcomponent becomes an element
    whose name carries its position as the number after the last dot — <code>&lt;PID.5&gt;</code>,
    and inside it <code>&lt;XPN.1&gt;</code> when the dictionary knows the type or
    <code>&lt;PID.5.1&gt;</code> when it does not.
  </p>
  <p>
    That naming rule is why the reverse direction needs no dictionary at all: the position is in
    the name. It is the single most useful thing to know about these four crates.
  </p>
  <p>
    There is no official “v2.json”. The JSON mapping here is defined by
    <a href="/crates/hl7-2-from-er7-into-json/"><code>hl7-2-from-er7-into-json</code></a> and is
    designed to preserve everything v2.xml preserves while using idiomatic JSON: real arrays for
    repetition, real <code>null</code> for the HL7 null, and every scalar as a string, never a
    number — HL7 numeric text carries leading zeros, explicit signs, and trailing precision that
    a numeric type would silently destroy.
  </p>

  <h2 id="v3">HL7 v3: the RIM and the envelope</h2>
  <p>
    Where v2 is a set of message layouts, v3 is one object model — the Reference Information Model
    — with six backbone classes (<code>Act</code>, <code>Entity</code>, <code>Role</code>,
    <code>ActRelationship</code>, <code>Participation</code>, <code>RoleLink</code>) that every
    domain payload is assembled from.
  </p>
  <p>Every v3 interaction shares a three-level envelope:</p>
  <CodeSample language="text" code={v3} label="The HL7 v3 message envelope" />
  <p>
    <a href="/crates/hl7-3/"><code>hl7-3</code></a> implements the RIM backbone, the data types, and
    that envelope — the part that is the same in every domain. Decoding a specific interaction's
    payload is left to you, with the RIM types. See <a href="/guides/hl7-v3/">the v3 guide</a>.
  </p>

  <h2 id="transports">Transports: MLLP and SOAP</h2>
  <p>
    A v2 message carries no length prefix and no self-delimiting syntax, so a receiver reading a
    TCP socket cannot tell where one message stops and the next begins. MLLP — the Minimal Lower
    Layer Protocol — is the three-byte answer, and it is the whole protocol:
  </p>
  <CodeSample language="text" code={mllp} label="An MLLP frame" />
  <p>
    No length, no checksum, no session, no negotiation, no encryption. MLLP also has no
    acknowledgement of its own: the reply HL7 expects is an HL7 message, an <code>ACK</code> whose
    <code>MSA-2</code> echoes the control ID of the message being answered.
  </p>
  <p>
    SOAP is the other transport — v2's exception, and v3's norm. Same job, over HTTP, with an
    envelope and a WSDL. See <a href="/guides/mllp/">MLLP</a> and <a href="/guides/soap/">SOAP</a>.
  </p>

  <h2 id="z">Z-segments, and degrading rather than rejecting</h2>
  <p>
    Any segment whose name begins with <code>Z</code> is local by definition: the standard reserves
    them and says nothing about what is inside. Nearly every real interface carries at least one.
    The same is true of fields past the end of the published segment, and of data types the
    dictionary has never seen.
  </p>
  <p>The choice these crates make, everywhere, is to degrade rather than reject:</p>
  <ul>
    <li>An unknown segment converts, using positional generic names (<code>ZDS.1</code>).</li>
    <li>A structure with no grammar renders flat, which is still well-formed and lossless.</li>
    <li>An unknown data type falls back to positional component names.</li>
    <li>A local Z-segment does not make an otherwise conformant message fail strict mode.</li>
  </ul>
  <p>
    The cost of that choice is a lost typed <em>name</em>, never a lost <em>value</em> and never a
    rejected message. That is the trade these crates make deliberately, and it is what makes the
    three-stage workflow in <a href="/tutorials/vendor-dialect/">Taming a vendor dialect</a>
    possible.
  </p>
</DocPage>
