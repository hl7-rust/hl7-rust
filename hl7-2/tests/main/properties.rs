//! Property tests: rules that must hold for every input, not just the
//! examples the other tests name (spec §13). Each property states the rule
//! it checks; proptest generates the cases and shrinks any failure to a
//! minimal one.

use hl7_2::Message;
use proptest::prelude::*;

/// A header and one PID, the smallest message the properties write into.
const BASE: &str = "MSH|^~\\&|APP|FAC|EHR|CLINIC|20260101120000||ADT^A01^ADT_A01|1|P|2.5\rPID|1";

/// Text a sender might put in a field: letters, digits, spaces, non-ASCII,
/// and every ER7 delimiter and the escape character, which `set` must
/// escape and `get` must restore.
///
/// It includes whitespace beyond ASCII (tab, no-break space, ideographic
/// space), because §2.1 trims all of it from the end of a segment.
fn field_text() -> impl Strategy<Value = String> {
    let alphabet = vec![
        'a', 'Z', '0', '9', ' ', '.', '-', 'é', '東', '|', '^', '~', '\\', '&', '\t', '\u{a0}',
        '\u{3000}',
    ];
    proptest::collection::vec(proptest::sample::select(alphabet), 1..24)
        .prop_map(|chars| chars.into_iter().collect::<String>())
        .prop_filter("ends in whitespace (see above)", |text| {
            !text.ends_with(' ')
        })
}

/// §2.1: parsing trims all Unicode whitespace from both ends of a line, so
/// trailing whitespace in a segment's last value does not survive a round
/// trip, while the same whitespace mid-segment does.
#[test]
fn line_trimming_drops_unicode_whitespace_at_a_segment_end() {
    let mut message = hl7_2::parse(BASE).unwrap();
    message.set("PID-3", "SMITH \t\u{a0}\u{3000}").unwrap();
    assert_eq!(
        message.get("PID-3").unwrap().as_deref(),
        Some("SMITH \t\u{a0}\u{3000}")
    );
    let reparsed = hl7_2::parse(&message.to_er7()).unwrap();
    assert_eq!(reparsed.get("PID-3").unwrap().as_deref(), Some("SMITH"));
    message.set("PID-5", "EVE").unwrap();
    let reparsed = hl7_2::parse(&message.to_er7()).unwrap();
    assert_eq!(
        reparsed.get("PID-3").unwrap().as_deref(),
        Some("SMITH \t\u{a0}\u{3000}")
    );
}

/// A PID position that `set` can create: a field and a component.
fn position() -> impl Strategy<Value = String> {
    (2_usize..=30, 1_usize..=5).prop_map(|(field, component)| format!("PID-{field}.{component}"))
}

proptest! {
    /// §7.1 and §2.1: any text written with `set` reads back unchanged with
    /// `get`; after the message goes through ER7 and back it is the same
    /// text, or, when it ended its segment, the same text with its trailing
    /// whitespace trimmed.
    #[test]
    fn set_then_get_returns_the_text_written(path in position(), text in field_text()) {
        let mut message = hl7_2::parse(BASE).unwrap();
        message.set(&path, &text).unwrap();
        prop_assert_eq!(message.get(&path).unwrap(), Some(text.clone()));
        let reparsed = hl7_2::parse(&message.to_er7()).unwrap().get(&path).unwrap();
        let trimmed = text.trim_end();
        prop_assert!(
            reparsed.as_deref() == Some(text.as_str())
                || reparsed.as_deref() == Some(trimmed)
                || (trimmed.is_empty() && reparsed.is_none()),
            "{reparsed:?} is neither {text:?} nor {trimmed:?}"
        );
    }

    /// §2 and §7: once a built message has been through ER7 and back (which
    /// applies §2.1's trimming), further round trips change nothing.
    #[test]
    fn rendered_er7_is_a_fixed_point(
        writes in proptest::collection::vec((position(), field_text()), 0..8)
    ) {
        let mut message = hl7_2::parse(BASE).unwrap();
        for (path, text) in &writes {
            message.set(path, text).unwrap();
        }
        let once = hl7_2::parse(&message.to_er7()).unwrap().to_er7();
        let twice = hl7_2::parse(&once).unwrap().to_er7();
        prop_assert_eq!(once, twice);
    }

    /// §2.2 and §8: reading never panics. Arbitrary text is an `Ok` or an
    /// `Err`, and a message that parses can always be validated and shown
    /// as a tree.
    #[test]
    fn reading_arbitrary_text_never_panics(
        tail in "[\\x20-\\x7e\\r|^~&\\\\]{0,200}",
        header in any::<bool>(),
    ) {
        let text = if header {
            format!("MSH|^~\\&|A|B|C|D|20260101||ADT^A01|1|P|2.5\r{tail}")
        } else {
            tail
        };
        if let Ok(message) = hl7_2::parse(&text) {
            let _ = message.validate();
            let _ = message.tree();
            let _ = Message::to_er7(&message);
        }
    }

    /// §8.0: a diagnostic's redacted form never contains message text, even
    /// when every value is a marker that validation will complain about.
    #[test]
    fn redacted_diagnostics_never_contain_message_text(
        fields in proptest::collection::vec(2_usize..=30, 1..6)
    ) {
        let mut message = hl7_2::parse(BASE).unwrap();
        message.append_segment("OBX");
        message.set("OBX-2", "NM").unwrap();
        message.set("OBX-5", "MARKERVALUE").unwrap();
        for field in fields {
            message.set(&format!("PID-{field}"), "MARKERVALUE").unwrap();
        }
        for diagnostic in message.validate() {
            prop_assert!(!diagnostic.redacted().contains("MARKER"), "{}", diagnostic.redacted());
        }
    }
}
