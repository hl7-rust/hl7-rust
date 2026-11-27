//! A small hand-written JSON reader, used to load dictionaries.
//!
//! Dictionaries — the bundled ones in `schemas/` and the ones callers write
//! themselves for schema mode — are JSON, so something has to read JSON.
//! Doing it here, in about the space a dependency declaration would take,
//! keeps this crate's runtime dependency list at exactly one entry
//! (`er7`, which itself has none), which is worth more in a domain where
//! dependency trees get audited than the few hundred lines it costs. The
//! sibling `hl7-2-from-er7-into-json` crate hand-rolls its JSON *writer*
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
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The string, if this is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// The members, if this is an object.
    #[must_use]
    pub fn as_object(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Object(members) => Some(members),
            _ => None,
        }
    }

    /// The elements, if this is an array.
    #[must_use]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The boolean, if this is one.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// True when this is `null`.
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// The name of this value's kind, for error messages.
    #[must_use]
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
/// # Errors
///
/// [`Error`] when the text is not valid JSON, with where parsing gave up.
pub fn parse(text: &str) -> Result<Value, Error> {
    let mut reader = Reader {
        text,
        rest: text,
        depth: 0,
    };
    reader.skip_whitespace();
    let value = reader.value()?;
    reader.skip_whitespace();
    if !reader.rest.is_empty() {
        return Err(reader.error("unexpected trailing content"));
    }
    Ok(value)
}

/// How deeply objects and arrays may nest before reading gives up.
///
/// Reading is recursive, so nesting depth is stack depth: without a limit a
/// few kilobytes of `[[[[…` abort the process with a stack overflow, which
/// a library must never do to a caller who merely read a file from
/// somewhere. A dictionary nests a handful of levels, so this is far above
/// anything real and far below what threatens the stack.
const MAX_DEPTH: usize = 256;

struct Reader<'a> {
    text: &'a str,
    /// The unread part of `text`, always a suffix of it. Holding the suffix
    /// rather than a byte offset means reading never does offset arithmetic
    /// or slices at an index (RCA-LEN-2).
    rest: &'a str,
    depth: usize,
}

impl Reader<'_> {
    fn error(&self, detail: &str) -> Error {
        Error {
            detail: detail.to_string(),
            // `rest` is a suffix of `text`, so the difference is the bytes read.
            offset: self.text.len().abs_diff(self.rest.len()),
        }
    }

    fn peek(&self) -> Option<char> {
        self.rest.chars().next()
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

    /// Consume one character, returning it.
    fn next_char(&mut self) -> Option<char> {
        let mut chars = self.rest.chars();
        let next = chars.next();
        self.rest = chars.as_str();
        next
    }

    fn skip_whitespace(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\t', '\r', '\n']);
    }

    /// Consume `word` if it is next, reporting a useful error if not.
    fn literal(&mut self, word: &str, value: Value) -> Result<Value, Error> {
        if self.eat(word) {
            Ok(value)
        } else {
            Err(self.error(&format!("expected `{word}`")))
        }
    }

    fn value(&mut self) -> Result<Value, Error> {
        match self.peek() {
            None => Err(self.error("expected a value")),
            Some('{') => self.nested(Reader::object),
            Some('[') => self.nested(Reader::array),
            Some('"') => Ok(Value::String(self.string()?)),
            Some('t') => self.literal("true", Value::Bool(true)),
            Some('f') => self.literal("false", Value::Bool(false)),
            Some('n') => self.literal("null", Value::Null),
            Some(_) => self.number(),
        }
    }

    /// Read one container, counting the level so that nesting cannot run
    /// the stack out from under the caller.
    fn nested(&mut self, read: fn(&mut Self) -> Result<Value, Error>) -> Result<Value, Error> {
        if self.depth >= MAX_DEPTH {
            return Err(self.error(&format!("nested more than {MAX_DEPTH} deep")));
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

    fn object(&mut self) -> Result<Value, Error> {
        self.eat("{");
        let mut members = Vec::new();
        self.skip_whitespace();
        if self.eat("}") {
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err(self.error("expected a member name"));
            }
            let name = self.string()?;
            self.skip_whitespace();
            if !self.eat(":") {
                return Err(self.error("expected `:` after a member name"));
            }
            self.skip_whitespace();
            members.push((name, self.value()?));
            self.skip_whitespace();
            if self.eat("}") {
                return Ok(Value::Object(members));
            }
            if !self.eat(",") {
                return Err(self.error("expected `,` or `}`"));
            }
        }
    }

    fn array(&mut self) -> Result<Value, Error> {
        self.eat("[");
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.eat("]") {
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value()?);
            self.skip_whitespace();
            if self.eat("]") {
                return Ok(Value::Array(items));
            }
            if !self.eat(",") {
                return Err(self.error("expected `,` or `]`"));
            }
        }
    }

    fn string(&mut self) -> Result<String, Error> {
        self.eat("\""); // opening quote
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err(self.error("unterminated string")),
                Some('"') => {
                    self.next_char();
                    return Ok(out);
                }
                Some('\\') => {
                    self.next_char();
                    self.escape(&mut out)?;
                }
                Some('\0'..='\u{1f}') => {
                    return Err(self.error("unescaped control character in string"));
                }
                Some(c) => {
                    self.next_char();
                    out.push(c);
                }
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<(), Error> {
        let Some(c) = self.next_char() else {
            return Err(self.error("unterminated escape sequence"));
        };
        out.push(match c {
            '"' => '"',
            '\\' => '\\',
            '/' => '/',
            'b' => '\u{8}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => return self.unicode_escape(out),
            _ => return Err(self.error("unknown escape sequence")),
        });
        Ok(())
    }

    /// Resolve `\\uXXXX`, joining a surrogate pair when one follows.
    fn unicode_escape(&mut self, out: &mut String) -> Result<(), Error> {
        let high = self.hex4()?;
        let scalar = if (0xd800..0xdc00).contains(&high) {
            if !self.eat("\\u") {
                return Err(self.error("lone high surrogate"));
            }
            let low = self.hex4()?;
            // UTF-16 decoding joins the pair without any arithmetic here, and
            // rejects a second unit that is not a low surrogate.
            char::decode_utf16([high, low])
                .next()
                .and_then(Result::ok)
                .ok_or_else(|| self.error("expected a low surrogate"))?
        } else {
            char::from_u32(u32::from(high)).ok_or_else(|| self.error("lone low surrogate"))?
        };
        out.push(scalar);
        Ok(())
    }

    fn hex4(&mut self) -> Result<u16, Error> {
        let Some((digits, tail)) = self.rest.split_at_checked(4) else {
            return Err(self.error("truncated `\\u` escape"));
        };
        // Four hex digits, read with checked arithmetic; anything else
        // (including a sign, which `from_str_radix` would accept) is refused.
        let value = digits.chars().try_fold(0_u16, |value, c| {
            let digit = u16::try_from(c.to_digit(16)?).ok()?;
            value.checked_mul(16)?.checked_add(digit)
        });
        let Some(value) = value else {
            return Err(self.error("`\\u` escape needs four hex digits"));
        };
        self.rest = tail;
        Ok(value)
    }

    fn number(&mut self) -> Result<Value, Error> {
        let tail = self
            .rest
            .trim_start_matches(|c: char| matches!(c, '0'..='9' | '.' | 'e' | 'E' | '+' | '-'));
        // The characters a number may use, up to the first that it may not.
        // An empty run reads as "", which is not a number either.
        let text = self
            .rest
            .split_at_checked(self.rest.len().abs_diff(tail.len()))
            .map_or("", |(text, _)| text);
        match text.parse::<f64>() {
            Ok(number) => {
                self.rest = tail;
                Ok(Value::Number(number))
            }
            Err(_) => Err(self.error("expected a value")),
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
        assert_eq!(parse(r#""\uABCD""#).unwrap().as_str(), Some("ꯍ"));
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

    #[test]
    fn many_sibling_containers_do_not_count_as_nesting() {
        let siblings = format!("[{}]", vec!["[]"; 300].join(","));
        assert!(parse(&siblings).is_ok());
        let nested = format!("{}{}", "[".repeat(300), "]".repeat(300));
        assert!(parse(&nested).is_err());
    }

    #[test]
    fn rejects_each_malformed_json_shape() {
        for text in [
            "tru",
            "fals",
            "nul",
            "-",
            "[1 2]",
            "[1,]",
            "{1:2}",
            r#"{"a":1 "b":2}"#,
            r#""line
break""#,
            r#""\q""#,
            r#""\"#,
            r#""\u12""#,
            r#""\u12xz""#,
            r#""\ud800""#,
            r#""\ud800\u0041""#,
            r#""\udfff""#,
        ] {
            assert!(parse(text).is_err(), "accepted {text:?}");
        }
    }

    #[test]
    fn all_value_kinds_and_escape_spellings_are_observable() {
        let escaped = parse(r#""\"\\\/\b\f\n\r\t\u0041\ud83d\ude00""#).unwrap();
        assert_eq!(escaped.as_str(), Some("\"\\/\u{8}\u{c}\n\r\tA😀"));

        let values = [
            Value::Null,
            Value::Bool(false),
            Value::Number(1.0),
            Value::String(String::new()),
            Value::Array(Vec::new()),
            Value::Object(Vec::new()),
        ];
        assert_eq!(
            values.iter().map(Value::kind).collect::<Vec<_>>(),
            ["null", "boolean", "number", "string", "array", "object"]
        );
        for value in &values[..5] {
            assert_eq!(value.as_object(), None);
        }
        assert_eq!(Value::Null.as_array(), None);
        assert_eq!(Value::Null.as_bool(), None);
        assert_eq!(Value::Null.as_str(), None);
        assert_eq!(Value::Null.get("x"), None);
    }

    #[test]
    fn nesting_past_the_limit_is_an_error_not_a_stack_overflow() {
        // Reading is recursive, so nesting depth is stack depth. Without a
        // limit these aborted the process — a crash the caller cannot catch,
        // from a dictionary file they merely read off disk.
        let deep_objects = format!("{}1{}", "{\"a\":".repeat(1000), "}".repeat(1000));
        let deep_arrays = format!("{}1{}", "[".repeat(1000), "]".repeat(1000));
        for text in [deep_objects, deep_arrays] {
            let error = parse(&text).unwrap_err();
            assert!(error.detail.contains("nested more than"), "{error}");
        }
    }

    #[test]
    fn ordinary_nesting_depth_still_reads() {
        let depth = 64;
        let text = format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
        assert!(parse(&text).is_ok());
    }
}
