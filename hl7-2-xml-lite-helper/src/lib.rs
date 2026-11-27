//! The small, dependency-free XML reader the `hl7-2` crates share.
//!
//! The name says what it is for. Nothing here is HL7®-specific, but the
//! crate is scoped to serve its five callers in this family —
//! `hl7-2-soap`, `hl7-2-from-xml-into-er7`,
//! `hl7-2-from-xsd-into-json-dictionary`, `hl7-3`, and `hl7-3-soap` — and
//! every trade-off below is chosen for the documents those read.
//!
//! It is not a general-purpose parser and does not try to be. It reads the
//! subset that carries meaning in a data document — elements, attributes,
//! text, and nesting — and skips the rest: comments, processing
//! instructions, a `DOCTYPE`, and the XML declaration. There is no
//! validation, no schema, no DTD, no namespace resolution, and no streaming.
//!
//! What it is for: reading a document produced by a system you are talking
//! to, where you know which elements you want and simply need them out. A
//! SOAP envelope, an XML Schema, an HL7 v2.xml message. For anything where
//! the document is untrusted, unbounded, or genuinely unknown, use a real
//! parser.
//!
//! ```
//! let root = hl7_2_xml_lite_helper::parse(
//!     r#"<order id="7"><item qty="2">widget</item></order>"#,
//! )?;
//! assert_eq!(root.attribute("id"), Some("7"));
//! let item = root.child("item").unwrap();
//! assert_eq!(item.attribute("qty"), Some("2"));
//! assert_eq!(item.text, "widget");
//! # Ok::<(), hl7_2_xml_lite_helper::Error>(())
//! ```
//!
//! # Trademarks
//!
//! HL7®, and FHIR® are the registered trademarks of Health Level Seven
//! International and their use of these trademarks does not constitute an
//! endorsement by HL7.
//!
//! # Namespace prefixes are ignored, not resolved
//!
//! Elements and attributes are matched on their **local name**, so
//! `soapenv:Body`, `soap:Body`, `SOAP-ENV:Body` and `Body` are the same
//! element. This is the single most important thing to understand about
//! this crate, and it is a deliberate trade: the prefix is chosen by
//! whoever serialized the document, and code that insists on one prefix
//! rejects valid documents from every other tool. A document that binds the
//! same prefix to two namespaces, or relies on the distinction between two
//! namespaces that happen to use the same local names, will be misread.
//! Reach for a namespace-aware parser there.
//!
//! See `spec/index.md` for the exact rules (source of truth).

// No `unsafe` anywhere in this crate, enforced rather than merely true:
// `forbid` cannot be lifted by an `allow` further down, so this is a
// property a reviewer can rely on without reading the sources. See
// SECURITY.md.
#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

use std::collections::BTreeMap;
use std::fmt;

/// One element: its name, attributes, text, and children.
///
/// An element has both `text` and `children` because some documents use
/// both, and dropping either would make this reader useless for one of the
/// documents it is meant to read. Whitespace-only text beside children is
/// dropped, because it is indentation rather than content (§3.3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Element {
    /// The tag as written, prefix included (`soapenv:Body`).
    pub name: String,
    /// Attributes, keyed by name as written, in name order.
    pub attributes: BTreeMap<String, String>,
    /// Text content, entity-decoded.
    pub text: String,
    /// Child elements, in document order.
    pub children: Vec<Element>,
}

impl Element {
    /// The tag without its namespace prefix: `Body` for `soapenv:Body`.
    #[must_use]
    pub fn local_name(&self) -> &str {
        local_name(&self.name)
    }

    /// An attribute's value, by local name, ignoring any prefix.
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| local_name(key) == name)
            .map(|(_, value)| value.as_str())
    }

    /// This element's text, or `None` when it has none.
    ///
    /// The distinction between empty and absent is the same one here; an
    /// element with no text and one with empty text are not distinguishable
    /// in XML without preserving the difference between `<a/>` and `<a></a>`,
    /// which no document this crate is for depends on.
    #[must_use]
    pub fn text_opt(&self) -> Option<&str> {
        Some(self.text.as_str()).filter(|text| !text.is_empty())
    }

    /// The first direct child with this local name.
    #[must_use]
    pub fn child<'a>(&'a self, name: &str) -> Option<&'a Element> {
        self.children
            .iter()
            .find(|child| child.local_name() == name)
    }

    /// Direct children with this local name.
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> + 'a {
        self.children
            .iter()
            .filter(move |child| child.local_name() == name)
    }

    /// The first descendant with this local name, this element included, in
    /// document order.
    #[must_use]
    pub fn find<'a>(&'a self, name: &str) -> Option<&'a Element> {
        if self.local_name() == name {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(name))
    }

    /// Follow a chain of local names down from here and return the first
    /// non-blank text found at the end of it.
    ///
    /// Every element at each step is followed, not only the first: a
    /// repeating field puts several elements of the same name side by side,
    /// and the value wanted may be under any of them.
    ///
    /// ```
    /// let root = hl7_2_xml_lite_helper::parse(
    ///     "<PID><PID.3><CX.1>a</CX.1></PID.3><PID.3><CX.4>b</CX.4></PID.3></PID>",
    /// )?;
    /// assert_eq!(root.text_at(&["PID.3", "CX.4"]), Some("b"));
    /// # Ok::<(), hl7_2_xml_lite_helper::Error>(())
    /// ```
    #[must_use]
    pub fn text_at<'a>(&'a self, path: &[&str]) -> Option<&'a str> {
        let mut level: Vec<&Element> = vec![self];
        for step in path {
            let mut next: Vec<&Element> = Vec::new();
            for element in level {
                next.extend(
                    element
                        .children
                        .iter()
                        .filter(|child| child.local_name() == *step),
                );
            }
            if next.is_empty() {
                return None;
            }
            level = next;
        }
        level
            .into_iter()
            .map(|element| element.text.trim())
            .find(|text| !text.is_empty())
    }
}

/// The local part of a possibly-prefixed XML name.
#[must_use]
pub fn local_name(name: &str) -> &str {
    match name.split_once(':') {
        Some((_, local)) => local,
        None => name,
    }
}

/// Why a document could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The document has no root element.
    NoRootElement,
    /// The input ended before an open element was closed.
    Unclosed(String),
    /// A closing tag did not match the element it was meant to close.
    Mismatched {
        /// The name the open tag gave.
        open: String,
        /// The name the close tag gave.
        close: String,
    },
    /// The input is not well-formed XML; carries a reason and a byte offset.
    Malformed(String, usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoRootElement => write!(f, "no root element found"),
            Error::Unclosed(name) => write!(f, "element <{name}> is never closed"),
            Error::Mismatched { open, close } => write!(f, "<{open}> is closed by </{close}>"),
            Error::Malformed(reason, at) => write!(f, "{reason} at byte {at}"),
        }
    }
}

impl std::error::Error for Error {}

/// Parse a document and return its root element.
///
/// Anything before the root — the XML declaration, comments, a `DOCTYPE` —
/// is skipped, and anything after the root's closing tag is ignored, which
/// is how every reader treats a trailing newline.
///
/// # Errors
///
/// [`Error`] when the document is not well formed: no root element, an
/// element left open, a mismatched closing tag, or anything else, with the
/// byte offset where reading gave up. There is no recovery; see
/// `spec/index.md` §3.6.
pub fn parse(xml: &str) -> Result<Element, Error> {
    let mut cursor = Cursor::new(xml.strip_prefix('\u{feff}').unwrap_or(xml));
    cursor.skip_prolog()?;
    cursor.skip_whitespace();
    if cursor.at_end() {
        return Err(Error::NoRootElement);
    }
    cursor.parse_element()
}

/// Escape text for element content or an attribute value.
///
/// All five predefined entities, because a value may be quoted into either
/// position and this crate does not get to choose the reader at the far end.
#[must_use]
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// How deeply elements may nest before reading gives up.
///
/// Reading is recursive, so nesting depth is stack depth: without a limit a
/// small document of nothing but open tags aborts the process with a stack
/// overflow, which a library must never do to its caller. The documents
/// this crate is for nest a dozen levels at the outside — a v2.xml message
/// reaches subcomponents in six, a SOAP envelope in about the same — so
/// this is far above anything real and still far below the stack.
const MAX_DEPTH: usize = 256;

struct Cursor<'a> {
    text: &'a str,
    /// The unread part of `text`, always a suffix of it. Holding the suffix
    /// rather than a byte offset means reading never does offset arithmetic
    /// or slices at an index (RCA-LEN-2): every step is a `strip_prefix`,
    /// `split_once` or `split_at_checked` on what is left.
    rest: &'a str,
    depth: usize,
}

impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Cursor<'a> {
        Cursor {
            text,
            rest: text,
            depth: 0,
        }
    }

    /// The byte offset of the unread part, for error messages.
    fn position(&self) -> usize {
        // `rest` is a suffix of `text`, so the difference is the bytes read.
        self.text.len().abs_diff(self.rest.len())
    }

    fn malformed(&self, what: &str) -> Error {
        Error::Malformed(what.into(), self.position())
    }

    fn at_end(&self) -> bool {
        self.rest.is_empty()
    }

    fn starts_with(&self, pattern: &str) -> bool {
        self.rest.starts_with(pattern)
    }

    /// Consume `prefix` if the unread text starts with it.
    fn eat(&mut self, prefix: &str) -> bool {
        match self.rest.strip_prefix(prefix) {
            Some(tail) => {
                self.rest = tail;
                true
            }
            None => false,
        }
    }

    /// Consume and return the unread text before the first character that
    /// matches `stop`, or `None` (consuming nothing) when none does.
    fn take_until(&mut self, stop: impl Fn(char) -> bool) -> Option<&'a str> {
        let index = self.rest.find(stop)?;
        // `find` returns a character boundary, so this split always succeeds.
        let (head, tail) = self.rest.split_at_checked(index)?;
        self.rest = tail;
        Some(head)
    }

    /// Consume up to and including `end`, returning what came before it.
    fn take_through(&mut self, end: &str) -> Result<&'a str, Error> {
        let (head, tail) = self
            .rest
            .split_once(end)
            .ok_or_else(|| self.malformed(&format!("unterminated {end:?}")))?;
        self.rest = tail;
        Ok(head)
    }

    fn skip_whitespace(&mut self) {
        self.rest = self.rest.trim_start();
    }

    fn skip_prolog(&mut self) -> Result<(), Error> {
        loop {
            self.skip_whitespace();
            if self.starts_with("<?") {
                self.take_through("?>")?;
            } else if self.starts_with("<!--") {
                self.take_through("-->")?;
            } else if self.starts_with("<!") {
                self.take_through(">")?;
            } else {
                return Ok(());
            }
        }
    }

    fn parse_element(&mut self) -> Result<Element, Error> {
        if !self.starts_with("<") {
            return Err(self.malformed("expected '<'"));
        }
        if self.depth >= MAX_DEPTH {
            return Err(self.malformed(&format!("elements nested more than {MAX_DEPTH} deep")));
        }
        self.eat("<");
        let name = self.read_name()?;
        let (attributes, self_closing) = self.read_attributes()?;
        if self_closing {
            return Ok(Element {
                name,
                attributes,
                text: String::new(),
                children: Vec::new(),
            });
        }
        let (text, children) = self.parse_content(&name)?;
        Ok(Element {
            name,
            attributes,
            text,
            children,
        })
    }

    /// An XML name: everything up to whitespace, `/`, or `>`.
    fn read_name(&mut self) -> Result<String, Error> {
        let name = self
            .take_until(|c| c.is_whitespace() || c == '/' || c == '>')
            .ok_or_else(|| self.malformed("unterminated tag"))?;
        if name.is_empty() {
            return Err(self.malformed("empty element name"));
        }
        Ok(name.to_string())
    }

    /// Read `name="value"` pairs up to the tag's close. Returns the
    /// attributes and whether the tag was self-closing.
    fn read_attributes(&mut self) -> Result<(BTreeMap<String, String>, bool), Error> {
        let mut attributes = BTreeMap::new();
        loop {
            self.skip_whitespace();
            if self.eat("/>") {
                return Ok((attributes, true));
            }
            if self.eat(">") {
                return Ok((attributes, false));
            }
            if self.at_end() {
                return Err(self.malformed("unterminated tag"));
            }
            let name = self
                .take_until(|c| c.is_whitespace() || c == '=')
                .ok_or_else(|| self.malformed("malformed attribute"))?
                .to_string();
            self.skip_whitespace();
            if !self.eat("=") {
                return Err(self.malformed("attribute without a value"));
            }
            self.skip_whitespace();
            let (quote, quoted) = ['"', '\'']
                .into_iter()
                .find_map(|quote| self.rest.strip_prefix(quote).map(|tail| (quote, tail)))
                .ok_or_else(|| self.malformed("unquoted attribute value"))?;
            self.rest = quoted;
            let (raw, tail) = self
                .rest
                .split_once(quote)
                .ok_or_else(|| self.malformed("unterminated attribute"))?;
            self.rest = tail;
            attributes.insert(name, decode(raw));
        }
    }

    fn parse_content(&mut self, open_name: &str) -> Result<(String, Vec<Element>), Error> {
        let mut text = String::new();
        let mut children = Vec::new();
        loop {
            let before = self
                .take_until(|c| c == '<')
                .ok_or_else(|| Error::Unclosed(open_name.to_string()))?;
            if !before.is_empty() {
                text.push_str(&decode(before));
            }
            if self.eat("</") {
                let close_name = self.read_name()?;
                self.skip_whitespace();
                if !self.eat(">") {
                    return Err(self.malformed("unterminated close tag"));
                }
                if close_name != open_name {
                    return Err(Error::Mismatched {
                        open: open_name.to_string(),
                        close: close_name,
                    });
                }
                // Whitespace between child elements is layout, not content.
                if !children.is_empty() && text.trim().is_empty() {
                    text.clear();
                }
                return Ok((text, children));
            }
            if self.starts_with("<!--") {
                self.take_through("-->")?;
                continue;
            }
            if self.starts_with("<?") {
                self.take_through("?>")?;
                continue;
            }
            if self.eat("<![CDATA[") {
                let (data, tail) = self
                    .rest
                    .split_once("]]>")
                    .ok_or_else(|| self.malformed("unterminated CDATA"))?;
                self.rest = tail;
                text.push_str(data);
                continue;
            }
            // RCA-EXC-2: this element passed `depth < MAX_DEPTH` in
            // `parse_element`, so the increment is at most `MAX_DEPTH` and the
            // decrement undoes it.
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "RCA-EXC-2: depth is below MAX_DEPTH here, proof above"
            )]
            let child = {
                self.depth += 1;
                let child = self.parse_element();
                self.depth -= 1;
                child
            };
            children.push(child?);
        }
    }
}

/// Decode the five predefined entities and numeric character references.
/// Anything unrecognized is kept literally rather than rejected.
fn decode(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((before, after_ampersand)) = rest.split_once('&') {
        out.push_str(before);
        let Some((entity, after)) = after_ampersand.split_once(';') else {
            out.push('&');
            out.push_str(after_ampersand);
            return out;
        };
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => entity
                .strip_prefix('#')
                .and_then(|number| match number.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => number.parse().ok(),
                })
                .and_then(char::from_u32),
        };
        if let Some(c) = decoded {
            out.push(c);
        } else {
            out.push('&');
            out.push_str(entity);
            out.push(';');
        }
        rest = after;
    }
    out.push_str(rest);
    out
}
