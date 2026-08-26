# hl7-2-xml-lite-helper

> HL7® is the registered trademark of Health Level Seven International, and we are requesting permission to use it here. Use of the HL7 trademark does not constitute endorsement of this library by HL7.

The small, dependency-free XML reader the `hl7-2` crates share.

It reads the subset that carries meaning in a data document — elements,
attributes, text, and nesting — and skips the rest. No validation, no
schema, no DTD, no namespace resolution, no streaming.

The name says what it is for. Nothing in the code is HL7-specific, but the
crate is scoped to serve `hl7-2-soap`, `hl7-2-from-xml-into-er7` and
`hl7-2-from-xsd-into-json-dictionary`, and its trade-offs are chosen for
the documents those read. It is not offered as a general-purpose parser,
and it does not claim a general-purpose name.

```rust
let xml = r#"<order id="7"><item qty="2">widget</item></order>"#;
let root = hl7_2_xml_lite_helper::parse(xml)?;
assert_eq!(root.attribute("id"), Some("7"));
assert_eq!(root.child("item").unwrap().text, "widget");
# Ok::<(), hl7_2_xml_lite_helper::Error>(())
```

## What it is for

Reading a document produced by a system you are talking to, where you know
which elements you want and simply need them out: a SOAP envelope, an XML
Schema, an HL7 v2.xml message. It exists because three crates in this
family had each written their own version of exactly this, and three copies
of a parser is three places for a bug.

**Not** for untrusted, unbounded, or genuinely unknown documents, and not
for anyone outside this family who wants a small XML reader. Use
`quick-xml` or `roxmltree` there — they are better at it and they are
maintained for that purpose.

## Namespace prefixes are ignored, not resolved

This is the single most important thing to understand about this crate.
Elements and attributes are matched on their **local name**, so
`soapenv:Body`, `soap:Body`, `SOAP-ENV:Body` and `Body` are the same
element.

It is a deliberate trade. The prefix is chosen by whoever serialized the
document, and code that insists on one prefix rejects valid documents from
every other tool — which is the single most common way a working SOAP
integration breaks when the other end changes stack. The cost is that a
document relying on the distinction between two namespaces that share local
names will be misread. Reach for a namespace-aware parser there.

## Finding things

```rust
let root = hl7_2_xml_lite_helper::parse(
    "<PID><PID.3><CX.1>a</CX.1></PID.3><PID.3><CX.4><HD.1>NHS</HD.1></CX.4></PID.3></PID>",
)?;

// A path down to the first non-blank value, following *every* branch —
// a repeating field puts several elements of the same name side by side.
assert_eq!(root.text_at(&["PID.3", "CX.4", "HD.1"]), Some("NHS"));

// Or walk it yourself.
assert_eq!(root.child("PID.3").unwrap().child("CX.1").unwrap().text, "a");
assert_eq!(root.children_named("PID.3").count(), 2);
assert_eq!(root.find("HD.1").unwrap().text, "NHS");   // first descendant
# Ok::<(), hl7_2_xml_lite_helper::Error>(())
```

## What is skipped, and what is not

**Skipped:** the XML declaration, comments, processing instructions, and a
`DOCTYPE`, wherever they appear. **Kept:** CDATA *content*, as text.
Entities — the five predefined ones and numeric character references —
decode; anything else that looks like an entity is kept literally rather
than rejected.

Whitespace-only text beside child elements is dropped, because it is
indentation. Text in a leaf is kept exactly as it arrived, because a leading
or trailing space can be part of a value.

## Writing

`escape` covers all five predefined entities, so a value is safe in element
content or in an attribute.

## Dependencies

None, and staying that way: a crate whose whole argument is that it is small
enough to read cannot have dependencies you also have to read.

## Specification

`spec/index.md` is the source of truth. If it and the README disagree, the
spec wins.

## License

Licensed under any of MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only, or
GPL-3.0-only, at your option.
