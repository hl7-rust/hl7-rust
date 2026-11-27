//! The behaviours the callers depend on, stated once.
//!
//! Each case names the kind of document that motivated it, because the
//! whole argument for this crate is that three readers wanted the same
//! subset and only differed in which half of it they threw away.

// RCA-EXC-5: test code may use plain operators, `unwrap` and `expect`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::integer_division,
    clippy::default_numeric_fallback,
    reason = "RCA-EXC-5: test code may use plain operators, unwrap and expect"
)]

use hl7_2_xml_lite_helper::{Element, Error, escape, local_name, parse};

// --------------------------------------------------------------------
// Elements, attributes, text, nesting
// --------------------------------------------------------------------

#[test]
fn an_element_carries_all_four_things() {
    let root = parse(r#"<a id="1"><b>text</b></a>"#).unwrap();
    assert_eq!(root.name, "a");
    assert_eq!(root.attribute("id"), Some("1"));
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.child("b").unwrap().text, "text");
}

#[test]
fn a_self_closing_element_is_empty_not_absent() {
    let root = parse("<a><b/></a>").unwrap();
    let b = root.child("b").unwrap();
    assert_eq!(b.text, "");
    assert_eq!(b.text_opt(), None);
    assert!(b.children.is_empty());
}

#[test]
fn text_is_kept_where_it_is_content_and_dropped_where_it_is_layout() {
    // Text with nothing but a leaf around it is content, even all spaces...
    let root = parse("<a>   </a>").unwrap();
    assert_eq!(root.text, "   ");
    assert_eq!(root.text_opt(), Some("   "));
    // ...mixed content keeps its text beside the children...
    let root = parse("<a>hello<b/></a>").unwrap();
    assert_eq!(root.text, "hello");
    assert_eq!(root.text_opt(), Some("hello"));
    // ...and whitespace that only separates children is layout.
    let root = parse("<a>\n  <b/>\n  <c/>\n</a>").unwrap();
    assert_eq!(root.text, "");
}

#[test]
fn many_sibling_elements_do_not_count_as_nesting() {
    // The depth comes back down after each child, so 300 siblings (more than
    // the 256-level nesting limit) are fine, and 300 nested are not.
    let siblings = format!("<a>{}</a>", "<b/>".repeat(300));
    assert_eq!(parse(&siblings).unwrap().children.len(), 300);
    let nested = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
    assert!(parse(&nested).is_err());
}

#[test]
fn attributes_are_read_in_both_quote_styles_and_decoded() {
    let root = parse(r#"<a one="x &amp; y" two='z &#65;'/>"#).unwrap();
    assert_eq!(root.attribute("one"), Some("x & y"));
    assert_eq!(root.attribute("two"), Some("z A"));
}

// --------------------------------------------------------------------
// Namespace prefixes (the SOAP case)
// --------------------------------------------------------------------

#[test]
fn any_prefix_reads_the_same() {
    for prefix in ["soapenv:", "soap:", "SOAP-ENV:", ""] {
        let xml = format!("<{prefix}Envelope><{prefix}Body>x</{prefix}Body></{prefix}Envelope>");
        let root = parse(&xml).unwrap();
        assert_eq!(root.local_name(), "Envelope");
        assert_eq!(root.child("Body").unwrap().text, "x", "prefix {prefix:?}");
    }
}

#[test]
fn a_prefixed_attribute_is_found_by_its_local_name() {
    let root = parse(r#"<xsd:element xsd:name="MSH.1" type="ST"/>"#).unwrap();
    assert_eq!(root.attribute("name"), Some("MSH.1"));
    assert_eq!(root.attribute("type"), Some("ST"));
    assert_eq!(local_name("xsd:element"), "element");
    assert_eq!(local_name("element"), "element");
}

// --------------------------------------------------------------------
// Whitespace (the XML Schema case: indented documents)
// --------------------------------------------------------------------

#[test]
fn indentation_beside_children_is_not_content() {
    let root = parse("<a>\n  <b>x</b>\n  <c/>\n</a>").unwrap();
    assert_eq!(root.text, "", "indentation is layout");
    assert_eq!(root.child("b").unwrap().text, "x");
}

#[test]
fn text_in_a_leaf_is_kept_exactly() {
    // An HL7 field value can carry meaningful leading or trailing spaces.
    let root = parse("<a>  spaced  </a>").unwrap();
    assert_eq!(root.text, "  spaced  ");
}

// --------------------------------------------------------------------
// Searching (the HL7 v2.xml case: repeating fields)
// --------------------------------------------------------------------

#[test]
fn text_at_follows_every_branch_not_only_the_first() {
    let root = parse(
        "<PID><PID.3><CX.1>a</CX.1></PID.3><PID.3><CX.4><HD.1>NHS</HD.1></CX.4></PID.3></PID>",
    )
    .unwrap();
    assert_eq!(root.text_at(&["PID.3", "CX.4", "HD.1"]), Some("NHS"));
    assert_eq!(root.text_at(&["PID.3", "CX.1"]), Some("a"));
    assert_eq!(root.text_at(&["PID.3", "CX.9"]), None);
    assert_eq!(root.text_at(&["NOPE"]), None);
}

#[test]
fn children_named_returns_every_match_and_child_the_first() {
    let root = parse("<a><b>1</b><c/><b>2</b></a>").unwrap();
    let texts: Vec<&str> = root.children_named("b").map(|e| e.text.as_str()).collect();
    assert_eq!(texts, ["1", "2"]);
    assert_eq!(root.child("b").unwrap().text, "1");
}

#[test]
fn find_reaches_a_descendant_and_includes_self() {
    let root =
        parse("<Envelope><Body><Fault><faultcode>C</faultcode></Fault></Body></Envelope>").unwrap();
    assert_eq!(root.find("faultcode").unwrap().text, "C");
    assert_eq!(root.find("Envelope").unwrap().local_name(), "Envelope");
    assert!(root.find("nope").is_none());
}

// --------------------------------------------------------------------
// What is skipped, and what is not
// --------------------------------------------------------------------

#[test]
fn the_prolog_and_comments_are_skipped_wherever_they_appear() {
    let root = parse(
        r#"<?xml version="1.0"?>
           <!DOCTYPE a SYSTEM "a.dtd">
           <!-- leading -->
           <a><!-- inside --><b>x</b><?pi here?></a>
           <!-- trailing -->"#,
    )
    .unwrap();
    assert_eq!(root.local_name(), "a");
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.child("b").unwrap().text, "x");
}

#[test]
fn cdata_content_is_kept_as_text() {
    let root = parse("<a><![CDATA[MSH|^~\\&|<not markup>]]></a>").unwrap();
    assert_eq!(root.text, "MSH|^~\\&|<not markup>");
}

#[test]
fn a_byte_order_mark_does_not_stop_it() {
    assert_eq!(parse("\u{feff}<a/>").unwrap().local_name(), "a");
}

#[test]
fn an_unknown_entity_is_kept_rather_than_rejected() {
    let root = parse("<a>1 &lt; 2 &amp;&amp; 3 &#x41; &nope;</a>").unwrap();
    assert_eq!(root.text, "1 < 2 && 3 A &nope;");
}

// --------------------------------------------------------------------
// Errors
// --------------------------------------------------------------------

#[test]
fn it_says_what_is_wrong_and_where() {
    assert_eq!(parse("   "), Err(Error::NoRootElement));
    assert_eq!(parse(""), Err(Error::NoRootElement));
    assert!(matches!(parse("<a>"), Err(Error::Unclosed(name)) if name == "a"));
    assert_eq!(
        parse("<a></b>"),
        Err(Error::Mismatched {
            open: "a".into(),
            close: "b".into()
        })
    );
    assert!(matches!(parse("<a b/>"), Err(Error::Malformed(_, _))));
    assert!(matches!(parse("<a b=c/>"), Err(Error::Malformed(_, _))));
    assert!(
        parse("<a>")
            .unwrap_err()
            .to_string()
            .contains("never closed")
    );

    assert_eq!(Error::NoRootElement.to_string(), "no root element found");
    assert_eq!(
        Error::Mismatched {
            open: "a".into(),
            close: "b".into(),
        }
        .to_string(),
        "<a> is closed by </b>"
    );
    assert_eq!(
        Error::Malformed("bad input".into(), 7).to_string(),
        "bad input at byte 7"
    );
}

#[test]
fn every_malformed_xml_route_is_a_recoverable_error() {
    for xml in [
        "<?never closed",
        "<a",
        "<>",
        "<a ",
        "<a attribute",
        "<a attribute >",
        "<a attribute='never closed",
        "<a></a ",
        "<a><![CDATA[never closed",
    ] {
        assert!(matches!(parse(xml), Err(Error::Malformed(..))), "{xml:?}");
    }

    let root = parse("<a>&not-an-entity-without-a-semicolon</a>").unwrap();
    assert_eq!(root.text, "&not-an-entity-without-a-semicolon");
}

// --------------------------------------------------------------------
// Writing
// --------------------------------------------------------------------

#[test]
fn escape_covers_all_five_and_round_trips() {
    let raw = r#"a&b<c>d"e'f"#;
    assert_eq!(escape(raw), "a&amp;b&lt;c&gt;d&quot;e&apos;f");
    let root = parse(&format!("<a>{}</a>", escape(raw))).unwrap();
    assert_eq!(root.text, raw);
    let root = parse(&format!(r#"<a v="{}"/>"#, escape(raw))).unwrap();
    assert_eq!(root.attribute("v"), Some(raw));
}

#[test]
fn a_default_element_is_usable() {
    let element = Element::default();
    assert_eq!(element.local_name(), "");
    assert_eq!(element.text_opt(), None);
    assert!(element.child("x").is_none());
}

/// Reading is recursive, so nesting depth is stack depth. Without a limit a
/// small document of nothing but open tags aborted the process with a stack
/// overflow — a crash, not an error, which a library must never hand its
/// caller. Past the limit it is an ordinary `Malformed` error.
#[test]
fn nesting_past_the_limit_is_an_error_not_a_stack_overflow() {
    let deep = format!("{}{}", "<a>".repeat(20_000), "</a>".repeat(20_000));
    assert!(matches!(
        hl7_2_xml_lite_helper::parse(&deep),
        Err(hl7_2_xml_lite_helper::Error::Malformed(..))
    ));
}

/// The limit is far above anything the documents this crate is for reach:
/// a v2.xml message needs six levels, a SOAP envelope about the same.
#[test]
fn ordinary_nesting_depth_still_reads() {
    let depth = 64;
    let xml = format!("{}x{}", "<a>".repeat(depth), "</a>".repeat(depth));
    let root = hl7_2_xml_lite_helper::parse(&xml).unwrap();
    let mut node = &root;
    let mut seen = 1;
    while let Some(kid) = node.children.first() {
        node = kid;
        seen += 1;
    }
    assert_eq!(seen, depth);
    assert_eq!(node.text, "x");
}

#[test]
fn empty_and_entity_only_attribute_values_survive_the_suffix_cursor() {
    let root = hl7_2_xml_lite_helper::parse(
        "<a x=\"\" y='&amp;' z = \"&#x41;&#66;\"><![CDATA[<raw>]]>&unknown;&amp</a>",
    )
    .unwrap();
    assert_eq!(root.attribute("x"), Some(""));
    assert_eq!(root.attribute("y"), Some("&"));
    assert_eq!(root.attribute("z"), Some("AB"));
    assert_eq!(root.text, "<raw>&unknown;&amp");
}

#[test]
fn errors_still_report_the_byte_offset_they_stopped_at() {
    let error = hl7_2_xml_lite_helper::parse("<a b=c/>").unwrap_err();
    assert_eq!(error.to_string(), "unquoted attribute value at byte 5");
}
