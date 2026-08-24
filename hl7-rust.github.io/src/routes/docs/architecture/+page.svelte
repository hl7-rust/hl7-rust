<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { CATEGORIES, CRATES, cratesIn, type CrateCategory } from '$lib/data/crates';

  const contents = [
    { id: 'map', label: 'The map' },
    { id: 'layers', label: 'One crate per layer' },
    { id: 'standards', label: 'One module per standard' },
    { id: 'dependencies', label: 'Why the dependency counts are so small' },
    { id: 'forward-reverse', label: 'Forward and reverse are not symmetric' },
    { id: 'shared', label: 'What is shared, and what is deliberately not' },
    { id: 'workspace', label: 'The workspace, and the history behind it' },
    { id: 'table', label: 'Every crate, by layer' }
  ];

  const map = `er7                                    the ER7 encoding: delimiters,
                                       escapes, paths, byte-for-byte
                                       rendering, batch splitting
  |
hl7-2                                  the HL7 v2 dictionary: releases
  |                                    2.1-2.9, data types, message
  |                                    structures; three parsing modes;
  |                                    mutation; validation
  |
  +-- hl7-2-mllp                       transport: HL7 v2 over TCP (MLLP)
  +-- hl7-2-soap                       transport: HL7 v2 over HTTP (SOAP)
  +-- hl7-2-from-er7-into-json         format conversions
  +-- hl7-2-from-er7-into-xml
  +-- hl7-2-from-json-into-er7
  +-- hl7-2-from-xml-into-er7
  +-- hl7-2-from-xsd-into-json-dictionary   writes the dictionaries
  |                                          hl7-2 reads, from HL7
  |                                          v2.xml XSDs
  +-- hl7-2-xml-lite-helper            shared minimal XML reader, also
        |                              used directly by:
        +-- hl7-3                      HL7 v3: RIM backbone classes,
              |                        coded values, the three-level
              |                        message envelope — a foundation,
              |                        not a full implementation
              +-- hl7-3-derive         #[derive(FromElement)] for hl7-3's
              |                        struct mode
              +-- hl7-3-soap           transport: HL7 v3 over HTTP (SOAP)

hl7                                    the umbrella crate — hl7::v2 and
                                       hl7::v3 today, room for hl7::fhir`;

  const umbrella = `// hl7/src/lib.rs, in full, minus the documentation
pub use hl7_2 as v2;
pub use hl7_3 as v3;`;

  const order: CrateCategory[] = ['core', 'transport', 'conversion', 'tooling'];
</script>

<DocPage
  title="Architecture"
  lede="Fourteen crates in one Cargo workspace, plus er7 outside it. This page is the reasoning: what each seam is for, and why you can stop at any of them."
  {contents}
>
  <h2 id="map">The map</h2>
  <CodeSample language="text" code={map} label="The workspace as a dependency map" />

  <h2 id="layers">One crate per layer</h2>
  <p>
    Read the map top to bottom and each line is a layer that can be taken on its own. That is the
    organising principle, and it is worth spelling out because it is what makes fourteen crates
    less alarming than it sounds.
  </p>
  <dl>
    <dt><a href="https://crates.io/crates/er7"><code>er7</code></a> — syntax</dt>
    <dd>
      Delimiters, escapes, paths, byte-for-byte rendering, batch splitting. No knowledge of what
      any field means. It lives in its own repository, outside this workspace, because plenty of
      people want the encoding and not the dictionary.
    </dd>
    <dt><a href="/crates/hl7-2/"><code>hl7-2</code></a> — meaning</dt>
    <dd>
      The per-release data-type tables, the message structures, and the three ways to apply them.
      This is where nearly all the v2 capability lives, and it is the only crate most callers need.
    </dd>
    <dt>The transports — delivery</dt>
    <dd>
      <a href="/crates/hl7-2-mllp/"><code>hl7-2-mllp</code></a> and
      <a href="/crates/hl7-2-soap/"><code>hl7-2-soap</code></a> know how to get a message across a
      network and how to answer it. Neither has an opinion about HL7 semantics, and neither owns a
      socket: <code>IoTransport</code> takes any reader/writer, and the SOAP crate leaves HTTP to
      whatever stack you already use.
    </dd>
    <dt>The conversions — representation</dt>
    <dd>
      Four crates covering ER7's two directions against both target formats. They are the layer
      you use when what is downstream is not Rust.
    </dd>
    <dt>The tooling — everything else</dt>
    <dd>
      The derive macros, the dictionary builder, and the shared XML reader. Each is in a crate of
      its own for a specific reason, given below.
    </dd>
  </dl>

  <h2 id="standards">One module per standard</h2>
  <p>The umbrella crate is, in its entirety:</p>
  <CodeSample language="rust" code={umbrella} />
  <p>
    Nothing lives at its root, and that is a decision rather than an omission. A “message”, a
    “segment”, and a “code” mean different things in v2, v3, and FHIR, and flattening them into one
    namespace would invite mixing them up — in a domain where mixing them up means a clinical
    record. Depend on <code>hl7-2</code> or <code>hl7-3</code> directly if you want one standard
    without the indirection; the umbrella exists for callers who want the room left for
    <code>hl7::fhir</code>.
  </p>
  <Callout heading="One asymmetry to know about">
    <p>
      <code>hl7</code>'s <code>derive</code> feature forwards to <code>hl7-2</code>'s only. If you
      want <code>hl7-3</code>'s <code>#[derive(FromElement)]</code>, enable it on
      <code>hl7-3</code> — or depend on that crate directly.
    </p>
  </Callout>

  <h2 id="dependencies">Why the dependency counts are so small</h2>
  <p>
    <code>hl7-2</code> has one dependency. <code>hl7-3</code> has one.
    <code>hl7-2-soap</code> has one. <code>hl7-2-xml-lite-helper</code> has none, and intends to
    keep it that way. <code>hl7-2-mllp</code> with <code>--no-default-features</code> has none.
  </p>
  <p>
    That is not minimalism for its own sake. Healthcare software gets audited, and a dependency
    tree is part of what gets audited. The JSON reader that loads dictionaries in
    <code>hl7-2</code> is hand-written for exactly this reason — in a domain where trees get read
    line by line, a two-crate tree is worth a few hundred lines of code.
  </p>
  <p>
    The same logic is why the derive macros live in separate crates. <code>syn</code> and
    <code>quote</code> are large, and they are compiled only for callers who ask for the macros;
    the default build of <code>hl7-2</code> never sees them.
  </p>

  <h2 id="forward-reverse">Forward and reverse are not symmetric</h2>
  <p>
    The four conversion crates look like two symmetric pairs. They are not, and the asymmetry is
    the most interesting thing about them.
  </p>
  <p>
    A <strong>forward</strong> crate — ER7 into XML or JSON — needs the full HL7 v2.5 dictionary,
    because naming <code>&lt;XPN.1&gt;</code> rather than <code>&lt;PID.5.1&gt;</code> requires
    knowing that <code>PID-5</code> holds an <code>XPN</code>. It also needs the message-structure
    grammars, to decide what nests inside what.
  </p>
  <p>
    A <strong>reverse</strong> crate needs neither. Every element or key name the forward crate
    writes already carries its own position, as the number after the name's last dot — whether the
    name in front of that number is a recognized data type or a positional fallback. So reversal is
    a purely structural rebuild. Each reverse crate's <code>spec/index.md</code> §1.1 says exactly
    this, and it is why those two crates are so much smaller than their counterparts.
  </p>

  <h2 id="shared">What is shared, and what is deliberately not</h2>
  <p>
    <a href="/crates/hl7-2-xml-lite-helper/"><code>hl7-2-xml-lite-helper</code></a> exists because
    three crates in this family had each written their own minimal XML reader, and three copies of a
    parser is three places for a bug. It now serves four:
    <code>hl7-2-soap</code>, <code>hl7-3-soap</code>, <code>hl7-2-from-xml-into-er7</code>, and
    <code>hl7-2-from-xsd-into-json-dictionary</code> — plus <code>hl7-3</code>, which reads v3's
    native XML through it. Its name says it is scoped to this family; it is not offered as a
    general-purpose parser.
  </p>
  <p>
    What is <em>not</em> shared is just as deliberate. The two forward conversion crates keep their
    own renderers, because the target formats differ in ways no shared abstraction would model
    honestly — JSON has real arrays and a real <code>null</code>; XML has repeated sibling elements
    and empty elements. Their specs are kept consistent with each other instead, and each one's §0
    states exactly where they are meant to diverge.
  </p>
  <p>
    <code>hl7-3-soap</code> deliberately does not depend on <code>hl7-3</code>. A SOAP envelope is
    XML, and reading one requires no HL7 knowledge beyond the names of a few elements — so the
    transport stays usable by anyone who wants to route v3 traffic without decoding it.
  </p>

  <h2 id="workspace">The workspace, and the history behind it</h2>
  <p>
    This is one Cargo workspace with one <code>Cargo.lock</code> at its root. Member crates depend
    on each other by relative path, and the flat one-directory-per-crate layout was kept
    specifically so those paths did not have to change — because most of these directories used to
    be standalone repositories, merged in with <code>git subtree</code> so that each crate's commit
    history is still walkable under its own directory.
  </p>
  <p>
    Each crate keeps its own edition, feature set, and dependency list; the workspace does not use
    <code>[workspace.package]</code> inheritance or a shared
    <code>[workspace.dependencies]</code> table. That is a live decision rather than an oversight —
    adding either would touch every member's manifest at once.
  </p>
  <p>
    One historical wrinkle shows up in names. Every <code>hl7-v2*</code> crate was renamed to
    <code>hl7-2*</code> — <code>hl7-v2</code> itself because that name was already taken on
    crates.io by an unrelated crate, and the rest for consistency. The archived repositories those
    directories came from still carry the original names, and the installed CLI binary is still
    called <code>hl7-v2</code>.
  </p>

  <h2 id="table">Every crate, by layer</h2>
  {#each order as category (category)}
    <h3>{CATEGORIES[category].label}</h3>
    <p>{CATEGORIES[category].blurb}</p>
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>Crate</th><th>Depends on</th><th>What it adds</th></tr>
        </thead>
        <tbody>
          {#each cratesIn(category) as crate (crate.name)}
            <tr>
              <td><a href={`/crates/${crate.slug}/`}><code>{crate.name}</code></a></td>
              <td>{crate.dependencies.length === 0 ? 'nothing' : crate.dependencies.join(', ')}</td>
              <td>{crate.tagline}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/each}
  <p>
    {CRATES.length} crates here, plus <code>er7</code> in its own repository. See
    <a href="/crates/">the crate reference</a> for a page on each.
  </p>
</DocPage>
