use hl7_v2_from_er7_into_xml::{
    Options, convert, convert_with_dictionary, convert_with_options, split_messages,
};

/// The spec's ORM^O01 example (with an ORC/OBR order so the message is a
/// complete ORM_O01), checked against the exact expected v2.xml document,
/// including the `ORM_O01.PATIENT` / `ORM_O01.ORDER` group nesting.
#[test]
fn golden_orm_o01() {
    let er7 = "MSH|^~\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\r\
               PID|1||241900||MEDIANO^FOUAZ\r\
               ORC|NW|ORD1\r\
               OBR|1|ORD1||24331-1^Lipid Panel^LN";
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<ORM_O01 xmlns="urn:hl7-org:v2xml">
  <MSH>
    <MSH.1>|</MSH.1>
    <MSH.2>^~\&amp;</MSH.2>
    <MSH.3>
      <HD.1>hphis</HD.1>
    </MSH.3>
    <MSH.5>
      <HD.1>EPIC</HD.1>
    </MSH.5>
    <MSH.7>
      <TS.1>20131011093851</TS.1>
    </MSH.7>
    <MSH.9>
      <MSG.1>ORM</MSG.1>
      <MSG.2>O01</MSG.2>
    </MSH.9>
    <MSH.10>14AAACVDD</MSH.10>
    <MSH.11>
      <PT.1>P</PT.1>
    </MSH.11>
    <MSH.12>
      <VID.1>2.5</VID.1>
    </MSH.12>
  </MSH>
  <ORM_O01.PATIENT>
    <PID>
      <PID.1>1</PID.1>
      <PID.3>
        <CX.1>241900</CX.1>
      </PID.3>
      <PID.5>
        <XPN.1>
          <FN.1>MEDIANO</FN.1>
        </XPN.1>
        <XPN.2>FOUAZ</XPN.2>
      </PID.5>
    </PID>
  </ORM_O01.PATIENT>
  <ORM_O01.ORDER>
    <ORC>
      <ORC.1>NW</ORC.1>
      <ORC.2>
        <EI.1>ORD1</EI.1>
      </ORC.2>
    </ORC>
    <ORM_O01.ORDER_DETAIL>
      <OBR>
        <OBR.1>1</OBR.1>
        <OBR.2>
          <EI.1>ORD1</EI.1>
        </OBR.2>
        <OBR.4>
          <CE.1>24331-1</CE.1>
          <CE.2>Lipid Panel</CE.2>
          <CE.3>LN</CE.3>
        </OBR.4>
      </OBR>
    </ORM_O01.ORDER_DETAIL>
  </ORM_O01.ORDER>
</ORM_O01>
"#;
    assert_eq!(convert(er7).unwrap(), expected);
}

/// The spec's PID sample: PID|1||241900||TEST^FOUAZ.
#[test]
fn spec_pid_sample() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rPID|1||241900||TEST^FOUAZ";
    let xml = convert(er7).unwrap();
    let expected_pid = "  <PID>\n    <PID.1>1</PID.1>\n    <PID.3>\n      <CX.1>241900</CX.1>\n    \
                        </PID.3>\n    <PID.5>\n      <XPN.1>\n        <FN.1>TEST</FN.1>\n      \
                        </XPN.1>\n      <XPN.2>FOUAZ</XPN.2>\n    </PID.5>\n  </PID>\n";
    assert!(xml.contains(expected_pid), "got:\n{xml}");
    assert!(xml.contains("<XXX_X01 xmlns=\"urn:hl7-org:v2xml\">"));
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
    let xml = convert(er7).unwrap();
    assert!(xml.contains("<ORU_R01 xmlns=\"urn:hl7-org:v2xml\">"));
    assert!(xml.contains("<ORU_R01.PATIENT_RESULT>"));
    assert!(xml.contains("<ORU_R01.PATIENT>"));
    assert!(xml.contains("<ORU_R01.VISIT>"));
    assert!(xml.contains("<ORU_R01.ORDER_OBSERVATION>"));
    // Two OBX observation groups; the NTE nests inside the second one.
    assert_eq!(xml.matches("<ORU_R01.OBSERVATION>").count(), 2);
    assert!(xml.contains("<NTE.3>Fasting sample.</NTE.3>"));
    // OBX-7 reference range "<200" is XML-escaped.
    assert!(xml.contains("<OBX.7>&lt;200</OBX.7>"));
}

/// OBX-5's element names come from the value type declared in OBX-2.
#[test]
fn obx5_type_follows_obx2() {
    let er7 = "MSH|^~\\&|LAB||||20260814||ORU^R01|1|P|2.5\r\
               PID|1\r\
               OBR|1\r\
               OBX|1|CE|X||A^B\r\
               OBX|2|NM|Y||42";
    let xml = convert(er7).unwrap();
    assert!(xml.contains("<OBX.5>\n            <CE.1>A</CE.1>\n            <CE.2>B</CE.2>"));
    assert!(xml.contains("<OBX.5>42</OBX.5>"));
}

/// A message that does not fit its declared structure (here: a Z-segment)
/// renders flat, and unknown segments get positional generic names.
#[test]
fn falls_back_to_flat_on_structure_mismatch() {
    let er7 = "MSH|^~\\&|APP||||20260814||ORM^O01|1|P|2.5\r\
               PID|1\r\
               ORC|NW\r\
               ZDS|1.2.840^app^DICOM";
    let xml = convert(er7).unwrap();
    assert!(!xml.contains("ORM_O01.PATIENT"));
    assert!(xml.contains("<ZDS>"));
    assert!(xml.contains("<ZDS.1.1>1.2.840</ZDS.1.1>"));
    assert!(xml.contains("<ZDS.1.2>app</ZDS.1.2>"));
}

/// The --flat option disables grouping even for well-formed messages.
#[test]
fn flat_option_disables_grouping() {
    let er7 = "MSH|^~\\&|APP||||20260814||ORM^O01|1|P|2.5\rPID|1\rORC|NW";
    let xml = convert_with_options(
        er7,
        Options {
            flat: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!xml.contains("ORM_O01.PATIENT"));
    assert!(xml.contains("<PID.1>1</PID.1>"));
}

/// Field repetitions (~) become repeated sibling elements.
#[test]
fn repetitions_become_sibling_elements() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rPID|1||111~222";
    let xml = convert(er7).unwrap();
    assert_eq!(xml.matches("<PID.3>").count(), 2);
    assert!(xml.contains("<CX.1>111</CX.1>"));
    assert!(xml.contains("<CX.1>222</CX.1>"));
}

/// Subcomponents of a composite component use its own type's names,
/// e.g. the assigning authority (HD) inside a CX identifier.
#[test]
fn subcomponents_use_component_type_names() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\r\
               PID|1||123^^^HOSP&1.2.3.4&ISO";
    let xml = convert(er7).unwrap();
    assert!(xml.contains(
        "<CX.4>\n        <HD.1>HOSP</HD.1>\n        <HD.2>1.2.3.4</HD.2>\n        \
         <HD.3>ISO</HD.3>\n      </CX.4>"
    ));
}

/// The HL7 explicit null `""` becomes an empty element.
#[test]
fn explicit_null_becomes_empty_element() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rPID|1|\"\"";
    let xml = convert(er7).unwrap();
    assert!(xml.contains("<PID.2/>"));
}

/// Escape sequences decode to delimiter characters and XML-escape cleanly.
#[test]
fn er7_escapes_decode() {
    let er7 = "MSH|^~\\&|APP||||20260814||XXX^X01|1|P|2.5\rNTE|1||A\\T\\B \\F\\ C\\S\\D";
    let xml = convert(er7).unwrap();
    assert!(xml.contains("<NTE.3>A&amp;B | C^D</NTE.3>"));
}

/// MSH-9.3, when present, wins as the message structure / root name.
#[test]
fn msh9_3_names_the_root() {
    let er7 = "MSH|^~\\&|APP||||20260814||ADT^A08^ADT_A01|1|P|2.5\r\
               EVN|A08|20260814\r\
               PID|1\r\
               PV1|1|O";
    let xml = convert(er7).unwrap();
    assert!(xml.contains("<ADT_A01 xmlns=\"urn:hl7-org:v2xml\">"));
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
    let xml = convert(&messages[1]).unwrap();
    assert!(xml.contains("<ACK xmlns=\"urn:hl7-org:v2xml\">"));
    assert!(xml.contains("<MSA.1>AA</MSA.1>"));
}

// --------------------------------------------------------------------------
// Schema mode: the dictionary decides the document's shape.
// --------------------------------------------------------------------------

/// A dictionary that states cardinality the way a generated one does.
fn schema_dictionary() -> hl7_v2::Dictionary {
    hl7_v2::Dictionary::from_json(
        r#"{
             "inherits": "2.5",
             "segments": {
               "PID": [
                 "SI",
                 {"type": "CX", "required": true},
                 {"type": "CX", "repeats": true},
                 "CX",
                 {"type": "XPN", "required": true}
               ]
             }
           }"#,
        "test",
    )
    .unwrap()
}

fn schema_convert(er7: &str) -> String {
    convert_with_dictionary(
        er7,
        &schema_dictionary(),
        Options {
            schema_shape: true,
            ..Default::default()
        },
    )
    .unwrap()
}

const HEADER: &str = "MSH|^~\\&|APP||||1||ADT^A01^ADT_A01|1|P|2.5\r";

#[test]
fn a_required_field_is_written_even_when_the_message_leaves_it_empty() {
    // PID-2 and PID-5 are required; the message supplies neither. Each is
    // written as the full tree its data type declares, because the schema
    // requires those elements too, not just the field element around them.
    let xml = schema_convert(&format!("{HEADER}PID|1"));
    assert!(xml.contains("<PID.2>"), "{xml}");
    assert!(xml.contains("<CX.1/>"), "{xml}");
    assert!(xml.contains("<PID.5>"), "{xml}");
    assert!(xml.contains("<XPN.1>"), "{xml}"); // XPN.1 is an FN, so it nests
    assert!(xml.contains("<FN.1/>"), "{xml}");
    // PID-4 is optional and empty, so it is not written at all.
    assert!(!xml.contains("<PID.4"), "{xml}");
}

#[test]
fn every_component_a_composite_declares_is_written() {
    // HD.1/HD.2/HD.3 are all `minOccurs="1"` in the HL7 schemas, so a
    // CX.4 carrying only an HD.1 still writes the other two.
    let xml = schema_convert(&format!("{HEADER}PID|1||700001^^^169^PI"));
    assert!(
        xml.contains("<HD.1>169</HD.1><HD.2/><HD.3/>") || xml.contains("<HD.2/>"),
        "{xml}"
    );
    let cx: Vec<_> = (1..=10).map(|n| format!("<CX.{n}")).collect();
    for tag in &cx {
        assert!(xml.contains(tag.as_str()), "missing {tag} in {xml}");
    }
}

#[test]
fn a_field_that_cannot_repeat_keeps_its_repetition_separator_as_text() {
    // PID-4 does not repeat, so `A~B` stays one element...
    let xml = schema_convert(&format!("{HEADER}PID|1||x|A~B"));
    assert_eq!(xml.matches("<PID.4>").count(), 1, "{xml}");
    assert!(xml.contains("A~B"), "{xml}");
    // ...while PID-3, which does repeat, becomes two.
    let xml = schema_convert(&format!("{HEADER}PID|1||A~B"));
    assert_eq!(xml.matches("<PID.3>").count(), 2, "{xml}");
    assert!(!xml.contains("A~B"), "{xml}");
}

#[test]
fn no_element_is_written_for_a_field_the_dictionary_does_not_declare() {
    // The dictionary stops at PID-5, so a PID-6 in the message is not
    // written: the schema the output is validated against has no place
    // for it.
    let xml = schema_convert(&format!("{HEADER}PID|1||||Smith^John|extra"));
    assert!(xml.contains("<PID.5>"), "{xml}");
    assert!(!xml.contains("<PID.6"), "{xml}");
}

#[test]
fn without_schema_shape_the_message_still_decides() {
    // The same dictionary, read as a table rather than as a schema: no
    // padding, no gating, and nothing dropped.
    let xml = convert_with_dictionary(
        &format!("{HEADER}PID|1||||Smith^John|extra"),
        &schema_dictionary(),
        Options::default(),
    )
    .unwrap();
    assert!(!xml.contains("<PID.2/>"), "{xml}");
    assert!(xml.contains("<PID.6>extra</PID.6>"), "{xml}");
}

#[test]
fn a_dialect_dictionary_names_elements_its_own_way() {
    // A vendor that redefines PID-3 as a plain string gets a leaf where the
    // standard would have nested CX components.
    let dictionary = hl7_v2::Dictionary::from_json(
        r#"{"inherits": "2.5", "segments": {"PID": {"3": "ST"}}}"#,
        "vendor",
    )
    .unwrap();
    let xml = convert_with_dictionary(
        &format!("{HEADER}PID|1||abc"),
        &dictionary,
        Options::default(),
    )
    .unwrap();
    assert!(xml.contains("<PID.3>abc</PID.3>"), "{xml}");
    assert!(!xml.contains("<CX.1>"), "{xml}");
}
