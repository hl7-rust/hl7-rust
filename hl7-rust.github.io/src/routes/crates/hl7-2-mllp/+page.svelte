<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-mllp');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'framing', label: 'Framing' },
    { id: 'streaming', label: 'Streaming' },
    { id: 'transport', label: 'Transport' },
    { id: 'ack', label: 'Acknowledgement' },
    { id: 'strictness', label: 'Strictness' },
    { id: 'features', label: 'Features' },
    { id: 'examples', label: 'Examples' },
    { id: 'not-doing', label: 'What this crate does not do' },
    { id: 'related', label: 'Related crates' }
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

let framer = Framer::new().with_tolerance(Tolerance::Lenient);   // for that one sender`;

  const examples = `cargo run --example tcp_listener     # accepts, reads, acknowledges
cargo run --example tcp_sender       # sends, waits, checks the echo`;

  const install = `cargo add hl7-2-mllp
cargo add hl7-2-mllp --no-default-features    # framing only, zero dependencies`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    A TCP stream is bytes without edges, and an HL7 v2 message carries no length prefix and no
    self-delimiting syntax — so a receiver reading a socket cannot tell where one message stops and
    the next begins. MLLP is the three-byte answer to that, and nothing more:
  </p>
  <CodeSample language="text" code={protocol} />
  <p>
    That is the whole protocol. No length, no checksum, no session, no negotiation, no encryption.
    What people actually need on top of it — whole messages out of a chopped-up stream, an
    acknowledgement that names the message it answers, and a way to bound what a broken peer can
    allocate — is what this crate provides.
  </p>
  <CodeSample language="sh" code={install} />

  <h2 id="framing">Framing</h2>
  <CodeSample language="rust" code={framing} />
  <p>
    The payload is copied verbatim — not trimmed, not validated, not normalized. A message's own
    <code>\r</code> segment terminators are the same byte as the frame's trailer, and survive
    untouched.
  </p>

  <h2 id="streaming">Streaming</h2>
  <p>
    The one a socket needs. Frames arrive split across reads, several to a read, or both, and
    <code>Framer</code> is the small amount of state that puts them back together.
  </p>
  <CodeSample language="rust" code={streaming} />
  <p>
    A partial frame is <code>Ok(None)</code>, not an error — it means “read more” — and a frame may
    be split anywhere, including between <code>&lt;FS&gt;</code> and its <code>&lt;CR&gt;</code>.
    Because MLLP has no length field, a <code>Framer</code> also caps what it will buffer (16 MiB by
    default), so a peer that never sends an end block cannot grow the process until it dies.
  </p>

  <h2 id="transport">Transport</h2>
  <CodeSample language="rust" code={transport} />
  <p>
    <code>IoTransport</code> works over anything that reads and writes bytes — a
    <code>TcpStream</code>, a TLS stream, a Unix socket, a buffer in a test — and the
    <code>Transport</code> trait is there for carriers it does not know about.
  </p>
  <Callout type="warning" heading="One distinction it insists on">
    <p>
      A peer closing <strong>between</strong> frames is the end of the stream
      (<code>Ok(None)</code>); a peer closing <strong>mid-frame</strong> is an error. The message it
      was sending is lost, and handing back what arrived would mean handing back a truncated clinical
      message.
    </p>
  </Callout>

  <h2 id="ack">Acknowledgement</h2>
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

  <h2 id="strictness">Strictness</h2>
  <p>
    By default a frame must start with <code>&lt;VT&gt;</code>, end with
    <code>&lt;FS&gt;&lt;CR&gt;</code>, and contain neither block character in between. Real senders
    are not always strict, so the <code>noncompliance</code> feature forgives the two common sins —
    a missing <code>&lt;CR&gt;</code> after <code>&lt;FS&gt;</code>, and stray bytes between frames
    — and nothing else.
  </p>
  <p>
    It is off by default because a receiver that quietly accepts malformed framing is how a truncated
    message becomes a clinical record. Either tolerance is always reachable by name, whatever the
    features say:
  </p>
  <CodeSample language="rust" code={tolerance} />

  <h2 id="features">Features</h2>
  <p>
    See the table above. <code>--no-default-features</code> gives framing, streaming, and transport
    with <strong>no dependencies at all</strong>.
  </p>

  <h2 id="examples">Examples</h2>
  <p>Two programs that talk to each other:</p>
  <CodeSample language="sh" code={examples} />
  <p>
    The listener is commented with what it shows and what a production listener also needs — TLS, a
    read timeout, a connection bound, and persistence before acknowledging. Building one step by
    step: <a href="/tutorials/mllp-listener/">An MLLP listener that answers</a>.
  </p>

  <h2 id="not-doing">What this crate does not do</h2>
  <p>
    MLLP is a small protocol and this is a small crate. It has no TLS (compose it —
    <code>IoTransport</code> takes any stream), no async runtime, no connection pooling, no retry or
    reconnect policy, and no opinion on HL7 v2 semantics.
  </p>
  <p>
    Sending <code>AA</code> promises the message is safe; making that true before you send it is your
    application's job, and no library can do it for you.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
