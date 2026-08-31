<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'contents', label: 'What is inside it' },
    { id: 'checklist', label: 'The checklist it exists to save you from forgetting' },
    { id: 'using-it', label: 'Using it' }
  ];

  const frontmatter = `---
name: hl7-rust-maintainer-skill
description: Technical, implementation-level guidance for maintainers
  and contributors changing code, specs, or docs inside the hl7-rust
  workspace itself — repo layout, the spec-first rule, the exact pre-PR
  checklist, adding dictionary coverage, benchmarking a performance
  claim, fixing the website, and what CI actually gates. Use when the
  task modifies this repository's own crates, specs, docs, or website
  (not when the task is merely using the published crates to process
  an HL7® message — see hl7-skill for that).
---`;

  const checklist = `cargo test                                    # unit and integration tests
cargo clippy --all-targets -- -D warnings     # lint-clean
cargo fmt --check                             # formatting
cargo rustdoc --lib -- -W missing-docs        # every public item documented
cargo +1.96 check --workspace --all-targets   # the MSRV floor
./bin/check-trademarks                        # HL7®/FHIR®/CDA® fair-use rules
./bin/check-docs                              # doc size budget + link integrity`;
</script>

<DocPage
  lede="hl7-rust-maintainer-skill/ is the second Agent Skill in this workspace — implementation-level conventions for changing this repository's own crates, specs, and docs, rather than for using the published crates."
  {contents}
>
  <h2 id="what">What it is</h2>
  <p>
    <a href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-rust-maintainer-skill"><code>hl7-rust-maintainer-skill/</code></a>
    sits beside <a href="/docs/agent-skill/"><code>hl7-skill/</code></a> at
    the workspace root, in the same
    <a href="https://code.claude.com/docs/en/skills">Agent Skills</a>
    format. Where <code>hl7-skill</code> is for anyone using HL7 or this
    workspace, this one is for anyone changing the workspace itself —
    fixing a bug, widening dictionary coverage, editing a spec, or
    touching the website.
  </p>
  <CodeSample language="yaml" caption="hl7-rust-maintainer-skill/SKILL.md — frontmatter" code={frontmatter} />

  <h2 id="contents">What is inside it</h2>
  <p>
    It distills <code>AGENTS.md</code> and <code>CONTRIBUTING.md</code>
    rather than replacing them — both stay canonical, and this skill is
    updated alongside them when a convention changes:
  </p>
  <ul>
    <li>
      The repo layout: one <code>Cargo.lock</code> at the root, members
      depending on each other by relative path, and where MSRV is pinned
      once and inherited.
    </li>
    <li>
      <strong>The rule that matters most</strong> — each crate's
      <code>spec/index.md</code> is the single source of truth for its
      behavior; a code change that contradicts it is either a bug fix or
      an unstated spec change, and the two must never drift.
    </li>
    <li>The exact pre-PR checklist, reproduced below.</li>
    <li>
      Recipes for the contributions that come up most: adding dictionary
      coverage, arguing a performance change with a benchmark
      before/after, and fixing the website in the right repository.
    </li>
    <li>Exactly what each CI job in <code>ci.yml</code> gates.</li>
  </ul>

  <h2 id="checklist">The checklist it exists to save you from forgetting</h2>
  <CodeSample language="sh" code={checklist} />
  <p>
    Every line here is exactly what <code>.github/workflows/ci.yml</code>
    runs — passing it locally is passing CI, not a proxy for it. Touched
    <code>hl7-rust.github.io/</code>? Its own <code>pnpm run check</code>
    and <code>pnpm run build</code> are the remaining two gates.
  </p>
  <Callout type="warning" heading="Never paste real patient data">
    Not into an issue, a commit, a test fixture, or a prompt to an AI
    tool — synthesize instead. See <a href="/docs/patient-data/">Patient data</a>.
  </Callout>

  <h2 id="using-it">Using it</h2>
  <p>
    Same as its sibling: nothing to install. An agent working in this
    checked-out repository picks up whichever skill's description
    matches the task — <a href="/docs/agent-skill/">Agent skill</a> for
    using HL7 or this workspace, this one for changing the workspace
    itself.
  </p>
</DocPage>
