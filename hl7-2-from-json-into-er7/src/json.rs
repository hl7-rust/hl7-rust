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
        s: text,
        rest: text.strip_prefix('\u{feff}').unwrap_or(text),
        depth: 0,
    };
    cursor.skip_ws();
    if cursor.at_end() {
        return Err(JsonError::Empty);
    }
    let value = cursor.parse_value()?;
    cursor.skip_ws();
    if !cursor.at_end() {
        return Err(cursor.err("trailing data after the JSON value"));
    }
    Ok(value)
}

/// How deeply objects and arrays may nest before reading gives up.
///
/// Reading is recursive, so nesting depth is stack depth: without a limit a
/// few kilobytes of `[[[[…` abort the process with a stack overflow, which
/// a library must never do to a caller reading a document it did not write.
/// A converted message nests a handful of levels — structure, group,
/// segment, field, component, subcomponent — so this is far above anything
/// this crate is for and far below what threatens the stack.
const MAX_DEPTH: usize = 256;

struct Cursor<'a> {
    s: &'a str,
    /// The unread part of `s`, always a suffix of it. Holding the suffix
    /// rather than a byte offset means reading never does offset arithmetic
    /// or slices at an index (RCA-LEN-2).
    rest: &'a str,
    depth: usize,
}

impl Cursor<'_> {
    /// The byte offset of the unread part, for error messages.
    fn position(&self) -> usize {
        // `rest` is a suffix of `s`, so the difference is the bytes read.
        self.s.len().abs_diff(self.rest.len())
    }

    fn at_end(&self) -> bool {
        self.rest.is_empty()
    }

    fn peek(&self) -> Option<char> {
        self.rest.chars().next()
    }

    fn advance_char(&mut self) {
        let mut chars = self.rest.chars();
        chars.next();
        self.rest = chars.as_str();
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

    /// [`Cursor::eat`], also appending what was consumed to `out`.
    fn eat_into(&mut self, out: &mut String, prefix: &str) -> bool {
        let eaten = self.eat(prefix);
        if eaten {
            out.push_str(prefix);
        }
        eaten
    }

    fn skip_ws(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\t', '\n', '\r']);
    }

    fn err(&self, reason: &str) -> JsonError {
        JsonError::Malformed(reason.to_string(), self.position())
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
            Some('{') => self.nested(Cursor::parse_object),
            Some('[') => self.nested(Cursor::parse_array),
            Some('"') => self.parse_string().map(Value::String),
            Some('t') if self.eat("true") => Ok(Value::Bool(true)),
            Some('f') if self.eat("false") => Ok(Value::Bool(false)),
            Some('n') if self.eat("null") => Ok(Value::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            _ => Err(self.err("expected a JSON value")),
        }
    }

    /// Read one container, counting the level so that nesting cannot run
    /// the stack out from under the caller.
    fn nested(
        &mut self,
        read: fn(&mut Self) -> Result<Value, JsonError>,
    ) -> Result<Value, JsonError> {
        if self.depth >= MAX_DEPTH {
            return Err(self.err(&format!("nested more than {MAX_DEPTH} deep")));
        }
        // RCA-EXC-2: `depth < MAX_DEPTH` was checked just above, so the
        // increment is at most `MAX_DEPTH` and the decrement undoes it.
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "RCA-EXC-2: depth is below MAX_DEPTH here, proof above"
        )]
        let value = {
            self.depth += 1;
            let value = read(self);
            self.depth -= 1;
            value
        };
        value
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
                            out.push(self.read_unicode_escape()?);
                            continue; // hex digits already consumed; don't advance_char again
                        }
                        _ => return Err(self.err("invalid escape sequence")),
                    }
                    self.advance_char();
                }
                Some('\0'..='\u{1f}') => return Err(self.err("unescaped control character")),
                Some(c) => {
                    out.push(c);
                    self.advance_char();
                }
            }
        }
    }

    fn read_unicode_escape(&mut self) -> Result<char, JsonError> {
        let first = self.read_hex4()?;
        if !(0xD800..=0xDBFF).contains(&first) {
            return char::from_u32(u32::from(first))
                .ok_or_else(|| self.err("unpaired low surrogate"));
        }
        if !self.eat(r"\u") {
            return Err(self.err("expected a low surrogate"));
        }
        let low = self.read_hex4()?;
        // UTF-16 decoding combines the pair without any arithmetic here, and
        // rejects a second unit that is not a low surrogate.
        char::decode_utf16([first, low])
            .next()
            .and_then(Result::ok)
            .ok_or_else(|| self.err("invalid low surrogate"))
    }

    /// Read exactly four hex digits (a `\u` escape's payload) and advance
    /// past them.
    fn read_hex4(&mut self) -> Result<u16, JsonError> {
        let Some((digits, tail)) = self.rest.split_at_checked(4) else {
            return Err(self.err("truncated \\u escape"));
        };
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(self.err("invalid \\u escape"));
        }
        let value = u16::from_str_radix(digits, 16).map_err(|_| self.err("invalid \\u escape"))?;
        self.rest = tail;
        Ok(value)
    }

    /// A number, kept as the text it was written as. Each part is appended
    /// as it is read, so the text is never sliced back out of the input.
    fn parse_number(&mut self) -> Result<Value, JsonError> {
        let mut number = String::new();
        self.eat_into(&mut number, "-");
        if !self.eat_into(&mut number, "0") {
            self.read_digits(&mut number)?;
        }
        if self.eat_into(&mut number, ".") {
            self.read_digits(&mut number)?;
        }
        if self.eat_into(&mut number, "e") || self.eat_into(&mut number, "E") {
            if !self.eat_into(&mut number, "+") {
                self.eat_into(&mut number, "-");
            }
            self.read_digits(&mut number)?;
        }
        Ok(Value::Number(number))
    }

    fn read_digits(&mut self, out: &mut String) -> Result<(), JsonError> {
        let mut any = false;
        while let Some(digit) = self.peek().filter(char::is_ascii_digit) {
            out.push(digit);
            self.advance_char();
            any = true;
        }
        if !any {
            return Err(self.err("expected a digit in number"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn many_sibling_containers_do_not_count_as_nesting() {
        // The depth goes back down after each container, so 300 empty arrays
        // side by side (more than MAX_DEPTH) are fine, while 300 nested are not.
        let siblings = format!("[{}]", vec!["[]"; 300].join(","));
        assert!(parse_document(&siblings).is_ok());
        let nested = format!("{}{}", "[".repeat(300), "]".repeat(300));
        assert!(parse_document(&nested).is_err());
    }

    #[test]
    fn a_value_that_is_not_a_number_is_not_read_as_one() {
        for text in ["x", "+1", ".5", "tru", "nul"] {
            assert!(
                matches!(
                    parse_document(text),
                    Err(JsonError::Malformed(ref reason, _)) if reason == "expected a JSON value"
                ),
                "{text:?}"
            );
        }
    }

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
        assert!(matches!(
            parse_document(r#"{"b": 1, "a": 2}"#),
            Ok(Value::Object(entries)) if entries[0].0 == "b" && entries[1].0 == "a"
        ));
    }

    #[test]
    fn parses_arrays() {
        assert!(matches!(
            parse_document("[1, \"two\", null]"),
            Ok(Value::Array(items)) if items.len() == 3
        ));
    }

    #[test]
    fn decodes_string_escapes() {
        assert!(matches!(
            parse_document(r#""a\tb\"c\\d&e""#),
            Ok(Value::String(s)) if s == "a\tb\"c\\d&e"
        ));
    }

    #[test]
    fn decodes_a_surrogate_pair_escape() {
        // U+1F600 (😀) written as a UTF-16 surrogate pair, the way a JSON
        // writer represents a character outside the Basic Multilingual
        // Plane.
        let input = "\"\\uD83D\\uDE00\"";
        assert!(matches!(
            parse_document(input),
            Ok(Value::String(s)) if s == "\u{1F600}"
        ));
    }

    #[test]
    fn passes_through_literal_non_ascii_utf8() {
        assert!(matches!(
            parse_document(r#""😀""#),
            Ok(Value::String(s)) if s == "\u{1F600}"
        ));
    }

    #[test]
    fn covers_empty_containers_false_and_every_string_escape() {
        assert!(matches!(parse_document("[]"), Ok(Value::Array(items)) if items.is_empty()));
        assert!(matches!(parse_document("false"), Ok(Value::Bool(false))));
        assert!(matches!(
            parse_document(r#""\/\b\f\n\r""#),
            Ok(Value::String(s)) if s == "/\u{8}\u{c}\n\r"
        ));
    }

    #[test]
    fn reports_every_remaining_malformed_route() {
        assert_eq!(JsonError::Empty.to_string(), "no JSON value found");
        for text in [
            r#"{"a" 1}"#,
            "[1 2]",
            r#""unterminated"#,
            r#""\q""#,
            r#""\u12""#,
        ] {
            assert!(
                matches!(parse_document(text), Err(JsonError::Malformed(..))),
                "{text:?}"
            );
        }

        let mut empty = Cursor {
            s: "",
            rest: "",
            depth: 0,
        };
        empty.advance_char();
        assert_eq!(empty.position(), 0);
    }

    #[test]
    fn numbers_keep_the_text_they_were_written_as() {
        for text in [
            "0", "-0", "7", "-12", "3.25", "1e9", "1E+9", "2.5e-3", "-0.0E0",
        ] {
            assert!(
                matches!(parse_document(text), Ok(Value::Number(number)) if number == text),
                "{text}"
            );
        }
        for text in ["-", "1.", "1e", "1e+", "-x", ".5"] {
            assert!(parse_document(text).is_err(), "{text:?}");
        }
        // A pair whose second unit is not a low surrogate is rejected.
        assert!(parse_document(r#""\uD83D\u0041""#).is_err());
    }

    #[test]
    fn rejects_trailing_data() {
        assert!(parse_document("{} {}").is_err());
    }

    #[test]
    fn error_offsets_include_the_bom_and_multibyte_text() {
        let text = "\u{feff}\"東京\n\"";
        let error = parse_document(text).unwrap_err();
        assert!(matches!(error, JsonError::Malformed(_, at) if at == text.find('\n').unwrap()));
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(parse_document("   "), Err(JsonError::Empty)));
    }

    #[test]
    fn nesting_past_the_limit_is_an_error_not_a_stack_overflow() {
        // Reading is recursive, so nesting depth is stack depth. Without a
        // limit these aborted the process — a crash the caller cannot catch,
        // from a document another system sent them.
        let deep_objects = format!("{}1{}", "{\"a\":".repeat(1000), "}".repeat(1000));
        let deep_arrays = format!("{}1{}", "[".repeat(1000), "]".repeat(1000));
        for text in [deep_objects, deep_arrays] {
            assert!(matches!(
                parse_document(&text),
                Err(JsonError::Malformed(ref reason, _)) if reason.contains("nested more than")
            ));
        }
    }

    #[test]
    fn ordinary_nesting_depth_still_reads() {
        let depth = 64;
        let text = format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
        assert!(parse_document(&text).is_ok());
    }
}
