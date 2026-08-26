<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'category', label: 'First, which kind of thing do you need?' },
    { id: 'engines', label: 'Interface engines' },
    { id: 'hapi', label: 'HAPI, and the mature libraries' },
    { id: 'rust', label: 'The other Rust crates' },
    { id: 'yourself', label: 'Splitting on pipes yourself' },
    { id: 'here', label: 'What this project offers' },
    { id: 'not', label: 'When this project is the wrong answer' }
  ];

  const naive = `// The bug: this is not how HL7 works.
let fields: Vec<&str> = segment.split('|').collect();
let name = fields[5];`;

  const rustCrates = [
    {
      name: 'hl7-mllp-codec',
      version: '0.4.0',
      published: '22 July 2022',
      downloads: '25,755',
      scope: 'A Tokio codec for MLLP framing. Transport only, no v2 semantics.'
    },
    {
      name: 'hl7-parser',
      version: '0.3.0',
      published: '24 February 2025',
      downloads: '16,625',
      scope: 'Parses message structure; states that it does not validate correctness.'
    },
    {
      name: 'rust-hl7',
      version: '0.5.0',
      published: '8 September 2021',
      downloads: '14,777',
      scope: 'Parser and object builder; describes itself as experimental.'
    }
  ];
</script>

<DocPage
  title="Compared with the alternatives"
  lede="Interface engines, the mature Java libraries, the other Rust crates, and the pipe-splitting you were about to write yourself. When each is the right answer, and when this project is the wrong one."
  {contents}
>
  <Callout heading="No performance comparison is claimed here">
    <p>
      Nothing in this project has been benchmarked against another library. Doing that fairly means
      matching what each one actually does, and a parser that only splits on pipes is not doing the
      same work as one that resolves a dictionary. So this page compares <em>capability and
      shape</em>, which is checkable, rather than speed, which would not be. Our own measured
      figures, with their method, are on <a href="/docs/benchmarks/">Benchmarks</a>.
    </p>
  </Callout>

  <h2 id="category">First, which kind of thing do you need?</h2>
  <p>
    Most comparisons in this space go wrong by putting products from three different categories in
    one table. The honest first question is what you are building.
  </p>
  <dl>
    <dt>You need to <em>run</em> interfaces — routes, retries, queues, monitoring, on-call</dt>
    <dd>You want an interface engine. Skip to the next section; a library is not that.</dd>
    <dt>You are writing an application that happens to speak HL7 v2</dt>
    <dd>You want a library. That is what this project is.</dd>
    <dt>You need a one-off transformation at a shell prompt</dt>
    <dd>
      You want a command-line tool, and <a href="/docs/cli/">the ones here</a> may be enough on their
      own — no Rust required.
    </dd>
  </dl>

  <h2 id="engines">Interface engines</h2>
  <p>
    <a href="https://github.com/OpenIntegrationEngine">Open Integration Engine</a> — the
    community fork of Mirth Connect, after Mirth moved to a commercial-only licence at 4.6 in 2025 —
    and its commercial siblings are a different category of thing entirely. An engine gives you
    channels, routing, a management UI, JavaScript transformers, persistence, retry and alerting, and
    an operations story. It is a system you deploy and run.
  </p>
  <p>
    A library gives you a function call. If your problem is “forty interfaces, three hospitals, and
    someone has to be paged when one stops”, an engine is the right answer and no amount of crate is
    going to substitute for it.
  </p>
  <p>These crates are useful <em>alongside</em> an engine rather than instead of it:</p>
  <ul>
    <li>The service at the end of a channel, where you would otherwise be writing the v2 parsing again.</li>
    <li>
      A shell-level check or transformation, using the command-line tools, without standing anything
      up.
    </li>
    <li>
      A dedicated high-volume path where a JVM-per-message and a channel round trip are more than
      the job needs.
    </li>
  </ul>

  <h2 id="hapi">HAPI, and the mature libraries</h2>
  <p>
    <a href="https://hapifhir.github.io/hapi-hl7v2/">HAPI HL7v2</a> is the reference open-source HL7
    v2 library, in Java, dual licensed under MPL 1.1 and GPL 2.0. It has been maintained for two
    decades, ships a generated typed model for every segment and message of every release, and has
    seen far more real-world traffic than anything here. Its .NET port, and the mature Python
    libraries, are in the same position.
  </p>
  <p>
    <strong>If your platform is the JVM, use HAPI.</strong> That is not modesty; a twenty-year-old
    library with complete release coverage and a large user base is the lower-risk choice, and
    reimplementing it in a language you were not otherwise using is a bad trade.
  </p>
  <p>Where this project differs, stated as trade-offs rather than wins:</p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th></th><th>The mature libraries</th><th>Here</th></tr>
      </thead>
      <tbody>
        <tr>
          <td>Release coverage</td>
          <td>Complete generated model, every segment, every release</td>
          <td>24 segments, 42 types, 4 structures, extensible in JSON — see <a href="/docs/conformance/">Conformance</a></td>
        </tr>
        <tr>
          <td>Runtime</td>
          <td>A JVM, or a CLR, or a Python interpreter</td>
          <td>A static binary, no runtime, no GC</td>
        </tr>
        <tr>
          <td>Dependency tree</td>
          <td>Substantial, and audited as such</td>
          <td>One crate, itself dependency-free</td>
        </tr>
        <tr>
          <td>Track record</td>
          <td>Two decades of production traffic</td>
          <td>Published in 2026. New.</td>
        </tr>
        <tr>
          <td>Licence</td>
          <td>MPL 1.1 or GPL 2.0 for HAPI</td>
          <td>MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or GPL-3.0-only, at your option</td>
        </tr>
      </tbody>
    </table>
  </div>
  <p>
    The licensing row is the one that decides some evaluations outright. A permissive option matters
    if you are linking into a closed-source product; a copyleft option matters if your organisation
    prefers one. Offering five is how this project avoids having that conversation with anyone.
  </p>

  <h2 id="rust">The other Rust crates</h2>
  <p>
    If Rust is already your platform, the existing options are narrower than they first appear.
    Figures from crates.io, checked 26 August 2026:
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Crate</th><th>Latest</th><th>Published</th><th>Downloads</th><th>Scope, as it describes itself</th></tr>
      </thead>
      <tbody>
        {#each rustCrates as crate (crate.name)}
          <tr>
            <td><code>{crate.name}</code></td>
            <td>{crate.version}</td>
            <td>{crate.published}</td>
            <td>{crate.downloads}</td>
            <td>{crate.scope}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    Read those publication dates carefully rather than dismissively. A library that has not been
    released in three years may be dormant, or may simply be finished and stable for what it does —
    <code>hl7-mllp-codec</code> in particular does one small thing and does it well, and if you are
    already on Tokio and want framing alone, it remains a reasonable choice.
  </p>
  <p>
    What none of them offers is the whole span: a release dictionary, validation, mutation and
    building, format conversion, and both transports, maintained together. That gap is the reason
    this project exists.
  </p>

  <h2 id="yourself">Splitting on pipes yourself</h2>
  <p>
    This is the real competition, and usually it is the incumbent: a hundred lines somewhere in the
    codebase that split a segment on <code>|</code> and index the result.
  </p>
  <CodeSample language="rust" code={naive} />
  <p>Each of the following is a production incident that code has already caused somewhere:</p>
  <ul>
    <li>
      <strong>The delimiters are declared by the message</strong>, in MSH-1 and MSH-2. The usual set
      is a convention, not a guarantee, and a sender that uses different ones will be parsed into
      nonsense rather than rejected.
    </li>
    <li>
      <strong>Escape sequences</strong> mean an ampersand or a backslash inside a value is not what
      it looks like. Splitting first and decoding later is the wrong order.
    </li>
    <li>
      <strong>The explicit null is not an empty field.</strong> One says “we have nothing here”; the
      other says “delete what you have on file”. Collapsing the two writes wrong data into a patient
      record, silently.
    </li>
    <li>
      <strong>Repetitions, components, and subcomponents</strong> are four levels deep, and the
      naive version handles one.
    </li>
    <li>
      <strong>MSH is off by one</strong>, because MSH-1 is the field separator itself. Every
      hand-rolled parser meets this bug.
    </li>
  </ul>
  <p>
    None of that is a reason to feel bad about the hundred lines; it is a reason to replace them with
    something whose round trip is a test. If you replace them with a different library than this one,
    the goal is still met.
  </p>

  <h2 id="here">What this project offers</h2>
  <p>Stated without adjectives, so you can check each one:</p>
  <ul>
    <li>
      <strong>One dependency.</strong> <code>hl7-2</code> depends on <code>er7</code>, which depends
      on nothing. That is the whole tree, which matters where dependency trees get audited.
    </li>
    <li>
      <strong>No runtime.</strong> A static binary, no JVM, no GC pause, and a command-line tool an
      integration analyst can use without writing any Rust.
    </li>
    <li>
      <strong>Byte-for-byte round trip</strong>, as a test rather than an aspiration.
    </li>
    <li>
      <strong>The explicit null kept distinct</strong> from an absent value, at every level.
    </li>
    <li>
      <strong>A stated conformance position</strong> — <a href="/docs/conformance/">exactly which</a>
      segments, types, and structures, and what happens outside them.
    </li>
    <li>
      <strong>A stated position on patient data</strong> — <a href="/docs/patient-data/">what these
      crates do with it</a>, and where a value can escape into a log.
    </li>
    <li>
      <strong>An MSRV of current stable minus three</strong>, because hospital toolchains are
      approved on a cycle measured in quarters.
    </li>
    <li>
      <strong>Five licences, at your option</strong>, so licensing is not a conversation.
    </li>
    <li>
      <strong>A vendor dialect is one JSON file</strong>, or generated from your own XSDs.
    </li>
  </ul>

  <h2 id="not">When this project is the wrong answer</h2>
  <Callout type="warning" heading="Read this section before adopting, not after">
    <p>The four cases where you should choose something else:</p>
  </Callout>
  <ol>
    <li>
      <strong>You need FHIR.</strong> Not implemented here, at all. The umbrella crate reserves a
      module path and nothing more.
    </li>
    <li>
      <strong>You need CDA</strong>, or a substantial HL7 v3 implementation. The v3 crate is a
      foundation — six RIM classes, six data types, a generic envelope — and says so in its own first
      section.
    </li>
    <li>
      <strong>You are on the JVM, .NET, or Python already.</strong> Use the mature library for your
      platform. Adding a language to a hospital's supported stack is a bigger decision than picking a
      parser.
    </li>
    <li>
      <strong>You need an operations story, not a function call.</strong> Routing, retries, queues,
      monitoring, and an on-call runbook are an interface engine's job.
    </li>
  </ol>
  <p>
    A fifth, softer one: this project is new, published in 2026, at 0.x. If your risk posture needs a
    long production track record, it does not have one yet. That is a fact about the calendar, and
    the only honest thing to do is say so.
  </p>
</DocPage>
