//! 검사 유형: 고정형 6개(독립 Print ±2px), 관계형 5개(내용 소속·보존·실제 기호 공급 전진).
//! 주석 갱신: 2026-11-07 10:35:03 KST.
//! 2026-11-07 측정: canonical PDFium96DPI·compat2022·2px 색상 경계 보정 전 내용 실루엣 raw.
//! 원본 samples/issue5679/10857_delegation_rules.hwp / 독립 한컴2020 Print 전103쪽.
//! 전103쪽 최저 Native 99.21307% / fresh WASM 99.21221% (27쪽), 98% 미달·누락0.
//! 26쪽 Native 99.98249% / fresh WASM 99.98249%; 저장 슬롯과 마지막 독립 괘선.
//! 27쪽 Native 99.21307% / fresh WASM 99.21221%; 이어받은 내용 소속은 관계형으로 보호.
//! 19쪽 Native 100.00000% / fresh WASM 100.00000%.
//! 30쪽 Native 100.00000% / fresh WASM 100.00000%.
//! 44쪽 Native 100.00000% / fresh WASM 100.00000%.
//! 51쪽 Native 100.00000% / fresh WASM 100.00000%.
//! 76쪽 Native 100.00000% / fresh WASM 100.00000%.
//! 86쪽 Native 99.70650% / fresh WASM 99.70611%.
//! 굴림체 U+2024의 실제 결측 공급 전진은 관계형, 44쪽 앞뒤 문자 원점은 독립 PDF 고정형이다.
//! 기호 보정7 PASS·2 FAIL→9 PASS, 슬롯 보정9 PASS·2 FAIL→11 PASS; 글꼴 바이너리 불포함.
//! 근거: mydocs/pr/assets/issue7700/one-dot-program-20261107/README.md.
//! 첫 저장 조각: mydocs/pr/assets/issue7700/opening-cell-frame-20261107/README.md.
//! 기존6건의 선행 근거: mydocs/pr/assets/issue7700/repair-20261106/phase2/README.md.
//! 슬롯 근거: mydocs/pr/assets/issue7700/reset-span-slot-20261107/README.md.
//! 다른 문서·PR 전체 검증 완료가 아니다. 과거 잘못된 좌표·105쪽 기대값은 복원하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue5679/10857_delegation_rules.hwp";

fn core() -> DocumentCore {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    DocumentCore::from_bytes(&bytes).expect("위임전결규정 원본 파싱")
}

fn table(node: &RenderNode, paragraph: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t)
        if t.para_index == Some(paragraph) && t.cell_context.is_none())
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table(child, paragraph))
}

fn cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    let cells: Vec<_> = table.children.iter().filter(|child|
        matches!(&child.node_type, RenderNodeType::TableCell(c) if c.row == row && c.col == col)
    ).collect();
    assert_eq!(cells.len(), 1, "원본 셀 ({row},{col})의 조각 소유");
    cells[0]
}

fn bottom(node: &RenderNode) -> f64 {
    node.bbox.y + node.bbox.height
}
fn pdf_px(actual: f64, independent: f64, label: &str) {
    assert!(
        (actual - independent).abs() <= 2.0,
        "독립 한컴 Print ±2px: {label}, actual={actual}, expected={independent}"
    );
}

#[test]
fn issue_7700_nested_span_growth_is_not_added_to_the_parent_remainder_again() {
    let core = core();
    let page = core.build_page_render_tree(18).unwrap();
    let grid = table(&page.root, 67).expect("19쪽 물품관리 표");
    let parent = cell(grid, 30, 0);
    let peer = cell(grid, 30, 1);
    // 독립 PDF 19쪽의 같은 두 전체 괘선. 현재 rhwp 출력을 기대값으로 복사하지 않는다.
    pdf_px(parent.bbox.y, 299.67204, "병합 띠 상단");
    pdf_px(bottom(parent), 695.23999, "병합 띠 하단");
    pdf_px(bottom(peer), 695.23999, "같은 띠의 업무 라벨 하단");
    let source = &core.document().sections[0].paragraphs[67];
    let original = source
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => {
                t.cells.iter().find(|c| c.row == 30 && c.col == 0)
            }
            _ => None,
        })
        .expect("원본 병합 선언");
    assert!(
        (parent.bbox.height - f64::from(original.height) / 75.0).abs() <= 2.0,
        "작은 병합 셀의 성장분을 원본 부모 최소 높이에 중복 가산하지 않는다"
    );
    let last = cell(grid, 42, 6);
    assert!(
        (bottom(last) - bottom(parent)).abs() <= f64::EPSILON * bottom(parent).abs() * 128.0,
        "같은 닫힘 행의 실제 물리 하단을 공유한다"
    );
}

#[test]
fn issue_7700_fitting_final_row_keeps_its_independent_print_page_owner() {
    let core = core();
    let p25 = core.build_page_render_tree(24).unwrap();
    let p26 = core.build_page_render_tree(25).unwrap();
    let p27 = core.build_page_render_tree(26).unwrap();
    // 이어받는 (57,2)는 원문부터 두 행 병합이다. 그 칸의 합법적 계속을
    // 중복으로 오인하지 않고, 독립 Print의 온전한 행과 대응하는 단일 행 열을 검사한다.
    let source_table = core.document().sections[0].paragraphs[68]
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .unwrap();
    for row in [30, 57, 58] {
        let source = source_table
            .cells
            .iter()
            .find(|c| c.row == row && c.col == 7)
            .unwrap();
        assert_eq!(source.row_span, 1, "온전한 행의 원본 소유자");
    }
    let t25 = table(&p25.root, 68).unwrap();
    let t26 = table(&p26.root, 68).unwrap();
    let t27 = table(&p27.root, 68).unwrap();
    cell(t25, 30, 7);
    cell(t26, 57, 7);
    cell(t27, 58, 7);
    let starts = |t: &RenderNode, row| {
        t.children
            .iter()
            .any(|n| matches!(&n.node_type,RenderNodeType::TableCell(c) if c.row==row && c.col==7))
    };
    assert!(
        !starts(t26, 30),
        "25쪽의 마지막 온전한 행을 26쪽에서 다시 시작하지 않는다"
    );
    assert!(
        !starts(t27, 57),
        "26쪽의 마지막 온전한 행을 27쪽에서 다시 시작하지 않는다"
    );
    assert!(bottom(cell(t25, 30, 7)) <= bottom(t25) + 1e-9);
    assert!(bottom(cell(t26, 57, 7)) <= bottom(t26) + 1e-9);
}

fn lines<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(node);
        return;
    }
    for child in &node.children {
        lines(child, out);
    }
}

#[test]
fn issue_7700_continuation_replanning_preserves_the_new_spans_content_floor() {
    let core = core();
    let page = core.build_page_render_tree(29).unwrap();
    let grid = table(&page.root, 70).unwrap();
    let brm = cell(grid, 59, 2);
    let last = cell(grid, 60, 3);
    // PDF 30쪽 BRM 운영 칸의 위·아래 괘선과 그 옆 마지막 행의 위 괘선.
    pdf_px(brm.bbox.y, 251.88403, "BRM 상단");
    pdf_px(bottom(brm), 333.71468, "BRM 하단");
    pdf_px(last.bbox.y, 280.33333, "마지막 행 상단");
    pdf_px(bottom(last), 333.71468, "마지막 행 하단");
    let mut content = Vec::new();
    lines(brm, &mut content);
    assert_eq!(content.len(), 5, "독립 Print의 BRM 라벨 다섯 줄");
    for line in content {
        assert!(
            line.bbox.y >= brm.bbox.y && bottom(line) <= bottom(brm),
            "재계산된 조각의 clip 안에 실제 줄 상자가 보존되어야 한다"
        );
    }
}

fn host_lines<'a>(
    node: &'a RenderNode,
    paragraph: usize,
    inside_table: bool,
    out: &mut Vec<&'a RenderNode>,
) {
    let inside_table = inside_table || matches!(node.node_type, RenderNodeType::Table(_));
    if !inside_table
        && matches!(&node.node_type,RenderNodeType::TextLine(l) if l.para_index==Some(paragraph))
    {
        out.push(node);
    }
    for child in &node.children {
        host_lines(child, paragraph, inside_table, out);
    }
}

#[test]
fn issue_7700_deferred_float_keeps_its_empty_host_before_the_following_inline_table() {
    let core = core();
    let p76 = core.build_page_render_tree(75).unwrap();
    let p77 = core.build_page_render_tree(76).unwrap();
    let following = table(&p76.root, 132).unwrap();
    assert!(
        table(&p76.root, 131).is_none(),
        "이월된 부동 표는 76쪽에 남지 않는다"
    );
    assert!(
        table(&p77.root, 131).is_some(),
        "같은 부동 표는 77쪽에서 그린다"
    );
    let mut hosts = Vec::new();
    let mut next_hosts = Vec::new();
    host_lines(&p76.root, 131, false, &mut hosts);
    host_lines(&p77.root, 131, false, &mut next_hosts);
    assert_eq!(hosts.len(), 1, "빈 호스트 줄도 현재 쪽의 원문 점유다");
    assert!(
        next_hosts.is_empty(),
        "빈 호스트를 부동 표와 함께 다시 그리지 않는다"
    );
    assert!(
        bottom(hosts[0]) < following.bbox.y,
        "뒤 inline 표는 빈 호스트 아래에 배치한다"
    );
    pdf_px(following.bbox.y, 399.24268, "76쪽 후속 inline 표 상단");
}

fn body_text(node: &RenderNode, in_body: bool, out: &mut String) {
    if !node.visible || node.editor_only {
        return;
    }
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    if in_body {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.extend(run.display_or_text().chars().filter(|c| !c.is_whitespace()));
        }
    }
    for child in &node.children {
        body_text(child, in_body, out);
    }
}

#[test]
fn issue_7700_merged_labels_keep_single_source_ownership_across_all_103_pages() {
    let core = core();
    assert_eq!(core.page_count(), 103, "독립 Print 전쪽 수");
    let mut text = String::new();
    for page in 0..core.page_count() {
        body_text(
            &core.build_page_render_tree(page).unwrap().root,
            false,
            &mut text,
        );
    }
    for label in ["사무분장조정", "조사관교육훈련"] {
        assert_eq!(
            text.matches(label).count(),
            1,
            "원본 병합 라벨 {label}의 누락·중복"
        );
    }
}

fn run_context<'a>(
    node: &'a RenderNode,
    needle: &str,
    line: Option<&'a RenderNode>,
    cell: Option<&'a RenderNode>,
) -> Option<(&'a RenderNode, &'a RenderNode, &'a RenderNode)> {
    let line = if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        Some(node)
    } else {
        line
    };
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if matches!(&node.node_type,RenderNodeType::TextRun(r) if r.text.contains(needle)) {
        return Some((node, line?, cell?));
    }
    node.children
        .iter()
        .find_map(|c| run_context(c, needle, line, cell))
}

#[test]
fn issue_7700_justified_trailing_spaces_keep_only_their_natural_advance() {
    let core = core();
    let page = core.build_page_render_tree(10).unwrap();
    for needle in ["외부 평가전문위원", "단,"] {
        let (node, line, cell) =
            run_context(&page.root, needle, None, None).expect("원본 말미 공백 run");
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        let trailing: String = run
            .display_or_text()
            .chars()
            .rev()
            .take_while(|c| *c == ' ')
            .collect();
        assert!(!trailing.is_empty(), "배분 정렬 끝 공백 재현 조건");
        let mut natural = run.style.clone();
        natural.extra_word_spacing = 0.0;
        let natural_width = EmbeddedTextMeasurer.estimate_text_width(&trailing, &natural);
        let epsilon =
            f64::EPSILON * (node.bbox.x.abs() + node.bbox.width + cell.bbox.width) * 128.0;
        assert!(
            node.bbox.width - line.bbox.width <= natural_width + epsilon,
            "줄 폭 초과에는 자연 끝 공백만 허용: {needle}"
        );
        assert!(
            node.bbox.x + node.bbox.width
                <= cell.bbox.x + cell.bbox.width + natural_width + epsilon,
            "칸 오른쪽을 배분 여분으로 넘지 않는다: {needle}"
        );
    }
}

#[test]
fn issue_7700_first_partial_ordinary_cell_preserves_its_saved_frame_inside_the_completed_label_span(
) {
    let core = core();
    let source = core.document().sections[0].paragraphs[102]
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .unwrap();
    let owner = source
        .cells
        .iter()
        .find(|c| c.row == 28 && c.col == 3)
        .unwrap();
    let segs = &owner.paragraphs[0].line_segs;
    assert_eq!(segs.len(), 3);
    assert_eq!(segs[1].vertical_pos, 0);
    assert_eq!(segs[0].vertical_pos, 0);
    let whole = source
        .cells
        .iter()
        .find(|c| c.row == 28 && c.col == 8)
        .unwrap();
    assert!(owner.height < whole.height);
    let first = core.build_page_render_tree(50).unwrap();
    let next = core.build_page_render_tree(51).unwrap();
    let first_table = table(&first.root, 102).unwrap();
    let next_table = table(&next.root, 102).unwrap();
    let physical = cell(first_table, 28, 3);
    assert!(
        physical.bbox.height <= f64::from(owner.height) / 75.0,
        "첫 조각의 저장 선언을 넘지 않는다"
    );
    pdf_px(
        bottom(physical),
        1025.11865,
        "51쪽 첫 저장 조각의 독립 Print 하단",
    );
    for col in 4..=8 {
        assert!(
            (cell(first_table, 28, col).bbox.height - physical.bbox.height).abs() < 1e-9,
            "같은 행의 물리 하단 공유"
        );
    }
    let mut a = vec![];
    let mut b = vec![];
    lines(physical, &mut a);
    lines(cell(next_table, 28, 3), &mut b);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 2);
}

#[test]
fn issue_7700_missing_monospace_dot_uses_the_independently_verified_supplier_advance() {
    let core = core();
    let page = core.build_page_render_tree(43).unwrap();
    let grid = table(&page.root, 87).unwrap();
    let owner = cell(grid, 7, 2);
    let (run, _, _) = run_context(owner, "교육", None, None).unwrap();
    let RenderNodeType::TextRun(r) = &run.node_type else {
        panic!("원본 교육 라벨")
    };
    let mut provider = r.style.clone();
    provider.font_family = "Haansoft Batang".into();
    let measured = EmbeddedTextMeasurer.estimate_text_width("․", &r.style);
    let natural = EmbeddedTextMeasurer.estimate_text_width("․", &provider);
    assert!(
        (measured - natural).abs() < 1e-9,
        "원 프로그램 cmap 결측의 독립 공급자 전진 actual={measured}, provider={natural}"
    );
    assert!(measured < EmbeddedTextMeasurer.estimate_text_width("가", &r.style));
}
#[test]
fn issue_7700_centered_missing_dot_keeps_both_neighbor_origins_at_the_independent_print_positions()
{
    let core = core();
    let page = core.build_page_render_tree(43).unwrap();
    let grid = table(&page.root, 87).unwrap();
    let owner = cell(grid, 7, 2);
    let (a, _, _) = run_context(owner, "교육", None, None).unwrap();
    let (b, _, _) = run_context(owner, "홍보", None, None).unwrap();
    pdf_px(
        a.bbox.x,
        264.63999430338544,
        "44쪽 교육․홍보의 앞 글자 원점",
    );
    pdf_px(
        b.bbox.x,
        293.59999593098956,
        "44쪽 교육․홍보의 뒤 글자 원점",
    );
}

#[test]
fn issue_7700_original_reset_span_keeps_its_whole_last_line_slot_in_the_physical_row() {
    let core = core();
    let source = core.document().sections[0].paragraphs[68]
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .unwrap();
    let owner = source
        .cells
        .iter()
        .find(|c| c.row == 57 && c.col == 1)
        .unwrap();
    assert_eq!(owner.row_span, 3);
    assert_eq!(
        owner.vertical_align,
        rhwp::model::table::VerticalAlign::Center
    );
    let segs = &owner.paragraphs[0].line_segs;
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[1].vertical_pos, 0);
    assert!(segs[0].line_spacing > 0);
    let padding = if owner.apply_inner_margin {
        &owner.padding
    } else {
        &source.padding
    };
    let slot = f64::from(
        segs[0].line_height
            + segs[0].line_spacing
            + i32::from(padding.top)
            + i32::from(padding.bottom),
    ) / 75.0;
    let first = core.build_page_render_tree(25).unwrap();
    let next = core.build_page_render_tree(26).unwrap();
    let grid = table(&first.root, 68).unwrap();
    let rendered = cell(grid, 57, 1);
    assert!(
        rendered.bbox.height + 1e-9 >= slot,
        "원본 reset 앞 줄 슬롯은 수용과 paint 모두 소유 actual={}, source={slot}",
        rendered.bbox.height
    );
    for col in [3, 7, 8, 9, 10, 11] {
        assert!(
            (cell(grid, 57, col).bbox.height - rendered.bbox.height).abs() < 1e-9,
            "같은 컷의 일반 peer와 병합 슬롯은 물리 하단 공유"
        );
    }
    let mut a = String::new();
    let mut b = String::new();
    body_text(rendered, true, &mut a);
    body_text(cell(table(&next.root, 68).unwrap(), 57, 1), true, &mut b);
    assert!(a.contains("통계관리"));
    assert!(!a.contains("분석"));
    assert!(b.contains("분석"));
    assert!(!b.contains("통계관리"));
}
#[test]
fn issue_7700_original_reset_span_closes_at_the_independent_print_bottom() {
    let core = core();
    let page = core.build_page_render_tree(25).unwrap();
    let grid = table(&page.root, 68).unwrap();
    pdf_px(
        bottom(cell(grid, 57, 7)),
        1023.68,
        "26쪽 원본 reset 병합 조각과 일반 peer의 닫힘선",
    );
}
