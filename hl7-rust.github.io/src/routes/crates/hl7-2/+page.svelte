<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'features', label: 'Cargo features' },
    { id: 'three-modes', label: 'Three modes' },
    { id: 'walkthrough', label: 'The walkthrough the modes were designed for' },
    { id: 'modify', label: 'Modify and build' },
    { id: 'validate', label: 'Validate' },
    { id: 'versions', label: 'Versions' },
    { id: 'cli', label: 'Command line' },
    { id: 'dependencies', label: 'Dependencies' },
    { id: 'name', label: 'A note on the name' },
    { id: 'related', label: 'Related crates' }
  ];

  const generic = `let message = hl7_2::parse(text)?;
let tree = message.tree();

assert_eq!(tree.name(), "ORU_R01");
assert_eq!(tree.find("XPN.1").unwrap().text(), "EVERYWOMAN");

// Every node knows the path that reads it back.
let second = tree.find_all("OBX").nth(1).unwrap();
assert_eq!(second.path(), "OBX[2]");
assert_eq!(message.get("OBX[2]-5.2")?.as_deref(), Some("Rh positive"));`;

  const schema = `let dictionary = hl7_2::Dictionary::from_json(r#"{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}"#, "acme")?;

let options = hl7_2::Options::new().with_dictionary(std::sync::Arc::new(dictionary));
let message = hl7_2::parse_with_options(text, &options)?;

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

  const stage1 = `$ hl7-v2 --paths samples/vendor.hl7
ADT_A01
  ZAC  [ZAC[1]]
    ZAC.1 = 7           [ZAC[1]-1[1]]
    ZAC.2               [ZAC[1]-2[1]]
      ZAC.2.1 = SMITH   [ZAC[1]-2[1].1]
      ZAC.2.2 = JOHN    [ZAC[1]-2[1].2]
    ZAC.3 = 20260814    [ZAC[1]-3[1]]`;

  const modify = `let mut message = hl7_2::parse(text)?;
message.set("PID-5.2", "EVELYN")?;      // escapes delimiters in the value
message.append_segment("NTE");
message.set("NTE[2]-3", "Amended.")?;
let er7 = message.to_er7();             // valid ER7, ready to send`;

  const ack = `let ack = hl7_2::builder::acknowledge(&message, "AA", "ACK00001", "20260814080100")
    .build_valid()?;
assert_eq!(ack.get("MSA-2")?.as_deref(), Some("MSG00042"));`;

  const validate = `for diagnostic in message.validate() {
    println!("{diagnostic}");
    // error: MSA[1]-4[1]: "many" is not a valid NM value
}`;

  const strict = `let options = hl7_2::Options::new().strict();
match hl7_2::parse_with_options(text, &options) {
    Err(hl7_2::Error::Invalid(diagnostics)) => { /* every error-level finding */ }
    Ok(message) => { /* conformant */ }
    Err(other) => { /* not a message at all */ }
}`;

  const cli = `hl7-v2 samples/oru_r01.hl7                       # look at it
hl7-v2 --query OBX-5 samples/oru_r01.hl7         # pull out every result
hl7-v2 --check samples/adt_a01.hl7               # check it
hl7-v2 --dictionary samples/acme.json vendor.hl7 # read a dialect
hl7-v2 --set 'PID-8=F' --er7 samples/orm_o01.hl7 # edit and re-emit`;

  const install = `cargo add hl7-2                     # library
cargo add hl7-2 --features derive   # library with the derive macros
cargo install hl7-2                 # command-line tool, named hl7-v2`;
</script>

<DocPage title={crate.name} lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>{crate.summary}</p>
  <p>
    HL7 v2 is the format most healthcare data still moves in, and the hard part is not the syntax.
    That is pipes and carets, and the <a href="https://crates.io/crates/er7"><code>er7</code></a>
    crate this one is built on already handles it. The hard part is knowing what the pipes and
    carets <em>mean</em> in the release the sender speaks. This crate owns that: the per-release
    data-type tables, the message structures, and the three ways to apply them.
  </p>

  <h2 id="features">Cargo features</h2>
  <CodeSample language="sh" code={install} />

  <h2 id="three-modes">Three modes</h2>
  <h3>Generic — for the vendor you have never seen</h3>
  <p>
    Parse anything into a navigable tree. Nothing is rejected, nothing is dropped, and what the
    dictionary recognises gets a name from HL7's own vocabulary.
  </p>
  <CodeSample language="rust" code={generic} />
  <p>
    Segments are grouped into the message structure when they fit it —
    <code>ORU_R01.PATIENT_RESULT.ORDER_OBSERVATION.OBSERVATION</code> — and read flat when they do
    not.
  </p>

  <h3>Schema-based — for the vendor whose format is not frozen</h3>
  <p>
    Write the shape as JSON, load it at runtime, and adding a field is a configuration change rather
    than a release.
  </p>
  <CodeSample language="rust" code={schema} />
  <p>
    The same format describes the bundled releases, so a schema can inherit one and state only its
    dialect. See <a href="/guides/dictionaries/">Vendor dictionaries</a>.
  </p>

  <h3>Struct-based — for the feed that does not change</h3>
  <CodeSample language="rust" code={structMode} />
  <p>
    That last field is the point. Real feeds are stable until they are not, and the usual choice at
    that moment is to re-parse the raw message or rewrite the library. A <code>Raw</code> field keeps
    the whole parsed message beside the typed data, so the fallback is a method call.
  </p>

  <h2 id="walkthrough">The walkthrough the modes were designed for</h2>
  <p>
    The three modes are not three libraries to choose between. They are three stages of the same
    job, and a real integration walks through them in order.
  </p>
  <p>
    <strong>Stage 1 — look at it.</strong> A vendor sends a message and nobody knows what is in it.
    Start with the tool, not with code:
  </p>
  <CodeSample language="sh" code={stage1} />
  <p>
    Everything standard is already named, and the vendor's own <code>ZAC</code> is there
    positionally, nothing lost. The bracketed paths are not decoration: each one is what reads that
    value back.
  </p>
  <p>
    <strong>Stage 2 — write down what you learned.</strong> <code>ZAC.2</code> is clearly a name. Say
    so, in JSON, without touching the code.
  </p>
  <p>
    <strong>Stage 3 — freeze what is stable.</strong> Once the interface has held still long enough
    to trust, move it into the type system — and keep the <code>raw</code> field, because stage 3 is
    never final.
  </p>
  <p>The full walkthrough is <a href="/tutorials/vendor-dialect/">Taming a vendor dialect</a>.</p>

  <h2 id="modify">Modify and build</h2>
  <CodeSample language="rust" code={modify} />
  <CodeSample language="rust" code={ack} />
  <p>
    An unmodified message writes back byte for byte — that guarantee is <code>er7</code>'s, and this
    crate does not weaken it.
  </p>

  <h2 id="validate">Validate</h2>
  <p>
    Parsing stays lenient: unknown segments, unknown types, and structure mismatches are never
    errors. Checking is a separate question with a separate answer.
  </p>
  <CodeSample language="rust" code={validate} />
  <p>
    Diagnostics split by whose problem it is. <strong>Errors</strong> are the message contradicting
    the dictionary it claims. <strong>Warnings</strong> are the dictionary not covering the message.
    Strict mode rejects the first and allows the second:
  </p>
  <CodeSample language="rust" code={strict} />
  <Callout heading="A Z-segment does not make a conformant message fail">
    <p>
      Most real interfaces carry one, and a strict mode that rejected them all would be turned off.
      See <a href="/guides/validating/#z-segments">Validating</a>.
    </p>
  </Callout>

  <h2 id="versions">Versions</h2>
  <p>
    The fourteen published releases from 2.1 to 2.9, chosen from <code>MSH-12</code> or forced with
    <code>Options::with_version</code>. A release string this crate has no dictionary for resolves to
    the nearest older one (<code>2.5.2</code> reads as <code>2.5.1</code>) rather than failing.
  </p>
  <p>
    v2.5 is the complete base dictionary; every other release is a delta of it covering the
    differences this crate models today, and inherits the rest. That incompleteness is bounded by
    design: an unmodelled difference costs a typed name, never a rejected message or a lost value.
    See <a href="/docs/versions/">Versions and compatibility</a>.
  </p>

  <h2 id="cli">Command line</h2>
  <CodeSample language="sh" code={cli} />
  <p>
    <code>hl7-v2 --help</code> lists everything. Exit status is 0 on success, 1 on a usage or parse
    error, and 2 when <code>--check</code> or <code>--strict</code> found something wrong. Full
    reference: <a href="/docs/cli/#hl7-v2">Command line</a>.
  </p>

  <h2 id="dependencies">Dependencies</h2>
  <p>
    One: <a href="https://crates.io/crates/er7"><code>er7</code></a>, which has none of its own. The
    JSON reader that loads dictionaries is hand-written here for the same reason the sibling crates
    hand-write their writers — in a domain where dependency trees get audited, a two-crate tree is
    worth a few hundred lines. Enabling the <code>derive</code> feature adds
    <a href="/crates/hl7-2-derive/"><code>hl7-2-derive</code></a>, and with it
    <code>syn</code> and <code>quote</code>.
  </p>

  <h2 id="name">A note on the name</h2>
  <p>
    The crate is published as <code>hl7-2</code>; the binary it installs is called
    <code>hl7-v2</code>. The package could not take the readable name because
    <code>hl7-v2</code> on crates.io is an unrelated crate, and the rest of the workspace was renamed
    from <code>hl7-v2*</code> to <code>hl7-2*</code> for consistency with it. The archived
    repositories those directories came from still carry the original names.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
