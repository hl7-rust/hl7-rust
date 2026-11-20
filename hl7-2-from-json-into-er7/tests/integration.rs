use hl7_2_from_json_into_er7::{
    Hl7Error, convert, convert_with_options, er7::RenderOptions, er7::Terminator,
};
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_cli(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_hl7-2-from-json-into-er7"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn cli_contract_covers_success_and_argument_errors() {
    let json = r#"{"X":{"MSH":{"MSH.1":"|","MSH.2":"^~\\&"},"PID":{"PID.1":"1"}}}"#;
    let output = run_cli(
        &["--terminator", "crlf", "--trailing-terminator", "-"],
        json,
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "MSH|^~\\&\r\nPID|1\r\n"
    );

    for flag in ["--help", "--version"] {
        assert!(run_cli(&[flag], "").status.success());
    }
    assert!(run_cli(&["--terminator", "lf"], json).status.success());
    assert!(run_cli(&["--terminator", "cr"], json).status.success());
    for args in [
        &["--unknown"][..],
        &["--output"][..],
        &["--terminator"][..],
        &["--terminator", "other"][..],
        &["first", "second"][..],
        &["first", "-"][..],
    ] {
        let output = run_cli(args, "");
        assert!(!output.status.success());
        assert!(String::from_utf8(output.stderr).unwrap().contains("error:"));
    }
    assert!(!run_cli(&[], "not json").status.success());

    let directory =
        std::env::temp_dir().join(format!("hl7-json-er7-cli-errors-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    assert!(
        !run_cli(&["--output", directory.to_str().unwrap()], json)
            .status
            .success()
    );
    let output = Command::new(env!("CARGO_BIN_EXE_hl7-2-from-json-into-er7"))
        .stdin(Stdio::from(std::fs::File::open(&directory).unwrap()))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_hl7-2-from-json-into-er7"))
            .arg(directory.join("missing.json"))
            .output()
            .unwrap()
            .status
            .success()
    );

    let large_input = directory.join("large.json");
    std::fs::write(
        &large_input,
        format!(
            r#"{{"X":{{"MSH":{{"MSH.1":"|","MSH.2":"^~\\&"}},"OBX":{{"OBX.5":"{}"}}}}}}"#,
            "x".repeat(128 * 1024)
        ),
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_hl7-2-from-json-into-er7"))
        .arg(&large_input)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    assert!(!child.wait().unwrap().success());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cli_reads_and_writes_named_files() {
    let directory = std::env::temp_dir().join(format!("hl7-json-er7-cli-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let input = directory.join("input.json");
    let output = directory.join("output.hl7");
    std::fs::write(
        &input,
        r#"{"X":{"MSH":{"MSH.1":"|","MSH.2":"^~\\&"},"PID":{"PID.1":"1"}}}"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_hl7-2-from-json-into-er7"))
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "MSH|^~\\&\rPID|1"
    );
    std::fs::remove_dir_all(directory).unwrap();
}

fn convert_observation(value: &str) -> Result<String, Hl7Error> {
    let document = r#"{"X":{"MSH":{"MSH.1":"|","MSH.2":"^~\\&"},"OBX":{"OBX.5":VALUE}}}"#;
    convert(&document.replace("VALUE", value))
}

#[test]
fn literal_and_escaped_multilingual_text_render_identically() {
    let expected = "MSH|^~\\&\rOBX|||||東京 / مختبر / e\u{301} / 𠮷 / �";
    for value in [
        r#""東京 / مختبر / é / 𠮷 / �""#,
        r#""\u6771\u4eac / مختبر / e\u0301 / \ud842\udfb7 / \uFFFD""#,
    ] {
        assert_eq!(convert_observation(value).unwrap(), expected);
    }
}

#[test]
fn strings_that_cannot_be_decoded_return_a_json_error() {
    for value in [
        r#""\ud800""#,
        r#""\udfff""#,
        r#""\ud800\u0041""#,
        r#""\u+041""#,
        r#""\u123""#,
        "\"a\nb\"",
        "\"a\0b\"",
    ] {
        assert!(
            matches!(convert_observation(value), Err(Hl7Error::Json(_))),
            "{value:?}"
        );
    }
}

#[test]
fn numeric_fields_keep_the_exact_json_number_spelling() {
    for value in [
        "0",
        "-0",
        "12",
        "-0.0100",
        "1E+002",
        "12345678901234567890e-999",
    ] {
        assert_eq!(
            convert_observation(value).unwrap(),
            format!("MSH|^~\\&\rOBX|||||{value}")
        );
    }
    for value in [
        "-", "00", "-01", "1.", "1e", "1e+", "3.4.5", "1+2", "+2", ".5",
    ] {
        assert!(
            matches!(convert_observation(value), Err(Hl7Error::Json(_))),
            "{value}"
        );
    }
}

/// The exact converted-JSON document the sibling `hl7-2-from-er7-into-json`
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
    let message = hl7_2_from_json_into_er7::parse(json).unwrap();
    assert_eq!(message.control_id().as_deref(), Some("14AAACVDD"));
    assert_eq!(
        message.query("PID-5.1").unwrap().as_deref(),
        Some("MEDIANO")
    );
}
