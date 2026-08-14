use hl7_2_5_to_json::{Options, convert, convert_with_options, split_messages};

/// The spec's ORM^O01 example (with an ORC/OBR order so the message is a
/// complete ORM_O01), checked against the exact expected JSON document,
/// including the `ORM_O01.PATIENT` / `ORM_O01.ORDER` group nesting.
#[test]
fn golden_orm_o01() {
    let er7 = "MSH|^~\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\r\
               PID|1||241900||MEDIANO^FOUAZ\r\
               ORC|NW|ORD1\r\
               OBR|1|ORD1||24331-1^Lipid Panel^LN";
    let expected = r#"{
  "ORM_O01": {
    "MSH": {
      "MSH.1": "|",
      "MSH.2": "^~\\&",
      "MSH.3": {
        "HD.1": "hphis"
      },
      "MSH.5": {
        "HD.1": "EPIC"
      },
      "MSH.7": {
        "TS.1": "20131011093851"
      },
      "MSH.9": {
        "MSG.1": "ORM",
        "MSG.2": "O01"
      },
      "MSH.10": "14AAACVDD",
      "MSH.11": {
        "PT.1": "P"
      },
      "MSH.12": {
        "VID.1": "2.5"
      }
    },
    "ORM_O01.PATIENT": {
      "PID": {
        "PID.1": "1",
        "PID.3": {
          "CX.1": "241900"
        },
        "PID.5": {
          "XPN.1": {
            "FN.1": "MEDIANO"
          },
          "XPN.2": "FOUAZ"
        }
      }
    },
    "ORM_O01.ORDER": {
      "ORC": {
        "ORC.1": "NW",
        "ORC.2": {
          "EI.1": "ORD1"
        }
      },
      "ORM_O01.ORDER_DETAIL": {
        "OBR": {
          "OBR.1": "1",
          "OBR.2": {
            "EI.1": "ORD1"
          },
          "OBR.4": {
            "CE.1": "24331-1",
            "CE.2": "Lipid Panel",
            "CE.3": "LN"
          }
        }
      }
    }
  }
}
"#;
    assert_eq!(convert(er7).unwrap(), expected);
}

/// The spec's PID sample: PID|1||241900||TEST^FOUAZ.
#[test]
fn spec_pid_sample() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rPID|1||241900||TEST^FOUAZ";
    let json = convert(er7).unwrap();
    assert!(json.contains("\"PID.1\": \"1\""));
    assert!(json.contains("\"CX.1\": \"241900\""));
    assert!(json.contains("\"FN.1\": \"TEST\""));
    assert!(json.contains("\"XPN.2\": \"FOUAZ\""));
    assert!(json.contains("\"XXX_X01\": {"));
}

#[test]
fn oru_r01_grouping() {
    let er7 = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5\r\
               PID|1||444333222^^^ACME^MR||EVERYWOMAN^EVE\r\
               PV1|1|O\r\
               ORC|RE|ORD776655\r\
               OBR|1|ORD776655|LAB2233|24331-1^Lipid Panel^LN\r\
               OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F\r\
               OBX|2|CE|10331-7^Rh Type^LN||D^Rh positive^LN|||N|||F\r\
               NTE|1||Fasting sample.";
    let json = convert(er7).unwrap();
    assert!(json.contains("\"ORU_R01\": {"));
    assert!(json.contains("\"ORU_R01.PATIENT_RESULT\": {"));
    assert!(json.contains("\"ORU_R01.PATIENT\": {"));
    assert!(json.contains("\"ORU_R01.VISIT\": {"));
    // Two OBX observations -> one array under ORU_R01.OBSERVATION, and the
    // NTE nests inside the second observation's object, not a sibling array.
    assert!(json.contains("\"ORU_R01.OBSERVATION\": ["));
    assert_eq!(json.matches("\"OBX\": {").count(), 2);
    assert!(json.contains("\"NTE.3\": \"Fasting sample.\""));
    // JSON strings don't need `<`/`>` escaped, unlike XML.
    assert!(json.contains("\"OBX.7\": \"<200\""));
}

/// OBX-5's key names come from the value type declared in OBX-2.
#[test]
fn obx5_type_follows_obx2() {
    let er7 = "MSH|^~\\&|LAB||||20260814||ORU^R01|1|P|2.5\r\
               PID|1\r\
               OBR|1\r\
               OBX|1|CE|X||A^B\r\
               OBX|2|NM|Y||42";
    let json = convert(er7).unwrap();
    assert!(json.contains(
        "\"OBX.5\": {\n                \"CE.1\": \"A\",\n                \"CE.2\": \"B\""
    ));
    assert!(json.contains("\"OBX.5\": \"42\""));
}

/// A message that does not fit its declared structure (here: a Z-segment)
/// renders flat, and unknown segments get positional generic key names.
#[test]
fn falls_back_to_flat_on_structure_mismatch() {
    let er7 = "MSH|^~\\&|APP||||20260814||ORM^O01|1|P|2.5\r\
               PID|1\r\
               ORC|NW\r\
               ZDS|1.2.840^app^DICOM";
    let json = convert(er7).unwrap();
    assert!(!json.contains("ORM_O01.PATIENT"));
    assert!(json.contains("\"ZDS\": {"));
    assert!(json.contains("\"ZDS.1.1\": \"1.2.840\""));
    assert!(json.contains("\"ZDS.1.2\": \"app\""));
}

/// The --flat option disables grouping even for well-formed messages.
#[test]
fn flat_option_disables_grouping() {
    let er7 = "MSH|^~\\&|APP||||20260814||ORM^O01|1|P|2.5\rPID|1\rORC|NW";
    let json = convert_with_options(
        er7,
        Options {
            flat: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!json.contains("ORM_O01.PATIENT"));
    assert!(json.contains("\"PID.1\": \"1\""));
}

/// The --compact option emits single-line JSON with no insignificant
/// whitespace.
#[test]
fn compact_option_emits_single_line() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rPID|1";
    let json = convert_with_options(
        er7,
        Options {
            compact: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(json.lines().count(), 1);
    assert!(json.contains("\"XXX_X01\":{\"MSH\":"));
    assert!(!json.contains(' '));
}

/// Field repetitions (~) collapse into a JSON array under one key.
#[test]
fn repetitions_become_an_array() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rPID|1||111~222";
    let json = convert(er7).unwrap();
    assert!(json.contains("\"PID.3\": ["));
    assert!(json.contains("\"CX.1\": \"111\""));
    assert!(json.contains("\"CX.1\": \"222\""));
}

/// Subcomponents of a composite component use its own type's names,
/// e.g. the assigning authority (HD) inside a CX identifier.
#[test]
fn subcomponents_use_component_type_names() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\r\
               PID|1||123^^^HOSP&1.2.3.4&ISO";
    let json = convert(er7).unwrap();
    assert!(json.contains(
        "\"CX.4\": {\n          \"HD.1\": \"HOSP\",\n          \"HD.2\": \"1.2.3.4\",\n          \
         \"HD.3\": \"ISO\"\n        }"
    ));
}

/// The HL7 explicit null `""` becomes JSON `null`.
#[test]
fn explicit_null_becomes_json_null() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rPID|1|\"\"";
    let json = convert(er7).unwrap();
    assert!(json.contains("\"PID.2\": null"));
}

/// Escape sequences decode to delimiter characters; JSON needs no escaping
/// for `&`, `^`, `~` (unlike XML, which must escape `&`).
#[test]
fn er7_escapes_decode() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rNTE|1||A\\T\\B \\F\\ C\\S\\D";
    let json = convert(er7).unwrap();
    assert!(json.contains("\"NTE.3\": \"A&B | C^D\""));
}

/// MSH-9.3, when present, wins as the message structure / root key.
#[test]
fn msh9_3_names_the_root() {
    let er7 = "MSH|^~\\&|APP||||20260814||ADT^A08^ADT_A01|1|P|2.5\r\
               EVN|A08|20260814\r\
               PID|1\r\
               PV1|1|O";
    let json = convert(er7).unwrap();
    assert!(json.contains("\"ADT_A01\": {"));
}

#[test]
fn splits_batches_into_messages() {
    let text = "FHS|^~\\&|X\rBHS|^~\\&|X\r\
                MSH|^~\\&|A||||1||ACK|1|P|2.5\rMSA|AA|1\r\
                MSH|^~\\&|B||||2||ACK|2|P|2.5\rMSA|AA|2\r\
                BTS|1\rFTS|1";
    let messages = split_messages(text);
    assert_eq!(messages.len(), 2);
    assert!(messages[0].starts_with("MSH|^~\\&|A"));
    assert!(messages[1].starts_with("MSH|^~\\&|B"));
    let json = convert(&messages[1]).unwrap();
    assert!(json.contains("\"ACK\": {"));
    assert!(json.contains("\"MSA.1\": \"AA\""));
}
