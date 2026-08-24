<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-xml-lite-helper');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'for', label: 'What it is for — and what it is not' },
    { id: 'prefixes', label: 'Namespace prefixes are ignored, not resolved' },
    { id: 'finding', label: 'Finding things' },
    { id: 'skipped', label: 'What is skipped, and what is not' },
    { id: 'writing', label: 'Writing' },
    { id: 'dependencies', label: 'Dependencies' },
    { id: 'related', label: 'Related crates' }
  ];

  const basic = `let xml = r#"<order id="7"><item qty="2">widget</item></order>"#;
let root = hl7_2_xml_lite_helper::parse(xml)?;

assert_eq!(root.attribute("id"), Some("7"));
assert_eq!(root.child("item").unwrap().text, "widget");`;

  const finding = `let root = hl7_2_xml_lite_helper::parse(
    "<PID><PID.3><CX.1>a</CX.1></PID.3><PID.3><CX.4><HD.1>NHS</HD.1></CX.4></PID.3></PID>",
)?;

// A path down to the first non-blank value, following *every* branch —
// a repeating field puts several elements of the same name side by side.
assert_eq!(root.text_at(&["PID.3", "CX.4", "HD.1"]), Some("NHS"));

// Or walk it yourself.
assert_eq!(root.child("PID.3").unwrap().child("CX.1").unwrap().text, "a");
assert_eq!(root.children_named("PID.3").count(), 2);
assert_eq!(root.find("HD.1").unwrap().text, "NHS");   // first descendant`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    It reads the subset that carries meaning in a data document — elements, attributes, text, and
    nesting — and skips the rest. No validation, no schema, no DTD, no namespace resolution, no
    streaming.
  </p>
  <CodeSample language="rust" code={basic} />
  <p>
    The name says what it is for. Nothing in the code is HL7-specific, but the crate is scoped to
    serve <a href="/crates/hl7-2-soap/"><code>hl7-2-soap</code></a>,
    <a href="/crates/hl7-3-soap/"><code>hl7-3-soap</code></a>,
    <a href="/crates/hl7-2-from-xml-into-er7/"><code>hl7-2-from-xml-into-er7</code></a>,
    <a href="/crates/hl7-2-from-xsd-into-json-dictionary/"
      ><code>hl7-2-from-xsd-into-json-dictionary</code></a
    >, and <a href="/crates/hl7-3/"><code>hl7-3</code></a> — and its trade-offs are chosen for the
    documents those read.
  </p>

  <h2 id="for">What it is for — and what it is not</h2>
  <p>
    <strong>For:</strong> reading a document produced by a system you are talking to, where you know
    which elements you want and simply need them out — a SOAP envelope, an XML Schema, an HL7 v2.xml
    message. It exists because three crates in this family had each written their own version of
    exactly this, and three copies of a parser is three places for a bug.
  </p>
  <Callout type="warning" heading="Not for untrusted or genuinely unknown documents">
    <p>
      And not for anyone outside this family who wants a small XML reader. Use
      <code>quick-xml</code> or <code>roxmltree</code> there — they are better at it, and they are
      maintained for that purpose. This crate does not claim a general-purpose name, and it should not
      be given a general-purpose job.
    </p>
  </Callout>

  <h2 id="prefixes">Namespace prefixes are ignored, not resolved</h2>
  <p>
    This is the single most important thing to understand about this crate. Elements and attributes
    are matched on their <strong>local name</strong>, so <code>soapenv:Body</code>,
    <code>soap:Body</code>, <code>SOAP-ENV:Body</code> and <code>Body</code> are the same element.
  </p>
  <p>
    It is a deliberate trade. The prefix is chosen by whoever serialized the document, and code that
    insists on one prefix rejects valid documents from every other tool — which is the single most
    common way a working SOAP integration breaks when the other end changes stack.
  </p>
  <p>
    The cost is that a document relying on the distinction between two namespaces that share local
    names will be misread. Reach for a namespace-aware parser there.
  </p>

  <h2 id="finding">Finding things</h2>
  <CodeSample language="rust" code={finding} />
  <p>
    <code>text_at</code> follows <em>every</em> branch down to the first non-blank value, which
    matters because a repeating field puts several elements of the same name side by side — walking
    only the first would silently miss the value.
  </p>

  <h2 id="skipped">What is skipped, and what is not</h2>
  <p>
    <strong>Skipped:</strong> the XML declaration, comments, processing instructions, and a
    <code>DOCTYPE</code>, wherever they appear.
  </p>
  <p>
    <strong>Kept:</strong> CDATA <em>content</em>, as text. Entities — the five predefined ones and
    numeric character references — decode; anything else that looks like an entity is kept literally
    rather than rejected.
  </p>
  <p>
    Whitespace-only text beside child elements is dropped, because it is indentation. Text in a leaf
    is kept exactly as it arrived, because a leading or trailing space can be part of a value.
  </p>

  <h2 id="writing">Writing</h2>
  <p>
    <code>escape</code> covers all five predefined entities, so a value is safe in element content or
    in an attribute. That is the whole of the writing surface — this crate reads.
  </p>

  <h2 id="dependencies">Dependencies</h2>
  <p>
    None, and staying that way: a crate whose whole argument is that it is small enough to read
    cannot have dependencies you also have to read.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
