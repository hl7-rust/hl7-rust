<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { postBySlug, formatDate } from '$lib/data/news';
  import { REPO } from '$lib/data/crates';

  const post = postBySlug('serde-support-for-hl7');

  const contents = [
    { id: 'what', label: 'What was released' },
    { id: 'why-separate', label: 'Why three crates, and not a feature' },
    { id: 'shapes', label: 'The shapes, in one paragraph each' },
    { id: 'strict', label: 'Strict mode, for fixtures' },
    { id: 'unchanged', label: 'What did not change' },
    { id: 'next', label: 'What is deferred' }
  ];

  const install = `cargo add serde-hl7          # serde_hl7::v2 and serde_hl7::v3, both on by default`;

  const roundTrip = `use serde_hl7::v2::Message;

let message = Message::parse(text)?;
let json = serde_json::to_string(&message)?;      // {"version":"2.5","er7":"MSH|^~\\\\&|..."}
let back: Message = serde_json::from_str(&json)?;
assert_eq!(back.to_er7(), text);`;
</script>

<DocPage
  eyebrow={formatDate(post.date)}
  lede={post.summary}
  {contents}
>
  <h2 id="what">What was released</h2>
  <p>
    The workspace's seventh release adds three crates, each at 0.1.0, and changes nothing already
    published: <code>hl7-2</code> 0.3.0 and <code>hl7-3</code> 0.2.0 are the versions the new
    crates depend on, and no other crate's manifest was touched.
  </p>
  <ul>
    <li>
      <a href="/crates/serde-hl7-v2/"><code>serde-hl7-v2</code></a> — Serde for
      <code>hl7-2</code>: the message, its dictionary-named tree, its validation findings, and the
      release it was read as.
    </li>
    <li>
      <a href="/crates/serde-hl7-v3/"><code>serde-hl7-v3</code></a> — Serde for
      <code>hl7-3</code>: the three-level envelope, the six RIM backbone classes, the data types,
      and the raw XML element tree beneath them.
    </li>
    <li>
      <a href="/crates/serde-hl7/"><code>serde-hl7</code></a> — the umbrella:
      <code>serde_hl7::v2</code> and <code>serde_hl7::v3</code>, each behind a feature of the same
      name, both on by default. The shape of the <code>hl7</code> crate, one layer up.
    </li>
  </ul>
  <CodeSample language="sh" code={install} />
  <CodeSample language="rust" caption="The flagship path" code={roundTrip} />
  <p>
    The full walkthrough is <a href="/guides/serde/">Serde: JSON, YAML, and any format</a>; the
    dated entry with the versions is in
    <a href={`${REPO}/blob/main/CHANGELOG.md`}><code>CHANGELOG.md</code></a>.
  </p>

  <h2 id="why-separate">Why three crates, and not a feature</h2>
  <p>
    The question that started this, asked on the day of the release, was whether code should be
    <em>extracted</em> from the workspace into a crate called <code>serde-hl7</code>, to make it
    clearer which code does serialization. The evaluation found nothing to extract: no crate here
    used Serde, and that was a stated property rather than an accident —
    <a href="/docs/patient-data/">Patient data</a> lists “no serialization framework” among the
    things a security review can grep for. So the brief changed: build a real
    <code>serde-hl7</code>, the way <code>serde-er7</code> had been built over <code>er7</code>, and
    publish it. The reasoning is recorded in
    <a href={`${REPO}/blob/main/serde-hl7/plan.md`}><code>serde-hl7/plan.md</code></a>.
  </p>
  <p>
    A feature flag on <code>hl7-2</code> would have put <code>serde</code> in the dependency tree
    of every caller who enabled it, and in the manifest every caller has to read. A separate crate
    keeps the fourteen existing crates' claim exactly as it was: <code>serde</code> reaches only the
    callers who add the bridge, and each bridge depends on exactly two things — <code>serde</code>
    and the crate it wraps.
  </p>

  <h2 id="shapes">The shapes, in one paragraph each</h2>
  <p>
    <strong>A v2 message is its ER7 text plus its release.</strong> Two keys,
    <code>version</code> and <code>er7</code>. Deserializing parses the text again with the version
    pinned, so the dictionary is resolved on the way in and never shipped. The segment/field tree
    is deliberately <em>not</em> repeated here: the encoding layer already has
    <code>serde-er7</code>, and two crates specifying one wire shape for one set of bytes is one
    too many. What only this crate can add — because only <code>hl7-2</code> has the dictionary —
    is the tree with <code>PID.5</code> and <code>XPN.1</code> for names: one object per node, six
    keys, <code>Serialize</code> only, because <code>hl7-2</code> builds a tree only from a parsed
    message. Findings serialize as four keys and round-trip. A message parsed through a vendor's
    own dictionary comes back through <code>Message::seed(&amp;options)</code>, a
    <code>DeserializeSeed</code> carrying what a stateless <code>Deserialize</code> cannot.
  </p>
  <p>
    <strong>A v3 value is an object whose keys are HL7® v3's own names.</strong>
    <code>classCode</code>, <code>codeSystem</code>, <code>interactionId</code>: the lowerCamelCase
    field name, which is the XML attribute or element name wherever a field has one, so JSON and
    XML cross-reference without a table. Every key is serialized every time; on the way back in,
    only the keys that name a thing are required. The fourteen object types are written by one
    <code>macro_rules!</code> so their shapes cannot drift — a local macro, not a derive, so still
    no proc-macro dependency. What is promised is a value round trip: every field is public plain
    data, so a value through any format and back compares equal. What is not promised is the
    original XML, because <code>hl7-3</code> has no XML writer.
  </p>

  <h2 id="strict">Strict mode, for fixtures</h2>
  <p>
    Both crates ignore an unknown key by default, which is right for reading what a sender sent
    and wrong for exactly one case: a typo in a hand-written test fixture, which is then read as if
    the key had never been written. <code>Strict&lt;T&gt;</code> is the opt-in that turns the typo
    into an error — a separate type at the one call site that wants it, and in v3 nested to any
    depth. Each crate ships a runnable example of the typo being caught, next to its round trip.
  </p>

  <h2 id="unchanged">What did not change</h2>
  <Callout heading="The parsing crates still do not depend on serde">
    <p>
      <code>hl7-2</code> still has one dependency and <code>hl7-3</code> still has one. The
      “no serialization framework” row in <code>spec/phi/index.md</code> now names the three bridge
      crates as the opt-in exception and is otherwise unchanged. A caller who never adds them never
      carries <code>serde</code>.
    </p>
  </Callout>
  <p>
    Two other things held. The workspace's release runbook says the first release of a brand-new
    crate stays the maintainer's call, and this one was directed by the maintainer explicitly. And
    the root plan's “no new crates” non-goal was overridden for these three and reworded rather than
    deleted, so the override is on the record.
  </p>

  <h2 id="next">What is deferred</h2>
  <ul>
    <li>
      <strong><code>Deserialize</code> for the v2 tree.</strong> It needs a public constructor on
      <code>hl7_2::Node</code>, which is <code>hl7-2</code>'s decision to make, not this crate's.
    </li>
    <li>
      <strong>Serde for struct mode.</strong> A <code>FromHl7</code> or <code>FromElement</code>
      struct is yours; deriving Serde for it is one line in your own crate and needs nothing from
      these.
    </li>
    <li>
      <strong>Dictionaries.</strong> They are JSON already, in the shape
      <code>Dictionary::from_json</code> reads. A second shape would be a second thing to keep in
      step.
    </li>
  </ul>
  <p>
    Each is stated, with its reasoning, in the roadmap section of the crate's own
    <code>spec/index.md</code>. If a wire shape on this page does not match what the crate emits,
    the spec is right, and the difference is a bug worth filing.
  </p>
</DocPage>
