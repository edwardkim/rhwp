//! 관계형·고정형: #7727 CBTA 원문의 표 소유·내용 보존과 독립 Print 쪽수·±2px 경계.
//! 주석 갱신: 2026-11-06T08:29:51+09:00; 직전 파일 커밋: 2026-09-05T20:44:19+09:00 (902a208b51).
//! 2026-11-06 측정: canonical 96DPI·compat2022·PDFium·2px raw, 원본 전204쪽.
//! 전쪽 최저 Native 92.05232% / fresh WASM 92.16419%(11쪽), 90% 미만·쪽 누락 0.
//! px 대상 186쪽 N/W 99.93606/99.93833%, 187쪽 99.93137/99.93362%.
//! 185쪽의 29×3 표 N/W 99.92888/99.93373%; 이 표와 후속 46×3 표의 원문155셀을 대조한다.
//! 고정 base 170b886f / code head fc378076: 전204쪽 각 backend의 base/후보 PNG 동일, raw 하락0.
//! 근거: mydocs/pr/assets/issue7727/verification-20261106/README.md.
//! PDF: pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf, SHA-256 0db1eb55eda0f1a4985eeaf187656c23499616b7dccddcbb7c874c0937e1b2e0.
//! 원 해결 d87b3037e5·29194a603f·c547a52b2a는 모두 base 조상이다. renderer는 이번에 바꾸지 않는다.
//! 기존 전쪽 최대 초과<100px는 다른 표도 섞은 임시 값이었다. 원문 소속·실제 용지·내용 보존으로 교정한다.
//! 과거 200/201/202쪽 관측을 고정하지 않고 같은 원문의 독립 한컴2020 Print 204쪽을 사용한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

mod deferred_scope {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/deferred_visual_regression_scope.rs"
    ));
}

const SAMPLE: &str = "samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp";

fn open_sample() -> DocumentCore {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("CBTA 원문 읽기");
    DocumentCore::from_bytes(&bytes).expect("CBTA 원문 파싱")
}

fn table(node: &RenderNode, paragraph: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t)
        if t.section_index == Some(2) && t.para_index == Some(paragraph)
            && t.control_index == Some(0) && t.cell_context.is_none())
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table(child, paragraph))
}

fn text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        text(child, out);
    }
}

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn assert_on_paper(node: &RenderNode, paper: &RenderNode) {
    let bottom = paper.bbox.y + paper.bbox.height;
    assert!(node.bbox.y >= paper.bbox.y && node.bbox.y + node.bbox.height <= bottom);
    if let RenderNodeType::TextRun(run) = &node.node_type {
        assert!(
            node.bbox.y + run.baseline <= bottom,
            "표 안의 글자도 실제 용지 안에 그린다"
        );
    }
    for child in &node.children {
        assert_on_paper(child, paper);
    }
}

#[test]
fn para_float_band_keeps_the_next_table_fragment_on_paper() {
    for input in [
        SAMPLE,
        "mydocs/pr/assets/issue7445/1613000-202200037-air-traffic-controller-cbta.hwp",
    ] {
        for axis in [
            "body",
            "off_canvas",
            "oracle",
            "overflow_cell",
            "render_page",
            "text_overlap",
        ] {
            assert!(
                !deferred_scope::is_deferred(Path::new(input), axis),
                "전쪽 수용한 CBTA 원문은 기존 {axis} 코퍼스에서도 검사해야 한다: {input}"
            );
        }
    }
    let core = open_sample();
    assert_eq!(core.page_count(), 204, "독립 한컴2020 Print의 전체 쪽수");
    let pages: Vec<_> = (0..core.page_count())
        .map(|p| core.build_page_render_tree(p as u32).expect("전쪽 트리"))
        .collect();
    // 원문 소속으로 선택한다. landscape 182쪽의 다른 247×7 표는 대상이 아니다.
    for (paragraph, rows, expected_pages) in [(1, 29, [185, 186]), (4, 46, [186, 187])] {
        let source = match &core.document().sections[2].paragraphs[paragraph].controls[0] {
            Control::Table(t) => t,
            _ => panic!("원문 자리차지/후속 블록 표"),
        };
        assert_eq!((source.row_count, source.col_count), (rows, 3));
        let mut actual = vec![String::new(); source.cells.len()];
        let mut occurrences = vec![0usize; source.cells.len()];
        let mut owners = Vec::new();
        for (page, tree) in pages.iter().enumerate() {
            let Some(grid) = table(&tree.root, paragraph) else {
                continue;
            };
            owners.push(page + 1);
            assert_on_paper(grid, &tree.root);
            for child in &grid.children {
                let RenderNodeType::TableCell(cell) = &child.node_type else {
                    continue;
                };
                let index = cell.model_cell_index.expect("원문 셀 소유 번호") as usize;
                let original = source.cells.get(index).expect("유효한 원문 셀");
                assert_eq!((cell.row, cell.col), (original.row, original.col));
                occurrences[index] += 1;
                text(child, &mut actual[index]);
            }
        }
        assert_eq!(owners, expected_pages, "독립 Print의 표 조각 쪽 소유");
        for (index, original) in source.cells.iter().enumerate() {
            assert!(
                occurrences[index] > 0,
                "원문 셀 {paragraph}/{index} 누락 금지"
            );
            let expected: String = original
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect();
            assert_eq!(
                normalized(&actual[index]),
                normalized(&expected),
                "원문 셀 {paragraph}/{index}: 합법적 계속 조각을 합쳐 글자 순서·누락·중복 검사"
            );
        }
        // 46×3 표는 전체 행 경계에서 나뉜다. 빈 셀도 중복 배치하지 않는다.
        if paragraph == 4 {
            assert!(occurrences.iter().all(|n| *n == 1));
        }
    }
}

fn pdf_px(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 한컴2020 Print ±2px: {label}, actual={actual}, expected={expected}"
    );
}

#[test]
fn issue_7727_following_table_keeps_independent_print_edges() {
    let core = open_sample();
    // 독립 PDF 벡터의 가로 괘선 중심을 96/72로 환산했다. rhwp bbox에서 역산하지 않는다.
    for (page, top, bottom) in [
        (185, 309.26131185, 1034.54800415),
        (186, 79.27335612, 945.04667155),
    ] {
        let tree = core.build_page_render_tree(page).unwrap();
        let grid = table(&tree.root, 4).expect("원문 구역2 문단4의 46×3 표");
        pdf_px(grid.bbox.y, top, "후속 표 상단");
        pdf_px(grid.bbox.y + grid.bbox.height, bottom, "후속 표 하단");
    }
}
