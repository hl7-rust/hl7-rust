<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'when', label: 'When struct mode is the right answer' },
    { id: 'enabling', label: 'Enabling it' },
    { id: 'reading', label: 'Reading: FromHl7' },
    { id: 'attributes', label: 'The attributes' },
    { id: 'types', label: 'Field types' },
    { id: 'raw', label: 'The raw escape hatch' },
    { id: 'writing', label: 'Writing: ToHl7' },
    { id: 'errors', label: 'Errors' },
    { id: 'v3', label: 'The v3 equivalent, and how it differs' }
  ];

  const cargo = `[dependencies]
hl7-2 = { version = "0.3", features = ["derive"] }

# Or through the umbrella crate, which forwards the feature:
hl7 = { version = "0.2", features = ["derive"] }`;

  const reading = `use hl7_2::{FromHl7, Raw};

#[derive(FromHl7)]
struct Admission {
    #[hl7("PID-1")]      sequence: u32,
    #[hl7("PID-3")]      identifiers: Vec<String>,
    #[hl7("PID-5.1.1")]  family_name: String,
    #[hl7("PID-8")]      sex: Option<String>,
    #[hl7(nested)]       visit: Visit,   // its own FromHl7
    #[hl7(raw)]          raw: Raw,       // the whole message, kept alongside
}

let admission: Admission = hl7_2::parse(text)?.decode()?;
assert_eq!(admission.family_name, "EVERYWOMAN");`;

  const raw = `// The one vendor field no struct models — same object, no second parse.
assert_eq!(admission.raw.get("ZPD-1")?.as_deref(), Some("local"));`;

  const writing = `use hl7_2::{FromHl7, ToHl7};

#[derive(FromHl7, ToHl7)]
struct Admission {
    #[hl7("PID-1")]  sequence: u32,
    #[hl7("PID-8")]  sex: Option<String>,
}

// Writing needs the segments to exist already.
let message = hl7_2::Builder::new(hl7_2::Version::V2_5)
    .message_type("ADT", "A01")
    .control_id("MSG00042")
    .segment("PID")
    .encode(&admission)
    .build_valid()?;`;

  const append = `// Or add the segment to a message you already hold:
let mut message = hl7_2::parse(text)?;
message.append_segment("ZPD");
message.set("ZPD-1", "local")?;`;

  const errors = `match hl7_2::parse(text)?.decode::<Admission>() {
    Ok(admission) => { /* every required path was present and fit */ }
    Err(hl7_2::Error::MissingField { path }) => { /* a plain field's path named nothing */ }
    Err(hl7_2::Error::BadValue { path, expected, found }) => { /* present, wrong shape */ }
    Err(other) => { /* a bad path, or a message-level problem */ }
}`;

  const v3 = `use hl7_3::FromElement;
use hl7_3::rim::Act;

#[derive(FromElement, Default)]
struct Observation {
    #[element("classCode")]          class_code: String,
    #[element("moodCode")]           mood_code: String,
    #[element(child = "note")]       note: Option<String>,
    #[element(nested = "component")] component: Act,
    #[element(raw)]                  raw: hl7_3::xml::Element,
}

let element = hl7_3::xml::parse(xml)?;
let observation = Observation::from_element(&element);   // no Result
assert_eq!(observation.class_code, "OBS");`;
</script>

<DocPage
  lede="Map a struct's fields to HL7 paths once, in the type definition, instead of writing the same accessor calls at every call site — and keep a door open to the parsed message for the day the feed changes."
  {contents}
>
  <h2 id="when">When struct mode is the right answer</h2>
  <p>
    Struct mode is the third of <a href="/guides/parsing/">three parsing modes</a>, and it is the
    one to reach for once an interface has held still long enough to trust. The payoff is that the
    compiler carries the mapping: a typo in a path is a build failure rather than a
    <code>None</code> at three in the morning.
  </p>
  <p>
    The risk is the mirror image. A struct is a claim that you know what arrives, and real feeds
    are stable until they are not. That is what <a href="#raw"><code>#[hl7(raw)]</code></a> is for,
    and it is why you should keep it even when you are sure you do not need it.
  </p>

  <h2 id="enabling">Enabling it</h2>
  <CodeSample language="toml" code={cargo} />
  <p>
    The macros live in <a href="/crates/hl7-2-derive/"><code>hl7-2-derive</code></a>, which you do
    not depend on directly. Keeping them in a crate of their own is what lets the default build of
    <code>hl7-2</code> hold exactly one dependency: <code>syn</code> and <code>quote</code> are
    compiled only for callers who ask for the macros.
  </p>

  <h2 id="reading">Reading: FromHl7</h2>
  <CodeSample language="rust" code={reading} />
  <p>
    <code>decode</code> is a method on the parsed message, so nothing is re-read from text — the
    struct is filled in from the tree you already have.
  </p>

  <h2 id="attributes">The attributes</h2>
  <p>One attribute per field.</p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Attribute</th><th>On read</th><th>On write</th></tr>
      </thead>
      <tbody>
        <tr>
          <td><code>#[hl7("PID-5.1")]</code></td>
          <td>Read the path.</td>
          <td>Write the path.</td>
        </tr>
        <tr>
          <td><code>#[hl7(nested)]</code></td>
          <td>The field's own <code>FromHl7</code>.</td>
          <td>The field's own <code>ToHl7</code>.</td>
        </tr>
        <tr>
          <td><code>#[hl7(raw)]</code></td>
          <td>The whole message, as a <code>Raw</code>.</td>
          <td>Skipped.</td>
        </tr>
        <tr>
          <td>none</td>
          <td><code>Default::default()</code></td>
          <td>Skipped.</td>
        </tr>
      </tbody>
    </table>
  </div>
  <p>
    <code>#[hl7(nested)]</code> is how a large message becomes several small types rather than one
    struct with forty fields — a <code>Visit</code>, a <code>Patient</code>, an
    <code>Order</code>, each deriving <code>FromHl7</code> in its own right.
  </p>

  <h2 id="types">Field types</h2>
  <p>
    Field types convert through <code>hl7_2::FromHl7Value</code> and
    <code>ToHl7Value</code>: <code>String</code>, <code>bool</code>, the integer and floating-point
    types, and <code>Option&lt;T&gt;</code> and <code>Vec&lt;T&gt;</code> of those.
  </p>
  <ul>
    <li><code>Option&lt;T&gt;</code> for a value that may be absent.</li>
    <li><code>Vec&lt;T&gt;</code> for a value that repeats.</li>
    <li>
      A plain type is <strong>required</strong> — a path that names nothing gives
      <code>Error::MissingField</code>.
    </li>
  </ul>
  <p>
    For a domain type of your own, implement <code>hl7_2::FromHl7Text</code>; then
    <code>Option</code> and <code>Vec</code> of it follow automatically.
  </p>
  <Callout heading="Be sparing with plain types">
    <p>
      Every plain (non-<code>Option</code>) field is a promise that the sender always populates it.
      In HL7 that promise is rarely safe outside <code>MSH</code>: optionality in the standard is
      generous, and a field that has been present in every message for a year can be absent
      tomorrow. Prefer <code>Option&lt;T&gt;</code> and handle the absence, unless the whole
      message is meaningless without the value.
    </p>
  </Callout>

  <h2 id="raw">The raw escape hatch</h2>
  <CodeSample language="rust" code={raw} />
  <p>
    A <code>#[hl7(raw)]</code> field keeps the whole parsed message beside the typed data. When a
    message arrives with something the struct does not model, the fallback is a method call on the
    object you already have — no second parse, no rewrite, no redeployment to add a field you did
    not know about.
  </p>
  <p>
    This is the same object you would have had in generic mode, so everything in
    <a href="/guides/navigating/">Navigating</a> applies to it.
  </p>

  <h2 id="writing">Writing: ToHl7</h2>
  <p>
    <code>#[derive(ToHl7)]</code> is the reverse direction, and it has one requirement worth
    knowing up front: <strong>writing needs the segments to exist already</strong>.
  </p>
  <CodeSample language="rust" code={writing} />
  <CodeSample language="rust" code={append} />
  <p>
    <code>Builder::encode</code> takes any <code>ToHl7</code>, so a struct is written into a message
    under construction. <code>build_valid</code> then runs
    <a href="/guides/validating/">validation</a> and refuses to hand back something the dictionary
    says is malformed.
  </p>

  <h2 id="errors">Errors</h2>
  <CodeSample language="rust" code={errors} />
  <p>
    The two struct-specific variants say different things.
    <code>MissingField</code> means a non-optional field's path named nothing — usually a sign the
    field should have been <code>Option</code>. <code>BadValue</code> means the value was present
    but did not fit the Rust type, and carries the path, what was expected, and what was found,
    which is normally enough to identify the sender's bug without opening the message.
  </p>

  <h2 id="v3">The v3 equivalent, and how it differs</h2>
  <p>
    <a href="/crates/hl7-3/"><code>hl7-3</code></a> has its own derive, behind its own
    <code>derive</code> feature, mapping struct fields to XML attributes and children instead of
    HL7 paths.
  </p>
  <CodeSample language="rust" code={v3} />
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Attribute</th><th>Reads</th></tr>
      </thead>
      <tbody>
        <tr><td><code>#[element("classCode")]</code></td><td>The <code>classCode</code> attribute.</td></tr>
        <tr><td><code>#[element(child = "note")]</code></td><td>The <code>note</code> child's text.</td></tr>
        <tr>
          <td><code>#[element(nested = "component")]</code></td>
          <td>The <code>component</code> child, via the field type's own <code>FromElement</code>.</td>
        </tr>
        <tr>
          <td><code>#[element(raw)]</code></td>
          <td>The whole element, as <code>hl7_3::xml::Element</code>.</td>
        </tr>
        <tr><td>none</td><td><code>Default::default()</code></td></tr>
      </tbody>
    </table>
  </div>
  <p>Two differences from the v2 macro are deliberate and worth stating plainly:</p>
  <ul>
    <li>
      <strong>No <code>Result</code> anywhere.</strong> A missing attribute or child is not an
      error — it reads as that field's <code>Default</code>, matching the degrade-don't-reject
      choice <code>hl7-3</code>'s own <code>rim</code> types make.
    </li>
    <li>
      <strong>No <code>Vec&lt;T&gt;</code> support yet.</strong> A repeating child needs
      <code>element.children_named(...)</code> by hand for now.
    </li>
    <li>
      <strong>No <code>#[derive(ToElement)]</code>.</strong> <code>hl7-3</code> has no XML-writing
      capability yet, so a write-direction macro would have nothing real to generate.
    </li>
  </ul>
  <p>
    Note that the umbrella crate's <code>derive</code> feature forwards to <code>hl7-2</code>'s
    only. For the v3 macro, enable the feature on <code>hl7-3</code>.
  </p>
</DocPage>
