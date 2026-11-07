//! 검사 유형: 고정형 4개(독립 Print ±2px), 관계형 3개(내용 소속·보존·자연 전진).
//! 주석 갱신: 2026-11-07 09:27:46 KST.
//! 고정형(px): nested_span_growth / continuation_replanning / deferred_float / first_partial_ordinary_cell의 독립 Print ±2px.
//! 관계형: fitting_final_row / merged_labels / justified_trailing_spaces의 원문 소유·보존·자연 전진.
//! 독립 전103쪽 수는 범위 검사이며 절대 페이지 좌표 핀과 구분한다.
//! 현재 측정: parent0a7adeef5 + 저장 블록 프레임 공통2파일; 2026-11-07 전103쪽 재출력.
//! canonical 96DPI·compat2022·PDFium exact physical DPI·2px raw; 양쪽 103쪽·누락0·90% 미만0.
//! 전쪽 최저 Native 96.64273% / fresh WASM 96.64099%(양쪽 26쪽).
//! 새 px 대상 51쪽 Native 99.51311% / fresh WASM 99.51281%; 실제 이전6 PASS·1 FAIL→7 PASS.
//! px 대상 19쪽 N/W 100/100%, 30쪽 99.36627/99.36595%, 76쪽 99.57115/99.57141%.
//! 관계형 대상 25쪽 99.39152/99.38292%, 26쪽 96.64273/96.64099%, 27쪽 98.40958/98.40462%.
//! 말미 공백 대상 11쪽 99.82667/99.81696%; 기존 451.8px·8/14px 허용값을 복원하지 않는다.
//! 독립 Print SHA 90eda86b2b4d, 한컴2020 PrintMethod0 one-up, job 6bef4e6b-8a06-463e-8c09-539211ad7a25.
//! 임시 동일 본문 검증: 보정 전 2PASS4FAIL → 보정 후 6PASS; 조각 하한 미보정 중간본의 해당 검사 1FAIL.
//! 첫 초안의 (58,2) 합법적 병합 계속을 오인한 부분을 원문 단일행 열7로 정정한 결과다.
//! 증적: mydocs/pr/assets/issue7700/opening-cell-frame-20261107/README.md.
//! 기존6건의 선행 근거: mydocs/pr/assets/issue7700/repair-20261106/phase2/README.md.
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
