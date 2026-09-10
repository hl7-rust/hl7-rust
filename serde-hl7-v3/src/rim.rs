//! The six RIM backbone classes — [`Act`], [`Entity`], [`Role`],
//! [`Participation`], [`ActRelationship`], [`RoleLink`] — each serialized
//! as an object.

use crate::{Cd, Ii};

object_wrapper! {
    /// A Serde-enabled [`hl7_3::rim::Act`].
    ///
    /// Keys: `"classCode"` and `"moodCode"` (strings, required), `"id"` (an
    /// array of [`Ii`], `[]` when empty), `"code"` and `"statusCode"`
    /// ([`Cd`] or `null`), `"effectiveTime"` and `"text"` (string or
    /// `null`).
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Act;
    ///
    /// let element = hl7_3::xml::parse(r#"<observation classCode="OBS" moodCode="EVN">
    ///     <id root="1.2.3" extension="1"/><code code="X"/></observation>"#)?;
    /// let act = Act(hl7_3::rim::Act::from_element(&element));
    ///
    /// let json = serde_json::to_value(&act)?;
    /// assert_eq!(json["classCode"], "OBS");
    /// assert_eq!(json["id"][0]["extension"], "1");
    /// let back: Act = serde_json::from_value(json)?;
    /// assert_eq!(back, act);
    /// # Ok(())
    /// # }
    /// ```
    Act wraps hl7_3::rim::Act,
    expecting "an Act object with \"classCode\", \"moodCode\", \"id\", \"code\", \"statusCode\", \"effectiveTime\", and \"text\"",
    derive [Eq, Default],
    fields {
        class_code: req String => "classCode",
        mood_code: req String => "moodCode",
        id: vecw Ii => "id",
        code: optw Cd => "code",
        status_code: optw Cd => "statusCode",
        effective_time: opt String => "effectiveTime",
        text: opt String => "text",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::rim::Entity`].
    ///
    /// Keys: `"classCode"` (string, required), `"determinerCode"` (string
    /// or `null`), `"id"` (an array of [`Ii`]), `"code"` ([`Cd`] or
    /// `null`), `"name"` (string or `null`).
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Entity;
    ///
    /// let entity: Entity = serde_json::from_str(r#"{"classCode":"DEV","determinerCode":"INSTANCE"}"#)?;
    /// assert!(entity.id.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    Entity wraps hl7_3::rim::Entity,
    expecting "an Entity object with \"classCode\", \"determinerCode\", \"id\", \"code\", and \"name\"",
    derive [Eq, Default],
    fields {
        class_code: req String => "classCode",
        determiner_code: opt String => "determinerCode",
        id: vecw Ii => "id",
        code: optw Cd => "code",
        name: opt String => "name",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::rim::Role`].
    ///
    /// Keys: `"classCode"` (string, required), `"id"` (an array of
    /// [`Ii`]), `"code"` and `"statusCode"` ([`Cd`] or `null`),
    /// `"effectiveTime"` (string or `null`).
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Role;
    ///
    /// let role: Role = serde_json::from_str(r#"{"classCode":"PAT","id":[{"root":"1.2.3"}]}"#)?;
    /// assert_eq!(role.id[0].root, "1.2.3");
    /// # Ok(())
    /// # }
    /// ```
    Role wraps hl7_3::rim::Role,
    expecting "a Role object with \"classCode\", \"id\", \"code\", \"statusCode\", and \"effectiveTime\"",
    derive [Eq, Default],
    fields {
        class_code: req String => "classCode",
        id: vecw Ii => "id",
        code: optw Cd => "code",
        status_code: optw Cd => "statusCode",
        effective_time: opt String => "effectiveTime",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::rim::Participation`].
    ///
    /// Keys: `"typeCode"` (string, required), `"time"` (string or `null`),
    /// `"functionCode"` ([`Cd`] or `null`).
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Participation;
    ///
    /// let p: Participation = serde_json::from_str(r#"{"typeCode":"AUT"}"#)?;
    /// assert_eq!(p.type_code, "AUT");
    /// # Ok(())
    /// # }
    /// ```
    Participation wraps hl7_3::rim::Participation,
    expecting "a Participation object with \"typeCode\", \"time\", and \"functionCode\"",
    derive [Eq, Default],
    fields {
        type_code: req String => "typeCode",
        time: opt String => "time",
        function_code: optw Cd => "functionCode",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::rim::ActRelationship`].
    ///
    /// Keys: `"typeCode"` (string, required), `"inversionInd"` (boolean or
    /// `null`).
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::ActRelationship;
    ///
    /// let r: ActRelationship = serde_json::from_str(r#"{"typeCode":"COMP","inversionInd":true}"#)?;
    /// assert_eq!(r.inversion_ind, Some(true));
    /// # Ok(())
    /// # }
    /// ```
    ActRelationship wraps hl7_3::rim::ActRelationship,
    expecting "an ActRelationship object with \"typeCode\" and \"inversionInd\"",
    derive [Eq, Default],
    fields {
        type_code: req String => "typeCode",
        inversion_ind: opt bool => "inversionInd",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::rim::RoleLink`].
    ///
    /// Keys: `"typeCode"` (string, required).
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::RoleLink;
    ///
    /// let link = RoleLink(hl7_3::rim::RoleLink { type_code: "DIRAUTH".into() });
    /// assert_eq!(serde_json::to_string(&link)?, r#"{"typeCode":"DIRAUTH"}"#);
    /// # Ok(())
    /// # }
    /// ```
    RoleLink wraps hl7_3::rim::RoleLink,
    expecting "a RoleLink object with \"typeCode\"",
    derive [Eq, Default],
    fields {
        type_code: req String => "typeCode",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Strict;

    fn act() -> Act {
        let element = hl7_3::xml::parse(
            r#"<observation classCode="OBS" moodCode="EVN">
                 <id root="1.2.3" extension="1"/><id root="1.2.3" extension="2"/>
                 <code code="X" codeSystem="2.16"/><statusCode code="completed"/>
                 <effectiveTime value="20260101"/><text>note</text>
               </observation>"#,
        )
        .unwrap();
        Act(hl7_3::rim::Act::from_element(&element))
    }

    #[test]
    fn act_round_trips_with_every_field_set() {
        let act = act();
        let json = serde_json::to_string(&act).unwrap();
        let back: Act = serde_json::from_str(&json).unwrap();
        assert_eq!(back, act);
        assert_eq!(back.id.len(), 2);
        assert_eq!(back.status_code.as_ref().unwrap().code, "completed");
    }

    #[test]
    fn act_serializes_every_key_always() {
        let json = serde_json::to_value(Act::default()).unwrap();
        let mut keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "classCode",
                "code",
                "effectiveTime",
                "id",
                "moodCode",
                "statusCode",
                "text"
            ]
        );
        assert_eq!(json["id"], serde_json::json!([]));
        assert_eq!(json["code"], serde_json::Value::Null);
    }

    #[test]
    fn required_codes_are_required() {
        let err = serde_json::from_str::<Act>(r#"{"moodCode":"EVN"}"#).unwrap_err();
        assert!(err.to_string().contains("classCode"), "{err}");
        let err = serde_json::from_str::<Entity>("{}").unwrap_err();
        assert!(err.to_string().contains("classCode"), "{err}");
        let err = serde_json::from_str::<Role>("{}").unwrap_err();
        assert!(err.to_string().contains("classCode"), "{err}");
        let err = serde_json::from_str::<Participation>(r#"{"time":"1"}"#).unwrap_err();
        assert!(err.to_string().contains("typeCode"), "{err}");
        assert!(serde_json::from_str::<ActRelationship>("{}").is_err());
        assert!(serde_json::from_str::<RoleLink>("{}").is_err());
    }

    #[test]
    fn every_class_round_trips_from_default() {
        macro_rules! check {
            ($($ty:ident),+) => {$(
                let value = $ty::default();
                let back: $ty = serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap();
                assert_eq!(back, value);
            )+};
        }
        check!(Act, Entity, Role, Participation, ActRelationship, RoleLink);
    }

    #[test]
    fn strict_reaches_into_ids_and_codes() {
        let json = r#"{"classCode":"OBS","moodCode":"EVN","id":[{"root":"1","extention":"x"}]}"#;
        assert!(serde_json::from_str::<Act>(json).is_ok());
        let err = serde_json::from_str::<Strict<Act>>(json).unwrap_err();
        assert!(err.to_string().contains("extention"), "{err}");

        let json = r#"{"typeCode":"AUT","functionCode":{"code":"X","display":"y"}}"#;
        let err = serde_json::from_str::<Strict<Participation>>(json).unwrap_err();
        assert!(err.to_string().contains("display"), "{err}");
    }

    #[test]
    fn inversion_ind_is_a_boolean() {
        let json = serde_json::to_value(ActRelationship(hl7_3::rim::ActRelationship {
            type_code: "COMP".into(),
            inversion_ind: Some(false),
        }))
        .unwrap();
        assert_eq!(json["inversionInd"], false);
    }
}
