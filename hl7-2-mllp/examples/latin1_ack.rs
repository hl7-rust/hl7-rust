//! Exchange a synthetic ISO-8859-1 message and acknowledgement over MLLP.
//!
//! Run with: cargo run -p hl7-2-mllp --example latin1_ack

// RCA-EXC-5: test code may use plain operators, `unwrap` and `expect`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::integer_division,
    clippy::default_numeric_fallback,
    reason = "RCA-EXC-5: test code may use plain operators, unwrap and expect"
)]

use hl7_2_mllp::{AckCode, ack, decode, encode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This interface has agreed on ISO-8859-1 in both directions.
    let payload = b"MSH|^~\\&|Caf\xe9|LAB|EHR|CLINIC|20260814||ORU^R01|MSG42|P|2.5||||||8859/1";
    let frame = encode(payload);
    let received = decode(&frame)?;
    assert_eq!(received, payload);
    assert!(matches!(
        ack::acknowledge(received, AckCode::Accept, "ACK1", "20260814"),
        Err(ack::Error::NotText)
    ));

    // Each Latin-1 byte maps to the Unicode code point with the same value.
    let text: String = received.iter().copied().map(char::from).collect();
    let message = hl7_2::parse(&text)?;
    assert_eq!(message.get("MSH-18")?.as_deref(), Some("8859/1"));
    let mut reply = ack::acknowledge_message(&message, AckCode::Accept, "ACK1", "20260814")?;
    reply.set("MSH-18", "8859/1")?;

    // Fail if any reply character falls outside Latin-1.
    let bytes = reply
        .to_er7()
        .chars()
        .map(u8::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let reply_frame = encode(&bytes);
    assert_eq!(
        reply_frame,
        b"\x0bMSH|^~\\&|EHR|CLINIC|Caf\xe9|LAB|20260814||ACK^R01^ACK|ACK1|P|2.5||||||8859/1\rMSA|AA|MSG42\x1c\r",
    );
    Ok(())
}

#[test]
fn a_latin1_exchange_matches_the_declared_encoding() -> Result<(), Box<dyn std::error::Error>> {
    main()
}
