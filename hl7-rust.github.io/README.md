# hl7-rust.github.io

Source for the [HL7® Rust](https://hl7-rust.github.io) site: documentation,
guides, tutorials, examples, and a reference page for every crate in the
[hl7-rust/hl7-rust](https://github.com/hl7-rust/hl7-rust) workspace.

Built with [SvelteKit](https://svelte.dev/docs/kit) (static adapter) and the
[Lily Design System](https://github.com/LilyDesignSystem) headless Svelte
components, styled with Lily's `light` theme plus a dark token layer.

## Where this lives, and where to edit it

The source of truth is the `hl7-rust.github.io/` directory of the
[hl7-rust/hl7-rust](https://github.com/hl7-rust/hl7-rust) workspace. The
standalone [hl7-rust/hl7-rust.github.io](https://github.com/hl7-rust/hl7-rust.github.io)
repository holds the same directory as its root, published from the workspace
with `make publish`.

It has to work that way: <https://hl7-rust.github.io> is an organization GitHub
Pages site, and GitHub only ever serves one from a repository named
`hl7-rust.github.io` — so the site cannot be published from the workspace
directly, even though that is where its source belongs.

**Edit in the workspace, not in the published repository.** A commit made
directly there is not in the workspace's history, and `make publish` forces, so
it would be overwritten without warning.

## Publishing

From the **workspace root**, not from this directory:

```sh
make publish
```

That splits this directory out of the workspace's history and pushes the result
to the `hl7-rust.github.io` repository, adding the `website` remote first if the
clone does not have it. It pushes as you, over your own SSH key, which is what
makes it work at all: a CI job would need a credential able to write
`.github/workflows/deploy.yml` in the far repository, and GitHub refuses that to
a deploy key. Publishing is therefore deliberate rather than automatic — run it
when a change is ready to be seen.

The push is forced, because the two histories are unrelated: that repository
grew on its own before this directory existed, and everything it held before
the switch is kept on its `archive/standalone` branch. Forcing is also what
lets a rewrite of the workspace's history be published at all. The cost is that
a commit made directly in the website repository is overwritten rather than
reported — so do not make one.

Once the push lands, the `deploy.yml` in this directory — which arrives at that
repository's root — runs `pnpm check` and `pnpm build` and publishes `build/`
to GitHub Pages.

**Nothing here is normative.** Each crate's own `spec/index.md` is the single
source of truth for its behavior; this site summarises the crates' READMEs and
specs and links back to them. If the two disagree, the spec is right and this
site is the bug.

## Development

```sh
pnpm install
pnpm dev      # http://localhost:5173
pnpm build    # static output in build/
pnpm preview  # preview the production build
pnpm check    # svelte-check
```

Deployment happens in two hops, and the first one is manual: `make publish`
from the workspace root sends this directory to the `hl7-rust.github.io`
repository (see [Publishing](#publishing)), and its `deploy.yml` then runs
`pnpm check` and `pnpm build` and publishes `build/` to GitHub Pages.

## Layout

```
src/routes/+layout.svelte        Site chrome: skip link, header, section
                                 sidebar, footer — and the only place that
                                 owns prose styling.
src/routes/+page.svelte          Home.
src/routes/docs/                 Install, quick start, concepts,
                                 architecture, command line, versions.
src/routes/guides/               One task per page.
src/routes/tutorials/            Longer start-to-finish walkthroughs.
src/routes/examples/             Copyable snippets, grouped by task.
src/routes/crates/<name>/        One page per workspace member.
src/routes/help/                 FAQ, troubleshooting, support.
src/routes/spec/                 Pointers to each crate's normative spec.
src/routes/sitemap.xml/          Generated from the navigation data.

src/lib/data/crates.ts           The crate catalog. Every list, table, and
                                 link to a crate reads it from here.
src/lib/data/navigation.ts       The site map. The header nav, the sidebar,
                                 previous/next, and the sitemap all read it.

src/lib/components/DocPage.svelte    Page shell: breadcrumb, title, lede,
                                     on-this-page, previous/next.
src/lib/components/CodeSample.svelte Labelled code block with a copy button.
src/lib/components/CrateMeta.svelte  The facts panel on a crate page.
src/lib/components/Callout.svelte    Note and warning.
src/lib/components/LinkCards.svelte  The card grid every index page uses.
src/lib/components/ThemeToggle.svelte     Site wiring around the published
                                          lily-design-system-svelte-theme-picker.
src/lib/components/TextSizeControl.svelte Site wiring around the published
                                          lily-design-system-svelte-text-size-picker.
src/lib/components/ShareControl.svelte    Site wiring around the published
                                          lily-design-system-svelte-share-picker.

static/lily-light.css            Lily "light" theme, copied verbatim from
                                 lily-design-system/themes/light.css.
static/lily-dark.css             Only the token block from Lily's "dark"
                                 theme, re-scoped to :root[data-theme="dark"]
                                 and to prefers-color-scheme, so the two
                                 layer instead of replacing each other.
```

## Adding a page

1. Add the route under `src/routes/`, using `DocPage` for the shell.
2. Add it to the right section's `links` in `src/lib/data/navigation.ts`.

That second step is what puts it in the sidebar, in the section index, in the
previous/next chain, and in the sitemap. Nothing else needs editing.

## Adding a crate

Add an entry to `CRATES` in `src/lib/data/crates.ts`, then add a page under
`src/routes/crates/<name>/`. The crate index, the architecture tables, the
install table, the versions table, and the sidebar all pick it up from the
catalog.

## Updating the theme

`static/lily-light.css` is a verbatim copy of Lily's `themes/light.css` — do
not hand-edit it; re-copy it. `static/lily-dark.css` is generated from Lily's
`themes/dark.css` by taking only its token block and re-scoping it; regenerate
it the same way if the upstream tokens change.

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
