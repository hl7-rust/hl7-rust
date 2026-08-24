<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'goal', label: 'What you will build' },
    { id: 'step-1', label: '1. Set up' },
    { id: 'step-2', label: '2. Parse it' },
    { id: 'step-3', label: '3. Find the patient' },
    { id: 'step-4', label: '4. Find every result' },
    { id: 'step-5', label: '5. Check it before trusting it' },
    { id: 'step-6', label: '6. Answer it' },
    { id: 'whole', label: 'The whole program' },
    { id: 'next', label: 'Next' }
  ];

  const setup = `cargo new hl7-first
cd hl7-first
cargo add hl7`;

  const sample = `MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5
PID|1||444333222^^^ACME&1.2.3.4&ISO^MR~987654321^^^NHS^NH||EVERYWOMAN^EVE^E||19620320|F
OBR|1|ORD776655|LAB2233|24331-1^Lipid Panel^LN|||20260813071500
OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F
OBX|2|NM|2085-9^HDL^LN||55|mg/dL|>40|N|||F
OBX|3|CE|10331-7^Rh Type^LN||D^Rh positive^LN|||N|||F
NTE|1||Fasting sample.`;

  const parse = `use hl7::v2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string("oru_r01.hl7")?;
    let message = v2::parse(&text)?;

    println!("structure: {}", message.structure_id());
    // structure: ORU_R01
    Ok(())
}`;

  const patient = `let family = message.get("PID-5.1")?.unwrap_or_default();
let given  = message.get("PID-5.2")?.unwrap_or_default();
let sex    = message.get("PID-8")?;
println!("{family}, {given} ({sex:?})");
// EVERYWOMAN, EVE (Some("F"))

// PID-3 repeats: an ACME medical record number AND an NHS number.
for identifier in message.repetitions("PID-3")? {
    println!("identifier: {identifier}");
}`;

  const identifiers = `// Better: read the id and its assigning authority together, and pick.
let mut nhs_number = None;
for index in 1..=message.repetitions("PID-3")?.len() {
    let authority = message.get(&format!("PID-3[{index}].4"))?;
    if authority.as_deref() == Some("NHS") {
        nhs_number = message.get(&format!("PID-3[{index}].1"))?;
    }
}
assert_eq!(nhs_number.as_deref(), Some("987654321"));`;

  const results = `for observation in message.tree().find_all("OBX") {
    let path = observation.path();
    let code  = message.get(&format!("{path}-3.2"))?.unwrap_or_default();
    let value = message.get(&format!("{path}-5"))?.unwrap_or_default();
    let units = message.get(&format!("{path}-6"))?.unwrap_or_default();
    let flag  = message.get(&format!("{path}-8"))?.unwrap_or_default();
    println!("{code}: {value} {units} [{flag}]");
}
// Cholesterol: 187 mg/dL [N]
// HDL: 55 mg/dL [N]
// Rh Type: D [N]`;

  const check = `let diagnostics = message.validate();
for diagnostic in &diagnostics {
    println!("{diagnostic}");
}

let has_errors = diagnostics
    .iter()
    .any(|diagnostic| diagnostic.severity == v2::Severity::Error);`;

  const answer = `let code = if has_errors { "AE" } else { "AA" };

let mut ack = v2::builder::acknowledge(&message, code, "ACK00001", "20260814080100")
    .build_valid()?;

if has_errors {
    ack.set("MSA-3", "message did not validate")?;
}

println!("{}", ack.to_er7());
assert_eq!(ack.get("MSA-2")?.as_deref(), Some("MSG00042"));`;

  const whole = `use hl7::v2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string("oru_r01.hl7")?;
    let message = v2::parse(&text)?;

    // Who
    let family = message.get("PID-5.1")?.unwrap_or_default();
    let given = message.get("PID-5.2")?.unwrap_or_default();
    println!("{family}, {given}");

    // What
    for observation in message.tree().find_all("OBX") {
        let path = observation.path().to_string();
        let code = message.get(&format!("{path}-3.2"))?.unwrap_or_default();
        let value = message.get(&format!("{path}-5"))?.unwrap_or_default();
        let units = message.get(&format!("{path}-6"))?.unwrap_or_default();
        println!("  {code}: {value} {units}");
    }

    // Whether we trust it
    let diagnostics = message.validate();
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == v2::Severity::Error);
    for diagnostic in &diagnostics {
        eprintln!("{diagnostic}");
    }

    // What we say back
    let code = if has_errors { "AE" } else { "AA" };
    let mut ack = v2::builder::acknowledge(&message, code, "ACK00001", "20260814080100")
        .build_valid()?;
    if has_errors {
        ack.set("MSA-3", "message did not validate")?;
    }
    println!("{}", ack.to_er7());

    Ok(())
}`;
</script>

<DocPage
  title="Your first message"
  eyebrow="Tutorial"
  lede="From an empty directory to a program that reads a lab result, pulls out the patient and every observation, validates it, and produces the acknowledgement the sender is waiting for."
  {contents}
>
  <h2 id="goal">What you will build</h2>
  <p>
    A small program that behaves like the receiving end of a real lab interface. It answers four
    questions in order — who is this about, what does it say, do I trust it, and what do I say back
    — because that is the order a production receiver answers them in too.
  </p>
  <p>Save this as <code>oru_r01.hl7</code> in the project directory you are about to create:</p>
  <CodeSample language="er7" code={sample} label="An ORU^R01 with a repeating identifier" />
  <p>
    Note <code>PID-3</code>: it carries two identifiers separated by <code>~</code>, an ACME medical
    record number and an NHS number. That repetition is not decoration — it is the most common thing
    people get wrong on their first integration, and step 3 deals with it.
  </p>

  <h2 id="step-1">1. Set up</h2>
  <CodeSample language="sh" code={setup} />
  <p>
    <code>hl7</code> is the umbrella crate; <code>hl7::v2</code> is where everything in this
    tutorial lives. See <a href="/docs/install/">Install</a> if you would rather depend on
    <code>hl7-2</code> directly.
  </p>

  <h2 id="step-2">2. Parse it</h2>
  <CodeSample language="rust" caption="src/main.rs" code={parse} />
  <p>
    <code>structure_id</code> is the message's own claim about its shape, read from
    <code>MSH-9</code>. Because <code>ORU_R01</code> is one of the structures the dictionary has a
    grammar for, the segments are nested into groups rather than read flat — which you can see for
    yourself with <code>hl7-v2 --paths oru_r01.hl7</code>.
  </p>

  <h2 id="step-3">3. Find the patient</h2>
  <CodeSample language="rust" code={patient} />
  <Callout type="warning" heading="PID-3.1 is not “the patient id”">
    <p>
      An unindexed path reads the <em>first</em> repetition. Which identifier the sender put first
      is not guaranteed, and it can change between releases of their software. Code that reads
      <code>PID-3.1</code> and calls it the patient id will silently pick up the wrong number the
      day they reorder the field.
    </p>
  </Callout>
  <p>Read the identifier and its assigning authority together, and select the one you meant:</p>
  <CodeSample language="rust" code={identifiers} />
  <p>
    <code>PID-3.4</code> is the assigning authority component of the <code>CX</code> data type. That
    is the dictionary at work: it knows <code>PID-3</code> holds a <code>CX</code>, so it knows what
    the fourth component means.
  </p>

  <h2 id="step-4">4. Find every result</h2>
  <p>
    There are three <code>OBX</code> segments. Rather than hardcoding <code>OBX[1]</code> through
    <code>OBX[3]</code>, walk the tree and let each node tell you its own path.
  </p>
  <CodeSample language="rust" code={results} />
  <p>
    That <code>observation.path()</code> call is the bridge between the two vocabularies:
    <code>find_all</code> searches by <em>name</em>, and <code>path()</code> hands back the
    <em>address</em> that <code>get</code> takes. See
    <a href="/guides/navigating/">Navigating</a>.
  </p>

  <h2 id="step-5">5. Check it before trusting it</h2>
  <CodeSample language="rust" code={check} />
  <p>
    Split on severity rather than on emptiness. An <strong>error</strong> means the message
    contradicts the dictionary it claims, and is the sender's problem. A <strong>warning</strong>
    means the dictionary does not cover the message, which is a coverage gap rather than a fault —
    rejecting on warnings would reject valid traffic. See
    <a href="/guides/validating/">Validating</a>.
  </p>

  <h2 id="step-6">6. Answer it</h2>
  <CodeSample language="rust" code={answer} />
  <p>
    <code>MSA-2</code> echoing <code>MSG00042</code> is the whole mechanism by which the sender
    knows which message you answered. The control ID and timestamp are arguments rather than
    generated, so this code is testable and traceable — the same reasoning behind
    <a href="/guides/mllp/#acknowledgement">MLLP's acknowledgement API</a>.
  </p>
  <Callout type="warning" heading="AA means “it is safe”, not “it parsed”">
    <p>
      In a real receiver, send <code>AA</code> only after you have persisted the message. This
      program prints it immediately, which is fine for a tutorial and wrong for production:
      acknowledging before writing turns a restart into silent data loss.
    </p>
  </Callout>

  <h2 id="whole">The whole program</h2>
  <CodeSample language="rust" caption="src/main.rs, complete" code={whole} />

  <h2 id="next">Next</h2>
  <ul>
    <li>
      <a href="/tutorials/vendor-dialect/">Taming a vendor dialect</a> — what to do when the message
      contains segments the standard has never heard of.
    </li>
    <li>
      <a href="/tutorials/mllp-listener/">An MLLP listener that answers</a> — put this program behind
      a socket.
    </li>
    <li>
      <a href="/guides/struct-mode/">Struct mode</a> — once the feed has held still, replace those
      <code>get</code> calls with a struct the compiler checks.
    </li>
  </ul>
</DocPage>
