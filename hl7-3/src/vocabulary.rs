//! The two data types RIM attributes are built from: an identifier, and a
//! coded value drawn from a vocabulary domain.
//!
//! HL7 v3 defines a much larger data type hierarchy than these two (the
//! "R2" and "ITS" specs run to dozens of types: `PQ` for physical
//! quantities, `IVL<T>` for intervals, `ED` for encapsulated data, and so
//! on). This crate models only the two that appear on every RIM backbone
//! class — see `spec/index.md` §1 for what's deliberately out of scope.

/// An **II** (Instance Identifier): a globally unique identifier, as an
/// OID or UUID `root` plus an optional locally-scoped `extension`.
///
/// This is how every RIM class names itself (`Act.id`, `Entity.id`,
/// `Role.id`) and how a message names its own interaction
/// ([`crate::message::Message::interaction_id`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ii {
    /// An OID (`2.16.840.1.113883.19.5`) or UUID that names the assigning
    /// authority or identifier scheme.
    pub root: String,
    /// The identifier within that root's scheme. Absent when `root` alone
    /// already names one specific thing (a UUID used as a instance id).
    pub extension: Option<String>,
}

/// A **CD** (Concept Descriptor): a coded value from a vocabulary domain —
/// `classCode`, `moodCode`, `typeCode`, and every other RIM attribute that
/// takes a code rather than free text.
///
/// HL7 v3's vocabulary domains (`ActClass`, `ActMood`, `EntityClass`,
/// `RoleClass`, `ParticipationType`, `ActRelationshipType`, and dozens more
/// tables) are not modeled as Rust enums here — there are too many, and a
/// caller reading a message from one sender rarely needs the full table for
/// a vocabulary it never sees a code outside of. `code` carries whatever
/// string the message actually used; validating it against a domain's
/// allowed values is future work (see `spec/index.md` §6).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cd {
    /// The code itself, e.g. `"OBS"`, `"EVN"`, `"PAT"`.
    pub code: String,
    /// The OID naming the code system (vocabulary domain) `code` is drawn
    /// from, when the message states one.
    pub code_system: Option<String>,
    /// Human-readable text for `code`, when the message carries one.
    pub display_name: Option<String>,
}

impl Ii {
    /// Read an `II` from an element's `root`/`extension` attributes.
    /// `None` when the element has no `root` — an `II` with no root names
    /// nothing.
    #[must_use]
    pub(crate) fn from_element(element: &hl7_v2_xml_lite_helper::Element) -> Option<Ii> {
        let root = element.attribute("root")?.to_string();
        let extension = element.attribute("extension").map(str::to_string);
        Some(Ii { root, extension })
    }
}

impl Cd {
    /// Read a `CD` from an element's `code`/`codeSystem`/`displayName`
    /// attributes. `None` when the element has no `code` — a `CD` with no
    /// code names nothing.
    #[must_use]
    pub(crate) fn from_element(element: &hl7_v2_xml_lite_helper::Element) -> Option<Cd> {
        let code = element.attribute("code")?.to_string();
        let code_system = element.attribute("codeSystem").map(str::to_string);
        let display_name = element.attribute("displayName").map(str::to_string);
        Some(Cd {
            code,
            code_system,
            display_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ii_reads_root_and_extension() {
        let element = hl7_v2_xml_lite_helper::parse(
            r#"<id root="2.16.840.1.113883.19.5" extension="12345"/>"#,
        )
        .unwrap();
        let ii = Ii::from_element(&element).unwrap();
        assert_eq!(ii.root, "2.16.840.1.113883.19.5");
        assert_eq!(ii.extension.as_deref(), Some("12345"));
    }

    #[test]
    fn ii_extension_is_optional() {
        let element =
            hl7_v2_xml_lite_helper::parse(r#"<id root="2.16.840.1.113883.19.5"/>"#).unwrap();
        let ii = Ii::from_element(&element).unwrap();
        assert_eq!(ii.extension, None);
    }

    #[test]
    fn ii_with_no_root_is_none() {
        let element = hl7_v2_xml_lite_helper::parse(r#"<id extension="12345"/>"#).unwrap();
        assert_eq!(Ii::from_element(&element), None);
    }

    #[test]
    fn cd_reads_code_system_and_display_name() {
        let element = hl7_v2_xml_lite_helper::parse(
            r#"<code code="OBS" codeSystem="2.16.840.1.113883.5.6" displayName="Observation"/>"#,
        )
        .unwrap();
        let cd = Cd::from_element(&element).unwrap();
        assert_eq!(cd.code, "OBS");
        assert_eq!(cd.code_system.as_deref(), Some("2.16.840.1.113883.5.6"));
        assert_eq!(cd.display_name.as_deref(), Some("Observation"));
    }

    #[test]
    fn cd_with_no_code_is_none() {
        let element =
            hl7_v2_xml_lite_helper::parse(r#"<code displayName="Observation"/>"#).unwrap();
        assert_eq!(Cd::from_element(&element), None);
    }
}
