//! XML → `Message` → JSON → `Message`, equal — the crate's flagship path.
//!
//! Run with: `cargo run -p serde-hl7-v3 --example round_trip_via_json`
#![forbid(unsafe_code)]

use serde_hl7_v3::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"
    <PRPA_IN201305UV02 xmlns="urn:hl7-org:v3">
      <id root="2.16.840.1.113883.19.5" extension="MSG00042"/>
      <creationTime value="20260815081500"/>
      <interactionId root="2.16.840.1.113883.1.6" extension="PRPA_IN201305UV02"/>
      <sender typeCode="SND"><device classCode="DEV" determinerCode="INSTANCE">
        <id root="2.16.840.1.113883.19.5.1"/></device></sender>
      <receiver typeCode="RCV"><device classCode="DEV" determinerCode="INSTANCE">
        <id root="2.16.840.1.113883.19.5.2"/></device></receiver>
      <controlActProcess classCode="CACT" moodCode="EVN">
        <code code="PRPA_TE201305UV02"/>
        <subject>
          <patient classCode="PAT">
            <id root="2.16.840.1.113883.19.5" extension="444333222"/>
            <statusCode code="active"/>
          </patient>
        </subject>
      </controlActProcess>
    </PRPA_IN201305UV02>"#;

    let message = Message::parse(xml)?;
    println!(
        "Interaction {}, trigger event {}.\n",
        message
            .interaction_id
            .as_ref()
            .and_then(|id| id.extension.as_deref())
            .unwrap_or("?"),
        message
            .control_act
            .as_ref()
            .and_then(|act| act.code.as_ref())
            .map_or("?", |code| code.code.as_str()),
    );

    // Any Serde format works; JSON is simply the easiest to read on a page.
    let json = serde_json::to_string_pretty(&message)?;
    println!("As JSON:\n{json}\n");

    let back: Message = serde_json::from_str(&json)?;
    assert_eq!(back, message, "every field survives the round trip");
    println!("Round trip: equal value, domain payload element tree included.");
    Ok(())
}
