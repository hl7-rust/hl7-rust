[index](../index.md) → §1 Purpose and scope

# §1 Purpose and scope

## 1.1 What this crate is

`serde-hl7-v2` gives the public value types of the
[`hl7-2`](../../../hl7-2/spec/index.md) crate — `Message`, `Node` and its
`generic::Kind`, `Diagnostic` with its `Severity` and `validate::Kind`,
and `Version` — a hand-written `Serialize` and, wherever the type can be
rebuilt, `Deserialize` implementation, via a same-named wrapper type per
value (`NodeKind` and `DiagnosticKind` for the two enums both called
`Kind`). That is the entire feature surface.

It is reached directly as `serde_hl7_v2::...`, or as `serde_hl7::v2::...`
through the `serde-hl7` umbrella crate, the same way `hl7-2` is reached
as `hl7::v2`.

## 1.2 Why a wrapper crate rather than an added dependency

`hl7-2` has exactly one runtime dependency, `er7`, by its own rule, and
its `AGENTS.md` lists pulling in `serde` under *don't*: it sits near the
bottom of a stack of HL7® crates in a domain where dependency trees are
audited (`spec/phi/index.md` at the workspace root records "no
serialization framework" as a property of the whole family). Adding
`serde` to `hl7-2` directly would impose that dependency on every
consumer, including ones that never touch Serde. A separate crate lets the
choice be the caller's: depend on `hl7-2` alone, or add `serde-hl7-v2` on
top when a Serde format is actually needed. This is the same argument
`serde-er7` makes for `er7` (its spec §1.2), one layer up.

## 1.3 What problem this solves

Once a message is an `hl7_2::Message`, a caller often wants to hand it, or
what `hl7-2` derived from it, to something that only speaks Serde: a
document database driver, a web framework's JSON response type, a
structured logger, a snapshot-testing library. Three things in particular:

- **the message itself**, to store or forward, keeping the release it was
  read as;
- **the dictionary-named tree** (`PID.5`, `XPN.1`) — the view of a
  message that reads sensibly in a log or a query result, which until now
  only `hl7-2-from-er7-into-json` could render, to one fixed JSON shape;
- **validation findings**, to return from an API or write to a log.

With this crate each is one `serde_json::to_string` (or any other
format's) away, and the first and third come back the same way.

## 1.4 Non-goals

- **A second tree shape for ER7.** The encoding layer's tree —
  segments, fields, repetitions, components, subcomponents — is
  `serde-er7`'s to serialize, over `er7`'s types, which `hl7-2` exposes as
  `hl7_2::er7` and `Message::raw()`. This crate serializes a message as
  ER7 text (S3), not as a tree; see [§2.3](../02-wire-shapes/index.md).
- **A format.** This crate never mentions JSON, YAML, or any other format
  in its own runtime dependencies or public API. See
  [§3](../03-dependencies-and-format-agnosticism/index.md).
- **Serde for `Dictionary`.** Dictionaries are already JSON, in the shape
  `hl7_2::Dictionary::from_json` reads and `hl7-2-from-xsd-into-json-dictionary`
  writes; a second shape would be a second thing to keep in step.
- **Serde for struct mode.** `FromHl7`/`ToHl7` map a caller's own struct
  onto a message; that struct is the caller's to derive `Serialize` for.
- **A validator.** Structurally malformed *Serde* input is rejected with a
  `Deserialize` error; ER7 text that `hl7_2::parse` accepts is accepted.
- **A replacement for `hl7-2`'s own API.** Every wrapper `Deref`s to its
  `hl7-2` type ([§6](../06-ergonomics/index.md)), so this crate adds a
  capability rather than a parallel API surface.

## 1.5 Which goal wins when two conflict

See [the table in the index](../index.md#which-goal-wins-when-two-conflict).

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
