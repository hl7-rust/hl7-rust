//! [`NullFlavor`]: why a value is explicitly absent, serialized as its
//! code.

use std::fmt;
use std::ops::Deref;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A Serde-enabled [`hl7_3::NullFlavor`].
///
/// Serializes as the `nullFlavor` code, a bare string — `"NI"`, `"UNK"`,
/// `"ASKU"`, `"NASK"`, `"NAV"`, `"NA"`, `"OTH"` — through
/// [`hl7_3::NullFlavor::as_code`], and deserializes through
/// [`hl7_3::NullFlavor::parse`], which never fails: a code outside those
/// seven comes back as [`hl7_3::NullFlavor::Unrecognized`] carrying the
/// code as written, exactly as `hl7-3` itself reads an attribute. So,
/// unlike this crate's other string-shaped values, there is no unknown
/// variant to reject here.
///
/// Example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde_hl7_v3::NullFlavor;
///
/// assert_eq!(serde_json::to_string(&NullFlavor(hl7_3::NullFlavor::Unknown))?, r#""UNK""#);
/// let odd: NullFlavor = serde_json::from_str(r#""TRC""#)?;
/// assert_eq!(odd.0, hl7_3::NullFlavor::Unrecognized("TRC".into()));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullFlavor(pub hl7_3::NullFlavor);

impl From<hl7_3::NullFlavor> for NullFlavor {
    fn from(inner: hl7_3::NullFlavor) -> NullFlavor {
        NullFlavor(inner)
    }
}

impl From<NullFlavor> for hl7_3::NullFlavor {
    fn from(outer: NullFlavor) -> hl7_3::NullFlavor {
        outer.0
    }
}

impl Deref for NullFlavor {
    type Target = hl7_3::NullFlavor;

    fn deref(&self) -> &hl7_3::NullFlavor {
        &self.0
    }
}

impl fmt::Display for NullFlavor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_code())
    }
}

impl Serialize for NullFlavor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_code())
    }
}

struct NullFlavorVisitor;

impl Visitor<'_> for NullFlavorVisitor {
    type Value = NullFlavor;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a nullFlavor code such as \"NI\" or \"UNK\"")
    }

    fn visit_str<E>(self, value: &str) -> Result<NullFlavor, E>
    where
        E: de::Error,
    {
        Ok(NullFlavor(hl7_3::NullFlavor::parse(value)))
    }
}

impl<'de> Deserialize<'de> for NullFlavor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(NullFlavorVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_named_code() {
        for code in ["NI", "UNK", "ASKU", "NASK", "NAV", "NA", "OTH"] {
            let flavor = NullFlavor(hl7_3::NullFlavor::parse(code));
            assert!(!matches!(flavor.0, hl7_3::NullFlavor::Unrecognized(_)));
            let json = serde_json::to_string(&flavor).unwrap();
            assert_eq!(json, format!("\"{code}\""));
            let back: NullFlavor = serde_json::from_str(&json).unwrap();
            assert_eq!(back, flavor);
        }
    }

    #[test]
    fn an_unrecognized_code_is_carried_not_rejected() {
        let back: NullFlavor = serde_json::from_str(r#""MSK""#).unwrap();
        assert_eq!(back.0, hl7_3::NullFlavor::Unrecognized("MSK".into()));
        assert_eq!(serde_json::to_string(&back).unwrap(), r#""MSK""#);
    }

    #[test]
    fn rejects_a_non_string() {
        assert!(serde_json::from_str::<NullFlavor>("1").is_err());
    }

    #[test]
    fn display_is_the_code() {
        assert_eq!(NullFlavor(hl7_3::NullFlavor::NotAsked).to_string(), "NASK");
    }
}
