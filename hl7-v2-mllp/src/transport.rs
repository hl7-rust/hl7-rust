//! Frames over anything that reads and writes bytes.
//!
//! The [`Transport`] trait is the seam between MLLP and whatever carries
//! it. Most callers want [`IoTransport`] over a `TcpStream`; the trait
//! exists because "most" is not "all" — a TLS stream, a Unix socket, a
//! serial line, an in-memory pair for tests, or an async runtime's blocking
//! adapter all frame identically and differ only in how bytes move.

use crate::{Error, Framer, encode};
use std::io::{self, Read, Write};

/// Sending and receiving whole MLLP frames.
///
/// Implement it for a carrier this crate does not know about. The contract
/// is small on purpose:
///
/// - `send` writes one complete frame, or fails.
/// - `receive` returns one complete message, `None` at a clean end of
///   stream, or fails. It blocks if that is what the carrier does.
///
/// A framing violation is an [`io::Error`] of kind `InvalidData`, so a
/// caller that already handles I/O errors handles these too — and should,
/// because a stream that has lost framing cannot be resynchronized.
pub trait Transport {
    /// Frame `payload` and send it.
    /// # Errors
    ///
    /// Whatever the underlying stream reports on write.
    fn send(&mut self, payload: &[u8]) -> io::Result<()>;

    /// Receive the next complete message, or `None` at end of stream.
    /// # Errors
    ///
    /// Whatever the underlying stream reports on read, and
    /// [`io::ErrorKind::InvalidData`] for a stream that cannot be framed.
    fn receive(&mut self) -> io::Result<Option<Vec<u8>>>;

    /// Send `message` as UTF-8. HL7 v2 is usually ASCII, but MSH-18 can
    /// name another character set, and a message already encoded in one
    /// should go through [`Transport::send`] as bytes instead.
    /// # Errors
    ///
    /// Whatever the underlying stream reports on write.
    fn send_str(&mut self, message: &str) -> io::Result<()> {
        self.send(message.as_bytes())
    }
}

/// A [`Transport`] over any byte stream: `TcpStream`, a TLS stream, a Unix
/// socket, or a `Cursor` in a test.
///
/// ```
/// use hl7_v2_mllp::{IoTransport, Transport};
///
/// // Two frames waiting on a "connection" that is really a byte slice.
/// let incoming: &[u8] = b"\x0bMSH|one\x1c\r\x0bMSH|two\x1c\r";
/// let mut transport = IoTransport::new(std::io::Cursor::new(incoming.to_vec()));
///
/// assert_eq!(transport.receive()?.unwrap(), b"MSH|one");
/// assert_eq!(transport.receive()?.unwrap(), b"MSH|two");
/// assert_eq!(transport.receive()?, None);   // clean end of stream
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug)]
pub struct IoTransport<S> {
    stream: S,
    framer: Framer,
    chunk: Vec<u8>,
}

/// How much is read from the stream at a time. Large enough that a typical
/// message arrives in one read, small enough to be nothing on a heap.
const CHUNK: usize = 8 * 1024;

impl<S> IoTransport<S> {
    /// Wrap a stream, with the default [`Framer`].
    pub fn new(stream: S) -> IoTransport<S> {
        IoTransport::with_framer(stream, Framer::new())
    }

    /// Wrap a stream with a framer configured by the caller — a different
    /// size limit, or a different [`crate::Tolerance`].
    pub fn with_framer(stream: S, framer: Framer) -> IoTransport<S> {
        IoTransport {
            stream,
            framer,
            chunk: vec![0; CHUNK],
        }
    }

    /// The underlying stream, to inspect the peer address or set a timeout.
    pub fn stream(&self) -> &S {
        &self.stream
    }

    /// The underlying stream, mutably.
    pub fn stream_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    /// Take the stream back, dropping anything buffered mid-frame.
    pub fn into_stream(self) -> S {
        self.stream
    }

    /// The framer, to ask what is buffered mid-message.
    pub fn framer(&self) -> &Framer {
        &self.framer
    }
}

impl<S: Read + Write> Transport for IoTransport<S> {
    fn send(&mut self, payload: &[u8]) -> io::Result<()> {
        // One write of one frame, then flush: a half-written frame on the
        // wire is indistinguishable from a message still arriving, so the
        // frame is assembled in memory first.
        self.stream.write_all(&encode(payload))?;
        self.stream.flush()
    }

    fn receive(&mut self) -> io::Result<Option<Vec<u8>>> {
        loop {
            if let Some(frame) = self.framer.next_frame()? {
                return Ok(Some(frame));
            }
            let read = self.stream.read(&mut self.chunk)?;
            if read == 0 {
                // End of stream. Clean if it fell between frames; a peer
                // that hung up mid-message loses that message, and saying
                // so beats returning a truncated one.
                return if self.framer.is_empty() {
                    Ok(None)
                } else {
                    Err(io::Error::from(Error::Incomplete))
                };
            }
            self.framer.push(&self.chunk[..read]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stream that reads from one buffer and writes to another, and
    /// hands out its reads in fixed-size bites — which is how a socket
    /// behaves and a `Cursor` does not.
    #[derive(Debug)]
    struct Pipe {
        incoming: Vec<u8>,
        position: usize,
        bite: usize,
        outgoing: Vec<u8>,
    }

    impl Pipe {
        fn new(incoming: &[u8], bite: usize) -> Pipe {
            Pipe {
                incoming: incoming.to_vec(),
                position: 0,
                bite,
                outgoing: Vec::new(),
            }
        }
    }

    impl Read for Pipe {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let remaining = self.incoming.len() - self.position;
            let count = remaining.min(self.bite).min(out.len());
            out[..count].copy_from_slice(&self.incoming[self.position..self.position + count]);
            self.position += count;
            Ok(count)
        }
    }

    impl Write for Pipe {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.outgoing.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn receives_messages_however_the_stream_chops_them() {
        let wire = b"\x0bMSH|one\x1c\r\x0bMSH|two\x1c\r";
        for bite in [1, 2, 7, 1024] {
            let mut transport = IoTransport::new(Pipe::new(wire, bite));
            assert_eq!(
                transport.receive().unwrap().unwrap(),
                b"MSH|one",
                "bite {bite}"
            );
            assert_eq!(
                transport.receive().unwrap().unwrap(),
                b"MSH|two",
                "bite {bite}"
            );
            assert_eq!(transport.receive().unwrap(), None, "bite {bite}");
        }
    }

    #[test]
    fn sends_one_framed_message() {
        let mut transport = IoTransport::new(Pipe::new(b"", 64));
        transport.send_str("MSH|^~\\&|LAB").unwrap();
        assert_eq!(transport.stream().outgoing, b"\x0bMSH|^~\\&|LAB\x1c\r");
    }

    #[test]
    fn a_peer_that_hangs_up_mid_message_is_an_error_not_a_message() {
        let mut transport = IoTransport::new(Pipe::new(b"\x0bMSH|trunc", 4));
        let error = transport.receive().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("incomplete"), "{error}");
    }

    #[test]
    fn a_clean_close_between_frames_is_the_end_of_the_stream() {
        let mut transport = IoTransport::new(Pipe::new(b"\x0bonly\x1c\r", 3));
        assert_eq!(transport.receive().unwrap().unwrap(), b"only");
        assert_eq!(transport.receive().unwrap(), None);
        // And staying closed is not an error either.
        assert_eq!(transport.receive().unwrap(), None);
    }

    #[test]
    fn framing_violations_surface_as_invalid_data() {
        let mut transport = IoTransport::with_framer(
            Pipe::new(b"garbage\x0bMSH|\x1c\r", 64),
            Framer::new().with_tolerance(crate::Tolerance::Strict),
        );
        let error = transport.receive().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
