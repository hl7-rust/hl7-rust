//! The HL7 v3 data types: [`Ii`], [`Cd`], [`Ivl`], [`Pq`], and [`Ed`],
//! each serialized as an object. (The sixth, [`crate::NullFlavor`], is a
//! bare string and lives in its own module.)

object_wrapper! {
    /// A Serde-enabled [`hl7_3::Ii`]: an instance identifier.
    ///
    /// Serializes as `{"root": "…", "extension": "…" | null}`. `"root"` is
    /// required on the way back in; `"extension"` defaults to `null`.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Ii;
    ///
    /// let ii: Ii = serde_json::from_str(r#"{"root":"2.16.840.1.113883.19.5","extension":"MSG00001"}"#)?;
    /// assert_eq!(ii.extension.as_deref(), Some("MSG00001"));
    /// # Ok(())
    /// # }
    /// ```
    Ii wraps hl7_3::Ii,
    expecting "an II object with \"root\" and \"extension\"",
    derive [Eq, Default],
    fields {
        root: req String => "root",
        extension: opt String => "extension",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::Cd`]: a coded value.
    ///
    /// Serializes as `{"code": "…", "codeSystem": … | null, "displayName":
    /// … | null}`. `"code"` is required on the way back in.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Cd;
    ///
    /// let cd = Cd(hl7_3::Cd { code: "OBS".into(), code_system: None, display_name: None });
    /// assert_eq!(serde_json::to_string(&cd)?, r#"{"code":"OBS","codeSystem":null,"displayName":null}"#);
    /// # Ok(())
    /// # }
    /// ```
    Cd wraps hl7_3::Cd,
    expecting "a CD object with \"code\", \"codeSystem\", and \"displayName\"",
    derive [Eq, Default],
    fields {
        code: req String => "code",
        code_system: opt String => "codeSystem",
        display_name: opt String => "displayName",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::Ivl`]: an interval, as raw bound text.
    ///
    /// Serializes as `{"value": …, "low": …, "high": …}`, each a string or
    /// `null`; every key is optional on the way back in.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Ivl;
    ///
    /// let ivl: Ivl = serde_json::from_str(r#"{"low":"20260101","high":"20261231"}"#)?;
    /// assert_eq!(ivl.value, None);
    /// assert_eq!(ivl.high.as_deref(), Some("20261231"));
    /// # Ok(())
    /// # }
    /// ```
    Ivl wraps hl7_3::Ivl,
    expecting "an IVL object with \"value\", \"low\", and \"high\"",
    derive [Eq, Default],
    fields {
        value: opt String => "value",
        low: opt String => "low",
        high: opt String => "high",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::Pq`]: a physical quantity, as raw text.
    ///
    /// Serializes as `{"value": …, "unit": …}`, each a string or `null`;
    /// both keys are optional on the way back in.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Pq;
    ///
    /// let pq: Pq = serde_json::from_str(r#"{"value":"5","unit":"mg"}"#)?;
    /// assert_eq!(pq.unit.as_deref(), Some("mg"));
    /// # Ok(())
    /// # }
    /// ```
    Pq wraps hl7_3::Pq,
    expecting "a PQ object with \"value\" and \"unit\"",
    derive [Eq, Default],
    fields {
        value: opt String => "value",
        unit: opt String => "unit",
    }
}

object_wrapper! {
    /// A Serde-enabled [`hl7_3::Ed`]: encapsulated data.
    ///
    /// Serializes as `{"mediaType": …, "representation": …, "text": …}`,
    /// each a string or `null`; every key is optional on the way back in.
    /// `"text"` is carried exactly as `hl7-3` holds it — base64 stays
    /// base64.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Ed;
    ///
    /// let ed: Ed = serde_json::from_str(r#"{"mediaType":"text/plain","text":"hello"}"#)?;
    /// assert_eq!(ed.representation, None);
    /// # Ok(())
    /// # }
    /// ```
    Ed wraps hl7_3::Ed,
    expecting "an ED object with \"mediaType\", \"representation\", and \"text\"",
    derive [Eq, Default],
    fields {
        media_type: opt String => "mediaType",
        representation: opt String => "representation",
        text: opt String => "text",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Strict;

    #[test]
    fn ii_round_trips_and_requires_root() {
        let ii = Ii(hl7_3::Ii {
            root: "1.2.3".into(),
            extension: Some("7".into()),
        });
        let json = serde_json::to_string(&ii).unwrap();
        assert_eq!(json, r#"{"root":"1.2.3","extension":"7"}"#);
        let back: Ii = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ii);
        let err = serde_json::from_str::<Ii>(r#"{"extension":"7"}"#).unwrap_err();
        assert!(err.to_string().contains("root"), "{err}");
    }

    #[test]
    fn cd_round_trips() {
        let cd = Cd(hl7_3::Cd {
            code: "OBS".into(),
            code_system: Some("2.16.840.1.113883.5.6".into()),
            display_name: None,
        });
        let back: Cd = serde_json::from_str(&serde_json::to_string(&cd).unwrap()).unwrap();
        assert_eq!(back, cd);
    }

    #[test]
    fn ivl_pq_and_ed_round_trip_and_default_to_empty() {
        let ivl: Ivl = serde_json::from_str("{}").unwrap();
        assert_eq!(ivl, Ivl::default());
        let pq: Pq = serde_json::from_str(r#"{"value":"5"}"#).unwrap();
        assert_eq!(pq.value.as_deref(), Some("5"));
        assert_eq!(pq.unit, None);
        let ed = Ed(hl7_3::Ed {
            media_type: Some("application/pdf".into()),
            representation: Some("B64".into()),
            text: Some("AAAA".into()),
        });
        let back: Ed = serde_json::from_str(&serde_json::to_string(&ed).unwrap()).unwrap();
        assert_eq!(back, ed);
    }

    #[test]
    fn ignores_unknown_fields_but_strict_does_not() {
        let json = r#"{"root":"1","exension":"x"}"#;
        let plain: Ii = serde_json::from_str(json).unwrap();
        assert_eq!(plain.extension, None, "the typo is silently absent");
        let err = serde_json::from_str::<Strict<Ii>>(json).unwrap_err();
        assert!(err.to_string().contains("exension"), "{err}");
    }

    #[test]
    fn rejects_a_duplicate_key() {
        let err = serde_json::from_str::<Cd>(r#"{"code":"A","code":"B"}"#).unwrap_err();
        assert!(err.to_string().contains("duplicate"), "{err}");
    }
}
