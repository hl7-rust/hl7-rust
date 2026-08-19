//! The other end: connect, send a message, wait for the acknowledgement,
//! and check that it is the acknowledgement for *this* message.
//!
//! ```sh
//! cargo run --example tcp_listener      # in one terminal
//! cargo run --example tcp_sender        # in another
//! ```
//!
//! The check at the end is the point of the whole exercise. MLLP guarantees
//! that a message arrived whole; only the control ID echoed in MSA-2 says
//! that *this* message is the one that arrived, and a sender that does not
//! compare it is a sender that will one day treat someone else's
//! acknowledgement as its own.

use hl7_2_mllp::{IoTransport, Transport, ack};
use std::io;
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:2575".to_string());

    let control_id = "MSG00042";
    let message = format!(
        "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|{control_id}|P|2.5\r\
         PID|1||444333222^^^ACME^MR||EVERYWOMAN^EVE^E||19620320|F\r\
         OBR|1|ORD776655|LAB2233|24331-1^Lipid Panel^LN\r\
         OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F"
    );

    let mut transport = IoTransport::new(TcpStream::connect(&address)?);
    println!("connected to {address}");

    transport.send_str(&message)?;
    println!("sent {control_id}");

    // A real sender needs a read timeout here: without one, a receiver that
    // accepts the connection and then says nothing blocks this thread for
    // as long as the socket stays open.
    let Some(reply) = transport.receive()? else {
        eprintln!("the receiver closed the connection without acknowledging");
        std::process::exit(1);
    };

    let acknowledgement = match ack::parse(&reply) {
        Ok(message) => message,
        Err(error) => {
            eprintln!("the reply is not an HL7 message: {error}");
            std::process::exit(1);
        }
    };

    let code = acknowledgement
        .get("MSA-1")
        .ok()
        .flatten()
        .unwrap_or_default();
    let echoed = acknowledgement
        .get("MSA-2")
        .ok()
        .flatten()
        .unwrap_or_default();

    println!("received {code} for {echoed}");

    if echoed != control_id {
        eprintln!("this acknowledgement answers {echoed:?}, not {control_id:?}");
        std::process::exit(1);
    }
    if code != "AA" && code != "CA" {
        eprintln!("the receiver did not accept the message: {code}");
        if let Ok(Some(reason)) = acknowledgement.get("MSA-3") {
            eprintln!("  {reason}");
        }
        std::process::exit(1);
    }
    println!("accepted");
    Ok(())
}
