# Install

Three ways in, depending on what you are doing. If you only want to look at
a message and get on with your day, start with the command line — it needs
no Rust knowledge at all.

- [The command line](#the-command-line)
- [As a library](#as-a-library)
- [From source](#from-source)
- [Requirements](#requirements)
- [Feature flags](#feature-flags)
- [Verifying the install](#verifying-the-install)
- [Uninstalling](#uninstalling)

Full documentation is at <https://hl7-rust.github.io>; the
[install page](https://hl7-rust.github.io/docs/install/) there covers the
same ground with more examples.

## The command line

Six binaries ship in this workspace. Each is a single self-contained
executable with no runtime to install alongside it — no JVM, no
interpreter, no service.

```sh
cargo install hl7-2                                   # installs `hl7-v2`
cargo install hl7-2-from-er7-into-json
cargo install hl7-2-from-er7-into-xml
cargo install hl7-2-from-json-into-er7
cargo install hl7-2-from-xml-into-er7
cargo install hl7-2-from-xsd-into-json-dictionary
```

Note the first line: the crate is `hl7-2`, and the binary it installs is
`hl7-v2`. The command keeps the name of the thing it works on. The other
five install a binary named after the crate.

`cargo install` puts them in `~/.cargo/bin`, which Rust's installer adds to
your `PATH`.

```sh
hl7-v2 --tree --paths message.hl7   # every node, its value, and its path
hl7-v2 --check message.hl7          # validate, and report
hl7-v2 --query PID-5.1 message.hl7  # read one field
hl7-v2 --er7 message.hl7            # parse and render back out
```

Reading is from a named file or standard input, writing to standard output
or a named file with `-o`. Nothing else is read or written — no config file
is searched for and no environment variable is consulted. See
[`spec/phi/index.md`](spec/phi/index.md) before piping patient data around.

Every flag of every binary is documented at
<https://hl7-rust.github.io/docs/cli/>.

## As a library

For HL7® v2, one line is usually enough:

```sh
cargo add hl7
```

That is the umbrella crate: `hl7::v2` is `hl7-2` and `hl7::v3` is `hl7-3`.

```rust
let message = hl7::v2::parse(text)?;
let family_name = message.get("PID-5.1")?;
```

Or take the layer you want directly, and skip what you do not:

```sh
cargo add hl7-2                                  # HL7 v2 alone
cargo add hl7-2-mllp                             # MLLP framing over TCP
cargo add hl7-2-soap                             # SOAP envelopes over HTTP
cargo add hl7-2-from-er7-into-json               # one conversion direction
cargo add hl7-3                                  # the HL7 v3 foundation
```

Nothing forces you to take the whole workspace. `hl7-2` depends on `er7`,
which depends on nothing — that is the entire tree, which matters where
dependency trees get audited.

## From source

```sh
git clone https://github.com/hl7-rust/hl7-rust.git
cd hl7-rust
cargo build
cargo test
```

One `Cargo.lock` at the workspace root covers every member; a crate does
not carry its own. Use `-p <crate>` to scope a command to one member:

```sh
cargo test -p hl7-2
cargo run -p hl7-2 --bin hl7-v2 -- --tree message.hl7
cargo bench -p hl7-2
```

Mirrors of the repository are on
[GitLab](https://gitlab.com/hl7-rust) and
[Codeberg](https://codeberg.org/hl7-rust); issues live on GitHub.

## Requirements

| | |
|---|---|
| Rust | Current stable minus three releases. Today that is **1.95**. |
| Edition | 2024, which needs 1.85 — no longer the binding constraint. |
| Platform | Anything Rust targets. No platform-specific code, no C dependency, no build script that shells out. |
| Network | None, at build time or run time. The release dictionaries are compiled into the binary. |

The Rust floor is a rolling window, and the policy behind it is
[`spec/rust-msrv-n-minus-3/index.md`](spec/rust-msrv-n-minus-3/index.md).
It exists because healthcare toolchains are approved on a cycle measured in
quarters, so a library demanding the compiler released this month is a
library that cannot be adopted.

Check a build against the floor with:

```sh
cargo +1.95 check --workspace --all-targets
```

If you do not have Rust: <https://rustup.rs>.

## Feature flags

Everything is off by default, so the dependency-free build stays
dependency-free.

| Crate | Feature | Default | Effect |
|---|---|---|---|
| `hl7` | `derive` | off | Forwards to `hl7-2`'s `derive` |
| `hl7-2` | `derive` | off | `#[derive(FromHl7)]` and `#[derive(ToHl7)]`; pulls in `syn` and `quote` at compile time |
| `hl7-3` | `derive` | off | `#[derive(FromElement)]` |
| `hl7-2-mllp` | `ack` | **on** | Turn a received message into the acknowledgement HL7 expects; pulls in `hl7-2` |
| `hl7-2-mllp` | `clock` | off | `acknowledge_now`: take the timestamp from the system clock. Implies `ack`; pulls in `chrono` |
| `hl7-2-mllp` | `noncompliance` | off | Accept the two framing sins real senders commit: a missing carriage return after the end block, and bytes between frames |

```sh
cargo add hl7-2 --features derive
cargo add hl7-2-mllp --no-default-features   # framing only, zero dependencies
```

## Verifying the install

```sh
hl7-v2 --version
printf 'MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ADT^A08|MSG1|P|2.5\r' | hl7-v2 --tree
```

The second prints a tree rooted at `ADT_A01` — the structure that
`ADT^A08` resolves to. If it does, the dictionary loaded and you are done.

## Uninstalling

```sh
cargo uninstall hl7-2       # and any other installed crate by name
```

Nothing is left behind: no config directory is created, no cache is
written, no service is registered.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
