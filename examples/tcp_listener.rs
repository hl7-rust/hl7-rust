//! A working MLLP listener: accept connections, read messages, answer each
//! one, keep the connection open.
//!
//! ```sh
//! cargo run --example tcp_listener              # listens on 127.0.0.1:2575
//! cargo run --example tcp_listener 0.0.0.0:2575
//! ```
//!
//! Then, from another terminal:
//!
//! ```sh
//! cargo run --example tcp_sender
//! ```
//!
//! What this shows, and what a real listener also needs, are different
//! lists. This one is honest about the difference:
//!
//! - **Shown**: framing, one connection per thread, an acknowledgement per
//!   message, answering with `AE` and a reason rather than dropping a
//!   message that will not parse, and keeping the connection open for the
//!   next message — which is what senders expect and what makes MLLP
//!   cheap.
//! - **Not shown, and needed in production**: TLS (MLLP has no encryption
//!   and HL7 messages are patient data), a read timeout so a silent peer
//!   cannot hold a thread forever, a bound on concurrent connections,
//!   persistence before acknowledging (an `AA` promises the message is
//!   safe), and a real logger.

use hl7_v2_mllp::{AckCode, IoTransport, Transport, ack};
use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};

/// Control IDs for the acknowledgements this process sends. A real one
/// would draw from something that survives a restart, because a control ID
/// is what an operator greps for when a sender asks what happened.
static SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn main() -> io::Result<()> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:2575".to_string());
    let listener = TcpListener::bind(&address)?;
    println!("listening for MLLP on {address}");

    for stream in listener.incoming() {
        let stream = stream?;
        // One thread per connection: MLLP connections are long-lived and
        // few, so this is the right shape here in a way it would not be for
        // a web server.
        std::thread::spawn(move || {
            let peer = stream
                .peer_addr()
                .map(|address| address.to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            if let Err(error) = serve(stream) {
                eprintln!("{peer}: {error}");
            } else {
                println!("{peer}: closed");
            }
        });
    }
    Ok(())
}

/// Read messages until the peer hangs up, answering each one.
fn serve(stream: TcpStream) -> io::Result<()> {
    let peer = stream.peer_addr()?;
    println!("{peer}: connected");
    let mut transport = IoTransport::new(stream);

    while let Some(payload) = transport.receive()? {
        let control_id = format!("ACK{:06}", SEQUENCE.fetch_add(1, Ordering::Relaxed));
        let timestamp = timestamp();

        // Decide what to say, then say it. A message that will not parse
        // still gets an answer — silence leaves the sender retrying
        // forever, which is worse for everyone than a clear rejection.
        let reply = match ack::parse(&payload) {
            Ok(message) => {
                println!(
                    "{peer}: {} {} ({} segments)",
                    message.structure_id(),
                    message.get("MSH-10").ok().flatten().unwrap_or_default(),
                    message.segments().count()
                );
                // An `AA` says "this is safely mine now", so in a real
                // listener everything that makes that true — writing to a
                // queue, a database, a file — happens here, before the
                // acknowledgement is sent.
                ack::acknowledge_message(&message, AckCode::Accept, &control_id, &timestamp)
                    .map(|ack| hl7_v2_mllp::encode(ack.to_er7().as_bytes()))
                    .map_err(|error| error.to_string())
            }
            Err(error) => {
                eprintln!("{peer}: unreadable message: {error}");
                Err(error.to_string())
            }
        };

        match reply {
            Ok(frame) => transport.send(&frame_payload(&frame))?,
            // Nothing to echo a control ID from, so this is as much as can
            // honestly be said.
            Err(reason) => transport.send_str(&nack(&control_id, &timestamp, &reason))?,
        }
    }
    Ok(())
}

/// The acknowledgement built above is already framed; `Transport::send`
/// frames what it is given, so hand it the payload.
fn frame_payload(frame: &[u8]) -> Vec<u8> {
    hl7_v2_mllp::decode(frame)
        .map(<[u8]>::to_vec)
        .unwrap_or_else(|_| frame.to_vec())
}

/// The last-resort answer, for a payload that is not a message at all: an
/// `AE` with no control ID to echo, because there was none to read.
fn nack(control_id: &str, timestamp: &str, reason: &str) -> String {
    format!(
        "MSH|^~\\&|||||{timestamp}||ACK|{control_id}|P|2.5\r\
         MSA|AE||{}",
        reason.replace(['|', '^', '~', '\\', '&'], " ")
    )
}

/// `YYYYMMDDHHMMSS` without pulling in a date library: this example keeps
/// the crate's default features, where the clock is opt-in. Enable the
/// `clock` feature and call `ack::now()` instead.
fn timestamp() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    // Civil-from-days, so the example stays dependency-free.
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    let (hour, minute, second) = (rest / 3600, (rest % 3600) / 60, rest % 60);
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    let year = year + i64::from(month <= 2);
    format!("{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}")
}
