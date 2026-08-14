//! Parsing of HL7 v2.x ER7 ("pipe-delimited") encoding into a value tree.
//!
//! ER7 nesting is: message -> segment -> field -> repetition -> component
//! -> subcomponent. The delimiter characters are declared by the message
//! itself in MSH-1 (field separator) and MSH-2 (encoding characters), so
//! nothing here hardcodes `|^~\&` beyond the defaults.

use crate::Hl7Error;

/// The five ER7 delimiter characters, as declared by MSH-1/MSH-2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Separators {
    /// Separates fields within a segment. Read from MSH-1 itself (the byte
    /// right after `MSH`). Traditionally `|`.
    pub field: char,
    /// Separates components within a field repetition. Read from MSH-2,
    /// encoding-characters position 1. Traditionally `^`.
    pub component: char,
    /// Separates repetitions within a field. Read from MSH-2, encoding-
    /// characters position 2. Traditionally `~`.
    pub repetition: char,
    /// Introduces and closes an escape sequence (see [`unescape`]). Read
    /// from MSH-2, encoding-characters position 3. Traditionally `\`.
    pub escape: char,
    /// Separates subcomponents within a component. Read from MSH-2,
    /// encoding-characters position 4. Traditionally `&`.
    pub subcomponent: char,
}

impl Default for Separators {
    fn default() -> Self {
        Separators {
            field: '|',
            component: '^',
            repetition: '~',
            escape: '\\',
            subcomponent: '&',
        }
    }
}

/// One parsed ER7 message: its delimiter set plus, in order, every segment
/// line (MSH first).
#[derive(Debug, Clone)]
pub struct Message {
    /// The delimiter set declared by this message's MSH-1/MSH-2.
    pub separators: Separators,
    /// Segments in the order they appeared in the input.
    pub segments: Vec<Segment>,
}

/// One segment line, e.g. `PID|1||241900||MEDIANO^FOUAZ`.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Segment ID, e.g. "PID".
    pub id: String,
    /// `fields[0]` is SEG-1, `fields[1]` is SEG-2, and so on.
    pub fields: Vec<Field>,
}

/// One field's value: its repetitions (split on the `~` repetition
/// separator). A non-repeating field has exactly one [`Repeat`].
#[derive(Debug, Clone)]
pub struct Field {
    /// One or more repetitions, in order.
    pub repeats: Vec<Repeat>,
}

/// One field repetition: its components (split on the `^` component
/// separator). A field with no components has exactly one [`Component`].
#[derive(Debug, Clone)]
pub struct Repeat {
    /// Components, in order.
    pub components: Vec<Component>,
}

/// One component: its subcomponents (split on the `&` subcomponent
/// separator). A component with no subcomponents has exactly one entry.
#[derive(Debug, Clone)]
pub struct Component {
    /// Escape-decoded subcomponent values.
    pub subcomponents: Vec<String>,
}

impl Component {
    /// True when every subcomponent is an empty string, i.e. the field
    /// carried no value at this position (`||`, not the explicit null `""`).
    pub fn is_empty(&self) -> bool {
        self.subcomponents.iter().all(|s| s.is_empty())
    }
}

impl Repeat {
    /// True when every component is [`Component::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.components.iter().all(Component::is_empty)
    }
}

impl Field {
    /// True when every repetition is [`Repeat::is_empty`]; such fields are
    /// omitted from the XML output rather than rendered as an empty element.
    pub fn is_empty(&self) -> bool {
        self.repeats.iter().all(Repeat::is_empty)
    }

    /// A field holding one literal, pre-decoded value (used for MSH-1/MSH-2,
    /// which must not be split on delimiters or escape-decoded).
    fn literal(value: String) -> Self {
        Field {
            repeats: vec![Repeat {
                components: vec![Component {
                    subcomponents: vec![value],
                }],
            }],
        }
    }
}

impl Segment {
    /// First subcomponent of the first repetition of SEG-`field` component
    /// `component` (both 1-based), if present.
    pub fn value(&self, field: usize, component: usize) -> Option<&str> {
        self.fields
            .get(field.checked_sub(1)?)?
            .repeats
            .first()?
            .components
            .get(component.checked_sub(1)?)?
            .subcomponents
            .first()
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    }
}

/// Parse one ER7 message. The first non-empty line must be an MSH segment.
pub fn parse_message(text: &str) -> Result<Message, Hl7Error> {
    let lines: Vec<&str> = text
        .trim_start_matches('\u{feff}')
        .split(['\r', '\n'])
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let first = lines.first().ok_or(Hl7Error::Empty)?;
    if !first.starts_with("MSH") {
        return Err(Hl7Error::MissingMsh);
    }
    let separators = read_separators(first)?;
    let segments = lines
        .iter()
        .map(|l| parse_segment(l, &separators))
        .collect();
    Ok(Message {
        separators,
        segments,
    })
}

/// Read the delimiter set from an MSH header line ("MSH|^~\&|...").
fn read_separators(msh: &str) -> Result<Separators, Hl7Error> {
    let mut chars = msh.chars().skip(3);
    let field = chars
        .next()
        .ok_or_else(|| Hl7Error::BadMshHeader("missing field separator after MSH".into()))?;
    if field.is_alphanumeric() {
        return Err(Hl7Error::BadMshHeader(format!(
            "invalid field separator {field:?} after MSH"
        )));
    }
    let d = Separators::default();
    let enc: Vec<char> = chars.take_while(|&c| c != field).take(4).collect();
    Ok(Separators {
        field,
        component: enc.first().copied().unwrap_or(d.component),
        repetition: enc.get(1).copied().unwrap_or(d.repetition),
        escape: enc.get(2).copied().unwrap_or(d.escape),
        subcomponent: enc.get(3).copied().unwrap_or(d.subcomponent),
    })
}

fn parse_segment(line: &str, seps: &Separators) -> Segment {
    let mut tokens = line.split(seps.field);
    let id = tokens.next().unwrap_or("").to_string();
    let mut fields = Vec::new();
    // MSH-1 is the field separator itself and MSH-2 is the encoding
    // characters; both are taken literally, never split or unescaped.
    // FHS/BHS batch headers share this layout.
    if matches!(id.as_str(), "MSH" | "FHS" | "BHS") {
        fields.push(Field::literal(seps.field.to_string()));
        if let Some(enc) = tokens.next() {
            fields.push(Field::literal(enc.to_string()));
        }
    }
    for raw in tokens {
        fields.push(parse_field(raw, seps));
    }
    Segment { id, fields }
}

fn parse_field(raw: &str, seps: &Separators) -> Field {
    if raw.is_empty() {
        return Field { repeats: vec![] };
    }
    Field {
        repeats: raw
            .split(seps.repetition)
            .map(|r| parse_repeat(r, seps))
            .collect(),
    }
}

fn parse_repeat(raw: &str, seps: &Separators) -> Repeat {
    Repeat {
        components: raw
            .split(seps.component)
            .map(|c| Component {
                subcomponents: raw_subcomponents(c, seps),
            })
            .collect(),
    }
}

fn raw_subcomponents(raw: &str, seps: &Separators) -> Vec<String> {
    raw.split(seps.subcomponent)
        .map(|s| unescape(s, seps))
        .collect()
}

/// Decode ER7 escape sequences. `\F\ \S\ \T\ \R\ \E\` become the field,
/// component, subcomponent, repetition, and escape characters; `\Xhh..\`
/// decodes hex-encoded bytes. Unrecognized sequences (formatting commands
/// such as `\.br\`, `\H\`, locally defined `\Z..\`) are kept literally.
pub fn unescape(s: &str, seps: &Separators) -> String {
    if !s.contains(seps.escape) {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != seps.escape {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        match chars[i + 1..].iter().position(|&c| c == seps.escape) {
            // Unmatched escape character: keep the rest literally.
            None => {
                out.extend(chars[i..].iter());
                break;
            }
            Some(offset) => {
                let j = i + 1 + offset;
                let body: String = chars[i + 1..j].iter().collect();
                match body.as_str() {
                    "F" => out.push(seps.field),
                    "S" => out.push(seps.component),
                    "T" => out.push(seps.subcomponent),
                    "R" => out.push(seps.repetition),
                    "E" => out.push(seps.escape),
                    _ => match body.strip_prefix('X').and_then(decode_hex) {
                        Some(decoded) => out.push_str(&decoded),
                        None => {
                            out.push(seps.escape);
                            out.push_str(&body);
                            out.push(seps.escape);
                        }
                    },
                }
                i = j + 1;
            }
        }
    }
    out
}

fn decode_hex(hex: &str) -> Option<String> {
    if hex.is_empty() || !hex.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut it = hex.chars();
    while let (Some(a), Some(b)) = (it.next(), it.next()) {
        bytes.push((a.to_digit(16)? * 16 + b.to_digit(16)?) as u8);
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_separators() {
        let msg = parse_message("MSH|^~\\&|APP\rPID|1").unwrap();
        assert_eq!(msg.separators, Separators::default());
        assert_eq!(msg.segments.len(), 2);
        assert_eq!(msg.segments[0].value(1, 1), Some("|"));
        assert_eq!(msg.segments[0].value(2, 1), Some("^~\\&"));
        assert_eq!(msg.segments[0].value(3, 1), Some("APP"));
        assert_eq!(msg.segments[1].value(1, 1), Some("1"));
    }

    #[test]
    fn parses_custom_separators() {
        let msg = parse_message("MSH#*!?@#APP#F1*C2!R2").unwrap();
        assert_eq!(
            msg.separators,
            Separators {
                field: '#',
                component: '*',
                repetition: '!',
                escape: '?',
                subcomponent: '@',
            }
        );
        let msh = &msg.segments[0];
        assert_eq!(msh.value(1, 1), Some("#"));
        assert_eq!(msh.value(2, 1), Some("*!?@"));
        assert_eq!(msh.value(4, 1), Some("F1"));
        assert_eq!(msh.value(4, 2), Some("C2"));
        assert_eq!(
            msh.fields[3].repeats[1].components[0].subcomponents[0],
            "R2"
        );
    }

    #[test]
    fn unescapes_standard_sequences() {
        let seps = Separators::default();
        assert_eq!(unescape(r"a\F\b", &seps), "a|b");
        assert_eq!(unescape(r"a\S\b", &seps), "a^b");
        assert_eq!(unescape(r"a\T\b", &seps), "a&b");
        assert_eq!(unescape(r"a\R\b", &seps), "a~b");
        assert_eq!(unescape(r"a\E\b", &seps), r"a\b");
        assert_eq!(unescape(r"\X0D\", &seps), "\r");
        assert_eq!(unescape(r"\X4142\", &seps), "AB");
        // Unknown sequences are preserved literally.
        assert_eq!(unescape(r"line\.br\next", &seps), r"line\.br\next");
        // Unterminated escape is preserved literally.
        assert_eq!(unescape(r"a\Fb", &seps), r"a\Fb");
    }

    #[test]
    fn rejects_non_msh_start() {
        assert!(matches!(parse_message("PID|1"), Err(Hl7Error::MissingMsh)));
        assert!(matches!(parse_message("  \n "), Err(Hl7Error::Empty)));
    }
}
