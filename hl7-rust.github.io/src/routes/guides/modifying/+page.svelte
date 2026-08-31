<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'set', label: 'Setting a value' },
    { id: 'escaping', label: 'Escaping happens for you' },
    { id: 'null', label: 'Deleting versus emptying' },
    { id: 'segments', label: 'Adding segments' },
    { id: 'render', label: 'Rendering back to ER7' },
    { id: 'builder', label: 'Building a message from nothing' },
    { id: 'ack', label: 'Acknowledgements' },
    { id: 'cli', label: 'The same, from the shell' }
  ];

  const set = `let mut message = hl7_2::parse(text)?;

message.set("PID-5.2", "EVELYN")?;
message.set("PID-3[2].1", "999888777")?;
message.set("MSH-7", "20260814080100")?;

assert_eq!(message.get("PID-5.2")?.as_deref(), Some("EVELYN"));`;

  const escaping = `// The value contains a field separator. You do not escape it yourself.
message.set("OBX-5", "Reading was 90|100 mmHg")?;

// The rendered ER7 carries \\F\\ where the pipe was, so the message still
// parses as one field — and reading it back gives you the pipe again.
assert_eq!(message.get("OBX-5")?.as_deref(), Some("Reading was 90|100 mmHg"));`;

  const nulls = `// "Delete the value you currently hold for this field."
message.set_null("PID-11")?;

// "Send nothing about this field." Not the same thing.
message.set("PID-11", "")?;`;

  const segments = `message.append_segment("NTE");
message.set("NTE[2]-1", "2")?;
message.set("NTE[2]-3", "Amended after review.")?;`;

  const render = `let er7 = message.to_er7();

// An unmodified message writes back byte for byte. That guarantee is er7's,
// and hl7-2 does not weaken it — so a read-modify-write touches only what
// you actually changed.`;

  const builder = `let ack = hl7_2::builder::acknowledge(&message, "AA", "ACK00001", "20260814080100")
    .build_valid()?;

assert_eq!(ack.get("MSA-1")?.as_deref(), Some("AA"));
assert_eq!(ack.get("MSA-2")?.as_deref(), Some("MSG00042"));`;

  const nack = `let mut nack = hl7_2::builder::acknowledge(&message, "AE", "N1", "20260814080100")
    .build_valid()?;
nack.set("MSA-3", "OBR-4 is required")?;`;

  const cli = `# Change something and write it back out
hl7-v2 --set 'PID-8=F' --er7 in.hl7 > out.hl7

# Several edits in one pass; --set is repeatable
hl7-v2 --set 'PID-8=F' --set 'PID-5.2=EVELYN' --er7 in.hl7

# Write the explicit HL7 null
hl7-v2 --null 'PID-11' --er7 in.hl7`;
</script>

<DocPage
  lede="A system that reads HL7 usually has to answer in it. Setting values, adding segments, rendering valid ER7 back out, and building an acknowledgement that names the message it answers."
  {contents}
>
  <h2 id="set">Setting a value</h2>
  <p>
    <code>set</code> takes the same <a href="/guides/navigating/#path-grammar">path grammar</a> as
    <code>get</code>, so anything you can read you can write.
  </p>
  <CodeSample language="rust" code={set} />
  <p>
    Setting a path that does not exist yet creates the intermediate structure — components,
    repetitions, and fields are filled in as needed. Setting a path in a segment that does not
    exist is an error; add the segment first (<a href="#segments">below</a>).
  </p>

  <h2 id="escaping">Escaping happens for you</h2>
  <p>
    A value that contains a delimiter character has to be escaped, or it will re-parse as extra
    structure. <code>set</code> does that.
  </p>
  <CodeSample language="rust" code={escaping} />
  <Callout type="warning" heading="Do not pre-escape">
    <p>
      If you escape the value yourself before calling <code>set</code>, it will be escaped twice
      and the reader at the other end will see a literal <code>\F\</code> in the text. Pass the
      value you mean; the library owns the encoding.
    </p>
  </Callout>

  <h2 id="null">Deleting versus emptying</h2>
  <p>
    HL7 has two different “no value”, and in an update message they do opposite things. Be
    deliberate about which one you write.
  </p>
  <CodeSample language="rust" code={nulls} />
  <p>See <a href="/docs/concepts/#null">the HL7 null</a> for how each is encoded in each format.</p>

  <h2 id="segments">Adding segments</h2>
  <CodeSample language="rust" code={segments} />
  <p>
    The new segment goes at the end. Note the index in the paths afterwards: if the message already
    had one <code>NTE</code>, the one you just added is <code>NTE[2]</code>.
  </p>

  <h2 id="render">Rendering back to ER7</h2>
  <CodeSample language="rust" code={render} />
  <p>
    Byte-for-byte round-tripping of an unmodified message is a property of the
    <a href="https://crates.io/crates/er7"><code>er7</code></a> layer, and it matters more than it
    might seem: it means a message that passes through your system unchanged is provably unchanged,
    which is a question auditors ask.
  </p>

  <h2 id="builder">Building a message from nothing</h2>
  <p>
    <code>hl7_2::Builder</code> constructs a message rather than editing one. Its
    <code>encode</code> method takes a value implementing <code>ToHl7</code>, so a struct with
    <a href="/guides/struct-mode/">derive</a> can be written out directly. <code>build_valid</code>
    runs validation and refuses to hand you something the dictionary says is malformed.
  </p>

  <h2 id="ack">Acknowledgements</h2>
  <p>
    The reply an HL7 v2 sender waits for is an HL7 message: an <code>ACK</code> whose
    <code>MSA-2</code> echoes the control ID of the message being answered.
  </p>
  <CodeSample language="rust" code={builder} />
  <p>
    That echo is the whole mechanism. A sender that does not compare it will eventually take one
    answer for another's — which is why every call here takes the acknowledgement's own control ID
    and timestamp as arguments rather than inventing them. A message that invents them is
    untestable and untraceable.
  </p>
  <p>To reject something, say why:</p>
  <CodeSample language="rust" code={nack} />
  <p>
    The codes are HL7's: <code>AA</code> accept, <code>AE</code> error, <code>AR</code> reject.
    <code>MSA-3</code> is the human-readable reason. To put one on a socket, see
    <a href="/guides/mllp/#acknowledgement">MLLP acknowledgement</a>.
  </p>
  <Callout type="warning" heading="AA is a promise about your system, not about the parse">
    <p>
      Sending <code>AA</code> tells the sender the message is safe — usually meaning it has been
      persisted and will not be lost. Acknowledging before you have written it down turns a
      restart into silent data loss. No library can make that true for you; it is your
      application's job to acknowledge at the right moment.
    </p>
  </Callout>

  <h2 id="cli">The same, from the shell</h2>
  <CodeSample language="sh" code={cli} />
  <p>
    Useful for a one-off correction to a stuck message without writing a program. Full flags:
    <a href="/docs/cli/#hl7-v2">Command line</a>.
  </p>
</DocPage>
