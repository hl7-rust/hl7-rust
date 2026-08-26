<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import LinkCards from '$lib/components/LinkCards.svelte';
  import { SECTIONS } from '$lib/data/navigation';
  import { ORG, REPO } from '$lib/data/crates';

  const help = SECTIONS.find((section) => section.href === '/help/')!;

  const contents = [
    { id: 'pages', label: 'In this section' },
    { id: 'where-to-look', label: 'Where to look first' },
    { id: 'quick', label: 'Quick answers' }
  ];
</script>

<DocPage
  title="Help"
  lede="Answers to the questions that come up before the first line of code, symptoms and fixes for the failures people actually hit, and where to ask when neither covers it."
  {contents}
>
  <h2 id="pages">In this section</h2>
  <LinkCards links={help.links} label="Help pages" />

  <h2 id="where-to-look">Where to look first</h2>
  <ol>
    <li>
      <strong>The crate's own <code>spec/index.md</code></strong> — numbered section by section, and
      the single source of truth. If the spec and anything else disagree, the spec is right. See
      <a href="/spec/">Specifications</a>.
    </li>
    <li>
      <strong>docs.rs</strong> for the crate, which is generated from the code that actually
      shipped.
    </li>
    <li>
      <strong><a href="/help/troubleshooting/">Troubleshooting</a></strong> on this site, for the
      failures with a known cause.
    </li>
    <li>
      <strong>The repository</strong> at <a href={REPO}>github.com/hl7-rust/hl7-rust</a> — issues,
      and the crate's own <code>AGENTS.md</code>, which documents the conventions a change has to
      meet.
    </li>
  </ol>

  <h2 id="quick">Quick answers</h2>
  <dl>
    <dt>Which crate do I install?</dt>
    <dd>
      <code>cargo add hl7</code>, and use <code>hl7::v2</code>. See
      <a href="/docs/#which-crate">Which crate do I need?</a>
    </dd>
    <dt>Why is the binary called <code>hl7-v2</code> when the crate is <code>hl7-2</code>?</dt>
    <dd>
      <code>hl7-v2</code> on crates.io is an unrelated package, so the crate took the other name; the
      binary kept the readable one. See <a href="/crates/hl7-2/#name">the crate page</a>.
    </dd>
    <dt>My message has segments the standard does not define. Is that a problem?</dt>
    <dd>
      No. Unknown segments convert positionally and never fail a parse, and a Z-segment does not fail
      strict mode either. When you want them <em>named</em>, write a
      <a href="/guides/dictionaries/">dictionary</a>.
    </dd>
    <dt>Is any of this a validator?</dt>
    <dd>
      <a href="/crates/hl7-2/"><code>hl7-2</code></a> checks a message against its dictionary and
      reports diagnostics. The four conversion crates check nothing at all. See
      <a href="/guides/validating/">Validating</a>.
    </dd>
    <dt>Can I use these commercially?</dt>
    <dd>
      Yes. Every crate is offered under MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or GPL-3.0-only,
      at your option. See <a href="/docs/versions/#license">Licensing</a>.
    </dd>
    <dt>Is the HL7® FHIR® standard supported?</dt>
    <dd>
      Not yet. The umbrella crate deliberately leaves <code>hl7::fhir</code> free for it.
    </dd>
  </dl>
  <p>
    More at <a href="/help/faq/">the FAQ</a>, and the organization's other repositories are at
    <a href={ORG}>github.com/hl7-rust</a>.
  </p>
</DocPage>
