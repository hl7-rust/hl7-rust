//! Arranging a message's segments into the groups its structure defines.
//!
//! The v2.xml encoding nests segments inside group elements such as
//! `<ORM_O01.PATIENT>` or `<ORU_R01.ORDER_OBSERVATION>`. Which groups exist,
//! and which segments go in them, is dictionary knowledge; the matching is
//! done by [`hl7_2::structure::group`]. This module is the small piece in
//! between: it turns the [`Layout`] that matcher reports into the XML nodes
//! this crate emits.
//!
//! Until 0.5.0 the grammars and the matcher both lived here, in a copy of
//! the tables `hl7-2` now owns. Reading them from a dictionary instead is
//! what lets a caller convert against a vendor's own schemas.

use crate::xml::Node;
use hl7_2::dictionary::Item;
use hl7_2::structure::{Layout, group};

/// Arrange segment nodes into the structure's groups.
///
/// Returns `None` when the message does not fit the grammar exactly — an
/// unknown segment, a missing required one, segments out of order — and the
/// caller then renders the segments flat under the root element. That is
/// deliberate: a partial match would have to guess where an unexpected
/// segment belongs, and a wrong guess is worse than no grouping at all.
#[must_use]
pub fn group_segments(root: &str, items: &[Item], segs: &[(String, Node)]) -> Option<Vec<Node>> {
    let names: Vec<&str> = segs.iter().map(|(name, _)| name.as_str()).collect();
    let layouts = group(items, &names)?;
    Some(to_nodes(root, &layouts, segs))
}

/// Turn matched layouts into nodes, naming groups the way the family names
/// them: the message structure ID, a dot, then the group name.
fn to_nodes(root: &str, layouts: &[Layout], segs: &[(String, Node)]) -> Vec<Node> {
    layouts
        .iter()
        .map(|layout| match layout {
            Layout::Segment(index) => segs[*index].1.clone(),
            Layout::Group { name, items } => {
                let mut node = Node::group(format!("{root}.{name}"));
                node.kids = to_nodes(root, items, segs);
                node
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hl7_2::Version;

    fn segs(names: &[&str]) -> Vec<(String, Node)> {
        names
            .iter()
            .map(|name| (name.to_string(), Node::group(*name)))
            .collect()
    }

    #[test]
    fn nests_matched_groups_under_prefixed_names() {
        let dictionary = Version::V2_5.dictionary();
        let items = dictionary.structure("ORU_R01").unwrap();
        let nodes = group_segments("ORU_R01", items, &segs(&["MSH", "PID", "OBR", "OBX"]))
            .expect("a conforming message groups");
        assert_eq!(nodes[0].name, "MSH");
        assert_eq!(nodes[1].name, "ORU_R01.PATIENT_RESULT");
        assert_eq!(nodes[1].kids[0].name, "ORU_R01.PATIENT");
        assert_eq!(nodes[1].kids[0].kids[0].name, "PID");
    }

    #[test]
    fn a_message_that_does_not_fit_reports_none_so_the_caller_stays_flat() {
        let dictionary = Version::V2_5.dictionary();
        let items = dictionary.structure("ORU_R01").unwrap();
        assert!(group_segments("ORU_R01", items, &segs(&["MSH", "ZZZ"])).is_none());
    }
}
