//! Case-insensitive search and replacement use source character spans.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

#[test]
fn ignore_case_replacement_preserves_textbox_suffix() {
    for (text, query) in [("İX", "i\u{307}"), ("i\u{307}X", "İ")] {
        let hml = format!(
            r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION>
          <P><TEXT><RECTANGLE X0="0" X1="5000" X2="5000" X3="0" Y0="0" Y1="0" Y2="2400" Y3="2400">
            <SHAPEOBJECT><SIZE Width="5000" Height="2400"/></SHAPEOBJECT>
            <DRAWINGOBJECT><SHAPECOMPONENT XPos="0" YPos="0" OriWidth="5000" OriHeight="2400" CurWidth="5000" CurHeight="2400"/>
              <LINESHAPE Width="0" Style="Solid" EndCap="Flat" Alpha="0"/>
              <DRAWTEXT><TEXTMARGIN Left="0" Right="0" Top="0" Bottom="0"/><PARALIST>
                <P><TEXT><CHAR>{text}</CHAR></TEXT></P>
              </PARALIST></DRAWTEXT>
            </DRAWINGOBJECT>
          </RECTANGLE></TEXT></P>
        </SECTION></BODY><TAIL/></HWPML>"#
        );
        let mut doc = DocumentCore::from_bytes(hml.as_bytes()).unwrap();
        doc.replace_all_native(query, "Z", false).unwrap();
        assert_eq!(doc.grep("Z", true, None)[0].text, "ZX");
    }
}

#[test]
fn ignore_case_replacement_preserves_text_after_expanded_match() {
    for (text, query) in [("İX", "i\u{307}"), ("i\u{307}X", "İ")] {
        for operation in ["one", "all", "nth"] {
            let mut doc = DocumentCore::new_empty();
            doc.create_blank_document_native().unwrap();
            doc.insert_text_native(0, 0, 0, text).unwrap();
            match operation {
                "one" => doc.replace_one_native(query, "Z", false).unwrap(),
                "all" => doc.replace_all_native(query, "Z", false).unwrap(),
                _ => doc.replace_nth_native(query, "Z", false, 0).unwrap(),
            };
            assert_eq!(
                doc.grep("Z", true, None)[0].text,
                "ZX",
                "{operation}: {text}"
            );
        }
    }
}

#[test]
fn ignore_case_search_reports_original_match_length() {
    for (text, query, length) in [("İX", "i\u{307}", 1), ("i\u{307}X", "İ", 2)] {
        let mut doc = DocumentCore::new_empty();
        doc.create_blank_document_native().unwrap();
        doc.insert_text_native(0, 0, 0, text).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &doc.search_all_text_native(query, false, true).unwrap()
            )
            .unwrap()[0]["length"]
                .as_u64()
                .unwrap() as usize,
            length
        );
        assert_eq!(doc.grep(query, false, None)[0].length, length);
    }
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "İX İX").unwrap();
    doc.replace_nth_native("i\u{307}", "Z", false, 1).unwrap();
    assert_eq!(doc.grep("Z", true, None)[0].text, "İX ZX");
    doc.replace_all_native("i\u{307}", "Z", true).unwrap();
    assert_eq!(doc.grep("Z", true, None)[0].text, "İX ZX");
}

#[test]
fn ignore_case_replacement_preserves_cell_and_equation_suffixes() {
    for (text, query) in [("İX", "i\u{307}"), ("i\u{307}X", "İ")] {
        let hml = format!(
            r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION>
              <P><TEXT><TABLE RowCount="1" ColCount="1">
                <SHAPEOBJECT><SIZE Width="4000" Height="1200"/></SHAPEOBJECT>
                <ROW><CELL ColAddr="0" RowAddr="0" Width="4000" Height="1200"><PARALIST>
                  <P><TEXT><CHAR>{text}</CHAR></TEXT></P>
                </PARALIST></CELL></ROW>
              </TABLE></TEXT></P>
            </SECTION></BODY><TAIL/></HWPML>"#
        );
        let mut doc = DocumentCore::from_bytes(hml.as_bytes()).unwrap();
        doc.replace_all_native(query, "Z", false).unwrap();
        assert_eq!(doc.grep("Z", true, None)[0].text, "ZX");

        let mut doc = DocumentCore::new_empty();
        doc.create_blank_document_native().unwrap();
        doc.insert_equation_native(0, 0, 0, text, 1000, 0).unwrap();
        doc.replace_all_native(query, "Z", false).unwrap();
        assert_eq!(doc.grep("Z", true, None)[0].text, "ZX");
    }
}
