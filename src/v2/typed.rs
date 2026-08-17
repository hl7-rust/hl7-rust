//! Struct mode: compile-time types for the feed that does not change.
//!
//! For a stable, long-lived interface, the right shape for a message is a
//! struct — the field names are checked, the types are converted once, and
//! the compiler notices when the code and the interface drift apart. That
//! is what [`FromHl7`] and [`ToHl7`] are, and what
//! `#[derive(FromHl7)]` writes for you when the `derive` feature is on.
//!
//! ## The escape hatch is part of the design
//!
//! Real feeds are stable until they are not: one vendor puts something in
//! `ZPD-2` that no struct models, and the choice becomes re-parse the raw
//! message or rewrite the library. Neither is necessary here. A struct can
//! carry a [`Raw`] field, which holds the whole parsed message alongside
//! the typed data, so the fallback is a method call on the object you
//! already have:
//!
//! ```
//! # #[cfg(feature = "derive")] fn main() -> Result<(), hl7::v2::Error> {
//! use hl7::v2::{FromHl7, Raw};
//!
//! #[derive(FromHl7)]
//! struct Admission {
//!     #[hl7("PID-3.1")]
//!     patient_id: String,
//!     #[hl7("PID-7.1")]
//!     birth_date: Option<String>,
//!     #[hl7("PID-3")]
//!     all_identifiers: Vec<String>,
//!     #[hl7(raw)]
//!     raw: Raw,
//! }
//!
//! let text = "MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1||241900~99~7||SMITH\rZPD|local";
//! let admission: Admission = hl7::v2::parse(text)?.decode()?;
//! assert_eq!(admission.patient_id, "241900");
//! assert_eq!(admission.all_identifiers.len(), 3);
//! assert_eq!(admission.birth_date, None);
//! // The one field no struct models, without a second parse:
//! assert_eq!(admission.raw.get("ZPD-1")?.as_deref(), Some("local"));
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "derive"))] fn main() {}
//! ```

use crate::v2::{Error, Message};

/// A type that can be read out of a message — struct mode's entry point.
///
/// Implement it by hand, or derive it with `#[derive(FromHl7)]` and one
/// `#[hl7(...)]` attribute per field:
///
/// | attribute | meaning |
/// |---|---|
/// | `#[hl7("PID-5.1")]` | read this path through [`FromHl7Value`] |
/// | `#[hl7(nested)]` | the field's own `FromHl7` reads the same message |
/// | `#[hl7(raw)]` | the field is a [`Raw`] holding the whole message |
pub trait FromHl7: Sized {
    /// Read `message` into `Self`.
    fn from_hl7(message: &Message) -> Result<Self, Error>;
}

/// A type that can be written back into a message, the inverse of
/// [`FromHl7`]. Derive it with `#[derive(ToHl7)]`, which uses the same
/// attributes and skips `raw` fields.
pub trait ToHl7 {
    /// Write `self` into `message`, creating fields as needed. Segments
    /// must already exist — see [`crate::v2::Builder`].
    fn to_hl7(&self, message: &mut Message) -> Result<(), Error>;
}

/// A type one struct field can be read from, given a path.
///
/// Implemented for [`String`], the integer and floating-point types,
/// [`bool`], and for `Option<T>` and `Vec<T>` of those: `Option` for a
/// field that may be absent, `Vec` for one that repeats. A `Vec` reads
/// through [`Message::repetitions`], so `#[hl7("PID-3")]` on a `Vec` gives
/// one entry per repetition rather than one string holding all of them.
pub trait FromHl7Value: Sized {
    /// Read the value at `path` out of `message`.
    fn from_hl7_value(message: &Message, path: &str) -> Result<Self, Error>;
}

/// The inverse of [`FromHl7Value`].
pub trait ToHl7Value {
    /// Write `self` to `path` in `message`.
    fn to_hl7_value(&self, message: &mut Message, path: &str) -> Result<(), Error>;
}

/// A scalar that can be converted from the text of one HL7 value.
///
/// This is the piece to implement for a domain type of your own — a
/// `PatientId`, a date type from whichever calendar crate you use — after
/// which `Option<T>` and `Vec<T>` work too, via [`FromHl7Value`].
pub trait FromHl7Text: Sized {
    /// Convert `text`, which is the decoded value at `path`. `path` is for
    /// the error message only.
    fn from_hl7_text(text: &str, path: &str) -> Result<Self, Error>;
}

/// The inverse of [`FromHl7Text`]: how a scalar is written as HL7 text.
pub trait ToHl7Text {
    /// The text to write, or `None` to leave the field empty.
    fn to_hl7_text(&self) -> Option<String>;
}

impl FromHl7Text for String {
    fn from_hl7_text(text: &str, _path: &str) -> Result<String, Error> {
        Ok(text.to_string())
    }
}

impl ToHl7Text for String {
    fn to_hl7_text(&self) -> Option<String> {
        Some(self.clone())
    }
}

impl ToHl7Text for str {
    fn to_hl7_text(&self) -> Option<String> {
        Some(self.to_string())
    }
}

/// `ID`-typed yes/no fields, which HL7 writes as `Y` and `N`. `1`/`0` and
/// `true`/`false` are accepted too, because senders write them; `Y`/`N` is
/// what gets written back.
impl FromHl7Text for bool {
    fn from_hl7_text(text: &str, path: &str) -> Result<bool, Error> {
        match text.trim() {
            "Y" | "y" | "1" | "true" | "TRUE" => Ok(true),
            "N" | "n" | "0" | "false" | "FALSE" => Ok(false),
            other => Err(Error::BadValue {
                path: path.to_string(),
                expected: "Y or N".to_string(),
                found: other.to_string(),
            }),
        }
    }
}

impl ToHl7Text for bool {
    fn to_hl7_text(&self) -> Option<String> {
        Some(if *self { "Y" } else { "N" }.to_string())
    }
}

/// Numbers parse with their own `FromStr`, reported against the path.
macro_rules! numbers {
    ($($type:ty),*) => {$(
        impl FromHl7Text for $type {
            fn from_hl7_text(text: &str, path: &str) -> Result<$type, Error> {
                text.trim().parse().map_err(|_| Error::BadValue {
                    path: path.to_string(),
                    expected: stringify!($type).to_string(),
                    found: text.to_string(),
                })
            }
        }

        impl ToHl7Text for $type {
            fn to_hl7_text(&self) -> Option<String> {
                Some(self.to_string())
            }
        }
    )*};
}

numbers!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

/// One value, required: a path that names nothing is an error, because a
/// struct field that is not `Option` says the interface promises it.
macro_rules! scalars {
    ($($type:ty),*) => {$(
        impl FromHl7Value for $type {
            fn from_hl7_value(message: &Message, path: &str) -> Result<$type, Error> {
                match message.get(path)?.filter(|text| !text.is_empty()) {
                    Some(text) => <$type as FromHl7Text>::from_hl7_text(&text, path),
                    None => Err(Error::MissingField { path: path.to_string() }),
                }
            }
        }

        impl ToHl7Value for $type {
            fn to_hl7_value(&self, message: &mut Message, path: &str) -> Result<(), Error> {
                match <$type as ToHl7Text>::to_hl7_text(self) {
                    Some(text) => message.set(path, &text),
                    None => message.clear(path),
                }
            }
        }

        /// Absent, empty, and the HL7 explicit null all read as `None`.
        impl FromHl7Value for Option<$type> {
            fn from_hl7_value(message: &Message, path: &str) -> Result<Option<$type>, Error> {
                match message.get(path)?.filter(|text| !text.is_empty()) {
                    Some(text) => <$type as FromHl7Text>::from_hl7_text(&text, path).map(Some),
                    None => Ok(None),
                }
            }
        }

        impl ToHl7Value for Option<$type> {
            fn to_hl7_value(&self, message: &mut Message, path: &str) -> Result<(), Error> {
                match self {
                    Some(value) => value.to_hl7_value(message, path),
                    None => message.clear(path),
                }
            }
        }

        /// Every repetition (or every matching segment, for a path that
        /// leaves the occurrence open), in message order.
        impl FromHl7Value for Vec<$type> {
            fn from_hl7_value(message: &Message, path: &str) -> Result<Vec<$type>, Error> {
                message
                    .repetitions(path)?
                    .iter()
                    .filter(|text| !text.is_empty())
                    .map(|text| <$type as FromHl7Text>::from_hl7_text(text, path))
                    .collect()
            }
        }

        impl ToHl7Value for Vec<$type> {
            fn to_hl7_value(&self, message: &mut Message, path: &str) -> Result<(), Error> {
                // Repetitions are written as `path[n]`, so the path must
                // not already fix one.
                for (index, value) in self.iter().enumerate() {
                    let path = crate::v2::typed::with_repetition(path, index + 1)?;
                    value.to_hl7_value(message, &path)?;
                }
                Ok(())
            }
        }
    )*};
}

scalars!(
    String, bool, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

/// Rewrite `PID-3.1` as `PID-3[2].1` so a `Vec` can write its repetitions.
pub(crate) fn with_repetition(path: &str, repetition: usize) -> Result<String, Error> {
    let parsed = er7::Path::parse(path)?;
    let field = parsed
        .field
        .ok_or_else(|| Error::UnwritablePath(format!("{path}: a repeating value needs a field")))?;
    let mut out = parsed.segment.clone();
    if let Some(occurrence) = parsed.segment_occurrence {
        out.push_str(&format!("[{occurrence}]"));
    }
    out.push_str(&format!("-{field}[{repetition}]"));
    if let Some(component) = parsed.component {
        out.push_str(&format!(".{component}"));
        if let Some(subcomponent) = parsed.subcomponent {
            out.push_str(&format!(".{subcomponent}"));
        }
    }
    Ok(out)
}

/// The whole parsed message, kept alongside typed data.
///
/// A struct field of this type is what makes struct mode multi-modal: the
/// typed fields cover the interface as specified, and this covers the day
/// it turns out not to have been. See the module documentation.
#[derive(Debug, Clone)]
pub struct Raw {
    message: Message,
}

impl Raw {
    /// Keep `message` alongside typed data. `#[hl7(raw)]` calls this.
    pub fn new(message: Message) -> Raw {
        Raw { message }
    }

    /// The message itself, with everything on [`Message`] available.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// The value at `path`; see [`Message::get`].
    pub fn get(&self, path: &str) -> Result<Option<String>, Error> {
        self.message.get(path)
    }

    /// Every value at `path`; see [`Message::get_all`].
    pub fn get_all(&self, path: &str) -> Result<Vec<String>, Error> {
        self.message.get_all(path)
    }

    /// The message as a navigable tree; see [`Message::tree`].
    pub fn tree(&self) -> crate::v2::generic::Node {
        self.message.tree()
    }

    /// The message as ER7, exactly as it arrived.
    pub fn to_er7(&self) -> String {
        self.message.to_er7()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1||241900~99||SMITH^JOHN|||M";

    #[derive(Debug)]
    struct Patient {
        id: String,
        all_ids: Vec<String>,
        sequence: u32,
        middle: Option<String>,
        raw: Raw,
    }

    // The shape `#[derive(FromHl7)]` generates, written out so the trait is
    // tested without the derive feature.
    impl FromHl7 for Patient {
        fn from_hl7(message: &Message) -> Result<Patient, Error> {
            Ok(Patient {
                id: FromHl7Value::from_hl7_value(message, "PID-3.1")?,
                all_ids: FromHl7Value::from_hl7_value(message, "PID-3.1")?,
                sequence: FromHl7Value::from_hl7_value(message, "PID-1")?,
                middle: FromHl7Value::from_hl7_value(message, "PID-5.3")?,
                raw: Raw::new(message.clone()),
            })
        }
    }

    #[test]
    fn reads_scalars_repetitions_and_absences() {
        let patient: Patient = crate::v2::parse(TEXT).unwrap().decode().unwrap();
        assert_eq!(patient.id, "241900");
        assert_eq!(patient.all_ids, ["241900", "99"]);
        assert_eq!(patient.sequence, 1);
        assert_eq!(patient.middle, None);
        // And the escape hatch still reaches what the struct does not model.
        assert_eq!(patient.raw.get("PID-8").unwrap().as_deref(), Some("M"));
    }

    #[test]
    fn a_required_field_that_is_absent_is_an_error() {
        let message = crate::v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1").unwrap();
        let error = Patient::from_hl7(&message).unwrap_err();
        assert!(
            matches!(&error, Error::MissingField { path } if path == "PID-3.1"),
            "{error}"
        );
    }

    #[test]
    fn a_value_of_the_wrong_shape_names_the_path() {
        let message = crate::v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|x||9").unwrap();
        let error = Patient::from_hl7(&message).unwrap_err();
        assert!(
            matches!(&error, Error::BadValue { path, .. } if path == "PID-1"),
            "{error}"
        );
        assert!(error.to_string().contains("u32"), "{error}");
    }

    #[test]
    fn writes_scalars_options_and_repetitions_back() {
        let mut message = crate::v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1").unwrap();
        "SMITH"
            .to_string()
            .to_hl7_value(&mut message, "PID-5.1")
            .unwrap();
        7u32.to_hl7_value(&mut message, "PID-1").unwrap();
        true.to_hl7_value(&mut message, "PID-30").unwrap();
        None::<String>.to_hl7_value(&mut message, "PID-8").unwrap();
        vec!["A".to_string(), "B".to_string()]
            .to_hl7_value(&mut message, "PID-3.1")
            .unwrap();
        assert_eq!(message.get("PID-5.1").unwrap().as_deref(), Some("SMITH"));
        assert_eq!(message.get("PID-1").unwrap().as_deref(), Some("7"));
        assert_eq!(message.get("PID-30").unwrap().as_deref(), Some("Y"));
        assert_eq!(message.get_all("PID-3.1").unwrap(), ["A", "B"]);
    }

    #[test]
    fn rewrites_a_path_to_address_one_repetition() {
        assert_eq!(with_repetition("PID-3.1", 2).unwrap(), "PID-3[2].1");
        assert_eq!(with_repetition("OBX[2]-5", 1).unwrap(), "OBX[2]-5[1]");
        assert!(with_repetition("PID", 1).is_err());
    }

    #[test]
    fn booleans_read_the_spellings_senders_use() {
        for (text, expected) in [("Y", true), ("n", false), ("1", true), ("0", false)] {
            assert_eq!(bool::from_hl7_text(text, "PID-30").unwrap(), expected);
        }
        assert!(bool::from_hl7_text("maybe", "PID-30").is_err());
    }
}
