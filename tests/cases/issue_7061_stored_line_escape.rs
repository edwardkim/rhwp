//! [#7061] 여섯째 축 — 글자가 자기 저장 줄을 벗어나 **남의 저장 줄** baseline 에 앉았다.
//!
//! 다섯 축은 전부 "상자를 넘었나 / 둘이 겹쳤나"라, 이탈이 쪽 밖으로 나가지도 다른 상자와
//! 겹치지도 않으면 전부 0 이었다. `#7018` 수정 두 커밋을 되돌려 잰 A/B 에서 165px 짜리
//! 배치 이탈이 있으나 없으나 봉투가 한 글자도 다르지 않았다.
//!
//! # 이 시험이 잠그는 양쪽
//!
//! **대상** — `#7018` 재현체의 실제 저장값으로 만든 형상. 표가 자기 저장 줄을 가진 문단에서
//! 앞 텍스트가 표 줄 baseline(13,423 HU = 178.97px)을 받아 자기 줄(1,020 HU = 13.60px)에서
//! 165.37px 아래로 떨어진다.
//!
//! **반례** — 한/글 2020 정본이 "rhwp 가 맞다"고 말한 네 형상. 좁힘 조건을 하나라도 빼면
//! 이 넷이 위양성으로 올라온다(전수에서 각각 145·14·1·27 건).
//!
//! | 문서 | rhwp | 정본 | 저장 `baseline_distance` |
//! | --- | ---: | ---: | ---: |
//! | `exam_eng.hwp` 1쪽 `①` | 494.76 | **494.08** | 490.16 |
//! | `pr-1674.hwp` 23쪽 `1.` | 172.67 | **173.60** | 167.47 |
//! | `tac-case-003.hwp` 1쪽 `표 다음` | 214.72 | **214.49** | 238.91 |
//! | `issue6181/…` 6쪽 `회피` | 158.43 | **158.40** | 161.58 |
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page_with_source, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::renderer::render_tree::{
    BoundingBox, PageNode, RenderNode, RenderNodeType, TextLineNode, TextRunNode,
};
use rhwp::renderer::TextStyle;

/// HWPUNIT → px (96 DPI). 검출기와 같은 환산.
fn px(hwpunit: i32) -> f64 {
    f64::from(hwpunit) / 75.0
}

fn seg(text_start: u32, vertical_pos: i32, line_height: i32, baseline_distance: i32) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos,
        line_height,
        text_height: line_height,
        baseline_distance,
        tag: 0x0006_0000,
        ..LineSeg::default()
    }
}

fn doc_with(text: &str, line_segs: Vec<LineSeg>) -> Document {
    let para = Paragraph {
        text: text.to_string(),
        line_segs,
        ..Paragraph::default()
    };
    let section = Section {
        paragraphs: vec![para],
        ..Section::default()
    };
    Document {
        sections: vec![section],
        ..Document::default()
    }
}

/// `#7018` 재현체 `samples/issue7018/2769535-records-inspection-plan.hwpx` 의 문단 0/27 을
/// 저장값 그대로 옮긴 것 — 글자 줄(1,020 HU) 하나와 표 줄(13,423 HU) 하나.
fn doc_7018_shape() -> Document {
    doc_with(
        "  마. 행정박물류 ",
        vec![seg(0, 39764, 1200, 1020), seg(11, 41924, 15792, 13423)],
    )
}

fn text_run(text: &str, char_start: usize, baseline: f64, bbox: BoundingBox) -> RenderNode {
    RenderNode::new(
        3,
        RenderNodeType::TextRun(TextRunNode {
            text: text.to_string(),
            style: TextStyle::default(),
            char_shape_id: None,
            para_shape_id: None,
            section_index: Some(0),
            para_index: Some(0),
            char_start: Some(char_start),
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        }),
        bbox,
    )
}

fn page_with(children: Vec<RenderNode>) -> RenderNode {
    let mut body = RenderNode::new(
        1,
        RenderNodeType::Body { clip_rect: None },
        BoundingBox::new(0.0, 0.0, 600.0, 800.0),
    );
    body.children = children;
    let mut root = RenderNode::new(
        0,
        RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 600.0,
            height: 800.0,
            section_index: 0,
        }),
        BoundingBox::new(0.0, 0.0, 600.0, 800.0),
    );
    root.children.push(body);
    root
}

fn escapes(root: &RenderNode, doc: &Document) -> usize {
    scan_page_with_source(0, root, 1, &AnomalyOptions::default(), Some(doc))
        .stored_line_escape
        .len()
}

// ── 줄 노드 없는 경로 (`#7018` 이 이쪽이다) ───────────────────────────────

/// 상자가 `줄 위 ~ baseline` 형상인 고아 런은 `baseline == bbox.height` 다.
fn orphan_run(text: &str, char_start: usize, baseline: f64) -> RenderNode {
    text_run(
        text,
        char_start,
        baseline,
        BoundingBox::new(100.0, 605.8, 200.0, baseline),
    )
}

#[test]
fn orphan_run_that_took_another_stored_lines_baseline_is_reported() {
    let doc = doc_7018_shape();
    let root = page_with(vec![orphan_run("  마. 행정박물류", 0, px(13423))]);
    let pa = scan_page_with_source(0, &root, 1, &AnomalyOptions::default(), Some(&doc));

    assert_eq!(pa.stored_line_escape.len(), 1, "이탈 1건이어야 합니다");
    let a = &pa.stored_line_escape[0];
    assert!(!a.from_line_node);
    assert_eq!(
        a.line_seg_index, 0,
        "이 글자가 속한 줄은 0번(글자 줄)입니다"
    );
    assert_eq!(
        a.took_line_seg_index, 1,
        "실제로 앉은 줄은 1번(표 줄)입니다"
    );
    assert!(
        (a.deviation() - 165.37).abs() < 0.05,
        "이탈 거리가 165.37px 여야 합니다: {}",
        a.deviation()
    );

    // 다섯 축은 이 이탈을 여전히 못 본다 — 그게 이 축이 필요한 이유다.
    assert!(pa.overflow.is_empty() && pa.off_canvas.is_empty());
    assert!(pa.overlap.is_empty() && pa.text_overlap.is_empty());
}

#[test]
fn orphan_run_on_its_own_stored_line_is_clean() {
    let doc = doc_7018_shape();
    let root = page_with(vec![orphan_run("  마. 행정박물류", 0, px(1020))]);
    assert_eq!(escapes(&root, &doc), 0);
}

/// 반례 ①(`exam_eng`·`pr-1674`·`issue6181` 형상) — 저장값과 다르지만 **자기 줄 상자 안**이다.
///
/// 정본은 이 모양에서 rhwp 가 0.03~0.93px 안에서 맞다고 말한다. 저장 `baseline_distance`
/// 자체가 한/글이 그리는 위치가 아니므로 신고하면 안 된다.
#[test]
fn baseline_inside_its_own_stored_line_box_is_not_an_escape() {
    // 자기 줄: 높이 1,800 HU(24.0px) · baseline 575 HU(7.67px).
    // 렌더는 920 HU(12.27px) 로 저장값과 4.6px 다르고, 그 값이 우연히 둘째 줄 값과 같다.
    let doc = doc_with(
        "①\u{a0}보기",
        vec![seg(0, 0, 1800, 575), seg(3, 1800, 1800, 920)],
    );
    let root = page_with(vec![orphan_run("①\u{a0}보", 0, px(920))]);
    assert!(
        px(920) < px(1800),
        "전제: 렌더 baseline 이 자기 줄 상자 안이다"
    );
    assert_eq!(escapes(&root, &doc), 0);
}

/// 반례 ②(`tac-case-002` 형상) — 런이 저장 줄 경계를 걸친다.
///
/// 시작 글자만 보면 앞 줄(표가 있는 줄)에 속한 것으로 읽혀 위양성이 된다.
#[test]
fn run_spanning_a_stored_line_boundary_is_not_judged() {
    let doc = doc_with(
        "tacglkj 표 3 배치 시작    4 tacglkj 표 다음",
        vec![seg(0, 1600, 3134, 2664), seg(31, 5334, 1000, 850)],
    );
    // 23..34 — 저장 경계 31 을 넘는다.
    let root = page_with(vec![orphan_run("tacglkj 표 다음", 23, px(850))]);
    assert_eq!(escapes(&root, &doc), 0);

    // 같은 문단에서 경계 안에 온전히 들어가면 판정 대상이다.
    let inside = page_with(vec![orphan_run("표 다음", 31, px(2664))]);
    assert_eq!(escapes(&inside, &doc), 1, "경계 안 런은 판정한다");
}

/// 반례 ③(`tac-case-003` 형상) — 저장 `text_start` 가 문단 글자 수를 넘는다.
///
/// 저장 서수와 IR `char_start` 가 같은 색인 공간이 아니므로 줄을 서수로 고를 수 없다.
#[test]
fn paragraph_whose_stored_index_space_overflows_is_not_judged() {
    let doc = doc_with(
        "tacglkj 표 3 배치 시작    4 tacglkj 표 다음",
        vec![seg(0, 1600, 3134, 2664), seg(39, 5334, 1000, 850)],
    );
    let root = page_with(vec![orphan_run("표 다음", 31, px(850))]);
    assert_eq!(escapes(&root, &doc), 0);
}

// ── 줄 노드 경로 ─────────────────────────────────────────────────────────

fn line_node(vpos: i32, height: f64, baseline: f64, run: RenderNode) -> RenderNode {
    let mut l = TextLineNode::new(height, baseline);
    l.section_index = Some(0);
    l.para_index = Some(0);
    l.line_index = Some(0);
    l.vpos = Some(vpos);
    let mut node = RenderNode::new(
        2,
        RenderNodeType::TextLine(l),
        BoundingBox::new(100.0, 300.0, 200.0, height),
    );
    node.children.push(run);
    node
}

#[test]
fn line_node_that_took_another_stored_lines_baseline_is_reported() {
    let doc = doc_7018_shape();
    let run = text_run(
        "  마. 행정박물류",
        0,
        px(13423),
        BoundingBox::new(100.0, 300.0, 200.0, 16.0),
    );
    // 줄 노드가 저장 줄 0 을 가리키고(vpos 39764) 줄 높이(1,200 HU = 16.0px)까지 재현했다.
    let root = page_with(vec![line_node(39764, px(1200), px(13423), run)]);
    let pa = scan_page_with_source(0, &root, 1, &AnomalyOptions::default(), Some(&doc));
    assert_eq!(pa.stored_line_escape.len(), 1);
    assert!(pa.stored_line_escape[0].from_line_node);
    assert_eq!(pa.stored_line_escape[0].took_line_seg_index, 1);
}

#[test]
fn line_node_reproducing_its_stored_baseline_is_clean() {
    let doc = doc_7018_shape();
    let run = text_run(
        "  마. 행정박물류",
        0,
        px(1020),
        BoundingBox::new(100.0, 300.0, 200.0, 16.0),
    );
    let root = page_with(vec![line_node(39764, px(1200), px(1020), run)]);
    assert_eq!(escapes(&root, &doc), 0);
}

#[test]
fn line_node_that_did_not_reproduce_the_stored_line_height_is_not_judged() {
    let doc = doc_7018_shape();
    let run = text_run(
        "  마. 행정박물류",
        0,
        px(13423),
        BoundingBox::new(100.0, 300.0, 200.0, 40.0),
    );
    // 줄 높이가 저장(16.0px)과 다르면 그 줄은 저장 줄을 재현한 것이 아니다.
    let root = page_with(vec![line_node(39764, 40.0, px(13423), run)]);
    assert_eq!(escapes(&root, &doc), 0);
}

/// `source` 가 없으면 이 축만 비고 나머지 다섯은 그대로다.
#[test]
fn axis_is_silent_without_the_document_ir() {
    let root = page_with(vec![orphan_run("  마. 행정박물류", 0, px(13423))]);
    let pa = scan_page_with_source(0, &root, 1, &AnomalyOptions::default(), None);
    assert!(pa.stored_line_escape.is_empty());
}

// ── 실문서 반례 — 정본이 "rhwp 가 맞다"고 말한 쪽들 ──────────────────────

fn escapes_on_page(path: &str, page: u32) -> usize {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("재현체 {path}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("로드 {path}: {e}"));
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("렌더 {path} p{page}: {e}"));
    scan_page_with_source(
        page,
        &tree.root,
        core.page_count(),
        &AnomalyOptions::default(),
        Some(core.document()),
    )
    .stored_line_escape
    .len()
}

#[test]
fn oracle_backed_pages_report_no_escape() {
    // 좁힘 조건을 빼면 이 쪽들이 각각 145·14·1·27·3 건으로 올라왔다. 한/글 2020 정본은
    // 이 쪽들에서 rhwp 가 1px 안에서 맞다고 말한다(모듈 주석의 표).
    for (path, page) in [
        ("samples/exam_eng.hwp", 0u32),
        ("samples/pr-1674.hwp", 22),
        ("samples/tac-case-003.hwp", 0),
        (
            "samples/issue6181/156562368_inline_tac_table_line_advance.hwpx",
            5,
        ),
        (
            "samples/issue6180/156745974_tac_object_line_spacing.hwpx",
            6,
        ),
    ] {
        assert_eq!(
            escapes_on_page(path, page),
            0,
            "{path} p{page} 는 이탈이 없어야 합니다"
        );
    }
}

/// `#7018` 재현체 자신 — 그 수정이 살아 있는 devel 에서는 깨끗하다.
#[test]
fn issue_7018_fixture_is_clean_on_the_fixed_engine() {
    assert_eq!(
        escapes_on_page("samples/issue7018/2769535-records-inspection-plan.hwpx", 1),
        0
    );
}
