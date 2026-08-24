<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import LinkCards from '$lib/components/LinkCards.svelte';
  import { CATEGORIES, CRATES, cratesIn, type CrateCategory } from '$lib/data/crates';

  const contents = [
    { id: 'core', label: 'Core' },
    { id: 'transport', label: 'Transports' },
    { id: 'conversion', label: 'Format conversions' },
    { id: 'tooling', label: 'Tooling and helpers' },
    { id: 'er7', label: 'And one crate outside the workspace' },
    { id: 'all', label: 'All of them, at a glance' }
  ];

  const order: CrateCategory[] = ['core', 'transport', 'conversion', 'tooling'];

  const er7 = `use er7::Message;

// Pipes and carets, with no opinion about what any field means.
let message = er7::parse(text)?;
assert_eq!(message.query("PID-5.1")?.as_deref(), Some("SMITH"));`;

  const links = (category: CrateCategory) =>
    cratesIn(category).map((crate) => ({
      href: `/crates/${crate.slug}/`,
      label: crate.name,
      blurb: crate.tagline
    }));
</script>

<DocPage
  title="Crates"
  lede={`One page per workspace member — ${CRATES.length} of them — with what it does, what it depends on, what its features do, and a tour of its API. Start with hl7 or hl7-2 unless you know otherwise.`}
  {contents}
>
  {#each order as category (category)}
    <h2 id={category}>{CATEGORIES[category].label}</h2>
    <p>{CATEGORIES[category].blurb}</p>
    <LinkCards links={links(category)} label={CATEGORIES[category].label} />
  {/each}

  <h2 id="er7">And one crate outside the workspace</h2>
  <p>
    <a href="https://crates.io/crates/er7"><code>er7</code></a> is the ER7 encoding layer: delimiters,
    escapes, paths, byte-for-byte rendering, and batch splitting. It has no dependencies of its own,
    and it lives in <a href="https://github.com/er7-rust/er7-rust">its own repository</a>, in its own
    organization, rather than in this workspace — because plenty of people want the encoding
    without the dictionary.
  </p>
  <CodeSample language="rust" code={er7} />
  <p>
    You rarely add it directly. <code>hl7-2</code> and the conversion crates re-export what you
    need, and <code>hl7-2</code>'s byte-for-byte round-trip guarantee is really
    <code>er7</code>'s guarantee, kept intact.
  </p>

  <h2 id="all">All of them, at a glance</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Crate</th><th>Layer</th><th>Version</th><th>Depends on</th><th>Spec</th><th>CLI</th>
        </tr>
      </thead>
      <tbody>
        {#each CRATES as crate (crate.name)}
          <tr>
            <td><a href={`/crates/${crate.slug}/`}><code>{crate.name}</code></a></td>
            <td>{CATEGORIES[crate.category].label}</td>
            <td>{crate.version}</td>
            <td>{crate.dependencies.length === 0 ? '—' : crate.dependencies.join(', ')}</td>
            <td>{crate.spec ? 'yes' : '—'}</td>
            <td>{crate.binary ? crate.binary : '—'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    Versions are the latest published at the time of writing, shown so you can gauge how settled a
    crate is — not as a constraint to copy. See <a href="/docs/versions/">Versions and
    compatibility</a>, and <a href="/docs/architecture/">Architecture</a> for why the seams fall
    where they do.
  </p>
</DocPage>
