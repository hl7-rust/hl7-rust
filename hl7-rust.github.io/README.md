# hl7-rust.github.io

Source for the [HL7 Rust](https://hl7-rust.github.io) site: documentation,
guides, tutorials, examples, and a reference page for every crate in the
[hl7-rust/hl7-rust](https://github.com/hl7-rust/hl7-rust) workspace.

Built with [SvelteKit](https://svelte.dev/docs/kit) (static adapter) and the
[Lily Design System](https://github.com/LilyDesignSystem) headless Svelte
components, styled with Lily's `light` theme plus a dark token layer.

## Where this lives, and where to edit it

The source of truth is the `hl7-rust.github.io/` directory of the
[hl7-rust/hl7-rust](https://github.com/hl7-rust/hl7-rust) workspace. The
standalone [hl7-rust/hl7-rust.github.io](https://github.com/hl7-rust/hl7-rust.github.io)
repository is a **mirror** of that directory, kept in step by the workspace's
`.github/workflows/publish-website.yml` on every push to `main`.

It has to work that way: <https://hl7-rust.github.io> is an organization GitHub
Pages site, and GitHub only ever serves one from a repository named
`hl7-rust.github.io` — so the site cannot be published from the workspace
directly, even though that is where its source belongs.

**Edit in the workspace, not in the mirror.** A commit made directly to the
mirror is overwritten by the next push to the workspace.

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

Deployment is automatic, in two hops. A push to the workspace's `main` mirrors
this directory into the `hl7-rust.github.io` repository; the `deploy.yml` in
this directory — which lands at that repository's root — then runs `pnpm check`
and `pnpm build` and publishes `build/` to GitHub Pages.

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
src/lib/components/ThemeToggle.svelte

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
