//! Matching a message's segments against an abstract message structure.
//!
//! HL7 sends a flat list of segments and expects the receiver to know that
//! in an `ORU_R01` the third `OBX` belongs to the second `OBR`. The
//! grammars that say so live in the dictionary ([`crate::dictionary::Item`]);
//! this module is the greedy recursive-descent matcher that applies one to
//! a segment list and reports the nesting it found.
//!
//! Matching is all-or-nothing and never rewrites the message: either the
//! whole segment list fits the grammar, or the caller keeps the flat list
//! it already has. That is deliberate — a partial match would have to guess
//! where an unexpected Z-segment belongs, and a wrong guess is worse than
//! no grouping at all. [`crate::Message::tree`] takes exactly this fallback,
//! and [`crate::Message::validate`] reports the failure as a diagnostic
//! instead of hiding it.

use crate::dictionary::Item;

/// Where one segment, or one group of segments, sits in the matched
/// structure. Segments are named by their index into the list that was
/// matched, so the caller keeps ownership of the segments themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    /// The segment at this index in the input list.
    Segment(usize),
    /// A group occurrence and what it contains.
    Group {
        /// The group's name from the grammar, e.g. `ORDER_OBSERVATION`.
        name: String,
        /// The group's contents, in order.
        items: Vec<Layout>,
    },
}

impl Layout {
    /// The indices of every segment under this layout, in order.
    pub fn segment_indices(&self, out: &mut Vec<usize>) {
        match self {
            Layout::Segment(index) => out.push(*index),
            Layout::Group { items, .. } => {
                for item in items {
                    item.segment_indices(out);
                }
            }
        }
    }
}

/// Arrange `segments` (their names, in message order) into `items`.
///
/// Returns `None` unless every segment is consumed by the grammar, which
/// is what makes an unknown or misplaced segment fall back to a flat
/// reading rather than being silently dropped.
pub fn group(items: &[Item], segments: &[&str]) -> Option<Vec<Layout>> {
    let mut position = 0;
    let mut out = Vec::new();
    if match_items(items, segments, &mut position, &mut out) && position == segments.len() {
        Some(out)
    } else {
        None
    }
}

/// Match `items` against `segments` starting at `position`, appending what
/// matched to `out`. Greedy: a repeating item consumes as many occurrences
/// as it can before the next item is tried.
fn match_items(
    items: &[Item],
    segments: &[&str],
    position: &mut usize,
    out: &mut Vec<Layout>,
) -> bool {
    for item in items {
        let mut occurrences = 0;
        loop {
            let before = *position;
            if *position < segments.len() && item.can_start(segments[*position]) {
                match item {
                    Item::Segment { .. } => {
                        out.push(Layout::Segment(*position));
                        *position += 1;
                    }
                    Item::Group { name, items, .. } => {
                        let mut contents = Vec::new();
                        if !match_items(items, segments, position, &mut contents) {
                            return false;
                        }
                        out.push(Layout::Group {
                            name: name.clone(),
                            items: contents,
                        });
                    }
                }
                occurrences += 1;
            }
            // A group whose leading items are all optional can match while
            // consuming nothing; stop rather than loop forever on it.
            if *position == before || !item.repeats() {
                break;
            }
        }
        if occurrences == 0 && item.required() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Version;

    fn layout(structure: &str, segments: &[&str]) -> Option<Vec<Layout>> {
        let dictionary = Version::V2_5.dictionary();
        group(dictionary.structure(structure).unwrap(), segments)
    }

    #[test]
    fn groups_a_message_that_fits() {
        let found = layout("ORU_R01", &["MSH", "PID", "OBR", "OBX", "OBX"]).unwrap();
        assert_eq!(found[0], Layout::Segment(0));
        let Layout::Group { name, items } = &found[1] else {
            panic!("expected a group, got {:?}", found[1]);
        };
        assert_eq!(name, "PATIENT_RESULT");
        // PATIENT wraps the PID; ORDER_OBSERVATION wraps OBR and both OBXs.
        assert_eq!(items.len(), 2);
        let mut indices = Vec::new();
        found[1].segment_indices(&mut indices);
        assert_eq!(indices, [1, 2, 3, 4]);
    }

    #[test]
    fn repeats_a_group_once_per_occurrence() {
        let found = layout("ORU_R01", &["MSH", "PID", "OBR", "OBX", "OBR", "OBX"]).unwrap();
        let Layout::Group { items, .. } = &found[1] else {
            panic!("expected PATIENT_RESULT");
        };
        // One PATIENT group, then two ORDER_OBSERVATION groups.
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn refuses_a_message_that_does_not_fit() {
        // A Z-segment the grammar has no place for.
        assert_eq!(layout("ORU_R01", &["MSH", "PID", "OBR", "ZZZ"]), None);
        // A required segment missing.
        assert_eq!(layout("ACK", &["MSH"]), None);
        // Segments in an order the grammar does not allow.
        assert_eq!(layout("ACK", &["MSA", "MSH"]), None);
    }

    #[test]
    fn matches_the_flat_structures_too() {
        let found = layout("ACK", &["MSH", "MSA", "ERR", "ERR"]).unwrap();
        assert_eq!(
            found,
            [
                Layout::Segment(0),
                Layout::Segment(1),
                Layout::Segment(2),
                Layout::Segment(3)
            ]
        );
    }
}
