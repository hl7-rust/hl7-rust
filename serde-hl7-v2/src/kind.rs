//! The three C-like enums `hl7-2` exposes, each serialized as its variant
//! name: [`NodeKind`], [`Severity`], and [`DiagnosticKind`].
//!
//! All three follow the same rule (S7 in the spec): the wire value is the
//! Rust variant identifier, written with `serialize_str` rather than
//! `serialize_unit_variant`, so the value reads the same in every format —
//! `"Segment"` is `"Segment"` in JSON, YAML, and a self-describing binary
//! format alike, at the cost of the compact integer index a binary format's
//! `serialize_unit_variant` could otherwise use. An unrecognized string is a
//! deserialize error naming the string and listing what it could have been.

use std::fmt;
use std::ops::Deref;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Generates one string-valued enum wrapper: the newtype, `From` both ways,
/// `Deref`, `Display`, and `Serialize`/`Deserialize` over a fixed table of
/// variant names.
macro_rules! variant_string {
    (
        $(#[$meta:meta])*
        $name:ident wraps $inner:path,
        expecting $expecting:literal,
        variants { $( $variant:ident => $text:literal ),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(pub $inner);

        impl From<$inner> for $name {
            fn from(inner: $inner) -> $name {
                $name(inner)
            }
        }

        impl From<$name> for $inner {
            fn from(outer: $name) -> $inner {
                outer.0
            }
        }

        impl Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &$inner {
                &self.0
            }
        }

        impl $name {
            /// Every wire value this type accepts, in declaration order.
            pub const NAMES: &'static [&'static str] = &[$($text),+];

            /// The wire value for this variant.
            #[must_use]
            pub fn as_str(self) -> &'static str {
                match self.0 {
                    $( <$inner>::$variant => $text, )+
                }
            }

            /// The variant named by `text`, or `None`.
            #[must_use]
            pub fn parse(text: &str) -> Option<$name> {
                match text {
                    $( $text => Some($name(<$inner>::$variant)), )+
                    _ => None,
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct V;

                impl Visitor<'_> for V {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str($expecting)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                    where
                        E: de::Error,
                    {
                        $name::parse(value)
                            .ok_or_else(|| de::Error::unknown_variant(value, $name::NAMES))
                    }
                }

                deserializer.deserialize_str(V)
            }
        }
    };
}

variant_string! {
    /// A Serde-enabled [`hl7_2::generic::Kind`]: which level of the tree a
    /// [`crate::Node`] sits at.
    ///
    /// Serializes as one of `"Group"`, `"Segment"`, `"Field"`,
    /// `"Component"`, `"Subcomponent"`.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v2::NodeKind;
    ///
    /// let json = serde_json::to_string(&NodeKind(hl7_2::generic::Kind::Field))?;
    /// assert_eq!(json, r#""Field""#);
    /// let back: NodeKind = serde_json::from_str(&json)?;
    /// assert_eq!(back.0, hl7_2::generic::Kind::Field);
    /// # Ok(())
    /// # }
    /// ```
    NodeKind wraps hl7_2::generic::Kind,
    expecting "one of \"Group\", \"Segment\", \"Field\", \"Component\", \"Subcomponent\"",
    variants {
        Group => "Group",
        Segment => "Segment",
        Field => "Field",
        Component => "Component",
        Subcomponent => "Subcomponent",
    }
}

variant_string! {
    /// A Serde-enabled [`hl7_2::Severity`]: how much a [`crate::Diagnostic`]
    /// matters.
    ///
    /// Serializes as `"Error"` or `"Warning"`.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v2::Severity;
    ///
    /// let json = serde_json::to_string(&Severity(hl7_2::Severity::Warning))?;
    /// assert_eq!(json, r#""Warning""#);
    /// # Ok(())
    /// # }
    /// ```
    Severity wraps hl7_2::Severity,
    expecting "\"Error\" or \"Warning\"",
    variants {
        Error => "Error",
        Warning => "Warning",
    }
}

variant_string! {
    /// A Serde-enabled [`hl7_2::validate::Kind`]: what kind of problem a
    /// [`crate::Diagnostic`] reports.
    ///
    /// Serializes as the variant name — `"Header"`, `"StructureUnknown"`,
    /// `"StructureMismatch"`, `"SegmentMissing"`, `"SegmentUnknown"`,
    /// `"FieldUnknown"`, `"ComponentUnknown"`, or `"ValueFormat"`.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v2::DiagnosticKind;
    ///
    /// let kind: DiagnosticKind = serde_json::from_str(r#""SegmentMissing""#)?;
    /// assert_eq!(kind.0, hl7_2::validate::Kind::SegmentMissing);
    /// # Ok(())
    /// # }
    /// ```
    DiagnosticKind wraps hl7_2::validate::Kind,
    expecting "a diagnostic kind such as \"SegmentMissing\" or \"ValueFormat\"",
    variants {
        Header => "Header",
        StructureUnknown => "StructureUnknown",
        StructureMismatch => "StructureMismatch",
        SegmentMissing => "SegmentMissing",
        SegmentUnknown => "SegmentUnknown",
        FieldUnknown => "FieldUnknown",
        ComponentUnknown => "ComponentUnknown",
        ValueFormat => "ValueFormat",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_kind_round_trips_every_variant() {
        for text in NodeKind::NAMES {
            let kind = NodeKind::parse(text).unwrap();
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(json, format!("\"{text}\""));
            let back: NodeKind = serde_json::from_str(&json).unwrap();
            assert_eq!(back, kind);
        }
    }

    #[test]
    fn severity_round_trips_every_variant() {
        for text in Severity::NAMES {
            let severity = Severity::parse(text).unwrap();
            let back: Severity =
                serde_json::from_str(&serde_json::to_string(&severity).unwrap()).unwrap();
            assert_eq!(back, severity);
        }
    }

    #[test]
    fn diagnostic_kind_round_trips_every_variant() {
        for text in DiagnosticKind::NAMES {
            let kind = DiagnosticKind::parse(text).unwrap();
            let back: DiagnosticKind =
                serde_json::from_str(&serde_json::to_string(&kind).unwrap()).unwrap();
            assert_eq!(back, kind);
        }
    }

    #[test]
    fn rejects_an_unknown_variant() {
        let err = serde_json::from_str::<NodeKind>(r#""Repetition""#).unwrap_err();
        assert!(err.to_string().contains("Repetition"), "{err}");
        let err = serde_json::from_str::<Severity>(r#""Fatal""#).unwrap_err();
        assert!(err.to_string().contains("Fatal"), "{err}");
        let err = serde_json::from_str::<DiagnosticKind>(r#""Typo""#).unwrap_err();
        assert!(err.to_string().contains("Typo"), "{err}");
    }

    #[test]
    fn a_variant_is_written_as_a_string_not_an_index() {
        // S7: `serialize_str`, so the value is legible in every format.
        let value = serde_json::to_value(NodeKind(hl7_2::generic::Kind::Group)).unwrap();
        assert!(value.is_string());
    }
}
