[index](../index.md) → §2 Wire shapes

# §2 Wire shapes

This is the normative table: what each wrapper type must serialize as, and
must accept when deserializing. A change to any shape here is a breaking
change (see [§8](../08-versioning-and-compatibility/index.md), rule S10).

## 2.1 The table

Every struct wrapper is an object. Keys are listed in serialization order;
**bold** keys are required on deserialize (S5), the rest default (S4).

| Type | Keys | Values |
|------|------|--------|
| `Message` | `id`, `creationTime`, `interactionId`, `sender`, `receiver`, `controlAct` | `Ii`/`null`, string/`null`, `Ii`/`null`, `Element`/`null`, `Element`/`null`, `ControlAct`/`null` |
| `ControlAct` | **`classCode`**, **`moodCode`**, `code`, `domain` | string, string, `Cd`/`null`, `Element`/`null` |
| `Element` | **`name`**, `attributes`, `text`, `children` | string, object of strings (name order), string, array of `Element` |
| `Ii` | **`root`**, `extension` | string, string/`null` |
| `Cd` | **`code`**, `codeSystem`, `displayName` | string, string/`null`, string/`null` |
| `Ivl` | `value`, `low`, `high` | string/`null` each |
| `Pq` | `value`, `unit` | string/`null` each |
| `Ed` | `mediaType`, `representation`, `text` | string/`null` each |
| `NullFlavor` | *(a bare string)* | the code: `"NI"`, `"UNK"`, `"ASKU"`, `"NASK"`, `"NAV"`, `"NA"`, `"OTH"`, or any other |
| `Act` | **`classCode`**, **`moodCode`**, `id`, `code`, `statusCode`, `effectiveTime`, `text` | string, string, array of `Ii`, `Cd`/`null`, `Cd`/`null`, string/`null`, string/`null` |
| `Entity` | **`classCode`**, `determinerCode`, `id`, `code`, `name` | string, string/`null`, array of `Ii`, `Cd`/`null`, string/`null` |
| `Role` | **`classCode`**, `id`, `code`, `statusCode`, `effectiveTime` | string, array of `Ii`, `Cd`/`null`, `Cd`/`null`, string/`null` |
| `Participation` | **`typeCode`**, `time`, `functionCode` | string, string/`null`, `Cd`/`null` |
| `ActRelationship` | **`typeCode`**, `inversionInd` | string, boolean/`null` |
| `RoleLink` | **`typeCode`** | string |

## 2.2 Rules

- **S3**: keys are the `hl7-3` field names converted to lowerCamelCase:
  `class_code` → `classCode`, `interaction_id` → `interactionId`,
  `control_act` → `controlAct`. For every field that corresponds to an
  HL7® v3 XML attribute or element, this is that attribute's or element's
  own name, so the JSON cross-references against the XML and the standard
  without a translation table. Every key is serialized every time, in
  declaration order — `"code": null` and `"id": []` included — so the
  schema a consumer sees is fixed, not dependent on which fields a given
  message happened to fill.

- **S4**: on deserialize, an `Option` key may be absent or `null` and
  reads `None`; a `Vec` key may be absent and reads empty; `Element`'s
  `attributes` and `text` may be absent and read empty. This mirrors
  `hl7-3`'s own reading rule (its spec §5.2, "nothing here fails on a
  missing wrapper"): a hand-written fixture states what it has, and the
  rest is absent.

- **S5**: the fields `hl7-3` types as plain `String` — the codes that say
  *what kind of thing this is* (`classCode`, `moodCode`, `typeCode`), the
  identifier's `root`, the coded value's `code`, and an element's `name`
  — are required. An act with no class code or an identifier with no root
  is not a value `hl7-3` would produce, so it is not one this crate reads.

- **S6**: `NullFlavor` serializes through `hl7_3::NullFlavor::as_code` and
  deserializes through `hl7_3::NullFlavor::parse`, which maps the seven
  named codes to their variants and anything else to
  `Unrecognized(code)`. Deserialization therefore never fails on a
  string: an unknown code is data, as it is when `hl7-3` reads it off an
  attribute (its spec §3.6).

- **S7**: `Element` serializes its four public fields. `attributes` is an
  object because `hl7-3` holds it as a `BTreeMap`, so it is already in
  name order and already the shape JSON readers expect for a map;
  `children` recurses. An element's `text` and `children` are both
  present because some documents use both (`hl7-2-xml-lite-helper`'s
  spec §3.3).

- **S15**: all fourteen struct wrappers are produced by one
  `macro_rules!`, `object_wrapper!`, in `src/object.rs`, from a field list
  that names each key and its mode (`req`, `opt`, `dflt`, `reqw`, `optw`,
  `vecw`). A shape rule is therefore implemented once, and the strictness
  flag of [§11](../11-strict-mode/index.md) reaches every nested value
  through the same three seed types. This is a `macro_rules!`, not a
  derive, so it adds no proc-macro dependency (S1) and produces exactly
  the visitor serde's own manual-implementation guide shows.

## 2.3 Why not derive

`#[derive(Serialize)]` cannot be placed on a foreign type, and
`#[serde(remote = ...)]` would need the `serde_derive` proc-macro crate —
a third dependency, compiled for every downstream build, to produce what
some two hundred lines of `macro_rules!` produce with none. The shapes here
*are* close to what a derive with `rename_all = "camelCase"` would give;
the reason to hand-write them is the dependency surface, and the reason
to write them with a local macro is that fourteen hand-written copies of
the same visitor would drift.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven
International and their use of these trademarks does not constitute an
endorsement by HL7.
