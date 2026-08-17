//! A small hand-written JSON reader, used to load dictionaries.
//!
//! Dictionaries — the bundled ones in `schemas/` and the ones callers write
//! themselves for schema mode — are JSON, so something has to read JSON.
//! Doing it here, in about the space a dependency declaration would take,
//! keeps this crate's runtime dependency list at exactly one entry
//! (`er7`, which itself has none), which is worth more in a domain where
//! dependency trees get audited than the few hundred lines it costs. The
//! sibling `hl7-v2-from-er7-into-json` crate hand-rolls its JSON *writer*
//! for the same reason; this is the mirror of it.
//!
//! The reader is deliberately plain: it accepts RFC 8259 JSON, keeps object
//! members in file order (dictionaries are read, diffed, and eyeballed by
//! people, so order is worth preserving), and reports the byte offset of a
//! syntax error so a typo in a 1,300-line dictionary is findable.

use std::fmt;

/// A parsed JSON value.
///
/// Object members are a `Vec` rather than a map so that document order
/// survives; dictionaries are small enough that lookup by scan is not worth
/// a `BTreeMap`'s allocation per object.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// Any JSON number, held as `f64`.
    Number(f64),
    /// A string, with escape sequences already resolved.
    String(String),
    /// An array, in order.
    Array(Vec<Value>),
    /// An object: members in the order they appeared.
    Object(Vec<(String, Value)>),
}

impl Value {
    /// The member named `key`, if this is an object that has one.
    ///
    /// A duplicate key returns the first occurrence, matching the "first
    /// wins" reading most JSON tooling settles on.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The string, if this is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// The members, if this is an object.
    pub fn as_object(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Object(members) => Some(members),
            _ => None,
        }
    }

    /// The elements, if this is an array.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The boolean, if this is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// True when this is `null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// The name of this value's kind, for error messages.
    pub fn kind(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

/// A JSON syntax error: what was wrong, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// What the reader expected, or what it found instead.
    pub detail: String,
    /// Byte offset into the input where the problem was noticed.
    pub offset: usize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid JSON at byte {}: {}", self.offset, self.detail)
    }
}

impl std::error::Error for Error {}

/// Read a complete JSON document. Trailing content after the top-level
/// value is an error, so a truncated or double-pasted file is caught here
/// rather than silently half-read.
pub fn parse(text: &str) -> Result<Value, Error> {
    let mut reader = Reader {
        bytes: text.as_bytes(),
        pos: 0,
    };
    reader.skip_whitespace();
    let value = reader.value()?;
    reader.skip_whitespace();
    if reader.pos != reader.bytes.len() {
        return Err(reader.error("unexpected trailing content"));
    }
    Ok(value)
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn error(&self, detail: &str) -> Error {
        Error {
            detail: detail.to_string(),
            offset: self.pos,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }

    /// Consume `word` if it is next, reporting a useful error if not.
    fn literal(&mut self, word: &str, value: Value) -> Result<Value, Error> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(self.error(&format!("expected `{word}`")))
        }
    }

    fn value(&mut self) -> Result<Value, Error> {
        match self.peek() {
            None => Err(self.error("expected a value")),
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'n') => self.literal("null", Value::Null),
            Some(_) => self.number(),
        }
    }

    fn object(&mut self) -> Result<Value, Error> {
        self.pos += 1; // `{`
        let mut members = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.error("expected a member name"));
            }
            let name = self.string()?;
            self.skip_whitespace();
            if self.peek() != Some(b':') {
                return Err(self.error("expected `:` after a member name"));
            }
            self.pos += 1;
            self.skip_whitespace();
            members.push((name, self.value()?));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Object(members));
                }
                _ => return Err(self.error("expected `,` or `}`")),
            }
        }
    }

    fn array(&mut self) -> Result<Value, Error> {
        self.pos += 1; // `[`
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.error("expected `,` or `]`")),
            }
        }
    }

    fn string(&mut self) -> Result<String, Error> {
        self.pos += 1; // opening quote
        let mut out = String::new();
        loop {
            let byte = match self.peek() {
                Some(byte) => byte,
                None => return Err(self.error("unterminated string")),
            };
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    self.escape(&mut out)?;
                }
                0x00..=0x1f => return Err(self.error("unescaped control character in string")),
                _ => {
                    // Multi-byte UTF-8 passes through whole: find the end of
                    // this character in the original text rather than
                    // pushing bytes one at a time.
                    let start = self.pos;
                    self.pos += 1;
                    while matches!(self.peek(), Some(b) if b & 0xc0 == 0x80) {
                        self.pos += 1;
                    }
                    match std::str::from_utf8(&self.bytes[start..self.pos]) {
                        Ok(text) => out.push_str(text),
                        Err(_) => return Err(self.error("invalid UTF-8 in string")),
                    }
                }
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<(), Error> {
        let byte = match self.peek() {
            Some(byte) => byte,
            None => return Err(self.error("unterminated escape sequence")),
        };
        self.pos += 1;
        out.push(match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.unicode_escape(out),
            _ => return Err(self.error("unknown escape sequence")),
        });
        Ok(())
    }

    /// Resolve `\uXXXX`, joining a surrogate pair when one follows.
    fn unicode_escape(&mut self, out: &mut String) -> Result<(), Error> {
        let high = self.hex4()?;
        let scalar = if (0xd800..0xdc00).contains(&high) {
            if !self.bytes[self.pos..].starts_with(b"\\u") {
                return Err(self.error("lone high surrogate"));
            }
            self.pos += 2;
            let low = self.hex4()?;
            if !(0xdc00..0xe000).contains(&low) {
                return Err(self.error("expected a low surrogate"));
            }
            0x10000 + ((high - 0xd800) << 10) + (low - 0xdc00)
        } else if (0xdc00..0xe000).contains(&high) {
            return Err(self.error("lone low surrogate"));
        } else {
            high
        };
        match char::from_u32(scalar) {
            Some(c) => out.push(c),
            None => return Err(self.error("escape is not a Unicode scalar value")),
        }
        Ok(())
    }

    fn hex4(&mut self) -> Result<u32, Error> {
        let end = self.pos + 4;
        if end > self.bytes.len() {
            return Err(self.error("truncated `\\u` escape"));
        }
        let mut value = 0;
        for &byte in &self.bytes[self.pos..end] {
            let digit = match byte {
                b'0'..=b'9' => u32::from(byte - b'0'),
                b'a'..=b'f' => u32::from(byte - b'a') + 10,
                b'A'..=b'F' => u32::from(byte - b'A') + 10,
                _ => return Err(self.error("`\\u` escape needs four hex digits")),
            };
            value = value * 16 + digit;
        }
        self.pos = end;
        Ok(value)
    }

    fn number(&mut self) -> Result<Value, Error> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while matches!(
            self.peek(),
            Some(b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
        ) {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).unwrap_or("");
        match text.parse::<f64>() {
            Ok(number) => Ok(Value::Number(number)),
            Err(_) => {
                self.pos = start;
                Err(self.error("expected a value"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_shapes_a_dictionary_uses() {
        let value =
            parse(r#"{"types": {"XPN": ["FN", "ST"]}, "n": 2.5, "ok": true, "x": null}"#).unwrap();
        assert_eq!(
            value
                .get("types")
                .unwrap()
                .get("XPN")
                .unwrap()
                .as_array()
                .unwrap()[0]
                .as_str(),
            Some("FN")
        );
        assert_eq!(value.get("n"), Some(&Value::Number(2.5)));
        assert_eq!(value.get("ok").unwrap().as_bool(), Some(true));
        assert!(value.get("x").unwrap().is_null());
        assert_eq!(value.get("missing"), None);
    }

    #[test]
    fn keeps_object_order() {
        let value = parse(r#"{"b": 1, "a": 2}"#).unwrap();
        let names: Vec<&str> = value
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["b", "a"]);
    }

    #[test]
    fn resolves_escapes_including_surrogate_pairs() {
        assert_eq!(
            parse(r#""aé😀\n\"\\""#).unwrap().as_str(),
            Some("aé😀\n\"\\")
        );
        assert_eq!(parse("\"caf\u{e9}\"").unwrap().as_str(), Some("café"));
    }

    #[test]
    fn reports_where_the_problem_is() {
        // A dictionary is long; "somewhere" is not a useful answer.
        let error = parse(r#"{"a": 1,}"#).unwrap_err();
        assert_eq!(error.offset, 8);
        assert!(error.to_string().contains("byte 8"), "{error}");
        assert!(parse(r#"{"a": 1} {"b": 2}"#).is_err());
        assert!(parse(r#"{"a" 1}"#).is_err());
        assert!(parse(r#""unterminated"#).is_err());
        assert!(parse("").is_err());
    }
}
