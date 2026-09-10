//! [`Diagnostic`]: one validation finding, serialized as an object.

use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{DiagnosticKind, Severity, Strict};

/// A Serde-enabled [`hl7_2::Diagnostic`]: one finding from
/// [`hl7_2::Message::validate`].
///
/// Serializes as an object with four fields, all required on the way back
/// in: `"severity"` ([`Severity`]), `"kind"` ([`DiagnosticKind`]),
/// `"path"` (an `er7` path such as `"OBX[2]-5[1].1"`, a segment name, or
/// `""` for a whole-message finding), and `"detail"` (the sentence).
///
/// Example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde_hl7_v2::Diagnostic;
///
/// let message = hl7_2::parse("MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\r\
///                             PID|1\rOBR|1\rOBX|1|NM|X||7\rZZZ|1")?;
/// let findings: Vec<Diagnostic> = message.validate().into_iter().map(Diagnostic).collect();
///
/// let json = serde_json::to_value(&findings)?;
/// let first = &json[0];
/// assert_eq!(first["severity"], "Warning");
/// assert_eq!(first["kind"], "StructureMismatch");
/// assert!(first["detail"].as_str().unwrap().contains("Z-segments"));
///
/// let back: Vec<Diagnostic> = serde_json::from_value(json)?;
/// assert_eq!(back, findings);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic(pub hl7_2::Diagnostic);

impl From<hl7_2::Diagnostic> for Diagnostic {
    fn from(inner: hl7_2::Diagnostic) -> Diagnostic {
        Diagnostic(inner)
    }
}

impl From<Diagnostic> for hl7_2::Diagnostic {
    fn from(outer: Diagnostic) -> hl7_2::Diagnostic {
        outer.0
    }
}

impl Deref for Diagnostic {
    type Target = hl7_2::Diagnostic;

    fn deref(&self) -> &hl7_2::Diagnostic {
        &self.0
    }
}

impl DerefMut for Diagnostic {
    fn deref_mut(&mut self) -> &mut hl7_2::Diagnostic {
        &mut self.0
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Serialize for Diagnostic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Diagnostic", 4)?;
        state.serialize_field("severity", &Severity(self.0.severity))?;
        state.serialize_field("kind", &DiagnosticKind(self.0.kind))?;
        state.serialize_field("path", &self.0.path)?;
        state.serialize_field("detail", &self.0.detail)?;
        state.end()
    }
}

const FIELDS: &[&str] = &["severity", "kind", "path", "detail"];

struct DiagnosticVisitor {
    strict: bool,
}

impl<'de> Visitor<'de> for DiagnosticVisitor {
    type Value = Diagnostic;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a Diagnostic object with \"severity\", \"kind\", \"path\", and \"detail\"")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Diagnostic, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut severity: Option<Severity> = None;
        let mut kind: Option<DiagnosticKind> = None;
        let mut path: Option<String> = None;
        let mut detail: Option<String> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "severity" => {
                    if severity.is_some() {
                        return Err(de::Error::duplicate_field("severity"));
                    }
                    severity = Some(map.next_value()?);
                }
                "kind" => {
                    if kind.is_some() {
                        return Err(de::Error::duplicate_field("kind"));
                    }
                    kind = Some(map.next_value()?);
                }
                "path" => {
                    if path.is_some() {
                        return Err(de::Error::duplicate_field("path"));
                    }
                    path = Some(map.next_value()?);
                }
                "detail" => {
                    if detail.is_some() {
                        return Err(de::Error::duplicate_field("detail"));
                    }
                    detail = Some(map.next_value()?);
                }
                _ if self.strict => {
                    return Err(de::Error::unknown_field(&key, FIELDS));
                }
                _ => {
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        Ok(Diagnostic(hl7_2::Diagnostic {
            severity: severity
                .ok_or_else(|| de::Error::missing_field("severity"))?
                .0,
            kind: kind.ok_or_else(|| de::Error::missing_field("kind"))?.0,
            path: path.ok_or_else(|| de::Error::missing_field("path"))?,
            detail: detail.ok_or_else(|| de::Error::missing_field("detail"))?,
        }))
    }
}

impl<'de> Deserialize<'de> for Diagnostic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Diagnostic", FIELDS, DiagnosticVisitor { strict: false })
    }
}

impl<'de> Deserialize<'de> for Strict<Diagnostic> {
    /// See [`Strict`] (rule S13): an unrecognized key is a
    /// `serde::de::Error::unknown_field` rather than being ignored.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_struct("Diagnostic", FIELDS, DiagnosticVisitor { strict: true })
            .map(Strict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Diagnostic {
        Diagnostic(hl7_2::Diagnostic {
            severity: hl7_2::Severity::Error,
            kind: hl7_2::validate::Kind::ValueFormat,
            path: "OBX[1]-5[1]".to_string(),
            detail: "NM holds letters".to_string(),
        })
    }

    #[test]
    fn round_trips_a_diagnostic() {
        let json = serde_json::to_string(&sample()).unwrap();
        assert_eq!(
            json,
            r#"{"severity":"Error","kind":"ValueFormat","path":"OBX[1]-5[1]","detail":"NM holds letters"}"#
        );
        let back: Diagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sample());
    }

    #[test]
    fn round_trips_real_findings() {
        let message =
            hl7_2::parse("MSH|^~\\&|LAB||EPIC||20240101||ORU^R01|1|P|2.5\rZZZ|1\rOBX|1|NM|X||abc")
                .unwrap();
        let findings: Vec<Diagnostic> = message.validate().into_iter().map(Diagnostic).collect();
        assert!(!findings.is_empty());
        let json = serde_json::to_string(&findings).unwrap();
        let back: Vec<Diagnostic> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, findings);
    }

    #[test]
    fn rejects_a_missing_field() {
        let err =
            serde_json::from_str::<Diagnostic>(r#"{"severity":"Error","kind":"Header","path":""}"#)
                .unwrap_err();
        assert!(err.to_string().contains("detail"), "{err}");
    }

    #[test]
    fn rejects_a_duplicate_field() {
        let err = serde_json::from_str::<Diagnostic>(
            r#"{"severity":"Error","severity":"Warning","kind":"Header","path":"","detail":""}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("duplicate"), "{err}");
    }

    #[test]
    fn ignores_unknown_fields() {
        let json =
            r#"{"severity":"Warning","kind":"SegmentUnknown","path":"ZZZ","detail":"x","extra":1}"#;
        let back: Diagnostic = serde_json::from_str(json).unwrap();
        assert_eq!(back.kind, hl7_2::validate::Kind::SegmentUnknown);
    }

    #[test]
    fn strict_rejects_an_unknown_field() {
        let json =
            r#"{"severity":"Warning","kind":"SegmentUnknown","path":"ZZZ","detail":"x","extra":1}"#;
        let err = serde_json::from_str::<Strict<Diagnostic>>(json).unwrap_err();
        assert!(err.to_string().contains("extra"), "{err}");
    }

    #[test]
    fn strict_still_requires_every_field_the_plain_type_does() {
        let err =
            serde_json::from_str::<Strict<Diagnostic>>(r#"{"severity":"Error"}"#).unwrap_err();
        assert!(err.to_string().contains("kind"), "{err}");
    }

    #[test]
    fn deref_and_display_reach_the_inner_value() {
        let diagnostic = sample();
        assert_eq!(diagnostic.path, "OBX[1]-5[1]");
        assert_eq!(diagnostic.to_string(), diagnostic.0.to_string());
    }
}
