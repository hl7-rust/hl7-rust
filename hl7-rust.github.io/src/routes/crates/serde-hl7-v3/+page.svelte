<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('serde-hl7-v3');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'message', label: 'The message' },
    { id: 'rim', label: 'RIM classes and data types' },
    { id: 'element', label: 'The element tree' },
    { id: 'strict', label: 'Strict deserialization' },
    { id: 'not-doing', label: 'What this crate does not do' },
    { id: 'related', label: 'Related crates' }
  ];

  const message = `use serde_hl7_v3::Message;

let message = Message::parse(xml)?;

let json = serde_json::to_string_pretty(&message)?;
// {
//   "id": {"root": "2.16.840.1.113883.19.5", "extension": "MSG00001"},
//   "creationTime": "20260101120000",
//   "interactionId": {"root": "2.16.840.1.113883.1.6", "extension": "QUQI_IN000001UV01"},
//   "sender": {"name": "device", "attributes": {"classCode": "DEV", ...}, "text": "", "children": []},
//   "receiver": ...,
//   "controlAct": {"classCode": "CACT", "moodCode": "EVN", "code": {...}, "domain": {...}}
// }

let back: Message = serde_json::from_str(&json)?;
assert_eq!(back, message);`;

  const rim = `use serde_hl7_v3::{Act, Role};

let domain = message.control_act.as_ref()?.domain.as_ref()?;
let patient = Role(hl7_3::rim::Role::from_element(domain));

let json = serde_json::to_string(&patient)?;
// {"classCode":"PAT","id":[{"root":"2.16.840.1.113883.19.5","extension":"444333222"}],
//  "code":null,"statusCode":{"code":"active","codeSystem":null,"displayName":null},
//  "effectiveTime":"20260101"}`;

  const element = `use serde_hl7_v3::Element;

let element = Element(hl7_3::xml::parse(r#"<id root="1.2.3" extension="7"/>"#)?);
let json = serde_json::to_string(&element)?;
// {"name":"id","attributes":{"extension":"7","root":"1.2.3"},"text":"","children":[]}

let back: Element = serde_json::from_str(r#"{"name":"device"}"#)?;   // the rest defaults`;

  const strict = `use serde_hl7_v3::{Message, Strict};

// "extention" — three levels down, inside the payload's id element.
let fixture = r#"{"controlAct":{"classCode":"CACT","moodCode":"EVN",
  "domain":{"name":"patient","children":[{"name":"id","attrs":{"root":"1.2.3"}}]}}}"#;

let plain: Message = serde_json::from_str(fixture)?;       // Ok — "attrs" ignored
let strict: Result<Strict<Message>, _> = serde_json::from_str(fixture);
assert!(strict.is_err());                                   // "unknown field \`attrs\`"`;

  const install = `cargo add serde-hl7-v3`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <CodeSample language="sh" code={install} />
  <p>
    HL7 v3's own serialization is XML, and <code>hl7-3</code> deliberately reads it without writing
    it back — so once an interaction is decoded, the value has no way out except code you write.
    This crate is that way out, for any Serde format. Most callers reach it as
    <code>serde_hl7::v3</code> through <a href="/crates/serde-hl7/"><code>serde-hl7</code></a>.
  </p>

  <h2 id="message">The message</h2>
  <CodeSample language="rust" code={message} />
  <p>
    Keys are the <code>hl7-3</code> field names in lowerCamelCase — which, wherever a field
    corresponds to an HL7 v3 attribute or element, is that name: <code>classCode</code>,
    <code>codeSystem</code>, <code>interactionId</code>. Every key is serialized every time, so the
    schema a consumer sees is fixed; on the way back in, only the keys that name a thing are
    required and everything else defaults, matching <code>hl7-3</code>'s rule that a missing
    wrapper reads as absent rather than failing.
  </p>
  <Callout type="note" heading="A value round trip, not an XML one">
    <p>
      Every field of every type is public plain data and every one is on the wire, so a value
      through any format and back compares equal. What is not promised is the original document:
      <code>hl7-3</code> has no XML writer, and its reader normalizes on the way in.
    </p>
  </Callout>

  <h2 id="rim">RIM classes and data types</h2>
  <CodeSample language="rust" code={rim} />
  <p>
    The payload's shape is the interaction's business, not this crate's: <code>hl7-3</code> reads it
    into whichever RIM class you say it is, and the wrapper serializes the typed value. All six
    backbone classes and the six data types — <code>Ii</code>, <code>Cd</code>, <code>Ivl</code>,
    <code>Pq</code>, <code>Ed</code>, and <code>NullFlavor</code> as its bare code — have one. The
    fourteen object types are written by one <code>macro_rules!</code>, so their shapes and their
    strictness cannot drift; that is a local macro, not a derive, and adds no proc-macro dependency.
  </p>

  <h2 id="element">The element tree</h2>
  <CodeSample language="rust" code={element} />
  <p>
    The raw tree beneath everything: name, attributes as an object in name order, text, children.
    This is what <code>sender</code>, <code>receiver</code>, and the domain payload serialize as,
    and it round-trips exactly.
  </p>

  <h2 id="strict">Strict deserialization</h2>
  <CodeSample language="rust" code={strict} />
  <p>
    Almost every key is optional, which is right for reading what a sender sent and exactly the
    situation in which a typo in a fixture is silent. <code>Strict&lt;T&gt;</code> exists for every
    object type and nests to any depth — into identifiers, codes, and every element of the payload
    tree. It leaves <code>attributes</code> alone: attribute names are the document's data, and no
    crate can know which ones an element should have.
  </p>

  <h2 id="not-doing">What this crate does not do</h2>
  <p>
    It does not write XML, pick a wire format, validate codes against a vocabulary domain, or give
    struct mode's <code>FromElement</code> types Serde. Every rule is in the crate's
    <a href="https://github.com/hl7-rust/hl7-rust/blob/main/serde-hl7-v3/spec/index.md">spec</a>,
    with a coverage table <code>cargo test</code> checks.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
