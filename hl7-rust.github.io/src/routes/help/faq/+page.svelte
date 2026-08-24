<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';

  const contents = [
    { id: 'choosing', label: 'Choosing' },
    { id: 'v2', label: 'HL7 v2' },
    { id: 'formats', label: 'Formats and conversion' },
    { id: 'v3', label: 'HL7 v3 and FHIR' },
    { id: 'transports', label: 'Transports' },
    { id: 'project', label: 'The project itself' }
  ];

  const nullCheck = `// get returns the text at a path, and an explicit null and an empty
// field both come back as an empty string. When the difference matters,
// read the node from the tree instead of the value from the path.
let node = message.tree().find("PID.2");`;

  const forceVersion = `let options = hl7_2::Options::new().with_version(hl7_2::Version::V2_5);
let message = hl7_2::parse_with_options(text, &options)?;`;
</script>

<DocPage
  title="FAQ"
  lede="The questions that come up before the first line of code, and a few that come up on the second day."
  {contents}
>
  <h2 id="choosing">Choosing</h2>

  <h3>Which crate should I depend on?</h3>
  <p>
    <code>hl7</code>, and use <code>hl7::v2</code>. It is a thin re-export of
    <a href="/crates/hl7-2/"><code>hl7-2</code></a>, where essentially all the v2 capability lives.
    Depend on <code>hl7-2</code> directly if you want one standard with no umbrella indirection —
    there is no functional difference.
  </p>

  <h3>Do I have to take all fourteen crates?</h3>
  <p>
    No. Each is a layer you can stop at. Most integrations use one or two:
    <code>hl7</code> plus a transport, or just one conversion binary and no Rust at all. See
    <a href="/docs/architecture/">Architecture</a>.
  </p>

  <h3>Which parsing mode should I start in?</h3>
  <p>
    Generic, always — even when you know the feed. It rejects nothing, so it is the mode you explore
    in. Schema mode and struct mode are what you graduate to. See
    <a href="/guides/parsing/">Parsing</a>.
  </p>

  <h3>Is there an async API?</h3>
  <p>
    No. The parsing crates are synchronous and CPU-bound, and
    <a href="/crates/hl7-2-mllp/"><code>hl7-2-mllp</code></a>'s
    <code>IoTransport</code> works over any <code>Read</code> + <code>Write</code>. Wrapping the
    transport for an async runtime is left to you rather than being baked in, so the crate does not
    pick a runtime on your behalf.
  </p>

  <h2 id="v2">HL7 v2</h2>

  <h3>Which HL7 releases are supported?</h3>
  <p>
    The fourteen published releases from 2.1 to 2.9. v2.5 is the complete base; every other release
    is a delta of it. See <a href="/docs/versions/">Versions and compatibility</a>.
  </p>

  <h3>The sender's <code>MSH-12</code> says 2.3 but they send 2.5 fields. What do I do?</h3>
  <p>Believe the content, not the declaration, and force the release:</p>
  <CodeSample language="rust" code={forceVersion} />

  <h3>My message has a Z-segment. Will it parse?</h3>
  <p>
    Yes, and it will not even produce a warning — the standard says nothing about Z-segments, so
    neither does this crate. The fields read positionally
    (<code>ZDS.1</code>, <code>ZDS.1.1</code>) until you write a
    <a href="/guides/dictionaries/">dictionary</a> that names them.
  </p>

  <h3>Why did my message render flat instead of grouped?</h3>
  <p>
    Because its segments did not fit the structure <code>MSH-9</code> declared — usually a
    Z-segment, or a structure with no built-in grammar. Flat is well-formed and lossless; only the
    nesting is missing. Paths are unaffected either way.
  </p>

  <h3>How do I tell an explicit HL7 null from an empty field?</h3>
  <CodeSample language="rust" code={nullCheck} />
  <p>
    In an update message that difference is the difference between leaving a value alone and erasing
    it. See <a href="/docs/concepts/#null">the HL7 null</a>.
  </p>

  <h3>Why does <code>PID-3.1</code> give me the wrong identifier?</h3>
  <p>
    Because <code>PID-3</code> repeats, and an unindexed path reads the first repetition. Read all of
    them and select by assigning authority — <code>PID-3[n].4</code>. See
    <a href="/guides/navigating/#repetition">Navigating</a>.
  </p>

  <h3>Is it a validator?</h3>
  <p>
    <code>hl7-2</code> checks a message against its dictionary and reports diagnostics, split into
    errors (the message contradicts the dictionary) and warnings (the dictionary does not cover the
    message). It does not check HL7 table membership, field lengths, or clinical sense. The four
    conversion crates check nothing at all.
  </p>

  <h2 id="formats">Formats and conversion</h2>

  <h3>Is the conversion lossless?</h3>
  <p>
    For values, yes, with two documented exceptions: a repetition that was present but entirely blank
    is dropped by the forward encoding, and — in JSON only — a segment name that repeats
    non-adjacently loses its place in the sequence. See
    <a href="/guides/converting/#fallbacks">Converting formats</a>.
  </p>

  <h3>Why is there no official “v2.json”?</h3>
  <p>
    Because HL7 never defined one. The mapping here is this project's own, designed to preserve
    everything the official v2.xml encoding preserves while using idiomatic JSON.
  </p>

  <h3>Why is every JSON value a string?</h3>
  <p>
    HL7 numeric text carries leading zeros, explicit signs, and trailing precision. A JSON number
    would silently destroy all three, and a lost leading zero in an identifier is a wrong patient.
  </p>

  <h3>The round trip does not match my input file byte for byte.</h3>
  <p>
    It is not supposed to. The output is the original message
    <em>canonicalized</em>. Compare against <code>hl7-v2 --er7 input.hl7</code> rather than against
    the raw file. See <a href="/tutorials/round-trip/">A lossless round trip</a>.
  </p>

  <h3>Can I convert against my vendor's XSDs rather than the bundled v2.5 tables?</h3>
  <p>
    Yes — generate a dictionary with
    <a href="/crates/hl7-2-from-xsd-into-json-dictionary/"
      ><code>hl7-2-from-xsd-into-json-dictionary</code></a
    >, and pass <code>--dictionary</code> (and, for schema-shaped output,
    <code>--schema-shape</code>) to the converter.
  </p>

  <h2 id="v3">HL7 v3 and FHIR</h2>

  <h3>How complete is the v3 support?</h3>
  <p>
    It is a foundation: the RIM backbone classes, the data types, and the three-level envelope. No
    specific interaction, no vocabulary validation, no CDA document model, and no writing. Read
    <a href="/crates/hl7-3/"><code>hl7-3</code></a>'s <code>spec/index.md</code> §1 before depending
    on it.
  </p>

  <h3>Can I write HL7 v3 XML?</h3>
  <p>Not yet. <code>hl7-3</code> reads; there is no XML-writing capability, which is also why there
    is no <code>#[derive(ToElement)]</code>.</p>

  <h3>Does <code>hl7</code>'s <code>derive</code> feature give me the v3 macro?</h3>
  <p>
    No — it forwards to <code>hl7-2</code>'s only. Enable <code>derive</code> on
    <code>hl7-3</code> for <code>#[derive(FromElement)]</code>.
  </p>

  <h3>Is FHIR supported?</h3>
  <p>
    Not yet. The umbrella crate deliberately leaves <code>hl7::fhir</code> free so it can land there
    when it is implemented, alongside <code>hl7::v2</code> and <code>hl7::v3</code>.
  </p>

  <h2 id="transports">Transports</h2>

  <h3>Does the MLLP crate do TLS?</h3>
  <p>
    No, and it should not — compose it. <code>IoTransport</code> takes any stream, so a TLS stream
    from whichever TLS crate you already audit drops straight in.
  </p>

  <h3>My sender omits the trailing <code>&lt;CR&gt;</code>. Can I accept that?</h3>
  <p>
    Yes: <code>Framer::new().with_tolerance(Tolerance::Lenient)</code>, per connection. Prefer that
    over the crate-wide <code>noncompliance</code> feature, which loosens every connection at once.
  </p>

  <h3>When should I send <code>AA</code>?</h3>
  <p>
    After the message is persisted, not after it parses. <code>AA</code> tells the sender they may
    forget it. <code>AE</code> means there is an error in their message; <code>AR</code> means you
    are rejecting for reasons unrelated to its content — say the true one, because their retry logic
    depends on it.
  </p>

  <h3>Do the SOAP crates include an HTTP server?</h3>
  <p>
    No. They turn bytes into meaning and back, and leave the socket to whatever you already use.
  </p>

  <h2 id="project">The project itself</h2>

  <h3>What is the minimum Rust version?</h3>
  <p>
    Current stable minus three releases, pinned in every crate. Raising it is treated as a breaking
    change. See <a href="/docs/versions/#msrv">Versions and compatibility</a>.
  </p>

  <h3>Why five licenses?</h3>
  <p>
    So you can pick the one your organization has already cleared. <code>OR</code> means exactly
    that: comply with one, and you do not have to say which.
  </p>

  <h3>Why are the dependency counts so small?</h3>
  <p>
    Healthcare software gets audited, and a dependency tree is part of what gets audited. The JSON
    reader in <code>hl7-2</code>, the JSON writer in the dictionary builder, and the shared XML reader
    are all hand-written for that reason.
  </p>

  <h3>Where does <code>er7</code> live?</h3>
  <p>
    In <a href="https://github.com/er7-rust/er7-rust">er7-rust/er7-rust</a> — its own repository in its own
    organization, outside this workspace, because plenty of people want the ER7 encoding without
    the HL7 dictionary.
  </p>

  <h3>Something on this site is wrong. Where do I say so?</h3>
  <p><a href="/help/support/">Support and contributing</a>.</p>
</DocPage>
