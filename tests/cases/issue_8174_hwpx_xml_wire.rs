//! XML 1.0 filtering, escaping and quick-xml event formatting at the public boundary.
use quick_xml::Writer;
use rhwp::serializer::hwpx::utils::{
    empty_tag, end_tag, start_tag, start_tag_attrs, text, write_xml_decl, xml_escape,
};

#[test]
fn xml_markup_and_invalid_characters_keep_the_public_wire_contract() {
    let value = "\u{0}한\u{3}&<>\"'\t\n\r😀\u{fffe}\u{ffff}";
    let escaped = "한&amp;&lt;&gt;&quot;&apos;\t\n\r😀";
    assert_eq!(xml_escape(value), escaped);
    let attribute = "한&amp;&lt;&gt;&quot;&apos;&#9;&#10;&#13;😀";
    let node = "한&amp;&lt;&gt;&quot;&apos;\t\n&#13;😀";
    let mut writer = Writer::new(Vec::new());
    start_tag_attrs(&mut writer, "hp:a", &[("v", value), ("empty", "")]).unwrap();
    text(&mut writer, value).unwrap();
    end_tag(&mut writer, "hp:a").unwrap();
    empty_tag(&mut writer, "hp:b", &[("v", value)]).unwrap();
    assert_eq!(
        String::from_utf8(writer.into_inner()).unwrap(),
        format!("<hp:a v=\"{attribute}\" empty=\"\">{node}</hp:a><hp:b v=\"{attribute}\"/>")
    );
}

#[test]
fn xml_events_preserve_writer_indentation_and_empty_text_behavior() {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    write_xml_decl(&mut writer).unwrap();
    start_tag_attrs(&mut writer, "root", &[("v", "&")]).unwrap();
    start_tag(&mut writer, "child").unwrap();
    text(&mut writer, "<\"'>").unwrap();
    end_tag(&mut writer, "child").unwrap();
    empty_tag(&mut writer, "empty", &[]).unwrap();
    start_tag(&mut writer, "text").unwrap();
    text(&mut writer, "").unwrap();
    end_tag(&mut writer, "text").unwrap();
    end_tag(&mut writer, "root").unwrap();
    assert_eq!(String::from_utf8(writer.into_inner()).unwrap(),
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<root v=\"&amp;\">\n  <child>&lt;&quot;&apos;&gt;</child>\n  <empty/>\n  <text></text>\n</root>"
    );
}
