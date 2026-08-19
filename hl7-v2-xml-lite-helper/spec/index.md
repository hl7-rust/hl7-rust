# Specification: hl7-v2-xml-lite-helper

This is the single source of truth for what this crate does and how. It
describes observable behavior, not implementation details. `README.md`
summarizes this document — if the two disagree, this document wins.

Every rule below is exercised by a test in `tests/integration.rs`. A change
to this document that isn't backed by a test, or a code change that isn't
reflected here, is a bug.

## 1. Scope

**In scope:** reading a well-formed XML document into a tree of elements,
each with a name, attributes, text, and children; and escaping text for
writing one.

**Out of scope, deliberately:** validation, schemas, DTDs, entity
declarations, namespace resolution, streaming, XPath, mutation, and
serialization of a tree. Anything that needs those needs a different crate,
and saying so here is cheaper than half-implementing them.

## 2. Why it exists, and why the name

Three crates in this family each carried their own reader for this same
subset, differing only in which half they discarded:
`hl7-v2-from-xml-into-er7` kept text and dropped attributes,
`hl7-v2-from-xsd-into-json-dictionary` kept attributes and dropped text, and
`hl7-v2-soap` kept text and dropped attributes again. Keeping all four
things satisfies all three, and healthcare integration code that gets
audited is better with one parser to read than three.

The name places it in the family rather than claiming a general one. The
code is not HL7-specific and could serve anything, but it is maintained for
these three callers, its trade-offs (§3.2, §6) are chosen for their
documents, and a neutral name would invite users it is not meant for.

## 3. Reading (`parse`)

### 3.1 Structure

`parse` returns the root element. Anything before it — the XML declaration,
comments, a `DOCTYPE`, in any order and repetition — is skipped, and
anything after its closing tag is ignored, which is how every reader treats
a trailing newline. A document with no root element is `NoRootElement`.

Within an element: comments, processing instructions and a `DOCTYPE` are
skipped; CDATA delimiters are skipped and their content kept as text;
child elements are collected in document order.

### 3.2 Names and prefixes

Prefixes are **not resolved**. `local_name` is the part after the first
colon, and `local_name`, `attribute`, `child`, `children_named`, `find` and
`text_at` all match on it. `xmlns` declarations are ordinary attributes and
are not interpreted.

The consequence, stated plainly: two elements from different namespaces with
the same local name are indistinguishable. This crate is for documents where
that cannot happen or does not matter.

### 3.3 Text

Text is entity-decoded and accumulated across the element's content.
**Whitespace-only text is dropped when the element has children**, because
it is indentation. Text in a leaf is kept byte for byte, including leading
and trailing spaces, because a value may depend on them.

Mixed content — text *and* children — keeps both. This crate does not decide
what that means; a caller that never expects mixed content should say so
itself rather than rely on the reader to discard it.

`text_opt` reports empty text as `None`. `<a/>` and `<a></a>` are the same
element: nothing in the documents this crate is for depends on the
difference.

### 3.4 Attributes

Read from either quote style, entity-decoded, and stored by name as written
in a `BTreeMap`, so iteration is in name order and repeated names collapse
to the last (a repeated attribute is not well-formed XML). `attribute`
matches by local name.

### 3.5 Entities

The five predefined entities and numeric character references (`&#65;`,
`&#x41;`) decode. Anything else that looks like an entity is **kept
literally** rather than rejected — leniency chosen because the alternative
is refusing a document over a fragment nobody was going to read.

### 3.6 Errors

| Error | Means |
|---|---|
| `NoRootElement` | nothing but prolog and whitespace |
| `Unclosed(name)` | input ended inside an element |
| `Mismatched { open, close }` | a closing tag names a different element |
| `Malformed(reason, offset)` | anything else, with a byte offset |

There is no recovery: a document that is not well-formed is an error, not a
best guess.

## 4. Writing (`escape`)

`escape` replaces all five of `& < > " '`. All five, not the three that
element content strictly needs, because a value may be quoted into an
attribute instead and this crate does not get to choose.

## 5. Dependencies

None, and it stays that way. A crate whose argument is that it is small
enough to audit cannot have dependencies that also need auditing.

## 6. What this is not

It is not fast. It allocates a `String` for every name, value and text node,
and it copies text it could borrow. That is the correct trade for documents
of the size this reads — an envelope, a schema, a message — and the wrong
one for a gigabyte of XML, which is a reason to use a different crate rather
than to complicate this one.
