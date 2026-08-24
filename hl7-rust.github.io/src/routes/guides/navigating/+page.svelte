<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'two-vocabularies', label: 'Two vocabularies' },
    { id: 'path-grammar', label: 'The path grammar' },
    { id: 'get', label: 'Reading a value: get' },
    { id: 'tree', label: 'Walking the tree' },
    { id: 'repetition', label: 'Repetition, and the [n] you need' },
    { id: 'groups', label: 'Groups, and reading through them' },
    { id: 'discovering', label: 'Discovering paths you do not know' },
    { id: 'null-trap', label: 'The one thing get cannot tell you' }
  ];

  const grammar = `SEG-F.C.S            segment, field, component, subcomponent
SEG[n]-F             the nth segment with that name
SEG-F[n]             the nth repetition of that field
SEG[n]-F[m].C.S      all of it at once

PID-3                field 3 of the first PID
PID-5.1              component 1 of field 5
PID-3.4.2            subcomponent 2 of component 4 of field 3
OBX[2]-5             field 5 of the second OBX
PID-3[2].1           component 1 of the second repetition of field 3`;

  const get = `// Some, when there is a value there.
assert_eq!(message.get("PID-5.1")?.as_deref(), Some("EVERYWOMAN"));

// None, when the path names nothing at all.
assert_eq!(message.get("ZZZ-1")?, None);

// A path that is not valid syntax is an error, not a None.
assert!(message.get("PID-").is_err());`;

  const tree = `let tree = message.tree();

assert_eq!(tree.name(), "ORU_R01");            // the structure id

let pid = tree.find("PID").unwrap();           // first descendant named PID
assert_eq!(pid.path(), "PID[1]");

let family = tree.find("XPN.1").unwrap();      // first descendant named XPN.1
assert_eq!(family.text(), "EVERYWOMAN");

for observation in tree.find_all("OBX") {
    println!("{} = {}", observation.path(), observation.text());
}`;

  const repetition = `// A repeating field: PID-3 may carry several identifiers.
// PID|1||111^^^A~222^^^B||...
assert_eq!(message.get("PID-3[1].1")?.as_deref(), Some("111"));
assert_eq!(message.get("PID-3[2].1")?.as_deref(), Some("222"));

// Unindexed reads the first repetition.
assert_eq!(message.get("PID-3.1")?.as_deref(), Some("111"));

// From the CLI, --query prints every value at the path, one per line.
// $ hl7-v2 --query 'PID-3.1' message.hl7
// 111
// 222`;

  const groups = `// The tree nests groups when the structure grammar fits:
//   ORU_R01
//     ORU_R01.PATIENT_RESULT
//       ORU_R01.PATIENT
//         PID
//       ORU_R01.ORDER_OBSERVATION
//         OBR
//         ORU_R01.OBSERVATION
//           OBX
//
// Paths do not mention groups. This works either way:
message.get("OBX[2]-5")?;

// So does this, when you want the group node itself:
let group = tree.find("ORU_R01.ORDER_OBSERVATION").unwrap();`;

  const discover = `$ hl7-v2 --paths samples/vendor.hl7
ADT_A01
  ZAC  [ZAC[1]]
    ZAC.1 = 7           [ZAC[1]-1[1]]
    ZAC.2               [ZAC[1]-2[1]]
      ZAC.2.1 = SMITH   [ZAC[1]-2[1].1]
      ZAC.2.2 = JOHN    [ZAC[1]-2[1].2]

$ hl7-v2 --query 'ZAC-2.1' samples/vendor.hl7
SMITH
JONES`;

  const nullTrap = `// Both of these come back as Some("") from get:
//   PID|1||""        the explicit HL7 null — "delete what you have"
//   PID|1||          not sent — "I am telling you nothing about this"
//
// When the difference matters — and in an update message it does — go
// to the tree node rather than the path value.`;
</script>

<DocPage
  title="Navigating"
  lede="How to find a value in a parsed message: the path grammar that addresses one, the tree that names one, and how repetition and groups interact with both."
  {contents}
>
  <h2 id="two-vocabularies">Two vocabularies</h2>
  <p>
    A parsed message exposes two ways of talking about the same value, and they are not
    interchangeable.
  </p>
  <dl>
    <dt>A path addresses</dt>
    <dd>
      <code>PID-5.1</code> means segment <code>PID</code>, field 5, component 1. It is positional,
      it works whether or not the dictionary recognises anything, and it is what
      <code>get</code>, <code>set</code>, <code>--query</code>, and the
      <code>#[hl7(...)]</code> derive attributes take.
    </dd>
    <dt>A name describes</dt>
    <dd>
      <code>XPN.1</code> means the family-name component of an extended person name. It comes from
      the dictionary, and it is what the tree, the XML elements, and the JSON keys use.
    </dd>
  </dl>
  <p>
    Reach for a path when you know where something is. Reach for a name when you know what
    something is and want the parser to find it.
  </p>

  <h2 id="path-grammar">The path grammar</h2>
  <CodeSample language="text" code={grammar} label="HL7 path syntax" />
  <p>
    Indices are one-based, matching HL7's own numbering. An index omitted means the first
    occurrence, which is why <code>PID-5.1</code> and <code>PID[1]-5[1].1</code> read the same
    value.
  </p>

  <h2 id="get">Reading a value: get</h2>
  <CodeSample language="rust" code={get} />
  <p>
    Note the three-way distinction: <code>Some(value)</code> when there is something there,
    <code>None</code> when the path is valid but names nothing, and <code>Err</code> when the path
    is not a path. A path that names nothing is not an error — messages are routinely missing
    optional fields, and treating that as a failure would make every optional read a special case.
  </p>

  <h2 id="tree">Walking the tree</h2>
  <CodeSample language="rust" code={tree} />
  <p>
    <code>find</code> returns the first descendant with that name; <code>find_all</code> iterates
    all of them. The important method is <code>path()</code>: every node knows the path that reads
    it back, which is how you get from “I found this by walking” to “here is the constant I will
    put in my code”.
  </p>

  <h2 id="repetition">Repetition, and the [n] you need</h2>
  <p>
    An HL7 field may repeat, separated by <code>~</code>. Repetition is the most common source of
    quietly wrong integrations, because an unindexed path silently reads the first one.
  </p>
  <CodeSample language="rust" code={repetition} />
  <Callout type="warning" heading="Do not assume one identifier">
    <p>
      <code>PID-3</code> repeats in practice, all the time — a medical record number, a national
      identifier, a visit number, all in one field. Code that reads <code>PID-3.1</code> and calls
      it “the patient id” will pick whichever the sender happened to put first, and that order is
      not guaranteed to be stable. Read all repetitions and select by assigning authority.
    </p>
  </Callout>

  <h2 id="groups">Groups, and reading through them</h2>
  <p>
    When a message's segments fit its declared structure, the tree nests them into the structure's
    groups. Paths are unaffected: a path names segments and fields, never groups, so the same path
    works whether the message grouped or fell back to flat.
  </p>
  <CodeSample language="rust" code={groups} />
  <p>
    That is a deliberate property. A message that contains a Z-segment renders flat rather than
    grouped — and a path-based reader does not notice. See
    <a href="/docs/concepts/#z">degrading rather than rejecting</a>.
  </p>

  <h2 id="discovering">Discovering paths you do not know</h2>
  <p>
    The fastest route from an unfamiliar message to working code is the command-line tool's
    <code>--paths</code> flag. The bracketed paths beside each node are exactly what reads that
    value back.
  </p>
  <CodeSample language="sh" code={discover} />
  <p>
    Everything standard is already named — <code>PID.5</code> broke into <code>XPN.1</code> and
    <code>XPN.2</code> — and the vendor's own <code>ZAC</code> is there positionally, nothing lost.
    That is the starting point for <a href="/tutorials/vendor-dialect/">taming a vendor dialect</a>.
  </p>

  <h2 id="null-trap">The one thing get cannot tell you</h2>
  <CodeSample language="rust" code={nullTrap} />
  <p>
    HL7 distinguishes the explicit null <code>""</code> (“delete the value you hold”) from an empty
    field (“I am telling you nothing about this”). <code>get</code> returns the text at a path, and
    both come back as an empty string. In an update message that difference is the difference
    between leaving an address alone and erasing it, so when it matters, read the node rather than
    the value. See <a href="/docs/concepts/#null">the HL7 null</a>.
  </p>
</DocPage>
