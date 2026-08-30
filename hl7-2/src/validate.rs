//! Checking a message against the dictionary it claims to speak.
//!
//! The sibling conversion crates are explicitly not validators, and this
//! crate does not become one by accident: parsing stays fallback-first, and
//! [`crate::Message::validate`] is a separate call that reports and never
//! refuses. What makes it worth having here is that schema mode already
//! requires the caller to state the shape of their messages — once that
//! shape exists, "does this message match it?" is a question with an
//! answer, and an ingest pipeline that wants the answer as a hard failure
//! can ask for it with [`crate::Options::strict`].
//!
//! The two severities divide along whose problem it is:
//!
//! - [`Severity::Error`] — the message contradicts the dictionary it
//!   claims: a required segment is missing, the segments do not fit the
//!   structure, a numeric field holds letters. Strict mode rejects these.
//! - [`Severity::Warning`] — the dictionary does not cover the message: an
//!   unknown segment, a field past the end of the table, a structure this
//!   crate has no grammar for. These are usually a local extension or a
//!   coverage gap, not a malformed message, so strict mode allows them.

use crate::Message;
use crate::dictionary::{Dictionary, Item, VARIABLE};
use er7::{Segment, Separators};
use std::fmt;

/// How much a diagnostic matters; see the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The message contradicts its dictionary. Strict mode rejects these.
    Error,
    /// The dictionary does not describe part of the message.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        })
    }
}

/// What kind of problem a diagnostic reports, for callers that route on it
/// rather than reading the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A header field a message cannot do without is empty.
    Header,
    /// The dictionary has no grammar for this message structure.
    StructureUnknown,
    /// The segments do not fit the structure's grammar.
    StructureMismatch,
    /// A segment the structure requires is absent.
    SegmentMissing,
    /// A segment the dictionary does not define.
    SegmentUnknown,
    /// A field beyond the end of the segment's definition.
    FieldUnknown,
    /// A component beyond the end of the data type's definition.
    ComponentUnknown,
    /// A value that does not match its data type's format.
    ValueFormat,
}

/// One finding: what, where, and how much it matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// How much this matters.
    pub severity: Severity,
    /// What kind of problem this is.
    pub kind: Kind,
    /// Where it is, as an `er7` path — `OBX[2]-5[1].1` — or a segment name,
    /// or empty for a whole-message finding.
    pub path: String,
    /// What is wrong, in a sentence.
    pub detail: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}: {}", self.severity, self.detail)
        } else {
            write!(f, "{}: {}: {}", self.severity, self.path, self.detail)
        }
    }
}

impl Diagnostic {
    fn error(kind: Kind, path: impl Into<String>, detail: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            kind,
            path: path.into(),
            detail: detail.into(),
        }
    }

    fn warning(kind: Kind, path: impl Into<String>, detail: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            kind,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

/// Check `message` against its dictionary. See [`crate::Message::validate`].
#[must_use]
pub fn validate(message: &Message) -> Vec<Diagnostic> {
    let dictionary = message.dictionary();
    let mut found = Vec::new();
    header(message, &mut found);
    structure(message, dictionary, &mut found);
    let mut occurrences: std::collections::BTreeMap<&str, usize> =
        std::collections::BTreeMap::default();
    for segment in message.segments() {
        let occurrence = occurrences.entry(segment.name.as_str()).or_default();
        *occurrence += 1;
        check_segment(
            segment,
            *occurrence,
            dictionary,
            message.separators(),
            &mut found,
        );
    }
    found
}

/// The header fields a receiver cannot route without.
fn header(message: &Message, found: &mut Vec<Diagnostic>) {
    for (path, what) in [
        ("MSH-9.1", "the message type"),
        ("MSH-10", "the message control ID"),
    ] {
        let empty = message
            .get(path)
            .ok()
            .flatten()
            .is_none_or(|value| value.trim().is_empty());
        if empty {
            found.push(Diagnostic::error(
                Kind::Header,
                path,
                format!("{path} ({what}) is empty"),
            ));
        }
    }
    let declared = message.get("MSH-12.1").ok().flatten().unwrap_or_default();
    if declared.trim().is_empty() {
        found.push(Diagnostic::warning(
            Kind::Header,
            "MSH-12",
            format!(
                "MSH-12 (version ID) is empty; reading the message as v{}",
                message.version()
            ),
        ));
    } else if crate::Version::parse(declared.trim()).is_none() {
        found.push(Diagnostic::warning(
            Kind::Header,
            "MSH-12",
            format!(
                "MSH-12 declares version {declared:?}, which this crate has no dictionary for; \
                 reading the message as v{}",
                message.version()
            ),
        ));
    }
}

/// The message structure: is it known, and do the segments fit it?
fn structure(message: &Message, dictionary: &Dictionary, found: &mut Vec<Diagnostic>) {
    let id = message.structure_id();
    let Some(items) = dictionary.structure(&id) else {
        found.push(Diagnostic::warning(
            Kind::StructureUnknown,
            "MSH-9",
            format!(
                "dictionary {} has no grammar for message structure {id}; \
                 segments are read flat and their order is not checked",
                dictionary.name()
            ),
        ));
        return;
    };
    // Name the specific missing segments before falling back to the general
    // "does not fit": "MSA is missing" is actionable where "does not match
    // the ACK structure" is a puzzle.
    let mut missing = false;
    for item in items {
        if item.required() && !starts_present(item, message) {
            missing = true;
            found.push(Diagnostic::error(
                Kind::SegmentMissing,
                item.name(),
                match item {
                    Item::Segment { name, .. } => {
                        format!("structure {id} requires a {name} segment")
                    }
                    Item::Group { name, .. } => {
                        format!("structure {id} requires the {name} group")
                    }
                },
            ));
        }
    }
    if !missing && message.layout().is_none() {
        // A local Z-segment is a legal extension that no standard structure
        // describes, and most real interfaces carry one. If the standard
        // segments fit on their own, the message is conformant and only the
        // grouping suffers, so say that instead of rejecting it.
        let standard: Vec<&str> = message
            .segments()
            .map(|segment| segment.name.as_str())
            .filter(|name| !name.starts_with('Z'))
            .collect();
        let extensions = standard.len() < message.segments().count();
        if extensions && crate::structure::group(items, &standard).is_some() {
            found.push(Diagnostic::warning(
                Kind::StructureMismatch,
                "",
                format!(
                    "the standard segments fit structure {id}, but the message also carries \
                     local Z-segments, which no structure describes; segments are read flat"
                ),
            ));
        } else {
            found.push(Diagnostic::error(
                Kind::StructureMismatch,
                "",
                format!(
                    "the segments do not fit structure {id}: an unexpected segment, or one out \
                     of order, or one repeated where the structure does not allow it"
                ),
            ));
        }
    }
}

/// Is any segment that could begin `item` present in the message at all?
fn starts_present(item: &Item, message: &Message) -> bool {
    message
        .segments()
        .any(|segment| item.can_start(&segment.name))
}

/// One segment's fields, components, and values.
// Already at the edge of the line-count lint; threading a `&Separators`
// through to `variable_type` (er7 R26) tipped it over without adding real
// complexity. Splitting this walk up would cost more clarity than the
// count buys back.
#[allow(clippy::too_many_lines)]
fn check_segment(
    segment: &Segment,
    occurrence: usize,
    dictionary: &Dictionary,
    separators: &Separators,
    found: &mut Vec<Diagnostic>,
) {
    let base = format!("{}[{occurrence}]", segment.name);
    let Some(fields) = dictionary.segment_fields(&segment.name) else {
        // A Z-segment is a local extension by definition — the standard
        // says nothing about it, so neither does this.
        if !segment.name.starts_with('Z') {
            found.push(Diagnostic::warning(
                Kind::SegmentUnknown,
                &base,
                format!(
                    "dictionary {} does not define segment {}",
                    dictionary.name(),
                    segment.name
                ),
            ));
        }
        return;
    };
    let variable = dictionary
        .variable_type(segment, separators)
        .map(str::to_string);
    let defined = fields.len();
    for (index, field) in segment.fields.iter().enumerate() {
        if field.is_empty() {
            continue;
        }
        let number = index + 1;
        if number > defined {
            found.push(Diagnostic::warning(
                Kind::FieldUnknown,
                format!("{base}-{number}"),
                format!(
                    "{}-{number} is past the {defined} fields dictionary {} defines for {}",
                    segment.name,
                    dictionary.name(),
                    segment.name
                ),
            ));
            continue;
        }
        let data_type = match dictionary.field_type(&segment.name, number) {
            Some(VARIABLE) => variable.as_deref(),
            other => other,
        };
        let Some(data_type) = data_type else {
            continue;
        };
        for (repeat, repetition) in field.repetitions.iter().enumerate() {
            if repetition.is_empty() || repetition.is_null() {
                continue;
            }
            let path = format!("{base}-{number}[{}]", repeat + 1);
            match dictionary.composite_components(data_type) {
                None => check_value(data_type, &repetition.to_text(separators), &path, found),
                Some(components) => {
                    for (index, component) in repetition.components.iter().enumerate() {
                        if component.is_empty() || component.is_null() {
                            continue;
                        }
                        let path = format!("{path}.{}", index + 1);
                        let Some(component_type) = components.get(index) else {
                            found.push(Diagnostic::warning(
                                Kind::ComponentUnknown,
                                &path,
                                format!(
                                    "component {} is past the {} components dictionary {} \
                                     defines for {data_type}",
                                    index + 1,
                                    components.len(),
                                    dictionary.name()
                                ),
                            ));
                            continue;
                        };
                        // HL7 nests composites one level: a component's own
                        // type may be composite, a subcomponent's is not.
                        match dictionary.composite_components(component_type) {
                            None => check_value(
                                component_type,
                                &component.to_text(separators),
                                &path,
                                found,
                            ),
                            Some(subtypes) => {
                                for (index, subcomponent) in
                                    component.subcomponents.iter().enumerate()
                                {
                                    if subcomponent.is_empty() || subcomponent.is_null() {
                                        continue;
                                    }
                                    if let Some(subtype) = subtypes.get(index) {
                                        check_value(
                                            subtype,
                                            &subcomponent.value(separators),
                                            &format!("{path}.{}", index + 1),
                                            found,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Does `text` look like a `data_type` value?
///
/// Only the types with a machine-checkable shape are checked. `ST`, `TX`,
/// `ID`, `IS` and the rest are constrained by HL7 tables and by length,
/// neither of which this crate models, so it says nothing about them rather
/// than guessing.
fn check_value(data_type: &str, text: &str, path: &str, found: &mut Vec<Diagnostic>) {
    let value = text.trim();
    if value.is_empty() {
        return;
    }
    let ok = match data_type {
        "SI" => value.bytes().all(|b| b.is_ascii_digit()),
        "NM" => is_number(value),
        "DT" => is_date(value),
        "TM" => is_time(value),
        "DTM" => is_datetime(value),
        _ => return,
    };
    if !ok {
        found.push(Diagnostic::error(
            Kind::ValueFormat,
            path,
            format!("{value:?} is not a valid {data_type} value"),
        ));
    }
}

/// `NM`: an optional sign, digits, an optional fractional part.
fn is_number(value: &str) -> bool {
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (whole, fraction) = match digits.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (digits, None),
    };
    !whole.is_empty()
        && whole.bytes().all(|b| b.is_ascii_digit())
        && fraction.is_none_or(|f| f.bytes().all(|b| b.is_ascii_digit()))
}

/// `DT`: `YYYY`, `YYYYMM`, or `YYYYMMDD`.
fn is_date(value: &str) -> bool {
    matches!(value.len(), 4 | 6 | 8) && value.bytes().all(|b| b.is_ascii_digit())
}

/// `TM`: `HH[MM[SS[.S[S[S[S]]]]]]` with an optional `+/-ZZZZ` offset.
fn is_time(value: &str) -> bool {
    let (value, offset) = split_offset(value);
    if !offset {
        return false;
    }
    let (whole, fraction) = match value.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (value, None),
    };
    matches!(whole.len(), 2 | 4 | 6)
        && whole.bytes().all(|b| b.is_ascii_digit())
        && fraction
            .is_none_or(|f| (1..=4).contains(&f.len()) && f.bytes().all(|b| b.is_ascii_digit()))
}

/// `DTM`: a date, then optionally a time, then optionally an offset.
fn is_datetime(value: &str) -> bool {
    let (value, offset) = split_offset(value);
    if !offset {
        return false;
    }
    let (whole, fraction) = match value.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (value, None),
    };
    matches!(whole.len(), 4 | 6 | 8 | 10 | 12 | 14)
        && whole.bytes().all(|b| b.is_ascii_digit())
        && fraction
            .is_none_or(|f| (1..=4).contains(&f.len()) && f.bytes().all(|b| b.is_ascii_digit()))
}

/// Split a trailing `+ZZZZ` / `-ZZZZ` offset off a time, reporting whether
/// what was there (if anything) was well formed.
fn split_offset(value: &str) -> (&str, bool) {
    match value.rfind(['+', '-']) {
        Some(index) if index > 0 => {
            let offset = &value[index + 1..];
            (
                &value[..index],
                offset.len() == 4 && offset.bytes().all(|b| b.is_ascii_digit()),
            )
        }
        _ => (value, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostics(text: &str) -> Vec<Diagnostic> {
        crate::parse(text).unwrap().validate()
    }

    fn kinds(text: &str) -> Vec<Kind> {
        diagnostics(text).into_iter().map(|d| d.kind).collect()
    }

    const ACK: &str = "MSH|^~\\&|A||||20240101||ACK^A01|1|P|2.5\rMSA|AA|1";

    #[test]
    fn a_conforming_message_reports_nothing() {
        assert_eq!(diagnostics(ACK), []);
    }

    #[test]
    fn names_the_missing_required_segment() {
        let found = diagnostics("MSH|^~\\&|A||||20240101||ACK^A01|1|P|2.5");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].kind, Kind::SegmentMissing);
        assert_eq!(found[0].severity, Severity::Error);
        assert!(found[0].detail.contains("MSA"), "{}", found[0]);
    }

    #[test]
    fn reports_segments_out_of_order_as_a_mismatch() {
        let found = diagnostics("MSH|^~\\&|A||||20240101||ACK^A01|1|P|2.5\rERR|x\rMSA|AA|1");
        assert!(
            found.iter().any(|d| d.kind == Kind::StructureMismatch),
            "{found:?}"
        );
    }

    #[test]
    fn unknown_segments_and_fields_are_warnings_not_errors() {
        let found = diagnostics(&format!("{ACK}\rZPD|anything"));
        // A Z-segment is a local extension; nothing to say about it.
        assert_eq!(
            found
                .iter()
                .filter(|d| d.kind == Kind::SegmentUnknown)
                .count(),
            0
        );
        // But it does break the ACK structure, which is an error.
        assert!(found.iter().any(|d| d.kind == Kind::StructureMismatch));

        let found = diagnostics("MSH|^~\\&|A||||20240101||ACK^A01|1|P|2.5\rMSA|AA|1|||||||x");
        let past_end: Vec<&Diagnostic> = found
            .iter()
            .filter(|d| d.kind == Kind::FieldUnknown)
            .collect();
        assert_eq!(past_end.len(), 1, "{found:?}");
        assert_eq!(past_end[0].severity, Severity::Warning);
        assert_eq!(past_end[0].path, "MSA[1]-9");
    }

    #[test]
    fn checks_the_formats_that_have_one() {
        let found = diagnostics("MSH|^~\\&|A||||NOT-A-DATE||ACK^A01|1|P|2.5\rMSA|AA|1||x");
        let formats: Vec<&Diagnostic> = found
            .iter()
            .filter(|d| d.kind == Kind::ValueFormat)
            .collect();
        // MSH-7 is a TS whose first component is a DTM, and MSA-4 is an NM.
        assert_eq!(formats.len(), 2, "{found:?}");
        assert!(formats.iter().all(|d| d.severity == Severity::Error));
        assert_eq!(formats[0].path, "MSH[1]-7[1].1");
        assert_eq!(formats[1].path, "MSA[1]-4[1]");
    }

    #[test]
    fn accepts_the_datetime_shapes_hl7_allows() {
        assert!(is_datetime("2024"));
        assert!(is_datetime("20240101"));
        assert!(is_datetime("20240101093851"));
        assert!(is_datetime("20240101093851.1234"));
        assert!(is_datetime("20240101093851+0100"));
        assert!(is_datetime("20240101093851.5-0500"));
        assert!(!is_datetime("2024010"));
        assert!(!is_datetime("2024-01-01"));
        assert!(!is_datetime("20240101093851+01"));
        assert!(is_time("0938"));
        assert!(is_time("093851.25+0100"));
        assert!(!is_time("9:38"));
        assert!(is_number("-7.25"));
        assert!(!is_number("7,25"));
        assert!(is_date("202401"));
        assert!(!is_date("20240"));
    }

    #[test]
    fn an_unknown_structure_is_a_warning_about_the_dictionary() {
        let found = diagnostics("MSH|^~\\&|A||||20240101||ZZZ^Z01|1|P|2.5");
        assert_eq!(
            kinds("MSH|^~\\&|A||||20240101||ZZZ^Z01|1|P|2.5"),
            [Kind::StructureUnknown]
        );
        assert_eq!(found[0].severity, Severity::Warning);
    }

    #[test]
    fn an_unmodelled_version_is_a_warning_and_the_message_still_reads() {
        let found = diagnostics("MSH|^~\\&|A||||20240101||ACK^A01|1|P|2.5.2\rMSA|AA|1");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].kind, Kind::Header);
        assert!(found[0].detail.contains("2.5.1"), "{}", found[0]);
    }

    #[test]
    fn an_empty_control_id_is_an_error() {
        let found = diagnostics("MSH|^~\\&|A||||20240101||ACK^A01||P|2.5\rMSA|AA|1");
        assert_eq!(found[0].kind, Kind::Header);
        assert_eq!(found[0].severity, Severity::Error);
        assert_eq!(found[0].path, "MSH-10");
    }
}
