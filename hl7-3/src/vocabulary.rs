//! The data types RIM attributes are built from: identifiers, coded
//! values, intervals, quantities, encapsulated data, and the explicit-null
//! mechanism every one of them can carry instead of a value.
//!
//! HL7 v3's full "R2"/"ITS" data type hierarchy runs to dozens of types;
//! this module models the ones that appear across the RIM backbone classes
//! and real messages often enough to be worth a type of their own — see
//! `spec/index.md` §1 for what's still deliberately out of scope (the rest
//! of the hierarchy, and vocabulary domain validation).

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

/// An **IVL** (Interval): a range with an optional low bound, high bound,
/// or a single point value — HL7 v3's shape for "when"
/// (`effectiveTime`) and "over what range" (a dose interval, a reference
/// range).
///
/// Bounds are kept as the raw text HL7 v3 serializes them as (a `TS`
/// timestamp string, a number) — this crate does not parse timestamps or
/// numbers further; see `spec/index.md` §1.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ivl {
    /// A single point in the interval, present instead of `low`/`high`
    /// when the interval collapses to one instant:
    /// `<effectiveTime value="20260101"/>`.
    pub value: Option<String>,
    /// The interval's lower bound: `<low value="20260101"/>`.
    pub low: Option<String>,
    /// The interval's upper bound: `<high value="20261231"/>`.
    pub high: Option<String>,
}

/// A **PQ** (Physical Quantity): a numeric value with a unit —
/// `<doseQuantity value="5" unit="mg"/>`.
///
/// `unit` follows UCUM when the sender does; this crate does not validate
/// it against that table, the same "keep what was sent" choice [`Cd`]
/// makes for `code`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pq {
    /// The number, as sent — not parsed, since HL7 v3 allows more numeric
    /// forms (fractions, uncertainty ranges) than one Rust type covers.
    pub value: Option<String>,
    /// The unit, e.g. `"mg"`, `"mL"`, `"1"` for a dimensionless count.
    pub unit: Option<String>,
}

/// An **ED** (Encapsulated Data): a blob of content with a declared
/// shape — a clinical note's narrative, an attached document, an image.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ed {
    /// The IANA media type, e.g. `"text/plain"`, `"application/pdf"`.
    pub media_type: Option<String>,
    /// How `text` is encoded, e.g. `"B64"` for base64. Absent means plain
    /// text.
    pub representation: Option<String>,
    /// The content itself: plain text if `representation` is absent,
    /// otherwise still the raw encoded text (base64 is not decoded here).
    pub text: Option<String>,
}

/// Why a value is explicitly absent, when the sender says so rather than
/// simply omitting the attribute or element — HL7 v3's `nullFlavor`,
/// which any data type can carry instead of a value.
///
/// This is the v3 equivalent of the distinction
/// [`hl7-2`](https://crates.io/crates/hl7-2) draws between an absent field
/// and its explicit HL7 null `""`: "nobody said" and "someone said there
/// is nothing to say, and why" are different facts.
///
/// HL7 v3's real `NullFlavor` domain has many more values than these
/// seven; these are the ones common enough across real interfaces to be
/// worth a named variant. Anything else reads as [`NullFlavor::Unrecognized`],
/// carrying the code as sent, rather than being rejected — see
/// `spec/index.md` §3.6 for why this crate does not enumerate the whole
/// domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullFlavor {
    /// `NI` — no information at all.
    NoInformation,
    /// `UNK` — unknown.
    Unknown,
    /// `ASKU` — asked, but the source did not know.
    AskedButUnknown,
    /// `NASK` — not asked.
    NotAsked,
    /// `NAV` — temporarily unavailable.
    TemporarilyUnavailable,
    /// `NA` — not applicable in this context.
    NotApplicable,
    /// `OTH` — a value exists but is outside the domain this field allows.
    Other,
    /// Any other code, carried exactly as sent.
    Unrecognized(String),
}

impl NullFlavor {
    /// Read a `NullFlavor` from a `nullFlavor` code, e.g. as read off an
    /// attribute.
    #[must_use]
    pub fn parse(code: &str) -> NullFlavor {
        match code {
            "NI" => NullFlavor::NoInformation,
            "UNK" => NullFlavor::Unknown,
            "ASKU" => NullFlavor::AskedButUnknown,
            "NASK" => NullFlavor::NotAsked,
            "NAV" => NullFlavor::TemporarilyUnavailable,
            "NA" => NullFlavor::NotApplicable,
            "OTH" => NullFlavor::Other,
            other => NullFlavor::Unrecognized(other.to_string()),
        }
    }

    /// The code this value reads back as — the inverse of [`NullFlavor::parse`].
    #[must_use]
    pub fn as_code(&self) -> &str {
        match self {
            NullFlavor::NoInformation => "NI",
            NullFlavor::Unknown => "UNK",
            NullFlavor::AskedButUnknown => "ASKU",
            NullFlavor::NotAsked => "NASK",
            NullFlavor::TemporarilyUnavailable => "NAV",
            NullFlavor::NotApplicable => "NA",
            NullFlavor::Other => "OTH",
            NullFlavor::Unrecognized(code) => code,
        }
    }

    /// Read the `nullFlavor` attribute off `element`, if it has one.
    #[must_use]
    pub fn of(element: &hl7_2_xml_lite_helper::Element) -> Option<NullFlavor> {
        element.attribute("nullFlavor").map(NullFlavor::parse)
    }
}

impl Ii {
    /// Read an `II` from an element's `root`/`extension` attributes.
    /// `None` when the element has no `root` — an `II` with no root names
    /// nothing.
    #[must_use]
    pub(crate) fn from_element(element: &hl7_2_xml_lite_helper::Element) -> Option<Ii> {
        let root = element.attribute("root")?.to_string();
        let extension = element.attribute("extension").map(str::to_string);
        Some(Ii { root, extension })
    }
}

impl Ivl {
    /// Read an `IVL` from an element: its own `value` attribute for a
    /// point, or `low`/`high` children's `value` attributes for a range.
    #[must_use]
    pub fn from_element(element: &hl7_2_xml_lite_helper::Element) -> Ivl {
        Ivl {
            value: element.attribute("value").map(str::to_string),
            low: element
                .child("low")
                .and_then(|low| low.attribute("value"))
                .map(str::to_string),
            high: element
                .child("high")
                .and_then(|high| high.attribute("value"))
                .map(str::to_string),
        }
    }
}

impl Pq {
    /// Read a `PQ` from an element's `value`/`unit` attributes.
    #[must_use]
    pub fn from_element(element: &hl7_2_xml_lite_helper::Element) -> Pq {
        Pq {
            value: element.attribute("value").map(str::to_string),
            unit: element.attribute("unit").map(str::to_string),
        }
    }
}

impl Ed {
    /// Read an `ED` from an element's `mediaType`/`representation`
    /// attributes and its own text.
    #[must_use]
    pub fn from_element(element: &hl7_2_xml_lite_helper::Element) -> Ed {
        Ed {
            media_type: element.attribute("mediaType").map(str::to_string),
            representation: element.attribute("representation").map(str::to_string),
            text: element.text_opt().map(str::to_string),
        }
    }
}

impl Cd {
    /// Read a `CD` from an element's `code`/`codeSystem`/`displayName`
    /// attributes. `None` when the element has no `code` — a `CD` with no
    /// code names nothing.
    #[must_use]
    pub(crate) fn from_element(element: &hl7_2_xml_lite_helper::Element) -> Option<Cd> {
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
        let element = hl7_2_xml_lite_helper::parse(
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
            hl7_2_xml_lite_helper::parse(r#"<id root="2.16.840.1.113883.19.5"/>"#).unwrap();
        let ii = Ii::from_element(&element).unwrap();
        assert_eq!(ii.extension, None);
    }

    #[test]
    fn ii_with_no_root_is_none() {
        let element = hl7_2_xml_lite_helper::parse(r#"<id extension="12345"/>"#).unwrap();
        assert_eq!(Ii::from_element(&element), None);
    }

    #[test]
    fn cd_reads_code_system_and_display_name() {
        let element = hl7_2_xml_lite_helper::parse(
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
        let element = hl7_2_xml_lite_helper::parse(r#"<code displayName="Observation"/>"#).unwrap();
        assert_eq!(Cd::from_element(&element), None);
    }

    #[test]
    fn ivl_reads_a_point_value() {
        let element = hl7_2_xml_lite_helper::parse(r#"<effectiveTime value="20260101"/>"#).unwrap();
        let ivl = Ivl::from_element(&element);
        assert_eq!(ivl.value.as_deref(), Some("20260101"));
        assert_eq!(ivl.low, None);
        assert_eq!(ivl.high, None);
    }

    #[test]
    fn ivl_reads_low_and_high() {
        let element = hl7_2_xml_lite_helper::parse(
            r#"<effectiveTime><low value="20260101"/><high value="20261231"/></effectiveTime>"#,
        )
        .unwrap();
        let ivl = Ivl::from_element(&element);
        assert_eq!(ivl.value, None);
        assert_eq!(ivl.low.as_deref(), Some("20260101"));
        assert_eq!(ivl.high.as_deref(), Some("20261231"));
    }

    #[test]
    fn ivl_with_nothing_reads_all_none() {
        let element = hl7_2_xml_lite_helper::parse(r"<effectiveTime/>").unwrap();
        assert_eq!(Ivl::from_element(&element), Ivl::default());
    }

    #[test]
    fn pq_reads_value_and_unit() {
        let element =
            hl7_2_xml_lite_helper::parse(r#"<doseQuantity value="5" unit="mg"/>"#).unwrap();
        let pq = Pq::from_element(&element);
        assert_eq!(pq.value.as_deref(), Some("5"));
        assert_eq!(pq.unit.as_deref(), Some("mg"));
    }

    #[test]
    fn ed_reads_media_type_representation_and_text() {
        let element = hl7_2_xml_lite_helper::parse(
            r#"<text mediaType="text/plain" representation="TXT">Patient reports pain.</text>"#,
        )
        .unwrap();
        let ed = Ed::from_element(&element);
        assert_eq!(ed.media_type.as_deref(), Some("text/plain"));
        assert_eq!(ed.representation.as_deref(), Some("TXT"));
        assert_eq!(ed.text.as_deref(), Some("Patient reports pain."));
    }

    #[test]
    fn ed_with_no_attributes_still_reads_text() {
        let element = hl7_2_xml_lite_helper::parse(r"<text>Plain narrative.</text>").unwrap();
        let ed = Ed::from_element(&element);
        assert_eq!(ed.media_type, None);
        assert_eq!(ed.representation, None);
        assert_eq!(ed.text.as_deref(), Some("Plain narrative."));
    }

    #[test]
    fn null_flavor_parses_the_seven_named_codes() {
        assert_eq!(NullFlavor::parse("NI"), NullFlavor::NoInformation);
        assert_eq!(NullFlavor::parse("UNK"), NullFlavor::Unknown);
        assert_eq!(NullFlavor::parse("ASKU"), NullFlavor::AskedButUnknown);
        assert_eq!(NullFlavor::parse("NASK"), NullFlavor::NotAsked);
        assert_eq!(NullFlavor::parse("NAV"), NullFlavor::TemporarilyUnavailable);
        assert_eq!(NullFlavor::parse("NA"), NullFlavor::NotApplicable);
        assert_eq!(NullFlavor::parse("OTH"), NullFlavor::Other);
    }

    #[test]
    fn null_flavor_parse_and_as_code_round_trip() {
        for code in ["NI", "UNK", "ASKU", "NASK", "NAV", "NA", "OTH", "MSK"] {
            assert_eq!(NullFlavor::parse(code).as_code(), code);
        }
    }

    #[test]
    fn null_flavor_of_reads_the_attribute() {
        let element = hl7_2_xml_lite_helper::parse(r#"<value nullFlavor="UNK"/>"#).unwrap();
        assert_eq!(NullFlavor::of(&element), Some(NullFlavor::Unknown));

        let element = hl7_2_xml_lite_helper::parse(r#"<value value="7"/>"#).unwrap();
        assert_eq!(NullFlavor::of(&element), None);
    }
}
