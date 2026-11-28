//! 관계형 — [#9479] 쪽을 넘는 1×1 칸 안의 흐름은 저장 사다리를 따른다: TAC 표 host 줄의 음수 줄간격,
//! 전진 0 겹침 빈 줄의 앞 간격, 꼬리 조각 상자의 선언 높이 나머지. 쪽수 검사는 고정형이다.
//! 주석 갱신: 2026-11-28 KST (처음 작성)
//!
//! 입력: `samples/issue9479/cell_tac_negative_line_spacing.hwp` — 공개 문서(교육부 재외 한국학교장 선발 계획)
//! 20쪽 원본의 5~6쪽을 `rhwp extract-pages --from 5 --to 6` 으로 발췌. 원본은 rhwp 20쪽 / 한/글 19쪽
//! (끼어든 빈 쪽, 별도 결함)이라 원본 대신 발췌본을 쓴다.
//! 기준: `pdf/issue9479/cell_tac_negative_line_spacing-2020.pdf` — 한/글 2020(11.0.0.9172) Hancom PDF Print, 3쪽.
//!
//! 측정 결과는 PR 본문과 `mydocs/pr/assets/issue_9479/` 의 Native·fresh WASM 전쪽 TSV 에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/issue9479/cell_tac_negative_line_spacing.hwp";
const HANCOM_PAGES: u32 = 3;
/// 발췌본 구역 0 의 1×1 RowBreak 표 host 문단.
const HOST_PARA: usize = 4;

fn px(hu: i64) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn open() -> DocumentCore {
    let bytes = std::fs::read(FIXTURE).expect("fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), HANCOM_PAGES, "한/글 Print 의 전체 쪽수");
    core
}

fn outer_table(core: &DocumentCore) -> &Table {
    match &core.document().sections[0].paragraphs[HOST_PARA].controls[0] {
        Control::Table(table) => table,
        other => panic!("host 문단의 첫 개체는 표여야 한다: {other:?}"),
    }
}

fn cell_para(core: &DocumentCore, index: usize) -> &Paragraph {
    &outer_table(core).cells[0].paragraphs[index]
}

fn first_vpos(para: &Paragraph) -> i64 {
    i64::from(para.line_segs[0].vertical_pos)
}

fn collect_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(run.display_text.as_deref().unwrap_or(&run.text));
    }
    for child in &node.children {
        collect_text(child, text);
    }
}

/// `needle` 을 담은 글줄의 위끝 y.
fn line_top(node: &RenderNode, needle: &str) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        collect_text(node, &mut text);
        return text.contains(needle).then_some(node.bbox.y);
    }
    node.children.iter().find_map(|child| line_top(child, needle))
}

fn page_line_top(core: &mut DocumentCore, page: u32, needle: &str) -> f64 {
    let tree = core.build_page_render_tree(page).unwrap();
    line_top(&tree.root, needle).unwrap_or_else(|| panic!("{}쪽 「{needle}」 글줄", page + 1))
}

fn table_cells(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        out.push((node.bbox.y, node.bbox.height));
    }
    for child in &node.children {
        table_cells(child, out);
    }
}

#[test]
fn tac_table_host_line_advances_by_its_stored_negative_line_spacing() {
    let mut core = open();
    // 칸 p[2]: 2×4 TAC 표 하나만 든 저장 줄 lh 7873 (= 표 7591 + 바깥 여백 282), ls −480.
    let host = cell_para(&core, 2);
    let seg = &host.line_segs[0];
    assert!(seg.line_spacing < 0, "fixture 의 음수 줄간격");
    let Control::Table(nested) = &host.controls[0] else {
        panic!("p[2] 는 표 host");
    };
    assert!(nested.common.treat_as_char);
    assert_eq!(
        seg.line_height,
        nested.common.height as i32
            + i32::from(nested.outer_margin_top)
            + i32::from(nested.outer_margin_bottom),
        "저장 줄 상자가 표 바깥 상자와 같다"
    );
    let before_vpos = first_vpos(cell_para(&core, 1));
    let after_vpos = first_vpos(cell_para(&core, 4));
    assert_eq!(
        after_vpos - first_vpos(cell_para(&core, 3)),
        720,
        "p[3] 빈 줄(lh 1200 ls −480) 뒤 p[4]"
    );

    let before = page_line_top(&mut core, 0, "교육공무원 자격요건");
    let after = page_line_top(&mut core, 0, "표의 언어를 선택한 경우");
    let expected = px(after_vpos - before_vpos);
    assert!(
        ((after - before) - expected).abs() <= 1.0,
        "표 뒤 글줄 간격 {:.2}px 는 저장 사다리 {expected:.2}px (±1)",
        after - before
    );
}

#[test]
fn zero_advance_overlap_spacer_keeps_its_spacing_before() {
    let mut core = open();
    // 칸 p[18]: 빈 문단, lh 1200 ls −1200 (줄 전진 0), 앞 간격이 있어 저장 vpos 가 p[17] 끝보다 아래다.
    let spacer = cell_para(&core, 18);
    let seg = &spacer.line_segs[0];
    assert!(spacer.text.trim().is_empty());
    assert_eq!(seg.line_height + seg.line_spacing, 0, "전진 0 겹침 빈 줄");
    let previous = cell_para(&core, 17);
    let previous_last = previous.line_segs.last().unwrap();
    let previous_end = i64::from(previous_last.vertical_pos)
        + i64::from(previous_last.line_height)
        + i64::from(previous_last.line_spacing);
    assert!(first_vpos(spacer) > previous_end, "앞 간격이 저장 사다리에 있다");
    let vpos_19 = first_vpos(cell_para(&core, 19));
    assert_eq!(first_vpos(spacer), vpos_19);
    let expected = px(vpos_19 - i64::from(previous_last.vertical_pos));

    let last_line_of_17 = page_line_top(&mut core, 0, "이수증 제출");
    let line_19 = page_line_top(&mut core, 0, "교육부가 지정하는 연수");
    assert!(
        ((line_19 - last_line_of_17) - expected).abs() <= 1.0,
        "빈 줄을 사이에 둔 간격 {:.2}px 는 저장 사다리 {expected:.2}px (±1)",
        line_19 - last_line_of_17
    );
}

#[test]
fn closing_fragment_box_takes_the_declared_remainder() {
    let mut core = open();
    let table = outer_table(&core);
    let cell = &table.cells[0];
    let remainder = i64::from(cell.height) - i64::from(table.common.height);
    // 후속 빈 문단의 저장 원점이 그 나머지와 바깥 여백으로 정확히 닫힌다.
    let successor = &core.document().sections[0].paragraphs[HOST_PARA + 1];
    assert_eq!(
        remainder + i64::from(table.outer_margin_top) + i64::from(table.outer_margin_bottom),
        first_vpos(successor),
        "꼬리 조각 종료식"
    );
    // host 글은 표 위 한 줄뿐이다.
    let host = &core.document().sections[0].paragraphs[HOST_PARA];
    assert!(!host.text.trim().is_empty());
    assert_eq!(host.line_segs.len(), 1);

    let tree = core.build_page_render_tree(1).unwrap();
    let mut cells = Vec::new();
    table_cells(&tree.root, &mut cells);
    let (_, height) = cells.first().copied().expect("2쪽 꼬리 조각 칸");
    let expected = px(remainder);
    assert!(
        (height - expected).abs() <= 1.0,
        "꼬리 조각 칸 높이 {height:.2}px 는 선언 나머지 {expected:.2}px (±1)"
    );
    // 가운데 정렬 칸의 첫 글줄은 상자 위끝에서 내려와 있고, 칸 안 순서를 지킨다.
    let first = page_line_top(&mut core, 1, "기타 요건");
    let next = page_line_top(&mut core, 1, "국가관이 투철하고");
    assert!(first < next);
}
