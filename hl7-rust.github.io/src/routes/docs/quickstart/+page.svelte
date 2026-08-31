<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'the-message', label: 'The message' },
    { id: 'look-at-it', label: '1. Look at it, without writing code' },
    { id: 'parse', label: '2. Parse it' },
    { id: 'read', label: '3. Read values out of it' },
    { id: 'check', label: '4. Check it' },
    { id: 'answer', label: '5. Answer it' },
    { id: 'convert', label: '6. Convert it' },
    { id: 'next', label: 'Where to go next' }
  ];

  const message = `MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5
PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E||19620320|F|||2222 HOME ST^^ANN ARBOR^MI^48104
PV1|1|O|OP^^^ACME
ORC|RE|ORD776655
OBR|1|ORD776655|LAB2233|24331-1^Lipid Panel^LN|||20260813071500
OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F
OBX|2|CE|10331-7^Rh Type^LN||D^Rh positive^LN|||N|||F
NTE|1||Fasting sample.`;

  const cliTree = `$ cargo install hl7-2
$ hl7-v2 oru_r01.hl7
ORU_R01
  MSH
    MSH.1 = |
    MSH.2 = ^~\\&
    MSH.3
      HD.1 = LAB
    ...
  ORU_R01.PATIENT_RESULT
    ORU_R01.PATIENT
      PID
        PID.1 = 1
        PID.5
          XPN.1 = EVERYWOMAN
          XPN.2 = EVE`;

  const cliQuery = `$ hl7-v2 --query OBX-5 oru_r01.hl7
187
D^Rh positive^LN

$ hl7-v2 --paths oru_r01.hl7 | head -3
$ hl7-v2 --check oru_r01.hl7
ok`;

  const cargo = `cargo new hl7-hello
cd hl7-hello
cargo add hl7`;

  const parse = `use hl7::v2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string("oru_r01.hl7")?;
    let message = v2::parse(&text)?;

    // MSH-9 said ORU^R01, so the dictionary knows the structure.
    assert_eq!(message.structure_id(), "ORU_R01");
    Ok(())
}`;

  const read = `// A path addresses a value. This is what --query takes, and what set takes.
assert_eq!(message.get("PID-5.1")?.as_deref(), Some("EVERYWOMAN"));
assert_eq!(message.get("OBX[2]-5.2")?.as_deref(), Some("Rh positive"));

// The tree names a value, using HL7's own vocabulary.
let tree = message.tree();
assert_eq!(tree.name(), "ORU_R01");
assert_eq!(tree.find("XPN.1").unwrap().text(), "EVERYWOMAN");

// Every node knows the path that reads it back.
let second_obx = tree.find_all("OBX").nth(1).unwrap();
assert_eq!(second_obx.path(), "OBX[2]");`;

  const check = `for diagnostic in message.validate() {
    println!("{diagnostic}");
    // error: MSA[1]-4[1]: "many" is not a valid NM value
}

// Or refuse anything that does not conform, up front:
let options = v2::Options::new().strict();
match v2::parse_with_options(&text, &options) {
    Ok(message) => { /* conformant */ }
    Err(v2::Error::Invalid(diagnostics)) => { /* every error-level finding */ }
    Err(other) => { /* not a message at all */ }
}`;

  const answer = `let ack = v2::builder::acknowledge(&message, "AA", "ACK00001", "20260814080100")
    .build_valid()?;

// MSA-2 echoes the control ID of the message being answered. That echo is
// the whole mechanism: it is what says *which* message this answers.
assert_eq!(ack.get("MSA-2")?.as_deref(), Some("MSG00042"));

let er7 = ack.to_er7();   // valid ER7, ready to send`;

  const modify = `let mut message = v2::parse(&text)?;
message.set("PID-5.2", "EVELYN")?;   // escapes delimiters in the value
message.append_segment("NTE");
message.set("NTE[2]-3", "Amended.")?;

let er7 = message.to_er7();`;

  const convert = `$ hl7-2-from-er7-into-xml oru_r01.hl7 > oru_r01.xml
$ hl7-2-from-er7-into-json oru_r01.hl7 > oru_r01.json

# And back again — the round trip is the smoke test
$ hl7-2-from-er7-into-xml oru_r01.hl7 | hl7-2-from-xml-into-er7`;
</script>

<DocPage
  lede="A real lab result, parsed, queried, validated, answered, and converted — from an empty directory to a working acknowledgement in about five minutes."
  {contents}
>
  <h2 id="the-message">The message</h2>
  <p>
    Save this as <code>oru_r01.hl7</code>. It is an ORU^R01 — an unsolicited observation result,
    the message a lab sends when it has an answer. Segment terminators are carriage returns in the
    real thing; a text editor's newlines are fine here, because the parser accepts either.
  </p>
  <CodeSample language="er7" code={message} label="An ORU^R01 lipid panel result" />
  <p>
    Read left to right, <code>MSH</code> is the header that carries the delimiters and says what
    the message is; <code>PID</code> identifies the patient; <code>OBR</code> is the order the
    result answers; each <code>OBX</code> is one observation; <code>NTE</code> is a note.
  </p>

  <h2 id="look-at-it">1. Look at it, without writing code</h2>
  <p>
    Start with the tool, not with a project. The first question about any message is “what is
    actually in here”, and the tree answers it.
  </p>
  <CodeSample language="sh" code={cliTree} />
  <p>
    Notice what the dictionary did: <code>PID.5</code> broke into <code>XPN.1</code> and
    <code>XPN.2</code>, because HL7 v2.5 says <code>PID-5</code> holds an <code>XPN</code> — an
    extended person name. Nothing about that is hardcoded per field; it comes from the release the
    header claimed.
  </p>
  <CodeSample language="sh" code={cliQuery} />

  <h2 id="parse">2. Parse it</h2>
  <CodeSample language="sh" code={cargo} />
  <CodeSample language="rust" caption="src/main.rs" code={parse} />
  <p>
    <code>parse</code> is lenient by design. An unknown segment, an unknown data type, or a
    structure that does not match its declared grammar is never an error — it degrades to a
    positional name or a flat layout. Checking is a
    <a href="/guides/validating/">separate question with a separate answer</a>.
  </p>

  <h2 id="read">3. Read values out of it</h2>
  <p>There are two vocabularies here, and keeping them apart saves a lot of confusion.</p>
  <CodeSample language="rust" code={read} />
  <Callout heading="Names describe, paths address">
    <p>
      <code>XPN.1</code> is a <em>node name</em>: it says this component is the family-name part of
      an extended person name. <code>PID-5.1</code> is a <em>path</em>: it says segment
      <code>PID</code>, field 5, component 1. The tree and the XML and JSON output speak names;
      <code>--query</code>, <code>get</code>, and <code>set</code> take paths. See
      <a href="/guides/navigating/">Navigating</a> for the full path grammar.
    </p>
  </Callout>

  <h2 id="check">4. Check it</h2>
  <CodeSample language="rust" code={check} />
  <p>
    Diagnostics split by whose problem it is. <strong>Errors</strong> are the message contradicting
    the dictionary it claims — a required segment missing, a numeric field holding letters.
    <strong>Warnings</strong> are the dictionary not covering the message — an unknown segment, a
    structure with no grammar yet. Strict mode rejects the first and allows the second, so a local
    Z-segment does not make an otherwise conformant message fail.
  </p>

  <h2 id="answer">5. Answer it</h2>
  <p>A system that reads HL7 usually has to reply in it. First, editing:</p>
  <CodeSample language="rust" code={modify} />
  <p>Then the acknowledgement, which is what an ORU^R01 sender is waiting for:</p>
  <CodeSample language="rust" code={answer} />
  <p>
    An unmodified message writes back byte for byte — that guarantee comes from the
    <code>er7</code> layer, and <code>hl7-2</code> does not weaken it. To put that
    acknowledgement on a socket, see <a href="/guides/mllp/">MLLP over TCP</a>.
  </p>

  <h2 id="convert">6. Convert it</h2>
  <p>
    If what you actually need is the message in a format something downstream can read, skip the
    library entirely:
  </p>
  <CodeSample language="sh" code={convert} />
  <p>
    The last line is the round trip, and it is the smoke test the crates use on themselves: the
    output is the original ER7 message, canonicalized. See
    <a href="/tutorials/round-trip/">A lossless round trip</a>.
  </p>

  <h2 id="next">Where to go next</h2>
  <ul>
    <li>
      <a href="/docs/concepts/">Concepts</a> — ER7, v2.xml, the dictionary, names versus paths, and
      why the HL7 null is not the same as an empty field.
    </li>
    <li>
      <a href="/tutorials/vendor-dialect/">Taming a vendor dialect</a> — what to do when the
      message has segments the standard has never heard of. This is the common case, not the
      exception.
    </li>
    <li>
      <a href="/guides/">The guides</a> — one page per task, each with the API calls that do it.
    </li>
  </ul>
</DocPage>
