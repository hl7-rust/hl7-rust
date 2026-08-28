//! Black-box tests through the public API and the command line.
//!
//! Unit tests next to each module cover that module's rules; these cover
//! what crosses module boundaries — the three modes over one message, the
//! version machinery over real dictionaries, round trips, and the CLI
//! contract.

use hl7_2::{Builder, Dictionary, Options, Severity, Version};
use std::sync::Arc;

/// A lab result message: two results, a specimen, and a note.
const ORU: &str = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5\r\
                   PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E||19620320|F\r\
                   PV1|1|O|OP^^^ACME\r\
                   ORC|RE|ORD776655\r\
                   OBR|1|ORD776655|LAB2233|24331-1^Lipid Panel^LN|||20260813071500\r\
                   OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F\r\
                   OBX|2|CE|10331-7^Rh Type^LN||D^Rh positive^LN|||N|||F\r\
                   NTE|1||Fasting sample.";

#[test]
fn generic_mode_reads_a_message_nobody_described_in_advance() {
    let message = hl7_2::parse(ORU).unwrap();
    let tree = message.tree();

    assert_eq!(tree.name(), "ORU_R01");
    assert_eq!(message.version(), Version::V2_5);

    // Named by data type where the dictionary knows one ...
    assert_eq!(tree.find("XPN.1").unwrap().text(), "EVERYWOMAN");
    // ... and reachable by path from anywhere in the tree.
    let second_result = tree.find_all("OBX").nth(1).unwrap();
    assert_eq!(
        second_result.child("OBX.5").unwrap().text(),
        "D^Rh positive^LN"
    );
    assert_eq!(second_result.path(), "OBX[2]");

    // Grouping puts each OBX under its OBR.
    let result = tree.child("ORU_R01.PATIENT_RESULT").unwrap();
    let order = result.child("ORU_R01.ORDER_OBSERVATION").unwrap();
    assert_eq!(order.find_all("OBX").count(), 2);
    // And the flat reading is still one call away.
    assert_eq!(message.tree_with_options(false).find_all("OBX").count(), 2);
}

#[test]
fn schema_mode_teaches_the_parser_one_vendors_dialect() {
    let text = "MSH|^~\\&|ACME||||20260814080000||ADT^A01|1|P|2.5\r\
                EVN|A01|20260814075900\r\
                PID|1||9\r\
                PV1|1|I\r\
                ZAC|7|SMITH^JOHN|20260814";

    // Without a schema the vendor's segment is still readable, positionally.
    let message = hl7_2::parse(text).unwrap();
    let zac = message.tree().find("ZAC").unwrap().clone();
    assert_eq!(
        zac.child("ZAC.2").unwrap().child("ZAC.2.1").unwrap().text(),
        "SMITH"
    );
    assert_eq!(message.type_of("ZAC-2").unwrap(), None);

    // With one, it reads like any standard segment — and the schema is
    // JSON loaded at runtime, so adding a field is a config change.
    let dictionary = Dictionary::from_json(
        r#"{
            "inherits": "2.5",
            "segments": { "ZAC": ["SI", "XPN", "DT"] },
            "structures": {
                "ADT_A01": [
                    {"segment": "MSH", "required": true},
                    {"segment": "EVN", "required": true},
                    {"segment": "PID", "required": true},
                    {"segment": "PV1", "required": true},
                    {"segment": "ZAC", "repeats": true}
                ]
            }
        }"#,
        "acme",
    )
    .unwrap();
    let options = Options::new().with_dictionary(Arc::new(dictionary));
    let message = hl7_2::parse_with_options(text, &options).unwrap();

    assert_eq!(message.type_of("ZAC-2").unwrap().as_deref(), Some("XPN"));
    assert_eq!(message.tree().find("XPN.2").unwrap().text(), "JOHN");
    // The vendor's own structure now validates, Z-segment and all.
    assert_eq!(message.validate(), []);
}

#[test]
fn struct_mode_keeps_the_generic_escape_hatch_on_the_same_object() {
    // The shape `#[derive(FromHl7)]` writes; see the derive crate's tests
    // for the macro itself.
    struct Result {
        code: String,
        value: Option<String>,
        raw: hl7_2::Raw,
    }

    impl hl7_2::FromHl7 for Result {
        fn from_hl7(message: &hl7_2::Message) -> std::result::Result<Result, hl7_2::Error> {
            Ok(Result {
                code: hl7_2::FromHl7Value::from_hl7_value(message, "OBX-3.1")?,
                value: hl7_2::FromHl7Value::from_hl7_value(message, "OBX-5")?,
                raw: hl7_2::Raw::new(message.clone()),
            })
        }
    }

    let result: Result = hl7_2::parse(ORU).unwrap().decode().unwrap();
    assert_eq!(result.code, "2093-3");
    assert_eq!(result.value.as_deref(), Some("187"));
    // The field the struct does not model, without a second parse.
    assert_eq!(
        result.raw.get("OBX[2]-5.2").unwrap().as_deref(),
        Some("Rh positive")
    );
    assert_eq!(result.raw.tree().find_all("OBX").count(), 2);
}

#[test]
fn every_bundled_release_reads_a_message_that_declares_it() {
    for &version in hl7_2::version::ALL {
        let text = format!("MSH|^~\\&|A||||20260814||ACK^A01|1|P|{version}\rMSA|AA|1");
        let message = hl7_2::parse(&text).unwrap();
        assert_eq!(message.version(), version, "MSH-12 said {version}");
        assert_eq!(message.structure_id(), "ACK");
        assert_eq!(message.get("MSA-1").unwrap().as_deref(), Some("AA"));
        assert_eq!(message.validate(), [], "v{version} rejected a plain ACK");
    }
}

#[test]
fn a_release_difference_changes_how_a_field_reads() {
    // MSH-12 is a plain ID before v2.3.1 and the VID composite after, so
    // the same header reads differently under each.
    let text = "MSH|^~\\&|A||||20260814||ACK^A01|1|P|2.5\rMSA|AA|1";
    let modern = hl7_2::parse(text).unwrap();
    assert_eq!(modern.type_of("MSH-12").unwrap().as_deref(), Some("VID"));

    let options = Options::new().with_version(Version::V2_3);
    let old = hl7_2::parse_with_options(text, &options).unwrap();
    assert_eq!(old.type_of("MSH-12").unwrap().as_deref(), Some("ID"));
    // And v2.3 has no SFT segment at all, where v2.5 does.
    assert!(modern.dictionary().segment_fields("SFT").is_some());
    assert!(old.dictionary().segment_fields("SFT").is_none());
}

#[test]
fn reading_modifying_and_writing_round_trips() {
    let message = hl7_2::parse(ORU).unwrap();
    assert_eq!(message.to_er7(), ORU, "an untouched message is unchanged");

    let mut message = message;
    message.set("PID-5.2", "EVELYN").unwrap();
    message.append_segment("NTE");
    message.set("NTE[2]-3", "Amended.").unwrap();
    let written = message.to_er7();

    let reread = hl7_2::parse(&written).unwrap();
    assert_eq!(reread.get("PID-5.2").unwrap().as_deref(), Some("EVELYN"));
    assert_eq!(reread.get_all("NTE-3").unwrap().len(), 2);
    assert_eq!(reread.to_er7(), written, "and the change round-trips too");
}

#[test]
fn escaped_text_survives_the_whole_trip() {
    let mut message = hl7_2::parse(ORU).unwrap();
    // Every delimiter at once, in one value.
    let awkward = "a|b^c~d\\e&f";
    message.set("NTE-3", awkward).unwrap();
    let written = message.to_er7();
    assert!(
        !written.contains("a|b"),
        "delimiters must be escaped on the wire"
    );
    let reread = hl7_2::parse(&written).unwrap();
    assert_eq!(reread.get("NTE-3").unwrap().as_deref(), Some(awkward));
}

#[test]
fn building_a_reply_to_a_message() {
    let message = hl7_2::parse(ORU).unwrap();
    let ack = hl7_2::builder::acknowledge(&message, "AA", "ACK00001", "20260814080100")
        .build_valid()
        .unwrap();
    assert_eq!(ack.structure_id(), "ACK");
    assert_eq!(ack.get("MSA-2").unwrap().as_deref(), Some("MSG00042"));
    assert_eq!(ack.get("MSH-5.1").unwrap().as_deref(), Some("LAB"));
    assert_eq!(
        ack.to_er7(),
        "MSH|^~\\&|EHR|CLINIC|LAB|ACME|20260814080100||ACK^R01^ACK|ACK00001|P|2.5\r\
         MSA|AA|MSG00042"
    );
}

#[test]
fn strict_mode_is_the_difference_between_reporting_and_refusing() {
    // MSA-4 is an NM; "many" is not a number.
    let text = "MSH|^~\\&|A||||20260814||ACK^A01|1|P|2.5\rMSA|AA|1||many";

    let lenient = hl7_2::parse(text).unwrap();
    let diagnostics = lenient.validate();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Error);
    // ... and the value is still readable, because reporting is not refusing.
    assert_eq!(lenient.get("MSA-4").unwrap().as_deref(), Some("many"));

    let strict = hl7_2::parse_with_options(text, &Options::new().strict());
    assert!(matches!(strict, Err(hl7_2::Error::Invalid(_))));
}

#[test]
fn a_batch_file_becomes_one_message_each() {
    let batch = std::fs::read_to_string("samples/batch.hl7").unwrap();
    let messages = hl7_2::split_messages(&batch);
    assert_eq!(messages.len(), 2);
    let control_ids: Vec<String> = messages
        .iter()
        .map(|text| hl7_2::parse(text).unwrap().get("MSH-10").unwrap().unwrap())
        .collect();
    assert_eq!(control_ids, ["MSG00001", "MSG00002"]);
}

#[test]
fn the_samples_parse_and_report_what_they_should() {
    for name in ["orm_o01", "oru_r01", "adt_a01"] {
        let text = std::fs::read_to_string(format!("samples/{name}.hl7")).unwrap();
        let message = hl7_2::parse(&text).unwrap();
        assert_eq!(message.to_er7(), hl7_2::split_messages(&text)[0]);
        // The ADT sample carries a Z-segment on purpose: a local extension
        // must not make an otherwise conformant message fail.
        let diagnostics = message.validate();
        let errors: Vec<&hl7_2::Diagnostic> = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "sample {name}: {errors:?}");
    }
}

#[test]
fn a_builder_makes_a_message_from_nothing_that_parses_back() {
    let message = Builder::new(Version::V2_5)
        .message_type("ORU", "R01")
        .control_id("MSG1")
        .timestamp("20260814080000")
        .sending("LAB", "ACME")
        .receiving("EHR", "CLINIC")
        .segment("PID")
        .set("PID-3.1", "444333222")
        .set("PID-5.1.1", "EVERYWOMAN")
        .segment("OBR")
        .set("OBR-1", "1")
        .segment("OBX")
        .set("OBX-1", "1")
        .set("OBX-2", "NM")
        .set("OBX-5", "187")
        .build_valid()
        .unwrap();

    let text = message.to_er7();
    let reread = hl7_2::parse(&text).unwrap();
    assert_eq!(reread.structure_id(), "ORU_R01");
    assert_eq!(reread.tree().find("XPN.1").unwrap().text(), "EVERYWOMAN");
    assert_eq!(reread.to_er7(), text);
}

// ---- command line -------------------------------------------------------

/// Run the CLI binary with `arguments` and this input, returning
/// (exit status, stdout).
fn cli(arguments: &[&str], input: &str) -> (i32, String) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(env!("CARGO_BIN_EXE_hl7-v2"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary is built before its integration tests run");
    // A CLI given bad usage (the case this helper exists to test) exits
    // before it ever reads stdin, and closes that end of the pipe on the
    // way out. Whether this write lands before or after that close is a
    // race decided by process scheduling, not by anything this test
    // controls — so a BrokenPipe here means "the child didn't want the
    // input," which is a real, exercised outcome, not a test failure.
    // What the test actually asserts on is the exit status and stdout
    // below, which `wait_with_output` reports correctly either way.
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    let output = child.wait_with_output().unwrap();
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8(output.stdout).unwrap(),
    )
}

#[test]
fn the_cli_prints_a_tree_by_default() {
    let (status, out) = cli(&[], ORU);
    assert_eq!(status, 0);
    assert!(out.starts_with("ORU_R01\n"), "{out}");
    assert!(out.contains("      XPN.1 = EVERYWOMAN"), "{out}");
    assert!(out.contains("ORU_R01.ORDER_OBSERVATION"), "{out}");

    let (_, flat) = cli(&["--flat"], ORU);
    assert!(!flat.contains("ORDER_OBSERVATION"), "{flat}");
    assert!(flat.contains("XPN.1 = EVERYWOMAN"), "{flat}");
}

#[test]
fn the_cli_queries_sets_and_checks() {
    let (status, out) = cli(&["-q", "OBX-5"], ORU);
    assert_eq!((status, out.as_str()), (0, "187\nD^Rh positive^LN\n"));

    let (status, out) = cli(&["-s", "PID-8=M", "-e"], ORU);
    assert_eq!(status, 0);
    assert!(out.contains("|19620320|M\r"), "{out}");

    let (status, out) = cli(&["--check"], ORU);
    assert_eq!((status, out.as_str()), (0, "ok\n"));

    // Exit status 2 marks a message that failed its check, so a shell
    // pipeline can act on it.
    let bad = "MSH|^~\\&|A||||20260814||ACK^A01|1|P|2.5";
    let (status, out) = cli(&["--check"], bad);
    assert_eq!(status, 2);
    assert!(
        out.contains("error: MSA: structure ACK requires a MSA segment"),
        "{out}"
    );
}

#[test]
fn the_cli_reads_a_batch_and_a_dictionary() {
    let batch = std::fs::read_to_string("samples/batch.hl7").unwrap();
    let (status, out) = cli(&["-q", "MSH-10"], &batch);
    assert_eq!((status, out.as_str()), (0, "MSG00001\nMSG00002\n"));

    let (status, out) = cli(
        &["-d", "samples/acme.json", "--flat", "-q", "ZAC-2.2"],
        "MSH|^~\\&|ACME||||20260814||ADT^A01|1|P|2.5\rZAC|7|SMITH^JOHN",
    );
    assert_eq!((status, out.as_str()), (0, "JOHN\n"));
}

#[test]
fn the_cli_reports_bad_usage_without_a_stack_trace() {
    let (status, out) = cli(&["--nonsense"], ORU);
    assert_eq!((status, out.as_str()), (1, ""));
    let (status, _) = cli(&["-q", "PID-0"], ORU);
    assert_eq!(status, 1);
    let (status, out) = cli(&["--help"], "");
    assert_eq!(status, 0);
    assert!(out.contains("Usage: hl7-v2"), "{out}");
}
