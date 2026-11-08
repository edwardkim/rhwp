//! 관계형·고정형. 주석 갱신: 2026-11-06T10:59:55+09:00.
//! 측정일: 2026-11-06. 원본 70833 HWP, 독립 한컴2020 Print PDF, 96DPI/PDFium/2px raw.
//! 전18쪽 Native 최저94.62153%, fresh WASM 최저94.27517%, 누락0, 양쪽90% 미달0.
//! 핵심 쪽 N/W: 4쪽97.90408/97.99974%, 5쪽95.35356/95.36648%,
//! 6쪽94.62153/94.82126%, 7쪽94.79966/94.27517%, 10쪽99.57724/99.47384%.
//! 4~7쪽은 원본 열 폭·내용 소속을 비교하는 관계형이며 절대 px 위치를 고정하지 않는다.
//! 10쪽은 양쪽98% 이상·확대 판독·독립 PDF 벡터 경계를 확보해 ±2px로 고정한다.
//! 정식 근거: mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/README.md.
//! 기존 #6854 쪽수·고아 쪽 검사와 #7418의 정상12·13쪽 검사는 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp";

fn open() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("70833 원본 읽기"))
        .expect("70833 원본 파싱")
}

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn text(root: &RenderNode) -> String {
    nodes(root)
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn source_tables(paragraphs: &[Paragraph]) -> Vec<&Table> {
    let mut out: Vec<&Table> = Vec::new();
    for paragraph in paragraphs {
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                out.push(table);
                for cell in &table.cells {
                    out.extend(source_tables(&cell.paragraphs));
                }
            }
        }
    }
    out
}

#[test]
fn stored_blank_row_keeps_its_visible_line_on_the_opening_page() {
    let core = open();
    let opening = core.build_page_render_tree(3).expect("4쪽");
    let continuation = core.build_page_render_tree(4).expect("5쪽");
    assert_eq!(
        text(&opening.root).matches("<신설>").count(),
        1,
        "원본 큰 빈 행의 유일한 글줄은 독립 Print와 같이 첫 물리 쪽에 남는다"
    );
    assert!(
        !text(&continuation.root).contains("<신설>"),
        "빈 꼬리 조각이 첫 글줄을 다음 쪽으로 옮기거나 복제하면 안 된다"
    );
}

#[test]
fn closed_opening_rows_leave_both_omitted_rows_on_the_continuation_page() {
    let core = open();
    let tree = core.build_page_render_tree(5).expect("6쪽");
    for shape in [(11, 3), (14, 5)] {
        let tables: Vec<_> = nodes(&tree.root)
            .into_iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                if (table.row_count, table.col_count) == shape)
            })
            .collect();
        assert_eq!(
            tables.len(),
            1,
            "현행·개정안 계속 표 {shape:?}의 소속은 유일하다"
        );
        assert!(
            text(tables[0]).starts_with("(생략)"),
            "원본 개체 프레임이 닫은 앞 네 행 다음의 (생략) 행부터 계속해야 한다"
        );
    }
}

#[test]
fn missing_line_cache_does_not_resize_the_original_column_grid_in_paint() {
    let core = open();
    let source = source_tables(&core.document().sections[0].paragraphs[15..16]);
    let tree = core.build_page_render_tree(6).expect("7쪽");
    for shape in [(9, 4), (13, 4)] {
        let originals: Vec<_> = source
            .iter()
            .filter(|table| (table.row_count, table.col_count) == shape)
            .collect();
        assert_eq!(originals.len(), 1, "원본 별표4의 표 {shape:?}");
        let original = originals[0];
        let painted: Vec<_> = nodes(&tree.root)
            .into_iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                if (table.row_count, table.col_count) == shape)
            })
            .collect();
        assert_eq!(painted.len(), 1, "다음 쪽에서 완결되는 원본 표 {shape:?}");
        let source_width = f64::from(original.common.width) * 96.0 / 7200.0;
        assert!(
            (painted[0].bbox.width - source_width).abs() < 0.01,
            "원본 열 합과 최종 표 폭은 같아야 한다: source={source_width}, paint={}",
            painted[0].bbox.width
        );
    }
}

#[test]
fn independent_print_page10_table_keeps_its_final_position() {
    let core = open();
    let tree = core.build_page_render_tree(9).expect("10쪽");
    let tables: Vec<_> = nodes(&tree.root)
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.row_count == 3 && table.col_count == 3)
        })
        .collect();
    assert_eq!(tables.len(), 1, "독립 Print 10쪽 영향평가 표");
    // 같은 입력의 한컴 Print 벡터: 좌측77.39867, 상단439.83866px@96DPI.
    // 원본 그리드 관계 검사의 계산값을 픽셀 정답으로 재사용하지 않는다.
    for (actual, expected, label) in [
        (tables[0].bbox.x, 77.39867, "왼쪽"),
        (tables[0].bbox.y, 439.83866, "위쪽"),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print 영향평가 표 {label}: 실제{actual}, 기준{expected}, 허용2px"
        );
    }
}
