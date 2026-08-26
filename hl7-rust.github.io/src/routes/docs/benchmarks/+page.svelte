<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { REPO } from '$lib/data/crates';

  const contents = [
    { id: 'figures', label: 'The figures' },
    { id: 'reading', label: 'Reading them' },
    { id: 'method', label: 'The method' },
    { id: 'inputs', label: 'The inputs' },
    { id: 'run', label: 'Running them yourself' },
    { id: 'compare', label: 'Comparing a change' },
    { id: 'not', label: 'What these are not' }
  ];

  const spec = `${REPO}/blob/main/spec/benchmark/index.md`;

  const rows = [
    { group: 'parse', input: 'small, 177 B', time: '3.00 µs', interval: '2.97 – 3.04 µs', thrpt: '56.3 MiB/s' },
    { group: 'parse', input: 'large, 29,104 B', time: '381 µs', interval: '376 – 388 µs', thrpt: '72.8 MiB/s' },
    { group: 'get', input: 'small, PID-5.1', time: '110 ns', interval: '108 – 112 ns', thrpt: '—' },
    { group: 'get', input: 'large, OBX[200]-5', time: '1.94 µs', interval: '1.91 – 1.97 µs', thrpt: '—' },
    { group: 'tree', input: 'small', time: '13.4 µs', interval: '13.3 – 13.6 µs', thrpt: '—' },
    { group: 'tree', input: 'large', time: '1.50 ms', interval: '1.49 – 1.52 ms', thrpt: '—' },
    { group: 'validate', input: 'small', time: '6.95 µs', interval: '6.87 – 7.06 µs', thrpt: '—' },
    { group: 'validate', input: 'large', time: '633 µs', interval: '628 – 640 µs', thrpt: '—' },
    { group: 'render', input: 'small, 177 B', time: '365 ns', interval: '360 – 372 ns', thrpt: '463 MiB/s' },
    { group: 'render', input: 'large, 29,104 B', time: '29.4 µs', interval: '28.9 – 30.1 µs', thrpt: '944 MiB/s' }
  ];

  const measured = [
    { operation: 'parse', what: 'Text in, Message out', paid: 'Once per message' },
    { operation: 'get', what: 'One field read by path', paid: 'Once per field you care about' },
    { operation: 'tree', what: 'The whole generic tree, every value named', paid: 'Only when something walks the message' },
    { operation: 'validate', what: 'The message against its dictionary', paid: 'Only when validation is asked for' },
    { operation: 'render', what: 'Message back to ER7 text', paid: 'Once per message written out' }
  ];

  const run = `cargo bench -p hl7-2              # the five groups
cargo bench                       # every crate that has benchmarks
cargo bench -p hl7-2 -- parse     # one group`;

  const compare = `git stash
cargo bench -p hl7-2 -- --save-baseline before
git stash pop
cargo bench -p hl7-2 -- --baseline before`;
</script>

<DocPage
  title="Benchmarks"
  lede="Measured figures for the five things hl7-2 is asked to do, the method that produced them, and what they are not evidence of. A number without its method is a marketing claim, so the method is here too."
  {contents}
>
  <h2 id="figures">The figures</h2>
  <p>
    <strong>Machine:</strong> Apple M4 Max, 128 GB, macOS 26.6.1, arm64.
    <strong>Toolchain:</strong> rustc 1.98.0, release profile.
    <strong>Date:</strong> 26 August 2026.
    <strong>Crates:</strong> <code>hl7-2</code> 0.2.3 over <code>er7</code> 0.1.2.
    <strong>Method:</strong> <code>cargo bench -p hl7-2</code>, Criterion defaults, machine
    otherwise idle. The middle column is Criterion's point estimate; the interval is its confidence
    interval, reported rather than quietly dropped.
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Group</th><th>Input</th><th>Time</th><th>Interval</th><th>Throughput</th></tr>
      </thead>
      <tbody>
        {#each rows as row (row.group + row.input)}
          <tr>
            <td><code>{row.group}</code></td>
            <td>{row.input}</td>
            <td>{row.time}</td>
            <td>{row.interval}</td>
            <td>{row.thrpt}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <h2 id="reading">Reading them</h2>
  <p>
    Four things in that table are worth saying out loud, including the unflattering one.
  </p>
  <h3>Parsing is not your bottleneck</h3>
  <p>
    A small message parses in about 3 µs, so one core parses on the order of 300,000 ADTs a second.
    For essentially every real HL7 interface, the network, the database, and the downstream system
    decide the throughput. If you are choosing a library on parse speed, you are optimising the
    wrong number.
  </p>
  <h3>Rendering is about eight times cheaper than parsing</h3>
  <p>
    365 ns against 3.00 µs on the same message. That is what “stored as sent, decoded on demand”
    buys: writing back out is mostly copying bytes that were never transformed, which is also why
    the round trip comes back byte for byte.
  </p>
  <h3>Use paths, not the tree — this is the useful one</h3>
  <p>
    Reading two fields from the large message costs about 4 µs. Building its whole tree costs
    1.50 ms: <strong>nearly 400 times more</strong>. An integration that wants a handful of fields
    should reach for <a href="/guides/navigating/">paths</a> and never materialise the tree.
  </p>
  <h3>The tree on a large message is the slowest thing here</h3>
  <p>
    Slower than parsing the same message four times over. It allocates a named node for every value
    in a 600-segment message, so the cost is real work rather than waste — but it has had no
    optimisation attention, and it is the first place to look if a profile points this way. Saying
    so is more useful than omitting the row.
  </p>

  <h2 id="method">The method</h2>
  <ol>
    <li>Every published figure names its machine, its toolchain, and its date.</li>
    <li>Every benchmark is in the repository and runnable by anyone, in one command.</li>
    <li>
      Inputs are synthetic. No real patient data, ever — see
      <a href="/docs/patient-data/">Patient data</a>.
    </li>
    <li>Criterion, defaults, no tuning, so the numbers compare with anyone else's Criterion run.</li>
    <li>A performance claim in a pull request carries a before-and-after from one machine in one sitting.</li>
    <li>
      Correctness outranks speed. A faster parser that loses a value, or stops round-tripping byte
      for byte, is not faster.
    </li>
  </ol>
  <p>
    The five operations are measured separately rather than as one end-to-end figure, because a
    given interface pays for only some of them:
  </p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Group</th><th>The operation</th><th>Paid</th></tr>
      </thead>
      <tbody>
        {#each measured as row (row.operation)}
          <tr><td><code>{row.operation}</code></td><td>{row.what}</td><td>{row.paid}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    One combined “messages per second” would hide which of those you are actually paying for. A feed
    that reads two fields and forwards the message pays <code>parse</code> plus two
    <code>get</code>s; a converter pays <code>parse</code> plus <code>tree</code>; a strict receiver
    pays <code>parse</code> plus <code>validate</code>.
  </p>

  <h2 id="inputs">The inputs</h2>
  <dl>
    <dt>small — 177 bytes</dt>
    <dd>
      A four-segment <code>ADT^A08</code>. The shape most interfaces move in bulk, and where
      per-message overhead dominates.
    </dd>
    <dt>large — 29,104 bytes</dt>
    <dd>
      An <code>ORU^R01</code> carrying 200 observations as OBR/OBX/NTE triples. The shape that
      decides whether a parser keeps up with a day's traffic, and where per-segment cost dominates.
    </dd>
  </dl>
  <p>
    Both are built in code at the top of the benchmark file, so the input is readable next to the
    measurement and cannot drift. Neither is a claim about a representative message mix — there
    isn't one. Site traffic varies more between two hospitals than between these two sizes. They
    bracket the range rather than averaging it.
  </p>

  <h2 id="run">Running them yourself</h2>
  <CodeSample language="sh" code={run} />
  <p>
    Benchmarks build under the <code>bench</code> profile, which is release with debug assertions
    off. Running them under the dev profile measures the debug build and is meaningless.
  </p>

  <h2 id="compare">Comparing a change</h2>
  <p>The only comparison that means anything is the same machine, minutes apart:</p>
  <CodeSample language="sh" code={compare} />
  <p>
    Criterion prints the change and whether it considers it significant. Treat anything under about
    5% on a laptop as noise — thermal state, other processes, and allocator behaviour move numbers
    by that much between runs that changed nothing.
  </p>

  <h2 id="not">What these are not</h2>
  <Callout type="warning" heading="Not a comparison against another library">
    <p>
      Nothing here has been benchmarked against HAPI, Mirth, Open Integration Engine, or another
      Rust crate. Doing that fairly means matching what each one actually does — and a parser that
      only splits on pipes is not doing the same work as one that resolves a dictionary. Until that
      comparison exists with its method published, no claim is made. See
      <a href="/docs/comparison/">Compared with the alternatives</a> for the honest,
      capability-based version.
    </p>
  </Callout>
  <ul>
    <li><strong>Not a guarantee.</strong> One run, one machine, one day, two synthetic messages.</li>
    <li>
      <strong>Not a throughput figure for a system.</strong> These measure a library call. MLLP
      framing, TCP, acknowledgement round trips, disk, and the database behind the interface are all
      outside them, and usually they set the ceiling.
    </li>
    <li>
      <strong>Not a memory measurement.</strong> Nothing here reports allocations or peak resident
      size. That is a gap, and a contribution adding it would be welcome.
    </li>
  </ul>
  <p>The normative version of this page, with the full rules, is <a href={spec}>spec/benchmark/index.md</a>.</p>
</DocPage>
