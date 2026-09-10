//! Catch a typo in a hand-written JSON fixture with `Strict<Message>`,
//! wherever in the nested tree it is, instead of the plain type silently
//! reading the mistyped key as absent.
//!
//! Run with: `cargo run -p serde-hl7-v3 --example v3_catch_a_typo_with_strict`
#![forbid(unsafe_code)]

use serde_hl7_v3::{Message, Strict};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // "extention" — a typo of "extension" — three levels down, inside the
    // identifier of the domain payload's `id` child. Every key on every
    // object in this crate is optional or defaulted except the ones that
    // name a thing, so the plain type reads this fixture happily and the
    // identifier simply has no extension.
    let typo = r#"{
      "controlAct": {
        "classCode": "CACT", "moodCode": "EVN",
        "code": {"code": "PRPA_TE201305UV02"},
        "domain": {
          "name": "patient",
          "attributes": {"classCode": "PAT"},
          "children": [{"name": "id", "attributes": {"root": "1.2.3", "extention": "444333222"}}]
        }
      }
    }"#;

    // Inside an Element, attributes are a free-form map, so a typo *there*
    // is data, not a wrong key — this crate cannot know what attribute names
    // an element should have. The strict check bites where the keys are
    // this crate's own: rename the element's "attributes" key by mistake
    // and the difference shows.
    let plain: Message = serde_json::from_str(typo)?;
    println!(
        "Message::deserialize read the fixture; the id attributes are {:?}.",
        plain
            .control_act
            .as_ref()
            .and_then(|a| a.domain.as_ref())
            .and_then(|d| d.child("id"))
            .map(|id| &id.attributes)
    );

    let wrong_key = typo.replace(r#""attributes": {"root""#, r#""attrs": {"root""#);
    let plain: Message = serde_json::from_str(&wrong_key)?;
    println!(
        "\nWith \"attrs\" for \"attributes\" on the id element, the plain type sees {:?} attributes.",
        plain
            .control_act
            .as_ref()
            .and_then(|a| a.domain.as_ref())
            .and_then(|d| d.child("id"))
            .map(|id| id.attributes.len())
    );

    match serde_json::from_str::<Strict<Message>>(&wrong_key) {
        Ok(_) => unreachable!("Strict<Message> reports the typo"),
        Err(error) => println!("Strict<Message>::deserialize: {error}"),
    }

    let good: Strict<Message> = serde_json::from_str(typo)?;
    println!(
        "\nWith every key spelled right, strict deserialization passes: {} children under the payload.",
        good.control_act
            .as_ref()
            .and_then(|a| a.domain.as_ref())
            .map_or(0, |d| d.children.len())
    );
    Ok(())
}
