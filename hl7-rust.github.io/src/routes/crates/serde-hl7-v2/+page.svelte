<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('serde-hl7-v2');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'message', label: 'The message' },
    { id: 'tree', label: 'The tree' },
    { id: 'diagnostics', label: 'Validation findings' },
    { id: 'schema-mode', label: 'Schema mode through Serde' },
    { id: 'strict', label: 'Strict deserialization' },
    { id: 'not-doing', label: 'What this crate does not do' },
    { id: 'related', label: 'Related crates' }
  ];

  const message = `use serde_hl7_v2::Message;

let message = Message::parse(text)?;

let json = serde_json::to_string(&message)?;
// {"version":"2.5","er7":"MSH|^~\\\\&|LAB|...\\rPID|1||..."}

let back: Message = serde_json::from_str(&json)?;
assert_eq!(back.to_er7(), text);
assert_eq!(back.version(), message.version());`;

  const tree = `use serde_hl7_v2::Node;

let json = serde_json::to_value(Node(message.tree()))?;
// {"name":"ORU_R01","path":"","kind":"Group","text":"...","null":false,"children":[
//   {"name":"MSH","path":"MSH[1]","kind":"Segment", ...},
//   {"name":"PID","path":"PID[1]","kind":"Segment","children":[
//     {"name":"PID.5","path":"PID[1]-5[1]","kind":"Field","text":"SMITH^JOHN","children":[
//       {"name":"XPN.1","path":"PID[1]-5[1].1","kind":"Component","text":"SMITH", ...},
//       ...`;

  const diagnostics = `use serde_hl7_v2::Diagnostic;

let findings: Vec<Diagnostic> = message.validate().into_iter().map(Diagnostic).collect();
let json = serde_json::to_string(&findings)?;
// [{"severity":"Warning","kind":"SegmentUnknown","path":"ZPD","detail":"..."}]

let back: Vec<Diagnostic> = serde_json::from_str(&json)?;
assert_eq!(back, findings);`;

  const seed = `use serde::de::DeserializeSeed;
use serde_hl7_v2::Message;

let options = hl7_2::Options::new().with_dictionary(Arc::new(vendor_dictionary));
let mut deserializer = serde_json::Deserializer::from_str(json);
let message = Message::seed(&options).deserialize(&mut deserializer)?;
// read through the vendor's dictionary, as parse_with_options would`;

  const strict = `use serde_hl7_v2::{Message, Strict};

let typo = r#"{"verison":"2.3.1","er7":"MSH|^~\\\\&|LAB|...|P|2.5"}"#;

let plain: Message = serde_json::from_str(typo)?;      // Ok — reads 2.5 from MSH-12
let strict: Result<Strict<Message>, _> = serde_json::from_str(typo);
assert!(strict.is_err());                               // "unknown field \`verison\`"`;

  const install = `cargo add serde-hl7-v2`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <CodeSample language="sh" code={install} />
  <p>
    <code>hl7-2</code> has one dependency and no Serde support, on purpose. This crate is the bridge:
    add it when a message, its tree, or its findings needs to reach something that only speaks
    Serde — a document store, a web framework's response type, a structured logger, a snapshot
    test — and leave it out otherwise. Most callers reach it as <code>serde_hl7::v2</code> through
    <a href="/crates/serde-hl7/"><code>serde-hl7</code></a>.
  </p>

  <h2 id="message">The message</h2>
  <CodeSample language="rust" code={message} />
  <p>
    Two keys. <code>er7</code> is the message as ER7 text, which round-trips byte for byte;
    <code>version</code> is the release it was <em>read as</em> — which a forced
    <code>Options::version</code> or a <code>2.5.2</code> resolved to <code>2.5.1</code> may have
    made different from what MSH-12 literally says — so <code>message.version()</code> survives too.
    Deserializing parses the text again with the version pinned, so the dictionary is resolved on
    the way in, never shipped.
  </p>
  <Callout type="note" heading="Why text, not a tree">
    <p>
      The segment/field/component tree already has a Serde crate:
      <a href="https://crates.io/crates/serde-er7"><code>serde-er7</code></a>, over the
      <code>er7</code> types <code>hl7-2</code> exposes as <code>Message::raw()</code>. Repeating it
      here would be two crates specifying one wire shape for one set of bytes. What only this
      crate can add — because only <code>hl7-2</code> has the dictionary — is the tree below.
    </p>
  </Callout>

  <h2 id="tree">The tree</h2>
  <CodeSample language="rust" code={tree} />
  <p>
    One object per node, six keys always present: <code>name</code> (<code>PID.5</code>,
    <code>XPN.1</code>), <code>path</code> (the <code>er7</code> path that reads it back),
    <code>kind</code>, <code>text</code>, <code>null</code> (the explicit HL7 null, kept distinct
    from absent), and <code>children</code>. <code>Node</code> is <code>Serialize</code> only: a tree
    is a view of a message, <code>hl7-2</code> builds one only from a parsed message, and the message
    is what round-trips.
  </p>

  <h2 id="diagnostics">Validation findings</h2>
  <CodeSample language="rust" code={diagnostics} />
  <p>
    Four keys, all required on the way back in, so an API can return what <code>validate()</code>
    found and the other side can read it back as diagnostics rather than as prose.
  </p>

  <h2 id="schema-mode">Schema mode through Serde</h2>
  <CodeSample language="rust" code={seed} />
  <p>
    A stateless <code>Deserialize</code> has no way to be handed <code>hl7_2::Options</code>, so
    the plain impl reads every message through the bundled dictionary for its release.
    <code>Message::seed</code> is a <code>DeserializeSeed</code> that carries the caller's options
    — a vendor dictionary, a forced release, strict validation — into the re-parse.
  </p>

  <h2 id="strict">Strict deserialization</h2>
  <CodeSample language="rust" code={strict} />
  <p>
    Unknown keys are ignored by default, matching <code>hl7-2</code>'s own fallback-first reading.
    <code>Strict&lt;T&gt;</code> is the opt-in for the one case that hides — a typo on the optional
    <code>version</code> key silently falls back to MSH-12 — and it is a separate type, requested
    at the one call site that wants it, never a global flag.
  </p>

  <h2 id="not-doing">What this crate does not do</h2>
  <p>
    It does not pick a wire format (<code>serde_json</code> is a dev-dependency only), define a
    second tree shape for ER7, serialize dictionaries (they are JSON already, in the shape
    <code>Dictionary::from_json</code> reads), or give struct mode's <code>FromHl7</code> types Serde
    — your struct is yours to derive for. Every rule is in the crate's
    <a href="https://github.com/hl7-rust/hl7-rust/blob/main/serde-hl7-v2/spec/index.md">spec</a>,
    with a coverage table <code>cargo test</code> checks.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
