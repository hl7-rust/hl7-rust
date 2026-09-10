//! [`Version`]: an HL7 v2 release, serialized as the string MSH-12 spells.

use std::fmt;
use std::ops::Deref;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A Serde-enabled [`hl7_2::Version`].
///
/// Serializes as the release string exactly as MSH-12.1 spells it —
/// `"2.5"`, `"2.5.1"`, `"2.9"` — through [`hl7_2::Version::as_str`], and
/// deserializes through [`hl7_2::Version::parse`], so only a release
/// `hl7-2` knows is accepted. (A message whose MSH-12 names a release
/// `hl7-2` does not know is still readable: [`crate::Message`] resolves
/// that the way `hl7_2::parse` does, to the nearest older release, before
/// this type is ever involved.)
///
/// Example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde_hl7_v2::Version;
///
/// let json = serde_json::to_string(&Version(hl7_2::Version::V2_5_1))?;
/// assert_eq!(json, r#""2.5.1""#);
///
/// let back: Version = serde_json::from_str(&json)?;
/// assert_eq!(back.0, hl7_2::Version::V2_5_1);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(pub hl7_2::Version);

impl From<hl7_2::Version> for Version {
    fn from(inner: hl7_2::Version) -> Version {
        Version(inner)
    }
}

impl From<Version> for hl7_2::Version {
    fn from(outer: Version) -> hl7_2::Version {
        outer.0
    }
}

impl Deref for Version {
    type Target = hl7_2::Version;

    fn deref(&self) -> &hl7_2::Version {
        &self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

struct VersionVisitor;

impl Visitor<'_> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an HL7 v2 release string such as \"2.5\" or \"2.5.1\"")
    }

    fn visit_str<E>(self, value: &str) -> Result<Version, E>
    where
        E: de::Error,
    {
        hl7_2::Version::parse(value)
            .map(Version)
            .ok_or_else(|| de::Error::invalid_value(de::Unexpected::Str(value), &self))
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_release() {
        for &release in hl7_2::version::ALL {
            let json = serde_json::to_string(&Version(release)).unwrap();
            assert_eq!(json, format!("\"{}\"", release.as_str()));
            let back: Version = serde_json::from_str(&json).unwrap();
            assert_eq!(back.0, release);
        }
    }

    #[test]
    fn rejects_an_unknown_release() {
        let err = serde_json::from_str::<Version>(r#""2.5.2""#).unwrap_err();
        assert!(err.to_string().contains("2.5.2"), "{err}");
    }

    #[test]
    fn rejects_a_non_string() {
        assert!(serde_json::from_str::<Version>("2.5").is_err());
    }

    #[test]
    fn deref_and_display_reach_the_inner_value() {
        let version = Version(hl7_2::Version::V2_3_1);
        assert_eq!(version.as_str(), "2.3.1");
        assert_eq!(version.to_string(), "2.3.1");
    }
}
