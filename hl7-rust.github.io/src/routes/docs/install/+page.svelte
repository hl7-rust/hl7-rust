<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { CRATES } from '$lib/data/crates';

  const contents = [
    { id: 'requirements', label: 'Requirements' },
    { id: 'library', label: 'As a library' },
    { id: 'features', label: 'Cargo features' },
    { id: 'cli', label: 'As a command-line tool' },
    { id: 'every-crate', label: 'Every crate, and its install line' },
    { id: 'source', label: 'Building from source' },
    { id: 'verify', label: 'Verifying the install' }
  ];

  const umbrella = `# The umbrella crate: hl7::v2 and hl7::v3
cargo add hl7

# With the v2 derive macros
cargo add hl7 --features derive`;

  const direct = `# HL7 v2 only, no umbrella indirection
cargo add hl7-2
cargo add hl7-2 --features derive

# HL7 v3 only
cargo add hl7-3
cargo add hl7-3 --features derive`;

  const manifest = `[dependencies]
hl7 = { version = "0.2", features = ["derive"] }

# Or, taking v2 directly:
hl7-2 = { version = "0.3", features = ["derive"] }`;

  const mllp = `# Everything: framing, streaming, transport, acknowledgement
cargo add hl7-2-mllp

# Framing, streaming, and transport with no dependencies at all
cargo add hl7-2-mllp --no-default-features

# Add acknowledge_now, which needs a clock
cargo add hl7-2-mllp --features clock`;

  const cli = `# The HL7 v2 tool. The binary is named hl7-v2.
cargo install hl7-2

# The four converters, each its own binary
cargo install hl7-2-from-er7-into-xml
cargo install hl7-2-from-xml-into-er7
cargo install hl7-2-from-er7-into-json
cargo install hl7-2-from-json-into-er7

# The dictionary builder
cargo install hl7-2-from-xsd-into-json-dictionary`;

  const source = `git clone https://github.com/hl7-rust/hl7-rust
cd hl7-rust

cargo build            # every workspace member
cargo test             # every workspace member's tests
cargo build -p hl7-2   # just one

# Check against the minimum supported Rust version
cargo +1.96 check --workspace --all-targets`;

  const verify = `use hl7::v2;

fn main() -> Result<(), v2::Error> {
    let text = "MSH|^~\\\\&|LAB||EPIC||20260814080000||ORU^R01|MSG00042|P|2.5\\r\\
                PID|1||241900||EVERYWOMAN^EVE";

    let message = v2::parse(text)?;
    assert_eq!(message.structure_id(), "ORU_R01");
    assert_eq!(message.get("PID-5.1")?.as_deref(), Some("EVERYWOMAN"));
    println!("ok");
    Ok(())
}`;

  const verifyCli = `$ printf 'MSH|^~\\\\&|LAB||EPIC||20260814080000||ORU^R01|1|P|2.5\\rPID|1||241900\\r' \\
    | hl7-v2 --query PID-3
241900`;
</script>

<DocPage
  lede="Every crate, every feature flag, and the command-line tools. Most people need exactly one line of this page."
  {contents}
>
  <h2 id="requirements">Requirements</h2>
  <p>
    A Rust toolchain, and nothing else. There is no C dependency, no build script that shells out,
    and no vendored binary anywhere in the workspace.
  </p>
  <p>
    The minimum supported Rust version is <strong>current stable minus two releases</strong>,
    pinned once in the workspace root and inherited by every member. At the time of
    writing that is <code>1.96</code>. Raising the floor is routine and expected, not a breaking
    change to be avoided. See <a href="/docs/versions/">Versions and compatibility</a> for the
    policy in full.
  </p>

  <h2 id="library">As a library</h2>
  <p>
    Most users want the umbrella crate. It is a thin re-export — <code>hl7::v2</code> is the
    <a href="/crates/hl7-2/"><code>hl7-2</code></a> crate and <code>hl7::v3</code> is
    <a href="/crates/hl7-3/"><code>hl7-3</code></a> — with room left for
    <code>hl7::fhir</code>. Nothing lives at its root, because a “message”, a “segment”, and a
    “code” mean different things in each standard.
  </p>
  <CodeSample language="sh" code={umbrella} />
  <p>Take a standard directly if you specifically want it without the indirection:</p>
  <CodeSample language="sh" code={direct} />
  <p>Or write the manifest yourself:</p>
  <CodeSample language="toml" caption="Cargo.toml" code={manifest} />

  <h2 id="features">Cargo features</h2>
  <p>
    Nothing in this workspace is on by default that costs you a dependency, with one exception —
    <code>hl7-2-mllp</code>'s <code>ack</code> feature, which is on because an MLLP receiver that
    cannot acknowledge is not much use.
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Crate</th><th>Feature</th><th>Default</th><th>Effect</th></tr>
      </thead>
      <tbody>
        {#each CRATES.filter((crate) => crate.features?.length) as crate (crate.name)}
          {#each crate.features ?? [] as feature (feature.name)}
            <tr>
              <td><a href={`/crates/${crate.slug}/`}><code>{crate.name}</code></a></td>
              <td><code>{feature.name}</code></td>
              <td>{feature.default ? 'on' : 'off'}</td>
              <td>{feature.effect}</td>
            </tr>
          {/each}
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    The derive macros live in their own crates precisely so that the default build of
    <code>hl7-2</code> keeps exactly one dependency: <code>syn</code> and <code>quote</code> are
    compiled only for callers who ask for the macros.
  </p>
  <CodeSample language="sh" caption="MLLP, whose feature set is the most interesting" code={mllp} />
  <Callout type="warning" heading="noncompliance is off for a reason">
    <p>
      <code>hl7-2-mllp</code>'s <code>noncompliance</code> feature forgives two common framing sins
      — a missing <code>&lt;CR&gt;</code> after <code>&lt;FS&gt;</code>, and stray bytes between
      frames. It is off by default because a receiver that quietly accepts malformed framing is how
      a truncated message becomes a clinical record. Prefer per-connection
      <code>Tolerance::Lenient</code> for the one sender that needs it, rather than a crate-wide
      feature flag that loosens every connection at once.
    </p>
  </Callout>

  <h2 id="cli">As a command-line tool</h2>
  <p>
    Six crates ship a binary. They are useful on their own — the first thing to do with a message
    from a vendor you have never seen is look at it, and that does not need a Rust project.
  </p>
  <CodeSample language="sh" code={cli} />
  <p>
    Note the name: <code>cargo install hl7-2</code> installs a binary called
    <code>hl7-v2</code>. The crate is published as <code>hl7-2</code> because
    <code>hl7-v2</code> on crates.io is an unrelated package; the binary kept the readable name.
    Full flag reference: <a href="/docs/cli/">Command line</a>.
  </p>

  <h2 id="every-crate">Every crate, and its install line</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Crate</th><th>Latest</th><th>Install</th><th>What for</th></tr>
      </thead>
      <tbody>
        {#each CRATES as crate (crate.name)}
          <tr>
            <td><a href={`/crates/${crate.slug}/`}><code>{crate.name}</code></a></td>
            <td>{crate.version}</td>
            <td><code>cargo add {crate.name}</code></td>
            <td>{crate.tagline}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    Versions are the latest published at the time of writing, shown so you can tell roughly how
    settled a crate is — not as a constraint to copy. Let <code>cargo add</code> pick.
  </p>
  <p>
    One crate in the dependency map is not in this workspace:
    <a href="https://crates.io/crates/er7"><code>er7</code></a>, the ER7 encoding layer, which is
    its own repository and its own crate. You rarely add it directly — <code>hl7-2</code> and the
    conversion crates re-export what you need.
  </p>

  <h2 id="source">Building from source</h2>
  <CodeSample language="sh" code={source} />
  <p>
    One <code>Cargo.lock</code> at the workspace root covers every member; a crate does not carry
    its own. Member crates depend on each other by relative path, exactly as they did when they
    were sibling repositories — the flat one-directory-per-crate layout was kept specifically so
    those paths did not have to change.
  </p>

  <h2 id="verify">Verifying the install</h2>
  <p>The smallest thing that proves the library is wired up:</p>
  <CodeSample language="rust" code={verify} />
  <p>And the same for the command-line tool:</p>
  <CodeSample language="sh" code={verifyCli} />
  <p>
    If either works, go to the <a href="/docs/quickstart/">quick start</a>. If neither does, the
    <a href="/help/troubleshooting/">troubleshooting page</a> covers the failures people actually
    hit.
  </p>
</DocPage>
