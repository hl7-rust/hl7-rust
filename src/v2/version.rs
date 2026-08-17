//! Which release of HL7 v2 a message speaks, and which dictionary that
//! selects.
//!
//! HL7 v2 is not one format but fourteen releases that share a syntax and
//! disagree about the details — MSH-9 grew a third component, MSH-12 turned
//! from a plain string into a composite, ERR grew from one field to twelve.
//! A parser that assumes one release reads the others slightly wrong, so
//! this crate carries the release around ([`crate::v2::Message::version`]) and
//! looks every field type up through it.
//!
//! The release comes from MSH-12.1. When it is missing, unreadable, or
//! names a release this crate has no dictionary for, resolution falls back
//! to the nearest older known release (see [`Version::nearest`]) rather
//! than failing: a message that says `2.5.2` is far better read as 2.5.1
//! than not at all.

use crate::v2::dictionary::Dictionary;
use std::sync::{Arc, OnceLock};

/// A published release of HL7 v2.
///
/// Ordering follows release order, so `Version::V2_3 < Version::V2_3_1`,
/// which is what makes "nearest older release" a comparison rather than a
/// table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum Version {
    /// HL7 v2.1 (1990).
    V2_1,
    /// HL7 v2.2 (1994).
    V2_2,
    /// HL7 v2.3 (1997).
    V2_3,
    /// HL7 v2.3.1 (1999), which introduced MSH-9.3, the message structure.
    V2_3_1,
    /// HL7 v2.4 (2000).
    V2_4,
    /// HL7 v2.5 (2003). This crate's base dictionary and its default.
    V2_5,
    /// HL7 v2.5.1 (2007), the release most US interfaces still speak.
    V2_5_1,
    /// HL7 v2.6 (2007).
    V2_6,
    /// HL7 v2.7 (2011).
    V2_7,
    /// HL7 v2.7.1 (2012).
    V2_7_1,
    /// HL7 v2.8 (2014).
    V2_8,
    /// HL7 v2.8.1 (2014).
    V2_8_1,
    /// HL7 v2.8.2 (2015).
    V2_8_2,
    /// HL7 v2.9 (2019).
    V2_9,
}

use Version::*;

/// Every release this crate knows, in release order.
pub const ALL: &[Version] = &[
    V2_1, V2_2, V2_3, V2_3_1, V2_4, V2_5, V2_5_1, V2_6, V2_7, V2_7_1, V2_8, V2_8_1, V2_8_2, V2_9,
];

/// The release assumed when a message does not say and the caller does not
/// either. v2.5 is both this crate's complete base dictionary and the
/// release the installed base clusters around.
pub const DEFAULT: Version = V2_5;

/// The bundled dictionary files, in the same order as [`FILES`]' contents.
/// Several releases can share one file: a point release that changed
/// nothing this crate models does not need a dictionary of its own.
const FILES: &[(&str, &str)] = &[
    ("2.1", include_str!("../../schemas/v2.1.json")),
    ("2.2", include_str!("../../schemas/v2.2.json")),
    ("2.3", include_str!("../../schemas/v2.3.json")),
    ("2.3.1", include_str!("../../schemas/v2.3.1.json")),
    ("2.4", include_str!("../../schemas/v2.4.json")),
    ("2.5", include_str!("../../schemas/v2.5.json")),
    ("2.5.1", include_str!("../../schemas/v2.5.1.json")),
    ("2.6", include_str!("../../schemas/v2.6.json")),
    ("2.7", include_str!("../../schemas/v2.7.json")),
    ("2.8", include_str!("../../schemas/v2.8.json")),
    ("2.9", include_str!("../../schemas/v2.9.json")),
];

/// One lazily parsed dictionary per bundled file. Parsing v2.5 takes long
/// enough (it is the largest file) that doing it once per process, not once
/// per message, is worth the `OnceLock`.
static LOADED: [OnceLock<Arc<Dictionary>>; FILES.len()] = [const { OnceLock::new() }; FILES.len()];

impl Version {
    /// The release string as MSH-12.1 spells it, e.g. `"2.5.1"`.
    pub fn as_str(self) -> &'static str {
        match self {
            V2_1 => "2.1",
            V2_2 => "2.2",
            V2_3 => "2.3",
            V2_3_1 => "2.3.1",
            V2_4 => "2.4",
            V2_5 => "2.5",
            V2_5_1 => "2.5.1",
            V2_6 => "2.6",
            V2_7 => "2.7",
            V2_7_1 => "2.7.1",
            V2_8 => "2.8",
            V2_8_1 => "2.8.1",
            V2_8_2 => "2.8.2",
            V2_9 => "2.9",
        }
    }

    /// The release named exactly by `text`, or `None`. Use
    /// [`Version::nearest`] when reading a real message, where a version
    /// string this crate does not know should degrade rather than fail.
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.trim();
        ALL.iter().copied().find(|v| v.as_str() == text)
    }

    /// The best release to read `text` as: the exact match if there is one,
    /// otherwise the newest known release no newer than `text`, otherwise
    /// (for a version older than everything, or unreadable) `None`.
    ///
    /// Reading 2.5.2 as 2.5.1 is right far more often than it is wrong:
    /// point releases are additive, so the older dictionary names what it
    /// knows and the rest degrades to generic positional names, which is
    /// exactly the fallback the unknown-segment case already takes.
    pub fn nearest(text: &str) -> Option<Version> {
        if let Some(version) = Version::parse(text) {
            return Some(version);
        }
        let wanted = numeric(text)?;
        ALL.iter()
            .copied()
            .rfind(|v| numeric(v.as_str()).is_some_and(|known| known <= wanted))
    }

    /// The release a parsed message declares in MSH-12.1, resolved through
    /// [`Version::nearest`]. `None` when MSH-12 is absent or unreadable.
    pub fn from_message(message: &er7::Message) -> Option<Version> {
        Version::nearest(&message.version()?)
    }

    /// The bundled dictionary for this release.
    ///
    /// Parsed on first use and shared thereafter; the returned `Arc` is
    /// cheap to clone and is what a [`crate::v2::Message`] holds.
    pub fn dictionary(self) -> Arc<Dictionary> {
        let index = self.file_index();
        LOADED[index]
            .get_or_init(|| {
                let (name, text) = FILES[index];
                // A bundled dictionary that does not parse is a bug in this
                // crate, caught by `bundled_dictionaries_all_load` below,
                // not something a caller can act on.
                let dictionary =
                    Dictionary::from_json_resolving(text, format!("v{name}"), |base| {
                        Version::parse(base).map(Version::dictionary)
                    })
                    .unwrap_or_else(|error| {
                        panic!("bundled dictionary v{name} is invalid: {error}")
                    });
                Arc::new(dictionary)
            })
            .clone()
    }

    /// Which bundled file backs this release. Point releases that changed
    /// nothing this crate models share their base release's file.
    fn file_index(self) -> usize {
        let name = match self {
            V2_7_1 => "2.7",
            V2_8_1 | V2_8_2 => "2.8",
            other => other.as_str(),
        };
        FILES
            .iter()
            .position(|(file, _)| *file == name)
            .expect("every release maps to a bundled file")
    }
}

impl Default for Version {
    fn default() -> Version {
        DEFAULT
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Version {
    type Err = UnknownVersion;

    fn from_str(text: &str) -> Result<Version, UnknownVersion> {
        Version::parse(text).ok_or_else(|| UnknownVersion(text.to_string()))
    }
}

/// The error from `"2.4.7".parse::<Version>()`: a version string that is
/// not one of the releases this crate knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownVersion(pub String);

impl std::fmt::Display for UnknownVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown HL7 version {:?}; known versions are {}",
            self.0,
            ALL.iter()
                .map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        )
    }
}

impl std::error::Error for UnknownVersion {}

/// A dotted version as comparable numbers: `"2.5.1"` becomes `[2, 5, 1]`,
/// padded so `2.5` and `2.5.1` compare as `[2,5,0]` and `[2,5,1]`.
/// `None` when the text does not begin with a number.
fn numeric(text: &str) -> Option<[u32; 3]> {
    let mut parts = [0u32; 3];
    let mut any = false;
    for (slot, part) in parts.iter_mut().zip(text.trim().split('.')) {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            break;
        }
        *slot = digits.parse().ok()?;
        any = true;
    }
    any.then_some(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_release_string() {
        for &version in ALL {
            assert_eq!(Version::parse(version.as_str()), Some(version));
            assert_eq!(version.as_str().parse::<Version>(), Ok(version));
        }
        assert_eq!(Version::parse(" 2.5.1 "), Some(V2_5_1));
        assert!("2.5.2".parse::<Version>().is_err());
    }

    #[test]
    fn falls_back_to_the_nearest_older_release() {
        // Point releases this crate does not model read as their base.
        assert_eq!(Version::nearest("2.5.2"), Some(V2_5_1));
        assert_eq!(Version::nearest("2.4.1"), Some(V2_4));
        // A release newer than anything known reads as the newest known.
        assert_eq!(Version::nearest("3.0"), Some(V2_9));
        // A release older than anything known has no sensible answer.
        assert_eq!(Version::nearest("2.0"), None);
        assert_eq!(Version::nearest("HL7"), None);
        assert_eq!(Version::nearest(""), None);
    }

    #[test]
    fn reads_the_release_out_of_msh_12() {
        let message = er7::parse("MSH|^~\\&|A||||1||ACK|1|P|2.3.1\rMSA|AA|1").unwrap();
        assert_eq!(Version::from_message(&message), Some(V2_3_1));
        let message = er7::parse("MSH|^~\\&|A||||1||ACK|1|P|\rMSA|AA|1").unwrap();
        assert_eq!(Version::from_message(&message), None);
    }

    #[test]
    fn bundled_dictionaries_all_load() {
        // Every release resolves to a dictionary that knows MSH, whether it
        // has a file of its own or shares its base release's.
        for &version in ALL {
            let dictionary = version.dictionary();
            assert!(
                dictionary.segment_fields("MSH").is_some(),
                "v{version} has no MSH"
            );
        }
    }
}
