<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { postBySlug, formatDate } from '$lib/data/news';

  const post = postBySlug('what-this-project-claims');

  const contents = [
    { id: 'problem', label: 'The question every evaluation asks' },
    { id: 'conformance', label: 'A conformance statement with numbers in it' },
    { id: 'benchmarks', label: 'Benchmarks, with their method' },
    { id: 'phi', label: 'What we do with patient data' },
    { id: 'rest', label: 'And three smaller things' },
    { id: 'next', label: 'What would improve these' }
  ];

  const evaluate = `hl7-v2 --tree --paths redacted-sample.hl7   # what does it name?
hl7-v2 --er7 redacted-sample.hl7 | diff - redacted-sample.hl7
hl7-v2 --check redacted-sample.hl7          # how big is the gap?`;
</script>

<DocPage
  eyebrow={formatDate(post.date)}
  lede={post.summary}
  {contents}
>
  <h2 id="problem">The question every evaluation asks</h2>
  <p>
    “Supports HL7 v2” is a claim every HL7 library makes and almost none define. It is also not the
    question anyone actually has. The real questions are narrower and harder: <em>which</em>
    segments do you know about, what happens to the ones you have never seen, how fast is it on our
    message sizes, and what does it do with the patient data we hand it?
  </p>
  <p>
    Those are answerable. They just have to be written down, with numbers, in a form somebody can
    check. Three documents are published today that do that, and none of them is flattering
    throughout — which is rather the point.
  </p>

  <h2 id="conformance">A conformance statement with numbers in it</h2>
  <p>
    <a href="/docs/conformance/">Conformance</a> states the coverage exactly: 24 segments, 42
    composite data types, and 4 message structures in the complete v2.5 base, with the other
    thirteen releases modelled as deltas of it. Every segment and type is listed by name, so there
    is nothing to guess at.
  </p>
  <p>
    HL7 v2.5 defines well over a hundred segments. Publishing “24” invites an obvious objection, and
    the answer to it is the design decision the whole project rests on:
  </p>
  <p>
    <strong>
      An unmodelled segment, field, data type, structure, or release difference costs you a
      <em>name</em>, never a <em>value</em>.
    </strong>
  </p>
  <p>
    An unknown segment parses. Its fields read positionally instead of by data type, a warning says
    so, every value is still reachable by path, and rendering still returns the original bytes. Only
    four things fail a call at all, and none of them is “I have not seen this segment before”. A
    library that rejected what it did not recognise would turn every coverage gap into a dropped
    clinical message.
  </p>
  <p>
    So the coverage number bounds how much of your message we can <em>name</em>, not how much we can
    carry. Those are very different claims, and conflating them is how “supports HL7 v2” became
    meaningless.
  </p>
  <Callout heading="You can check this in an hour, on your own feed">
    <p>On a redacted sample, and without writing any Rust:</p>
  </Callout>
  <CodeSample language="sh" code={evaluate} />

  <h2 id="benchmarks">Benchmarks, with their method</h2>
  <p>
    <a href="/docs/benchmarks/">Benchmarks</a> publishes measured figures for parsing, reading a
    field, building the tree, validating, and rendering — each with its confidence interval, on a
    named machine, with a named toolchain, on a stated date, from benchmarks that are in the
    repository and that you can run with one command.
  </p>
  <p>The two figures worth carrying away:</p>
  <ul>
    <li>
      <strong>Parsing a small ADT costs about 3 µs.</strong> One core parses on the order of 300,000
      a second. For essentially every real interface, parsing is not the bottleneck — the network and
      the downstream system are. If you are choosing a library on parse speed, you are optimising
      the wrong number.
    </li>
    <li>
      <strong>Reading two fields by path costs about 4 µs; building the whole tree of the same
      message costs 1.5 ms.</strong> Nearly 400 times more. Use paths.
    </li>
  </ul>
  <p>
    That second one is the most useful thing in the document and it is a criticism of our own slowest
    operation. The tree on a large message is slower than parsing that message four times over. It
    has had no optimisation attention. Saying so is more useful than omitting the row, and it tells
    you exactly where to look if a profile ever points that way.
  </p>
  <p>
    What is deliberately <em>not</em> there: any comparison against HAPI, Mirth, Open Integration
    Engine, or another Rust crate. Comparing fairly means matching what each one actually does, and a
    parser that only splits on pipes is not doing the same work as one that resolves a dictionary.
    Until that comparison exists with its method published, no claim is made. The honest,
    capability-based version is <a href="/docs/comparison/">Compared with the alternatives</a>, which
    includes four cases where you should choose something else.
  </p>

  <h2 id="phi">What we do with patient data</h2>
  <p>
    <a href="/docs/patient-data/">Patient data</a> is the document a security review actually needs.
    The short version: a message goes in as text, stays in memory as text, and comes back out when
    you ask for it. Nothing is written to disk, sent over a network, logged, counted, or cached. The
    libraries do not open files, read environment variables, spawn processes, or open sockets — not
    conditionally, not on a feature flag, not at all. Each of those is a grep away from being
    confirmed, and the page says which grep.
  </p>
  <p>
    The part that is not reassuring, and is the reason the page exists: <strong>error and diagnostic
    messages can quote a value from the message.</strong> A bad-value error carries the offending
    text verbatim, and a value-format diagnostic formats the value into its detail string. Log the
    whole error and you have logged a value from a clinical record — into the system that is
    typically shipped, aggregated, and retained under different rules than your message store.
  </p>
  <p>
    The page shows how to match on the error and drop that field. It also lists what is not defended
    against: memory is not zeroed, there is no de-identification function, no audit trail, and no
    encryption anywhere. A vague answer to a security review is worse than a stated limitation.
  </p>

  <h2 id="rest">And three smaller things</h2>
  <p>
    A <a href="https://github.com/hl7-rust/hl7-rust/blob/main/CONTRIBUTING.md">CONTRIBUTING.md</a>
    that leads with “never paste patient data” and shows the redaction that keeps a bug report
    useful. Benchmarks in <code>hl7-2</code> itself, not only in the conversion crates. And this
    news section, so that the next thing worth saying has somewhere to go that is not somebody
    else's platform.
  </p>

  <h2 id="next">What would improve these</h2>
  <p>
    All three documents have stated gaps, and each is a good first contribution:
  </p>
  <ul>
    <li>
      <strong>Dictionary coverage.</strong> The fastest way to widen it is a redacted message that
      reads positionally when it should not. That is one JSON file and a test.
    </li>
    <li>
      <strong>Memory measurement.</strong> The benchmarks report time only. Allocations and peak
      resident size are missing.
    </li>
    <li>
      <strong>A fair cross-library comparison.</strong> Hard to do honestly, which is why it does not
      exist yet — but it is the thing an evaluation most wants and cannot get anywhere.
    </li>
  </ul>
  <p>
    If one of these documents claims something the code does not do, that is a bug and worth filing.
    Each is written so that it can be checked rather than believed, which only works if people
    actually check.
  </p>
</DocPage>
