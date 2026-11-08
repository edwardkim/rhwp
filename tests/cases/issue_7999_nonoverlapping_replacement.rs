//! Literal replacement preserves text outside non-overlapping source matches.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

fn document(text: &str) -> DocumentCore {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, text).unwrap();
    doc
}

#[test]
fn punctuation_replacement_preserves_following_words_after_save() {
    for (before, query, replacement, expected, count) in [
        ("검토.... 다음 안건", "...", "…", "검토…. 다음 안건", 1),
        ("검토......완료", "...", "…", "검토……완료", 2),
        ("보류---재검토", "--", "—", "보류—-재검토", 1),
        ("AAAA끝", "AA", "ZZZ", "ZZZZZZ끝", 2),
        ("검토...완료...제출", "...", "…", "검토…완료…제출", 2),
    ] {
        let mut doc = document(before);
        let result: Value =
            serde_json::from_str(&doc.replace_all_native(query, replacement, true).unwrap())
                .unwrap();
        assert_eq!(result["count"], count, "{before}");
        assert_eq!(doc.document().sections[0].paragraphs[0].text, expected);
        let bytes = doc.export_hwp_with_adapter().unwrap();
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, expected);
    }
}

#[test]
fn search_grep_and_nth_replacement_share_nonoverlapping_addresses() {
    for (before, query, sensitive, offsets) in [
        ("AAAA끝", "AA", true, vec![0, 2]),
        ("AaAa끝", "aa", false, vec![0, 2]),
        ("İİİ끝", "i\u{307}i\u{307}", false, vec![0]),
        ("İİİ끝", "\u{307}i", false, vec![0]),
    ] {
        let mut doc = document(before);
        let hits: Vec<Value> =
            serde_json::from_str(&doc.search_all_text_native(query, sensitive, true).unwrap())
                .unwrap();
        assert_eq!(
            hits.iter()
                .map(|h| h["charOffset"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            offsets
        );
        assert_eq!(doc.grep(query, sensitive, None).len(), offsets.len());
        doc.replace_nth_native(query, "Z", sensitive, offsets.len() - 1)
            .unwrap();
        let expected = if sensitive {
            "AAZ끝"
        } else if before.starts_with('İ') {
            "Zİ끝"
        } else {
            "AaZ끝"
        };
        assert_eq!(doc.document().sections[0].paragraphs[0].text, expected);
    }
}

#[test]
fn overlapping_replacements_are_independent_in_nested_text_containers() {
    let hml = r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION>
      <P><TEXT><CHAR>검토......완료</CHAR><TABLE RowCount="1" ColCount="2">
        <SHAPEOBJECT><SIZE Width="8000" Height="1200"/></SHAPEOBJECT><ROW>
        <CELL ColAddr="0" RowAddr="0" Width="4000" Height="1200"><PARALIST>
          <P><TEXT><CHAR>검토......완료</CHAR></TEXT></P>
        </PARALIST></CELL>
        <CELL ColAddr="1" RowAddr="0" Width="4000" Height="1200"><PARALIST>
          <P><TEXT><CHAR>검토......완료</CHAR></TEXT></P>
        </PARALIST></CELL></ROW></TABLE>
        <RECTANGLE X0="0" X1="5000" X2="5000" X3="0" Y0="0" Y1="0" Y2="2400" Y3="2400">
          <SHAPEOBJECT><SIZE Width="5000" Height="2400"/></SHAPEOBJECT>
          <DRAWINGOBJECT><SHAPECOMPONENT XPos="0" YPos="0" OriWidth="5000" OriHeight="2400" CurWidth="5000" CurHeight="2400"/>
            <LINESHAPE Width="0" Style="Solid" EndCap="Flat" Alpha="0"/>
            <DRAWTEXT><PARALIST><P><TEXT><CHAR>검토......완료</CHAR></TEXT></P></PARALIST></DRAWTEXT>
          </DRAWINGOBJECT>
        </RECTANGLE></TEXT></P>
      </SECTION></BODY><TAIL/></HWPML>"#;
    let mut doc = DocumentCore::from_bytes(hml.as_bytes()).unwrap();
    doc.insert_equation_native(0, 0, 0, "......x", 1000, 0)
        .unwrap();
    let result: Value =
        serde_json::from_str(&doc.replace_all_native("...", "…", true).unwrap()).unwrap();
    assert_eq!(result["count"], 10);
    let suffixes = doc.grep("완료", true, None);
    assert_eq!(suffixes.len(), 4);
    assert!(suffixes.iter().all(|hit| hit.text.contains("검토……완료")));
    assert_eq!(doc.grep("……x", true, None).len(), 1);
}
