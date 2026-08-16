use json_to_hl7_2_5::{convert, convert_with_options, er7::RenderOptions, er7::Terminator};

/// The exact converted-JSON document the sibling `hl7-2-5-to-json-using-rust`
/// crate's own golden test produces from
/// `MSH|^~\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\r\
/// PID|1||241900||MEDIANO^FOUAZ\rORC|NW|ORD1\rOBR|1|ORD1||24331-1^Lipid Panel^LN`
/// — converting it back reproduces that message exactly (`samples/orm_o01.json`).
#[test]
fn golden_orm_o01_round_trips() {
    let json = include_str!("../samples/orm_o01.json");
    let er7 = convert(json).unwrap();
    let expected = "MSH|^~\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\r\
                    PID|1||241900||MEDIANO^FOUAZ\r\
                    ORC|NW|ORD1\r\
                    OBR|1|ORD1||24331-1^Lipid Panel^LN";
    assert_eq!(er7, expected);
}

/// `samples/oru_r01.json` nests three levels of message-structure groups
/// and repeats both a group (`ORU_R01.OBSERVATION`) and a field
/// (`PID.3`); all of that must flatten and un-array back to the original
/// segment sequence.
#[test]
fn oru_r01_groups_and_arrays_flatten_to_the_original_segment_sequence() {
    let json = include_str!("../samples/oru_r01.json");
    let er7 = convert(json).unwrap();
    assert!(
        er7.starts_with("MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5\r")
    );
    assert!(er7.contains("\rPID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E||19620320|F"));
    assert!(er7.contains("\rPV1|1|O|OP^^^ACME\r"));
    assert!(er7.contains("\rORC|RE|ORD776655\r"));
    assert!(er7.contains("\rOBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F\r"));
    assert!(er7.contains("\rOBX|2|CE|10331-7^Rh Type^LN||D^Rh positive^LN|||N|||F\r"));
    assert!(er7.ends_with("\rNTE|1||Fasting sample."));
    // No trace of the group wrappers or array shape survives.
    assert!(!er7.contains("ORU_R01"));
}

/// `samples/misc.json` exercises field repetitions (a JSON array), the
/// explicit HL7 null (JSON `null`), decoded delimiter characters that must
/// be re-escaped, a formatting escape sequence that must be left exactly
/// as it was (the forward crate never decodes it), and an unrecognized
/// (Z-)segment using the generic positional names.
#[test]
fn misc_features_round_trip() {
    let json = include_str!("../samples/misc.json");
    let er7 = convert(json).unwrap();
    assert!(er7.contains("\rPID|1||111~222||\"\"\r"));
    assert!(er7.contains(r"NTE|1||A\T\B \F\ C\S\D\.br\next"));
    assert!(er7.ends_with(r"ZDS|1.2.840^app^DICOM"));
}

/// `convert_with_options` chooses the segment terminator and whether the
/// last segment gets one.
#[test]
fn render_options_choose_the_terminator() {
    let json = r#"{"X": {"MSH": {"MSH.1": "|", "MSH.2": "^~\\&"}, "PID": {"PID.1": "1"}}}"#;
    let options = RenderOptions {
        terminator: Terminator::CrLf,
        trailing_terminator: true,
    };
    let er7 = convert_with_options(json, options).unwrap();
    assert_eq!(er7, "MSH|^~\\&\r\nPID|1\r\n");
}

/// `parse` hands back the full `er7::Message`, not just its rendered text,
/// so a caller can query or edit it with `er7`'s own API.
#[test]
fn parse_returns_a_queryable_message() {
    let json = include_str!("../samples/orm_o01.json");
    let message = json_to_hl7_2_5::parse(json).unwrap();
    assert_eq!(message.control_id().as_deref(), Some("14AAACVDD"));
    assert_eq!(
        message.query("PID-5.1").unwrap().as_deref(),
        Some("MEDIANO")
    );
}
