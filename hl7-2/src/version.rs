//! Which release of HL7 v2 a message speaks, and which dictionary that
//! selects.
//!
//! HL7 v2 is not one format but fourteen releases that share a syntax and
//! disagree about the details — MSH-9 grew a third component, MSH-12 turned
//! from a plain string into a composite, ERR grew from one field to twelve.
//! A parser that assumes one release reads the others slightly wrong, so
//! this crate carries the release around ([`crate::Message::version`]) and
//! looks every field type up through it.
//!
//! The release comes from MSH-12.1. When it is missing, unreadable, or
//! names a release this crate has no dictionary for, resolution falls back
//! to the nearest older known release (see [`Version::nearest`]) rather
//! than failing: a message that says `2.5.2` is far better read as 2.5.1
//! than not at all.

use crate::dictionary::Dictionary;
use std::sync::{Arc, OnceLock};

/// A published release of HL7 v2.
///
/// Ordering follows release order, so `Version::V2_3 < Version::V2_3_1`,
/// which is what makes "nearest older release" a comparison rather than a
/// table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

use Version::{
    V2_1, V2_2, V2_3, V2_3_1, V2_4, V2_5, V2_5_1, V2_6, V2_7, V2_7_1, V2_8, V2_8_1, V2_8_2, V2_9,
};

/// Every release this crate knows, in release order.
pub const ALL: &[Version] = &[
    V2_1, V2_2, V2_3, V2_3_1, V2_4, V2_5, V2_5_1, V2_6, V2_7, V2_7_1, V2_8, V2_8_1, V2_8_2, V2_9,
];

/// The release assumed when a message does not say and the caller does not
/// either. v2.5 is both this crate's complete base dictionary and the
/// release the installed base clusters around.
pub const DEFAULT: Version = V2_5;

/// One bundled dictionary file and its lazily parsed form. Parsing v2.5
/// takes long enough (it is the largest file) that doing it once per
/// process, not once per message, is worth the `OnceLock`.
struct Bundled {
    name: &'static str,
    text: &'static str,
    loaded: OnceLock<Result<Arc<Dictionary>, crate::dictionary::Error>>,
}

/// The bundled dictionary files. Several releases can share one file: a
/// point release that changed nothing this crate models does not need a
/// dictionary of its own ([`Version::bundled`] says which).
static FILES: [Bundled; 11] = [
    Bundled {
        name: "2.1",
        text: include_str!("../schemas/v2.1.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.2",
        text: include_str!("../schemas/v2.2.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.3",
        text: include_str!("../schemas/v2.3.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.3.1",
        text: include_str!("../schemas/v2.3.1.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.4",
        text: include_str!("../schemas/v2.4.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.5",
        text: include_str!("../schemas/v2.5.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.5.1",
        text: include_str!("../schemas/v2.5.1.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.6",
        text: include_str!("../schemas/v2.6.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.7",
        text: include_str!("../schemas/v2.7.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.8",
        text: include_str!("../schemas/v2.8.json"),
        loaded: OnceLock::new(),
    },
    Bundled {
        name: "2.9",
        text: include_str!("../schemas/v2.9.json"),
        loaded: OnceLock::new(),
    },
];

impl Version {
    /// The release string as MSH-12.1 spells it, e.g. `"2.5.1"`.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn from_message(message: &er7::Message) -> Option<Version> {
        Version::nearest(&message.version()?)
    }

    /// The bundled dictionary for this release.
    ///
    /// Parsed on first use and shared thereafter; the returned `Arc` is
    /// cheap to clone and is what a [`crate::Message`] holds. The outcome
    /// is cached either way, so a file that failed to load fails the same way
    /// on every call rather than being parsed again.
    /// # Errors
    ///
    /// [`Error::Dictionary`](crate::Error::Dictionary) when the bundled
    /// file does not load. The bundled files are embedded at compile time and
    /// `bundled_dictionaries_all_load` loads every one, so this would mean
    /// the crate shipped a malformed file; it is reported, never a panic
    /// (`spec/rust-checked-arithmetic` RCA-P-1).
    pub fn dictionary(self) -> Result<Arc<Dictionary>, crate::Error> {
        let file = self.bundled();
        file.loaded
            .get_or_init(|| {
                Dictionary::from_json_resolving(file.text, format!("v{}", file.name), |base| {
                    Version::parse(base).and_then(|base| base.dictionary().ok())
                })
                .map(Arc::new)
            })
            .clone()
            .map_err(crate::Error::Dictionary)
    }

    /// Which bundled file backs this release. Point releases that changed
    /// nothing this crate models share their base release's file. Every
    /// index is a constant inside the array, so the compiler checks it.
    #[must_use]
    fn bundled(self) -> &'static Bundled {
        match self {
            V2_1 => &FILES[0],
            V2_2 => &FILES[1],
            V2_3 => &FILES[2],
            V2_3_1 => &FILES[3],
            V2_4 => &FILES[4],
            V2_5 => &FILES[5],
            V2_5_1 => &FILES[6],
            V2_6 => &FILES[7],
            V2_7 | V2_7_1 => &FILES[8],
            V2_8 | V2_8_1 | V2_8_2 => &FILES[9],
            V2_9 => &FILES[10],
        }
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
    fn default_is_the_documented_default_release() {
        assert_eq!(Version::default(), DEFAULT);
    }

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
    fn an_unknown_version_names_itself_and_the_known_ones() {
        let message = UnknownVersion("9.9".to_string()).to_string();
        assert!(message.contains("unknown HL7 version \"9.9\""), "{message}");
        assert!(
            message.contains("2.5") && message.contains("2.9"),
            "{message}"
        );
    }

    #[test]
    fn bundled_dictionaries_all_load() {
        // Every release resolves to a dictionary that knows MSH, whether it
        // has a file of its own or shares its base release's.
        for &version in ALL {
            let dictionary = version.dictionary().unwrap();
            assert!(
                dictionary.segment_fields("MSH").is_some(),
                "v{version} has no MSH"
            );
        }
    }
}
