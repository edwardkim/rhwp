//! [#3500][#5451] HWPX 왕복이 연속 동일-id `char_shapes` 경계를 접지 않는다.
//!
//! `samples/re-multisize-10-10-empty-hancom.hwp` 원본은
//! `[(0,0),(34,0),(53,0)]` 이다. 세 entry 모두 id 0 이라 렌더는 같지만,
//! PARA_CHAR_SHAPE start_pos 는 IR 비교 대상이다. 직렬화가 run 을 하나으로
//! 접으면 재파싱이 `[(0,0)]` 만 남긴다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::style::{CharShape, UnderlineType};
use rhwp::parse_document;
use rhwp::serializer::hwpx::char_shape_tables::{
    attr_field, attr_flag, emphasis_id, issue_3500_row, lang_index, line_shape_id, line_shape_str,
    outline_type_id, outline_type_str, set_attr_field, set_attr_flag, shadow_type_id,
    shadow_type_str, strike_shape_str, sym_mark_str, underline_type_from_bits,
    underline_type_from_hwpx, underline_type_str, underline_type_to_bits, ATTR_BIT_BOLD,
    ATTR_BIT_ITALIC, ATTR_BIT_OUTLINE, ATTR_WIDTH_OUTLINE, BASE_SIZE_UNITS_PER_PT,
    CHAR_PR_TOKEN_CASES, CHAR_SHAPE_MIN_BYTES, EMPHASIS_HWPX, ISSUE_3500_BODY_BASE_SIZE,
    ISSUE_3500_NINE_PT_BASE_SIZE, ISSUE_3500_REFS, ISSUE_3500_SAMPLE, LANG_ATTRS, LANG_SLOTS,
    LINE_SHAPE_HWPX, OFF_ATTR, OFF_BASE_SIZE, OFF_CHAR_OFFSETS, OFF_FONT_IDS, OFF_RATIOS,
    OFF_RELATIVE_SIZES, OFF_SHADOW_X, OFF_SHADOW_Y, OFF_SPACINGS, OFF_STRIKE_COLOR, OFF_TEXT_COLOR,
    OUTLINE_TYPE_HWPX, SAME_ID_PARAS, SHADOW_TYPE_HWPX, SHAPE_TABLES,
};
use rhwp::serializer::hwpx::char_shapes::{
    char_pr_xml, collapse_same_id, plan_run_boundaries, same_id_extra_count,
    xml_preserves_same_id_runs,
};
use rhwp::serializer::hwpx::roundtrip::diff_documents;
use rhwp::serializer::hwpx::serialize_hwpx;

const SAMPLE: &str = "samples/re-multisize-10-10-empty-hancom.hwp";
const SAME_ID_JSONL: &str =
    include_str!("../fixtures/char_shapes/corpus_same_id_para_char_shapes.jsonl");
const ISSUE_DUMP: &str = include_str!("../fixtures/char_shapes/issue_3500_re_multisize.json");

fn refs_of(para: &rhwp::model::paragraph::Paragraph) -> Vec<(u32, u32)> {
    para.char_shapes
        .iter()
        .map(|cs| (cs.start_pos, cs.char_shape_id))
        .collect()
}

#[test]
fn issue_3500_sample_matches_extracted_ir() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = parse_document(&bytes).expect("parse hwp5");
    let para = &doc.sections[0].paragraphs[0];
    let refs = refs_of(para);
    assert_eq!(refs, ISSUE_3500_REFS);
    assert_eq!(same_id_extra_count(&refs), 2);
    assert_eq!(collapse_same_id(&refs), vec![(0, 0)]);
    let row = issue_3500_row().expect("catalog row");
    assert_eq!(row.file, ISSUE_3500_SAMPLE);
    assert_eq!(row.refs, refs.as_slice());
    assert!(ISSUE_DUMP.contains("re-multisize-10-10-empty-hancom.hwp"));
    assert!(ISSUE_DUMP.contains("[0, 0]"));
    assert!(ISSUE_DUMP.contains("[34, 0]"));
    assert!(ISSUE_DUMP.contains("[53, 0]"));
}

#[test]
fn issue_3500_hwpx_roundtrip_keeps_same_id_boundaries() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let original = parse_document(&bytes).expect("parse hwp5");
    let hwpx = serialize_hwpx(&original).expect("serialize hwpx");
    let roundtripped = parse_document(&hwpx).expect("reparse hwpx");

    let expected = refs_of(&original.sections[0].paragraphs[0]);
    let actual = refs_of(&roundtripped.sections[0].paragraphs[0]);
    assert_eq!(
        expected, actual,
        "HWPX 왕복이 동일-id 경계를 접으면 안 된다 (#3500)"
    );
    assert_eq!(expected, ISSUE_3500_REFS);

    let diffs = diff_documents(&original, &roundtripped);
    let char_shape_diffs: Vec<_> = diffs
        .iter()
        .filter(|d| format!("{d:?}").contains("ParagraphCharShapes"))
        .collect();
    assert!(
        char_shape_diffs.is_empty(),
        "char_shapes IR 차이: {char_shape_diffs:?}"
    );
}

#[test]
fn issue_3500_export_hwpx_native_matches_serialize() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let hwpx = core.export_hwpx_native().expect("export");
    let original = parse_document(&bytes).expect("parse");
    let roundtripped = parse_document(&hwpx).expect("reparse");
    assert_eq!(
        refs_of(&original.sections[0].paragraphs[0]),
        refs_of(&roundtripped.sections[0].paragraphs[0])
    );
}

#[test]
fn same_id_jsonl_matches_catalog() {
    let jsonl_rows = SAME_ID_JSONL.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(jsonl_rows, SAME_ID_PARAS.len());
}

#[test]
fn planner_keeps_every_same_id_corpus_row() {
    for row in SAME_ID_PARAS {
        let refs: Vec<rhwp::model::paragraph::CharShapeRef> = row
            .refs
            .iter()
            .map(
                |&(start_pos, char_shape_id)| rhwp::model::paragraph::CharShapeRef {
                    start_pos,
                    char_shape_id,
                },
            )
            .collect();
        assert_eq!(plan_run_boundaries(&refs), row.refs);
    }
}

#[test]
fn shape_catalog_includes_issue_3500_table() {
    let table = SHAPE_TABLES
        .iter()
        .find(|t| t.file == ISSUE_3500_SAMPLE)
        .expect("shape table");
    assert_eq!(table.count, 5);
    assert_eq!(table.shapes.len(), 5);
    assert_eq!(table.shapes[0].base_size, 1000);
    assert_eq!(table.shapes[2].base_size, 900);
    assert_eq!(table.shapes[0].font_ids, [1, 1, 1, 1, 1, 1, 1]);
}

#[test]
fn collapsed_xml_is_rejected_for_issue_3500() {
    assert!(!xml_preserves_same_id_runs(
        r#"<hp:run charPrIDRef="0"><hp:t>가나다</hp:t></hp:run>"#,
        ISSUE_3500_REFS
    ));
    let kept = r#"<hp:run charPrIDRef="0"><hp:t>a</hp:t></hp:run><hp:run charPrIDRef="0"><hp:t>b</hp:t></hp:run><hp:run charPrIDRef="0"><hp:t>c</hp:t></hp:run>"#;
    assert!(xml_preserves_same_id_runs(kept, ISSUE_3500_REFS));
}

#[test]
fn lang_slots_match_attr_array() {
    for (i, slot) in LANG_SLOTS.iter().enumerate() {
        assert_eq!(slot.index as usize, i);
        assert_eq!(slot.attr, LANG_ATTRS[i]);
        assert_eq!(lang_index(slot.attr), Some(i));
    }
}

#[test]
fn mapping_tables_roundtrip() {
    for (id, token) in LINE_SHAPE_HWPX.iter().enumerate() {
        assert_eq!(line_shape_str(id as u8), *token);
        assert_eq!(line_shape_id(token), Some(id as u8));
    }
    assert_eq!(line_shape_str(99), "SOLID");
    assert_eq!(strike_shape_str(false, 3), "NONE");
    for (id, token) in OUTLINE_TYPE_HWPX.iter().enumerate() {
        assert_eq!(outline_type_str(id as u8), *token);
        assert_eq!(outline_type_id(token), id as u8);
    }
    assert_eq!(shadow_type_str(3), "NONE");
    assert_eq!(shadow_type_id("DROP"), 1);
    for (id, token) in EMPHASIS_HWPX.iter().enumerate() {
        assert_eq!(sym_mark_str(id as u8), *token);
    }
    for kind in [
        UnderlineType::None,
        UnderlineType::Bottom,
        UnderlineType::Top,
    ] {
        assert_eq!(underline_type_from_hwpx(underline_type_str(kind)), kind);
        assert_eq!(underline_type_from_bits(underline_type_to_bits(kind)), kind);
    }
}

#[test]
fn attr_and_layout_offsets() {
    let mut attr = 0u32;
    set_attr_flag(&mut attr, ATTR_BIT_BOLD, true);
    set_attr_field(&mut attr, ATTR_BIT_OUTLINE, ATTR_WIDTH_OUTLINE, 5);
    assert!(attr_flag(attr, ATTR_BIT_BOLD));
    assert!(!attr_flag(attr, ATTR_BIT_ITALIC));
    assert_eq!(attr_field(attr, ATTR_BIT_OUTLINE, ATTR_WIDTH_OUTLINE), 5);
    assert_eq!(OFF_RATIOS, OFF_FONT_IDS + 7 * 2);
    assert_eq!(OFF_SPACINGS, OFF_RATIOS + 7);
    assert_eq!(OFF_RELATIVE_SIZES, OFF_SPACINGS + 7);
    assert_eq!(OFF_CHAR_OFFSETS, OFF_RELATIVE_SIZES + 7);
    assert_eq!(OFF_BASE_SIZE, OFF_CHAR_OFFSETS + 7);
    assert_eq!(OFF_ATTR, OFF_BASE_SIZE + 4);
    assert_eq!(OFF_SHADOW_X, OFF_ATTR + 4);
    assert_eq!(OFF_TEXT_COLOR, OFF_SHADOW_Y + 1);
    assert_eq!(OFF_STRIKE_COLOR + 4, CHAR_SHAPE_MIN_BYTES);
    assert_eq!(ISSUE_3500_BODY_BASE_SIZE / BASE_SIZE_UNITS_PER_PT, 10);
    assert_eq!(ISSUE_3500_NINE_PT_BASE_SIZE / BASE_SIZE_UNITS_PER_PT, 9);
}

#[test]
fn write_char_pr_uses_lang_attr_names() {
    let xml = char_pr_xml(0, &CharShape::default()).expect("xml");
    for attr in LANG_ATTRS {
        assert!(xml.contains(attr), "missing {attr} in {xml}");
    }
    assert!(xml.contains(r#"shadeColor="none""#), "{xml}");
    assert!(xml.contains(r#"symMark="NONE""#), "{xml}");
}

#[test]
fn encoding_matrix_tokens_appear_in_char_pr() {
    for case in CHAR_PR_TOKEN_CASES {
        let mut cs = CharShape::default();
        cs.underline_type = match case.underline_type {
            "BOTTOM" => UnderlineType::Bottom,
            "TOP" => UnderlineType::Top,
            _ => UnderlineType::None,
        };
        cs.underline_shape = line_shape_id(case.underline_shape).unwrap_or(0);
        cs.strikethrough = case.strike_on;
        cs.strike_shape = line_shape_id(case.strike_shape).unwrap_or(0);
        cs.outline_type = outline_type_id(case.outline);
        cs.shadow_type = shadow_type_id(case.shadow);
        cs.emphasis_dot = emphasis_id("NONE");
        let xml = char_pr_xml(0, &cs).expect("xml");
        assert!(
            xml.contains(&format!(r#"type="{}""#, case.underline_type)),
            "{xml}"
        );
        assert!(
            xml.contains(&format!(r#"shape="{}""#, case.underline_shape)),
            "{xml}"
        );
        assert!(
            xml.contains(&format!(r#"<hh:strikeout shape="{}""#, case.strike_shape)),
            "{xml}"
        );
        assert!(
            xml.contains(&format!(r#"<hh:outline type="{}""#, case.outline)),
            "{xml}"
        );
        assert!(
            xml.contains(&format!(r#"<hh:shadow type="{}""#, case.shadow)),
            "{xml}"
        );
    }
}
