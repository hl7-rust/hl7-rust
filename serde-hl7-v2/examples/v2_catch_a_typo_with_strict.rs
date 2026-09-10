//! Catch a typo in a hand-written JSON fixture with `Strict<Message>`,
//! instead of it silently reading the version from MSH-12 as if the key
//! had never been written.
//!
//! Run with: `cargo run -p serde-hl7-v2 --example v2_catch_a_typo_with_strict`
#![forbid(unsafe_code)]

use serde_hl7_v2::{Message, Strict};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // "verison" — a typo of "version", the one optional key this type has.
    // The plain type ignores it (unknown keys are tolerated, by design) and
    // falls back to MSH-12, so the fixture's intent — read this as 2.3.1
    // whatever the header says — is lost without a word.
    let typo =
        r#"{"verison": "2.3.1", "er7": "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260815||ADT^A01|1|P|2.5"}"#;

    let plain: Message = serde_json::from_str(typo)?;
    println!(
        "Message::deserialize read the release as {} — the typo went unnoticed.",
        plain.version()
    );

    let strict: Result<Strict<Message>, _> = serde_json::from_str(typo);
    match strict {
        Ok(_) => unreachable!("Strict<Message> reports the typo"),
        Err(error) => println!("Strict<Message>::deserialize: {error}"),
    }

    // And a fixture with no typo passes strict deserialization unchanged.
    let good = typo.replace("verison", "version");
    let message: Strict<Message> = serde_json::from_str(&good)?;
    println!("With the key spelled right: read as {}.", message.version());
    Ok(())
}
