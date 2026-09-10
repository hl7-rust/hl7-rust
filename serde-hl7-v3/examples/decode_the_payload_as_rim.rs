//! Decode the domain payload into a RIM class with `hl7-3`, then serialize
//! the typed value — `Role`, `Act` — rather than the raw element tree.
//!
//! Run with: `cargo run -p serde-hl7-v3 --example decode_the_payload_as_rim`
#![forbid(unsafe_code)]

use serde_hl7_v3::{Act, Message, Role};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"
    <PRPA_IN201305UV02 xmlns="urn:hl7-org:v3">
      <controlActProcess classCode="CACT" moodCode="EVN">
        <subject>
          <patient classCode="PAT">
            <id root="2.16.840.1.113883.19.5" extension="444333222"/>
            <statusCode code="active"/>
            <effectiveTime value="20260101"/>
          </patient>
        </subject>
      </controlActProcess>
    </PRPA_IN201305UV02>"#;

    let message = Message::parse(xml)?;
    let domain = message
        .control_act
        .as_ref()
        .and_then(|act| act.domain.as_ref())
        .expect("the sample carries a payload");

    // The payload's shape is the interaction's business, not this crate's:
    // `hl7-3` reads it into whichever RIM class the caller says it is.
    let patient = Role(hl7_3::rim::Role::from_element(domain));
    println!("As a Role:\n{}\n", serde_json::to_string_pretty(&patient)?);

    let back: Role = serde_json::from_str(&serde_json::to_string(&patient)?)?;
    assert_eq!(back, patient);

    // The same element read as an Act shows what the RIM reading does with
    // attributes it does not expect: keeps the ones it knows, ignores the rest.
    let as_act = Act(hl7_3::rim::Act::from_element(domain));
    println!(
        "The same element read as an Act:\n{}",
        serde_json::to_string_pretty(&as_act)?
    );
    Ok(())
}
