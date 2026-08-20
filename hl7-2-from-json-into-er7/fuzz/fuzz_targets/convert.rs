//! Converting arbitrary JSON is total, bounded, and produces valid ER7.
//!
//! Three properties:
//!
//! 1. **Total.** Any bytes produce ER7 or an [`Hl7Error`]; below the header
//!    no shape of input is rejected (spec §5), and none of them may panic.
//! 2. **Bounded.** The output cannot be wildly larger than the input.
//!    Positions are dense — position `n` costs `n` slots — so before
//!    `MAX_POSITION` existed, a hundred-byte document asking for
//!    `"PID.100000000"` produced a hundred megabytes of separators.
//! 3. **Valid.** Whatever comes out parses as ER7 and carries no
//!    unterminated escape sequence, which would swallow the rest of a value
//!    in the next reader down the line.

#![no_main]

use er7::escape::{Escape, escapes};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(converted) = hl7_2_from_json_into_er7::convert(text) else {
        return;
    };
    assert!(
        converted.len() <= text.len().saturating_mul(64) + 4096,
        "{} bytes of JSON became {} bytes of ER7",
        text.len(),
        converted.len()
    );
    let message = er7::parse(&converted).expect("converted ER7 does not parse");
    let separators = &message.separators;
    for segment in &message.segments {
        // The header's first two fields are the delimiters themselves,
        // which are structure rather than escaped data.
        for field in segment.fields.iter().skip(usize::from(segment.is_header()) * 2) {
            for repetition in &field.repetitions {
                for component in &repetition.components {
                    for subcomponent in &component.subcomponents {
                        for token in escapes(&subcomponent.raw, separators) {
                            assert!(
                                !matches!(token, Escape::Unterminated(_)),
                                "value {:?} carries an unterminated escape sequence",
                                subcomponent.raw
                            );
                        }
                    }
                }
            }
        }
    }
});
