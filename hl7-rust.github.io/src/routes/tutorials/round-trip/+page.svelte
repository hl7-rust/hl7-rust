<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'why', label: 'Why the round trip is the test' },
    { id: 'shell', label: '1. From the shell' },
    { id: 'canonical', label: '2. Canonicalized, not identical' },
    { id: 'json', label: '3. The same through JSON' },
    { id: 'test', label: '4. As a test in your own crate' },
    { id: 'lossy', label: '5. Where it is genuinely lossy' },
    { id: 'batch', label: '6. Over a directory' }
  ];

  const shell = `# ER7 → v2.xml → ER7
hl7-2-from-er7-into-xml samples/orm_o01.hl7 \\
  | hl7-2-from-xml-into-er7

# The whole assertion, in one line
hl7-2-from-er7-into-xml samples/orm_o01.hl7 \\
  | hl7-2-from-xml-into-er7 \\
  | diff - <(hl7-2-from-er7-into-xml samples/orm_o01.hl7 \\
             | hl7-2-from-xml-into-er7) && echo "stable"`;

  const canonical = `# The output is the ORIGINAL message, canonicalized. Compare against a
# canonicalized original rather than the raw file:
hl7-v2 --er7 samples/orm_o01.hl7 > canonical.hl7

hl7-2-from-er7-into-xml samples/orm_o01.hl7 \\
  | hl7-2-from-xml-into-er7 \\
  | diff - canonical.hl7 && echo "round trip is lossless"`;

  const json = `hl7-2-from-er7-into-json samples/orm_o01.hl7 \\
  | hl7-2-from-json-into-er7 \\
  | diff - canonical.hl7`;

  const test = `#[test]
fn er7_survives_a_trip_through_xml() {
    let original = include_str!("../samples/orm_o01.hl7");

    let xml = hl7_2_from_er7_into_xml::convert(original).expect("into xml");
    let back = hl7_2_from_xml_into_er7::convert(&xml).expect("back into er7");

    // Canonicalize the original the same way the pipeline does, so the
    // comparison is about the conversion rather than about whitespace.
    let canonical = hl7_2::parse(original).expect("parse").to_er7();
    assert_eq!(back, canonical);
}`;

  const property = `#[test]
fn every_sample_survives_both_round_trips() {
    for entry in std::fs::read_dir("samples").expect("samples directory") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("hl7") {
            continue;
        }
        let original = std::fs::read_to_string(&path).expect("read");

        for message in hl7_2_from_er7_into_xml::split_messages(&original) {
            let canonical = hl7_2::parse(&message).expect("parse").to_er7();

            let xml = hl7_2_from_er7_into_xml::convert(&message).expect("into xml");
            assert_eq!(
                hl7_2_from_xml_into_er7::convert(&xml).expect("back"),
                canonical,
                "xml round trip failed for {}", path.display(),
            );

            let json = hl7_2_from_er7_into_json::convert(&message).expect("into json");
            assert_eq!(
                hl7_2_from_json_into_er7::convert(&json).expect("back"),
                canonical,
                "json round trip failed for {}", path.display(),
            );
        }
    }
}`;

  const blank = `# A repetition that is present but entirely blank:
PID|1||111~~333

# The middle repetition carries no value at all, so the forward encoding
# has nothing to write for it — and the reverse cannot invent it back.
# The explicit null is different, and DOES survive:
PID|1||111~""~333`;

  const nonadjacent = `# JSON only. A segment name that repeats NON-ADJACENTLY:
NTE|1||first
OBX|1|NM|...
NTE|2||second

# A JSON object cannot carry the key "NTE" twice, so the second NTE is
# grouped with the first — and the OBX that sat between them loses its
# place in the sequence. Adjacent repeats, and repeats inside a group the
# grammar knows, are unaffected.`;

  const directory = `#!/bin/sh
# Round-trip every message in a directory and report the ones that drift.
status=0
for message in samples/*.hl7; do
    canonical=$(hl7-v2 --er7 "$message")
    actual=$(hl7-2-from-er7-into-xml "$message" | hl7-2-from-xml-into-er7)
    if [ "$canonical" != "$actual" ]; then
        echo "drift: $message"
        status=1
    fi
done
exit $status`;
</script>

<DocPage
  eyebrow="Tutorial"
  lede="ER7 to XML and back, ER7 to JSON and back — as a shell pipeline, and as a test in your own crate. It is the smoke test the conversion crates use on themselves, and it is the fastest way to find out whether a change broke something."
  {contents}
>
  <h2 id="why">Why the round trip is the test</h2>
  <p>
    The four conversion crates split into two forward crates that need the HL7 v2.5 dictionary, and
    two reverse crates that need none — because every name a forward crate writes carries its own
    position. That makes the pair tightly coupled in one specific way: a change to the forward
    crate's naming rules silently breaks the reverse crate's assumptions.
  </p>
  <p>
    So the round trip is not a nice-to-have. It is the assertion that the two halves still agree,
    and it costs one shell pipeline to run.
  </p>

  <h2 id="shell">1. From the shell</h2>
  <CodeSample language="sh" code={shell} />
  <p>
    If you are working inside the workspace rather than with installed binaries, the same thing runs
    as <code>cargo run -p hl7-2-from-er7-into-xml -- samples/orm_o01.hl7 | cargo run -p
    hl7-2-from-xml-into-er7 --</code> — both crates live in one workspace, so no
    <code>cd</code> is needed.
  </p>

  <h2 id="canonical">2. Canonicalized, not identical</h2>
  <Callout heading="The output is the original message, canonicalized">
    <p>
      Do not diff against the raw input file. The forward crate reads through <code>er7</code> and
      the reverse crate writes through it, so the result is normalized: a consistent segment
      terminator, trailing empty fields resolved consistently, escapes re-emitted in the crate's own
      form. Diff against a canonicalized original instead — which is exactly what
      <code>hl7-v2 --er7</code> produces.
    </p>
  </Callout>
  <CodeSample language="sh" code={canonical} />
  <p>Each crate's <code>spec/index.md</code> §2.1 defines precisely what canonicalization means.</p>

  <h2 id="json">3. The same through JSON</h2>
  <CodeSample language="sh" code={json} />
  <p>
    Both pipelines should land on the same canonical text, which is a useful property in itself: the
    two mappings are independent implementations of the same idea, and if they disagree, one of them
    has a bug.
  </p>

  <h2 id="test">4. As a test in your own crate</h2>
  <p>
    Worth having if you depend on any of the four, and essential if you have written a
    <a href="/guides/dictionaries/">vendor dictionary</a> — a dictionary that names a field wrongly
    will still round-trip, but a dictionary that names it <em>inconsistently</em> will not.
  </p>
  <CodeSample language="rust" code={test} />
  <p>And over every sample you have, in both directions:</p>
  <CodeSample language="rust" code={property} />
  <p>
    Note <code>split_messages</code>: a sample file may hold one message, several, or an HL7 batch.
    Round-tripping the file as a unit would compare the wrong things.
  </p>

  <h2 id="lossy">5. Where it is genuinely lossy</h2>
  <p>
    Two cases will not round-trip, and both are properties of the forward encoding rather than
    defects in the reversal. Knowing them saves an afternoon of debugging.
  </p>
  <h3>A blank repetition</h3>
  <CodeSample language="er7" code={blank} />
  <h3>A non-adjacent repeating segment, in JSON</h3>
  <CodeSample language="er7" code={nonadjacent} />
  <p>
    The XML mapping does not have this problem, because repeated sibling elements keep their
    document order. It is the one place the two formats diverge in a way that costs information
    rather than only looking different — see
    <a href="/guides/converting/#differences">Where XML and JSON deliberately diverge</a>.
  </p>

  <h2 id="batch">6. Over a directory</h2>
  <p>
    Useful in CI, or as a one-off against a corpus of real messages before trusting a conversion in
    production.
  </p>
  <CodeSample language="sh" code={directory} />
  <p>
    Run it against real traffic, not against the samples in the repository. The samples are the
    cases the crates already handle; your corpus is where the interesting failures are, and every
    one you find is worth reporting — see <a href="/help/support/">Support and contributing</a>.
  </p>
</DocPage>
