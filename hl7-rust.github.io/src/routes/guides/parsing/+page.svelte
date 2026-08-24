<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'not-three-libraries', label: 'Three modes, not three libraries' },
    { id: 'generic', label: 'Generic mode' },
    { id: 'schema', label: 'Schema mode' },
    { id: 'struct', label: 'Struct mode' },
    { id: 'choosing', label: 'Choosing between them' },
    { id: 'options', label: 'Parse options' },
    { id: 'batches', label: 'Batches and multiple messages' },
    { id: 'errors', label: 'What actually fails' }
  ];

  const generic = `use hl7::v2;

let message = v2::parse(text)?;
let tree = message.tree();

assert_eq!(tree.name(), "ORU_R01");
assert_eq!(tree.find("XPN.1").unwrap().text(), "EVERYWOMAN");

// Every node knows the path that reads it back.
let second = tree.find_all("OBX").nth(1).unwrap();
assert_eq!(second.path(), "OBX[2]");
assert_eq!(message.get("OBX[2]-5.2")?.as_deref(), Some("Rh positive"));`;

  const schema = `use std::sync::Arc;

let dictionary = v2::Dictionary::from_json(r#"{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}"#, "acme")?;

let options = v2::Options::new().with_dictionary(Arc::new(dictionary));
let message = v2::parse_with_options(text, &options)?;

// The vendor's own segment now reads like any standard one.
assert_eq!(message.tree().find("XPN.2").unwrap().text(), "JOHN");`;

  const structMode = `use hl7_2::{FromHl7, Raw};

#[derive(FromHl7)]
struct Admission {
    #[hl7("PID-3.1")]  patient_id: String,
    #[hl7("PID-7.1")]  birth_date: Option<String>,
    #[hl7("PID-3")]    all_identifiers: Vec<String>,
    #[hl7(raw)]        raw: Raw,
}

let admission: Admission = hl7_2::parse(text)?.decode()?;
assert_eq!(admission.patient_id, "241900");

// The one vendor field no struct models — same object, no second parse.
assert_eq!(admission.raw.get("ZPD-1")?.as_deref(), Some("local"));`;

  const cargo = `hl7-2 = { version = "0.2", features = ["derive"] }`;

  const options = `let options = v2::Options::new()
    .with_version(v2::Version::V2_3)        // ignore MSH-12
    .with_dictionary(Arc::new(dictionary)) // read a vendor dialect
    .strict();                             // reject error-level findings

let message = v2::parse_with_options(text, &options)?;`;

  const batch = `for text in v2::split_messages(batch) {
    match v2::parse(&text) {
        Ok(message) => handle(message),
        Err(error) => eprintln!("skipping malformed message: {error}"),
    }
}`;

  const errors = `match v2::parse(text) {
    Ok(message) => { /* something that is at least shaped like a message */ }
    Err(v2::Error::Invalid(diagnostics)) => { /* only reachable in strict mode */ }
    Err(other) => { /* no MSH, empty input, or a malformed header */ }
}`;
</script>

<DocPage
  title="Parsing"
  lede="hl7-2 offers three parsing modes. They are not three libraries to choose between — they are three stages of the same job, and a real integration walks through them in order."
  {contents}
>
  <h2 id="not-three-libraries">Three modes, not three libraries</h2>
  <p>
    Every mode shares one set of internals: the same ER7 layer, the same dictionary, the same
    message object. What differs is how much you have told the library in advance.
  </p>
  <ul>
    <li><strong>Generic</strong> — you have told it nothing. For the vendor you have never seen.</li>
    <li>
      <strong>Schema-based</strong> — you have told it the shape, in JSON, loaded at runtime. For
      the vendor whose format is not frozen.
    </li>
    <li>
      <strong>Struct-based</strong> — you have told it the shape in the type system. For the feed
      that does not change.
    </li>
  </ul>
  <p>
    You can move between them on the same message without re-parsing, and struct mode keeps a
    door open back to generic mode — see <a href="#struct">below</a>.
  </p>

  <h2 id="generic">Generic mode</h2>
  <p>
    Parse anything into a navigable tree. Nothing is rejected, nothing is dropped, and what the
    dictionary recognises gets a name from HL7's own vocabulary.
  </p>
  <CodeSample language="rust" code={generic} />
  <p>
    Segments are grouped into the message structure when they fit it —
    <code>ORU_R01.PATIENT_RESULT.ORDER_OBSERVATION.OBSERVATION</code> — and read flat when they do
    not. Neither outcome is an error.
  </p>
  <p>
    This is the mode to start in, always. Even when you know the feed, the first thing worth doing
    with an unfamiliar message is looking at its tree — from Rust, or from
    <a href="/docs/cli/">the command line</a>, which needs no project at all.
  </p>

  <h2 id="schema">Schema mode</h2>
  <p>
    Write the shape as JSON, load it at runtime, and adding a field becomes a configuration change
    rather than a release of your software.
  </p>
  <CodeSample language="rust" code={schema} />
  <p>
    The same format describes the bundled releases, so a schema can inherit one and state only its
    dialect — the example above is a complete, valid dictionary. Dictionaries can also be
    <a href="/guides/dictionaries/">generated from a site's own XML Schemas</a>, which additionally
    captures cardinality.
  </p>

  <h2 id="struct">Struct mode</h2>
  <p>Once the interface has held still long enough to trust, let the compiler carry it.</p>
  <CodeSample language="toml" caption="Cargo.toml — struct mode needs the derive feature" code={cargo} />
  <CodeSample language="rust" code={structMode} />
  <Callout heading="The raw field is the point">
    <p>
      Real feeds are stable until they are not, and the usual choice at that moment is to re-parse
      the raw message or rewrite the library. A <code>#[hl7(raw)]</code> field keeps the whole
      parsed message beside the typed data, so the fallback is a method call on the object you
      already have. Keep it even when you are sure you do not need it — stage three is never final.
    </p>
  </Callout>
  <p>The full attribute reference is in <a href="/guides/struct-mode/">Struct mode and derive</a>.</p>

  <h2 id="choosing">Choosing between them</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>If…</th><th>Use</th><th>Because</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>You have never seen this sender's output</td>
          <td>Generic</td>
          <td>Nothing to declare yet, and nothing is rejected.</td>
        </tr>
        <tr>
          <td>The vendor adds a field most quarters</td>
          <td>Schema</td>
          <td>A new field is one line in a JSON file, not a release.</td>
        </tr>
        <tr>
          <td>The site already has v2.xml XSDs</td>
          <td>Schema, generated</td>
          <td>The dictionary builder turns them into the file the parser reads.</td>
        </tr>
        <tr>
          <td>The feed has been identical for two years</td>
          <td>Struct</td>
          <td>The compiler checks the mapping, once, at build time.</td>
        </tr>
        <tr>
          <td>Mostly stable, with one field nobody can explain</td>
          <td>Struct, with <code>#[hl7(raw)]</code></td>
          <td>Typed where it is knowable, generic where it is not.</td>
        </tr>
      </tbody>
    </table>
  </div>

  <h2 id="options">Parse options</h2>
  <CodeSample language="rust" code={options} />
  <p>
    <code>with_version</code> matters more than it sounds like it should. Real senders declare
    <code>2.3</code> in <code>MSH-12</code> and send 2.5-shaped content, or the reverse. When the
    declaration and the content disagree, believe the content and force the release. See
    <a href="/docs/versions/">Versions and compatibility</a>.
  </p>

  <h2 id="batches">Batches and multiple messages</h2>
  <p>
    Input may hold one message, several, or an HL7 batch file. <code>split_messages</code> handles
    all three, dropping FHS/BHS envelopes, and yields each message's text.
  </p>
  <CodeSample language="rust" code={batch} />
  <p>
    Handle the error per message rather than for the batch. One malformed message in a file of two
    thousand should not cost you the other 1,999.
  </p>

  <h2 id="errors">What actually fails</h2>
  <p>
    Parsing is lenient by design, and the set of things that make it fail is deliberately small:
    empty input, no <code>MSH</code> segment, or a malformed <code>MSH</code> header. Anything
    below the header converts, degrading to positional names or a flat layout rather than failing.
  </p>
  <CodeSample language="rust" code={errors} />
  <p>
    <code>Error::Invalid</code> only appears in strict mode. Outside it, a conformance problem is a
    diagnostic you ask for, not an error you are handed — see
    <a href="/guides/validating/">Validating</a>.
  </p>
</DocPage>
