//! The behaviours the callers depend on, stated once.
//!
//! Each case names the kind of document that motivated it, because the
//! whole argument for this crate is that three readers wanted the same
//! subset and only differed in which half of it they threw away.

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
