//! HL7 v2.5 data-type knowledge used to name JSON keys.
//!
//! In this crate's JSON encoding a field key such as `"PID.5"` holds an
//! object keyed by the field's data type components (`"XPN.1"`, `"XPN.2"`,
//! ...), and a composite component nests its own type
//! (`"XPN.1": {"FN.1": ...}`). These tables carry the per-segment field
//! data types and the per-type component data types for the segments and
//! types that appear in common v2.5 messages. Anything not listed here
//! falls back to positional generic names (`"SEG.n.m"`).

/// Component data types of a composite data type, or `None` when the type
/// is primitive (ST, ID, IS, NM, SI, DT, DTM, TX, FT, ...) or unknown.
pub fn composite_components(dt: &str) -> Option<&'static [&'static str]> {
    let v: &'static [&'static str] = match dt {
        "AD" => &["ST", "ST", "ST", "ST", "ST", "ID", "ST", "ST"],
        "AUI" => &["ST", "DT", "ST"],
        "CCD" => &["ID", "TS"],
        "CE" => &["ST", "ST", "ID", "ST", "ST", "ID"],
        "CNE" => &["ST", "ST", "ID", "ST", "ST", "ID", "ST", "ST", "ST"],
        "CNN" => &["ST", "ST", "ST", "ST", "ST", "ST", "IS", "IS", "HD"],
        "CP" => &["MO", "ID", "NM", "NM", "CE", "ID"],
        "CQ" => &["NM", "CE"],
        "CWE" => &["ST", "ST", "ID", "ST", "ST", "ID", "ST", "ST", "ST"],
        "CX" => &["ST", "ST", "ID", "HD", "ID", "HD", "DT", "DT", "CWE", "CWE"],
        "DLD" => &["IS", "TS"],
        "DLN" => &["ST", "IS", "DT"],
        "DR" => &["TS", "TS"],
        "ED" => &["HD", "ID", "ID", "ID", "TX"],
        "EI" => &["ST", "IS", "ST", "ID"],
        "EIP" => &["EI", "EI"],
        "ELD" => &["ST", "NM", "NM", "CE"],
        "ERL" => &["ST", "NM", "NM", "NM", "NM", "NM"],
        "FC" => &["IS", "TS"],
        "FN" => &["ST", "ST", "ST", "ST", "ST"],
        "HD" => &["IS", "ST", "ID"],
        "JCC" => &["IS", "IS", "TX"],
        "MO" => &["NM", "ID"],
        "MOC" => &["MO", "CE"],
        "MSG" => &["ID", "ID", "ID"],
        "NDL" => &[
            "CNN", "TS", "TS", "IS", "IS", "IS", "HD", "IS", "IS", "IS", "IS",
        ],
        "PL" => &["IS", "IS", "IS", "HD", "IS", "IS", "IS", "IS", "ST", "EI"],
        "PRL" => &["CE", "ST", "TX"],
        "PT" => &["ID", "ID"],
        "RI" => &["IS", "ST"],
        "RP" => &["ST", "HD", "ID", "ID"],
        "SAD" => &["ST", "ST", "ST"],
        "SN" => &["ST", "NM", "ST", "NM"],
        "SPS" => &["CWE", "CWE", "TX", "CWE", "CWE", "CE", "CWE"],
        "TQ" => &[
            "CQ", "RI", "ST", "TS", "TS", "ST", "ST", "TX", "ID", "OSD", "CE", "NM",
        ],
        "TS" => &["DTM", "ID"],
        "VID" => &["ID", "CE", "CE"],
        "XAD" => &[
            "SAD", "ST", "ST", "ST", "ST", "ID", "ID", "ST", "IS", "IS", "ID", "DR", "TS", "TS",
        ],
        "XCN" => &[
            "ST", "FN", "ST", "ST", "ST", "ST", "IS", "IS", "HD", "ID", "ST", "ID", "ID", "HD",
            "ID", "CE", "DR", "ID", "TS", "TS", "ST", "CE", "CE",
        ],
        "XON" => &["ST", "IS", "NM", "NM", "ID", "HD", "ID", "HD", "ID", "ID"],
        "XPN" => &["FN", "ST", "ST", "ST", "ST", "IS", "ID", "ID", "DR", "TS"],
        "XTN" => &[
            "ST", "ID", "ID", "ST", "NM", "NM", "NM", "NM", "ST", "ST", "ST", "ST",
        ],
        _ => return None,
    };
    Some(v)
}

/// Field data types of a segment (index 0 is SEG-1). `None` for segments
/// not in the table (including Z-segments), which then use generic names.
/// The sentinel `"VAR"` marks OBX-5, whose type is given by OBX-2.
pub fn segment_fields(seg: &str) -> Option<&'static [&'static str]> {
    let v: &'static [&'static str] = match seg {
        "AL1" => &["SI", "CE", "CE", "CE", "ST", "TS"],
        "BLG" => &["CCD", "ID", "CX", "CWE"],
        "CTI" => &["EI", "CE", "CE"],
        "DG1" => &[
            "SI", "ID", "CE", "ST", "TS", "IS", "CE", "CE", "ID", "NM", "XCN", "NM", "CP", "IS",
            "ID", "XCN", "IS", "ID", "TS", "XCN", "EI",
        ],
        "DSC" => &["ST", "ID"],
        "ERR" => &[
            "ELD", "ERL", "CWE", "ID", "CWE", "ST", "TX", "TX", "IS", "CWE", "CWE", "XTN",
        ],
        "EVN" => &["ID", "TS", "TS", "IS", "XCN", "TS", "HD"],
        "IN1" => &[
            "SI", "CE", "CX", "XON", "XAD", "XPN", "XTN", "ST", "XON", "CX", "XON", "DT", "DT",
            "AUI", "IS", "XPN", "CE", "TS", "XAD", "IS", "IS", "IS", "ST", "DT", "ST", "DT", "IS",
            "ST", "TS", "XCN", "IS", "IS", "NM", "NM", "IS", "ST", "CP", "CP", "NM", "CP", "CP",
            "CE", "IS", "XAD", "ST", "IS", "IS", "IS", "CX", "IS", "DT", "IS", "IS",
        ],
        "MRG" => &["CX", "CX", "CX", "CX", "CX", "XPN", "CX"],
        "MSA" => &["ID", "ST", "ST", "NM", "ST", "CE"],
        "MSH" => &[
            "ST", "ST", "HD", "HD", "HD", "HD", "TS", "ST", "MSG", "ST", "PT", "VID", "NM", "ST",
            "ID", "ID", "ID", "ID", "CE", "ID", "EI",
        ],
        "NK1" => &[
            "SI", "XPN", "CE", "XAD", "XTN", "XTN", "CE", "DT", "DT", "ST", "JCC", "CX", "XON",
            "CE", "IS", "TS", "CE", "IS", "CE", "CE", "IS", "CE", "ID", "IS", "ST", "XPN", "CE",
            "CE", "CE", "XPN", "XTN", "XAD", "CX", "IS", "IS", "ST", "IS", "ST", "CWE",
        ],
        "NTE" => &["SI", "ID", "FT", "CE"],
        "OBR" => &[
            "SI", "EI", "EI", "CE", "ID", "TS", "TS", "TS", "CQ", "XCN", "ID", "CE", "ST", "TS",
            "SPS", "XCN", "XTN", "ST", "ST", "ST", "ST", "TS", "MOC", "ID", "ID", "PRL", "TQ",
            "XCN", "EIP", "ID", "CE", "NDL", "NDL", "NDL", "NDL", "TS", "NM", "CE", "CE", "CE",
            "ID", "ID", "CE", "CE", "CE", "CE", "CE", "CWE", "IS", "CWE",
        ],
        "OBX" => &[
            "SI", "ID", "CE", "ST", "VAR", "CE", "ST", "IS", "NM", "ID", "ID", "TS", "ST", "TS",
            "CE", "XCN", "CE", "EI", "TS",
        ],
        "ORC" => &[
            "ID", "EI", "EI", "EI", "ID", "ID", "TQ", "EIP", "TS", "XCN", "XCN", "XCN", "PL",
            "XTN", "TS", "CE", "CE", "CE", "XCN", "CE", "XON", "XAD", "XTN", "XAD", "CWE", "CWE",
            "CWE", "CWE", "CWE", "CNE", "CWE",
        ],
        "PD1" => &[
            "IS", "IS", "XON", "XCN", "IS", "IS", "IS", "IS", "ID", "CX", "CE", "ID", "DT", "XON",
            "CE", "IS", "DT", "CX", "IS", "TS", "IS",
        ],
        "PID" => &[
            "SI", "CX", "CX", "CX", "XPN", "XPN", "TS", "IS", "XPN", "CE", "XAD", "IS", "XTN",
            "XTN", "CE", "CE", "CE", "CX", "ST", "DLN", "CX", "CE", "ST", "ID", "NM", "CE", "CE",
            "CE", "TS", "ID", "ID", "ID", "TS", "HD", "CE", "CE", "CE", "ST", "CE",
        ],
        "PR1" => &[
            "SI", "IS", "CE", "ST", "TS", "IS", "NM", "IS", "NM", "XCN", "XCN", "XCN", "CE", "NM",
            "CE", "CE", "IS", "IS", "EI", "CE",
        ],
        "PV1" => &[
            "SI", "IS", "PL", "IS", "CX", "PL", "XCN", "XCN", "XCN", "IS", "PL", "IS", "IS", "IS",
            "IS", "IS", "XCN", "IS", "CX", "FC", "IS", "IS", "IS", "IS", "DT", "NM", "NM", "IS",
            "IS", "DT", "IS", "NM", "NM", "IS", "DT", "IS", "DLD", "CE", "IS", "IS", "IS", "PL",
            "PL", "TS", "TS", "NM", "NM", "NM", "NM", "CX", "IS", "XCN",
        ],
        "PV2" => &[
            "PL", "CE", "CE", "CE", "ST", "ST", "IS", "TS", "TS", "NM", "NM", "ST", "XCN", "DT",
            "ID", "IS", "DT", "IS", "ID", "NM", "IS", "ID", "XON", "IS", "IS", "DT", "IS", "TS",
            "DT", "CE", "IS", "ID", "TS", "ID", "ID", "ID", "ID", "CE", "CE", "CE", "IS", "IS",
            "IS", "TS", "XCN", "TS", "TS", "TS", "TS",
        ],
        "ROL" => &[
            "EI", "ID", "CE", "XCN", "TS", "TS", "CE", "CE", "CE", "XAD", "XTN",
        ],
        "SFT" => &["XON", "ST", "ST", "ST", "TX", "TS"],
        "SPM" => &[
            "SI", "EIP", "EIP", "CWE", "CWE", "CWE", "CWE", "CWE", "CWE", "CWE", "CWE", "CQ", "NM",
            "ST", "CWE", "CWE", "DR", "TS", "TS", "ID", "CWE", "CQ", "CWE", "CWE", "CWE", "NM",
            "CWE",
        ],
        _ => return None,
    };
    Some(v)
}

/// Data type of SEG-`index` (1-based), if the segment is in the table.
pub fn field_type(seg: &str, index: usize) -> Option<&'static str> {
    segment_fields(seg)?.get(index.checked_sub(1)?).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_up_field_types() {
        assert_eq!(field_type("PID", 5), Some("XPN"));
        assert_eq!(field_type("MSH", 9), Some("MSG"));
        assert_eq!(field_type("OBX", 5), Some("VAR"));
        assert_eq!(field_type("PID", 999), None);
        assert_eq!(field_type("ZZZ", 1), None);
    }

    #[test]
    fn looks_up_composites() {
        assert_eq!(composite_components("XPN").map(|c| c[0]), Some("FN"));
        assert_eq!(composite_components("ST"), None);
        assert_eq!(composite_components("DTM"), None);
    }
}
