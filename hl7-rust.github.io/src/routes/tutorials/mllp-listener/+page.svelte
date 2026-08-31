<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'goal', label: 'What you will build' },
    { id: 'step-1', label: '1. Accept a connection' },
    { id: 'step-2', label: '2. Answer every message' },
    { id: 'step-3', label: '3. Answer the unreadable ones too' },
    { id: 'step-4', label: '4. Persist before you promise' },
    { id: 'step-5', label: '5. Bound what a peer can cost you' },
    { id: 'run-it', label: 'Running it' },
    { id: 'production', label: 'The production checklist' }
  ];

  const setup = `cargo new mllp-listener
cd mllp-listener
cargo add hl7-2-mllp`;

  const accept = `use hl7_2_mllp::{IoTransport, Transport};
use std::io;
use std::net::{TcpListener, TcpStream};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2575")?;
    println!("listening for MLLP on 127.0.0.1:2575");

    for stream in listener.incoming() {
        let stream = stream?;
        // One thread per connection. MLLP connections are long-lived and
        // few, so this is the right shape here in a way it would not be
        // for a web server.
        std::thread::spawn(move || {
            if let Err(error) = serve(stream) {
                eprintln!("connection failed: {error}");
            }
        });
    }
    Ok(())
}

fn serve(stream: TcpStream) -> io::Result<()> {
    let mut transport = IoTransport::new(stream);
    while let Some(payload) = transport.receive()? {
        println!("{} bytes", payload.len());
    }
    Ok(())
}`;

  const answer = `use hl7_2_mllp::{AckCode, IoTransport, Transport, ack};
use std::sync::atomic::{AtomicU64, Ordering};

/// Control IDs for the acknowledgements this process sends. A real one
/// would draw from something that survives a restart, because a control ID
/// is what an operator greps for when a sender asks what happened.
static SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn serve(stream: TcpStream) -> io::Result<()> {
    let mut transport = IoTransport::new(stream);

    while let Some(payload) = transport.receive()? {
        let control_id = format!("ACK{:06}", SEQUENCE.fetch_add(1, Ordering::Relaxed));
        let timestamp = timestamp();

        let message = ack::parse(&payload)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        println!("{} {}", message.structure_id(), message.get("MSH-10").ok().flatten().unwrap_or_default());

        let reply = ack::acknowledge_message(&message, AckCode::Accept, &control_id, &timestamp)
            .expect("building an ACK from a parsed message");
        transport.send(reply.to_er7().as_bytes())?;
    }
    Ok(())
}`;

  const unreadable = `let reply = match ack::parse(&payload) {
    Ok(message) => {
        // ...persist it here, before promising it is safe...
        ack::acknowledge_message(&message, AckCode::Accept, &control_id, &timestamp)
            .map(|ack| ack.to_er7())
            .map_err(|error| error.to_string())
    }
    Err(error) => Err(error.to_string()),
};

match reply {
    Ok(text) => transport.send_str(&text)?,
    // Nothing to echo a control ID from, so this is as much as can
    // honestly be said.
    Err(reason) => transport.send_str(&nack(&control_id, &timestamp, &reason))?,
}`;

  const nack = `/// The last-resort answer, for a payload that is not a message at all: an
/// AE with no control ID to echo, because there was none to read.
fn nack(control_id: &str, timestamp: &str, reason: &str) -> String {
    format!(
        "MSH|^~\\\\&|||||{timestamp}||ACK|{control_id}|P|2.5\\r\\
         MSA|AE||{}",
        reason.replace(['|', '^', '~', '\\\\', '&'], " ")
    )
}`;

  const persist = `Ok(message) => {
    // An AA says "this is safely mine now". Everything that makes that
    // true — writing to a queue, a database, a file, and flushing it —
    // happens HERE, before the acknowledgement is sent.
    match store.append(&payload) {
        Ok(()) => ack::acknowledge_message(&message, AckCode::Accept, &control_id, &timestamp)
            .map(|ack| ack.to_er7())
            .map_err(|error| error.to_string()),
        // AR, not AE: nothing is wrong with the message. We are.
        Err(error) => {
            let mut reject =
                ack::acknowledge_message(&message, AckCode::Reject, &control_id, &timestamp)
                    .map_err(|e| e.to_string())?;
            reject.set("MSA-3", "receiver could not store the message")
                .map_err(|e| e.to_string())?;
            eprintln!("store failed: {error}");
            Ok(reject.to_er7())
        }
    }
}`;

  const bounds = `use hl7_2_mllp::Framer;
use std::time::Duration;

// A silent peer must not hold a thread forever.
stream.set_read_timeout(Some(Duration::from_secs(120)))?;

// MLLP has no length field, so cap what one frame may buffer. The default
// is 16 MiB; lower it to what your largest real message plus headroom is.
let framer = Framer::new().with_limit(4 * 1024 * 1024);
let mut transport = IoTransport::with_framer(stream, framer);`;

  const run = `# Terminal 1
cargo run

# Terminal 2 — the crate ships a sender that checks the echoed control ID
cd path/to/hl7-rust/hl7-2-mllp
cargo run --example tcp_sender

# Or read both examples end to end
cargo run --example tcp_listener`;
</script>

<DocPage
  eyebrow="Tutorial"
  lede="Accept a TCP connection, reassemble frames into whole messages, and send back an acknowledgement that names the message it answers — plus the four things a production listener needs that a demo does not."
  {contents}
>
  <h2 id="goal">What you will build</h2>
  <p>
    A listener that behaves the way senders expect: one connection carrying many messages, an answer
    for every one of them — including the ones that will not parse — and a promise it does not make
    until it can keep it.
  </p>
  <CodeSample language="sh" code={setup} />
  <p>
    <code>hl7-2-mllp</code>'s <code>ack</code> feature is on by default, which is what pulls in
    <code>hl7-2</code> for building acknowledgements. If you only need framing, see
    <a href="/guides/mllp/#features">the features table</a>.
  </p>

  <h2 id="step-1">1. Accept a connection</h2>
  <CodeSample language="rust" caption="src/main.rs" code={accept} />
  <p>
    <code>receive</code> returns whole messages. Frames arriving split across reads, or several to a
    read, are reassembled by the <code>Framer</code> inside <code>IoTransport</code> — you never see
    a partial one.
  </p>
  <Callout heading="None means the peer hung up cleanly">
    <p>
      A peer closing <strong>between</strong> frames ends the loop with <code>Ok(None)</code>. A
      peer closing <strong>mid-frame</strong> is an <code>io::Error</code>, because the message it
      was sending is lost and handing back what arrived would mean handing back a truncated clinical
      message. Both are handled correctly by the loop above.
    </p>
  </Callout>

  <h2 id="step-2">2. Answer every message</h2>
  <CodeSample language="rust" code={answer} />
  <p>
    <code>MSA-2</code> in that reply echoes the incoming <code>MSH-10</code>. That echo is the whole
    mechanism by which a sender knows which message you answered — MLLP guarantees a message arrived
    whole, and nothing else says <em>which</em>.
  </p>
  <p>
    The control ID and timestamp are arguments rather than generated inside the library, because a
    message that invents them is untestable and untraceable. (The <code>clock</code> feature adds
    <code>ack::now()</code> if you genuinely just want the current time.)
  </p>
  <p>Note also that the loop keeps going. A connection carries many messages; closing after one is a
    common bug and senders notice it immediately.</p>

  <h2 id="step-3">3. Answer the unreadable ones too</h2>
  <p>
    <code>ack::parse</code> can fail — the payload might not be an HL7 message at all. Returning
    early from <code>serve</code> at that point drops the connection and leaves the sender retrying
    forever, which is worse for everyone than a clear rejection.
  </p>
  <CodeSample language="rust" code={unreadable} />
  <CodeSample language="rust" code={nack} />
  <p>
    That fallback is deliberately minimal and deliberately honest: there is no control ID to echo,
    because there was none to read. Note the <code>replace</code> — the reason text goes into a
    field, so any delimiter in it has to go.
  </p>

  <h2 id="step-4">4. Persist before you promise</h2>
  <Callout type="warning" heading="This is the step that costs data when it is skipped">
    <p>
      <code>AA</code> tells the sender the message is safe and they may forget it. If your process
      restarts between sending <code>AA</code> and writing the message down, that message is gone and
      nobody knows. No library can make the promise true for you.
    </p>
  </Callout>
  <CodeSample language="rust" code={persist} />
  <p>
    Note the code when storage fails: <code>AR</code>, not <code>AE</code>. <code>AE</code> means
    “there is an error in your message; fix it and resend”, which is false and will send the sender
    hunting for a bug they do not have. <code>AR</code> means “rejected for reasons unrelated to
    your message's content” — the receiver is shutting down, unconfigured, or, as here, unable to
    store it. Say the true thing; the sender's retry logic depends on it.
  </p>

  <h2 id="step-5">5. Bound what a peer can cost you</h2>
  <CodeSample language="rust" code={bounds} />
  <p>
    Both bounds matter for the same reason: MLLP has no length field and no session, so a receiver
    cannot tell a slow sender from a stuck one, or a large message from an endless one. Neither
    limit is a tuning knob — they are the difference between a bad peer degrading one connection and
    a bad peer taking down the process.
  </p>
  <p>Add a bound on concurrent connections too, if the listener is reachable from anywhere you do
    not control.</p>

  <h2 id="run-it">Running it</h2>
  <CodeSample language="sh" code={run} />
  <p>
    The crate ships two runnable examples that talk to each other, and its listener is commented with
    exactly the distinction this tutorial draws — what is shown, and what a real listener also
    needs. Reading them alongside your own is worth the ten minutes.
  </p>

  <h2 id="production">The production checklist</h2>
  <ul>
    <li>
      <strong>TLS.</strong> MLLP has no encryption and HL7 messages are patient data. Compose it —
      <code>IoTransport</code> takes any stream, so a TLS stream drops straight in.
    </li>
    <li><strong>A read timeout</strong>, so a silent peer cannot hold a thread forever.</li>
    <li><strong>A bound on concurrent connections.</strong></li>
    <li><strong>Persistence before acknowledging</strong> — step 4.</li>
    <li>
      <strong>Control IDs that survive a restart.</strong> An in-memory counter restarts at 1, and a
      control ID is what an operator greps for when a sender asks what happened.
    </li>
    <li><strong>A real logger</strong>, rather than <code>println!</code>.</li>
    <li>
      <strong>Strict framing.</strong> Leave <code>Tolerance::Strict</code> on, and set
      <code>Lenient</code> per connection for the one sender that needs it. See
      <a href="/guides/mllp/#strictness">Strictness</a>.
    </li>
  </ul>
</DocPage>
