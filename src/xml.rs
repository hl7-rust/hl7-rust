//! A minimal, dependency-free XML reader for the documents this crate
//! consumes.
//!
//! This is not a general-purpose XML parser. It reads exactly the subset
//! that v2.xml (and XML generally) uses to shape a document tree — one root
//! element, nested elements, character data, the five predefined entities,
//! and numeric character references — and is lenient about the rest:
//! attributes are recognized and skipped (this crate has no use for
//! `xmlns`), and the XML declaration, comments, and a `DOCTYPE` are skipped
//! wherever they may appear before the root element. See `spec/index.md`
//! §2 for exactly what is and is not required of the input.

use std::fmt;

/// One node of the parsed tree: an element, either a container holding more
/// elements or a leaf holding text — never both, and an element that is
/// empty (self-closing, or with no content) is a leaf with no text.
///
/// This mirrors the `Node` shape the sibling `hl7-2-5-to-xml-using-rust`
/// crate renders from, so the reconstruction logic in `src/reconstruct.rs`
/// reads as the inverse of that crate's `src/xml.rs`.
#[derive(Debug, Clone)]
pub struct Node {
    /// The element's tag name (attributes are not retained).
    pub name: String,
    /// Text content, for a childless element that has some; `None` for a
    /// childless element with none (the reconstructed explicit null) and
    /// for any element with children.
    pub text: Option<String>,
    /// Child elements, in document order.
    pub kids: Vec<Node>,
}

/// What can go wrong reading the XML itself (as opposed to what it means as
/// HL7, which is [`crate::Hl7Error`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlError {
    /// The document has no root element at all.
    NoRootElement,
    /// The input ended before an open element was closed; carries the
    /// element's name.
    UnclosedElement(String),
    /// A closing tag's name didn't match the element it was meant to close.
    MismatchedClose {
        /// The name the open tag gave.
        open: String,
        /// The name the close tag gave.
        close: String,
    },
    /// The input isn't well-formed XML at all; carries a short reason and
    /// the byte offset where parsing gave up.
    Malformed(String, usize),
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::NoRootElement => write!(f, "no root element found"),
            XmlError::UnclosedElement(name) => write!(f, "element <{name}> is never closed"),
            XmlError::MismatchedClose { open, close } => {
                write!(f, "<{open}> is closed by </{close}>")
            }
            XmlError::Malformed(reason, at) => write!(f, "{reason} at byte {at}"),
        }
    }
}

impl std::error::Error for XmlError {}

/// Parse `xml` and return its root element.
///
/// Anything before the root element — a `<?xml ... ?>` declaration,
/// comments, a `DOCTYPE` — is skipped; anything after the root element's
/// closing tag is ignored, matching how every other XML reader treats a
/// trailing newline.
pub fn parse_document(xml: &str) -> Result<Node, XmlError> {
    let mut cursor = Cursor::new(xml.strip_prefix('\u{feff}').unwrap_or(xml));
    cursor.skip_prolog()?;
    cursor.skip_ws();
    if cursor.at_end() {
        return Err(XmlError::NoRootElement);
    }
    cursor.parse_element()
}

struct Cursor<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(s: &'a str) -> Self {
        Cursor { s, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.s[self.pos..]
    }

    fn at_end(&self) -> bool {
        self.pos >= self.s.len()
    }

    fn starts_with(&self, pat: &str) -> bool {
        self.rest().starts_with(pat)
    }

    fn advance(&mut self, bytes: usize) {
        self.pos += bytes;
    }

    fn skip_ws(&mut self) {
        let trimmed = self.rest().trim_start();
        self.pos = self.s.len() - trimmed.len();
    }

    /// Skip everything before the root element: the `<?xml ... ?>`
    /// declaration, comments, and a `DOCTYPE`, in any order and repetition.
    fn skip_prolog(&mut self) -> Result<(), XmlError> {
        loop {
            self.skip_ws();
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

    fn skip_through(&mut self, end: &str) -> Result<(), XmlError> {
        match self.rest().find(end) {
            Some(i) => {
                self.advance(i + end.len());
                Ok(())
            }
            None => Err(XmlError::Malformed(
                format!("unterminated {end:?}"),
                self.pos,
            )),
        }
    }

    /// Parse one element, cursor positioned at its opening `<`.
    fn parse_element(&mut self) -> Result<Node, XmlError> {
        if !self.starts_with("<") {
            return Err(XmlError::Malformed("expected '<'".into(), self.pos));
        }
        self.advance(1);
        let name = self.read_name()?;
        let self_closing = self.skip_attributes()?;
        if self_closing {
            return Ok(Node {
                name,
                text: None,
                kids: Vec::new(),
            });
        }
        let (text, kids) = self.parse_content(&name)?;
        Ok(Node { name, text, kids })
    }

    /// An XML name: everything up to whitespace, `/`, or `>`.
    fn read_name(&mut self) -> Result<String, XmlError> {
        let end = self
            .rest()
            .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .ok_or_else(|| XmlError::Malformed("unterminated tag".into(), self.pos))?;
        if end == 0 {
            return Err(XmlError::Malformed("empty element name".into(), self.pos));
        }
        let name = self.rest()[..end].to_string();
        self.advance(end);
        Ok(name)
    }

    /// Skip `name="value"` pairs up to the tag's close. Returns whether the
    /// tag was self-closing (`/>`).
    fn skip_attributes(&mut self) -> Result<bool, XmlError> {
        loop {
            self.skip_ws();
            if self.starts_with("/>") {
                self.advance(2);
                return Ok(true);
            }
            if self.starts_with(">") {
                self.advance(1);
                return Ok(false);
            }
            if self.at_end() {
                return Err(XmlError::Malformed("unterminated tag".into(), self.pos));
            }
            // attribute name
            let name_end = self
                .rest()
                .find(|c: char| c.is_whitespace() || c == '=')
                .ok_or_else(|| XmlError::Malformed("malformed attribute".into(), self.pos))?;
            self.advance(name_end);
            self.skip_ws();
            if !self.starts_with("=") {
                return Err(XmlError::Malformed(
                    "attribute without a value".into(),
                    self.pos,
                ));
            }
            self.advance(1);
            self.skip_ws();
            let quote = self
                .rest()
                .chars()
                .next()
                .filter(|&c| c == '"' || c == '\'')
                .ok_or_else(|| XmlError::Malformed("unquoted attribute value".into(), self.pos))?;
            self.advance(1);
            let close = self
                .rest()
                .find(quote)
                .ok_or_else(|| XmlError::Malformed("unterminated attribute".into(), self.pos))?;
            self.advance(close + 1);
        }
    }

    /// Content of an element up to (and consuming) its closing tag: a
    /// leaf's decoded text, or a container's child elements. Whitespace-only
    /// text between child elements (pretty-printing) is discarded; if the
    /// element genuinely mixes text and children, the text is discarded too
    /// — this crate never emits or expects mixed content (spec §2).
    fn parse_content(&mut self, open_name: &str) -> Result<(Option<String>, Vec<Node>), XmlError> {
        let mut text = String::new();
        let mut kids = Vec::new();
        loop {
            let next_lt = self
                .rest()
                .find('<')
                .ok_or_else(|| XmlError::UnclosedElement(open_name.to_string()))?;
            if next_lt > 0 {
                text.push_str(&decode_entities(&self.rest()[..next_lt]));
                self.advance(next_lt);
            }
            if self.starts_with("</") {
                self.advance(2);
                let close_name = self.read_name()?;
                self.skip_ws();
                if !self.starts_with(">") {
                    return Err(XmlError::Malformed(
                        "unterminated close tag".into(),
                        self.pos,
                    ));
                }
                self.advance(1);
                if close_name != open_name {
                    return Err(XmlError::MismatchedClose {
                        open: open_name.to_string(),
                        close: close_name,
                    });
                }
                let text = if kids.is_empty() {
                    Some(text).filter(|t| !t.is_empty())
                } else {
                    None
                };
                return Ok((text, kids));
            }
            if self.starts_with("<!--") {
                self.skip_through("-->")?;
                continue;
            }
            if self.starts_with("<![CDATA[") {
                self.advance("<![CDATA[".len());
                let end = self
                    .rest()
                    .find("]]>")
                    .ok_or_else(|| XmlError::Malformed("unterminated CDATA".into(), self.pos))?;
                text.push_str(&self.rest()[..end]);
                self.advance(end + "]]>".len());
                continue;
            }
            kids.push(self.parse_element()?);
        }
    }
}

/// Decode the five predefined XML entities and numeric character
/// references (`&#NN;`, `&#xHH;`). Anything else that looks like an entity
/// but isn't recognized is kept literally rather than rejected — the same
/// leniency the rest of this crate's fallback behavior follows.
fn decode_entities(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains('&') {
        return std::borrow::Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let Some(semi) = rest.find(';').filter(|&i| i <= 10) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let body = &rest[1..semi];
        let decoded = match body {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ if body.starts_with("#x") || body.starts_with("#X") => {
                u32::from_str_radix(&body[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            _ if body.starts_with('#') => body[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &rest[semi + 1..];
            }
            None => {
                // Not a recognized entity: keep the `&` literal and retry
                // from just past it, so a bare `&` in data isn't dropped.
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    std::borrow::Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_leaf_element() {
        let root = parse_document("<PID.1>1</PID.1>").unwrap();
        assert_eq!(root.name, "PID.1");
        assert_eq!(root.text.as_deref(), Some("1"));
        assert!(root.kids.is_empty());
    }

    #[test]
    fn parses_nested_elements_and_ignores_indentation() {
        let root = parse_document(
            "<?xml version=\"1.0\"?>\n<PID>\n  <PID.1>1</PID.1>\n  <PID.3>\n    <CX.1>241900</CX.1>\n  </PID.3>\n</PID>\n",
        )
        .unwrap();
        assert_eq!(root.name, "PID");
        assert_eq!(root.kids.len(), 2);
        assert_eq!(root.kids[0].name, "PID.1");
        assert_eq!(root.kids[0].text.as_deref(), Some("1"));
        assert_eq!(root.kids[1].kids[0].text.as_deref(), Some("241900"));
    }

    #[test]
    fn treats_self_closing_and_empty_as_null() {
        assert_eq!(parse_document("<PID.2/>").unwrap().text, None);
        assert_eq!(parse_document("<PID.2></PID.2>").unwrap().text, None);
    }

    #[test]
    fn decodes_entities() {
        let root = parse_document("<NTE.3>A&amp;B &lt;200&#62;&#x21;</NTE.3>").unwrap();
        assert_eq!(root.text.as_deref(), Some("A&B <200>!"));
    }

    #[test]
    fn ignores_attributes_including_the_namespace() {
        let root = parse_document("<ORM_O01 xmlns=\"urn:hl7-org:v2xml\"><MSH/></ORM_O01>").unwrap();
        assert_eq!(root.name, "ORM_O01");
        assert_eq!(root.kids[0].name, "MSH");
    }

    #[test]
    fn reports_mismatched_close_tags() {
        let err = parse_document("<A><B></A></B>").unwrap_err();
        assert!(matches!(err, XmlError::MismatchedClose { .. }));
    }

    #[test]
    fn reports_an_unclosed_element() {
        assert!(matches!(
            parse_document("<A><B>text"),
            Err(XmlError::UnclosedElement(_))
        ));
    }

    #[test]
    fn skips_comments_and_cdata() {
        let root = parse_document("<A><!-- note --><B><![CDATA[<raw>]]></B></A>").unwrap();
        assert_eq!(root.kids[0].text.as_deref(), Some("<raw>"));
    }
}
