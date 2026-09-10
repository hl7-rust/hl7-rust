//! [`Element`]: one XML element, serialized as an object.

use std::collections::BTreeMap;

object_wrapper! {
    /// A Serde-enabled [`hl7_3::xml::Element`]: the raw XML tree `hl7-3`
    /// reads through, and keeps for the parts of a message it does not
    /// model — [`hl7_3::Message::sender`], [`hl7_3::Message::receiver`],
    /// and the domain payload under [`hl7_3::ControlAct::domain`].
    ///
    /// Serializes as an object with four keys: `"name"` (the tag as
    /// written, prefix included), `"attributes"` (an object, keyed by
    /// attribute name, in name order), `"text"` (entity-decoded, `""` when
    /// none), and `"children"` (an array of elements, `[]` at a leaf).
    /// Only `"name"` is required on the way back in; the other three
    /// default to empty, so a hand-written fixture can say `{"name":
    /// "device"}` and mean exactly that.
    ///
    /// Example:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_hl7_v3::Element;
    ///
    /// let element = Element(hl7_3::xml::parse(r#"<id root="1.2.3" extension="7"/>"#)?);
    /// let json = serde_json::to_value(&element)?;
    /// assert_eq!(json["name"], "id");
    /// assert_eq!(json["attributes"]["root"], "1.2.3");
    /// assert_eq!(json["children"], serde_json::json!([]));
    ///
    /// let back: Element = serde_json::from_value(json)?;
    /// assert_eq!(back, element);
    /// assert_eq!(back.attribute("extension"), Some("7"));
    /// # Ok(())
    /// # }
    /// ```
    Element wraps hl7_3::xml::Element,
    expecting "an Element object with \"name\" and, optionally, \"attributes\", \"text\", and \"children\"",
    derive [Eq, Default],
    fields {
        name: req String => "name",
        attributes: dflt BTreeMap<String, String> => "attributes",
        text: dflt String => "text",
        children: vecw Element => "children",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Strict;

    const SAMPLE: &str = r#"<a x="1" y="2">hello<b/><c z="3">world</c></a>"#;

    fn sample() -> Element {
        Element(hl7_3::xml::parse(SAMPLE).unwrap())
    }

    #[test]
    fn round_trips_a_tree() {
        let json = serde_json::to_string(&sample()).unwrap();
        let back: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sample());
    }

    #[test]
    fn serializes_four_keys_in_order() {
        let json = serde_json::to_string(&sample()).unwrap();
        assert!(
            json.starts_with(
                r#"{"name":"a","attributes":{"x":"1","y":"2"},"text":"hello","children":["#
            ),
            "{json}"
        );
    }

    #[test]
    fn only_name_is_required() {
        let back: Element = serde_json::from_str(r#"{"name":"device"}"#).unwrap();
        assert_eq!(back.name, "device");
        assert!(back.attributes.is_empty());
        assert_eq!(back.text, "");
        assert!(back.children.is_empty());
        let err = serde_json::from_str::<Element>(r#"{"text":"x"}"#).unwrap_err();
        assert!(err.to_string().contains("name"), "{err}");
    }

    #[test]
    fn ignores_unknown_fields() {
        let back: Element = serde_json::from_str(r#"{"name":"a","extra":1}"#).unwrap();
        assert_eq!(back.name, "a");
    }

    #[test]
    fn strict_rejects_an_unknown_field_at_any_depth() {
        let top = r#"{"name":"a","extra":1}"#;
        let err = serde_json::from_str::<Strict<Element>>(top).unwrap_err();
        assert!(err.to_string().contains("extra"), "{err}");

        let nested = r#"{"name":"a","children":[{"name":"b","children":[{"name":"c","typo":1}]}]}"#;
        assert!(serde_json::from_str::<Element>(nested).is_ok());
        let err = serde_json::from_str::<Strict<Element>>(nested).unwrap_err();
        assert!(err.to_string().contains("typo"), "{err}");
    }

    #[test]
    fn deref_reaches_the_inner_api() {
        let element = sample();
        assert_eq!(element.attribute("x"), Some("1"));
        assert_eq!(element.child("c").unwrap().text, "world");
    }
}
