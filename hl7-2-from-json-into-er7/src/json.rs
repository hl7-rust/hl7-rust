//! A minimal, dependency-free JSON reader (RFC 8259) for the documents this
//! crate consumes.
//!
//! It reads the whole grammar — objects, arrays, strings, numbers,
//! `true`/`false`/`null` — because the input may be hand-edited even though
//! the forward `hl7-2-from-er7-into-json` crate never emits a number or a
//! boolean itself (its own spec §4.2, §6). Numbers and booleans are kept
//! (as their literal source text) rather than rejected, so that a
//! hand-edited document with an unexpected scalar still converts; see
//! `spec/index.md` §2 for exactly how those coerce.

use std::fmt;

/// A parsed JSON value. Object keys keep first-appearance order, matching
/// how the forward crate writes them (its spec §4.3) and how this crate's
/// reconstruction relies on them.
#[derive(Debug, Clone)]
pub enum Value {
    /// `{ "key": value, ... }`, in source order. JSON technically permits a
    /// duplicate key; if the input has one, both are kept here rather than
    /// one silently winning, and [`crate::reconstruct`] documents how it
    /// resolves that.
    Object(Vec<(String, Value)>),
    /// `[ value, ... ]`.
    Array(Vec<Value>),
    /// A JSON string, already unescaped.
    String(String),
    /// A JSON number, kept as its exact source text rather than parsed —
    /// this crate never needs its numeric value, only (in the rare case a
    /// hand-edited document has one where a string was expected) its text.
    Number(String),
    /// `true` or `false`.
    Bool(bool),
    /// `null`.
    Null,
}

/// What can go wrong reading the JSON itself (as opposed to what it means
/// as HL7, which is [`crate::Hl7Error`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonError {
    /// The input holds no value at all.
    Empty,
    /// The input isn't well-formed JSON; carries a short reason and the
    /// byte offset where parsing gave up.
    Malformed(String, usize),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::Empty => write!(f, "no JSON value found"),
            JsonError::Malformed(reason, at) => write!(f, "{reason} at byte {at}"),
        }
    }
}

impl std::error::Error for JsonError {}

/// Parse `text` as a single JSON document.
/// # Errors
///
/// [`JsonError`] when the text is not valid JSON, with the byte offset
/// where parsing gave up.
pub fn parse_document(text: &str) -> Result<Value, JsonError> {
    let mut cursor = Cursor {
        s: text.strip_prefix('\u{feff}').unwrap_or(text),
        pos: 0,
    };
    cursor.skip_ws();
    if cursor.at_end() {
        return Err(JsonError::Empty);
    }
    let value = cursor.parse_value()?;
    cursor.skip_ws();
    if !cursor.at_end() {
        return Err(JsonError::Malformed(
            "trailing data after the JSON value".into(),
            cursor.pos,
        ));
    }
    Ok(value)
}

struct Cursor<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn rest(&self) -> &'a str {
        &self.s[self.pos..]
    }

    fn at_end(&self) -> bool {
        self.pos >= self.s.len()
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn advance_char(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }

    fn starts_with(&self, pat: &str) -> bool {
        self.rest().starts_with(pat)
    }

    fn skip_ws(&mut self) {
        let trimmed = self.rest().trim_start_matches([' ', '\t', '\n', '\r']);
        self.pos = self.s.len() - trimmed.len();
    }

    fn err(&self, reason: &str) -> JsonError {
        JsonError::Malformed(reason.to_string(), self.pos)
    }

    fn expect(&mut self, c: char) -> Result<(), JsonError> {
        if self.peek() == Some(c) {
            self.advance_char();
            Ok(())
        } else {
            Err(self.err(&format!("expected {c:?}")))
        }
    }

    fn parse_value(&mut self) -> Result<Value, JsonError> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(Value::String),
            Some('t') if self.starts_with("true") => {
                self.pos += 4;
                Ok(Value::Bool(true))
            }
            Some('f') if self.starts_with("false") => {
                self.pos += 5;
                Ok(Value::Bool(false))
            }
            Some('n') if self.starts_with("null") => {
                self.pos += 4;
                Ok(Value::Null)
            }
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            _ => Err(self.err("expected a JSON value")),
        }
    }

    fn parse_object(&mut self) -> Result<Value, JsonError> {
        self.expect('{')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.advance_char();
            return Ok(Value::Object(entries));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.advance_char();
                }
                Some('}') => {
                    self.advance_char();
                    return Ok(Value::Object(entries));
                }
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, JsonError> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.advance_char();
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.advance_char();
                }
                Some(']') => {
                    self.advance_char();
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, JsonError> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string")),
                Some('"') => {
                    self.advance_char();
                    return Ok(out);
                }
                Some('\\') => {
                    self.advance_char();
                    match self.peek() {
                        Some('"') => out.push('"'),
                        Some('\\') => out.push('\\'),
                        Some('/') => out.push('/'),
                        Some('b') => out.push('\u{8}'),
                        Some('f') => out.push('\u{c}'),
                        Some('n') => out.push('\n'),
                        Some('r') => out.push('\r'),
                        Some('t') => out.push('\t'),
                        Some('u') => {
                            self.advance_char();
                            let high = self.read_hex4()?;
                            let code = if (0xD800..=0xDBFF).contains(&high)
                                && self.starts_with(r"\u")
                            {
                                self.advance_char(); // consume the backslash
                                self.advance_char(); // consume the 'u'
                                let low = self.read_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(self.err("invalid low surrogate"));
                                }
                                0x10000 + (u32::from(high - 0xD800) << 10) + u32::from(low - 0xDC00)
                            } else {
                                u32::from(high)
                            };
                            out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                            continue; // hex digits already consumed; don't advance_char again
                        }
                        _ => return Err(self.err("invalid escape sequence")),
                    }
                    self.advance_char();
                }
                Some(c) => {
                    out.push(c);
                    self.advance_char();
                }
            }
        }
    }

    /// Read exactly four hex digits (a `\u` escape's payload) and advance
    /// past them.
    fn read_hex4(&mut self) -> Result<u16, JsonError> {
        let rest = self.rest();
        if rest.len() < 4 || !rest.is_char_boundary(4) {
            return Err(self.err("truncated \\u escape"));
        }
        let digits = &rest[..4];
        let value = u16::from_str_radix(digits, 16).map_err(|_| self.err("invalid \\u escape"))?;
        self.pos += 4;
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<Value, JsonError> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.advance_char();
        }
        let digits = self
            .rest()
            .find(|c: char| {
                !c.is_ascii_digit() && c != '.' && c != 'e' && c != 'E' && c != '+' && c != '-'
            })
            .unwrap_or(self.rest().len());
        self.pos += digits;
        if self.pos == start {
            return Err(self.err("invalid number"));
        }
        Ok(Value::Number(self.s[start..self.pos].to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scalars() {
        assert!(matches!(parse_document("null").unwrap(), Value::Null));
        assert!(matches!(parse_document("true").unwrap(), Value::Bool(true)));
        assert!(matches!(parse_document("\"hi\"").unwrap(), Value::String(s) if s == "hi"));
        assert!(matches!(parse_document("42").unwrap(), Value::Number(s) if s == "42"));
        assert!(matches!(parse_document("-3.5e10").unwrap(), Value::Number(s) if s == "-3.5e10"));
    }

    #[test]
    fn parses_objects_preserving_key_order() {
        let Value::Object(entries) = parse_document(r#"{"b": 1, "a": 2}"#).unwrap() else {
            panic!("expected object")
        };
        assert_eq!(entries[0].0, "b");
        assert_eq!(entries[1].0, "a");
    }

    #[test]
    fn parses_arrays() {
        let Value::Array(items) = parse_document("[1, \"two\", null]").unwrap() else {
            panic!("expected array")
        };
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn decodes_string_escapes() {
        let Value::String(s) = parse_document(r#""a\tb\"c\\d&e""#).unwrap() else {
            panic!("expected string")
        };
        assert_eq!(s, "a\tb\"c\\d&e");
    }

    #[test]
    fn decodes_a_surrogate_pair_escape() {
        // U+1F600 (😀) written as a UTF-16 surrogate pair, the way a JSON
        // writer represents a character outside the Basic Multilingual
        // Plane.
        let input = "\"\\uD83D\\uDE00\"";
        let Value::String(s) = parse_document(input).unwrap() else {
            panic!("expected string")
        };
        assert_eq!(s, "\u{1F600}");
    }

    #[test]
    fn passes_through_literal_non_ascii_utf8() {
        let Value::String(s) = parse_document(r#""😀""#).unwrap() else {
            panic!("expected string")
        };
        assert_eq!(s, "\u{1F600}");
    }

    #[test]
    fn rejects_trailing_data() {
        assert!(parse_document("{} {}").is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(parse_document("   "), Err(JsonError::Empty)));
    }
}
