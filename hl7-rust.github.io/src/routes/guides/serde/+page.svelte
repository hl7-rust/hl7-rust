<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'three-crates', label: 'Three crates, and why they are separate' },
    { id: 'v2-message', label: 'A v2 message: ER7 text plus release' },
    { id: 'v2-tree', label: 'The dictionary-named tree' },
    { id: 'v2-findings', label: 'Validation findings' },
    { id: 'v2-schema-mode', label: 'Reading back through your own dictionary' },
    { id: 'v3', label: 'A v3 interaction' },
    { id: 'strict', label: 'Strict mode' },
    { id: 'any-format', label: 'Any format, not just JSON' },
    { id: 'not-conversion', label: 'This is not the conversion crate' }
  ];

  const install = `cargo add serde-hl7                                          # serde_hl7::v2 and serde_hl7::v3
cargo add serde-hl7 --no-default-features --features v2      # one standard only
cargo add serde-hl7-v2                                       # or the sub-crate directly`;

  const message = `use serde_hl7::v2::Message;

let text = "MSH|^~\\\\&|LAB|ACME|EHR|CLINIC|20260815081500||ORU^R01^ORU_R01|MSG00042|P|2.5\\r\\
            PID|1||444333222^^^ACME^MR||EVERYWOMAN^EVE^E||19620320|F";

let message = Message::parse(text)?;

let json = serde_json::to_string(&message)?;
let back: Message = serde_json::from_str(&json)?;

assert_eq!(back.to_er7(), text);                    // the ER7 text is unchanged
assert_eq!(back.version(), message.version());      // and so is the release
assert_eq!(back.get("PID-5.1")?.as_deref(), Some("EVERYWOMAN"));`;

  const messageJson = `{
  "version": "2.5",
  "er7": "MSH|^~\\\\&|LAB|ACME|EHR|CLINIC|20260815081500||ORU^R01^ORU_R01|MSG00042|P|2.5\\rPID|1||444333222^^^ACME^MR||EVERYWOMAN^EVE^E||19620320|F"
}`;

  const wrap = `// Already holding an hl7_2::Message? Wrap it; Deref reaches the whole hl7-2 API.
let message = Message(hl7_2::parse(text)?);
let message: Message = hl7_2::parse(text)?.into();
let inner: hl7_2::Message = message.into();`;

  const tree = `use serde_hl7::v2::Node;

let json = serde_json::to_string_pretty(&Node(message.tree()))?;`;

  const treeJson = `{
  "name": "PID.5",
  "path": "PID[1]-5[1]",
  "kind": "Field",
  "text": "EVERYWOMAN^EVE^E",
  "null": false,
  "children": [
    { "name": "XPN.1", "path": "PID[1]-5[1].1", "kind": "Component", "text": "EVERYWOMAN", "null": false, "children": [] },
    { "name": "XPN.2", "path": "PID[1]-5[1].2", "kind": "Component", "text": "EVE", "null": false, "children": [] },
    { "name": "XPN.3", "path": "PID[1]-5[1].3", "kind": "Component", "text": "E", "null": false, "children": [] }
  ]
}`;

  const findings = `use serde_hl7::v2::Diagnostic;

let findings: Vec<Diagnostic> = message.validate().into_iter().map(Diagnostic).collect();
let json = serde_json::to_string_pretty(&findings)?;

let back: Vec<Diagnostic> = serde_json::from_str(&json)?;
assert_eq!(back, findings);`;

  const findingsJson = `[
  {
    "severity": "Warning",
    "kind": "StructureMismatch",
    "path": "",
    "detail": "the standard segments fit structure ORU_R01, but the message also carries local Z-segments, which no structure describes; segments are read flat"
  }
]`;

  const seed = `use std::sync::Arc;
use serde::de::DeserializeSeed;
use serde_hl7::v2::Message;

let dictionary = hl7_2::Dictionary::from_json(
    r#"{"inherits": "2.5", "segments": {"ZPD": ["ST", "XPN"]}}"#,
    "acme",
)?;
let options = hl7_2::Options::new().with_dictionary(Arc::new(dictionary));

let json = r#"{"version":"2.5","er7":"MSH|^~\\\\&|ACME||||1||ADT^A01|1|P|2.5\\rZPD|7|SMITH^JOHN"}"#;
let mut deserializer = serde_json::Deserializer::from_str(json);
let message = Message::seed(&options).deserialize(&mut deserializer)?;

// The vendor's own segment reads through the vendor's dictionary.
assert_eq!(message.tree().find("XPN.2").unwrap().text(), "JOHN");`;

  const v3 = `use serde_hl7::v3::Message;

let message = Message::parse(xml)?;

let json = serde_json::to_string_pretty(&message)?;
let back: Message = serde_json::from_str(&json)?;
assert_eq!(back, message);      // every field of every type is public, so nothing is lost`;

  const v3Json = `{
  "id": { "root": "2.16.840.1.113883.19.5", "extension": "MSG00042" },
  "creationTime": "20260815081500",
  "interactionId": { "root": "2.16.840.1.113883.1.6", "extension": "PRPA_IN201305UV02" },
  "sender": {
    "name": "device",
    "attributes": { "classCode": "DEV", "determinerCode": "INSTANCE" },
    "text": "",
    "children": [ { "name": "id", "attributes": { "root": "2.16.840.1.113883.19.5.1" }, "text": "", "children": [] } ]
  },
  "receiver": { "name": "device", "attributes": { "classCode": "DEV", "determinerCode": "INSTANCE" }, "text": "", "children": [ ... ] },
  "controlAct": {
    "classCode": "CACT",
    "moodCode": "EVN",
    "code": { "code": "PRPA_TE201305UV02", "codeSystem": null, "displayName": null },
    "domain": { "name": "patient", "attributes": { "classCode": "PAT" }, "text": "", "children": [ ... ] }
  }
}`;

  const rim = `use serde_hl7::v3::{Act, Role};

let domain = message
    .control_act
    .as_ref()
    .and_then(|act| act.domain.as_ref())
    .expect("the sample carries a payload");

// The payload's shape is the interaction's business: hl7-3 reads it into
// whichever RIM class you say it is, and the wrapper serializes the typed value.
let patient = Role(hl7_3::rim::Role::from_element(domain));
let json = serde_json::to_string_pretty(&patient)?;
// {
//   "classCode": "PAT",
//   "id": [ { "root": "2.16.840.1.113883.19.5", "extension": "444333222" } ],
//   "code": null,
//   "statusCode": { "code": "active", "codeSystem": null, "displayName": null },
//   "effectiveTime": "20260101"
// }

let back: Role = serde_json::from_str(&json)?;
assert_eq!(back, patient);`;

  const strictV2 = `use serde_hl7::v2::{Message, Strict};

// "verison" — a typo of "version", the one optional key this type has.
let typo = r#"{"verison": "2.3.1", "er7": "MSH|^~\\\\&|LAB|ACME|EHR|CLINIC|20260815||ADT^A01|1|P|2.5"}"#;

let plain: Message = serde_json::from_str(typo)?;             // Ok — read as 2.5, from MSH-12
let strict: Result<Strict<Message>, _> = serde_json::from_str(typo);
assert!(strict.is_err());
// unknown field \`verison\`, expected \`version\` or \`er7\`

let message: Strict<Message> = serde_json::from_str(&typo.replace("verison", "version"))?;
assert_eq!(message.version().to_string(), "2.3.1");           // Deref: the wrapped API is right there`;

  const strictV3 = `use serde_hl7::v3::{Message, Strict};

// A wrong key three levels down, inside the payload's id element.
let fixture = r#"{"controlAct":{"classCode":"CACT","moodCode":"EVN",
  "domain":{"name":"patient","children":[{"name":"id","attrs":{"root":"1.2.3"}}]}}}"#;

let plain: Message = serde_json::from_str(fixture)?;          // Ok — "attrs" ignored, id has no attributes
let strict: Result<Strict<Message>, _> = serde_json::from_str(fixture);
assert!(strict.is_err());
// unknown field \`attrs\`, expected one of \`name\`, \`attributes\`, \`text\`, \`children\``;

  const anyFormat = `// The crates name no format. Pick one in your own Cargo.toml and use it as you would
// for any Serialize type — serde_json here only because JSON is the easiest to read.
let json = serde_json::to_string(&message)?;
let value = serde_json::to_value(&message)?;         // or an in-memory Value
let bytes = serde_json::to_vec(&message)?;           // or bytes for a store`;
</script>

<DocPage
  lede="Put a parsed HL7 v2 message, its dictionary-named tree, its validation findings, or a decoded HL7 v3 interaction through any Serde data format — and get it back. Three small crates, two dependencies each, and the parsing crates themselves never see serde."
  {contents}
>
  <h2 id="three-crates">Three crates, and why they are separate</h2>
  <p>
    <a href="/crates/hl7-2/"><code>hl7-2</code></a> has one dependency and
    <a href="/crates/hl7-3/"><code>hl7-3</code></a> has one, and neither is <code>serde</code> — on
    purpose, because healthcare dependency trees get audited and a parser should not cost every
    caller a framework they may not want. The Serde support is a bridge you add beside them:
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Crate</th><th>Wraps</th><th>Reach it as</th><th>Depends on</th></tr>
      </thead>
      <tbody>
        <tr>
          <td><a href="/crates/serde-hl7-v2/"><code>serde-hl7-v2</code></a></td>
          <td><code>hl7-2</code>: the message, the tree, the findings, the release</td>
          <td><code>serde_hl7::v2</code></td>
          <td><code>serde</code>, <code>hl7-2</code></td>
        </tr>
        <tr>
          <td><a href="/crates/serde-hl7-v3/"><code>serde-hl7-v3</code></a></td>
          <td><code>hl7-3</code>: the envelope, the six RIM classes, the data types, the element tree</td>
          <td><code>serde_hl7::v3</code></td>
          <td><code>serde</code>, <code>hl7-3</code></td>
        </tr>
        <tr>
          <td><a href="/crates/serde-hl7/"><code>serde-hl7</code></a></td>
          <td>the two above, behind <code>v2</code> and <code>v3</code> features, both on by default</td>
          <td>—</td>
          <td>the two above</td>
        </tr>
      </tbody>
    </table>
  </div>
  <CodeSample language="sh" code={install} />
  <p>
    Every type is a same-named wrapper — <code>serde_hl7::v2::Message</code> around
    <code>hl7_2::Message</code> — that <code>Deref</code>s to what it wraps and converts both ways
    with <code>From</code>, so the wrapped crate's whole API is reachable on the wrapper and nothing
    has to be re-learned. Every <code>Serialize</code> and <code>Deserialize</code> impl is written
    by hand against Serde's low-level traits: no derive, no proc-macro crate, and shapes that were
    chosen rather than inherited from a struct layout.
  </p>

  <h2 id="v2-message">A v2 message: ER7 text plus release</h2>
  <CodeSample language="rust" code={message} />
  <p>Which, as JSON, is two keys:</p>
  <CodeSample language="json" code={messageJson} />
  <p>
    <code>er7</code> is the message as ER7 text, which <code>hl7-2</code> already guarantees
    round-trips byte for byte. <code>version</code> is the release the message was <em>read
    as</em> — which a forced <code>Options::version</code>, or a <code>2.5.2</code> resolved to
    <code>2.5.1</code>, may have made different from what MSH-12 literally says — so that survives
    too. Deserializing parses the text again with that version pinned, so the value that comes back
    carries a resolved dictionary exactly as a freshly parsed message would; the dictionary itself
    is never on the wire.
  </p>
  <CodeSample language="rust" caption="Wrapping a message you already have" code={wrap} />
  <Callout heading="Why text and not a tree">
    <p>
      The segment/field/component tree is the ER7 encoding layer's, and that layer already has a
      Serde crate — <a href="https://crates.io/crates/serde-er7"><code>serde-er7</code></a>, over
      the <code>er7</code> types <code>hl7-2</code> exposes as <code>Message::raw()</code>. A second
      copy here would be two crates specifying one wire shape for one set of bytes. What only this
      crate can add, because only <code>hl7-2</code> has the dictionary, is the view below.
    </p>
  </Callout>

  <h2 id="v2-tree">The dictionary-named tree</h2>
  <CodeSample language="rust" code={tree} />
  <p>One object per node, six keys always present. The <code>PID.5</code> node of the message above:</p>
  <CodeSample language="json" code={treeJson} />
  <p>
    <code>name</code> is the dictionary name (<code>PID.5</code>, <code>XPN.1</code>, or the
    message structure at the root); <code>path</code> is the <code>er7</code> path that reads the
    node back, empty at the root; <code>kind</code> is one of <code>Group</code>,
    <code>Segment</code>, <code>Field</code>, <code>Component</code>, <code>Subcomponent</code>;
    <code>null</code> is <code>true</code> only where the sender wrote the explicit HL7 null
    <code>""</code>, kept distinct from an absent value; <code>children</code> is <code>[]</code>
    at a leaf.
  </p>
  <p>
    <code>Node</code> is <code>Serialize</code> only. <code>hl7-2</code> builds a tree only from a
    parsed message and <code>hl7_2::Node</code> has no public constructor, so the only way back to
    a tree is through a <code>Message</code>. A tree is a <em>view</em>; the message is what
    round-trips.
  </p>

  <h2 id="v2-findings">Validation findings</h2>
  <CodeSample language="rust" code={findings} />
  <p>For a message that carries a Z-segment, one warning:</p>
  <CodeSample language="json" code={findingsJson} />
  <p>
    Four keys, all required on the way back in — <code>severity</code> is <code>Error</code> or
    <code>Warning</code>, <code>kind</code> is the variant name — so an API can return what
    <code>validate()</code> found and the other side can read it back as diagnostics rather than
    as prose. Note that <code>detail</code> can quote a value from the message, as
    <a href="/docs/patient-data/">Patient data</a> explains; serializing a finding does not change
    that.
  </p>

  <h2 id="v2-schema-mode">Reading back through your own dictionary</h2>
  <p>
    A plain <code>Deserialize</code> impl is stateless: it has no way to be handed
    <code>hl7_2::Options</code>, so it reads every message through the bundled dictionary for the
    release on the wire. A message that was parsed in <a href="/guides/dictionaries/">schema
    mode</a> comes back through <code>Message::seed</code>, a <code>DeserializeSeed</code> that
    carries the options — a vendor dictionary, a forced release, strict validation — into the
    re-parse.
  </p>
  <CodeSample language="rust" code={seed} />
  <p>
    A <code>version</code> set in the options wins over the <code>version</code> key on the wire,
    which is what that option means for <code>parse_with_options</code>. The seed's
    <code>.strict()</code> makes it reject unknown keys, the same as <code>Strict&lt;Message&gt;</code>
    below.
  </p>

  <h2 id="v3">A v3 interaction</h2>
  <CodeSample language="rust" code={v3} />
  <p>
    Keys are the <code>hl7-3</code> field names in lowerCamelCase — which, wherever a field
    corresponds to an HL7 v3 attribute or element, is that name, so the JSON and the XML
    cross-reference without a table. Every key is serialized every time: an <code>Option</code> is
    <code>null</code> when absent, a <code>Vec</code> is <code>[]</code> when empty. The
    <code>sender</code>, <code>receiver</code>, and domain payload are the raw element tree —
    <code>name</code>, <code>attributes</code> as an object in name order, <code>text</code>,
    <code>children</code> — because <code>hl7-3</code> reads them that way.
  </p>
  <CodeSample language="json" caption="A PRPA_IN201305UV02, abbreviated" code={v3Json} />
  <p>
    On the way back in, only the keys that name a thing are required —
    <code>classCode</code> and <code>moodCode</code> on an act, <code>typeCode</code> on a
    participation, <code>root</code> on an identifier, <code>code</code> on a code, and an
    element's <code>name</code>; everything else defaults, matching <code>hl7-3</code>'s rule that
    a missing wrapper reads as absent rather than failing.
  </p>
  <CodeSample language="rust" caption="The payload as a RIM class rather than raw elements" code={rim} />
  <Callout heading="A value round trip, not an XML one">
    <p>
      Every field of every <code>hl7-3</code> type is public plain data and every one is on the
      wire, so a value through any format and back compares equal. What is not promised is the
      original document: <code>hl7-3</code> has no XML writer and normalizes on the way in, so there
      is no “back to the XML” here to guarantee.
    </p>
  </Callout>

  <h2 id="strict">Strict mode</h2>
  <p>
    Deserializing ignores a key it does not recognize, by default, matching both wrapped crates'
    own read-what-you-can behavior. That is right for reading what a sender sent and exactly the
    situation in which a typo in a hand-written fixture is silent. <code>Strict&lt;T&gt;</code> is
    the opt-in: a separate type, chosen at the one call site that wants it, never a global flag.
  </p>
  <CodeSample language="rust" caption="v2: the one optional key" code={strictV2} />
  <CodeSample language="rust" caption="v3: nested to any depth" code={strictV3} />
  <p>
    In v3 the check reaches every object type at every depth — identifiers, codes, and every
    element of the payload tree — but leaves an element's <code>attributes</code> map alone:
    attribute names are the document's data, and no crate can know which ones an element should
    have.
  </p>

  <h2 id="any-format">Any format, not just JSON</h2>
  <CodeSample language="rust" code={anyFormat} />
  <p>
    <code>serde_json</code> is a dev-dependency of these crates and nothing more; their library
    code names no format. The examples on this page use it because JSON is the easiest to read on a
    page, and the same calls work with a YAML crate, <code>bincode</code>, or any other Serde data
    format you add to your own manifest. Because every shape is chosen — a string for a release, an
    object for a node, a bare code for a <code>nullFlavor</code> — nothing depends on a feature
    only one format has.
  </p>

  <h2 id="not-conversion">This is not the conversion crate</h2>
  <p>
    <a href="/guides/converting/">Converting formats</a> is about a different question. The
    <a href="/crates/hl7-2-from-er7-into-json/"><code>hl7-2-from-er7-into-json</code></a> document —
    one key per field, named by data type, real arrays for repetitions — is a fixed mapping for a
    consumer that is not Rust, with a reverse crate to bring it back. The Serde shapes here are for
    Rust code that already holds a parsed value and needs it to pass through a document store, a
    web framework's response type, a structured logger, or a snapshot test. Same message, two
    shapes for two consumers; neither is a substitute for the other, and neither crate depends on
    the other.
  </p>
  <p>
    Each crate's normative <code>spec/index.md</code> —
    <a href="https://github.com/hl7-rust/hl7-rust/blob/main/serde-hl7-v2/spec/index.md">v2</a>,
    <a href="https://github.com/hl7-rust/hl7-rust/blob/main/serde-hl7-v3/spec/index.md">v3</a> —
    states every wire shape and every rule, with a coverage table <code>cargo test</code> checks.
    Runnable programs are in each crate's <code>examples/</code>; see
    <a href="/examples/#serde">Examples</a> for the commands.
  </p>
</DocPage>
