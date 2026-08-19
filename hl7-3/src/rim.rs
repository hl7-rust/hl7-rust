//! The Reference Information Model (RIM): the six backbone classes every
//! HL7 v3 domain payload is built from.
//!
//! Where HL7 v2 names segments and fields per message type, HL7 v3 names
//! six classes once and reuses them everywhere:
//!
//! - [`Act`] — something that happened, is happening, or is intended to
//!   happen: an observation, a procedure, an administration of a
//!   substance. `moodCode` is what makes this one class cover both "this
//!   was done" (`EVN`) and "please do this" (`RQO`) — an intent and its
//!   eventual fulfillment are the same kind of thing at different moods,
//!   not two different classes.
//! - [`Entity`] — a physical thing: a person, an organization, a device,
//!   a place. `determinerCode` distinguishes one specific instance
//!   (`INSTANCE`) from a kind of thing in general (`KIND`).
//! - [`Role`] — a competency one [`Entity`] has with respect to another:
//!   *this* person as *patient*, *that* organization as *provider*. The
//!   same [`Entity`] plays different [`Role`]s in different messages.
//! - [`Participation`] — links an [`Act`] to the [`Role`] that took part
//!   in it, and how (`AUT` author, `PRF` performer, `SBJ` subject, ...).
//! - [`ActRelationship`] — links two [`Act`]s (`COMP` component, `RSON`
//!   reason, `PERT` pertains to, ...), which is how a single act becomes
//!   a document made of sections made of entries.
//! - [`RoleLink`] — links two [`Role`]s, less common than the other five.
//!
//! Every attribute below is read straight off the matching XML element or
//! attribute by [`crate::message::parse`]'s domain-payload walk, or can be
//! read directly from a [`hl7_2_xml_lite_helper::Element`] with the
//! `from_element` methods here. Nothing here validates that a `classCode`
//! or `moodCode` is one of the values its vocabulary domain actually
//! allows — see `spec/index.md` §6 for why that is future work, not a
//! missing check.

use crate::vocabulary::{Cd, Ii};
use hl7_2_xml_lite_helper::Element;

/// Something that happened, is happening, or is intended to happen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Act {
    /// What kind of act this is (`ActClass`: `OBS` observation, `PROC`
    /// procedure, `SBADM` substance administration, ...).
    pub class_code: String,
    /// Where the act sits between intent and fact (`ActMood`: `EVN` it
    /// happened, `INT` it is intended, `RQO` it is requested, ...).
    pub mood_code: String,
    /// This act's own identifiers.
    pub id: Vec<Ii>,
    /// What the act is, more specifically than `classCode` alone says.
    pub code: Option<Cd>,
    /// Where the act stands (`ActStatus`: `active`, `completed`,
    /// `cancelled`, ...).
    pub status_code: Option<Cd>,
    /// When the act happened, happens, or is meant to happen. Carried as
    /// the raw text HL7 v3's `TS`/`IVL<TS>` types serialize to — see
    /// `spec/index.md` §1 for why this crate doesn't parse it further yet.
    pub effective_time: Option<String>,
    /// Free text describing the act, when the message carries one
    /// alongside or instead of `code`.
    pub text: Option<String>,
}

/// A physical thing: a person, organization, device, or place.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entity {
    /// What kind of thing this is (`EntityClass`: `PSN` person, `ORG`
    /// organization, `DEV` device, `PLC` place, ...).
    pub class_code: String,
    /// Whether this element names one specific entity (`INSTANCE`) or a
    /// kind of entity in general (`KIND`).
    pub determiner_code: Option<String>,
    /// This entity's own identifiers.
    pub id: Vec<Ii>,
    /// What kind of entity this is, more specifically than `classCode`
    /// alone says (a device's model, an organization's type).
    pub code: Option<Cd>,
    /// A name for this entity, when it has one as free text (a person's or
    /// organization's name is usually structured in real messages; this
    /// crate reads it as text — see `spec/index.md` §1).
    pub name: Option<String>,
}

/// A competency one [`Entity`] has with respect to another: this person as
/// patient, that organization as provider.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Role {
    /// What kind of role this is (`RoleClass`: `PAT` patient, `PROV`
    /// provider, `ASSIGNED`, ...).
    pub class_code: String,
    /// This role's own identifiers — distinct from the identifiers of the
    /// [`Entity`] playing it.
    pub id: Vec<Ii>,
    /// What the role is, more specifically than `classCode` alone says.
    pub code: Option<Cd>,
    /// Where the role stands (`RoleStatus`: `active`, `terminated`, ...).
    pub status_code: Option<Cd>,
    /// When this role applies, as raw text (see [`Act::effective_time`]).
    pub effective_time: Option<String>,
}

/// Links an [`Act`] to the [`Role`] that took part in it, and how.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Participation {
    /// How the role took part (`ParticipationType`: `AUT` author, `PRF`
    /// performer, `SBJ` subject, `LOC` location, ...).
    pub type_code: String,
    /// When this participation applies, as raw text.
    pub time: Option<String>,
    /// A finer-grained statement of the role's function in this act, when
    /// `typeCode` alone doesn't say enough.
    pub function_code: Option<Cd>,
}

/// Links two [`Act`]s — the mechanism a single top-level act becomes a
/// document built of sections built of entries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActRelationship {
    /// How the two acts relate (`ActRelationshipType`: `COMP` component,
    /// `RSON` reason, `PERT` pertains to, ...).
    pub type_code: String,
    /// Whether the relationship reads in the stated direction (`false`,
    /// the default) or reversed (`true`).
    pub inversion_ind: Option<bool>,
}

/// Links two [`Role`]s. Less common in practice than the other five
/// backbone classes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RoleLink {
    /// How the two roles relate (`RoleLinkType`).
    pub type_code: String,
}

impl Act {
    /// Read an [`Act`] from its XML element: `classCode`/`moodCode`
    /// attributes, an `id` child per identifier, a `code` child, a
    /// `statusCode` child, an `effectiveTime` child's `value` attribute,
    /// and a `text` child's text.
    #[must_use]
    pub fn from_element(element: &Element) -> Act {
        Act {
            class_code: element
                .attribute("classCode")
                .unwrap_or_default()
                .to_string(),
            mood_code: element
                .attribute("moodCode")
                .unwrap_or_default()
                .to_string(),
            id: element
                .children_named("id")
                .filter_map(Ii::from_element)
                .collect(),
            code: element.child("code").and_then(Cd::from_element),
            status_code: element.child("statusCode").and_then(Cd::from_element),
            effective_time: element
                .child("effectiveTime")
                .and_then(|time| time.attribute("value"))
                .map(str::to_string),
            text: element
                .child("text")
                .and_then(Element::text_opt)
                .map(str::to_string),
        }
    }
}

impl Entity {
    /// Read an [`Entity`] from its XML element, the same way
    /// [`Act::from_element`] reads an `Act`.
    #[must_use]
    pub fn from_element(element: &Element) -> Entity {
        Entity {
            class_code: element
                .attribute("classCode")
                .unwrap_or_default()
                .to_string(),
            determiner_code: element.attribute("determinerCode").map(str::to_string),
            id: element
                .children_named("id")
                .filter_map(Ii::from_element)
                .collect(),
            code: element.child("code").and_then(Cd::from_element),
            name: element
                .child("name")
                .and_then(Element::text_opt)
                .map(str::to_string),
        }
    }
}

impl Role {
    /// Read a [`Role`] from its XML element.
    #[must_use]
    pub fn from_element(element: &Element) -> Role {
        Role {
            class_code: element
                .attribute("classCode")
                .unwrap_or_default()
                .to_string(),
            id: element
                .children_named("id")
                .filter_map(Ii::from_element)
                .collect(),
            code: element.child("code").and_then(Cd::from_element),
            status_code: element.child("statusCode").and_then(Cd::from_element),
            effective_time: element
                .child("effectiveTime")
                .and_then(|time| time.attribute("value"))
                .map(str::to_string),
        }
    }
}

impl Participation {
    /// Read a [`Participation`] from its XML element.
    #[must_use]
    pub fn from_element(element: &Element) -> Participation {
        Participation {
            type_code: element
                .attribute("typeCode")
                .unwrap_or_default()
                .to_string(),
            time: element
                .child("time")
                .and_then(|time| time.attribute("value"))
                .map(str::to_string),
            function_code: element.child("functionCode").and_then(Cd::from_element),
        }
    }
}

impl ActRelationship {
    /// Read an [`ActRelationship`] from its XML element.
    #[must_use]
    pub fn from_element(element: &Element) -> ActRelationship {
        ActRelationship {
            type_code: element
                .attribute("typeCode")
                .unwrap_or_default()
                .to_string(),
            inversion_ind: element
                .attribute("inversionInd")
                .map(|value| value == "true"),
        }
    }
}

impl RoleLink {
    /// Read a [`RoleLink`] from its XML element.
    #[must_use]
    pub fn from_element(element: &Element) -> RoleLink {
        RoleLink {
            type_code: element
                .attribute("typeCode")
                .unwrap_or_default()
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn act_reads_class_mood_id_code_status_and_time() {
        let element = hl7_2_xml_lite_helper::parse(
            r#"<observation classCode="OBS" moodCode="EVN">
                 <id root="2.16.840.1.113883.19.5" extension="1"/>
                 <code code="8302-2" codeSystem="2.16.840.1.113883.6.1" displayName="Height"/>
                 <statusCode code="completed"/>
                 <effectiveTime value="20260101"/>
               </observation>"#,
        )
        .unwrap();
        let act = Act::from_element(&element);
        assert_eq!(act.class_code, "OBS");
        assert_eq!(act.mood_code, "EVN");
        assert_eq!(act.id.len(), 1);
        assert_eq!(act.id[0].extension.as_deref(), Some("1"));
        assert_eq!(act.code.unwrap().code, "8302-2");
        assert_eq!(act.status_code.unwrap().code, "completed");
        assert_eq!(act.effective_time.as_deref(), Some("20260101"));
    }

    #[test]
    fn act_with_no_optional_children_still_reads() {
        let element =
            hl7_2_xml_lite_helper::parse(r#"<act classCode="ACT" moodCode="EVN"/>"#).unwrap();
        let act = Act::from_element(&element);
        assert_eq!(act.id, Vec::new());
        assert_eq!(act.code, None);
        assert_eq!(act.status_code, None);
        assert_eq!(act.effective_time, None);
        assert_eq!(act.text, None);
    }

    #[test]
    fn entity_reads_class_determiner_and_name() {
        let element = hl7_2_xml_lite_helper::parse(
            r#"<representedOrganization classCode="ORG" determinerCode="INSTANCE">
                 <id root="2.16.840.1.113883.19.5"/>
                 <name>Acme Clinic</name>
               </representedOrganization>"#,
        )
        .unwrap();
        let entity = Entity::from_element(&element);
        assert_eq!(entity.class_code, "ORG");
        assert_eq!(entity.determiner_code.as_deref(), Some("INSTANCE"));
        assert_eq!(entity.name.as_deref(), Some("Acme Clinic"));
    }

    #[test]
    fn role_reads_class_and_status() {
        let element = hl7_2_xml_lite_helper::parse(
            r#"<patient classCode="PAT">
                 <id root="2.16.840.1.113883.19.5" extension="12345"/>
                 <statusCode code="active"/>
               </patient>"#,
        )
        .unwrap();
        let role = Role::from_element(&element);
        assert_eq!(role.class_code, "PAT");
        assert_eq!(role.id[0].extension.as_deref(), Some("12345"));
        assert_eq!(role.status_code.unwrap().code, "active");
    }

    #[test]
    fn participation_reads_type_and_function() {
        let element = hl7_2_xml_lite_helper::parse(
            r#"<author typeCode="AUT"><time value="20260101"/></author>"#,
        )
        .unwrap();
        let participation = Participation::from_element(&element);
        assert_eq!(participation.type_code, "AUT");
        assert_eq!(participation.time.as_deref(), Some("20260101"));
    }

    #[test]
    fn act_relationship_reads_type_and_inversion() {
        let element =
            hl7_2_xml_lite_helper::parse(r#"<component typeCode="COMP" inversionInd="true"/>"#)
                .unwrap();
        let relationship = ActRelationship::from_element(&element);
        assert_eq!(relationship.type_code, "COMP");
        assert_eq!(relationship.inversion_ind, Some(true));
    }

    #[test]
    fn role_link_reads_type() {
        let element = hl7_2_xml_lite_helper::parse(r#"<roleLink typeCode="REPL"/>"#).unwrap();
        assert_eq!(RoleLink::from_element(&element).type_code, "REPL");
    }
}
