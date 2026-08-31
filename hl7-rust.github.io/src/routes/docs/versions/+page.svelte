<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { CRATES, REPO } from '$lib/data/crates';

  const contents = [
    { id: 'hl7-releases', label: 'HL7 v2 releases 2.1 through 2.9' },
    { id: 'resolution', label: 'How a release is chosen' },
    { id: 'deltas', label: 'What each release actually claims' },
    { id: 'crate-versions', label: 'Crate versions' },
    { id: 'msrv', label: 'Minimum supported Rust version' },
    { id: 'editions', label: 'Rust edition' },
    { id: 'license', label: 'Licensing' },
    { id: 'stability', label: 'What “0.x” means here' }
  ];

  const resolution = `use hl7_2::Version;

// From MSH-12, which is the default.
let message = hl7_2::parse(text)?;

// Forced, for a sender whose MSH-12 does not match what it actually sends.
let options = hl7_2::Options::new().with_version(Version::V2_3);
let message = hl7_2::parse_with_options(text, &options)?;

// Or resolve a release string the way MSH-12 is resolved: exact if known,
// otherwise the newest release no newer than what was declared.
assert_eq!(Version::nearest("2.5.2"), Some(Version::V2_5_1));`;

  const msrv = `# Check a change against the floor
cargo +1.96 check --workspace --all-targets`;

  const licenseText = `MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only`;

  /** From hl7-2's spec/index.md §3.4 — the differences from v2.5 it models. */
  const deltas = [
    {
      release: '2.1, 2.2, 2.3',
      note: 'MSH-9 has no message-structure component; MSH-12 is a plain ID; ERR is the one-field form; no SFT, no SPM; the pre-2.5 ACK structure.'
    },
    {
      release: '2.3.1',
      note: 'ERR one-field form; no SFT, no SPM; pre-2.5 ACK. MSH-9.3 and the VID composite arrived here, so both are inherited.'
    },
    { release: '2.4', note: 'ERR one-field form; no SFT, no SPM; pre-2.5 ACK.' },
    { release: '2.5', note: 'The complete base dictionary. Everything else is stated relative to it.' },
    { release: '2.5.1, 2.6', note: 'Nothing this crate models changed.' },
    {
      release: '2.7, 2.8, 2.9',
      note: 'TS is withdrawn in favour of the primitive DTM, so a TS-typed field holds a scalar timestamp rather than a value^precision pair.'
    },
    {
      release: '2.7.1, 2.8.1, 2.8.2',
      note: 'No dictionary file of their own; each resolves to its base release.'
    }
  ];
</script>

<DocPage
  lede="Which HL7 releases are supported and what “supported” means for each; which crate versions are current; the Rust version floor; and what the five-way license actually gives you."
  {contents}
>
  <h2 id="hl7-releases">HL7 v2 releases 2.1 through 2.9</h2>
  <p>
    <a href="/crates/hl7-2/"><code>hl7-2</code></a> covers the fourteen published releases from
    2.1 to 2.9 — including the point releases 2.3.1, 2.5.1, 2.7.1, 2.8.1, and 2.8.2. v2.5 is the
    complete base dictionary; every other release is a <em>delta</em> of it, covering the
    differences this crate models today, and inheriting everything else. Three of the fourteen
    (2.7.1, 2.8.1, 2.8.2) have no dictionary file of their own and resolve to their base
    release.
  </p>
  <Callout heading="That incompleteness is bounded by design">
    <p>
      An unmodelled difference between releases costs you a typed <em>name</em> — a field reads with
      a positional generic name instead of its data type. It never costs a rejected message and
      never costs a value. Each release's exact claim is stated in
      <code>hl7-2</code>'s <code>spec/index.md</code> §3.4.
    </p>
  </Callout>

  <h2 id="resolution">How a release is chosen</h2>
  <p>
    By default the release comes from <code>MSH-12</code>, the version id the message declares
    about itself. A release string the crate has no dictionary for resolves to the nearest
    <em>older</em> one — <code>2.5.2</code> reads as <code>2.5.1</code> — rather than failing.
  </p>
  <CodeSample language="rust" code={resolution} />
  <p>
    Forcing a release is not a niche need. Real senders declare <code>2.3</code> and send fields
    that only exist in <code>2.5</code>, or declare <code>2.5</code> and send a 2.3-shaped
    <code>MSH-9</code>. When the declaration and the content disagree, believe the content.
  </p>

  <h2 id="deltas">What each release actually claims</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Release</th><th>Modelled as</th></tr>
      </thead>
      <tbody>
        {#each deltas as row (row.release)}
          <tr><td><code>{row.release}</code></td><td>{row.note}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    Three of those change how a message <em>reads</em>, not just what a field is called, so they
    are the ones worth remembering: <code>MSH-9</code> has no message-structure component before
    v2.3.1, which is why a pre-2.3.1 message's structure has to be resolved from
    <code>MSH-9.1</code> and <code>MSH-9.2</code>; <code>ERR</code> is a single field before v2.5;
    and <code>TS</code> is withdrawn in favour of the primitive <code>DTM</code> from v2.7, so a
    <code>TS</code>-typed field holds a scalar timestamp rather than a
    <code>value^precision</code> pair.
  </p>
  <p>
    HL7 v3 is versioned differently — by interaction, not by release — and
    <a href="/crates/hl7-3/"><code>hl7-3</code></a> is explicit that it implements the shared
    foundation rather than any particular interaction. Read that crate's
    <code>spec/index.md</code> §1 before filing anything as a bug.
  </p>

  <h2 id="crate-versions">Crate versions</h2>
  <p>
    Latest published at the time of writing. Each crate is versioned independently — they are
    released when they change, not on a common cadence.
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Crate</th><th>Version</th><th>Edition</th><th>MSRV</th></tr>
      </thead>
      <tbody>
        {#each CRATES as crate (crate.name)}
          <tr>
            <td><a href={`/crates/${crate.slug}/`}><code>{crate.name}</code></a></td>
            <td>{crate.version}</td>
            <td>2024</td>
            <td>1.96</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <h2 id="msrv">Minimum supported Rust version</h2>
  <p>
    The policy is <strong>current stable minus two releases</strong>, pinned once in the
    workspace's own <code>[workspace.package]</code> and inherited by every member as
    <code>rust-version.workspace = true</code>, stated in
    <a href={`${REPO}/blob/main/spec/rust-msrv-n-minus-2/index.md`}
      ><code>spec/rust-msrv-n-minus-2/index.md</code></a
    >. At the time of writing that is <code>1.96</code>.
  </p>
  <CodeSample language="sh" code={msrv} />
  <p>
    Raising the floor is routine and expected, not a breaking change to be avoided — it happens in
    the same change that observes a new stable release. If a crate here fails to build on a Rust
    version at or above its declared <code>rust-version</code>, that is a bug worth reporting.
  </p>

  <h2 id="editions">Rust edition</h2>
  <p>
    Every member is on edition 2024. Editions are per crate in Cargo, so your own crate's edition
    is unaffected by these — you can depend on an edition-2024 crate from an edition-2018 crate
    without trouble.
  </p>

  <h2 id="license">Licensing</h2>
  <p>Every crate, and the workspace root, carries the same five-way choice:</p>
  <CodeSample language="text" code={licenseText} label="SPDX license expression" />
  <p>
    <code>OR</code> means you pick. Take MIT if you want the simplest permissive terms; Apache-2.0
    if you want its explicit patent grant; BSD-3-Clause if that is what your organization has
    already cleared; or either GPL if you are integrating into a copyleft project that requires
    it. You do not have to comply with all five, and you do not have to tell anyone which you
    chose.
  </p>
  <p>
    The same multi-license boilerplate appears byte for byte in every crate's
    <code>LICENSE.md</code>, matching each <code>Cargo.toml</code>'s <code>license</code> field.
  </p>

  <h2 id="stability">What “0.x” means here</h2>
  <p>
    Most of these crates are pre-1.0, and Cargo treats a <code>0.x</code> minor bump as
    potentially breaking. In practice:
  </p>
  <ul>
    <li>
      <a href="/crates/hl7-2/"><code>hl7-2</code></a> is the most settled — the three modes, the
      path grammar, and the diagnostic split are stable shapes even while the dictionary keeps
      filling in.
    </li>
    <li>
      The conversion crates are governed by their specs rather than by their APIs. Their
      <code>convert</code> signature has been stable for several releases; what changes is coverage,
      and the specs record it.
    </li>
    <li>
      <a href="/crates/hl7-3/"><code>hl7-3</code></a> is explicitly a foundation. Expect it to grow
      rather than to change shape, but read its scope section before depending on a specific
      behavior.
    </li>
  </ul>
  <p>
    Pin what you depend on the way you would pin any pre-1.0 crate — <code>hl7-2 = "0.3"</code> —
    and read the crate's <code>spec/index.md</code> rather than inferring guarantees from the type
    signatures.
  </p>
</DocPage>
