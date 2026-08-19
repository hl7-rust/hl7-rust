//! The parsed message: what release it speaks, what structure it is, and
//! how to read, change, and write it back.
//!
//! A [`Message`] is an `er7::Message` plus the two things `er7` cannot
//! know — which HL7 release the sender used, and which dictionary that
//! selects — and every mode reads it through those. It is also the
//! multi-modal escape hatch the design calls for: the raw message is never
//! consumed or discarded, so a caller who has decoded into a struct can
//! still reach [`Message::raw`] for the one vendor field the struct does
//! not model, without re-parsing.

use crate::dictionary::Dictionary;
use crate::generic::Node;
use crate::structure::{self, Layout};
use crate::validate::{Diagnostic, Severity};
use crate::{Error, Options, Version, generic};
use er7::{Path, Segment, Separators};
use std::sync::Arc;

/// One parsed HL7 v2 message.
///
/// Built by [`crate::parse`] or [`crate::parse_with_options`]; see the
/// crate documentation for the three modes that read it.
#[derive(Debug, Clone)]
pub struct Message {
    raw: er7::Message,
    version: Version,
    dictionary: Arc<Dictionary>,
}

impl Message {
    /// Parse `text` under `options`. See [`crate::parse_with_options`],
    /// which is the same call under the name callers reach for.
    pub(crate) fn parse(text: &str, options: &Options) -> Result<Message, Error> {
        let raw = er7::parse(&crate::normalize(text))?;
        let version = options
            .version
            .or_else(|| Version::from_message(&raw))
            .unwrap_or_default();
        let dictionary = match &options.dictionary {
            Some(dictionary) => Arc::clone(dictionary),
            None => version.dictionary(),
        };
        let message = Message {
            raw,
            version,
            dictionary,
        };
        if options.strict {
            let failures: Vec<Diagnostic> = message
                .validate()
                .into_iter()
                .filter(|diagnostic| diagnostic.severity == Severity::Error)
                .collect();
            if !failures.is_empty() {
                return Err(Error::Invalid(failures));
            }
        }
        Ok(message)
    }

    /// The HL7 release this message is read as: MSH-12 resolved through
    /// [`Version::nearest`], or whatever [`Options::version`] forced.
    #[must_use]
    pub fn version(&self) -> Version {
        self.version
    }

    /// The dictionary this message is read through.
    #[must_use]
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    /// The message structure ID: MSH-9.3 when the sender supplied one,
    /// otherwise derived from MSH-9.1 and MSH-9.2 through the dictionary —
    /// `ORU_R01`, `ADT_A01`, `ACK`.
    ///
    /// Read from the message each time rather than cached at parse, so a
    /// message whose header was changed — by [`Message::set`] or by
    /// [`crate::Builder`] — reports what it now says it is.
    #[must_use]
    pub fn structure_id(&self) -> String {
        match self.raw.message_structure().filter(|id| !id.is_empty()) {
            Some(id) => id,
            None => self.dictionary.structure_id(
                &self.raw.message_code().unwrap_or_default(),
                &self.raw.trigger_event().unwrap_or_default(),
            ),
        }
    }

    /// The delimiters this message declared in MSH-1 and MSH-2.
    #[must_use]
    pub fn separators(&self) -> &Separators {
        &self.raw.separators
    }

    /// The underlying `er7` message — the escape hatch.
    ///
    /// Everything this crate knows is derived from here, and nothing is
    /// lost on the way in, so a caller who needs a field no mode models
    /// reads it here rather than parsing the text a second time.
    #[must_use]
    pub fn raw(&self) -> &er7::Message {
        &self.raw
    }

    /// The underlying `er7` message, mutably. Changes are visible to every
    /// other method immediately, including [`Message::to_er7`].
    pub fn raw_mut(&mut self) -> &mut er7::Message {
        &mut self.raw
    }

    /// Take the underlying `er7` message, dropping the dictionary.
    #[must_use]
    pub fn into_raw(self) -> er7::Message {
        self.raw
    }

    /// Every segment, in message order.
    pub fn segments(&self) -> impl Iterator<Item = &Segment> {
        self.raw.segments.iter()
    }

    /// The first segment named `name`.
    #[must_use]
    pub fn segment(&self, name: &str) -> Option<&Segment> {
        self.raw.segment(name)
    }

    /// The `occurrence`-th (1-based) segment named `name`.
    #[must_use]
    pub fn segment_at(&self, name: &str, occurrence: usize) -> Option<&Segment> {
        self.raw.segment_at(name, occurrence)
    }

    /// Write the message back as ER7.
    ///
    /// For a message parsed and not modified this reproduces the input,
    /// differing only where the input was not canonical (other segment
    /// terminators, blank lines, leading whitespace) — that guarantee is
    /// `er7`'s, and this crate does not weaken it.
    #[must_use]
    pub fn to_er7(&self) -> String {
        self.raw.to_er7()
    }

    /// Write the message back as ER7, choosing the segment terminator; see
    /// [`er7::RenderOptions`].
    #[must_use]
    pub fn to_er7_with(&self, options: er7::RenderOptions) -> String {
        self.raw.to_er7_with(options)
    }

    // ---- generic mode ---------------------------------------------------

    /// The whole message as a navigable tree, with segments grouped into
    /// the message structure when they fit it and left flat when they do
    /// not. See [`crate::generic`] for the naming rules.
    #[must_use]
    pub fn tree(&self) -> Node {
        self.tree_with_options(true)
    }

    /// The message as a tree, with message-structure grouping optionally
    /// suppressed. A flat tree is one node per segment under the root, and
    /// is what a caller who navigates by segment name wants.
    #[must_use]
    pub fn tree_with_options(&self, grouped: bool) -> Node {
        let separators = &self.raw.separators;
        let mut occurrences: Vec<usize> = Vec::with_capacity(self.raw.segments.len());
        let mut counts: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::default();
        for segment in &self.raw.segments {
            let count = counts.entry(segment.name.as_str()).or_default();
            *count += 1;
            occurrences.push(*count);
        }
        let nodes: Vec<Node> = self
            .raw
            .segments
            .iter()
            .zip(&occurrences)
            .map(|(segment, occurrence)| {
                generic::segment(segment, *occurrence, &self.dictionary, separators)
            })
            .collect();
        let structure_id = self.structure_id();
        let children = match grouped.then(|| self.layout()).flatten() {
            Some(layout) => build(&layout, &nodes, &structure_id),
            None => nodes,
        };
        generic::root(&structure_id, children)
    }

    /// How this message's segments fit its structure, or `None` when the
    /// dictionary has no grammar for it or the segments do not fit.
    #[must_use]
    pub fn layout(&self) -> Option<Vec<Layout>> {
        let items = self.dictionary.structure(&self.structure_id())?;
        let names: Vec<&str> = self
            .raw
            .segments
            .iter()
            .map(|segment| segment.name.as_str())
            .collect();
        structure::group(items, &names)
    }

    // ---- reading --------------------------------------------------------

    /// The value at `path`, e.g. `PID-5.1`, `OBX[2]-5`, `MSH-9.3`.
    ///
    /// Returns the decoded text of the first match, or `None` when the path
    /// names nothing in this message. Path syntax is `er7`'s; see
    /// [`er7::Path`].
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path.
    pub fn get(&self, path: &str) -> Result<Option<String>, Error> {
        Ok(self.raw.query(path)?)
    }

    /// Every value matching `path`. A path that omits an occurrence or a
    /// repetition matches all of them, so `OBX-5` reads every result in the
    /// message in one call.
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path.
    pub fn get_all(&self, path: &str) -> Result<Vec<String>, Error> {
        Ok(self.raw.query_all(path)?)
    }

    /// Every repetition of the field at `path`, in message order.
    ///
    /// This differs from [`Message::get_all`] at exactly one point: a path
    /// that names a whole field, such as `PID-3`, is one value to `er7` —
    /// the field's text, repetition separators and all — because that is
    /// what the field *is*. Here it is the repetitions, because a caller
    /// asking for a list of them is asking about `241900~99~7` as three
    /// identifiers rather than one string. Paths that already name a
    /// repetition or a component behave as [`Message::get_all`].
    ///
    /// ```
    /// let message = hl7_v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1||241900~99~7")?;
    /// assert_eq!(message.repetitions("PID-3")?, ["241900", "99", "7"]);
    /// assert_eq!(message.get("PID-3")?.as_deref(), Some("241900~99~7"));
    /// # Ok::<(), hl7_v2::Error>(())
    /// ```
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path.
    pub fn repetitions(&self, path: &str) -> Result<Vec<String>, Error> {
        let parsed = Path::parse(path)?;
        let Some(number) = parsed.field else {
            return self.get_all(path);
        };
        if parsed.repetition.is_some() || parsed.component.is_some() {
            return self.get_all(path);
        }
        let mut values = Vec::new();
        let mut occurrence = 0;
        for segment in &self.raw.segments {
            if segment.name != parsed.segment {
                continue;
            }
            occurrence += 1;
            if parsed
                .segment_occurrence
                .is_some_and(|wanted| wanted != occurrence)
            {
                continue;
            }
            if let Some(field) = segment.field(number) {
                for repetition in &field.repetitions {
                    values.push(repetition.to_text(&self.raw.separators));
                }
            }
        }
        Ok(values)
    }

    /// The data type the dictionary gives the field at `path`, resolving
    /// OBX-5 through OBX-2. `None` when the segment or field is unknown.
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path.
    pub fn type_of(&self, path: &str) -> Result<Option<String>, Error> {
        let path = Path::parse(path)?;
        let Some(field) = path.field else {
            return Ok(None);
        };
        let occurrence = path.segment_occurrence.unwrap_or(1);
        let Some(segment) = self.raw.segment_at(&path.segment, occurrence) else {
            return Ok(None);
        };
        Ok(match self.dictionary.field_type(&path.segment, field) {
            Some(crate::dictionary::VARIABLE) => self.dictionary.variable_type(segment),
            other => other,
        }
        .map(str::to_string))
    }

    // ---- writing --------------------------------------------------------

    /// Set the value at `path`, creating whatever the path names and the
    /// message does not yet have.
    ///
    /// `value` is data, not wire format: delimiters inside it are escaped,
    /// so setting `SMITH^JOHN` writes one component containing a literal
    /// caret. Use [`Message::set_er7`] to write text that is already
    /// encoded, and note that setting a level replaces everything beneath
    /// it — `set("PID-5", ...)` discards the components PID-5 had.
    ///
    /// The segment must already exist; [`Message::append_segment`] and
    /// [`crate::Builder`] create segments.
    ///
    /// ```
    /// let mut message = hl7_v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1")?;
    /// message.set("PID-5.1", "SMITH")?;
    /// message.set("PID-5.2", "JOHN")?;
    /// assert_eq!(message.get("PID-5")?.as_deref(), Some("SMITH^JOHN"));
    /// # Ok::<(), hl7_v2::Error>(())
    /// ```
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path, or names a
    /// position that cannot be written.
    pub fn set(&mut self, path: &str, value: &str) -> Result<(), Error> {
        let separators = self.raw.separators;
        let encoded = er7::escape::escape(value, &separators).into_owned();
        self.write(path, &encoded, Create::Yes)
    }

    /// Set the value at `path` to text that is already ER7-encoded, so its
    /// delimiters keep their structural meaning: `set_er7("PID-5",
    /// "SMITH^JOHN")` writes two components.
    ///
    /// The text is parsed only down to the levels the path leaves open —
    /// writing to `PID-5.1.2` treats the text as a single subcomponent
    /// value, because there is no level left for a delimiter to divide.
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path, or names a
    /// position that cannot be written.
    pub fn set_er7(&mut self, path: &str, er7_text: &str) -> Result<(), Error> {
        self.write(path, er7_text, Create::Yes)
    }

    /// Set the value at `path` to the HL7 explicit null `""`, which tells
    /// the receiver to clear the value rather than leave it alone. This is
    /// not the same as [`Message::clear`].
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path, or names a
    /// position that cannot be written.
    pub fn set_null(&mut self, path: &str) -> Result<(), Error> {
        self.write(path, er7::message::NULL, Create::Yes)
    }

    /// Empty the value at `path`, as if the sender had never populated it.
    /// Compare [`Message::set_null`], which says "clear this" out loud.
    ///
    /// Clearing what is already absent does nothing and succeeds — it does
    /// not create the empty field it would then be emptying, and it does
    /// not fail on a missing segment. That is what makes writing an
    /// `Option::None` in struct mode a no-op rather than a message full of
    /// empty components.
    /// # Errors
    ///
    /// [`Error::Path`] when `path` is not a valid HL7 path, or names a
    /// position that cannot be written.
    pub fn clear(&mut self, path: &str) -> Result<(), Error> {
        self.write(path, "", Create::No)
    }

    /// Write already-encoded text at `path`, growing the message to fit
    /// when `create` says to and stopping quietly when it does not.
    fn write(&mut self, path: &str, encoded: &str, create: Create) -> Result<(), Error> {
        let separators = self.raw.separators;
        let path = Path::parse(path)?;
        let Some(number) = path.field else {
            return Err(Error::UnwritablePath(
                "a path must name a field to be written".to_string(),
            ));
        };
        let occurrence = path.segment_occurrence.unwrap_or(1);
        let segment = match self.raw.segment_at_mut(&path.segment, occurrence) {
            Some(segment) => segment,
            None if create == Create::No => return Ok(()),
            None => {
                return Err(Error::NoSuchSegment {
                    name: path.segment.clone(),
                    occurrence,
                });
            }
        };
        if segment.fields.len() < number {
            if create == Create::No {
                return Ok(());
            }
            segment.fields.resize_with(number, Default::default);
        }
        let field = &mut segment.fields[number - 1];
        let repetition = path.repetition.unwrap_or(1);
        if field.repetitions.len() < repetition {
            if create == Create::No {
                return Ok(());
            }
            field.repetitions.resize_with(repetition, Default::default);
        }
        let repetition = &mut field.repetitions[repetition - 1];
        // Each level below the one the path names is filled by splitting
        // the text on that level's delimiter, so `set_er7("PID-5",
        // "SMITH^JOHN")` writes two components while `set` — which escaped
        // the caret first — writes one.
        match (path.component, path.subcomponent) {
            (None, _) => {
                repetition.components = encoded
                    .split(separators.component)
                    .map(|text| component_from(text, &separators))
                    .collect();
            }
            (Some(component), None) => {
                if !grow(&mut repetition.components, component, create) {
                    return Ok(());
                }
                repetition.components[component - 1] = component_from(encoded, &separators);
            }
            (Some(component), Some(subcomponent)) => {
                if !grow(&mut repetition.components, component, create) {
                    return Ok(());
                }
                let component = &mut repetition.components[component - 1];
                if !grow(&mut component.subcomponents, subcomponent, create) {
                    return Ok(());
                }
                component.subcomponents[subcomponent - 1] = er7::Subcomponent::new(encoded);
            }
        }
        Ok(())
    }

    /// Append an empty segment named `name` and return it for populating.
    ///
    /// ```
    /// let mut message = hl7_v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5")?;
    /// message.append_segment("PID");
    /// message.set("PID-3.1", "241900")?;
    /// assert!(message.to_er7().ends_with("\rPID|||241900"));
    /// # Ok::<(), hl7_v2::Error>(())
    /// ```
    pub fn append_segment(&mut self, name: &str) -> &mut Segment {
        self.insert_segment(self.raw.segments.len(), name)
    }

    /// Insert an empty segment named `name` at `index` in the segment list,
    /// clamped to the end. Inserting is how a segment lands in the place
    /// its structure expects rather than after everything else.
    pub fn insert_segment(&mut self, index: usize, name: &str) -> &mut Segment {
        let index = index.min(self.raw.segments.len());
        self.raw.segments.insert(
            index,
            Segment {
                name: name.to_string(),
                fields: Vec::new(),
            },
        );
        &mut self.raw.segments[index]
    }

    /// Remove the `occurrence`-th (1-based) segment named `name`, returning
    /// it. The MSH header cannot be removed.
    pub fn remove_segment(&mut self, name: &str, occurrence: usize) -> Option<Segment> {
        let mut seen = 0;
        let index = self.raw.segments.iter().position(|segment| {
            if segment.name == name {
                seen += 1;
            }
            seen == occurrence && segment.name == name
        })?;
        if index == 0 {
            return None;
        }
        Some(self.raw.segments.remove(index))
    }

    /// Remove every segment named `name`, returning how many went. The MSH
    /// header is never removed.
    pub fn remove_segments(&mut self, name: &str) -> usize {
        let before = self.raw.segments.len();
        let mut index = 0;
        self.raw.segments.retain(|segment| {
            index += 1;
            index == 1 || segment.name != name
        });
        before - self.raw.segments.len()
    }

    // ---- validation and struct mode -------------------------------------

    /// Check this message against its dictionary; see [`crate::validate`].
    ///
    /// Never fails and never changes the message: it reports. Parsing with
    /// [`Options::strict`] runs the same check and turns any
    /// [`Severity::Error`] into a parse failure.
    #[must_use]
    pub fn validate(&self) -> Vec<Diagnostic> {
        crate::validate::validate(self)
    }

    /// Decode into a type that implements [`crate::FromHl7`] — struct mode.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] fn main() -> Result<(), hl7_v2::Error> {
    /// use hl7_v2::FromHl7;
    ///
    /// #[derive(FromHl7)]
    /// struct Patient {
    ///     #[hl7("PID-3.1")]
    ///     id: String,
    ///     #[hl7("PID-5.1.1")]
    ///     family_name: String,
    /// }
    ///
    /// let message = hl7_v2::parse("MSH|^~\\&|A||||1||ADT^A01|1|P|2.5\rPID|1||241900||SMITH^JOHN")?;
    /// let patient: Patient = message.decode()?;
    /// assert_eq!(patient.id, "241900");
    /// assert_eq!(patient.family_name, "SMITH");
    /// # Ok(())
    /// # }
    /// # #[cfg(not(feature = "derive"))] fn main() {}
    /// ```
    /// # Errors
    ///
    /// Whatever the type's [`FromHl7`](crate::FromHl7) implementation
    /// reports: a path it could not read, or a value it could not convert.
    pub fn decode<T: crate::FromHl7>(&self) -> Result<T, Error> {
        T::from_hl7(self)
    }
}

impl std::fmt::Display for Message {
    /// The message as ER7; see [`Message::to_er7`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_er7())
    }
}

/// Whether a write may bring into being what it is writing to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Create {
    /// Grow the message so the path exists.
    Yes,
    /// Leave the message alone if the path does not already exist.
    No,
}

/// Grow `list` so that `position` (1-based) exists, reporting whether the
/// caller may go on to write there.
fn grow<T: Default>(list: &mut Vec<T>, position: usize, create: Create) -> bool {
    if list.len() < position {
        if create == Create::No {
            return false;
        }
        list.resize_with(position, Default::default);
    }
    true
}

/// Split already-encoded text into one component's subcomponents.
fn component_from(text: &str, separators: &Separators) -> er7::Component {
    er7::Component {
        subcomponents: text
            .split(separators.subcomponent)
            .map(er7::Subcomponent::new)
            .collect(),
    }
}

/// Turn a matched layout into tree nodes, cloning each segment's node into
/// the group it landed in.
fn build(layout: &[Layout], nodes: &[Node], root_name: &str) -> Vec<Node> {
    layout
        .iter()
        .map(|item| match item {
            Layout::Segment(index) => nodes[*index].clone(),
            Layout::Group { name, items } => {
                generic::group(root_name, name, build(items, nodes, root_name))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "MSH|^~\\&|hphis||EPIC||20131011093851||ADT^A01|14AAACVDD|P|2.5";

    fn message() -> Message {
        crate::parse(&format!("{HEADER}\rPID|1||241900||TEST^FOUAZ\rNK1|1")).unwrap()
    }

    #[test]
    fn reads_version_and_structure_off_the_header() {
        let message = message();
        assert_eq!(message.version(), Version::V2_5);
        assert_eq!(message.structure_id(), "ADT_A01");
        // A trigger event that shares a structure resolves through the
        // dictionary's aliases.
        let other =
            crate::parse("MSH|^~\\&|A||||1||ADT^A08|1|P|2.5\rEVN|A08\rPID|1\rPV1|1").unwrap();
        assert_eq!(other.structure_id(), "ADT_A01");
        // MSH-9.3 wins when the sender supplies one.
        let explicit = crate::parse("MSH|^~\\&|A||||1||ADT^A08^ADT_A08|1|P|2.5").unwrap();
        assert_eq!(explicit.structure_id(), "ADT_A08");
    }

    #[test]
    fn defaults_the_version_when_msh_12_is_missing_or_odd() {
        let message = crate::parse("MSH|^~\\&|A||||1||ACK|1|P|\rMSA|AA|1").unwrap();
        assert_eq!(message.version(), Version::V2_5);
        let message = crate::parse("MSH|^~\\&|A||||1||ACK|1|P|2.3.1\rMSA|AA|1").unwrap();
        assert_eq!(message.version(), Version::V2_3_1);
        // An unmodelled point release reads as the nearest older one.
        let message = crate::parse("MSH|^~\\&|A||||1||ACK|1|P|2.5.2\rMSA|AA|1").unwrap();
        assert_eq!(message.version(), Version::V2_5_1);
    }

    #[test]
    fn reads_values_by_path() {
        let message = message();
        assert_eq!(message.get("PID-5.1").unwrap().as_deref(), Some("TEST"));
        assert_eq!(message.get("PID-5").unwrap().as_deref(), Some("TEST^FOUAZ"));
        assert_eq!(message.get("PID-99").unwrap(), None);
        assert_eq!(message.get("ZZZ-1").unwrap(), None);
        assert!(matches!(message.get("PID-0"), Err(Error::Path(_))));
        assert_eq!(message.type_of("PID-5").unwrap().as_deref(), Some("XPN"));
        assert_eq!(message.type_of("ZZZ-1").unwrap(), None);
    }

    #[test]
    fn writes_values_creating_what_is_missing() {
        let mut message = message();
        message.set("PID-8", "F").unwrap();
        assert_eq!(message.get("PID-8").unwrap().as_deref(), Some("F"));
        // Beyond the current end of the segment.
        message.set("PID-11.3", "SEATTLE").unwrap();
        assert_eq!(message.get("PID-11.3").unwrap().as_deref(), Some("SEATTLE"));
        // Into a repetition that does not exist yet.
        message.set("PID[1]-3[2].1", "OTHER").unwrap();
        assert_eq!(
            message.get_all("PID-3.1").unwrap(),
            ["241900".to_string(), "OTHER".to_string()]
        );
        // A missing segment is an error, not a silent no-op.
        assert!(matches!(
            message.set("OBX-5", "x"),
            Err(Error::NoSuchSegment { .. })
        ));
        assert!(matches!(
            message.set("PID", "x"),
            Err(Error::UnwritablePath(_))
        ));
    }

    #[test]
    fn set_escapes_and_set_er7_does_not() {
        let mut message = message();
        message.set("PID-5.1", "SMITH^JOHN").unwrap();
        // One component holding a literal caret ...
        assert!(message.to_er7().contains("\\S\\"), "{}", message.to_er7());
        assert_eq!(
            message.get("PID-5.1").unwrap().as_deref(),
            Some("SMITH^JOHN")
        );
        // ... versus two components.
        let mut message = message2();
        message.set_er7("PID-5", "SMITH^JOHN").unwrap();
        assert_eq!(message.get("PID-5.2").unwrap().as_deref(), Some("JOHN"));
    }

    fn message2() -> Message {
        crate::parse(&format!("{HEADER}\rPID|1")).unwrap()
    }

    #[test]
    fn distinguishes_clearing_from_nulling() {
        let mut message = message();
        message.set_null("PID-5").unwrap();
        assert!(message.to_er7().contains("|\"\""));
        message.clear("PID-5").unwrap();
        assert!(!message.to_er7().contains("\"\""));
        // Clearing what was never there leaves no trace of having tried:
        // no empty components, no error about the missing segment.
        let before = message.to_er7();
        message.clear("PID-5.3").unwrap();
        message.clear("PID-40.2").unwrap();
        message.clear("OBX-5").unwrap();
        assert_eq!(message.to_er7(), before);
    }

    #[test]
    fn adds_and_removes_segments() {
        let mut message = message();
        message.append_segment("OBX");
        message.set("OBX-3.1", "GLU").unwrap();
        assert!(message.to_er7().ends_with("OBX|||GLU"));
        message.insert_segment(1, "EVN");
        assert_eq!(message.raw().segments[1].name, "EVN");
        assert_eq!(message.remove_segments("NK1"), 1);
        assert!(message.segment("NK1").is_none());
        // The header stays put whatever is asked.
        assert_eq!(message.remove_segments("MSH"), 0);
        assert!(message.remove_segment("MSH", 1).is_none());
    }

    #[test]
    fn round_trips_an_unmodified_message() {
        let text = format!("{HEADER}\rPID|1||241900||TEST^FOUAZ\rNK1|1");
        assert_eq!(crate::parse(&text).unwrap().to_er7(), text);
    }

    #[test]
    fn falls_back_to_a_flat_tree_when_the_structure_does_not_fit() {
        let message = message(); // ADT_A01 requires EVN and PV1
        assert!(message.layout().is_none());
        let tree = message.tree();
        assert_eq!(tree.name(), "ADT_A01");
        assert!(tree.child("PID").is_some(), "segments must stay reachable");
    }
}
