//! Black-box tests through the public API, including a real TCP connection.
//!
//! The unit tests next to each module cover framing rules against byte
//! slices; these cover what only shows up when the pieces are assembled —
//! a socket that splits frames wherever it likes, a conversation with
//! acknowledgements, and the round trip from message to wire and back.

#[cfg(feature = "ack")]
use hl7_2_mllp::{AckCode, ack};
use hl7_2_mllp::{Framer, IoTransport, Tolerance, Transport};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const MESSAGE: &str = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5\r\
                       PID|1||444333222^^^ACME^MR||EVERYWOMAN^EVE^E||19620320|F\r\
                       OBR|1|ORD776655|LAB2233|24331-1^Lipid Panel^LN\r\
                       OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F";

#[test]
#[cfg(feature = "ack")]
fn a_message_survives_the_round_trip_to_the_wire() {
    let frame = hl7_2_mllp::encode(MESSAGE.as_bytes());
    let payload = hl7_2_mllp::decode(&frame).unwrap();
    assert_eq!(payload, MESSAGE.as_bytes());

    // And it is still the same message afterwards, segment terminators and
    // all — the frame's trailer uses the same byte, and must not eat one.
    let message = hl7_2::parse(std::str::from_utf8(payload).unwrap()).unwrap();
    assert_eq!(message.to_er7(), MESSAGE);
    assert_eq!(message.segments().count(), 4);
}

#[test]
fn a_stream_that_chops_frames_anywhere_still_yields_whole_messages() {
    let wire = [
        hl7_2_mllp::encode(MESSAGE.as_bytes()),
        hl7_2_mllp::encode(b"MSH|^~\\&|A||||1||ACK|2|P|2.5"),
    ]
    .concat();

    // Every possible split point, one at a time.
    for split in 0..wire.len() {
        let mut framer = Framer::new().with_tolerance(Tolerance::Strict);
        framer.push(&wire[..split]);
        let mut frames = framer.frames().unwrap();
        framer.push(&wire[split..]);
        frames.extend(framer.frames().unwrap());
        assert_eq!(frames.len(), 2, "split at {split}");
        assert_eq!(frames[0], MESSAGE.as_bytes(), "split at {split}");
        assert!(framer.is_empty(), "split at {split}");
    }
}

#[test]
#[cfg(feature = "ack")]
fn a_conversation_over_a_real_socket() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    // A receiver that answers every message and then hangs up.
    let receiver = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut transport = IoTransport::new(stream);
        let mut answered = 0;
        while let Some(payload) = transport.receive().unwrap() {
            let message = ack::parse(&payload).unwrap();
            let reply =
                ack::acknowledge_message(&message, AckCode::Accept, "ACK1", "20260814080100")
                    .unwrap();
            transport.send(reply.to_er7().as_bytes()).unwrap();
            answered += 1;
        }
        answered
    });

    let mut sender = IoTransport::new(TcpStream::connect(address).unwrap());
    sender.send_str(MESSAGE).unwrap();

    let reply = sender.receive().unwrap().unwrap();
    let acknowledgement = ack::parse(&reply).unwrap();
    assert_eq!(acknowledgement.get("MSA-1").unwrap().as_deref(), Some("AA"));
    // The echo is what ties this answer to that question.
    assert_eq!(
        acknowledgement.get("MSA-2").unwrap().as_deref(),
        Some("MSG00042")
    );

    drop(sender);
    assert_eq!(receiver.join().unwrap(), 1);
}

#[test]
#[cfg(feature = "ack")]
fn many_messages_over_one_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    let receiver = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut transport = IoTransport::new(stream);
        let mut seen = Vec::new();
        while let Some(payload) = transport.receive().unwrap() {
            let message = ack::parse(&payload).unwrap();
            seen.push(message.get("MSH-10").unwrap().unwrap());
            let reply =
                ack::acknowledge_message(&message, AckCode::Accept, "A", "20260814").unwrap();
            transport.send(reply.to_er7().as_bytes()).unwrap();
        }
        seen
    });

    let mut sender = IoTransport::new(TcpStream::connect(address).unwrap());
    for number in 1..=25 {
        let message =
            format!("MSH|^~\\&|LAB|A|EHR|C|20260814||ORU^R01|MSG{number:03}|P|2.5\rPID|1");
        sender.send_str(&message).unwrap();
        let reply = sender.receive().unwrap().unwrap();
        let acknowledgement = ack::parse(&reply).unwrap();
        assert_eq!(
            acknowledgement.get("MSA-2").unwrap().unwrap(),
            format!("MSG{number:03}")
        );
    }
    drop(sender);

    let seen = receiver.join().unwrap();
    assert_eq!(seen.len(), 25);
    assert_eq!(seen[0], "MSG001");
    assert_eq!(seen[24], "MSG025");
}

#[test]
fn a_sender_that_hangs_up_mid_message_does_not_produce_half_a_message() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    let receiver = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        IoTransport::new(stream).receive()
    });

    // A start block and half a message, then a closed socket.
    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(b"\x0bMSH|^~\\&|LAB|ACME").unwrap();
    stream.flush().unwrap();
    drop(stream);

    let result = receiver.join().unwrap();
    let error = result.expect_err("a truncated message must not read as a message");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn the_wire_bytes_are_exactly_what_the_standard_says() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    let receiver = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        bytes
    });

    let mut sender = IoTransport::new(TcpStream::connect(address).unwrap());
    sender.send_str("MSH|one").unwrap();
    sender.send_str("MSH|two").unwrap();
    drop(sender);

    let wire = receiver.join().unwrap();
    assert_eq!(wire, b"\x0bMSH|one\x1c\r\x0bMSH|two\x1c\r");
}

#[test]
#[cfg(feature = "ack")]
fn an_unreadable_payload_can_still_be_answered() {
    // A receiver must be able to say "no" to something that is not a
    // message, or the sender retries forever.
    let error = ack::acknowledge(b"this is not HL7", AckCode::Error, "N1", "20260814")
        .expect_err("not a message");
    assert!(matches!(error, ack::Error::NotHl7(_)), "{error}");
    assert!(error.to_string().contains("not an HL7 message"), "{error}");
}
