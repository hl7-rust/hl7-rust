[index](../index.md) → §1 Purpose and scope

# §1 Purpose and scope

## 1.1 What this crate is

`serde-hl7-v3` gives every public value type of the
[`hl7-3`](../../../hl7-3/spec/index.md) crate — the message envelope
(`Message`, `ControlAct`), the six RIM backbone classes (`Act`, `Entity`,
`Role`, `Participation`, `ActRelationship`, `RoleLink`), the data types
(`Ii`, `Cd`, `Ivl`, `Pq`, `Ed`, `NullFlavor`), and the XML `Element` tree
they are all read from — a hand-written `Serialize` and `Deserialize`
implementation, via a same-named wrapper type per value. That is the entire
feature surface.

It is reached directly as `serde_hl7_v3::...`, or as `serde_hl7::v3::...`
through the `serde-hl7` umbrella crate, the same way `hl7-3` is reached
as `hl7::v3`.

## 1.2 Why a wrapper crate rather than an added dependency

`hl7-3` has exactly one runtime dependency, the workspace's own
dependency-free XML reader, and the workspace's `spec/phi/index.md`
records "no serialization framework" as a checkable property of the whole
family. Adding `serde` to `hl7-3` directly would impose that dependency on
every consumer, including ones that never touch Serde. A separate crate
lets the choice be the caller's. This is the same argument `serde-er7`
makes for `er7` (its spec §1.2) and `serde-hl7-v2` makes for `hl7-2`.

## 1.3 What problem this solves

Once an interaction is an `hl7_3::Message`, or a payload has been read into
a RIM class, a caller often wants to hand it to something that only speaks
Serde: a document database, a web framework's response type, a structured
logger, a snapshot test. HL7® v3's own serialization is XML, and `hl7-3`
deliberately has no XML *writer* (its spec §1) — so without this crate the
decoded value has no way out at all except the caller's own code. With it,
`Message`, every RIM class, and the raw `Element` tree are one
`serde_json::to_string` (or any other format's) away, and come back the
same way.

## 1.4 Non-goals

- **An XML writer, or XML as a Serde format.** This crate serializes
  `hl7-3`'s *values*; producing HL7 v3 XML from them is a different
  project, and one `hl7-3`'s spec §1 lists as out of scope for that crate
  too. `Element` round-trips as a value, not as a document
  ([§4](../04-round-trip-guarantee/index.md)).
- **A format.** This crate never mentions JSON, YAML, or any other format
  in its own runtime dependencies or public API. See
  [§3](../03-dependencies-and-format-agnosticism/index.md).
- **Serde for struct mode.** `FromElement` maps a caller's own struct onto
  an element; that struct is the caller's to derive `Serialize` for.
- **Vocabulary validation.** `Cd::code` and `NullFlavor` carry whatever
  string was sent, as `hl7-3` does (its spec §6); this crate checks shape,
  not domain membership.
- **A replacement for `hl7-3`'s own API.** Every wrapper `Deref`s to its
  `hl7-3` type ([§6](../06-ergonomics/index.md)).

## 1.5 Which goal wins when two conflict

See [the table in the index](../index.md#which-goal-wins-when-two-conflict).

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
