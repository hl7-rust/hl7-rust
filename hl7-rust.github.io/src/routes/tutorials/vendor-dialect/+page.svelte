<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'situation', label: 'The situation' },
    { id: 'stage-1', label: 'Stage 1 — look at it' },
    { id: 'stage-2', label: 'Stage 2 — write down what you learned' },
    { id: 'stage-3', label: 'Stage 3 — freeze what is stable' },
    { id: 'xsds', label: 'If the site has XSDs instead' },
    { id: 'never-final', label: 'Stage 3 is never final' },
    { id: 'checklist', label: 'Checklist' }
  ];

  const paths = `$ hl7-v2 --paths samples/vendor.hl7
ADT_A01
  MSH  [MSH[1]]
    ...
  ZAC  [ZAC[1]]
    ZAC.1 = 7           [ZAC[1]-1[1]]
    ZAC.2               [ZAC[1]-2[1]]
      ZAC.2.1 = SMITH   [ZAC[1]-2[1].1]
      ZAC.2.2 = JOHN    [ZAC[1]-2[1].2]
    ZAC.3 = 20260814    [ZAC[1]-3[1]]`;

  const query = `$ hl7-v2 --query 'ZAC-2.1' samples/vendor.hl7
SMITH
JONES`;

  const check = `$ hl7-v2 --check samples/vendor.hl7
warning: ZAC[1]: segment is not defined in this dictionary
ok`;

  const dictionary = `{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}`;

  const applied = `$ hl7-v2 --dictionary samples/acme.json --flat samples/vendor.hl7 | grep -A 2 'ZAC.2$'
    ZAC.2
      XPN.1 = SMITH
      XPN.2 = JOHN`;

  const loading = `use std::sync::Arc;

let text = std::fs::read_to_string("acme.json")?;
let dictionary = hl7_2::Dictionary::from_json(&text, "acme")?;
let options = hl7_2::Options::new().with_dictionary(Arc::new(dictionary));

let message = hl7_2::parse_with_options(&message_text, &options)?;`;

  const structs = `use hl7_2::{FromHl7, Raw};

#[derive(FromHl7)]
struct AcmeAdmission {
    #[hl7("PID-3.1")]  patient_id: String,
    #[hl7("ZAC-2.1")]  clinician_family: String,
    #[hl7("ZAC-3")]    effective: Option<String>,
    #[hl7(raw)]        raw: Raw,
}

let admission: AcmeAdmission = hl7_2::parse(text)?.decode()?;`;

  const escape = `// A field nobody warned you about arrived. You are back at stage 1,
// on the same object, with no re-parse and no rewrite.
if let Some(value) = admission.raw.get("ZAC-4")? {
    log::warn!("unmodelled ZAC-4: {value}");
}`;

  const xsd = `hl7-2-from-xsd-into-json-dictionary schemas/acme \\
    --name acme \\
    --alias ADT_A28=ADT_A05 \\
    --inherits 2.5 \\
    -o acme.json`;
</script>

<DocPage
  eyebrow="Tutorial"
  lede="A vendor sends a message with segments the standard has never heard of. This is the common case, not the exception — and the way through it is three stages, in order, each of which leaves you able to ship."
  {contents}
>
  <h2 id="situation">The situation</h2>
  <p>
    A new sender is going live in three weeks. They have sent one sample file. It is an
    <code>ADT^A01</code> with a <code>ZAC</code> segment nobody at either organization can fully
    explain, and their integration engineer is on leave.
  </p>
  <p>
    The instinct is to ask for a specification and wait. The better move is to look at what you have
    — because a message that will not be rejected can be explored, and these crates are built so
    that it never is.
  </p>

  <h2 id="stage-1">Stage 1 — look at it</h2>
  <p>Start with the tool, not with code. No project, no dependencies, no build.</p>
  <CodeSample language="sh" code={paths} />
  <p>Two things already happened without you doing anything:</p>
  <ul>
    <li>
      Everything standard is <strong>already named</strong>. <code>PID.5</code> broke into
      <code>XPN.1</code> and <code>XPN.2</code> because the dictionary knows what
      <code>PID-5</code> holds.
    </li>
    <li>
      The vendor's own <code>ZAC</code> is <strong>there, positionally</strong>, nothing lost.
      Unknown does not mean dropped.
    </li>
  </ul>
  <p>
    The bracketed paths are not decoration: each one is exactly what reads that value back. Pull one
    out and see what you get across the whole sample file:
  </p>
  <CodeSample language="sh" code={query} />
  <p>
    <code>ZAC.2</code> is clearly a person's name — family in component 1, given in component 2.
    That is a hypothesis you can now test against every message in the sample, which is more than
    the missing specification would have told you.
  </p>
  <p>Check what the dictionary thinks, too:</p>
  <CodeSample language="sh" code={check} />
  <Callout heading="A warning, not an error">
    <p>
      An unknown segment is the dictionary not covering the message, not the message being wrong.
      That is why it is a warning, and why a message with a Z-segment still passes
      <code>--check</code>. See <a href="/guides/validating/">Validating</a>.
    </p>
  </Callout>

  <h2 id="stage-2">Stage 2 — write down what you learned</h2>
  <p>
    You concluded that <code>ZAC-2</code> is an <code>XPN</code>. Say so — in JSON, without touching
    any code.
  </p>
  <CodeSample language="json" caption="acme.json" code={dictionary} />
  <CodeSample language="sh" code={applied} />
  <p>
    <code>ZAC.2</code> now reads as an <code>XPN</code> like any standard name field. And when the
    vendor adds <code>ZAC-4</code> next quarter, it is one line in a file rather than a release of
    your software.
  </p>
  <p>Load the same file from Rust:</p>
  <CodeSample language="rust" code={loading} />
  <Callout heading="Two vocabularies, again">
    <p>
      <code>XPN.1</code> is a <em>node name</em> — what the tree, and the XML and JSON siblings, call
      that component. <code>ZAC-2.1</code> is a <em>path</em> — what <code>--query</code>,
      <code>get</code>, and <code>set</code> take. Names describe; paths address. Confusing them is
      the most common stumble at this stage.
    </p>
  </Callout>

  <h2 id="stage-3">Stage 3 — freeze what is stable</h2>
  <p>
    Once the interface has held still long enough to trust, move it into the type system and let the
    compiler carry it.
  </p>
  <CodeSample language="rust" code={structs} />
  <p>
    A typo in <code>"ZAC-2.1"</code> is now a build failure. The mapping lives in one place instead
    of being scattered across every call site. This is the payoff for the first two stages.
  </p>

  <h2 id="xsds">If the site has XSDs instead</h2>
  <p>
    Some sites do not have a specification but do have XML Schemas — HL7's published v2.xml schemas,
    or a vendor's customised copy. Skip stage 2's hand-writing and generate the dictionary:
  </p>
  <CodeSample language="sh" code={xsd} />
  <p>
    A generated dictionary carries <strong>cardinality</strong> as well as data types, because the
    schemas know each field's <code>minOccurs</code> and <code>maxOccurs</code>. That is what lets
    the XML converter run in <code>--schema-shape</code> mode and emit a document which validates
    against the very schemas it came from. See
    <a href="/guides/dictionaries/#from-xsd">Vendor dictionaries</a>.
  </p>
  <p>
    One thing the schemas cannot tell you: which trigger events arrive carried by another message's
    structure. A directory holds <code>ADT_A05.xsd</code> but never says an <code>ADT^A28</code> is
    one — hence <code>--alias</code>.
  </p>

  <h2 id="never-final">Stage 3 is never final</h2>
  <p>
    Keep the <code>#[hl7(raw)]</code> field. It is the whole reason this workflow closes rather than
    dead-ends.
  </p>
  <CodeSample language="rust" code={escape} />
  <p>
    Real feeds are stable until they are not, and the usual choice at that moment is to re-parse the
    raw message or rewrite the library. A <code>Raw</code> field keeps the whole parsed message
    beside the typed data, so the day something arrives that the struct does not model, you are back
    at stage 1 on the same object — no second parse, no redeployment.
  </p>

  <h2 id="checklist">Checklist</h2>
  <ol>
    <li>
      <strong>Look.</strong> <code>hl7-v2 --paths sample.hl7</code>. Confirm nothing is dropped, and
      collect paths.
    </li>
    <li>
      <strong>Probe.</strong> <code>hl7-v2 --query</code> across every sample you have, to test each
      hypothesis about what a field means.
    </li>
    <li>
      <strong>Check.</strong> <code>hl7-v2 --check</code>. Warnings tell you where the dictionary
      needs extending.
    </li>
    <li>
      <strong>Write.</strong> A JSON dictionary inheriting a bundled release, stating only the
      dialect — or generate one from XSDs.
    </li>
    <li>
      <strong>Apply.</strong> <code>Options::with_dictionary</code>, and nothing else in your code
      changes.
    </li>
    <li><strong>Freeze.</strong> A struct, once the shape has held still.</li>
    <li><strong>Keep the escape hatch.</strong> <code>#[hl7(raw)]</code>, always.</li>
  </ol>
</DocPage>
