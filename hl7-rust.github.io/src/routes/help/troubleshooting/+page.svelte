<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'first', label: 'Start here' },
    { id: 'parse', label: 'Parsing fails' },
    { id: 'reading', label: 'Reading gives the wrong thing' },
    { id: 'writing', label: 'Writing fails or writes the wrong thing' },
    { id: 'validation', label: 'Validation says something unexpected' },
    { id: 'conversion', label: 'Conversion output is not what you expected' },
    { id: 'transport', label: 'Transport problems' },
    { id: 'build', label: 'Build and dependency problems' },
    { id: 'still-stuck', label: 'Still stuck' }
  ];

  const triage = `# Does it parse at all, and what is actually in it?
hl7-v2 --paths message.hl7

# What does the dictionary think is wrong with it?
hl7-v2 --check message.hl7; echo "exit $?"

# Does the path you are using name anything?
hl7-v2 --query 'PID-3.1' message.hl7`;

  const showBytes = `# Segment terminators are carriage returns. If a file has been through a
# text editor or a Windows share, check what is actually in it.
od -c message.hl7 | head`;

  const nullTrap = `// Both come back as Some("") — the explicit null and an empty field.
message.get("PID-2")?;

// Read the node when you need to tell them apart.
message.tree().find("PID.2");`;

  const repetition = `// Wrong: reads whichever identifier the sender happened to put first.
let id = message.get("PID-3.1")?;

// Right: select by assigning authority.
for index in 1..=message.repetitions("PID-3")?.len() {
    if message.get(&format!("PID-3[{index}].4"))?.as_deref() == Some("NHS") {
        let id = message.get(&format!("PID-3[{index}].1"))?;
    }
}`;

  const noSegment = `// Error::NoSuchSegment — the write named a segment the message does not have.
message.append_segment("NTE");
message.set("NTE[2]-3", "Amended.")?;`;

  const doubleEscape = `// Wrong: the value is escaped twice, and the far end sees a literal \\F\\
message.set("OBX-5", "90\\\\F\\\\100")?;

// Right: pass the value you mean. The library owns the encoding.
message.set("OBX-5", "90|100")?;`;
</script>

<DocPage
  title="Troubleshooting"
  lede="Symptom, cause, and fix for the failures people actually hit. Most of them are one of four things: a repetition, the HL7 null, a release mismatch, or a segment that does not exist yet."
  {contents}
>
  <h2 id="first">Start here</h2>
  <p>
    Three commands answer most questions faster than reading code, and none of them needs a project.
  </p>
  <CodeSample language="sh" code={triage} />
  <p>
    Exit status from <code>--check</code> is meaningful: 0 is fine, 1 means it is not a message at
    all, and 2 means it is a message and something is wrong with it.
  </p>

  <h2 id="parse">Parsing fails</h2>
  <dl>
    <dt><code>Error::Empty</code> — “input contains no HL7 segments”</dt>
    <dd>
      The input was empty or whitespace. If you read from a socket, you may have read zero bytes; if
      from a file, check you are not passing a directory or an empty file.
    </dd>
    <dt><code>Error::MissingMsh</code> — “message does not start with an MSH segment”</dt>
    <dd>
      Something is in front of the header. Common causes: a byte-order mark, a leading blank line, an
      MLLP start block that was not stripped, or an HTTP body still carrying its headers. Also check
      you are not handing a whole batch file to <code>parse</code> — use
      <code>split_messages</code> first.
    </dd>
    <dt><code>Error::BadMshHeader</code> — “malformed MSH header”</dt>
    <dd>
      <code>MSH-1</code> and <code>MSH-2</code> declare the delimiters, and this message's are
      unusable. Look at the raw bytes: a file that has been through a text editor may have had its
      carriage returns converted, which can merge the header with the next segment.
    </dd>
  </dl>
  <CodeSample language="sh" code={showBytes} />
  <Callout heading="Line endings are the most common cause of all three">
    <p>
      HL7 v2 terminates segments with a carriage return (<code>\r</code>), not a newline. A file
      committed to git, opened in an editor, or fetched through a Windows share may have had them
      rewritten. The parser accepts both, but a file whose terminators were <em>stripped</em> is one
      long segment.
    </p>
  </Callout>

  <h2 id="reading">Reading gives the wrong thing</h2>
  <dl>
    <dt><code>get</code> returns <code>None</code> for a path you can see in the message</dt>
    <dd>
      Check the index. Paths are one-based, and a segment that occurs more than once needs
      <code>SEG[n]</code>. Run <code>hl7-v2 --paths</code> and copy the bracketed path exactly.
    </dd>
    <dt><code>get</code> returns an empty string and you cannot tell what that means</dt>
    <dd>
      An explicit HL7 null and an empty field both read as an empty string from a path.
      <CodeSample language="rust" code={nullTrap} />
    </dd>
    <dt>The wrong identifier comes back</dt>
    <dd>
      <code>PID-3</code> repeats. An unindexed path reads the first repetition, and the order is not
      guaranteed.
      <CodeSample language="rust" code={repetition} />
    </dd>
    <dt>A component reads as a whole field, or splits in the wrong place</dt>
    <dd>
      The dictionary does not know that field's data type — either because the release resolved
      differently from what you expected, or because the field is past the end of the published
      segment. Check with <code>message.type_of("PID-3")</code>, and see
      <a href="/guides/dictionaries/">Vendor dictionaries</a> to name it.
    </dd>
    <dt><code>find</code> returns nothing for a name you can see in the tree</dt>
    <dd>
      Names come from the dictionary, so a field it does not recognise has a positional name
      (<code>PID.5.1</code>) rather than a typed one (<code>XPN.1</code>). Search for the positional
      form, or supply a dictionary.
    </dd>
  </dl>

  <h2 id="writing">Writing fails or writes the wrong thing</h2>
  <dl>
    <dt><code>Error::NoSuchSegment</code></dt>
    <dd>
      The write named a segment the message does not have. Add it first — and remember the index
      afterwards.
      <CodeSample language="rust" code={noSegment} />
    </dd>
    <dt><code>Error::UnwritablePath</code></dt>
    <dd>
      The path named something that cannot be written: a whole segment, or a repeating value without
      a field. Address the specific field.
    </dd>
    <dt>The far end sees a literal <code>\F\</code> in the text</dt>
    <dd>
      The value was escaped twice.
      <CodeSample language="rust" code={doubleEscape} />
    </dd>
    <dt>A field you meant to clear came out as a deletion instruction, or the other way round</dt>
    <dd>
      <code>set_null</code> writes the explicit HL7 null, meaning “delete the value you hold”.
      <code>clear</code> removes the value entirely, meaning “nothing is being said about this”. In an
      update message they do opposite things.
    </dd>
  </dl>

  <h2 id="validation">Validation says something unexpected</h2>
  <dl>
    <dt>Warnings about segments and fields you know are fine</dt>
    <dd>
      A warning means the dictionary does not cover the message, not that the message is wrong. It is
      the signal to write a dictionary — not a reason to reject.
    </dd>
    <dt>Strict mode rejects a message the sender insists is valid</dt>
    <dd>
      Look at which diagnostics are error-level. The four error kinds are a missing
      <code>MSH-9.1</code> or <code>MSH-10</code>, a required segment or group absent, segments that
      do not fit the structure, and an <code>SI</code>/<code>NM</code>/<code>DT</code>/<code>TM</code
      >/<code>DTM</code> value that is not one. A structure mismatch caused only by Z-segments is a
      warning, not an error.
    </dd>
    <dt>Nothing is reported about a field you know is wrong</dt>
    <dd>
      Only value formats with a machine-checkable shape are checked. <code>ST</code>,
      <code>TX</code>, <code>ID</code>, and <code>IS</code> are constrained by HL7 tables and lengths,
      which are not modelled — the crate says nothing rather than guessing. Table membership and
      clinical plausibility are your domain layer's job.
    </dd>
  </dl>

  <h2 id="conversion">Conversion output is not what you expected</h2>
  <dl>
    <dt>Everything came out flat instead of grouped</dt>
    <dd>
      The message's segments did not fit its declared structure — usually a Z-segment, or a structure
      with no built-in grammar. Grammars exist for ACK, ADT_A01 (and A04/A08/A13), ORM_O01, and
      ORU_R01. Flat output is well-formed and lossless.
    </dd>
    <dt>Elements are named <code>PID.5.1</code> instead of <code>XPN.1</code></dt>
    <dd>
      The dictionary did not recognise the type at that position. Supply one with
      <code>--dictionary</code>.
    </dd>
    <dt>A JSON key is sometimes a string and sometimes an array</dt>
    <dd>
      By design: a repeatable key is a bare value when it occurs once and an array only when it
      occurs more than once. Consumers that need a uniform shape must normalize.
    </dd>
    <dt>A segment vanished from the JSON output</dt>
    <dd>
      A segment name that repeats <em>non-adjacently</em> is grouped at its first occurrence, because
      a JSON object cannot carry the same key twice. Use the XML mapping if document order matters.
    </dd>
    <dt>The round trip does not match the input file</dt>
    <dd>
      It is not meant to — the output is the original message canonicalized. Compare against
      <code>hl7-v2 --er7 input.hl7</code>. See <a href="/tutorials/round-trip/">A lossless round
      trip</a>.
    </dd>
    <dt>The XML does not validate against the vendor's XSDs</dt>
    <dd>
      Pass both <code>--dictionary</code> and <code>--schema-shape</code>. Without the second, the
      dictionary only names what a field is; with it, the dictionary decides what the document
      contains.
    </dd>
  </dl>

  <h2 id="transport">Transport problems</h2>
  <dl>
    <dt><code>receive</code> returns an <code>InvalidData</code> error</dt>
    <dd>
      Framing was violated — a frame did not start with <code>&lt;VT&gt;</code>, did not end with
      <code>&lt;FS&gt;&lt;CR&gt;</code>, or contained a block character in between. A stream that has
      lost framing cannot be resynchronized: close the connection. If the sender's only sin is a
      missing trailing <code>&lt;CR&gt;</code>, set <code>Tolerance::Lenient</code> for that
      connection.
    </dd>
    <dt>The connection ends after one message</dt>
    <dd>
      Keep looping on <code>receive</code>. MLLP connections are long-lived and carry many messages;
      closing after one is a common bug and senders notice immediately.
    </dd>
    <dt>The sender keeps retrying even though you answered</dt>
    <dd>
      Check that <code>MSA-2</code> echoes the incoming <code>MSH-10</code>. That echo is the only
      thing that says which message you answered.
    </dd>
    <dt>A frame is never delivered and memory grows</dt>
    <dd>
      A peer that never sends an end block. The <code>Framer</code> caps its buffer at 16 MiB by
      default and errors past it; lower the limit to your largest real message plus headroom, and add
      a read timeout.
    </dd>
    <dt>A SOAP endpoint rejects your envelope, or you reject theirs</dt>
    <dd>
      Prefixes are matched on local name, so that is not it. Check for more than one child in the
      body — one payload per body is enforced deliberately.
    </dd>
    <dt>A SOAP sender retries forever against a working endpoint</dt>
    <dd>
      The two acceptance conventions. <code>response::evaluate</code> accepts
      <code>AA</code>, <code>CA</code>, and <code>Success</code> in a v2 <code>Status</code> element;
      an endpoint writing something else will read as a rejection.
    </dd>
  </dl>

  <h2 id="build">Build and dependency problems</h2>
  <dl>
    <dt><code>cannot find derive macro FromHl7</code></dt>
    <dd>
      The <code>derive</code> feature is off. Add
      <code>features = ["derive"]</code> to <code>hl7-2</code> or to <code>hl7</code>.
    </dd>
    <dt><code>cannot find derive macro FromElement</code></dt>
    <dd>
      Same, but on <code>hl7-3</code> specifically — the umbrella crate's
      <code>derive</code> feature forwards to <code>hl7-2</code>'s only.
    </dd>
    <dt><code>acknowledge_now</code> does not exist</dt>
    <dd>It is behind <code>hl7-2-mllp</code>'s <code>clock</code> feature, which is off by default.</dd>
    <dt><code>ack</code> module does not exist</dt>
    <dd>
      You built with <code>--no-default-features</code>, which turns off the <code>ack</code> feature
      to give you framing with zero dependencies.
    </dd>
    <dt>The crate does not build on your toolchain</dt>
    <dd>
      Check your Rust version against the crate's <code>rust-version</code>. The floor is current
      stable minus two releases. If you are at or above it and it still fails, that is a bug worth
      reporting.
    </dd>
    <dt><code>cargo install hl7-2</code> succeeded but <code>hl7-2</code> is not a command</dt>
    <dd>The binary is named <code>hl7-v2</code>.</dd>
  </dl>

  <h2 id="still-stuck">Still stuck</h2>
  <p>
    Read the crate's <code>spec/index.md</code> — it is numbered section by section, and the section
    that covers your symptom usually says exactly what the crate does and why. If the spec says one
    thing and the crate does another, that is a bug and worth reporting with the section number. See
    <a href="/help/support/">Support and contributing</a>.
  </p>
</DocPage>
