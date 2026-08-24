<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'binaries', label: 'The six binaries' },
    { id: 'hl7-v2', label: 'hl7-v2 — the HL7 v2 tool' },
    { id: 'converters', label: 'The four converters' },
    { id: 'dictionary-builder', label: 'The dictionary builder' },
    { id: 'exit-status', label: 'Exit status, and using it in a shell' },
    { id: 'recipes', label: 'Recipes' }
  ];

  const usage = `hl7-v2 [OPTIONS] [FILE]`;

  const examples = `# Look at a message you have never seen
hl7-v2 samples/oru_r01.hl7

# Pull out every result value
hl7-v2 --query OBX-5 samples/oru_r01.hl7

# Check it, with an exit status a shell can act on
hl7-v2 --check samples/adt_a01.hl7

# Read a vendor dialect
hl7-v2 --dictionary samples/acme.json samples/vendor.hl7

# Change something and write it back out
hl7-v2 --set 'PID-8=F' --er7 samples/orm_o01.hl7`;

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

  const forward = `# From a file to stdout
hl7-2-from-er7-into-xml samples/orm_o01.hl7

# From stdin, to a file
cat samples/oru_r01.hl7 | hl7-2-from-er7-into-xml -o out.xml

# Disable message-structure grouping
hl7-2-from-er7-into-xml --flat samples/orm_o01.hl7

# Convert against a dictionary built from XSDs instead of bundled v2.5
hl7-2-from-er7-into-xml --dictionary my-dialect.json samples/orm_o01.hl7

# Let that dictionary decide the document's exact shape
hl7-2-from-er7-into-xml --dictionary my-dialect.json --schema-shape samples/orm_o01.hl7`;

  const json = `# Compact (single-line) JSON instead of pretty-printed
hl7-2-from-er7-into-json --compact samples/orm_o01.hl7`;

  const reverse = `# v2.xml back to ER7
hl7-2-from-xml-into-er7 samples/orm_o01.xml

# Typed JSON back to ER7
hl7-2-from-json-into-er7 samples/orm_o01.json

# Choose the segment terminator
hl7-2-from-xml-into-er7 --terminator crlf samples/orm_o01.xml`;

  const xsd = `hl7-2-from-xsd-into-json-dictionary schemas/paris -o paris.json

# Two things the schemas cannot tell you, so you pass them in
hl7-2-from-xsd-into-json-dictionary schemas/paris \\
    --name paris \\
    --alias ADT_A28=ADT_A05 \\
    --alias ADT_A31=ADT_A05 \\
    -o paris.json

# Layer the document over a bundled release instead of standing alone
hl7-2-from-xsd-into-json-dictionary schemas/paris --inherits 2.5 -o paris.json`;

  const shell = `#!/bin/sh
# Reject anything that does not conform before it reaches the interface engine.
for message in inbox/*.hl7; do
    if hl7-v2 --check "$message" >/dev/null; then
        mv "$message" accepted/
    else
        hl7-v2 --check "$message" >> rejected.log
        mv "$message" rejected/
    fi
done`;

  const recipes = `# Every patient identifier in a directory of messages
cat inbox/*.hl7 | hl7-v2 --query PID-3

# Fix a sex code and re-emit, in one pass
hl7-v2 --set 'PID-8=F' --er7 in.hl7 > out.hl7

# Clear a field with the explicit HL7 null rather than emptying it
hl7-v2 --null 'PID-11' --er7 in.hl7

# Force a release when the sender's MSH-12 lies
hl7-v2 --hl7-version 2.3 vendor.hl7

# Round trip, as a smoke test
hl7-2-from-er7-into-xml in.hl7 | hl7-2-from-xml-into-er7 | diff - in.hl7`;
</script>

<DocPage
  title="Command line"
  lede="Six crates ship a binary. They are useful without writing any Rust at all — looking at an unfamiliar message, checking a batch in a shell script, converting a directory of files."
  {contents}
>
  <h2 id="binaries">The six binaries</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Binary</th><th>From crate</th><th>Does</th></tr>
      </thead>
      <tbody>
        <tr>
          <td><code>hl7-v2</code></td>
          <td><a href="/crates/hl7-2/"><code>hl7-2</code></a></td>
          <td>Tree, query, check, edit, and re-emit an HL7 v2 message</td>
        </tr>
        <tr>
          <td><code>hl7-2-from-er7-into-xml</code></td>
          <td><a href="/crates/hl7-2-from-er7-into-xml/">same name</a></td>
          <td>ER7 → v2.xml</td>
        </tr>
        <tr>
          <td><code>hl7-2-from-xml-into-er7</code></td>
          <td><a href="/crates/hl7-2-from-xml-into-er7/">same name</a></td>
          <td>v2.xml → ER7</td>
        </tr>
        <tr>
          <td><code>hl7-2-from-er7-into-json</code></td>
          <td><a href="/crates/hl7-2-from-er7-into-json/">same name</a></td>
          <td>ER7 → typed JSON</td>
        </tr>
        <tr>
          <td><code>hl7-2-from-json-into-er7</code></td>
          <td><a href="/crates/hl7-2-from-json-into-er7/">same name</a></td>
          <td>typed JSON → ER7</td>
        </tr>
        <tr>
          <td><code>hl7-2-from-xsd-into-json-dictionary</code></td>
          <td><a href="/crates/hl7-2-from-xsd-into-json-dictionary/">same name</a></td>
          <td>A directory of v2.xml XSDs → the JSON dictionary <code>hl7-2</code> reads</td>
        </tr>
      </tbody>
    </table>
  </div>
  <p>
    Install any of them with <code>cargo install &lt;crate&gt;</code> — see
    <a href="/docs/install/#cli">Install</a>. Note that <code>cargo install hl7-2</code> gives you a
    binary named <code>hl7-v2</code>.
  </p>

  <h2 id="hl7-v2">hl7-v2 — the HL7 v2 tool</h2>
  <CodeSample language="text" code={usage} label="hl7-v2 usage" />
  <p>
    Reads <code>FILE</code>, or standard input when <code>FILE</code> is absent or <code>-</code>.
    Input may hold one message, several, or a batch; each is handled separately.
  </p>

  <h3>Output modes — the first one given wins</h3>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Option</th><th>Prints</th></tr>
      </thead>
      <tbody>
        <tr><td><code>-t, --tree</code></td><td>The message as an indented tree. The default.</td></tr>
        <tr><td><code>-q, --query PATH</code></td><td>Every value at <code>PATH</code>, one per line.</td></tr>
        <tr><td><code>-c, --check</code></td><td>Validation diagnostics, or <code>ok</code>.</td></tr>
        <tr><td><code>-e, --er7</code></td><td>The message back as ER7.</td></tr>
      </tbody>
    </table>
  </div>

  <h3>Options</h3>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Option</th><th>Effect</th></tr>
      </thead>
      <tbody>
        <tr>
          <td><code>-s, --set PATH=VALUE</code></td>
          <td>Set a value before printing. Repeatable.</td>
        </tr>
        <tr>
          <td><code>-n, --null PATH</code></td>
          <td>Write the explicit HL7 null at a path before printing.</td>
        </tr>
        <tr>
          <td><code>-v, --hl7-version VER</code></td>
          <td>Force a release rather than reading it from <code>MSH-12</code>.</td>
        </tr>
        <tr>
          <td><code>-d, --dictionary FILE</code></td>
          <td>Read the message through a JSON schema dictionary.</td>
        </tr>
        <tr><td><code>-f, --flat</code></td><td>Suppress message-structure grouping.</td></tr>
        <tr><td><code>-p, --paths</code></td><td>Show each node's path beside it.</td></tr>
        <tr><td><code>-S, --strict</code></td><td>Fail on a validation error.</td></tr>
        <tr><td><code>-o, --output FILE</code></td><td>Write to a file instead of stdout.</td></tr>
      </tbody>
    </table>
  </div>
  <p>
    Successive trees are separated by a blank line. The value and ER7 outputs are not, so they can
    be piped straight into another tool.
  </p>
  <CodeSample language="sh" code={examples} />

  <Callout heading="--paths is the one to learn first">
    <p>
      The bracketed paths are not decoration: each one is exactly what reads that value back, with
      <code>--query</code>, with <code>get</code>, or in a <code>#[hl7(...)]</code> attribute. It is
      the fastest way to go from “what is in this message” to working code.
    </p>
  </Callout>
  <CodeSample language="sh" code={paths} />

  <h2 id="converters">The four converters</h2>
  <p>
    All four read a file or standard input and write standard output or <code>-o FILE</code>. The
    ER7-side tools accept one message, several, or an HL7 batch file (FHS/BHS envelopes are
    dropped); each message becomes one output document.
  </p>
  <CodeSample language="sh" caption="ER7 → XML, and the same flags apply to ER7 → JSON" code={forward} />
  <p>
    <code>--dictionary FILE</code> reads a JSON dictionary — such as one built by the XSD tool — in
    place of the bundled HL7 v2.5 tables. <code>--schema-shape</code> changes how that dictionary is
    read: instead of only naming what a field <em>is</em>, it decides what the document
    <em>contains</em> — required fields are written even when empty, fields that cannot repeat keep
    their repetition separator as text, and no field the dictionary does not declare is written.
  </p>
  <p>The JSON converter has one flag of its own:</p>
  <CodeSample language="sh" code={json} />
  <p>The two reverse tools take the segment terminator instead:</p>
  <CodeSample language="sh" code={reverse} />
  <p>
    Message-structure group elements and keys are flattened automatically, so grouped and
    <code>--flat</code> input from the forward tool both reconstruct the same message.
  </p>

  <h2 id="dictionary-builder">The dictionary builder</h2>
  <CodeSample language="sh" code={xsd} />
  <p>
    <code>--alias</code> says which trigger events arrive carried by another message's structure. A
    directory holds <code>ADT_A05.xsd</code> but never says that an <code>ADT^A28</code> is one, so
    that mapping has to come from you. See <a href="/guides/dictionaries/">Vendor dictionaries</a>.
  </p>

  <h2 id="exit-status">Exit status, and using it in a shell</h2>
  <p><code>hl7-v2</code> exits:</p>
  <ul>
    <li><strong>0</strong> — success.</li>
    <li><strong>1</strong> — a usage error, or the input was not a message at all.</li>
    <li>
      <strong>2</strong> — <code>--check</code> or <code>--strict</code> found something wrong with
      the message.
    </li>
  </ul>
  <p>
    Those three are distinct on purpose: “this file is not HL7” and “this file is HL7 and it is
    wrong” need different handling, and a script that conflates them will quarantine the wrong
    things.
  </p>
  <CodeSample language="sh" caption="A triage script that uses the distinction" code={shell} />

  <h2 id="recipes">Recipes</h2>
  <CodeSample language="sh" code={recipes} />
  <p>
    <code>hl7-v2 --help</code> lists everything, and each converter's own
    <code>spec/index.md</code> §8 is the normative statement of its command-line contract. See
    <a href="/spec/">Specifications</a>.
  </p>
</DocPage>
