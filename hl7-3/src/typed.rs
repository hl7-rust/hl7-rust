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
//! let observation = Observation::from_element(&element);
//! assert_eq!(observation.class_code, "OBS");
//! assert_eq!(observation.component.mood_code, "RQO");
//! # }
//! # #[cfg(not(feature = "derive"))] fn main() {}
//! ```
//!
//! ## Total, on purpose — unlike `hl7-2`'s struct mode
//!
//! `hl7-2`'s [`FromHl7`](https://docs.rs/hl7-2/latest/hl7_2/trait.FromHl7.html)
//! returns a `Result`: a required field absent from an HL7 v2 message is an
//! error, because v2's dictionary says what a message *should* carry.
//! [`FromElement`] returns `Self` directly — no `Result`, because that
//! matches how [`crate::rim`] and [`crate::message`] already read: a
//! missing attribute or child degrades to a default, never a failure. A
//! struct mapped onto the wrong element just reads defaults everywhere,
//! the same way `Act::from_element` on an unrelated element reads empty
//! `class_code`/`mood_code` rather than panicking.

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
    /// Read `element` into `Self`. Never fails — see the module
    /// documentation for why there is no `Result` here.
    fn from_element(element: &Element) -> Self;
}

/// A single field's value, read from an attribute or a child element's
/// text — the two atoms [`FromElement`] fields are built from.
///
/// Implemented for [`String`], [`bool`], and the integer and
/// floating-point types. A value that is absent, or present but not this
/// type's shape, reads as the type's `Default` — silently, the same
/// "degrade, don't reject" choice [`crate::rim`] makes for `classCode` and
/// friends. Prefer `String` and parse deliberately (with your own error
/// handling) where silent defaulting on bad input would hide something
/// you need to know about.
pub trait FromElementValue: Sized {
    /// Read from an attribute's value, or `None` when the attribute is
    /// absent.
    fn from_attribute(value: Option<&str>) -> Self;
    /// Read from a child element's text, or `None` when the child is
    /// absent or has no text.
    fn from_child_text(text: Option<&str>) -> Self;
}

impl FromElementValue for String {
    fn from_attribute(value: Option<&str>) -> String {
        value.unwrap_or_default().to_string()
    }

    fn from_child_text(text: Option<&str>) -> String {
        text.unwrap_or_default().to_string()
    }
}

impl FromElementValue for Option<String> {
    fn from_attribute(value: Option<&str>) -> Option<String> {
        value.map(str::to_string)
    }

    fn from_child_text(text: Option<&str>) -> Option<String> {
        text.map(str::to_string)
    }
}

/// Accepts the same spellings [`crate::rim::ActRelationship::inversion_ind`]
/// does implicitly (`"true"`); anything else, including absent, is `false`.
impl FromElementValue for bool {
    fn from_attribute(value: Option<&str>) -> bool {
        value == Some("true")
    }

    fn from_child_text(text: Option<&str>) -> bool {
        text == Some("true")
    }
}

/// Numbers parse with their own `FromStr`; absent or unparseable is `0`,
/// per this module's "total" rule.
macro_rules! numbers {
    ($($type:ty),*) => {$(
        impl FromElementValue for $type {
            fn from_attribute(value: Option<&str>) -> $type {
                value.and_then(|v| v.trim().parse().ok()).unwrap_or_default()
            }

            fn from_child_text(text: Option<&str>) -> $type {
                text.and_then(|v| v.trim().parse().ok()).unwrap_or_default()
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
    fn from_attribute(value: Option<&str>) -> Option<crate::vocabulary::NullFlavor> {
        value.map(crate::vocabulary::NullFlavor::parse)
    }

    fn from_child_text(text: Option<&str>) -> Option<crate::vocabulary::NullFlavor> {
        text.map(crate::vocabulary::NullFlavor::parse)
    }
}

impl FromElement for crate::vocabulary::Ivl {
    fn from_element(element: &Element) -> Self {
        crate::vocabulary::Ivl::from_element(element)
    }
}

impl FromElement for crate::vocabulary::Pq {
    fn from_element(element: &Element) -> Self {
        crate::vocabulary::Pq::from_element(element)
    }
}

impl FromElement for crate::vocabulary::Ed {
    fn from_element(element: &Element) -> Self {
        crate::vocabulary::Ed::from_element(element)
    }
}

impl FromElement for crate::rim::Act {
    fn from_element(element: &Element) -> Self {
        crate::rim::Act::from_element(element)
    }
}

impl FromElement for crate::rim::Entity {
    fn from_element(element: &Element) -> Self {
        crate::rim::Entity::from_element(element)
    }
}

impl FromElement for crate::rim::Role {
    fn from_element(element: &Element) -> Self {
        crate::rim::Role::from_element(element)
    }
}

impl FromElement for crate::rim::Participation {
    fn from_element(element: &Element) -> Self {
        crate::rim::Participation::from_element(element)
    }
}

impl FromElement for crate::rim::ActRelationship {
    fn from_element(element: &Element) -> Self {
        crate::rim::ActRelationship::from_element(element)
    }
}

impl FromElement for crate::rim::RoleLink {
    fn from_element(element: &Element) -> Self {
        crate::rim::RoleLink::from_element(element)
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
        fn from_element(element: &Element) -> Observation {
            Observation {
                class_code: FromElementValue::from_attribute(element.attribute("classCode")),
                mood_code: FromElementValue::from_attribute(element.attribute("moodCode")),
                note: FromElementValue::from_child_text(
                    element.child("note").and_then(Element::text_opt),
                ),
                component: element
                    .child("component")
                    .map(Act::from_element)
                    .unwrap_or_default(),
                raw: element.clone(),
            }
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
        let observation = Observation::from_element(&element);
        assert_eq!(observation.class_code, "OBS");
        assert_eq!(observation.mood_code, "EVN");
        assert_eq!(observation.note.as_deref(), Some("elevated"));
        assert_eq!(observation.component.class_code, "ACT");
        assert_eq!(observation.raw.local_name(), "observation");
    }

    #[test]
    fn missing_attributes_and_children_degrade_to_defaults() {
        let element = crate::xml::parse(r"<observation/>").unwrap();
        let observation = Observation::from_element(&element);
        assert_eq!(observation.class_code, "");
        assert_eq!(observation.note, None);
        assert_eq!(observation.component, Act::default());
    }

    #[test]
    fn numbers_and_bool_default_silently_on_bad_input() {
        assert_eq!(u32::from_attribute(Some("7")), 7);
        assert_eq!(u32::from_attribute(Some("not a number")), 0);
        assert_eq!(u32::from_attribute(None), 0);
        assert!(bool::from_attribute(Some("true")));
        assert!(!bool::from_attribute(Some("yes")));
        assert!(!bool::from_attribute(None));
    }
}
