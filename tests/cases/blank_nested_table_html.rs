//! 고정형: blank_nested_table_html의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-03T21:22:49+09:00; 직전 파일 변경 커밋: 2026-10-26T00:03:49+09:00 d5505a2690a0db46c9e20a1f6236b702e1796fa3.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. 시각 측정 비해당: HTML pt 입력에서 모델 HU·빈 중첩 표 보존과 제한 깊이를 검사한다. 페이지 배치·절대 px를 고정하지 않는다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/visual-results.json 및 전쪽 TSV·확대 PNG.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

fn tables(paragraphs: &[Paragraph]) -> usize {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .map(|control| match control {
            Control::Table(table) => {
                1 + table
                    .cells
                    .iter()
                    .map(|cell| tables(&cell.paragraphs))
                    .sum::<usize>()
            }
            _ => 0,
        })
        .sum()
}

fn first_table(paragraphs: &[Paragraph]) -> &rhwp::model::table::Table {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .unwrap()
}
fn assert_sizes(core: &DocumentCore) {
    let outer = first_table(&core.document().sections[0].paragraphs);
    assert_eq!((outer.common.width, outer.common.height), (30000, 20000));
    assert_eq!(outer.row_count, 1);
    assert_eq!(outer.cells[0].height, 20000);
    let inner = first_table(&outer.cells[0].paragraphs);
    assert_eq!((inner.common.width, inner.common.height), (15000, 7000));
}

#[test]
fn blank_nested_table_survives_html_paste_and_both_save_formats() {
    let html = "<table style=\"width:300pt;height:200pt\"><tr><td style=\"width:300pt;height:200pt\"><table style=\"width:150pt;height:70pt\"><tr><td style=\"width:150pt;height:70pt\"><p></p></td></tr></table></td></tr></table>";
    let multiple_rows = html.replace("height:70pt\"><p></p></td></tr>", "height:30pt\"><p></p></td></tr><tr><td style=\"width:150pt;height:40pt\"><p></p></td></tr>");
    assert_ne!(multiple_rows, html);
    for html in [html.to_string(), multiple_rows] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.paste_html_native(0, 0, 0, &html).unwrap();
        assert_eq!(tables(&core.document().sections[0].paragraphs), 2);
        assert_sizes(&core);
        for bytes in [
            core.export_hwp_with_adapter().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(tables(&reopened.document().sections[0].paragraphs), 2);
            assert_sizes(&reopened);
        }
    }
}

#[test]
fn nested_table_recursion_keeps_the_existing_html_depth_limit() {
    let mut html = "바닥 글".to_string();
    for _ in 0..40 {
        html = format!("<table><tr><td>{html}</td></tr></table>");
    }
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.paste_html_native(0, 0, 0, &html).unwrap();
    assert!(tables(&core.document().sections[0].paragraphs) <= 16);
    assert!(format!("{:?}", core.document()).contains("바닥 글"));
}
