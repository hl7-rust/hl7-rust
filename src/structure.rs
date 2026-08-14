//! HL7 v2.5 abstract message structures (segment groups).
//!
//! The v2.xml encoding nests segments inside group elements such as
//! `<ORM_O01.PATIENT>` or `<ORU_R01.ORDER_OBSERVATION>`. This module holds
//! grammars for a handful of common message structures and a greedy
//! recursive-descent matcher that arranges a message's segments into those
//! groups. If a message does not fit its grammar exactly (for example it
//! contains Z-segments, or uses a structure not defined here), the caller
//! falls back to a flat segment list under the root element.

use crate::xml::Node;

/// One entry in a message-structure grammar: either a bare segment or a
/// named group of items, each carrying whether it is required and whether
/// it may repeat (mirroring the HL7 chapter-2 structure tables).
#[derive(Debug)]
pub enum Item {
    /// A segment: code, required?, repeating?
    Seg(&'static str, bool, bool),
    /// A group: name suffix (appended to the message structure ID, e.g.
    /// `PATIENT` -> `ORM_O01.PATIENT`), required?, repeating?, children.
    Grp(&'static str, bool, bool, &'static [Item]),
}

use Item::{Grp, Seg};

impl Item {
    fn required(&self) -> bool {
        match self {
            Seg(_, req, _) | Grp(_, req, _, _) => *req,
        }
    }

    fn repeating(&self) -> bool {
        match self {
            Seg(_, _, rep) | Grp(_, _, rep, _) => *rep,
        }
    }

    /// Can this item begin with segment `seg`? For a group this walks the
    /// leading optional items plus the first required one (its FIRST set).
    fn can_start(&self, seg: &str) -> bool {
        match self {
            Seg(id, _, _) => *id == seg,
            Grp(_, _, _, items) => {
                for item in *items {
                    if item.can_start(seg) {
                        return true;
                    }
                    if item.required() {
                        return false;
                    }
                }
                false
            }
        }
    }
}

const ACK: &[Item] = &[
    Seg("MSH", true, false),
    Seg("SFT", false, true),
    Seg("MSA", true, false),
    Seg("ERR", false, true),
];

const ADT_A01: &[Item] = &[
    Seg("MSH", true, false),
    Seg("SFT", false, true),
    Seg("EVN", true, false),
    Seg("PID", true, false),
    Seg("PD1", false, false),
    Seg("ROL", false, true),
    Seg("NK1", false, true),
    Seg("PV1", true, false),
    Seg("PV2", false, false),
    Seg("ROL", false, true),
    Seg("DB1", false, true),
    Seg("OBX", false, true),
    Seg("AL1", false, true),
    Seg("DG1", false, true),
    Seg("DRG", false, false),
    Grp(
        "PROCEDURE",
        false,
        true,
        &[Seg("PR1", true, false), Seg("ROL", false, true)],
    ),
    Seg("GT1", false, true),
    Grp(
        "INSURANCE",
        false,
        true,
        &[
            Seg("IN1", true, false),
            Seg("IN2", false, false),
            Seg("IN3", false, true),
            Seg("ROL", false, true),
        ],
    ),
    Seg("ACC", false, false),
    Seg("UB1", false, false),
    Seg("UB2", false, false),
    Seg("PDA", false, false),
];

const ORM_O01: &[Item] = &[
    Seg("MSH", true, false),
    Seg("NTE", false, true),
    Grp(
        "PATIENT",
        false,
        false,
        &[
            Seg("PID", true, false),
            Seg("PD1", false, false),
            Seg("NTE", false, true),
            Grp(
                "PATIENT_VISIT",
                false,
                false,
                &[Seg("PV1", true, false), Seg("PV2", false, false)],
            ),
            Grp(
                "INSURANCE",
                false,
                true,
                &[
                    Seg("IN1", true, false),
                    Seg("IN2", false, false),
                    Seg("IN3", false, false),
                ],
            ),
            Seg("GT1", false, false),
            Seg("AL1", false, true),
        ],
    ),
    Grp(
        "ORDER",
        true,
        true,
        &[
            Seg("ORC", true, false),
            // The standard allows a choice of detail segments (OBR, RQD,
            // RQ1, RXO, ODS, ODT); OBR is supported here. Messages using
            // the other detail segments render flat.
            Grp(
                "ORDER_DETAIL",
                false,
                false,
                &[
                    Seg("OBR", true, false),
                    Seg("NTE", false, true),
                    Seg("CTD", false, false),
                    Seg("DG1", false, true),
                    Grp(
                        "OBSERVATION",
                        false,
                        true,
                        &[Seg("OBX", true, false), Seg("NTE", false, true)],
                    ),
                ],
            ),
            Seg("FT1", false, true),
            Seg("CTI", false, true),
            Seg("BLG", false, false),
        ],
    ),
];

const ORU_R01: &[Item] = &[
    Seg("MSH", true, false),
    Seg("SFT", false, true),
    Grp(
        "PATIENT_RESULT",
        true,
        true,
        &[
            Grp(
                "PATIENT",
                false,
                false,
                &[
                    Seg("PID", true, false),
                    Seg("PD1", false, false),
                    Seg("NTE", false, true),
                    Seg("NK1", false, true),
                    Grp(
                        "VISIT",
                        false,
                        false,
                        &[Seg("PV1", true, false), Seg("PV2", false, false)],
                    ),
                ],
            ),
            Grp(
                "ORDER_OBSERVATION",
                true,
                true,
                &[
                    Seg("ORC", false, false),
                    Seg("OBR", true, false),
                    Seg("NTE", false, true),
                    Grp(
                        "TIMING_QTY",
                        false,
                        true,
                        &[Seg("TQ1", true, false), Seg("TQ2", false, true)],
                    ),
                    Seg("CTD", false, false),
                    Grp(
                        "OBSERVATION",
                        false,
                        true,
                        &[Seg("OBX", true, false), Seg("NTE", false, true)],
                    ),
                    Seg("FT1", false, true),
                    Seg("CTI", false, true),
                    Grp(
                        "SPECIMEN",
                        false,
                        true,
                        &[Seg("SPM", true, false), Seg("OBX", false, true)],
                    ),
                ],
            ),
        ],
    ),
    Seg("DSC", false, false),
];

/// Grammar for a message structure ID (as derived from MSH-9).
pub fn structure_for(root: &str) -> Option<&'static [Item]> {
    match root {
        "ACK" => Some(ACK),
        "ADT_A01" => Some(ADT_A01),
        "ORM_O01" => Some(ORM_O01),
        "ORU_R01" => Some(ORU_R01),
        _ => None,
    }
}

/// Arrange segment nodes into the grammar's groups. Returns `None` when the
/// message does not fit the grammar exactly (caller then renders flat).
pub fn group_segments(root: &str, items: &[Item], segs: &[(String, Node)]) -> Option<Vec<Node>> {
    let mut pos = 0;
    let mut out = Vec::new();
    if match_items(root, items, segs, &mut pos, &mut out) && pos == segs.len() {
        Some(out)
    } else {
        None
    }
}

fn match_items(
    root: &str,
    items: &[Item],
    segs: &[(String, Node)],
    pos: &mut usize,
    out: &mut Vec<Node>,
) -> bool {
    for item in items {
        let mut count = 0;
        loop {
            let before = *pos;
            if *pos < segs.len() && item.can_start(&segs[*pos].0) {
                match item {
                    Seg(_, _, _) => {
                        out.push(segs[*pos].1.clone());
                        *pos += 1;
                    }
                    Grp(name, _, _, kids) => {
                        let mut group_kids = Vec::new();
                        if !match_items(root, kids, segs, pos, &mut group_kids) {
                            return false;
                        }
                        let mut node = Node::group(format!("{root}.{name}"));
                        node.kids = group_kids;
                        out.push(node);
                    }
                }
                count += 1;
            }
            if *pos == before || !item.repeating() {
                break;
            }
        }
        if count == 0 && item.required() {
            return false;
        }
    }
    true
}
