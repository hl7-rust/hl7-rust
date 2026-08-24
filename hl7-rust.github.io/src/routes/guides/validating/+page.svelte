<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'separate-question', label: 'Parsing is lenient; checking is a separate question' },
    { id: 'validate', label: 'Asking for diagnostics' },
    { id: 'errors', label: 'Errors — the message contradicts its dictionary' },
    { id: 'warnings', label: 'Warnings — the dictionary does not cover the message' },
    { id: 'z-segments', label: 'Z-segments do not fail a message' },
    { id: 'strict', label: 'Strict mode' },
    { id: 'not-checked', label: 'What is deliberately not checked' },
    { id: 'shell', label: 'Checking from a shell' }
  ];

  const validate = `let message = hl7_2::parse(text)?;

for diagnostic in message.validate() {
    println!("{diagnostic}");
    // error: MSA[1]-4[1]: "many" is not a valid NM value
}`;

  const severity = `use hl7_2::Severity;

let diagnostics = message.validate();

let (errors, warnings): (Vec<_>, Vec<_>) = diagnostics
    .into_iter()
    .partition(|diagnostic| diagnostic.severity == Severity::Error);

if !errors.is_empty() {
    // The sender's problem. Reject, and say why in MSA-3.
}
if !warnings.is_empty() {
    // Our problem, or nobody's. Log it; do not reject on it.
}`;

  const strict = `let options = hl7_2::Options::new().strict();

match hl7_2::parse_with_options(text, &options) {
    Ok(message) => { /* conformant */ }
    Err(hl7_2::Error::Invalid(diagnostics)) => { /* every error-level finding */ }
    Err(other) => { /* not a message at all */ }
}`;

  const nack = `let mut nack = hl7_2::builder::acknowledge(&message, "AE", "N1", "20260814080100")
    .build_valid()?;
nack.set("MSA-3", &reason)?;`;

  const shell = `$ hl7-v2 --check samples/adt_a01.hl7
ok

$ hl7-v2 --check broken.hl7
error: MSH[1]-10[1]: message control ID is empty
$ echo $?
2`;
</script>

<DocPage
  title="Validating"
  lede="Parsing never rejects. Checking is a separate call with a separate answer, and its findings are split by whose problem they are — which is the distinction that makes strict mode usable in production."
  {contents}
>
  <h2 id="separate-question">Parsing is lenient; checking is a separate question</h2>
  <p>
    <code>hl7_2::parse</code> accepts unknown segments, unknown data types, and structures that do
    not match what the header claimed. None of those is an error, because a parser that rejected
    them would reject most real traffic — nearly every production interface carries something the
    standard does not describe.
  </p>
  <p>
    So conformance is asked for, not imposed. <code>Message::validate</code> checks a message
    against its dictionary and returns diagnostics. It never fails and never changes the message.
  </p>

  <h2 id="validate">Asking for diagnostics</h2>
  <CodeSample language="rust" code={validate} />
  <p>Each diagnostic carries a severity, and the split is the whole design:</p>
  <CodeSample language="rust" code={severity} />

  <h2 id="errors">Errors — the message contradicts its dictionary</h2>
  <p>
    <code>Severity::Error</code> means the sender said one thing and did another. These are worth
    rejecting on.
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Kind</th><th>Raised when</th></tr>
      </thead>
      <tbody>
        <tr><td><code>Header</code></td><td><code>MSH-9.1</code> or <code>MSH-10</code> is empty.</td></tr>
        <tr>
          <td><code>SegmentMissing</code></td>
          <td>A segment or group the structure requires is absent.</td>
        </tr>
        <tr>
          <td><code>StructureMismatch</code></td>
          <td>The segments do not fit the declared structure.</td>
        </tr>
        <tr>
          <td><code>ValueFormat</code></td>
          <td>
            An <code>SI</code>, <code>NM</code>, <code>DT</code>, <code>TM</code>, or
            <code>DTM</code> value is not one.
          </td>
        </tr>
      </tbody>
    </table>
  </div>

  <h2 id="warnings">Warnings — the dictionary does not cover the message</h2>
  <p>
    <code>Severity::Warning</code> means the message may be perfectly fine and this crate simply
    does not model the thing in question. A coverage gap here is not the sender's error, and
    treating it as one would reject valid traffic.
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Kind</th><th>Raised when</th></tr>
      </thead>
      <tbody>
        <tr>
          <td><code>Header</code></td>
          <td><code>MSH-12</code> is empty or names an unmodelled release.</td>
        </tr>
        <tr><td><code>StructureUnknown</code></td><td>No grammar for this structure id.</td></tr>
        <tr>
          <td><code>StructureMismatch</code></td>
          <td>The standard segments fit on their own, but local Z-segments do not.</td>
        </tr>
        <tr><td><code>SegmentUnknown</code></td><td>A segment the dictionary does not define.</td></tr>
        <tr>
          <td><code>FieldUnknown</code></td>
          <td>A field past the end of the segment's definition.</td>
        </tr>
        <tr>
          <td><code>ComponentUnknown</code></td>
          <td>A component past the end of the data type's definition.</td>
        </tr>
      </tbody>
    </table>
  </div>
  <p>
    Several of those warnings are actionable — by you, not by the sender. A
    <code>SegmentUnknown</code> or <code>FieldUnknown</code> is exactly the signal that it is time
    to write a <a href="/guides/dictionaries/">vendor dictionary</a>.
  </p>

  <h2 id="z-segments">Z-segments do not fail a message</h2>
  <p>
    A segment whose name begins with <code>Z</code> is a local extension the standard says nothing
    about — so neither does this crate. No <code>SegmentUnknown</code> is raised for one.
  </p>
  <p>
    And when a message fails its structure <em>only</em> because of Z-segments — the standard
    segments fit on their own — the mismatch is a warning rather than an error.
  </p>
  <Callout heading="Why that special case exists">
    <p>
      Most real interfaces carry at least one Z-segment. If a Z-segment made a message fail strict
      mode, strict mode would be unusable in production, and an unusable safety feature gets turned
      off. This rule is what keeps it on.
    </p>
  </Callout>

  <h2 id="strict">Strict mode</h2>
  <p>
    <code>Options::strict</code> runs the same check at parse time and turns any
    <code>Severity::Error</code> into <code>Error::Invalid</code>, carrying the diagnostics.
    Warnings never fail a parse.
  </p>
  <CodeSample language="rust" code={strict} />
  <p>
    Note the three arms. “Not a message at all” — empty input, no <code>MSH</code>, a malformed
    header — is a different failure from “a message that does not conform”, and an interface that
    conflates them will send the wrong acknowledgement code. When you reject, say why:
  </p>
  <CodeSample language="rust" code={nack} />

  <h2 id="not-checked">What is deliberately not checked</h2>
  <ul>
    <li>
      <strong>Value formats without a machine-checkable shape.</strong> <code>ST</code>,
      <code>TX</code>, <code>ID</code>, <code>IS</code> and the rest are constrained by HL7 tables
      and lengths, which this crate does not model. It says nothing about them rather than
      guessing.
    </li>
    <li>
      <strong>Empty values and explicit nulls.</strong> Neither is checked against a value format —
      an absent value has no format to violate.
    </li>
    <li>
      <strong>Table membership.</strong> Whether <code>PID-8</code> holds a code from HL7 table
      0001 is not checked; that is a vocabulary question, not a syntax one.
    </li>
    <li>
      <strong>Clinical sense.</strong> Nothing here knows that a cholesterol of 1870 mg/dL is
      implausible. That is your domain layer's job.
    </li>
  </ul>
  <p>
    The conversion crates check nothing at all — none of the four is a validator, and each says so
    in its own <code>spec/index.md</code>. If you need conformance before conversion, run
    <code>hl7-2</code> first.
  </p>

  <h2 id="shell">Checking from a shell</h2>
  <CodeSample language="sh" code={shell} />
  <p>
    Exit status 0 is success, 1 is a usage or parse error, and 2 means <code>--check</code> or
    <code>--strict</code> found something wrong with the message. Those three being distinct is
    what lets a triage script quarantine correctly — see
    <a href="/docs/cli/#exit-status">Command line</a>.
  </p>
</DocPage>
