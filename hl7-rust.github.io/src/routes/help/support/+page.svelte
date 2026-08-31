<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { ORG, REPO } from '$lib/data/crates';

  const contents = [
    { id: 'where', label: 'Where to file' },
    { id: 'report', label: 'What to include in a report' },
    { id: 'redaction', label: 'Do not paste patient data' },
    { id: 'layout', label: 'How the repository is laid out' },
    { id: 'conventions', label: 'Conventions a change has to meet' },
    { id: 'coverage', label: 'Adding dictionary coverage' },
    { id: 'site', label: 'Fixing this website' },
    { id: 'license', label: 'Licensing your contribution' }
  ];

  const mirrors = [
    { label: 'GitHub', href: 'https://github.com/hl7-rust', note: 'The primary home. Issues live here.' },
    { label: 'GitLab', href: 'https://gitlab.com/hl7-rust', note: 'Mirror.' },
    { label: 'Codeberg', href: 'https://codeberg.org/hl7-rust', note: 'Mirror.' }
  ];

  const redacted = `MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5
PID|1||REDACTED^^^ACME&1.2.3.4&ISO^MR||REDACTED^REDACTED||REDACTED|F
OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F`;

  const checks = `cargo test                                # unit and integration tests
cargo clippy --all-targets -- -D warnings # lint-clean
cargo fmt --check                         # formatting
cargo rustdoc --lib -- -W missing-docs    # every public item documented

# And against the minimum supported Rust version
cargo +1.96 check --workspace --all-targets`;

  const workspace = `Cargo.toml           the workspace: [workspace] members, nothing else
Cargo.lock           one, at the root — never one inside a member
spec/                workspace-wide specs (the MSRV policy)
<crate>/
  Cargo.toml         the member's own manifest
  README.md          a tour
  spec/index.md      the normative specification, where behavior is normative
  AGENTS.md          conventions and required checks for changing this crate
  CLAUDE.md          points at AGENTS.md
  LICENSE.md         the same five-way boilerplate, byte for byte
  samples/           example inputs, where the crate has them`;
</script>

<DocPage
  lede="Where to file, what to include, and what a change has to satisfy before it can land. Reports that quote a spec section number get fixed fastest."
  {contents}
>
  <Callout heading="The short version is in the repository">
    <p>
      <a href={`${REPO}/blob/main/CONTRIBUTING.md`}><code>CONTRIBUTING.md</code></a> is the condensed
      form of this page — enough to file a good report or land a small change without reading
      anything else. This page is the long version.
    </p>
  </Callout>

  <h2 id="where">Where to file</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Host</th><th>Use it for</th></tr>
      </thead>
      <tbody>
        {#each mirrors as mirror (mirror.href)}
          <tr>
            <td><a href={mirror.href}>{mirror.label}</a></td>
            <td>{mirror.note}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    The workspace is <a href={REPO}>github.com/hl7-rust/hl7-rust</a>, and the organization
    around it — including the archived predecessor repositories — is at
    <a href={ORG}>github.com/hl7-rust</a>. The ER7 encoding layer is elsewhere again:
    <a href="https://github.com/er7-rust/er7-rust"><code>er7-rust/er7-rust</code></a>, its own organization, so file
    encoding-level bugs there rather than here.
  </p>
  <p>
    File against the workspace, not against a crate's archived former repository — those still carry
    the old <code>hl7-v2*</code> names and are kept only so their history stays reachable.
  </p>

  <h2 id="report">What to include in a report</h2>
  <ol>
    <li>
      <strong>The crate and its version.</strong> <code>cargo tree -p hl7-2</code> if you are not
      sure what resolved.
    </li>
    <li>
      <strong>A minimal message that reproduces it</strong>, redacted — see below. One segment is
      often enough.
    </li>
    <li>
      <strong>What you expected and what you got.</strong> The output of
      <code>hl7-v2 --tree --paths</code> on the message is usually the clearest way to show both.
    </li>
    <li>
      <strong>The spec section, if you can find it.</strong> Each crate's
      <code>spec/index.md</code> is numbered specifically so it can be cited — “§4.2 says positional
      names are used here, but the output has …” turns a discussion into a fix.
    </li>
    <li>
      <strong>Your Rust version</strong>, if it is a build failure.
    </li>
  </ol>
  <Callout heading="The spec is the referee">
    <p>
      If a crate's README, its rustdoc, or this website disagrees with its
      <code>spec/index.md</code>, the spec is right and the other three are the bug — that ordering is
      stated in the crates' own READMEs. A report that says “the spec says X, the code does Y” is
      unambiguous. A report that says “the README says X” may be a documentation fix instead, which
      is also worth filing.
    </p>
  </Callout>

  <h2 id="redaction">Do not paste patient data</h2>
  <p>
    HL7 messages are clinical records. An issue tracker is public and permanent, and a message pasted
    into one cannot be unpasted.
  </p>
  <CodeSample language="er7" caption="Redact identifying values; keep the structure" code={redacted} />
  <p>
    Structure is what reproduces a parsing bug — the delimiters, the field positions, the repetition
    separators, the component depth. Names, identifiers, dates of birth, and addresses are not.
    Replace them and the report still works.
  </p>
  <p>
    If a bug genuinely depends on a specific byte sequence in a value — an unusual escape, a
    non-ASCII character set — describe the byte sequence rather than the record it appeared in.
  </p>
  <p>
    <a href="/docs/patient-data/">Patient data</a> states the project's full position: what these
    crates do with what you hand them, what they never do, and the one place — error and diagnostic
    strings — where a value can escape into a log.
  </p>

  <h2 id="layout">How the repository is laid out</h2>
  <CodeSample language="text" code={workspace} />
  <p>
    Most crate directories are former standalone repositories, merged in with
    <code>git subtree</code> so their commit history survived the move — which means
    <code>git log &lt;crate&gt;/</code> reaches back before the workspace existed.
  </p>
  <p>
    Member crates depend on each other by relative path, and the flat one-directory-per-crate layout
    was kept specifically so those paths did not have to change.
  </p>

  <h2 id="conventions">Conventions a change has to meet</h2>
  <p>
    Read the crate's own <code>AGENTS.md</code> before working in it — the workspace-level
    <code>AGENTS.md</code> covers only workspace-wide concerns. The checks every crate expects:
  </p>
  <CodeSample language="sh" code={checks} />
  <ul>
    <li>
      <strong>One <code>Cargo.lock</code>, at the root.</strong> Do not add one inside a member
      directory.
    </li>
    <li>
      <strong>No shared <code>[workspace.dependencies]</code></strong> without discussion — it would
      touch every member's manifest at once. The one deliberate exception is
      <code>rust-version</code>, below, inherited via <code>[workspace.package]</code> because all
      fourteen members must always agree on that one value.
    </li>
    <li>
      <strong>MSRV is current stable minus two.</strong> Raising the floor is routine and expected,
      not a breaking change to be avoided.
    </li>
    <li>
      <strong>Licensing boilerplate is byte-for-byte identical</strong> in every crate. Do not invent
      different text for a new one.
    </li>
    <li>
      <strong>A change that spans crates updates every affected crate's
        <code>AGENTS.md</code> and <code>spec/index.md</code></strong> in the same change.
    </li>
    <li>
      <strong>Behavior changes go in the spec first.</strong> The spec is the source of truth, so a
      code change that contradicts it is either a bug fix or an unstated spec change.
    </li>
  </ul>
  <Callout type="warning" heading="The forward and reverse conversion crates are coupled">
    <p>
      A change to a forward crate's naming rules silently breaks its reverse crate's assumptions.
      Run the <a href="/tutorials/round-trip/">round trip</a> after touching either — it is the
      assertion that the two halves still agree.
    </p>
  </Callout>

  <p>
    A change argued on performance carries a before-and-after from the benchmarks in the same crate,
    produced the way <a href="/docs/benchmarks/">Benchmarks</a> describes. Correctness outranks speed
    here: a faster parser that loses a value, or that stops round-tripping byte for byte, is not
    faster.
  </p>

  <h2 id="coverage">Adding dictionary coverage</h2>
  <p>
    Dictionary gaps are the most useful contribution, and the cheapest: filling one means editing one
    JSON file under <code>hl7-2/schemas/</code> and adding a test.
  </p>
  <p>
    Add coverage when a real message motivates it, rather than speculatively — the crate's own spec
    says so, and a table filled in from the standard without a message behind it is a table nobody
    can check.
  </p>
  <p>
    Before filing, consider whether it is a gap at all. An unmodelled difference between releases
    shows up as a positional name rather than a typed one, or as a missing warning — never as a
    rejected message or a lost value. That is honest incompleteness, and
    <a href="/docs/conformance/">Conformance</a> states exactly what is and is not covered.
  </p>

  <h2 id="site">Fixing this website</h2>
  <p>
    This site lives in the <code>hl7-rust.github.io</code> directory of the same workspace, built
    with SvelteKit's static adapter and the
    <a href="https://github.com/LilyDesignSystem">Lily Design System</a> headless Svelte components.
    It deploys to GitHub Pages on every push to <code>main</code>.
  </p>
  <p>
    Nothing here is normative — the site summarises the crates' READMEs and specs and links back to
    them. So a correction here is a documentation fix; a correction to the underlying behavior belongs
    against the crate.
  </p>

  <h2 id="license">Licensing your contribution</h2>
  <p>
    Everything in the workspace is offered under MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or
    GPL-3.0-only, at the user's option. A contribution is offered on the same terms, so that the
    choice stays available to everyone downstream.
  </p>
  <p>Contact: <a href="mailto:joel@joelparkerhenderson.com">joel@joelparkerhenderson.com</a>.</p>
</DocPage>
