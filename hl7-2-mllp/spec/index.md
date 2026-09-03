# Specification: HL7® v2 over MLLP

This is the single source of truth for what this crate does and how. It
describes observable behavior, not implementation details; where a rule is
implemented, the module is named so the two stay in sync. `README.md`
summarizes this document for newcomers — if the two disagree, this document
wins and the README should be corrected to match.

Status: describes the behavior of `hl7_2_mllp` as implemented. Every rule
below is exercised by a unit test (next to the code that implements it, in
that module's `#[cfg(test)]` block) or an integration test
(`tests/integration.rs`). A change to this document that isn't backed by a
test, or a code change that isn't reflected here, is a bug.

## 0. Relationship to the rest of the family

```
er7                      the ER7 encoding: delimiters, escapes, rendering
  |
hl7-2                 the HL7 v2 dictionary: releases 2.1-2.9, data
                         types, structures, three parsing modes,
  |                      mutation, validation
  |
  +-- hl7-2-mllp        this crate: getting those messages across a
  |                      network, and answering them
  +-- hl7-2-soap        transport: HL7 v2 over HTTP
  +-- hl7-2-from-er7-into-json    format conversions
  +-- hl7-2-from-er7-into-xml
  +-- hl7-2-from-json-into-er7
  +-- hl7-2-from-xml-into-er7
```

The division is strict, and worth stating because MLLP is where people
reach for a framework:

- **This crate owns bytes on a wire.** Where a message starts, where it
  stops, and how a reply gets back.
- **`hl7-2` owns what a message means.** This crate parses a payload
  only to build an acknowledgement (§5), and validates nothing.
- **Neither owns policy.** Whether an `AA` may be sent before the message
  is persisted, how long to wait for a reply, when to reconnect, and what
  to do with a `AR` are decisions only the application can make.
- **`hl7-2-soap` sits beside this crate, not above it.** Both answer the
  same question — how does a message get from one system to another, and
  how does the receiver say what became of it — for two different
  transports.

## 1. Scope

Frame HL7 v2 messages for a byte stream, and unframe them from one:

- **Framing** (§3) — one frame in hand, encoded or decoded.
- **Streaming** (§4) — frames accumulated from a stream that respects no
  message boundary.
- **Acknowledgement** (§5) — turning a received message into the reply HL7
  expects, framed and ready to send.
- **Transport** (§6) — the two above over anything that reads and writes
  bytes.

Out of scope: TLS (compose it — `IoTransport` takes any stream), async
runtimes, connection pooling, retry and backoff policy, message
persistence, and HL7 v2 semantics of every kind.

## 2. The protocol

MLLP wraps each message in three bytes:

```text
<VT> message <FS><CR>
0x0B          0x1C 0x0D
```

| name | byte | constant |
|---|---|---|
| start block, `<VT>`, vertical tab | `0x0B` | [`START_BLOCK`] |
| end block, `<FS>`, file separator | `0x1C` | [`END_BLOCK`] |
| carriage return, `<CR>` | `0x0D` | [`CARRIAGE_RETURN`] |

[`START_BLOCK`]: https://docs.rs/hl7-2-mllp/latest/hl7_2_mllp/constant.START_BLOCK.html
[`END_BLOCK`]: https://docs.rs/hl7-2-mllp/latest/hl7_2_mllp/constant.END_BLOCK.html
[`CARRIAGE_RETURN`]: https://docs.rs/hl7-2-mllp/latest/hl7_2_mllp/constant.CARRIAGE_RETURN.html

There is nothing else: no length, no checksum, no version negotiation, no
session, no encryption. Two consequences follow, and both are load-bearing
in what follows.

**There is no escaping.** MLLP defines no escape character, so a payload
containing `<VT>` or `<FS>` cannot be framed unambiguously. HL7 v2 text
never contains either byte; `is_framable` is the check for a payload from
somewhere less certain. `encode` does not check, because for its intended
input the answer is always yes.

**`<CR>` is also HL7 v2's segment terminator.** A message's own trailing
`\r`, if the sender wrote one, sits before the `<FS>` and is part of the
payload. The frame's `<CR>` is the one *after* `<FS>`. Nothing in this
crate adds, removes, or normalizes a payload byte (§3.1).

## 3. Framing (`src/lib.rs`)

### 3.1 Encoding

`encode(payload)` returns `<VT>` + payload + `<FS><CR>`. The payload is
copied verbatim: not inspected, not validated, not modified, not trimmed.
A zero-length payload is a legal frame.

### 3.2 Decoding

`decode(frame)` returns the payload between the blocks, or an `Error`. It
is for a frame already in hand; a stream needs §4.

### 3.3 Tolerance

Two readings, chosen by `Tolerance`:

| | `Strict` | `Lenient` |
|---|---|---|
| `<FS>` with no `<CR>` | `Incomplete` | accepted |
| bytes before `<VT>` | `LeadingBytes` | discarded |
| no `<VT>` at all | `NoStartBlock` | `NoStartBlock` |
| `<VT>` inside the payload | `EmbeddedStartBlock` | `EmbeddedStartBlock` |

Lenient forgives exactly the two sins real senders commit, and nothing
else. The last two rows stay errors under both because the receiver
genuinely cannot tell what the sender meant.

`Tolerance::default()` is `Strict`, unless the `noncompliance` feature is
on, in which case it is `Lenient`. **The feature changes the default, not
the capability**: `Tolerance::strict()` and `Tolerance::lenient()` both
work in either build, so a library that must not be at the mercy of a
downstream feature flag names the one it wants. Strict is the default
because a receiver that quietly accepts a frame with no end block cannot
distinguish a complete message from a truncated one.

### 3.4 Errors

| variant | when |
|---|---|
| `NoStartBlock` | the frame does not begin with `<VT>` |
| `NoCarriageReturn` | `<FS>` is followed by something other than `<CR>` |
| `Incomplete` | no `<FS><CR>` yet — a wait against a stream, an error from `decode` |
| `TrailingBytes` | a complete frame, then bytes that do not begin another |
| `EmbeddedStartBlock` | a `<VT>` inside the payload |
| `LeadingBytes` | bytes before `<VT>`, outside any frame |
| `TooLarge` | more buffered than the limit allows without a complete frame (§4.3) |

Every variant means the *bytes* are not MLLP. None means the *message* is
wrong: that question belongs to `hl7_2::Message::validate`, one layer up,
and can only be asked once framing has succeeded. `Error` converts into
`std::io::Error` of kind `InvalidData`, which is how it reaches a caller
through §6.

## 4. Streaming (`src/framer.rs`)

A `Framer` turns a byte stream into whole frames: `push` what arrived,
`next_frame` for what is complete.

### 4.1 Partial frames are not errors

`next_frame` returns `Ok(None)` when no complete frame has arrived — the
normal state mid-message — and keeps the bytes for next time. A frame may
be split at any point, including between `<FS>` and its `<CR>`, and any
number of frames may arrive in one push.

### 4.2 A framing error discards the buffer

Once framing is lost there is no dependable way to find the next boundary,
so an error clears the buffer rather than leaving bytes that would produce
a plausible-looking frame from two real ones. The caller's usual response
is to log and close the connection.

### 4.3 A limit bounds what a peer can allocate

MLLP has no length field, so a peer that never sends `<FS>` — a broken
sender, another protocol, a port scanner — would otherwise grow the buffer
until the process dies. Exceeding the limit is `TooLarge`; the default is
`DEFAULT_LIMIT`, 16 MiB, and `Framer::with_limit` sets it to whatever the
interface can legitimately send.

## 5. Acknowledgement (`src/ack.rs`, feature `ack`)

MLLP has no acknowledgement of its own. The reply HL7 expects is an HL7
message — an `ACK` whose `MSA-2` echoes the control ID (`MSH-10`) of the
message being answered — framed and sent back over the same connection.

That echo is the entire mechanism. MLLP guarantees a message arrived whole;
only the echoed control ID says *which* message arrived, so a sender that
does not compare it will eventually treat one answer as another's.

### 5.1 What is generated

`acknowledge` parses the payload, builds the `ACK`, and frames it. It is
built by `hl7_2::builder::acknowledge`, so per that crate's spec §7.4:
sender and receiver swap (`MSH-3`/`MSH-4` ↔ `MSH-5`/`MSH-6`), `MSA-2`
echoes `MSH-10`, and the reply is in the release the sender declared in
`MSH-12`.

`acknowledge_message` takes a parsed message and returns the `ACK`
unframed, for a receiver that wants to add an `ERR` segment or a reason in
`MSA-3` before sending — which a receiver saying `AE` should.

### 5.2 Codes

`AckCode`: `AA`, `AE`, `AR`, and the enhanced-mode `CA`, `CE`, `CR`. The
enhanced codes distinguish "safely stored" from "processed" and are only
appropriate when the sender asked for enhanced mode in MSH-15/MSH-16.

### 5.3 The clock is opt-in

Every function here takes the acknowledgement's own control ID and
timestamp as arguments, because a message that invents them is untestable
(the output changes every run) and untraceable (the control ID is what an
operator greps for). The `clock` feature adds `acknowledge_now` and `now`,
which supply the timestamp — local time, `YYYYMMDDHHMMSS` — from the
system clock. The control ID stays the caller's.

### 5.4 Failures are distinguished

`Error::NotText` (not UTF-8) and `Error::NotHl7` (text, but not a message)
are separate variants because they mean different things about the peer: a
character-set or protocol mismatch, versus a sender writing something else
into the frame. Either way the receiver should still answer — silence
leaves the sender retrying forever.

## 6. Transport (`src/transport.rs`)

`Transport` is two methods: `send` one payload, framed; `receive` one
message, unframed. `IoTransport<S>` implements it for any `S: Read +
Write`, so a `TcpStream`, a TLS stream, a Unix socket, or a byte buffer in
a test all behave the same.

### 6.1 End of stream

`receive` returns `Ok(None)` when the peer closes **between** frames — a
clean close — and `Err(InvalidData)` when it closes **mid-frame**. The
distinction matters: the second case means the message the peer was sending
is lost, and returning what arrived would hand the caller a truncated
clinical message.

### 6.2 One write per frame

`send` assembles the frame in memory and writes it in one call, then
flushes. A partially written frame is indistinguishable on the wire from a
message still arriving.

## 7. Features

| feature | default | effect |
|---|---|---|
| `ack` | on | acknowledgement generation (§5); pulls in `hl7-2` |
| `clock` | off | `acknowledge_now`, `now`; pulls in `chrono`. Implies `ack` |
| `noncompliance` | off | `Tolerance::default()` becomes `Lenient` (§3.3) |

`--no-default-features` gives framing, streaming, and transport with **no
dependencies at all**.

## 8. Limitations

- **No TLS.** MLLP is plaintext and HL7 messages are patient data.
  `IoTransport` accepts any stream, so wrap it in one that encrypts.
- **No async.** The transport is blocking. A thread per connection suits
  MLLP's few, long-lived connections; an async caller can use `Framer`
  directly, which is I/O-free.
- **No retry, timeout, or reconnect policy.** A read timeout in particular
  is the caller's to set on the socket, and a production listener needs
  one.
- **No persistence guarantee.** Sending `AA` promises the message is safe;
  making that true is the application's job, before it calls `send`.
- **MLLP Release 1 only.** The commit-acknowledgement block of Release 2 is
  not implemented; the enhanced-mode HL7 codes in §5.2 are a different
  thing at a different layer, and are supported.
- **UTF-8 assumed for acknowledgement.** `MSH-18` may declare another
  character set; decode it yourself and use `acknowledge_message`.

## 9. Traceability

Every section above is pinned by at least one test. A rule with no test is
a rule nobody is holding.

| § | rule | test |
|---|---|---|
| 3.1 | encode wraps, and copies the payload verbatim | `tests::wraps_and_unwraps_a_message`, `tests::leaves_the_payload_exactly_as_it_was` |
| 3.1 | what cannot be framed | `tests::knows_what_cannot_be_framed` |
| 3.2 | decode, and each way it refuses | `tests::refuses_framing_that_is_not_framing` |
| 3.3 | tolerance forgives two things and no more | `tests::tolerance_forgives_a_missing_carriage_return_and_nothing_else`, `framer::tests::strict_mode_reports_what_lenient_mode_forgives` |
| 3.3 | the feature chooses the default, not the capability | `tests::the_feature_chooses_the_default_and_nothing_else` |
| 3.4 | errors cross the I/O boundary as `InvalidData` | `tests::errors_carry_across_the_io_boundary`, `transport::tests::framing_violations_surface_as_invalid_data` |
| 4.1 | frames split anywhere, and several per push | `framer::tests::reassembles_a_frame_split_across_reads`, `framer::tests::splits_several_frames_from_one_read`, `framer::tests::holds_a_partial_frame_without_reporting_it_as_an_error`, `framer::tests::keeps_the_second_frame_while_yielding_the_first`, `a_stream_that_chops_frames_anywhere_still_yields_whole_messages` |
| 4.1 | an empty payload is a frame | `framer::tests::an_empty_payload_is_a_frame` |
| 4.2 | an error discards the buffer | `framer::tests::a_second_start_block_is_an_unfinished_frame_not_a_payload`, `framer::tests::a_reset_discards_a_half_read_message` |
| 4.3 | the limit bounds allocation | `framer::tests::a_peer_that_never_ends_a_frame_cannot_exhaust_memory` |
| 5.1 | the acknowledgement's shape and echo | `ack::tests::answers_a_message_with_its_own_control_id`, `ack::tests::sender_and_receiver_change_places`, `ack::tests::answers_in_the_release_the_sender_spoke` |
| 5.1 | a receiver can say why | `ack::tests::a_receiver_can_say_what_was_wrong` |
| 5.2 | every code reaches MSA-1 | `ack::tests::every_code_reaches_msa_1` |
| 5.3 | the clock feature | `ack::tests::the_clock_feature_fills_in_the_timestamp` |
| 5.4 | the two ways a payload is not a message | `ack::tests::a_payload_that_is_not_a_message_says_which_way_it_failed`, `an_unreadable_payload_can_still_be_answered` |
| 6 | send and receive over a stream, and a real socket | `transport::tests::receives_messages_however_the_stream_chops_them`, `transport::tests::sends_one_framed_message`, `a_conversation_over_a_real_socket`, `many_messages_over_one_connection` |
| 6.1 | clean close versus mid-frame close | `transport::tests::a_clean_close_between_frames_is_the_end_of_the_stream`, `transport::tests::a_peer_that_hangs_up_mid_message_is_an_error_not_a_message`, `a_sender_that_hangs_up_mid_message_does_not_produce_half_a_message` |
| 6.2 | the exact bytes on the wire | `the_wire_bytes_are_exactly_what_the_standard_says` |
| 2, 3 | a message survives the round trip unchanged | `a_message_survives_the_round_trip_to_the_wire` |

## 10. References

- HL7 v2 standards, including the transport specification that defines
  MLLP: <https://www.hl7.org/implement/standards/>
- `hl7-2` (HL7 v2 itself): <https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2>
- `er7` (the ER7 encoding): <https://crates.io/crates/er7>

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
