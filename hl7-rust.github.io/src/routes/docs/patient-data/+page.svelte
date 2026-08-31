<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { REPO } from '$lib/data/crates';

  const contents = [
    { id: 'short', label: 'The short version' },
    { id: 'never', label: 'What the libraries never do' },
    { id: 'escape', label: 'Where a value can escape' },
    { id: 'not-defended', label: 'What is not defended against' },
    { id: 'cli', label: 'The command-line tools' },
    { id: 'transports', label: 'The transports' },
    { id: 'ours', label: 'This project’s own data' },
    { id: 'review', label: 'If you are reviewing this for a deployment' }
  ];

  const spec = `${REPO}/blob/main/spec/phi/index.md`;

  const never = [
    { not: 'No logging or tracing', check: 'No log, tracing, or any logging facade in any Cargo.toml in the workspace' },
    { not: 'No telemetry, analytics, or phone-home', check: 'No HTTP client anywhere; no network dependency of any kind' },
    { not: 'No filesystem access from library code that touches patient messages', check: 'std::fs appears in the CLI and in one named exception, hl7-2-from-xsd-into-json-dictionary, which reads XSD schema files from disk to build a dictionary — never a patient message' },
    { not: 'No environment variables', check: 'std::env appears in no library source, only in the CLI’s argument parsing' },
    { not: 'No sockets opened', check: 'std::net appears in no library source; MLLP is generic over a byte stream you supply' },
    { not: 'No subprocesses', check: 'std::process appears in no library source' },
    { not: 'No serialization framework', check: 'No serde; the JSON reader is hand-written and reads only dictionaries' }
  ];

  const redacted = `MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5
PID|1||REDACTED^^^ACME&1.2.3.4&ISO^MR||REDACTED^REDACTED||REDACTED|F
OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F`;

  const logging = `match message.get("PID-5.1") {
    Err(hl7_2::Error::BadValue { path, expected, .. }) => {
        // Deliberately drops \`found\`, which is text from the message.
        eprintln!("{path}: expected {expected}");
    }
    Err(error) => eprintln!("{error}"),
    Ok(value) => { /* ... */ }
}`;

  const review = `cargo tree -p hl7-2                    # one dependency: er7, itself dependency-free
grep -rn "std::net\\|std::fs\\|std::env" hl7-2/src/   # only main.rs, the CLI
grep -rn "^log = \\|^tracing" --include=Cargo.toml .  # nothing`;
</script>

<DocPage
  lede="Every message these crates touch is a clinical record. Here is what they do with it, what they never do with it, and the one place a value can escape into somewhere you did not intend."
  {contents}
>
  <Callout type="warning" heading="This is a description of behavior, not legal advice">
    <p>
      Nothing here is a HIPAA, GDPR, or MDR attestation. A library is not a covered entity and does
      not have a compliance posture; the system you build around it does. What is offered instead is
      that every claim on this page is a property of the code you can verify yourself in an
      afternoon. The normative version is <a href={spec}>spec/phi/index.md</a>.
    </p>
  </Callout>

  <h2 id="short">The short version</h2>
  <p>
    A message goes in as text, stays in memory as text, and comes back out when you ask for it.
    Nothing is written to disk, sent over a network, logged, counted, or cached. The libraries do not
    open files, read environment variables, spawn processes, or open sockets — not conditionally, not
    on a feature flag, not at all.
  </p>
  <p>
    The one thing to know before you log an error: <strong>error and diagnostic messages can quote a
    value from the message.</strong> That is the section worth reading twice.
  </p>

  <h2 id="never">What the libraries never do</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Not done</th><th>How to check</th></tr>
      </thead>
      <tbody>
        {#each never as row (row.not)}
          <tr><td>{row.not}</td><td>{row.check}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    The whole runtime dependency surface of the workspace is <code>er7</code>, plus
    <code>chrono</code> in <a href="/crates/hl7-2-mllp/"><code>hl7-2-mllp</code></a> — optional, off
    by default, used only to stamp a generated acknowledgement with the wall clock. The macro crates
    run at compile time only. Criterion and the fuzzing harness are development dependencies and are
    never linked into anything you ship.
  </p>
  <p>
    The release dictionaries are compiled into the binary and parsed on first use, so no file is
    opened for them at run time. A dictionary you supply comes from a string or from bytes you read
    yourself — the crate never fetches one.
  </p>

  <h2 id="escape">Where a value can escape</h2>
  <p>
    This is the part that matters in practice, because the usual way PHI leaves a well-behaved system
    is a log line.
  </p>
  <p><strong>Errors that carry message content:</strong></p>
  <ul>
    <li>
      <code>Error::BadValue</code> — its <code>found</code> field is the offending text from the
      message, verbatim.
    </li>
    <li><code>Error::BadMshHeader</code> — the detail can quote part of the malformed header.</li>
    <li><code>Error::Invalid</code> — carries every error-severity diagnostic, with the caveat below.</li>
  </ul>
  <p>
    <strong>Diagnostics that carry message content:</strong> a <code>ValueFormat</code> finding
    formats the offending value into its <code>detail</code> string. Every other diagnostic kind
    reports a location and a description of the problem, not the content at that location.
  </p>
  <p>
    <code>Display</code> for both types reproduces those strings — so <code>to_string()</code>,
    <code>{'{'}:?{'}'}</code>, <code>println!</code>, <code>panic!</code>, <code>unwrap()</code>,
    <code>expect()</code>, and any logging call you make yourself will carry a value if the error is
    one of those kinds.
  </p>
  <p>
    <strong>A path is not a value, but it is not nothing.</strong> <code>OBX[200]-5</code> says
    nothing about a patient. The <code>path</code> on a diagnostic is safe to log in a way the
    <code>detail</code> is not.
  </p>
  <p>
    If your logs are less trusted than your message store — which is usual, because logs are shipped,
    aggregated, and retained differently — match on the error and log the parts you want:
  </p>
  <CodeSample language="rust" code={logging} />
  <p>
    The same applies to <a href="/guides/validating/"><code>validate</code></a>: filter on severity
    and kind, log the path, and treat the detail as message content.
  </p>

  <h2 id="not-defended">What is not defended against</h2>
  <p>Stated plainly, because a security review will ask and a vague answer is worse than a limitation.</p>
  <ul>
    <li>
      <strong>Memory is not zeroed.</strong> Strings holding message text are freed normally; the
      bytes are not overwritten first. A core dump, a swap file, or a heap inspection can contain
      message content.
    </li>
    <li><strong>No constant-time anything.</strong> Nothing here is a cryptographic operation.</li>
    <li>
      <strong>No access control.</strong> These are parsing libraries. Who may read which message is
      entirely your question.
    </li>
    <li>
      <strong>No encryption, at rest or in transit.</strong> MLLP is plaintext framing on whatever
      stream you give it. If that stream should be TLS, you supply the TLS stream.
    </li>
    <li>
      <strong>No de-identification.</strong> There is no scrub, redact, or anonymise function, and
      none is planned. Redaction is a policy decision about a particular data set, not a library
      default.
    </li>
    <li>
      <strong>No audit trail.</strong> Nothing records that a message was read. If your environment
      requires an access log, that is above this layer.
    </li>
  </ul>

  <h2 id="cli">The command-line tools</h2>
  <p>
    The binaries do what you point them at and nothing else: read a named file or standard input,
    write a named file or standard output, exit. No config file is searched for, no environment
    variable is consulted, no history is kept, no temporary file is written.
  </p>
  <p>
    Two ordinary shell hazards are worth naming anyway, because they are how PHI most often leaks
    from a command line and neither is something a program can prevent. <strong>Shell history</strong>:
    a message passed as an argument lands in your history file — pipe it or redirect from a file
    instead. <strong>Terminal scrollback</strong>: output persists in the buffer, and in whatever
    recording of the session exists.
  </p>

  <h2 id="transports">The transports</h2>
  <p>
    Both transport crates are deliberately narrower than they sound.
    <a href="/crates/hl7-2-mllp/"><code>hl7-2-mllp</code></a> implements framing over any byte
    stream; it does not open, bind, connect, or configure anything. MLLP by itself provides no
    confidentiality, integrity, or authentication — a property of the protocol as specified, not a
    shortcut taken here. <a href="/crates/hl7-2-soap/"><code>hl7-2-soap</code></a> builds and parses
    envelopes and contains no HTTP client or server.
  </p>
  <p>
    So the network posture of a system built on these crates is decided entirely by the code around
    them, which is where a reviewer should look.
  </p>

  <h2 id="ours">This project's own data</h2>
  <p>
    <strong>Every sample, test fixture, and benchmark input in the repository is synthetic.</strong>
    There is no real patient data anywhere in the history, and none may be added — not in an issue,
    a pull request, a test that reproduces a bug, or a benchmark corpus. Benchmark inputs are either
    generated in code or are the repository's own synthetic sample files, so there is no external
    corpus that could quietly acquire a real message.
  </p>
  <p>
    <strong>Reports must be redacted.</strong> Keep the structure, replace the values — structure is
    what reproduces a parsing bug; names, identifiers, dates of birth, and addresses are not.
  </p>
  <CodeSample language="er7" caption="Redacted, and still a usable bug report" code={redacted} />
  <p>
    This website sets no cookies, runs no analytics, and loads no third-party script. The only
    browser storage it uses is one key remembering a light or dark theme choice.
  </p>
  <p>
    If real patient data does reach an issue, a pull request, or a commit, say so at
    <a href="mailto:joel@joelparkerhenderson.com">joel@joelparkerhenderson.com</a> and it will be
    handled as an incident: removed, and the history rewritten if it landed in one.
  </p>

  <h2 id="review">If you are reviewing this for a deployment</h2>
  <CodeSample language="sh" caption="The three checks that answer most of the questionnaire" code={review} />
  <p>
    Then read the <code>Error</code> enum in <code>hl7-2</code>'s <code>src/lib.rs</code> and
    <code>Diagnostic</code> in <code>src/validate.rs</code> against
    <a href="#escape">Where a value can escape</a> above, and check the licensing: MIT, Apache-2.0,
    BSD-3-Clause, GPL-2.0-only, or GPL-3.0-only at your option, the same five in every crate, byte
    for byte.
  </p>
</DocPage>
