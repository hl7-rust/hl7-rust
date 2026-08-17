# AGENTS.md

Instructions for coding agents (Claude Code, Codex, or any other) working in
this repository. `CLAUDE.md` is a pointer to this file — keep this one
canonical and don't fork the content between the two.

## What this is

A Rust crate implementing MLLP, the Minimal Lower Layer Protocol: the
three-byte framing that carries HL7 v2 messages over TCP, plus the
streaming, transport, and acknowledgement pieces a real interface needs
around it.

It is the **transport layer** of the `hl7-rust` family:

```
er7            the ER7 encoding
  |
hl7-rust       HL7 v2 itself (imported as `hl7`; API in `hl7::v2`)
  |
  +-- hl7-v2-mllp    this crate: bytes on a wire
  +-- hl7-v2-from-er7-into-json / -into-xml / from-json / from-xml
```

**The layer boundary is the point, and MLLP is where people erode it.**

- This crate owns *where a message starts and stops*, and how a reply gets
  back.
- `hl7-rust` owns *what a message means*. This crate parses a payload only
  to build an acknowledgement, and validates nothing.
- Neither owns *policy*: persistence before `AA`, timeouts, reconnection,
  and what to do about an `AR` are the application's.

If a change here needs to know what a segment means, it probably belongs in
`hl7-rust`. If it needs to know what the deployment wants, it belongs in
the caller.

See `README.md` for the user-facing pitch and `spec/index.md` for the exact,
normative rules — **`spec/index.md` is the single source of truth for
behavior.**

## Layout

```
src/lib.rs         Constants, encode/decode, Tolerance-aware decode_with,
                   the Error type, and the crate documentation.
src/framer.rs      Framer: bytes in, whole frames out. Tolerance lives here.
src/transport.rs   The Transport trait and IoTransport over Read + Write.
src/ack.rs         AckCode and acknowledgement generation (feature `ack`).
tests/integration.rs  Black-box tests, including real TCP connections.
examples/tcp_listener.rs  A working listener that acknowledges.
examples/tcp_sender.rs    The other end, which checks the echoed control ID.
spec/index.md      Normative specification (source of truth).
```

Each module has unit tests in a trailing `#[cfg(test)] mod tests` block;
anything needing a real socket, or crossing module boundaries, goes in
`tests/integration.rs`.

## Working conventions

- **Rust edition 2024.** Dependencies: `hl7-rust` behind the default-on
  `ack` feature, `chrono` behind the off-by-default `clock` feature, and
  nothing else. `--no-default-features` must keep compiling with **zero**
  dependencies — that is a promise in the README, and worth keeping in a
  domain where dependency trees get audited.
- **Never modify a payload.** Not trimming, not normalizing line endings,
  not validating, not re-encoding. A frame carries bytes; what they mean is
  someone else's question, and a receiver that "helpfully" adjusts a
  clinical message is a bug with consequences.
- **A partial frame is not an error.** `Ok(None)` means "read more", and
  confusing the two is the single easiest way to break a working interface.
- **Strict by default.** Every relaxation goes through `Tolerance`, and the
  `noncompliance` feature only changes the *default* — both tolerances must
  stay reachable by name in either build, so a library caller is never at
  the mercy of a downstream feature flag.
- **The clock stays opt-in.** No function outside the `clock` feature may
  read the system time, and even there the control ID stays the caller's.
  Generated timestamps make tests non-deterministic and messages
  untraceable.
- Every public item must have a doc comment; `src/lib.rs` carries
  `#![warn(missing_docs)]` to enforce it.
- Match the existing doc style: a one-paragraph summary of *what* and,
  where the *why* isn't obvious, a short rationale — see any existing `///`
  comment for the register to match.
- Before finishing a change, run:
  ```sh
  cargo test --all-features
  cargo test --no-default-features        # the zero-dependency build
  cargo clippy --all-targets --all-features -- -D warnings
  cargo fmt --check
  cargo rustdoc --lib -- -W missing-docs
  ```
  Feature-sensitive behavior needs testing under more than one combination;
  see `tests::the_feature_chooses_the_default_and_nothing_else` for the
  pattern (assert the invariant, not the build you happen to be in).
- The examples are documentation and must keep working. Run them against
  each other after touching framing or transport:
  ```sh
  cargo run --example tcp_listener 127.0.0.1:12575 &
  cargo run --example tcp_sender  127.0.0.1:12575
  ```

## Making a spec-affecting change

1. **Update `spec/index.md` first** so it states the new behavior precisely.
2. Implement it, matching the module boundaries above.
3. Add or update the tests that pin it.
4. **Add the rule to the §9 traceability table**, naming those tests.
5. Update `README.md` only if the user-facing summary or examples change.
6. Run the checks above.

## Non-goals (don't "fix" these without discussion)

- **TLS.** `IoTransport` takes any `Read + Write`, so encryption composes
  from outside. Adding a TLS dependency here would put a large tree behind
  a crate whose whole point is being small.
- **Async.** The transport is blocking on purpose: MLLP connections are few
  and long-lived, so a thread each is the right shape. `Framer` is I/O-free
  and is what an async caller should build on.
- **Retry, backoff, reconnection, connection pools, timeouts.** All policy,
  all the caller's. A read timeout in particular belongs on the socket,
  which is why `IoTransport::stream_mut` exists.
- **Validating the payload, or looking inside it** beyond what building an
  acknowledgement requires.
- **MLLP Release 2 commit blocks**, unless a real interface needs them —
  and note they are a different thing from the enhanced-mode HL7 codes in
  `AckCode`, which are supported.
- **Interpreting the acknowledgement for the caller** — deciding that an
  `AR` means "retry later" is application policy.
