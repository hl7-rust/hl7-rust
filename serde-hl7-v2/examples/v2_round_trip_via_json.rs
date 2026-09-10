//! ER7 → `Message` → JSON → `Message` → ER7, unchanged — the crate's
//! flagship path, with the release the message was read as riding along.
//!
//! Run with: `cargo run -p serde-hl7-v2 --example v2_round_trip_via_json`
#![forbid(unsafe_code)]

use serde_hl7_v2::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260815081500||ORU^R01^ORU_R01|MSG00042|P|2.5\r\
                PID|1||444333222^^^ACME^MR||EVERYWOMAN^EVE^E||19620320|F\r\
                OBR|1||LAB0042|2093-3^Cholesterol^LN\r\
                OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F";

    let message = Message::parse(text)?;
    println!(
        "Read as HL7 v{}, structure {}.\n",
        message.version(),
        message.structure_id()
    );

    // Any Serde format works; JSON is simply the easiest to read on a page.
    let json = serde_json::to_string_pretty(&message)?;
    println!("As JSON:\n{json}\n");

    let back: Message = serde_json::from_str(&json)?;
    assert_eq!(back.to_er7(), text, "the ER7 text is unchanged");
    assert_eq!(back.version(), message.version(), "and so is the release");
    println!("Round trip: identical ER7, same release, dictionary resolved again on the way in.");
    println!("PID-5.1 after the round trip: {:?}", back.get("PID-5.1")?);
    Ok(())
}
