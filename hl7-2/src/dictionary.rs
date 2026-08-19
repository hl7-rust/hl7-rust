//! The HL7 v2 dictionary: what a segment's fields mean, what a composite
//! data type is made of, and how a message's segments group.
//!
//! This is the knowledge that turns `PID|1||241900||TEST^FOUAZ` from text
//! into data — that PID-5 is an `XPN`, that an `XPN`'s first component is
//! an `FN`, that an `ORU_R01` nests its `OBX` segments inside
//! `PATIENT_RESULT.ORDER_OBSERVATION.OBSERVATION`. Every one of this
//! crate's three modes reads it: generic mode names tree nodes from it,
//! schema mode *is* it (a caller-supplied dictionary instead of a bundled
//! one), and struct mode uses it to resolve the paths a `#[hl7(...)]`
//! attribute names.
//!
//! A dictionary is JSON, and the same reader loads a bundled release and a
//! dictionary a caller wrote for one vendor's dialect — see
//! `spec/index.md` §3 for the format. Bundled dictionaries live in
//! `schemas/` and are embedded at compile time; v2.5 is complete and every
//! other release is expressed as a delta of it via `"inherits"`.

use crate::json::{self, Value};
use std::collections::BTreeMap;
use std::fmt;

/// The data type this crate uses for a field whose real type is carried in
/// another field: OBX-5, whose type OBX-2 names. Callers that look up a
/// field type will see this sentinel and should ask
/// [`Dictionary::variable_type`] instead.
pub const VARIABLE: &str = "VAR";

/// The placeholder a sparse delta leaves in positions it did not mention —
/// `{"MSH": {"12": "ID"}}` states field 12 and says nothing about fields
/// after the end of the inherited list. [`Dictionary::field_type`] reports
/// these as unknown, which is the same fallback an unlisted segment takes.
const UNSTATED: &str = "";

/// One entry in an abstract message structure: a segment, or a named group
/// of entries, each carrying whether the standard makes it required and
/// whether it may repeat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A segment, e.g. `MSH`.
    Segment {
        /// The three-character segment name.
        name: String,
        /// Whether the structure requires it.
        required: bool,
        /// Whether it may appear more than once here.
        repeats: bool,
    },
    /// A named group, e.g. `ORDER_OBSERVATION`.
    Group {
        /// The group name, as HL7's structure tables spell it.
        name: String,
        /// Whether the structure requires it.
        required: bool,
        /// Whether it may appear more than once here.
        repeats: bool,
        /// What the group contains, in order.
        items: Vec<Item>,
    },
}

impl Item {
    /// The segment or group name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Item::Segment { name, .. } | Item::Group { name, .. } => name,
        }
    }

    /// Whether the structure requires this item.
    #[must_use]
    pub fn required(&self) -> bool {
        match self {
            Item::Segment { required, .. } | Item::Group { required, .. } => *required,
        }
    }

    /// Whether this item may appear more than once in a row.
    #[must_use]
    pub fn repeats(&self) -> bool {
        match self {
            Item::Segment { repeats, .. } | Item::Group { repeats, .. } => *repeats,
        }
    }

    /// Can this item begin with a segment named `segment`?
    ///
    /// For a group this walks its leading optional items plus the first
    /// required one — the group's FIRST set — because an optional leading
    /// segment means a group can start at more than one segment name.
    #[must_use]
    pub fn can_start(&self, segment: &str) -> bool {
        match self {
            Item::Segment { name, .. } => name == segment,
            Item::Group { items, .. } => {
                for item in items {
                    if item.can_start(segment) {
                        return true;
                    }
                    if item.required() {
                        return false;
                    }
                }
                false
            }
        }
    }
}

/// Segment field types, composite component types, and message structures
/// for one HL7 release or one vendor dialect.
///
/// Build one with [`crate::Version::dictionary`] for a bundled release, or
/// [`Dictionary::from_json`] for schema mode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dictionary {
    name: String,
    version: Option<String>,
    types: BTreeMap<String, Vec<String>>,
    segments: BTreeMap<String, Vec<String>>,
    cardinality: BTreeMap<String, Vec<Cardinality>>,
    structures: BTreeMap<String, Vec<Item>>,
    aliases: BTreeMap<String, String>,
}

/// How many times a field may appear, and whether it has to.
///
/// A dictionary generated from XML Schema knows a field's `minOccurs` and
/// `maxOccurs` as well as its data type, and both change what a conversion
/// should emit: a required field is written even when the message leaves it
/// empty, so the position stays visible, and a field that cannot repeat
/// keeps its repetition separator as ordinary text rather than being split
/// into several elements. Hand-written dictionaries usually say nothing
/// about either, and then both default to false — see `spec/index.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cardinality {
    /// The schema requires this field to be present.
    pub required: bool,
    /// The schema lets this field appear more than once.
    pub repeats: bool,
}

impl Dictionary {
    /// An empty dictionary under which everything falls back to generic
    /// positional names. Useful as a base to build on, and as the honest
    /// answer for a message whose dialect is entirely unknown.
    pub fn empty(name: impl Into<String>) -> Dictionary {
        Dictionary {
            name: name.into(),
            ..Dictionary::default()
        }
    }

    /// Where this dictionary came from, for error and diagnostic messages:
    /// `"v2.5"` for a bundled release, or whatever name the caller gave.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The HL7 release this dictionary declares in its `"version"` member,
    /// if any. A vendor dialect need not declare one.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// The component data types of composite type `data_type`, or `None`
    /// when it is primitive (`ST`, `NM`, `DTM`, ...) or simply unknown —
    /// the two are indistinguishable here on purpose, because both mean
    /// "treat the value as a scalar".
    pub fn composite_components(&self, data_type: &str) -> Option<&[String]> {
        self.types.get(data_type).map(Vec::as_slice)
    }

    /// True when `data_type` is a composite this dictionary can break apart.
    #[must_use]
    pub fn is_composite(&self, data_type: &str) -> bool {
        self.types.contains_key(data_type)
    }

    /// The field data types of `segment`, index 0 being field 1. `None` for
    /// a segment the dictionary does not list, including Z-segments.
    pub fn segment_fields(&self, segment: &str) -> Option<&[String]> {
        self.segments.get(segment).map(Vec::as_slice)
    }

    /// The data type of `segment`-`field` (1-based), or `None` when either
    /// the segment or the field number is outside the dictionary, or when a
    /// sparse delta left this position unstated.
    ///
    /// May return the [`VARIABLE`] sentinel; see [`Dictionary::variable_type`].
    pub fn field_type(&self, segment: &str, field: usize) -> Option<&str> {
        let types = self.segment_fields(segment)?;
        match types.get(field.checked_sub(1)?).map(String::as_str) {
            Some(UNSTATED) | None => None,
            found => found,
        }
    }

    /// What the schema says about how often `segment`-`field` (1-based) may
    /// appear.
    ///
    /// Defaults to optional and non-repeating, which is both the XML Schema
    /// default for an unstated `maxOccurs` and the honest answer for a
    /// dictionary that never mentioned cardinality at all.
    #[must_use]
    pub fn field_cardinality(&self, segment: &str, field: usize) -> Cardinality {
        field
            .checked_sub(1)
            .and_then(|index| self.cardinality.get(segment)?.get(index).copied())
            .unwrap_or_default()
    }

    /// Resolve a [`VARIABLE`] field's real type from the message.
    ///
    /// Only OBX-5 works this way: OBX-2 names the data type of the value in
    /// OBX-5, so an `OBX|1|NM|...` carries a number where an `OBX|1|CE|...`
    /// carries a coded element. Returns `None` when OBX-2 is empty or names
    /// a type this dictionary does not know as a composite, in which case
    /// the value is treated as a scalar.
    #[must_use]
    pub fn variable_type(&self, segment: &er7::Segment) -> Option<&str> {
        let named = segment
            .component(2, 1)?
            .subcomponent(1)?
            .raw
            .trim()
            .to_string();
        self.types
            .get_key_value(&named)
            .map(|(key, _)| key.as_str())
    }

    /// The abstract message structure named `id`, e.g. `ORU_R01`.
    pub fn structure(&self, id: &str) -> Option<&[Item]> {
        self.structures.get(id).map(Vec::as_slice)
    }

    /// The message structure ID for a message-type code and trigger event,
    /// e.g. `("ADT", "A04")` -> `ADT_A01`.
    ///
    /// Several trigger events share one structure — an A04 admit and an A08
    /// update are both carried by `ADT_A01` — and which ones is dictionary
    /// knowledge, so it lives in the `"aliases"` section rather than in
    /// code. Resolution order: an alias, then a structure named
    /// `CODE_TRIGGER`, then one named `CODE` (which is how `ACK^A01`
    /// reaches `ACK`), then `CODE_TRIGGER` unresolved, so an unknown
    /// message type still gets the name HL7 would give it.
    #[must_use]
    pub fn structure_id(&self, code: &str, trigger: &str) -> String {
        if code.is_empty() {
            return "HL7Message".to_string();
        }
        let joined = if trigger.is_empty() {
            code.to_string()
        } else {
            format!("{code}_{trigger}")
        };
        if let Some(target) = self.aliases.get(&joined) {
            return target.clone();
        }
        if self.structures.contains_key(&joined) {
            return joined;
        }
        if self.structures.contains_key(code) {
            return code.to_string();
        }
        joined
    }

    /// Every structure ID this dictionary defines, in name order.
    pub fn structure_ids(&self) -> impl Iterator<Item = &str> {
        self.structures.keys().map(String::as_str)
    }

    /// Every segment name this dictionary defines, in name order.
    pub fn segment_names(&self) -> impl Iterator<Item = &str> {
        self.segments.keys().map(String::as_str)
    }

    /// Every composite data type this dictionary defines, in name order.
    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        self.types.keys().map(String::as_str)
    }

    /// Load a dictionary from JSON, resolving an `"inherits"` member
    /// against this crate's bundled releases.
    ///
    /// This is schema mode's entry point: write the shape of the vendor's
    /// messages as JSON, load it at runtime, and no recompile is needed
    /// when the business adds a field.
    ///
    /// ```
    /// let dictionary = hl7_2::Dictionary::from_json(r#"{
    ///   "inherits": "2.5",
    ///   "segments": { "ZPD": ["ST", "XPN", "TS"] }
    /// }"#, "acme").unwrap();
    /// assert_eq!(dictionary.field_type("ZPD", 2), Some("XPN"));
    /// assert_eq!(dictionary.field_type("PID", 5), Some("XPN")); // inherited
    /// ```
    /// # Errors
    ///
    /// [`Error::Json`] when the text is not valid JSON, [`Error::Field`] or
    /// [`Error::Missing`] when a member is the wrong shape or absent, and
    /// [`Error::UnknownBase`] when `inherits` names a release this crate has
    /// no dictionary for.
    pub fn from_json(text: &str, name: impl Into<String>) -> Result<Dictionary, Error> {
        Dictionary::from_json_resolving(text, name, |version| {
            crate::Version::parse(version).map(crate::Version::dictionary)
        })
    }

    /// Load a dictionary from JSON, layering it over `base` rather than
    /// over a bundled release. An `"inherits"` member is ignored.
    /// # Errors
    ///
    /// [`Error::Json`] when the text is not valid JSON, [`Error::Field`] or
    /// [`Error::Missing`] when a member is the wrong shape or absent, and
    /// [`Error::UnknownBase`] when `inherits` names a release this crate has
    /// no dictionary for.
    pub fn from_json_over(
        text: &str,
        name: impl Into<String>,
        base: &Dictionary,
    ) -> Result<Dictionary, Error> {
        let name = name.into();
        let value = json::parse(text).map_err(Error::Json)?;
        let mut dictionary = base.clone();
        dictionary.name = name;
        dictionary.version = None;
        dictionary.apply(&value)?;
        Ok(dictionary)
    }

    /// Load a dictionary from JSON, resolving `"inherits"` through
    /// `resolve`. Used internally to load the bundled releases (where
    /// resolution must not recurse back through the public entry point) and
    /// available to callers who keep their own set of base dictionaries.
    /// # Errors
    ///
    /// [`Error::Json`] when the text is not valid JSON, [`Error::Field`] or
    /// [`Error::Missing`] when a member is the wrong shape or absent, and
    /// [`Error::UnknownBase`] when `inherits` names a release this crate has
    /// no dictionary for.
    pub fn from_json_resolving(
        text: &str,
        name: impl Into<String>,
        resolve: impl Fn(&str) -> Option<std::sync::Arc<Dictionary>>,
    ) -> Result<Dictionary, Error> {
        let name = name.into();
        let value = json::parse(text).map_err(Error::Json)?;
        let mut dictionary = match value.get("inherits") {
            None => Dictionary::empty(name.clone()),
            Some(Value::String(base)) => match resolve(base) {
                Some(base) => Dictionary {
                    name: name.clone(),
                    ..(*base).clone()
                },
                None => return Err(Error::UnknownBase(base.clone())),
            },
            Some(other) => {
                return Err(Error::field("inherits", "a version string", other));
            }
        };
        dictionary.apply(&value)?;
        Ok(dictionary)
    }

    /// Layer one parsed dictionary document over `self`: listed entries
    /// replace what was there, `null` entries remove it, and everything
    /// unmentioned is inherited untouched. That is what makes a per-release
    /// delta file small enough to read.
    fn apply(&mut self, value: &Value) -> Result<(), Error> {
        if value.as_object().is_none() {
            return Err(Error::field("<document>", "an object", value));
        }
        if let Some(version) = value.get("version") {
            match version.as_str() {
                Some(text) => self.version = Some(text.to_string()),
                None => return Err(Error::field("version", "a version string", version)),
            }
        }
        for section in ["types", "segments"] {
            let Some(members) = value.get(section) else {
                continue;
            };
            let members = members
                .as_object()
                .ok_or_else(|| Error::field(section, "an object", members))?;
            for (key, entry) in members {
                let is_segments = section == "segments";
                let table = if is_segments {
                    &mut self.segments
                } else {
                    &mut self.types
                };
                if entry.is_null() {
                    table.remove(key);
                    if is_segments {
                        self.cardinality.remove(key);
                    }
                    continue;
                }
                let inherited = table.get(key).cloned().unwrap_or_default();
                let inherited_cardinality = if is_segments {
                    self.cardinality.get(key).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                };
                let (names, cardinality) = positions(
                    entry,
                    inherited,
                    inherited_cardinality,
                    &format!("{section}.{key}"),
                )?;
                table.insert(key.clone(), names);
                // Composite components do not repeat and are not
                // individually required, so cardinality is kept for
                // segments only.
                if is_segments {
                    self.cardinality.insert(key.clone(), cardinality);
                }
            }
        }
        if let Some(aliases) = value.get("aliases") {
            let members = aliases
                .as_object()
                .ok_or_else(|| Error::field("aliases", "an object", aliases))?;
            for (key, entry) in members {
                if entry.is_null() {
                    self.aliases.remove(key);
                    continue;
                }
                let target = entry.as_str().ok_or_else(|| {
                    Error::field(&format!("aliases.{key}"), "a structure ID", entry)
                })?;
                self.aliases.insert(key.clone(), target.to_string());
            }
        }
        if let Some(structures) = value.get("structures") {
            let members = structures
                .as_object()
                .ok_or_else(|| Error::field("structures", "an object", structures))?;
            for (key, entry) in members {
                if entry.is_null() {
                    self.structures.remove(key);
                    continue;
                }
                let items = parse_items(entry, &format!("structures.{key}"))?;
                self.structures.insert(key.clone(), items);
            }
        }
        Ok(())
    }
}

/// Read a list of data types in either of the two forms a dictionary may
/// write it.
///
/// An array states the whole list and replaces what was inherited. An
/// object states individual 1-based positions and leaves the rest of the
/// inherited list alone — `{"12": "ID"}` is how v2.1 says "MSH-12 is a
/// plain ID here" without restating the other twenty fields, and without
/// claiming anything about which fields that release did or did not have.
///
/// Either form may write a position as an object rather than a bare name
/// when the schema says more than the type — see [`entry_of`].
fn positions(
    value: &Value,
    inherited: Vec<String>,
    inherited_cardinality: Vec<Cardinality>,
    path: &str,
) -> Result<(Vec<String>, Vec<Cardinality>), Error> {
    if let Some(list) = value.as_array() {
        let mut names = Vec::with_capacity(list.len());
        let mut cardinality = Vec::with_capacity(list.len());
        for (index, item) in list.iter().enumerate() {
            let (name, card) = entry_of(item, &format!("{path}[{index}]"))?;
            names.push(name);
            cardinality.push(card);
        }
        return Ok((names, cardinality));
    }
    let members = value
        .as_object()
        .ok_or_else(|| Error::field(path, "an array, or an object of position overrides", value))?;
    let mut names = inherited;
    let mut cardinality = inherited_cardinality;
    for (key, entry) in members {
        let path = format!("{path}.{key}");
        let position: usize = key
            .parse()
            .ok()
            .filter(|position| *position > 0)
            .ok_or_else(|| Error::Field {
                path: path.clone(),
                expected: "a 1-based position number".to_string(),
                found: format!("{key:?}"),
            })?;
        let (name, card) = entry_of(entry, &path)?;
        if names.len() < position {
            names.resize(position, UNSTATED.to_string());
        }
        if cardinality.len() < position {
            cardinality.resize(position, Cardinality::default());
        }
        names[position - 1] = name;
        cardinality[position - 1] = card;
    }
    // A sparse delta may state a position past the end of what it inherited,
    // and the two tables are indexed together, so they stay the same length.
    cardinality.resize(names.len(), Cardinality::default());
    Ok((names, cardinality))
}

/// Read one position: a bare data type name, or an object that also carries
/// what the schema said about how often the field may appear.
///
/// `"XPN"` and `{"type": "XPN"}` mean the same thing. The object form exists
/// for dictionaries generated from XML Schema, which know `minOccurs` and
/// `maxOccurs` as well: `{"type": "XTN", "repeats": true}`.
fn entry_of(value: &Value, path: &str) -> Result<(String, Cardinality), Error> {
    if let Some(name) = value.as_str() {
        return Ok((name.to_string(), Cardinality::default()));
    }
    // Anything that is neither a name nor an object cannot be either form,
    // so it is reported against the commoner one.
    if value.as_object().is_none() {
        return Err(Error::field(path, "a data type name", value));
    }
    let name = value
        .get("type")
        .ok_or_else(|| Error::missing(&format!("{path}.type")))?
        .as_str()
        .ok_or_else(|| {
            Error::field(
                &format!("{path}.type"),
                "a data type name",
                value.get("type").unwrap_or(value),
            )
        })?;
    Ok((
        name.to_string(),
        Cardinality {
            required: flag(value, "required", path)?,
            repeats: flag(value, "repeats", path)?,
        },
    ))
}

/// Read a structure's item list: an array of segment names, segment
/// objects, and group objects.
fn parse_items(value: &Value, path: &str) -> Result<Vec<Item>, Error> {
    let list = value
        .as_array()
        .ok_or_else(|| Error::field(path, "an array of structure items", value))?;
    let mut items = Vec::with_capacity(list.len());
    for (index, entry) in list.iter().enumerate() {
        let path = format!("{path}[{index}]");
        // Shorthand: a bare string is an optional, non-repeating segment,
        // which is what most entries in a hand-written structure are.
        if let Some(name) = entry.as_str() {
            items.push(Item::Segment {
                name: name.to_string(),
                required: false,
                repeats: false,
            });
            continue;
        }
        let required = flag(entry, "required", &path)?;
        let repeats = flag(entry, "repeats", &path)?;
        if let Some(name) = entry.get("segment") {
            let name = name
                .as_str()
                .ok_or_else(|| Error::field(&format!("{path}.segment"), "a segment name", name))?;
            items.push(Item::Segment {
                name: name.to_string(),
                required,
                repeats,
            });
        } else if let Some(name) = entry.get("group") {
            let name = name
                .as_str()
                .ok_or_else(|| Error::field(&format!("{path}.group"), "a group name", name))?;
            let children = entry
                .get("items")
                .ok_or_else(|| Error::missing(&format!("{path}.items")))?;
            items.push(Item::Group {
                name: name.to_string(),
                required,
                repeats,
                items: parse_items(children, &format!("{path}.items"))?,
            });
        } else {
            return Err(Error::field(
                &path,
                "an item with a `segment` or `group` member",
                entry,
            ));
        }
    }
    Ok(items)
}

/// Read an optional boolean member, defaulting to false.
fn flag(entry: &Value, name: &str, path: &str) -> Result<bool, Error> {
    match entry.get(name) {
        None => Ok(false),
        Some(value) => value
            .as_bool()
            .ok_or_else(|| Error::field(&format!("{path}.{name}"), "true or false", value)),
    }
}

/// Why a dictionary could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The text is not valid JSON.
    Json(json::Error),
    /// A member is present but the wrong shape.
    Field {
        /// Where in the document, e.g. `segments.PID[3]`.
        path: String,
        /// What was expected there.
        expected: String,
        /// What was found instead.
        found: String,
    },
    /// A required member is absent.
    Missing {
        /// Where in the document the member was expected.
        path: String,
    },
    /// `"inherits"` names a release this crate has no dictionary for.
    UnknownBase(String),
}

impl Error {
    fn field(path: &str, expected: &str, found: &Value) -> Error {
        Error::Field {
            path: path.to_string(),
            expected: expected.to_string(),
            found: found.kind().to_string(),
        }
    }

    fn missing(path: &str) -> Error {
        Error::Missing {
            path: path.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Json(error) => write!(f, "{error}"),
            Error::Field {
                path,
                expected,
                found,
            } => write!(f, "{path}: expected {expected}, found {found}"),
            Error::Missing { path } => write!(f, "{path}: required member is missing"),
            Error::UnknownBase(base) => {
                write!(f, "`inherits`: {base:?} is not a known HL7 version")
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Version;

    #[test]
    fn reads_the_base_release() {
        let dictionary = Version::V2_5.dictionary();
        assert_eq!(dictionary.field_type("PID", 5), Some("XPN"));
        assert_eq!(dictionary.field_type("MSH", 9), Some("MSG"));
        assert_eq!(dictionary.field_type("OBX", 5), Some(VARIABLE));
        assert_eq!(dictionary.field_type("PID", 999), None);
        assert_eq!(dictionary.field_type("ZZZ", 1), None);
        assert_eq!(
            dictionary
                .composite_components("XPN")
                .map(|c| c[0].as_str()),
            Some("FN")
        );
        assert!(!dictionary.is_composite("ST"));
        assert!(dictionary.structure("ORU_R01").is_some());
    }

    #[test]
    fn a_delta_adds_removes_and_inherits() {
        let dictionary = Dictionary::from_json(
            r#"{
                "inherits": "2.5",
                "types": { "TS": ["ST"], "XPN": null },
                "segments": { "ZPD": ["ST", "CX"] },
                "structures": { "ORU_R01": null }
            }"#,
            "test",
        )
        .unwrap();
        assert_eq!(dictionary.composite_components("TS").unwrap(), ["ST"]); // replaced
        assert_eq!(dictionary.composite_components("XPN"), None); // removed
        assert_eq!(dictionary.field_type("ZPD", 2), Some("CX")); // added
        assert_eq!(dictionary.field_type("PID", 5), Some("XPN")); // inherited
        assert_eq!(dictionary.structure("ORU_R01"), None); // removed
        assert!(dictionary.structure("ACK").is_some()); // inherited
        assert_eq!(dictionary.name(), "test");
    }

    #[test]
    fn a_sparse_delta_restates_one_position_and_keeps_the_rest() {
        let dictionary = Dictionary::from_json(
            r#"{"inherits": "2.5", "segments": {"MSH": {"12": "ID"}}}"#,
            "test",
        )
        .unwrap();
        assert_eq!(dictionary.field_type("MSH", 12), Some("ID")); // overridden
        assert_eq!(dictionary.field_type("MSH", 9), Some("MSG")); // untouched
        assert_eq!(dictionary.field_type("MSH", 21), Some("EI")); // untouched
        // A position past the inherited end is stated; the gap before it is
        // unknown rather than silently typed.
        let dictionary =
            Dictionary::from_json(r#"{"segments": {"ZZZ": {"3": "CX"}}}"#, "test").unwrap();
        assert_eq!(dictionary.field_type("ZZZ", 3), Some("CX"));
        assert_eq!(dictionary.field_type("ZZZ", 1), None);
        let error =
            Dictionary::from_json(r#"{"segments": {"ZZZ": {"0": "CX"}}}"#, "test").unwrap_err();
        assert!(error.to_string().contains("1-based position"), "{error}");
    }

    #[test]
    fn reads_structures_including_the_string_shorthand() {
        let dictionary = Dictionary::from_json(
            r#"{"structures": {"ZZZ_Z01": [
                {"segment": "MSH", "required": true},
                "NTE",
                {"group": "ORDER", "repeats": true, "items": [{"segment": "ORC", "required": true}]}
            ]}}"#,
            "test",
        )
        .unwrap();
        let items = dictionary.structure("ZZZ_Z01").unwrap();
        assert!(matches!(&items[0], Item::Segment { name, required: true, .. } if name == "MSH"));
        assert!(matches!(
            &items[1],
            Item::Segment {
                required: false,
                repeats: false,
                ..
            }
        ));
        assert!(items[2].repeats() && !items[2].required());
        assert!(items[2].can_start("ORC"));
        assert!(!items[2].can_start("OBX"));
    }

    #[test]
    fn a_group_can_start_at_any_leading_optional_segment() {
        // ORU_R01's PATIENT_RESULT starts at PID (optional PATIENT group)
        // or at ORC/OBR (the required ORDER_OBSERVATION group).
        let dictionary = Version::V2_5.dictionary();
        let items = dictionary.structure("ORU_R01").unwrap();
        let patient_result = &items[2];
        assert_eq!(patient_result.name(), "PATIENT_RESULT");
        assert!(patient_result.can_start("PID"));
        assert!(patient_result.can_start("OBR"));
        assert!(!patient_result.can_start("MSA"));
    }

    #[test]
    fn resolves_obx_5_through_obx_2() {
        let dictionary = Version::V2_5.dictionary();
        let message = er7::parse("MSH|^~\\&|A||||1||ORU^R01|1|P|2.5\rOBX|1|CE|X||a^b").unwrap();
        let obx = message.segment("OBX").unwrap();
        assert_eq!(dictionary.variable_type(obx), Some("CE"));
        let message = er7::parse("MSH|^~\\&|A||||1||ORU^R01|1|P|2.5\rOBX|1|NM|X||7").unwrap();
        assert_eq!(
            dictionary.variable_type(message.segment("OBX").unwrap()),
            None
        );
    }

    #[test]
    fn a_field_may_state_its_cardinality_as_well_as_its_type() {
        let dictionary = Dictionary::from_json(
            r#"{"segments": {"PID": [
                 "SI",
                 {"type": "CX", "required": true},
                 {"type": "XTN", "repeats": true},
                 {"type": "ST", "required": true, "repeats": true}
               ]}}"#,
            "x",
        )
        .unwrap();
        // The bare name and the object form name the same data type.
        assert_eq!(dictionary.field_type("PID", 1), Some("SI"));
        assert_eq!(dictionary.field_type("PID", 2), Some("CX"));
        assert_eq!(
            dictionary.field_cardinality("PID", 1),
            Cardinality::default()
        );
        assert_eq!(
            dictionary.field_cardinality("PID", 2),
            Cardinality {
                required: true,
                repeats: false
            }
        );
        assert_eq!(
            dictionary.field_cardinality("PID", 3),
            Cardinality {
                required: false,
                repeats: true
            }
        );
        assert_eq!(
            dictionary.field_cardinality("PID", 4),
            Cardinality {
                required: true,
                repeats: true
            }
        );
        // Off the end, and a segment that was never mentioned, both default.
        assert_eq!(
            dictionary.field_cardinality("PID", 99),
            Cardinality::default()
        );
        assert_eq!(
            dictionary.field_cardinality("ZZZ", 1),
            Cardinality::default()
        );
        assert_eq!(
            dictionary.field_cardinality("PID", 0),
            Cardinality::default()
        );
    }

    #[test]
    fn cardinality_layers_and_is_removed_like_everything_else() {
        // A sparse override states one position and leaves the rest alone.
        let dictionary = Dictionary::from_json(
            r#"{"inherits": "2.5", "segments": {"PID": {"13": {"type": "XTN", "repeats": true}}}}"#,
            "x",
        )
        .unwrap();
        assert!(dictionary.field_cardinality("PID", 13).repeats);
        assert!(!dictionary.field_cardinality("PID", 5).repeats);
        assert_eq!(dictionary.field_type("PID", 5), Some("XPN")); // still inherited

        // Removing the segment removes what was said about its fields too.
        let dictionary =
            Dictionary::from_json(r#"{"inherits": "2.5", "segments": {"PID": null}}"#, "x")
                .unwrap();
        assert_eq!(
            dictionary.field_cardinality("PID", 13),
            Cardinality::default()
        );
    }

    #[test]
    fn reports_where_a_malformed_dictionary_is_wrong() {
        let error = Dictionary::from_json(r#"{"segments": {"PID": [1]}}"#, "x").unwrap_err();
        assert_eq!(
            error.to_string(),
            "segments.PID[0]: expected a data type name, found number"
        );
        let error = Dictionary::from_json(r#"{"inherits": "9.9"}"#, "x").unwrap_err();
        assert!(matches!(error, Error::UnknownBase(_)), "{error}");
        let error =
            Dictionary::from_json(r#"{"structures": {"A": [{"group": "G"}]}}"#, "x").unwrap_err();
        assert_eq!(
            error.to_string(),
            "structures.A[0].items: required member is missing"
        );
        assert!(matches!(
            Dictionary::from_json("not json", "x"),
            Err(Error::Json(_))
        ));
    }

    #[test]
    fn layering_over_an_explicit_base_ignores_inherits() {
        let base = Dictionary::from_json(r#"{"segments": {"AAA": ["ST"]}}"#, "base").unwrap();
        let over = Dictionary::from_json_over(
            r#"{"inherits": "2.5", "segments": {"BBB": ["NM"]}}"#,
            "over",
            &base,
        )
        .unwrap();
        assert_eq!(over.field_type("AAA", 1), Some("ST"));
        assert_eq!(over.field_type("BBB", 1), Some("NM"));
        assert_eq!(
            over.field_type("PID", 5),
            None,
            "2.5 must not have been pulled in"
        );
    }
}
