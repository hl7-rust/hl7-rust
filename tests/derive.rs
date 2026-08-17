//! Tests for the macros, run against the real `hl7-v2` crate.
//!
//! A proc macro can only really be tested by compiling code that uses it,
//! so these are integration tests rather than unit tests: each one defines
//! a struct, derives, and checks what the generated code does.

use hl7::v2::{Raw, Version};
use hl7_v2_derive::{FromHl7 as DeriveFromHl7, ToHl7 as DeriveToHl7};

const ADT: &str = "MSH|^~\\&|EPIC|CLINIC|LAB|ACME|20260814080000||ADT^A01|MSG1|P|2.5\r\
                   EVN|A01|20260814075900\r\
                   PID|1||444333222~99887766||EVERYWOMAN^EVE^E||19620320|F\r\
                   PV1|1|I|2000^2012^01\r\
                   ZPD|local";

#[derive(Debug, DeriveFromHl7, DeriveToHl7)]
struct Patient {
    #[hl7("PID-1")]
    sequence: u32,
    #[hl7("PID-3")]
    identifiers: Vec<String>,
    #[hl7("PID-5.1.1")]
    family_name: String,
    #[hl7("PID-5.3")]
    middle_name: Option<String>,
    #[hl7("PID-8")]
    sex: Option<String>,
}

#[test]
fn reads_each_annotated_field_from_its_path() {
    let patient: Patient = hl7::v2::parse(ADT).unwrap().decode().unwrap();
    assert_eq!(patient.sequence, 1);
    assert_eq!(patient.identifiers, ["444333222", "99887766"]);
    assert_eq!(patient.family_name, "EVERYWOMAN");
    assert_eq!(patient.middle_name.as_deref(), Some("E"));
    assert_eq!(patient.sex.as_deref(), Some("F"));
}

#[test]
fn writes_each_annotated_field_back_to_its_path() {
    let patient = Patient {
        sequence: 2,
        identifiers: vec!["A1".to_string(), "A2".to_string()],
        family_name: "SMITH".to_string(),
        middle_name: None,
        sex: Some("M".to_string()),
    };
    let message = hl7::v2::Builder::new(Version::V2_5)
        .message_type("ADT", "A01")
        .control_id("1")
        .timestamp("20260814080000")
        .segment("PID")
        .encode(&patient)
        .build()
        .unwrap();
    assert!(
        message.to_er7().ends_with("PID|2||A1~A2||SMITH|||M"),
        "{}",
        message.to_er7()
    );
    // And what was written reads back as what was written.
    let round_tripped: Patient = message.decode().unwrap();
    assert_eq!(round_tripped.identifiers, ["A1", "A2"]);
    assert_eq!(round_tripped.middle_name, None);
}

#[derive(Debug, DeriveFromHl7)]
struct Admission {
    #[hl7("EVN-1")]
    event: String,
    #[hl7(nested)]
    patient: Patient,
    #[hl7(raw)]
    raw: Raw,
    // No attribute: not read from the message at all.
    processed: bool,
}

#[test]
fn nests_structs_and_keeps_the_raw_message() {
    let admission: Admission = hl7::v2::parse(ADT).unwrap().decode().unwrap();
    assert_eq!(admission.event, "A01");
    assert_eq!(admission.patient.family_name, "EVERYWOMAN");
    assert!(!admission.processed, "an unannotated field is defaulted");
    // The vendor's own segment, which no struct here models.
    assert_eq!(
        admission.raw.get("ZPD-1").unwrap().as_deref(),
        Some("local")
    );
    assert_eq!(admission.raw.message().version(), Version::V2_5);
}

#[derive(Debug, DeriveFromHl7)]
struct Required {
    #[hl7("PID-99")]
    #[allow(dead_code, reason = "the point of the test is that reading it fails")]
    absent: String,
}

#[test]
fn a_missing_required_field_names_its_path() {
    let error = hl7::v2::parse(ADT)
        .unwrap()
        .decode::<Required>()
        .unwrap_err();
    assert!(
        matches!(&error, hl7::v2::Error::MissingField { path } if path == "PID-99"),
        "{error}"
    );
}

#[derive(Debug, DeriveFromHl7, DeriveToHl7)]
struct Generic<T>
where
    T: hl7::v2::FromHl7Value + hl7::v2::ToHl7Value,
{
    #[hl7("PID-1")]
    value: T,
}

#[test]
fn generic_structs_carry_their_bounds_through() {
    let typed: Generic<u32> = hl7::v2::parse(ADT).unwrap().decode().unwrap();
    assert_eq!(typed.value, 1);
    let text: Generic<String> = hl7::v2::parse(ADT).unwrap().decode().unwrap();
    assert_eq!(text.value, "1");
}
