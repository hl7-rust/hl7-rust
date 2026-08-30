---
name: hl7-skill
description: Explain Health Level Seven (HL7®) concepts, terminology, and ideas — what a segment, field, component, and repetition are; what MSH-9, ER7, the HL7 null, v2.xml, and Z-segments mean; how HL7 v2 differs from v3 and FHIR® — with worked examples from the hl7-rust workspace. Use whenever the task is understanding an HL7 message or vocabulary, not only when this repo's code is involved: "what does MSH mean", "explain HL7 segments/fields", "what is ER7", "what's the difference between HL7 v2 and v3", "what is the HL7 null", an unfamiliar pipe-delimited (`MSH|^~\&|...`) message, or an HL7 v2/v3 term used without explanation.
---

# HL7 concepts, terminology, and ideas

A general-purpose explainer for Health Level Seven (HL7®) itself — the
vocabulary, the encoding, and the ideas — not a how-to for any one tool.
Every example below is real output from the `hl7-rust` workspace, so it
doubles as a tour of that project, but the concepts apply to any HL7 v2
system. This skill is for people *using* HL7 or this workspace, not for
changing the workspace's own code — that is a separate, maintainer-facing
skill.

## HL7 is a family, not one standard

"HL7" names an organization and several unrelated standards it publishes.
They share little beyond the name and the problem — moving clinical data
between systems:

- **HL7 v2** — delimited text, releases 2.1 through 2.9, first published
  in the late 1980s and still the format most healthcare data actually
  moves in. Flexible to the point of being negotiable: two "v2" senders
  routinely disagree on details, which is why version- and dialect-aware
  tooling matters.
- **HL7 v3** — XML generated from one strict object model, the Reference
  Information Model (RIM). Traded v2's flexibility for rigor. V3
  *messaging* saw limited adoption; what did succeed, and still runs, is
  the Clinical Document Architecture (CDA®) and national registries built
  on the same model.
- **The HL7® FHIR® standard** — resources over HTTP, the current
  direction of travel, and a clean break from both v2 and v3's message
  shapes.

A "message", a "segment", and a "code" mean different things in each of
these — treat them as related but distinct vocabularies, not dialects of
one thing.

## ER7: the pipes and carets

ER7 (Encoding Rules version 7) is v2's traditional wire encoding — the
pipe-delimited text everyone pictures when they hear "HL7". Its delimiters
are not fixed by the standard; each message declares its own, in its first
line:

```
MSH|^~\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5
```

`MSH-1` is the character immediately after `MSH` — here `|`, the field
separator. `MSH-2` is the next four characters, `^~\&`, which are the
component, repetition, escape, and subcomponent separators, in that
order. Code that hardcodes those five characters will misread any sender
that chose differently — and senders do.

## The anatomy of a message

A message is a sequence of segments, one per line, each named by three
characters. A segment is a sequence of fields. A field may repeat, may
split into components, and a component may split into subcomponents —
four levels, four separators:

```
PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E
 ^  ^  ^                                       ^
 |  |  |                                       PID-5, an XPN (person name)
 |  |  PID-3, a CX (identifier), whose 4th component
 |  |  is itself an HD with subcomponents split by &
 |  PID-2, not sent
 PID-1, the set ID

Field        PID-3          separated by |
Repetition   PID-3[2]       separated by ~
Component    PID-3.4        separated by ^
Subcomponent PID-3.4.2      separated by &
```

## Names describe, paths address

Two vocabularies show up in every HL7 tool's output, and they are not
interchangeable:

- A **path** is an address: `PID-5.1` means segment `PID`, field 5,
  component 1. Query languages and edit operations take paths.
- A **name** is a description: `XPN.1` means the family-name component of
  an Extended Person Name. Trees, XML elements, and JSON keys use names,
  and a *dictionary* (below) is what supplies them.

## The dictionary

The dictionary is the per-release knowledge that turns positions into
meaning: which data type each field of each segment carries, which
components each composite type has, and what the message structures look
like. v2.5 is typically the most complete base; other releases are deltas
of it.

A dictionary doubles as a way to describe a vendor's own dialect —
state it as data (a JSON file, in `hl7-rust`'s case) rather than code, and
adding a local field becomes a configuration change:

```json
{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}
```

## Message structures, and MSH-9

`MSH-9` says what a message is: a message code (`ORU`), a trigger event
(`R01`), and, from v2.3.1 on, a message-structure id (`ORU_R01`). The
structure is a grammar over segments that says which ones group together
— an `ORU_R01` is a patient result containing an order observation
containing observations.

## The HL7 null is not an empty field

HL7 v2 distinguishes "I am not telling you anything about this field"
from "delete the value you currently hold." The second is the *explicit
null*, written as two double quotes:

```
PID|1||""||SMITH
      ^  ^
      |  the explicit HL7 null: "delete the value you have"
      not sent: "I am not telling you anything about this field"
```

In an update message that is the difference between leaving a patient's
address alone and erasing it — worth checking for explicitly in any code
that reads a field and finds it empty.

## Escape sequences

A value that needs to contain a delimiter escapes it, using whichever
character `MSH-2` declared as the escape character (conventionally `\`):

```
\F\   |     the field separator, as a value
\S\   ^     component separator
\T\   &     subcomponent separator
\R\   ~     repetition separator
\E\   \     the escape character itself
\X0A\ hex   an arbitrary byte
\.br\       a formatting command — often kept literally, not decoded
```

## v2.xml, and JSON mappings

HL7 also publishes an official XML encoding of v2, namespace
`urn:hl7-org:v2xml`. Every field, component, and subcomponent becomes an
element whose name carries its *position* as the number after the last
dot — `<PID.5>`, and inside it `<XPN.1>` when the dictionary knows the
type, or `<PID.5.1>` when it does not. That naming rule is why converting
XML *back* to ER7 needs no dictionary at all — the position is in the
name.

There is no official "v2.json". A JSON mapping such as the one
`hl7-rust`'s `hl7-2-from-er7-into-json` defines is one project's choice,
typically built to preserve everything v2.xml preserves while using
idiomatic JSON: real arrays for repetition, real `null` for the HL7 null,
and every scalar as a *string* — HL7 numeric text carries leading zeros,
explicit signs, and trailing precision that a JSON number would silently
destroy.

## HL7 v3: the RIM and the envelope

Where v2 is a set of message layouts, v3 is one object model — the
Reference Information Model — with six backbone classes (`Act`,
`Entity`, `Role`, `ActRelationship`, `Participation`, `RoleLink`) that
every domain payload is assembled from. Every v3 interaction shares a
three-level envelope:

```
Message                       level 1 — transport: sender, receiver, id
└── ControlAct                 level 2 — the real-world trigger event
    └── domain payload          level 3 — the interaction's own content
```

## Transports: MLLP and SOAP

A v2 message carries no length prefix and no self-delimiting syntax, so a
receiver reading a raw TCP socket cannot tell where one message stops and
the next begins. MLLP (the Minimal Lower Layer Protocol) is the answer,
and the entire protocol is three bytes:

```
<VT> message <FS><CR>
0x0B          0x1C 0x0D
```

No length, no checksum, no session, no negotiation, no encryption. MLLP
has no acknowledgement mechanism of its own either — the reply HL7 expects
is another HL7 message, an `ACK` whose `MSA-2` echoes the control ID of
the message being answered.

SOAP is the other transport: the same job over HTTP, with an envelope and
usually a WSDL — v2's exception and v3's historical norm.

## Z-segments, and degrading rather than rejecting

Any segment whose name begins with `Z` is local by definition — the
standard reserves the letter and says nothing about what's inside.
Nearly every real interface carries at least one, alongside fields past
the end of a published segment and data types no dictionary has ever
seen. A common, useful design choice (the one `hl7-rust` makes
throughout) is to *degrade rather than reject*: an unknown segment keeps
its positional values under a generic name, an unrecognized structure
renders flat instead of failing, and none of that trips strict
validation on its own. The cost is a lost typed *name*, never a lost
*value* and never a rejected message.

## Trying these ideas against a real message

`hl7-rust` ships a CLI for exactly this kind of exploration — pointing it
at an unfamiliar message and reading back paths, names, and structure:

```sh
hl7-v2 --paths samples/vendor.hl7      # every value, with its address
hl7-v2 --query PID-3 samples/vendor.hl7 # one field, by path
hl7-v2 --check samples/vendor.hl7       # does it parse and hold together?

hl7-2-from-er7-into-json samples/orm_o01.hl7   # ER7 -> the ideas above, as JSON
hl7-2-from-er7-into-xml  samples/orm_o01.hl7   # ER7 -> the official v2.xml
```

For the technical detail behind each command — every crate, every flag,
every conversion rule — see [`README.md`](../README.md) and each crate's
own `README.md` and `spec/index.md`, or the narrative version at
[hl7-rust.github.io](https://hl7-rust.github.io) (start with
[Concepts](https://hl7-rust.github.io/docs/concepts/), which this file
summarizes).

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
