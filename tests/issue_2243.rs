//! Issue #2243 — 결재문서의 쪽수와 표·후속 문단의 물리 배치 회귀 검사.
//!
//! 병인: 표 경로의 vpos 앵커 미확립/일괄 말소로 후속 문단 스냅이 드리프트를
//! 역산한 lazy base 에 고착 → 페이지당 +16~22px 팬텀 → razor 경계에서 sliver
//! 쪽(+1~+2). 수정: 표-경로 vpos 앵커 확립 + 저장 사다리 조건부 유지 + 빈 앵커
//! float 표 host 직후 lazy 이중 계상 가드 + 저장-앵커 safety 마진 면제.
//! 기존 쪽수만으로는 컨설팅의 붙임 문단 이동과 세운의 제목/표 순서 반전을
//! 놓쳤다. 기대값은 `pdf/task2243/*-hwpx-2020.pdf`의 쪽수, 페이지의 문단·표 소속과
//! 앞뒤 순서다. 원문 행열·셀 및 선언 줄간격의 보존도 실제 출력에서 검사한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// (샘플, 한글 실측 쪽수)
const PINS: &[(&str, u32)] = &[
    // 한컴 PDF 5쪽. 표와 붙임의 위치도 아래에서 검사한다.
    ("samples/task2243/36395325_gyeoljae_consulting.hwpx", 5),
    // 빈 앵커 float 표 host 직후 lazy 이중 계상 가드 (4→3쪽)
    ("samples/task2243/36382819_gyeoljae_pm_traffic.hwpx", 3),
    // 지속 결함 복구 (7→5쪽)
    ("samples/task2243/36386907_gyeoljae_sewoon.hwpx", 5),
    // 저장-앵커 safety 마진 면제 razor (2→1쪽, stored 881.3+fit 49.6=930.9≤933.6)
    ("samples/task2243/156631374_taxi_press.hwpx", 1),
];

/// 본문 단의 항목만 모아 같은 문단 번호를 갖는 셀·머리말과 혼동하지 않는다.
fn column_items(root: &RenderNode) -> Vec<&RenderNode> {
    if matches!(root.node_type, RenderNodeType::Column(_)) {
        return root.children.iter().collect();
    }
    if matches!(
        root.node_type,
        RenderNodeType::Page(_) | RenderNodeType::Body { .. }
    ) {
        return root.children.iter().flat_map(column_items).collect();
    }
    Vec::new()
}

fn text_content(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.text.clone();
    }
    node.children.iter().map(text_content).collect()
}

/// 기대 소속·행열·셀은 원문과 한컴 PDF에서 확인한 계약이다.
/// 용지의 고정 픽셀 원점이나 글자 기준선을 회귀 기준으로 쓰지 않는다.
fn table_on_page(
    core: &DocumentCore,
    source: &rhwp::model::document::Document,
    page: u32,
    para: usize,
) -> rhwp::renderer::render_tree::BoundingBox {
    let tree = core
        .build_page_render_tree(page - 1)
        .expect("본문 렌더 트리");
    let tables: Vec<_> = column_items(&tree.root)
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(para) && table.control_index == Some(0)
                && table.cell_context.is_none())
        })
        .collect();
    assert_eq!(tables.len(), 1, "{page}쪽 문단 {para}: 표의 누락·중복");
    let table_node = tables[0];
    let rhwp::model::control::Control::Table(source_table) =
        &source.sections[0].paragraphs[para].controls[0]
    else {
        panic!("원문의 표 소유");
    };
    let RenderNodeType::Table(table) = &table_node.node_type else {
        unreachable!()
    };
    assert_eq!(
        (table.row_count, table.col_count),
        (source_table.row_count, source_table.col_count),
        "{page}쪽 문단 {para}: 원문 행·열 보존"
    );
    let mut cells: Vec<_> = table_node
        .children
        .iter()
        .filter_map(|n| {
            if let RenderNodeType::TableCell(cell) = &n.node_type {
                cell.model_cell_index
            } else {
                None
            }
        })
        .collect();
    cells.sort_unstable();
    assert_eq!(
        cells,
        (0..source_table.cells.len() as u32).collect::<Vec<_>>(),
        "{page}쪽 문단 {para}: 원문 셀의 누락·중복"
    );
    fn body_bottom(node: &RenderNode) -> Option<f64> {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            Some(node.bbox.y + node.bbox.height)
        } else {
            node.children.iter().find_map(body_bottom)
        }
    }
    assert!(
        table_node.bbox.y + table_node.bbox.height
            <= body_bottom(&tree.root).expect("본문 경계") + 0.5,
        "{page}쪽 문단 {para}: 표가 본문 하단을 넘음"
    );
    table_node.bbox
}

fn text_on_page(
    core: &DocumentCore,
    page: u32,
    para: usize,
    text: &str,
) -> rhwp::renderer::render_tree::BoundingBox {
    let mut matches = Vec::new();
    for p in 1..=core.page_count() {
        let tree = core.build_page_render_tree(p - 1).expect("본문 렌더 트리");
        for node in column_items(&tree.root) {
            if matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(para))
                && text_content(node).contains(text)
            {
                matches.push((p, node.bbox));
            }
        }
    }
    assert_eq!(
        matches.len(),
        1,
        "문단 {para}: {text:?}의 누락·중복·잘못된 소유"
    );
    assert_eq!(matches[0].0, page, "문단 {para}: {text:?}의 쪽 소속");
    matches[0].1
}

fn assert_follows(
    before: rhwp::renderer::render_tree::BoundingBox,
    after: rhwp::renderer::render_tree::BoundingBox,
) {
    assert!(
        after.y + 0.5 >= before.y + before.height,
        "뒤 내용이 앞 내용의 점유 상자와 겹치거나 순서가 역전됨"
    );
}

#[test]
fn issue_2243_gyeoljae_sliver_page_pins() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    for (sample, expected) in PINS {
        let bytes = fs::read(Path::new(repo_root).join(sample))
            .unwrap_or_else(|e| panic!("read {sample}: {e}"));
        let source = rhwp::parser::parse_document(&bytes).expect("독립 원문 계약");
        let core =
            DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
        assert_eq!(
            core.page_count(),
            *expected,
            "{sample}: 한컴 PDF 쪽수와 불일치"
        );
        match *sample {
            "samples/task2243/36395325_gyeoljae_consulting.hwpx" => {
                let first_table = table_on_page(&core, &source, 3, 22);
                let after_first = text_on_page(&core, 3, 23, "검토대상");
                assert_follows(first_table, after_first);
                let before_second = text_on_page(&core, 3, 25, "*");
                let second_table = table_on_page(&core, &source, 3, 26);
                assert_follows(before_second, second_table);
                // 원문의 백분율 줄간격은 다음 개체가 오더라도 전량 남는다.
                // PDF 3쪽에서도 문단25 아래 간격을 소비한 뒤 표26이 시작한다.
                let para = &source.sections[0].paragraphs[25];
                let style = &source.doc_info.para_shapes[para.para_shape_id as usize];
                assert!(matches!(
                    style.line_spacing_type,
                    rhwp::model::style::LineSpacingType::Percent
                ));
                let font = &source.doc_info.char_shapes[para.char_shapes[0].char_shape_id as usize];
                let declared_spacing = f64::from(font.base_size) * 96.0 / 7200.0
                    * f64::from(style.line_spacing - 100)
                    / 100.0;
                assert!(
                    second_table.y + 0.5
                        >= before_second.y + before_second.height + declared_spacing,
                    "문단25의 선언 줄간격이 다음 표 원점에서 손실됨"
                );
                assert_follows(second_table, text_on_page(&core, 3, 27, "*"));
                let table = table_on_page(&core, &source, 5, 40);
                let attachment = text_on_page(&core, 5, 41, "붙임");
                let result = text_on_page(&core, 5, 42, "검토결과");
                assert_follows(table, attachment);
                assert_follows(attachment, result);
            }
            "samples/task2243/36382819_gyeoljae_pm_traffic.hwpx" => {
                let table = table_on_page(&core, &source, 3, 30);
                assert_follows(table, text_on_page(&core, 3, 41, "주민 의견 수렴 참고서"));
            }
            "samples/task2243/36386907_gyeoljae_sewoon.hwpx" => {
                let title = text_on_page(&core, 3, 35, "심사항목");
                let table = table_on_page(&core, &source, 3, 35);
                assert_follows(title, table);
                let title = text_on_page(&core, 4, 39, "결");
                let table = table_on_page(&core, &source, 4, 39);
                assert_follows(title, table);
                text_on_page(&core, 5, 44, "붙임");
            }
            _ => {}
        }
    }
}
