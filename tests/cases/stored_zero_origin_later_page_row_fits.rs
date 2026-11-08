//! 첫 개체 프레임 뒤 행의 원본 원점0 재시작은, 지금 흐름에서 그 행이 남은 예산에
//! 통째로 들어가면 쪽 경계가 아니다 — 합성 입력 계약.
//!
//! 원본 셀 문단의 인접한 두 저장 줄이 모두 `vertical_pos == 0` 이면 저장 당시 그 자리에
//! 쪽 나눔이 있었다는 기록이다(`stored_zero_origin_text_frame_break`). 표의 선언 첫 개체
//! 프레임(`common.height`)을 가로지르는 행의 재시작은 선언 기하가 인증하므로 언제나
//! 물리 경계다. 그러나 첫 프레임 **뒤** 행의 재시작은 저장 당시 쪽 나눔의 기록일 뿐이다.
//! 쪽 나눔이 다른 자리로 옮겨 가 그 행이 쪽 위·중간에 놓이면 그 기록으로 행을 자를 근거가
//! 없다.
//!
//! 독립 근거: `1342000_edu_curriculum_map.hwp` 구역 12 표 행 132 의 칸 「삶의 목적은
//! 무엇인가? (삶의 목적)」은 저장 2줄이 `vpos 0/0` 이다. devel 463c48cc2 는 그 행을 쪽 위
//! 120px 지점에서 끊어 213px 만 쓰는 쪽을 만들었다(전체 418 → 422쪽). 한/글 2022 PDF 는 그
//! 행을 끊지 않는다. 쪽 끝에 걸린 행은 종전대로 재시작에서 자른다 —
//! `table_scattered_header_rowbreak` 47→48쪽 row99 「6가 크로 / 뮴)」(이 수정 뒤에도
//! p47 rows 59..100 endCut [1]).
//!
//! 이 파일은 합성 입력 계약이다(한컴 출력과의 일치 증거가 아니다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::LineSeg;
use rhwp::model::shape::TextWrap;
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::scaffold::{build_scaffold, parse_scaffold_str};
use rhwp::serializer::serialize_hwpx;

const ROWS: usize = 60;
const TARGET_ROW: u16 = 55;
/// 120행 변형에서 둘째 쪽 끝에 걸리는 행(수정 전후 같은 실측).
const PROBE_ROWS: usize = 120;
const PAGE_END_ROW: u16 = 97;
const FIRST_LINE: &str = "가나다라마바";
const SECOND_LINE: &str = "사아자차카타";

fn table_mut(doc: &mut Document) -> &mut Table {
    doc.sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|p| p.controls.iter_mut())
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_mut()),
            _ => None,
        })
        .expect("표")
}

fn spec() -> String {
    spec_rows(ROWS)
}

fn spec_rows(n: usize) -> String {
    let mut rows = vec![r#"["No","항목","설명"]"#.to_string()];
    rows.extend((1..n).map(|i| format!(r#"["{i}","항목 {i}","설명 {i}"]"#)));
    format!(
        r#"{{"version":"1","title":"원점0","blocks":[
            {{"type":"table","rows":[{}]}},
            {{"type":"paragraph","text":"표 다음 문단"}}
        ]}}"#,
        rows.join(",")
    )
}

/// 첫 개체 프레임(`common.height`)을 앞 10행으로 선언하고, 뒤쪽 행 하나에 원본 원점0
/// 재시작 두 줄을 둔 저장 HWPX 를 다시 읽는다.
fn core_with_later_page_restart() -> DocumentCore {
    core_with_restart_at(ROWS, TARGET_ROW)
}

fn core_with_restart_at(rows: usize, target: u16) -> DocumentCore {
    let mut doc = build_scaffold(&parse_scaffold_str(&spec_rows(rows)).expect("명세"));
    let table = table_mut(&mut doc);
    table.page_break = TablePageBreak::RowBreak;
    table.common.treat_as_char = false;
    table.common.text_wrap = TextWrap::TopAndBottom;
    let raw = table.get_raw_row_heights();
    table.common.height = raw[..10].iter().sum();
    let cell = table
        .cells
        .iter_mut()
        .find(|cell| cell.row == target && cell.col == 2)
        .expect("대상 칸");
    let inner = cell.width as i32 - i32::from(cell.padding.left) - i32::from(cell.padding.right);
    let para = &mut cell.paragraphs[0];
    para.text = format!("{FIRST_LINE}{SECOND_LINE}");
    para.char_offsets = (0..para.text.chars().count() as u32).collect();
    para.char_count = para.text.chars().count() as u32 + 1;
    let row = |text_start: u32| LineSeg {
        text_start,
        vertical_pos: 0,
        line_height: 1000,
        text_height: 1000,
        baseline_distance: 850,
        line_spacing: 600,
        column_start: 0,
        segment_width: inner,
        tag: LineSeg::TAG_FIRST_SEGMENT | LineSeg::TAG_LAST_SEGMENT,
    };
    para.line_segs = vec![row(0), row(FIRST_LINE.chars().count() as u32)];
    let two_lines =
        1000 + 600 + 1000 + cell.padding.top.max(0) as u32 + cell.padding.bottom.max(0) as u32;
    for cell in table.cells.iter_mut().filter(|cell| cell.row == target) {
        cell.height = two_lines;
    }
    let bytes = serialize_hwpx(&doc).expect("HWPX 저장");
    DocumentCore::from_bytes(&bytes).expect("다시 읽기")
}

fn page_text(core: &DocumentCore, page: u32) -> String {
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut text = String::new();
    walk(&tree.root, &mut text);
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn later_page_zero_origin_restart_does_not_cut_a_row_that_fits() {
    let core = core_with_later_page_restart();
    let pages = core.page_count();
    let owner = |needle: &str| {
        (0..pages)
            .filter(|&page| page_text(&core, page).contains(needle))
            .collect::<Vec<_>>()
    };
    let first = owner(FIRST_LINE);
    let second = owner(SECOND_LINE);
    assert_eq!(first.len(), 1, "첫 줄 소속 쪽 {first:?}");
    assert_eq!(
        first, second,
        "두 저장 줄이 한 쪽에 남아야 한다 (쪽 {pages})"
    );
    // 행 위·아래 이웃도 같은 흐름에 있다 — 행 하나만 다음 쪽으로 잘려 나가지 않는다.
    assert!(owner(&format!("설명{}", TARGET_ROW + 1)).contains(&first[0]));
}

/// 대조군: 같은 재시작이 쪽 끝에 걸린 행(남은 예산에 두 줄이 다 들어가지 않는다)이면
/// 종전대로 그 재시작에서 자른다 — 첫 줄은 앞 쪽, 둘째 줄은 다음 쪽.
#[test]
fn later_page_zero_origin_restart_still_cuts_a_row_at_the_page_end() {
    let core = core_with_restart_at(PROBE_ROWS, PAGE_END_ROW);
    let pages = core.page_count();
    let owner = |needle: &str| {
        (0..pages)
            .filter(|&page| page_text(&core, page).contains(needle))
            .collect::<Vec<_>>()
    };
    let first = owner(FIRST_LINE);
    let second = owner(SECOND_LINE);
    assert_eq!(first.len(), 1, "첫 줄 소속 쪽 {first:?}");
    assert_eq!(second, vec![first[0] + 1], "둘째 줄은 다음 쪽에서 이어진다");
}
