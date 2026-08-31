<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'protocol', label: 'The whole protocol' },
    { id: 'framing', label: 'Framing one message' },
    { id: 'streaming', label: 'Streaming: the Framer' },
    { id: 'transport', label: 'Transport' },
    { id: 'acknowledgement', label: 'Acknowledgement' },
    { id: 'strictness', label: 'Strictness and tolerance' },
    { id: 'features', label: 'Features' },
    { id: 'not-doing', label: 'What this crate does not do' },
    { id: 'production', label: 'What a production listener also needs' }
  ];

  const protocol = `<VT> message <FS><CR>
0x0B          0x1C 0x0D`;

  const framing = `use hl7_2_mllp as mllp;

let frame = mllp::encode(message.as_bytes());
assert_eq!(frame[0], mllp::START_BLOCK);
assert_eq!(mllp::decode(&frame)?, message.as_bytes());`;

  const streaming = `use hl7_2_mllp::Framer;

let mut framer = Framer::new();
framer.push(b"\\x0bMSH|one\\x1c\\r\\x0bMSH|t");   // one and a half messages
framer.push(b"wo\\x1c\\r");                       // the other half

assert_eq!(framer.next_frame()?.unwrap(), b"MSH|one");
assert_eq!(framer.next_frame()?.unwrap(), b"MSH|two");
assert_eq!(framer.next_frame()?, None);          // nothing more yet`;

  const limit = `// The default cap on what a single frame may buffer is 16 MiB.
let framer = Framer::new().with_limit(4 * 1024 * 1024);
assert_eq!(hl7_2_mllp::DEFAULT_LIMIT, 16 * 1024 * 1024);`;

  const transport = `use hl7_2_mllp::{IoTransport, Transport};
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:2575")?;
for stream in listener.incoming() {
    let mut transport = IoTransport::new(stream?);
    while let Some(message) = transport.receive()? {
        // ... one whole HL7 message ...
    }
}`;

  const ack = `use hl7_2_mllp::{AckCode, ack};

let frame = ack::acknowledge(&payload, AckCode::Accept, "ACK00001", "20260814080100")?;
transport.send(hl7_2_mllp::decode(&frame)?)?;`;

  const nack = `let message = ack::parse(&payload)?;
let mut nack = ack::acknowledge_message(&message, AckCode::Error, "N1", "20260814080100")?;
nack.set("MSA-3", "OBR-4 is required")?;
transport.send(nack.to_er7().as_bytes())?;`;

  const tolerance = `use hl7_2_mllp::{Framer, Tolerance};

// For that one sender whose stack forgets the trailing <CR>.
let framer = Framer::new().with_tolerance(Tolerance::Lenient);

// The default, whatever the crate features say.
let framer = Framer::new().with_tolerance(Tolerance::Strict);`;

  const examples = `cargo run --example tcp_listener     # accepts, reads, acknowledges
cargo run --example tcp_sender       # sends, waits, checks the echo`;
</script>

<DocPage
  lede="The Minimal Lower Layer Protocol is three bytes of framing and nothing else. This guide covers what you actually need on top of it: whole messages out of a chopped-up stream, an acknowledgement that names what it answers, and a bound on what a broken peer can allocate."
  {contents}
>
  <h2 id="protocol">The whole protocol</h2>
  <p>
    A TCP stream is bytes without edges, and an HL7 v2 message carries no length prefix and no
    self-delimiting syntax — so a receiver reading a socket cannot tell where one message stops and
    the next begins. MLLP is the answer, and this is all of it:
  </p>
  <CodeSample language="text" code={protocol} label="An MLLP frame" />
  <p>
    No length, no checksum, no session, no negotiation, no encryption. Nothing else in the
    specification.
  </p>

  <h2 id="framing">Framing one message</h2>
  <CodeSample language="rust" code={framing} />
  <p>
    The payload is copied verbatim — not trimmed, not validated, not normalized. Note that a
    message's own <code>\r</code> segment terminators are the same byte as the frame's trailer, and
    they survive untouched.
  </p>

  <h2 id="streaming">Streaming: the Framer</h2>
  <p>
    This is the one a socket actually needs. Frames arrive split across reads, several to a read, or
    both, and <code>Framer</code> is the small amount of state that puts them back together.
  </p>
  <CodeSample language="rust" code={streaming} />
  <p>
    A partial frame is <code>Ok(None)</code>, not an error — it means “read more” — and a frame may
    be split anywhere, including between <code>&lt;FS&gt;</code> and its <code>&lt;CR&gt;</code>.
  </p>
  <Callout heading="The buffer cap is a denial-of-service bound, not a tuning knob">
    <p>
      Because MLLP has no length field, a receiver cannot know how much more is coming. A
      <code>Framer</code> therefore caps what it will buffer — 16 MiB by default — so a peer that
      never sends an end block cannot grow the process until it dies.
    </p>
  </Callout>
  <CodeSample language="rust" code={limit} />

  <h2 id="transport">Transport</h2>
  <CodeSample language="rust" code={transport} />
  <p>
    <code>IoTransport</code> works over anything that reads and writes bytes — a
    <code>TcpStream</code>, a TLS stream, a Unix socket, a <code>Cursor</code> in a test — and the
    <code>Transport</code> trait is there for carriers it does not know about.
    <code>send_str</code> is the convenience for messages you hold as text; send bytes when the
    message is already encoded in the character set <code>MSH-18</code> names.
  </p>
  <Callout type="warning" heading="Closing mid-frame is an error; closing between frames is not">
    <p>
      A peer closing <strong>between</strong> frames is the end of the stream —
      <code>Ok(None)</code>. A peer closing <strong>mid-frame</strong> is an error. The message it
      was sending is lost, and handing back what arrived would mean handing back a truncated
      clinical message. A framing violation surfaces as an <code>io::Error</code> of kind
      <code>InvalidData</code>, because a stream that has lost framing cannot be resynchronized —
      the right response is to close the connection.
    </p>
  </Callout>

  <h2 id="acknowledgement">Acknowledgement</h2>
  <p>
    MLLP has no acknowledgement of its own. The reply HL7 expects is an HL7 message: an
    <code>ACK</code> whose <code>MSA-2</code> echoes the control ID of the message being answered.
  </p>
  <CodeSample language="rust" code={ack} />
  <p>
    That echo is the whole mechanism. MLLP guarantees a message arrived whole; only the echoed
    control ID says <em>which</em> message arrived — so a sender that does not compare it will
    eventually take one answer for another's.
  </p>
  <p>When the receiver needs to look before it answers, which is the usual case:</p>
  <CodeSample language="rust" code={nack} />
  <p>
    Every call takes the acknowledgement's own control ID and timestamp as arguments, because a
    message that invents them is untestable and untraceable. The <code>clock</code> feature adds
    <code>acknowledge_now</code> for callers who genuinely just want the current time.
  </p>
  <Callout type="warning" heading="AA is a promise, and it is your promise">
    <p>
      Sending <code>AckCode::Accept</code> tells the sender the message is safe — in practice, that
      it has been persisted and will not be lost. Acknowledging before you have written it down
      turns a restart into silent data loss. No library can make that true for you.
    </p>
  </Callout>

  <h2 id="strictness">Strictness and tolerance</h2>
  <p>
    By default a frame must start with <code>&lt;VT&gt;</code>, end with
    <code>&lt;FS&gt;&lt;CR&gt;</code>, and contain neither block character in between. Real senders
    are not always strict.
  </p>
  <CodeSample language="rust" code={tolerance} />
  <p>
    <code>Tolerance::Lenient</code> forgives the two common sins — a missing
    <code>&lt;CR&gt;</code> after <code>&lt;FS&gt;</code>, and stray bytes between frames — and
    nothing else.
  </p>
  <p>
    Prefer setting it per connection, for the one sender that needs it, over enabling the
    crate-wide <code>noncompliance</code> feature. A receiver that quietly accepts malformed framing
    is how a truncated message becomes a clinical record, and loosening every connection at once to
    accommodate one peer is not a trade worth making.
  </p>

  <h2 id="features">Features</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Feature</th><th>Default</th><th>Effect</th></tr>
      </thead>
      <tbody>
        <tr><td><code>ack</code></td><td>on</td><td>Acknowledgement generation; pulls in <code>hl7-2</code>.</td></tr>
        <tr>
          <td><code>clock</code></td>
          <td>off</td>
          <td><code>acknowledge_now</code>; pulls in <code>chrono</code>. Implies <code>ack</code>.</td>
        </tr>
        <tr>
          <td><code>noncompliance</code></td>
          <td>off</td>
          <td>The default tolerance becomes lenient.</td>
        </tr>
      </tbody>
    </table>
  </div>
  <p>
    <code>--no-default-features</code> gives framing, streaming, and transport with
    <strong>no dependencies at all</strong> — useful when the process that terminates the socket is
    not the process that understands the message.
  </p>

  <h2 id="not-doing">What this crate does not do</h2>
  <p>MLLP is a small protocol and this is a small crate. It has:</p>
  <ul>
    <li>No TLS. Compose it — <code>IoTransport</code> takes any stream.</li>
    <li>No async runtime.</li>
    <li>No connection pooling, and no retry or reconnect policy.</li>
    <li>No opinion on HL7 v2 semantics — that is <a href="/crates/hl7-2/"><code>hl7-2</code></a>.</li>
  </ul>

  <h2 id="production">What a production listener also needs</h2>
  <p>
    The crate ships two runnable examples that talk to each other, and the listener is commented
    with exactly this list:
  </p>
  <CodeSample language="sh" code={examples} />
  <ul>
    <li><strong>TLS</strong>, unless the link is genuinely private end to end.</li>
    <li><strong>A read timeout</strong>, so a half-open connection does not hold a thread forever.</li>
    <li><strong>A connection bound</strong>, so a burst of connections does not exhaust the process.</li>
    <li>
      <strong>Persistence before acknowledging</strong> — see the warning above. This is the one
      that costs data when it is missed.
    </li>
  </ul>
  <p>
    For a step-by-step build of a listener that does all four, see
    <a href="/tutorials/mllp-listener/">An MLLP listener that answers</a>.
  </p>
</DocPage>
