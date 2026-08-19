# HL7 v2 MLLP

The Minimal Lower Layer Protocol — how HL7 v2 messages actually cross a
network — as a Rust library.

A TCP stream is bytes without edges, and an HL7 v2 message carries no length
prefix and no self-delimiting syntax, so a receiver reading a socket cannot
tell where one message stops and the next begins. MLLP is the three-byte
answer to that, and nothing more:

```
<VT> message <FS><CR>
0x0B          0x1C 0x0D
```

That is the whole protocol. No length, no checksum, no session, no
negotiation, no encryption. What people actually need on top of it — whole
messages out of a chopped-up stream, an acknowledgement that names the
message it answers, and a way to bound what a broken peer can allocate — is
what this crate provides.

```
er7                      the ER7 encoding
  |
hl7-2                 HL7 v2 itself: releases 2.1-2.9, three parsing
                         modes, mutation, validation
  |
  +-- hl7-v2-mllp        this crate: getting those messages across a
                         network, and answering them
```

This README is a tour. [`spec/index.md`](spec/index.md) is the normative
specification of every rule — the single source of truth this crate
implements against.

## Framing

```rust
use hl7_v2_mllp as mllp;

let frame = mllp::encode(message.as_bytes());
assert_eq!(frame[0], mllp::START_BLOCK);
assert_eq!(mllp::decode(&frame)?, message.as_bytes());
```

The payload is copied verbatim — not trimmed, not validated, not
normalized. A message's own `\r` segment terminators are the same byte as
the frame's trailer, and survive untouched.

## Streaming

The one a socket needs. Frames arrive split across reads, several to a
read, or both, and `Framer` is the small amount of state that puts them
back together.

```rust
use hl7_v2_mllp::Framer;

let mut framer = Framer::new();
framer.push(b"\x0bMSH|one\x1c\r\x0bMSH|t");   // one and a half messages
framer.push(b"wo\x1c\r");                      // the other half

assert_eq!(framer.next_frame()?.unwrap(), b"MSH|one");
assert_eq!(framer.next_frame()?.unwrap(), b"MSH|two");
assert_eq!(framer.next_frame()?, None);        // nothing more yet
```

A partial frame is `Ok(None)`, not an error — it means "read more" — and a
frame may be split anywhere, including between `<FS>` and its `<CR>`.
Because MLLP has no length field, a `Framer` also caps what it will buffer
(16 MiB by default), so a peer that never sends an end block cannot grow
the process until it dies.

## Transport

```rust
use hl7_v2_mllp::{IoTransport, Transport};
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:2575")?;
for stream in listener.incoming() {
    let mut transport = IoTransport::new(stream?);
    while let Some(message) = transport.receive()? {
        // ... one whole HL7 message ...
    }
}
```

`IoTransport` works over anything that reads and writes bytes — a
`TcpStream`, a TLS stream, a Unix socket, a buffer in a test — and the
`Transport` trait is there for carriers it does not know about.

One distinction it insists on: a peer closing **between** frames is the end
of the stream (`Ok(None)`); a peer closing **mid-frame** is an error. The
message it was sending is lost, and handing back what arrived would mean
handing back a truncated clinical message.

## Acknowledgement

MLLP has no acknowledgement of its own. The reply HL7 expects is an HL7
message: an `ACK` whose `MSA-2` echoes the control ID of the message being
answered.

```rust
use hl7_v2_mllp::{AckCode, ack};

let frame = ack::acknowledge(&payload, AckCode::Accept, "ACK00001", "20260814080100")?;
transport.send(hl7_v2_mllp::decode(&frame)?)?;
```

That echo is the whole mechanism. MLLP guarantees a message arrived whole;
only the echoed control ID says *which* message arrived — so a sender that
does not compare it will eventually take one answer for another's.

When the receiver needs to look before it answers, which is the usual case,
build the acknowledgement from the parsed message and say why:

```rust
let message = ack::parse(&payload)?;
let mut nack = ack::acknowledge_message(&message, AckCode::Error, "N1", "20260814080100")?;
nack.set("MSA-3", "OBR-4 is required")?;
transport.send(nack.to_er7().as_bytes())?;
```

Every call takes the acknowledgement's own control ID and timestamp as
arguments, because a message that invents them is untestable and
untraceable. The `clock` feature adds `acknowledge_now` for callers who
genuinely just want the current time.

## Strictness

By default a frame must start with `<VT>`, end with `<FS><CR>`, and contain
neither block character in between. Real senders are not always strict, so
the `noncompliance` feature forgives the two common sins — a missing `<CR>`
after `<FS>`, and stray bytes between frames — and nothing else.

It is off by default because a receiver that quietly accepts malformed
framing is how a truncated message becomes a clinical record. Either
tolerance is always reachable by name, whatever the features say:

```rust
use hl7_v2_mllp::{Framer, Tolerance};

let framer = Framer::new().with_tolerance(Tolerance::Lenient);   // for that one sender
```

## Examples

Two programs that talk to each other:

```sh
cargo run --example tcp_listener     # accepts, reads, acknowledges
cargo run --example tcp_sender       # sends, waits, checks the echo
```

The listener is commented with what it shows and what a production listener
also needs — TLS, a read timeout, a connection bound, and persistence
before acknowledging.

## Features

| feature | default | effect |
|---|---|---|
| `ack` | on | acknowledgement generation; pulls in `hl7-2` |
| `clock` | off | `acknowledge_now`; pulls in `chrono`. Implies `ack` |
| `noncompliance` | off | the default tolerance becomes lenient |

`--no-default-features` gives framing, streaming, and transport with **no
dependencies at all**.

## What this crate does not do

MLLP is a small protocol and this is a small crate. It has no TLS (compose
it — `IoTransport` takes any stream), no async runtime, no connection
pooling, no retry or reconnect policy, and no opinion on HL7 v2 semantics.
Sending `AA` promises the message is safe; making that true before you send
it is your application's job, and no library can do it for you.

## Install

```sh
cargo add hl7-v2-mllp
cargo add hl7-v2-mllp --no-default-features    # framing only, zero dependencies
```

## See also

- [`spec/index.md`](spec/index.md) — the normative specification
- [`hl7-2`](https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2) — HL7 v2 itself
- [`er7`](https://github.com/hl7-rust/er7) — the ER7 encoding layer

## License

MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only
