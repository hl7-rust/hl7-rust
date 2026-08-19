//! Accumulating frames from a byte stream.
//!
//! TCP delivers bytes, not messages. A read may return half a frame, three
//! frames, or two frames and half of a fourth, and the same message split
//! differently on the next connection. A [`Framer`] is the small amount of
//! state that turns that back into whole messages: push whatever arrived,
//! pull whatever is complete, keep the remainder for next time.

use crate::{CARRIAGE_RETURN, DEFAULT_LIMIT, END_BLOCK, Error, START_BLOCK};

/// How much malformed framing to accept.
///
/// Strict is the default and the right choice: a receiver that quietly
/// accepts a frame with no end block cannot tell a complete message from a
/// truncated one, and in this domain that difference is a patient record.
/// Lenient exists because specific senders are specifically wrong, and
/// forgiving two named sins beats abandoning framing altogether.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tolerance {
    /// `<VT>` payload `<FS><CR>`, exactly, with nothing between frames.
    Strict,
    /// Also accept `<FS>` without its `<CR>`, and discard bytes that arrive
    /// between frames instead of reporting them.
    ///
    /// Nothing else is forgiven: a frame with no start block, or with a
    /// `<VT>` inside it, is still an error, because in both cases the
    /// receiver genuinely cannot tell what the sender meant.
    Lenient,
}

impl Tolerance {
    /// The strict reading, unless the `noncompliance` feature is on.
    ///
    /// The feature changes the *default* rather than adding a capability:
    /// [`Tolerance::Lenient`] is always available to a caller who asks for
    /// it explicitly, and turning the feature on is how a caller says
    /// "everything on this deployment talks to that one sender".
    #[must_use]
    pub fn default_tolerance() -> Tolerance {
        if cfg!(feature = "noncompliance") {
            Tolerance::Lenient
        } else {
            Tolerance::Strict
        }
    }

    /// The strict reading, whatever the features say.
    #[must_use]
    pub fn strict() -> Tolerance {
        Tolerance::Strict
    }

    /// The lenient reading, whatever the features say.
    #[must_use]
    pub fn lenient() -> Tolerance {
        Tolerance::Lenient
    }

    /// Whether `<FS>` alone ends a frame.
    #[must_use]
    pub fn allows_missing_carriage_return(self) -> bool {
        self == Tolerance::Lenient
    }

    /// Whether bytes outside a frame are discarded rather than reported.
    #[must_use]
    pub fn allows_leading_bytes(self) -> bool {
        self == Tolerance::Lenient
    }
}

impl Default for Tolerance {
    fn default() -> Tolerance {
        Tolerance::default_tolerance()
    }
}

/// Turns a stream of bytes into whole MLLP frames.
///
/// ```
/// use hl7_2_mllp::Framer;
///
/// let mut framer = Framer::new();
/// // Two messages, arriving in three reads that respect no frame boundary.
/// framer.push(b"\x0bMSH|one\x1c\r\x0bMSH|t");
/// framer.push(b"w");
/// framer.push(b"o\x1c\r");
///
/// assert_eq!(framer.next_frame()?.unwrap(), b"MSH|one");
/// assert_eq!(framer.next_frame()?.unwrap(), b"MSH|two");
/// assert_eq!(framer.next_frame()?, None);   // nothing more yet
/// # Ok::<(), hl7_2_mllp::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Framer {
    buffer: Vec<u8>,
    limit: usize,
    tolerance: Tolerance,
}

impl Framer {
    /// A framer with the default limit ([`DEFAULT_LIMIT`]) and the default
    /// tolerance (strict, unless the `noncompliance` feature is on).
    #[must_use]
    pub fn new() -> Framer {
        Framer {
            buffer: Vec::new(),
            limit: DEFAULT_LIMIT,
            tolerance: Tolerance::default(),
        }
    }

    /// A framer that gives up after buffering `limit` bytes without seeing
    /// a complete frame. Set it to the largest message the interface can
    /// legitimately send; the point is to bound what a peer can make this
    /// process allocate.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Framer {
        self.limit = limit;
        self
    }

    /// A framer at a chosen [`Tolerance`], whatever the crate features say.
    #[must_use]
    pub fn with_tolerance(mut self, tolerance: Tolerance) -> Framer {
        self.tolerance = tolerance;
        self
    }

    /// How much malformed framing this framer accepts.
    #[must_use]
    pub fn tolerance(&self) -> Tolerance {
        self.tolerance
    }

    /// How many bytes are held, waiting for the rest of a frame.
    #[must_use]
    pub fn buffered(&self) -> usize {
        self.buffer.len()
    }

    /// Whether anything is buffered. A connection closing while this is
    /// true means the peer hung up mid-frame — worth logging, because the
    /// message it was sending is lost.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Discard whatever is buffered, after an error or a reconnect.
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Add bytes as they arrive.
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Take the next complete frame's payload, or `None` if one has not
    /// finished arriving.
    ///
    /// `None` means "read more and call again", and is the normal state of
    /// affairs mid-message; an error means the bytes are not MLLP and the
    /// connection cannot be trusted to resynchronize, so the usual response
    /// is to log it, [`Framer::reset`], and close.
    /// # Errors
    ///
    /// [`Error`] when the stream cannot be framed: bytes before a start
    /// block, a start block inside a frame, or more than the configured
    /// limit buffered without an end block. Framing cannot be resynchronised
    /// after any of these, so the connection is the caller's to close.
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>, Error> {
        let Some(start) = self.buffer.iter().position(|&byte| byte == START_BLOCK) else {
            // No frame has begun. Everything buffered is outside a frame.
            if self.buffer.is_empty() {
                return Ok(None);
            }
            if self.tolerance.allows_leading_bytes() {
                self.buffer.clear();
                return Ok(None);
            }
            return Err(self.fail(Error::LeadingBytes(self.buffer.len())));
        };
        if start > 0 {
            if !self.tolerance.allows_leading_bytes() {
                return Err(self.fail(Error::LeadingBytes(start)));
            }
            self.buffer.drain(..start);
        }

        // The frame runs from the start block to the first end block after
        // it. A second start block before that means the sender began a
        // frame without finishing the last one.
        let body = &self.buffer[1..];
        let Some(end) = body.iter().position(|&byte| byte == END_BLOCK) else {
            // No end block yet, so a second start block cannot belong to a
            // finished frame: the sender began one without ending the last.
            if body.contains(&START_BLOCK) {
                return Err(self.fail(Error::EmbeddedStartBlock));
            }
            self.check_limit()?;
            return Ok(None);
        };
        if body[..end].contains(&START_BLOCK) {
            return Err(self.fail(Error::EmbeddedStartBlock));
        }

        // `<FS>` at buffer index `end + 1`; its `<CR>` follows.
        let trailer = end + 2;
        match body.get(end + 1) {
            Some(&CARRIAGE_RETURN) => {
                let payload = body[..end].to_vec();
                self.buffer.drain(..=trailer);
                Ok(Some(payload))
            }
            Some(_) if self.tolerance.allows_missing_carriage_return() => {
                let payload = body[..end].to_vec();
                self.buffer.drain(..trailer);
                Ok(Some(payload))
            }
            Some(_) => Err(self.fail(Error::NoCarriageReturn)),
            // The end block arrived but its carriage return has not yet.
            None => {
                self.check_limit()?;
                Ok(None)
            }
        }
    }

    /// Every frame that has finished arriving, in order.
    ///
    /// Convenient when a read is expected to carry several messages. Stops
    /// at the first error, having already yielded the frames before it.
    /// # Errors
    ///
    /// The same conditions as [`Framer::next_frame`]; frames already pulled
    /// before the error are lost with it.
    pub fn frames(&mut self) -> Result<Vec<Vec<u8>>, Error> {
        let mut frames = Vec::new();
        while let Some(frame) = self.next_frame()? {
            frames.push(frame);
        }
        Ok(frames)
    }

    /// Fail, discarding the buffer: once framing is lost there is no
    /// dependable way to find the next boundary, and guessing would be
    /// worse than reporting.
    fn fail(&mut self, error: Error) -> Error {
        self.buffer.clear();
        error
    }

    fn check_limit(&mut self) -> Result<(), Error> {
        if self.buffer.len() > self.limit {
            let error = Error::TooLarge {
                buffered: self.buffer.len(),
                limit: self.limit,
            };
            return Err(self.fail(error));
        }
        Ok(())
    }
}

impl Default for Framer {
    fn default() -> Framer {
        Framer::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strict() -> Framer {
        Framer::new().with_tolerance(Tolerance::Strict)
    }

    #[test]
    fn reassembles_a_frame_split_across_reads() {
        let mut framer = strict();
        for chunk in [&b"\x0bMSH|"[..], b"^~\\&|LAB", b"\x1c\r"] {
            framer.push(chunk);
        }
        assert_eq!(framer.next_frame().unwrap().unwrap(), b"MSH|^~\\&|LAB");
        assert_eq!(framer.next_frame().unwrap(), None);
        assert!(framer.is_empty());
    }

    #[test]
    fn splits_several_frames_from_one_read() {
        let mut framer = strict();
        framer.push(b"\x0bone\x1c\r\x0btwo\x1c\r\x0bthree\x1c\r");
        let pulled = framer.frames().unwrap();
        assert_eq!(
            pulled,
            [b"one".to_vec(), b"two".to_vec(), b"three".to_vec()]
        );
        assert!(framer.is_empty());
    }

    #[test]
    fn holds_a_partial_frame_without_reporting_it_as_an_error() {
        let mut framer = strict();
        framer.push(b"\x0bMSH|");
        assert_eq!(framer.next_frame().unwrap(), None, "not an error, a wait");
        assert_eq!(framer.buffered(), 5);
        // Even the trailer can be split.
        framer.push(b"\x1c");
        assert_eq!(framer.next_frame().unwrap(), None);
        framer.push(b"\r");
        assert_eq!(framer.next_frame().unwrap().unwrap(), b"MSH|");
    }

    #[test]
    fn keeps_the_second_frame_while_yielding_the_first() {
        let mut framer = strict();
        framer.push(b"\x0bone\x1c\r\x0btw");
        assert_eq!(framer.next_frame().unwrap().unwrap(), b"one");
        assert_eq!(framer.next_frame().unwrap(), None);
        framer.push(b"o\x1c\r");
        assert_eq!(framer.next_frame().unwrap().unwrap(), b"two");
    }

    #[test]
    fn strict_mode_reports_what_lenient_mode_forgives() {
        let mut framer = strict();
        framer.push(b"garbage\x0bMSH|\x1c\r");
        assert_eq!(framer.next_frame(), Err(Error::LeadingBytes(7)));

        let mut framer = strict();
        framer.push(b"\x0bMSH|\x1cX");
        assert_eq!(framer.next_frame(), Err(Error::NoCarriageReturn));

        let mut framer = Framer::new().with_tolerance(Tolerance::Lenient);
        framer.push(b"garbage\x0bMSH|\x1cX");
        assert_eq!(framer.next_frame().unwrap().unwrap(), b"MSH|");
    }

    #[test]
    fn a_second_start_block_is_an_unfinished_frame_not_a_payload() {
        let mut framer = strict();
        framer.push(b"\x0bfirst\x0bsecond\x1c\r");
        assert_eq!(framer.next_frame(), Err(Error::EmbeddedStartBlock));
        // Even before any end block arrives, since no valid frame can
        // contain one.
        let mut framer = strict();
        framer.push(b"\x0bfirst\x0bsecond");
        assert_eq!(framer.next_frame(), Err(Error::EmbeddedStartBlock));
    }

    #[test]
    fn a_peer_that_never_ends_a_frame_cannot_exhaust_memory() {
        let mut framer = Framer::new().with_limit(64);
        framer.push(b"\x0b");
        framer.push(&[b'x'; 100]);
        assert_eq!(
            framer.next_frame(),
            Err(Error::TooLarge {
                buffered: 101,
                limit: 64
            })
        );
        // And the buffer is released, not held onto after the failure.
        assert!(framer.is_empty());
    }

    #[test]
    fn an_empty_payload_is_a_frame() {
        let mut framer = strict();
        framer.push(b"\x0b\x1c\r");
        assert_eq!(framer.next_frame().unwrap().unwrap(), b"");
    }

    #[test]
    fn a_reset_discards_a_half_read_message() {
        let mut framer = strict();
        framer.push(b"\x0bhalf");
        assert!(!framer.is_empty());
        framer.reset();
        assert!(framer.is_empty());
        assert_eq!(framer.next_frame().unwrap(), None);
    }
}
