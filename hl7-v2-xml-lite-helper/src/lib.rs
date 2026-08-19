//! The small, dependency-free XML reader the `hl7-2` crates share.
//!
//! The name says what it is for. Nothing here is HL7-specific, but the
//! crate is scoped to serve `hl7-v2-soap`, `hl7-v2-from-xml-into-er7` and
//! `hl7-v2-from-xsd-into-json-dictionary`, and every trade-off below is
//! chosen for the documents those read.
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
//! let root = hl7_v2_xml_lite_helper::parse(
//!     r#"<order id="7"><item qty="2">widget</item></order>"#,
//! )?;
//! assert_eq!(root.attribute("id"), Some("7"));
//! let item = root.child("item").unwrap();
//! assert_eq!(item.attribute("qty"), Some("2"));
//! assert_eq!(item.text, "widget");
//! # Ok::<(), hl7_v2_xml_lite_helper::Error>(())
//! ```
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
    /// let root = hl7_v2_xml_lite_helper::parse(
    ///     "<PID><PID.3><CX.1>a</CX.1></PID.3><PID.3><CX.4>b</CX.4></PID.3></PID>",
    /// )?;
    /// assert_eq!(root.text_at(&["PID.3", "CX.4"]), Some("b"));
    /// # Ok::<(), hl7_v2_xml_lite_helper::Error>(())
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

struct Cursor<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Cursor<'a> {
        Cursor { text, position: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.text[self.position..]
    }

    fn at_end(&self) -> bool {
        self.position >= self.text.len()
    }

    fn starts_with(&self, pattern: &str) -> bool {
        self.rest().starts_with(pattern)
    }

    fn advance(&mut self, bytes: usize) {
        self.position += bytes;
    }

    fn skip_whitespace(&mut self) {
        let trimmed = self.rest().trim_start();
        self.position = self.text.len() - trimmed.len();
    }

    fn skip_prolog(&mut self) -> Result<(), Error> {
        loop {
            self.skip_whitespace();
            if self.starts_with("<?") {
                self.skip_through("?>")?;
            } else if self.starts_with("<!--") {
                self.skip_through("-->")?;
            } else if self.starts_with("<!") {
                self.skip_through(">")?;
            } else {
                return Ok(());
            }
        }
    }

    fn skip_through(&mut self, end: &str) -> Result<(), Error> {
        match self.rest().find(end) {
            Some(index) => {
                self.advance(index + end.len());
                Ok(())
            }
            None => Err(Error::Malformed(
                format!("unterminated {end:?}"),
                self.position,
            )),
        }
    }

    fn parse_element(&mut self) -> Result<Element, Error> {
        if !self.starts_with("<") {
            return Err(Error::Malformed("expected '<'".into(), self.position));
        }
        self.advance(1);
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
        let end = self
            .rest()
            .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .ok_or_else(|| Error::Malformed("unterminated tag".into(), self.position))?;
        if end == 0 {
            return Err(Error::Malformed("empty element name".into(), self.position));
        }
        let name = self.rest()[..end].to_string();
        self.advance(end);
        Ok(name)
    }

    /// Read `name="value"` pairs up to the tag's close. Returns the
    /// attributes and whether the tag was self-closing.
    fn read_attributes(&mut self) -> Result<(BTreeMap<String, String>, bool), Error> {
        let mut attributes = BTreeMap::new();
        loop {
            self.skip_whitespace();
            if self.starts_with("/>") {
                self.advance(2);
                return Ok((attributes, true));
            }
            if self.starts_with(">") {
                self.advance(1);
                return Ok((attributes, false));
            }
            if self.at_end() {
                return Err(Error::Malformed("unterminated tag".into(), self.position));
            }
            let name_end = self
                .rest()
                .find(|c: char| c.is_whitespace() || c == '=')
                .ok_or_else(|| Error::Malformed("malformed attribute".into(), self.position))?;
            let name = self.rest()[..name_end].to_string();
            self.advance(name_end);
            self.skip_whitespace();
            if !self.starts_with("=") {
                return Err(Error::Malformed(
                    "attribute without a value".into(),
                    self.position,
                ));
            }
            self.advance(1);
            self.skip_whitespace();
            let quote = self
                .rest()
                .chars()
                .next()
                .filter(|&c| c == '"' || c == '\'')
                .ok_or_else(|| {
                    Error::Malformed("unquoted attribute value".into(), self.position)
                })?;
            self.advance(1);
            let close = self
                .rest()
                .find(quote)
                .ok_or_else(|| Error::Malformed("unterminated attribute".into(), self.position))?;
            let value = decode(&self.rest()[..close]);
            self.advance(close + 1);
            attributes.insert(name, value);
        }
    }

    fn parse_content(&mut self, open_name: &str) -> Result<(String, Vec<Element>), Error> {
        let mut text = String::new();
        let mut children = Vec::new();
        loop {
            let next = self
                .rest()
                .find('<')
                .ok_or_else(|| Error::Unclosed(open_name.to_string()))?;
            if next > 0 {
                text.push_str(&decode(&self.rest()[..next]));
                self.advance(next);
            }
            if self.starts_with("</") {
                self.advance(2);
                let close_name = self.read_name()?;
                self.skip_whitespace();
                if !self.starts_with(">") {
                    return Err(Error::Malformed(
                        "unterminated close tag".into(),
                        self.position,
                    ));
                }
                self.advance(1);
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
                self.skip_through("-->")?;
                continue;
            }
            if self.starts_with("<?") {
                self.skip_through("?>")?;
                continue;
            }
            if self.starts_with("<![CDATA[") {
                self.advance("<![CDATA[".len());
                let end = self
                    .rest()
                    .find("]]>")
                    .ok_or_else(|| Error::Malformed("unterminated CDATA".into(), self.position))?;
                text.push_str(&self.rest()[..end]);
                self.advance(end + "]]>".len());
                continue;
            }
            children.push(self.parse_element()?);
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
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        rest = &rest[index..];
        let Some(end) = rest.find(';') else {
            out.push_str(rest);
            return out;
        };
        let entity = &rest[1..end];
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
        match decoded {
            Some(c) => out.push(c),
            None => out.push_str(&rest[..=end]),
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}
