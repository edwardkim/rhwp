//! [#5177] scaffold 공개 스키마와 HWPX 생성 계약.

use rhwp::document_core::queries::structure::{build_structure, StructureMode};
use rhwp::model::control::Control;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::scaffold::{build_scaffold, parse_scaffold_str, Block, ScaffoldSpec};
use rhwp::serializer::hwpx::roundtrip::roundtrip_ir_diff;
use rhwp::serializer::serialize_hwpx;

fn full_spec() -> ScaffoldSpec {
    parse_scaffold_str(
        r#"{
            "version": "1",
            "title": "2026년 1분기 실적 보고서",
            "font": "함초롬바탕",
            "blocks": [
                {"type": "heading", "level": 1, "text": "1. 개요"},
                {"type": "paragraph", "text": "본 보고서는 자동 생성되었습니다."},
                {"type": "heading", "level": 2, "text": "1.1 매출"},
                {"type": "paragraph", "text": "매출은 전년 대비 증가했습니다."},
                {"type": "table", "rows": [
                    ["항목", "1분기", "2분기"],
                    ["매출", "100", "120"],
                    ["영업이익", "20", "25"]
                ]}
            ]
        }"#,
    )
    .expect("유효한 scaffold 명세")
}

#[test]
fn scaffold_round_trips_to_stable_hwpx() {
    let document = build_scaffold(&full_spec());
    let bytes = serialize_hwpx(&document).expect("HWPX 직렬화");
    assert!(
        bytes.len() > 100,
        "생성 산출물이 비어있다: {}바이트",
        bytes.len()
    );
    let diff = roundtrip_ir_diff(&bytes).expect("왕복 IR diff 계산");
    assert!(
        diff.is_empty(),
        "생성 HWPX 왕복이 안정적이지 않다: {:?}",
        diff.differences
    );
}

#[test]
fn scaffold_preserves_text_outline_and_table() {
    let bytes = serialize_hwpx(&build_scaffold(&full_spec())).expect("HWPX 직렬화");
    let reparsed = parse_hwpx(&bytes).expect("생성 HWPX 재파싱");
    let texts: Vec<String> = reparsed.sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.clone())
        .collect();
    for expected in [
        "2026년 1분기 실적 보고서",
        "본 보고서는 자동 생성되었습니다.",
        "매출은 전년 대비 증가했습니다.",
    ] {
        assert!(
            texts.iter().any(|text| text == expected),
            "{expected} 미복원: {texts:?}"
        );
    }

    let structure = build_structure(&reparsed, StructureMode::Outline);
    assert_eq!(structure.roots.len(), 1, "구조: {structure:?}");
    assert_eq!(structure.roots[0].level, 1);
    assert_eq!(structure.roots[0].heading, "1. 개요");
    assert_eq!(structure.roots[0].children.len(), 1);
    assert_eq!(structure.roots[0].children[0].level, 2);
    assert_eq!(structure.roots[0].children[0].heading, "1.1 매출");

    let table = reparsed.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표 컨트롤");
    assert_eq!((table.row_count, table.col_count), (3, 3));
    let cell_text = |row: u16, col: u16| -> String {
        table
            .cells
            .iter()
            .find(|cell| cell.row == row && cell.col == col)
            .map(|cell| {
                cell.paragraphs
                    .iter()
                    .map(|paragraph| paragraph.text.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default()
    };
    assert_eq!(cell_text(0, 0), "항목");
    assert_eq!(cell_text(0, 2), "2분기");
    assert_eq!(cell_text(1, 0), "매출");
    assert_eq!(cell_text(2, 2), "25");
}

#[test]
fn scaffold_defaults_version_validation_and_heading_clamp() {
    let empty = parse_scaffold_str(r#"{"version":"1","blocks":[]}"#).expect("기본 명세");
    assert_eq!(empty.font, "함초롬바탕");
    assert_eq!(empty.page_size.width_mm, 210.0);
    assert!(empty.title.is_none());
    let document = build_scaffold(&empty);
    assert_eq!(document.sections.len(), 1);
    assert!(!document.sections[0].paragraphs.is_empty());
    let bytes = serialize_hwpx(&document).expect("빈 명세 직렬화");
    assert!(roundtrip_ir_diff(&bytes).expect("왕복 검증").is_empty());

    let error = parse_scaffold_str(r#"{"version":"2","blocks":[]}"#).unwrap_err();
    assert!(format!("{error}").contains("스키마 버전"), "{error}");

    let clamped = parse_scaffold_str(
        r#"{"version":"1","blocks":[{"type":"heading","level":99,"text":"깊은 제목"}]}"#,
    )
    .expect("명세 파싱");
    let bytes = serialize_hwpx(&build_scaffold(&clamped)).expect("HWPX 직렬화");
    let structure = build_structure(&parse_hwpx(&bytes).expect("재파싱"), StructureMode::Outline);
    assert_eq!(structure.roots[0].level, 7, "구조: {structure:?}");
}

#[test]
fn scaffold_schema_rejects_invalid_blocks() {
    let sample: ScaffoldSpec = serde_json::from_str(
        r#"{"version":"1","blocks":[{"type":"heading","level":1,"text":"개요"},{"type":"paragraph","text":"본문"},{"type":"table","rows":[["항목","값"]]}]}"#,
    )
    .expect("샘플 스키마");
    assert_eq!(sample.blocks.len(), 3);
    assert!(matches!(sample.blocks[0], Block::Heading { level: 1, .. }));
    assert!(matches!(sample.blocks[1], Block::Paragraph { .. }));
    assert!(matches!(sample.blocks[2], Block::Table { .. }));

    for (json, expected) in [
        (
            r#"{"type":"paragraph","text":"a","rows":[["x"]]}"#,
            "paragraph 블록에 허용되지 않는 필드 'rows'",
        ),
        (r#"{"type":"heading","text":"제목"}"#, "'level'"),
        (
            r#"{"type":"image","text":"x"}"#,
            "알 수 없는 블록 type 'image'",
        ),
        (r#"{"type":"paragraph","text":"a","bold":true}"#, "bold"),
    ] {
        let error = serde_json::from_str::<Block>(json).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
    let typo = serde_json::from_str::<ScaffoldSpec>(r#"{"version":"1","fnt":"바탕","blocks":[]}"#)
        .unwrap_err()
        .to_string();
    assert!(typo.contains("fnt"), "{typo}");
}
