//! Struct mode: compile-time types read off an XML element, instead of
//! walking [`crate::rim`] types by hand.
//!
//! ```
//! # #[cfg(feature = "derive")] fn main() {
//! use hl7_3::rim::Act;
//! use hl7_3::FromElement;
//!
//! #[derive(FromElement, Default)]
//! struct Observation {
//!     #[element("classCode")] class_code: String,
//!     #[element(nested = "component")] component: Act, // Act's own FromElement
//!     #[element(raw)] raw: hl7_3::xml::Element,
//! }
//!
//! let element = hl7_3::xml::parse(
//!     r#"<observation classCode="OBS"><component classCode="ACT" moodCode="RQO"/></observation>"#,
//! )
//! .unwrap();
//! let observation = Observation::from_element(&element).unwrap();
//! assert_eq!(observation.class_code, "OBS");
//! assert_eq!(observation.component.mood_code, "RQO");
//! # }
//! # #[cfg(not(feature = "derive"))] fn main() {}
//! ```
//!
//! ## Absent is a default, malformed is an error
//!
//! `hl7-2`'s [`FromHl7`](https://docs.rs/hl7-2/latest/hl7_2/trait.FromHl7.html)
//! fails on a field a v2 dictionary says is required. `hl7-3` has no
//! dictionary, so an *absent* attribute or child still reads as a default,
//! the same way [`crate::rim`] and [`crate::message`] read. A value that is
//! *present but malformed* (a number that is not a number, one out of the
//! type's range, a boolean that is not a boolean) is not replaced: it is
//! [`crate::Error::InvalidValue`], naming the field.

use crate::Error;
use crate::xml::Element;

/// A type that can be read out of an XML element — struct mode's entry
/// point. Implement it by hand, or derive it with `#[derive(FromElement)]`
/// (`hl7-3-derive`, behind this crate's `derive` feature) and one
/// `#[element(...)]` attribute per field:
///
/// | attribute | reads |
/// |---|---|
/// | `#[element("classCode")]` | the `classCode` attribute, via [`FromElementValue::from_attribute`] |
/// | `#[element(child = "id")]` | the `id` child's text, via [`FromElementValue::from_child_text`] |
/// | `#[element(nested = "code")]` | the `code` child, via the field type's own `FromElement` |
/// | `#[element(raw)]` | the whole element (field type must be [`Element`]) |
/// | none | `Default::default()` |
pub trait FromElement: Sized {
    /// Read `element` into `Self`. A missing attribute or child reads as a
    /// default; one that is present but malformed is
    /// [`Error::InvalidValue`].
    ///
    /// # Errors
    ///
    /// [`Error::InvalidValue`] when a present value is not the field's type.
    fn from_element(element: &Element) -> Result<Self, Error>;
}

/// A single field's value, read from an attribute or a child element's
/// text — the two atoms [`FromElement`] fields are built from.
///
/// Implemented for [`String`], [`bool`], and the integer and
/// floating-point types. A value that is absent reads as the type's
/// `Default`; one that is present but not this type's shape is an
/// [`Error::InvalidValue`] naming the field.
pub trait FromElementValue: Sized {
    /// Read from an attribute's value, or `None` when the attribute is
    /// absent. `name` is the attribute, for the error message.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidValue`] when the value is present but malformed.
    fn from_attribute(name: &str, value: Option<&str>) -> Result<Self, Error>;
    /// Read from a child element's text, or `None` when the child is
    /// absent or has no text. `name` is the child, for the error message.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidValue`] when the text is present but malformed.
    fn from_child_text(name: &str, text: Option<&str>) -> Result<Self, Error>;
}

impl FromElementValue for String {
    fn from_attribute(_name: &str, value: Option<&str>) -> Result<String, Error> {
        Ok(value.unwrap_or_default().to_string())
    }

    fn from_child_text(_name: &str, text: Option<&str>) -> Result<String, Error> {
        Ok(text.unwrap_or_default().to_string())
    }
}

impl FromElementValue for Option<String> {
    fn from_attribute(_name: &str, value: Option<&str>) -> Result<Option<String>, Error> {
        Ok(value.map(str::to_string))
    }

    fn from_child_text(_name: &str, text: Option<&str>) -> Result<Option<String>, Error> {
        Ok(text.map(str::to_string))
    }
}

/// Accepts the `xs:boolean` spellings `true`, `false`, `1` and `0` (after
/// trimming). Absent is `false`; a value that is present but none of those
/// is [`Error::InvalidValue`].
impl FromElementValue for bool {
    fn from_attribute(name: &str, value: Option<&str>) -> Result<bool, Error> {
        parse_bool(name, value)
    }

    fn from_child_text(name: &str, text: Option<&str>) -> Result<bool, Error> {
        parse_bool(name, text)
    }
}

fn parse_bool(name: &str, value: Option<&str>) -> Result<bool, Error> {
    match value.map(str::trim) {
        Some("true" | "1") => Ok(true),
        // RCA-IN-3: absent only. hl7-3 spec section 8.1: an absent field reads as the type's default.
        None | Some("false" | "0") => Ok(false),
        Some(other) => Err(Error::InvalidValue {
            name: name.to_string(),
            value: other.to_string(),
            expected: "a boolean (true, false, 1 or 0)",
        }),
    }
}

/// Numbers parse with their own `FromStr`. Absent is `0`; a value that is
/// present but unparseable or outside the type's range is
/// [`Error::InvalidValue`] naming the field, never a default or a clamp.
macro_rules! numbers {
    ($($type:ty),*) => {$(
        impl FromElementValue for $type {
            fn from_attribute(name: &str, value: Option<&str>) -> Result<$type, Error> {
                match value {
                    // RCA-IN-3: absent only. hl7-3 spec section 8.1: an absent number reads as 0.
                    None => Ok(<$type as Default>::default()),
                    Some(text) => text.trim().parse().map_err(|_| Error::InvalidValue {
                        name: name.to_string(),
                        value: text.to_string(),
                        expected: stringify!($type),
                    }),
                }
            }

            fn from_child_text(name: &str, text: Option<&str>) -> Result<$type, Error> {
                <$type as FromElementValue>::from_attribute(name, text)
            }
        }
    )*};
}

numbers!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

/// Absent, or a code this crate doesn't recognize as one of its
/// [`crate::vocabulary::NullFlavor`] variants, both read as `None` — the
/// same "not every code needs a named variant" choice
/// [`crate::vocabulary::NullFlavor::Unrecognized`] makes, one level up.
/// Prefer `element.attribute("nullFlavor")` directly, or
/// [`crate::vocabulary::NullFlavor::of`], when you need the code even for
/// values this type doesn't name.
impl FromElementValue for Option<crate::vocabulary::NullFlavor> {
    fn from_attribute(
        _name: &str,
        value: Option<&str>,
    ) -> Result<Option<crate::vocabulary::NullFlavor>, Error> {
        Ok(value.map(crate::vocabulary::NullFlavor::parse))
    }

    fn from_child_text(
        _name: &str,
        text: Option<&str>,
    ) -> Result<Option<crate::vocabulary::NullFlavor>, Error> {
        Ok(text.map(crate::vocabulary::NullFlavor::parse))
    }
}

impl FromElement for crate::vocabulary::Ivl {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::vocabulary::Ivl::from_element(element))
    }
}

impl FromElement for crate::vocabulary::Pq {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::vocabulary::Pq::from_element(element))
    }
}

impl FromElement for crate::vocabulary::Ed {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::vocabulary::Ed::from_element(element))
    }
}

impl FromElement for crate::rim::Act {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::rim::Act::from_element(element))
    }
}

impl FromElement for crate::rim::Entity {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::rim::Entity::from_element(element))
    }
}

impl FromElement for crate::rim::Role {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::rim::Role::from_element(element))
    }
}

impl FromElement for crate::rim::Participation {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::rim::Participation::from_element(element))
    }
}

impl FromElement for crate::rim::ActRelationship {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::rim::ActRelationship::from_element(element))
    }
}

impl FromElement for crate::rim::RoleLink {
    fn from_element(element: &Element) -> Result<Self, Error> {
        Ok(crate::rim::RoleLink::from_element(element))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rim::Act;

    #[derive(Debug, Default)]
    struct Observation {
        class_code: String,
        mood_code: String,
        note: Option<String>,
        component: Act,
        raw: Element,
    }

    // The shape `#[derive(FromElement)]` generates, written out so the
    // trait is tested without the derive feature.
    impl FromElement for Observation {
        fn from_element(element: &Element) -> Result<Observation, Error> {
            Ok(Observation {
                class_code: FromElementValue::from_attribute(
                    "classCode",
                    element.attribute("classCode"),
                )?,
                mood_code: FromElementValue::from_attribute(
                    "moodCode",
                    element.attribute("moodCode"),
                )?,
                note: FromElementValue::from_child_text(
                    "note",
                    element.child("note").and_then(Element::text_opt),
                )?,
                component: element.child("component").map_or_default(Act::from_element),
                raw: element.clone(),
            })
        }
    }

    #[test]
    fn reads_attributes_child_text_and_nested_types() {
        let element = crate::xml::parse(
            r#"<observation classCode="OBS" moodCode="EVN">
                 <note>elevated</note>
                 <component classCode="ACT" moodCode="EVN"/>
               </observation>"#,
        )
        .unwrap();
        let observation = Observation::from_element(&element).unwrap();
        assert_eq!(observation.class_code, "OBS");
        assert_eq!(observation.mood_code, "EVN");
        assert_eq!(observation.note.as_deref(), Some("elevated"));
        assert_eq!(observation.component.class_code, "ACT");
        assert_eq!(observation.raw.local_name(), "observation");
    }

    #[test]
    fn missing_attributes_and_children_degrade_to_defaults() {
        let element = crate::xml::parse(r"<observation/>").unwrap();
        let observation = Observation::from_element(&element).unwrap();
        assert_eq!(observation.class_code, "");
        assert_eq!(observation.note, None);
        assert_eq!(observation.component, Act::default());
    }

    fn invalid(name: &str, value: &str, expected: &'static str) -> Error {
        Error::InvalidValue {
            name: name.to_string(),
            value: value.to_string(),
            expected,
        }
    }

    #[test]
    fn numbers_and_bool_reject_malformed_input_and_default_only_when_absent() {
        assert_eq!(u32::from_attribute("n", Some("7")), Ok(7));
        assert_eq!(u32::from_attribute("n", None), Ok(0));
        assert_eq!(u32::from_child_text("n", None), Ok(0));
        assert_eq!(
            u32::from_attribute("n", Some("not a number")),
            Err(invalid("n", "not a number", "u32"))
        );
        // First out-of-range input for each width is rejected, not wrapped, clamped or zeroed.
        assert_eq!(u8::from_attribute("n", Some("255")), Ok(255));
        assert_eq!(
            u8::from_attribute("n", Some("256")),
            Err(invalid("n", "256", "u8"))
        );
        assert_eq!(i8::from_child_text("n", Some("-128")), Ok(-128));
        assert_eq!(
            i8::from_child_text("n", Some("-129")),
            Err(invalid("n", "-129", "i8"))
        );
        assert_eq!(
            u8::from_child_text("n", Some("-1")),
            Err(invalid("n", "-1", "u8"))
        );
        assert_eq!(
            f64::from_attribute("n", Some("")),
            Err(invalid("n", "", "f64"))
        );
        assert_eq!(bool::from_attribute("b", Some("true")), Ok(true));
        assert_eq!(bool::from_attribute("b", Some(" 1 ")), Ok(true));
        assert_eq!(bool::from_attribute("b", Some("0")), Ok(false));
        assert_eq!(bool::from_attribute("b", None), Ok(false));
        assert_eq!(
            bool::from_attribute("b", Some("yes")),
            Err(invalid("b", "yes", "a boolean (true, false, 1 or 0)"))
        );
    }

    #[test]
    fn from_element_names_the_malformed_field() {
        #[derive(Debug)]
        struct Counted {
            _count: u8,
        }
        impl FromElement for Counted {
            fn from_element(element: &Element) -> Result<Counted, Error> {
                Ok(Counted {
                    _count: FromElementValue::from_attribute("count", element.attribute("count"))?,
                })
            }
        }
        let element = crate::xml::parse(r#"<x count="300"/>"#).unwrap();
        assert_eq!(
            Counted::from_element(&element).unwrap_err(),
            invalid("count", "300", "u8")
        );
    }

    #[test]
    fn every_scalar_implementation_reads_attributes_and_child_text() {
        use crate::vocabulary::NullFlavor;

        macro_rules! assert_number {
            ($($type:ty),*) => {$({
                let seven: $type = "7".parse().unwrap();
                let eight: $type = "8".parse().unwrap();
                assert_eq!(<$type>::from_attribute("n", Some(" 7 ")), Ok(seven));
                assert_eq!(<$type>::from_child_text("n", Some(" 8 ")), Ok(eight));
            })*};
        }
        assert_number!(
            i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
        );
        assert_eq!(bool::from_child_text("n", Some("true")), Ok(true));
        assert_eq!(bool::from_child_text("n", Some("false")), Ok(false));
        assert_eq!(String::from_child_text("n", Some("text")).unwrap(), "text");
        assert_eq!(
            Option::<String>::from_attribute("n", Some("text"))
                .unwrap()
                .as_deref(),
            Some("text")
        );

        assert_eq!(
            Option::<NullFlavor>::from_attribute("n", Some("UNK")),
            Ok(Some(NullFlavor::Unknown))
        );
        assert_eq!(
            Option::<NullFlavor>::from_child_text("n", Some("NA")),
            Ok(Some(NullFlavor::NotApplicable))
        );
    }

    #[test]
    fn nested_implementations_delegate_to_the_model_readers() {
        let element = crate::xml::parse(
            r#"<value classCode="ACT" moodCode="EVN" typeCode="COMP"
                      value="12" unit="mg" mediaType="text/plain">body
                  <low value="1"/><high value="2"/>
               </value>"#,
        )
        .unwrap();

        assert_eq!(
            <crate::vocabulary::Ivl as FromElement>::from_element(&element)
                .unwrap()
                .value
                .as_deref(),
            Some("12")
        );
        assert_eq!(
            <crate::vocabulary::Pq as FromElement>::from_element(&element)
                .unwrap()
                .unit
                .as_deref(),
            Some("mg")
        );
        assert_eq!(
            <crate::vocabulary::Ed as FromElement>::from_element(&element)
                .unwrap()
                .text
                .as_deref()
                .map(str::trim),
            Some("body")
        );
        assert_eq!(
            <crate::rim::Act as FromElement>::from_element(&element)
                .unwrap()
                .class_code,
            "ACT"
        );
        assert_eq!(
            <crate::rim::Entity as FromElement>::from_element(&element)
                .unwrap()
                .class_code,
            "ACT"
        );
        assert_eq!(
            <crate::rim::Role as FromElement>::from_element(&element)
                .unwrap()
                .class_code,
            "ACT"
        );
        assert_eq!(
            <crate::rim::Participation as FromElement>::from_element(&element)
                .unwrap()
                .type_code,
            "COMP"
        );
        assert_eq!(
            <crate::rim::ActRelationship as FromElement>::from_element(&element)
                .unwrap()
                .type_code,
            "COMP"
        );
        assert_eq!(
            <crate::rim::RoleLink as FromElement>::from_element(&element)
                .unwrap()
                .type_code,
            "COMP"
        );
    }
}
