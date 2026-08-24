<script lang="ts">
  /**
   * The facts panel at the top of a crate page: where it lives, what version
   * was published when this page was written, what it depends on, and what
   * its Cargo features do.
   *
   * Everything here is read from the catalog in $lib/data/crates.ts, so a crate
   * page cannot drift out of step with the crate list.
   */
  import { Badge, Table, TableBody, TableHead, TableRow, TableTD, TableTH } from 'lily-design-system-svelte-headless';
  import {
    CATEGORIES,
    CRATES,
    cratesIoUrl,
    docsRsUrl,
    repoUrl,
    specUrl,
    type Crate
  } from '$lib/data/crates';

  let { crate }: { crate: Crate } = $props();

  const spec = $derived(specUrl(crate));
  /** Dependencies that are themselves in this workspace get an internal link. */
  const internal = new Set(CRATES.map((candidate) => candidate.name));
</script>

<div class="crate-meta">
  <p class="crate-meta-badges">
    <Badge label="Category">{CATEGORIES[crate.category].label}</Badge>
    <Badge type="info" label="Latest published version">v{crate.version}</Badge>
    {#if crate.binary}
      <Badge type="success" label="Ships a command-line binary">CLI: {crate.binary}</Badge>
    {/if}
    {#if crate.spec}
      <Badge label="Has a normative specification">Specified</Badge>
    {/if}
  </p>

  <Table label={`${crate.name} at a glance`} class="crate-meta-table">
    <TableBody>
      <TableRow>
        <TableTH scope="row">Install</TableTH>
        <TableTD><code>cargo add {crate.name}</code></TableTD>
      </TableRow>
      <TableRow>
        <TableTH scope="row">Rust path</TableTH>
        <TableTD><code>{crate.ident}</code></TableTD>
      </TableRow>
      <TableRow>
        <TableTH scope="row">Dependencies</TableTH>
        <TableTD>
          {#if crate.dependencies.length === 0}
            None, and staying that way.
          {:else}
            {#each crate.dependencies as dependency, index (dependency)}{#if index > 0}, {/if}{#if internal.has(dependency)}<a
                  href={`/crates/${dependency}/`}><code>{dependency}</code></a
                >{:else}<code>{dependency}</code>{/if}{/each}
          {/if}
        </TableTD>
      </TableRow>
      <TableRow>
        <TableTH scope="row">Links</TableTH>
        <TableTD>
          <a href={cratesIoUrl(crate)}>crates.io</a>
          <span aria-hidden="true">·</span>
          <a href={docsRsUrl(crate)}>docs.rs</a>
          <span aria-hidden="true">·</span>
          <a href={repoUrl(crate)}>source</a>
          {#if spec}
            <span aria-hidden="true">·</span>
            <a href={spec}>spec</a>
          {/if}
        </TableTD>
      </TableRow>
    </TableBody>
  </Table>

  {#if crate.features && crate.features.length > 0}
    <h2 id="features">Cargo features</h2>
    <Table label={`${crate.name} Cargo features`}>
      <TableHead>
        <TableRow>
          <TableTH scope="col">Feature</TableTH>
          <TableTH scope="col">Default</TableTH>
          <TableTH scope="col">Effect</TableTH>
        </TableRow>
      </TableHead>
      <TableBody>
        {#each crate.features as feature (feature.name)}
          <TableRow>
            <TableTD><code>{feature.name}</code></TableTD>
            <TableTD>{feature.default ? 'on' : 'off'}</TableTD>
            <TableTD>{feature.effect}</TableTD>
          </TableRow>
        {/each}
      </TableBody>
    </Table>
  {/if}
</div>

<style>
  .crate-meta {
    margin: 2rem 0;
  }

  .crate-meta-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin: 0 0 1.25rem;
  }

  /* Lily's baseline badge is a solid neutral pill. Four of them in a row reads
     as a warning strip, so these are outline chips instead, tinted by type. */
  :global(.crate-meta-badges .badge) {
    background: transparent;
    color: inherit;
    border: 1px solid var(--color-base-300);
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.1875rem 0.625rem;
  }

  :global(.crate-meta-badges .badge[data-type='info']) {
    border-color: var(--color-info);
    color: var(--color-info);
  }

  :global(.crate-meta-badges .badge[data-type='success']) {
    border-color: var(--color-success);
    color: var(--color-success);
  }

  :global(.crate-meta .table) {
    width: 100%;
  }

  :global(.crate-meta-table th) {
    width: 10rem;
    white-space: nowrap;
  }
</style>
