//! [#8684] 관계형·고정형 회귀 — 누름틀 시작이 아닌 폭 없는 확장 제어 문자(책갈피·찾아보기 표식)
//! 뒤에 남은 줄 폭보다 넓은 글자처럼 취급 표가 와도 한/글은 제어 문자만 담은 첫 줄을 두고
//! 표를 다음 줄에서 시작한다. 한/글이 다시 저장한 책갈피 줄도 같은 자리에 그린다.
//!
//! 주석 갱신: 2026-11-28 15:23 KST(+09:00), 새 파일.
//!
//! 입력(모두 `samples/issue8773/field_led_wide_table.hwpx` = 공개 결재문서 36479199 에서 만든
//! 합성본, 문단 5 의 누름틀 시작·끝 제어만 바꾸고 다른 저장값은 그대로):
//! - `samples/issue8684/bookmark_led_wide_table.hwpx` — 누름틀 시작 → 책갈피, 누름틀 끝 삭제.
//!   문단 = [책갈피][16×16 표(바깥 50346HU > 줄 폭 49608HU)][공백], 저장 줄 없음(rhwp 재조판).
//! - `samples/issue8684/bookmark_only_wide_table.hwpx` — 위에서 끝 공백도 지운 글자 없는 host.
//! - `samples/issue8684/indexmark_only_wide_table.hwpx` — 글자 없는 host, 책갈피 대신 찾아보기 표식.
//! - `samples/issue8684/bookmark_*_wide_table_hancom.hwp` — 위 책갈피 두 입력을 원격 한/글
//!   2020(11.0.0.9172)이 HWP 로 다시 저장한 것. 저장 줄 2개(첫 줄 ts 0 글자 높이 1100HU, 표 줄
//!   ts 8 vpos +1320HU). 책갈피 첫 줄은 lh 57408(표 높이)·th 1100 으로 적힌다.
//! - `samples/issue8684/field_only_wide_table_hancom.hwp` — 같은 실험의 누름틀만 있는 글자 없는
//!   host 재저장본(첫 줄 lh = th = 1100). base devel 과 같이 표가 첫 줄 원점에 남는 기존 누름틀
//!   경로의 결함이라 이 검사에 넣지 않고 측정만 기록한다(아래).
//!
//! 독립 기준: `pdf/issue8684/<입력>-2020.pdf` — 원격 한/글 2020 11.0.0.9172 Print → Hancom PDF
//! 1.3.0.550, 각 2쪽. 1쪽 표 위치는 모든 입력이 #8773 누름틀 원본의 한/글 출력과 같다(래스터 동일).
//! 같은 실험에서 하이퍼링크·새 번호·감추기·쪽 번호 위치·쪽 번호 제어도 두 줄, 단 정의는 한 줄이다.
//!
//! 측정: 2026-11-28, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), base devel `ca96e9f342`,
//! 수정 `47238a00cf`. 값은 1쪽 Native / fresh WASM(수정 전 Native / fresh WASM), 2쪽은 모두 같다.
//! - `bookmark_led_wide_table.hwpx`·`bookmark_only_wide_table.hwpx`·`indexmark_only_wide_table.hwpx`:
//!   1쪽 100.00 / 98.89(49.05 / 50.23), 2쪽 94.15 / 94.50(수정 전과 같음).
//! - `bookmark_led_wide_table_hancom.hwp`·`bookmark_only_wide_table_hancom.hwp`: 1쪽 100.00 / 98.89
//!   (50.52 / 51.52), 2쪽 94.15 / 94.50.
//! - `field_only_wide_table_hancom.hwp`(검사 밖): 1쪽 50.52 / 51.52, 수정 전과 같음.
//! - 모든 입력 전체 2쪽 = PDF 2쪽. 검사 입력의 전쪽 최저 Native 94.15·fresh WASM 94.50(둘 다 2쪽).
//!   2쪽은 이 수정이 바꾸지 않은 #8773 원본과 같은 쪽이며 HY 계열 본문 글꼴을 공급하지 못한
//!   글꼴 차이만 남는다(#8773 증거 94.25 / 94.28). 같은 날 같은 fresh WASM 으로 잰 #8773 누름틀
//!   원본 1쪽도 base·수정 모두 98.89 라, WASM 1쪽 98.89 는 이 수정과 무관한 현재 WASM 글꼴 값이다.
//! - 증거: `mydocs/pr/assets/issue_8684/{native,native-base,wasm,wasm-base}/`(TSV·review·overlay PNG).
//!
//! 검사: 원시 순서(제어 문자가 표 앞)와 저장 줄 조건을 입력 조건으로 확인한다. 표 위끝 = #8773
//! 누름틀 원본의 표 위끝(관계형, ±1px), 재조판 줄은 첫 줄에 글자 없음(관계형), 표는 앞 결재 표
//! 아래(관계형), 쪽수 = 한/글 2쪽(고정형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIELD_LED: &str = "samples/issue8773/field_led_wide_table.hwpx";
/// 표를 담은 본문 문단(구역 0).
const HOST: usize = 5;

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"))
}

fn find_table(node: &RenderNode, rows: u16, cols: u16) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.row_count == rows && table.col_count == cols {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, rows, cols))
}

fn previous_tables_bottom(node: &RenderNode) -> f64 {
    // 문단 4 의 두 3행 표(결재란)가 본문 표 바로 앞 흐름이다.
    let mut bottom: f64 = 0.0;
    for (rows, cols) in [(3, 2), (3, 4)] {
        let table = find_table(node, rows, cols).expect("문단 4 결재 표");
        bottom = bottom.max(table.bbox.y + table.bbox.height);
    }
    bottom
}

fn body_table_top(core: &DocumentCore, sample: &str) -> f64 {
    let page = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("{sample} 1쪽: {e:?}"));
    let table = find_table(&page.root, 16, 16).expect("16×16 본문 표");
    assert!(
        table.bbox.y >= previous_tables_bottom(&page.root) - 0.5,
        "{sample}: 본문 표 {:.1} 는 앞 결재 표 아래",
        table.bbox.y
    );
    table.bbox.y
}

/// 표 앞 제어가 모두 `expected` 종류이고 표가 그 뒤에 있다.
fn assert_marker_before_table(core: &DocumentCore, sample: &str, expected: fn(&Control) -> bool) {
    let para = &core.document().sections[0].paragraphs[HOST];
    let table_index = para
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)))
        .unwrap_or_else(|| panic!("{sample}: 문단 5 표"));
    assert!(
        table_index > 0 && para.controls[..table_index].iter().all(expected),
        "{sample}: 원시 순서 = 제어 문자 → 표"
    );
}

fn is_bookmark(control: &Control) -> bool {
    matches!(control, Control::Bookmark(_))
}

fn is_index_mark(control: &Control) -> bool {
    matches!(control, Control::IndexMark(_))
}

fn field_led_table_top() -> f64 {
    let core = load(FIELD_LED);
    body_table_top(&core, FIELD_LED)
}

fn assert_same_row_as_field_led(sample: &str, expected: fn(&Control) -> bool) -> DocumentCore {
    let core = load(sample);
    assert_marker_before_table(&core, sample, expected);
    assert_eq!(core.page_count(), 2, "{sample}: 쪽수 = 한/글 2쪽");
    let top = body_table_top(&core, sample);
    let reference = field_led_table_top();
    assert!(
        (top - reference).abs() <= 1.0,
        "{sample}: 표 위끝 {top:.1} = 누름틀 원본 표 위끝 {reference:.1}(제어 문자 줄 아래)"
    );
    core
}

/// 재조판한 줄: 첫 줄은 제어 문자만 담고 글자는 모두 표 줄 이후다.
fn assert_reflowed_marker_row(core: &DocumentCore, sample: &str) {
    let para = &core.document().sections[0].paragraphs[HOST];
    assert_eq!(para.line_segs.len(), 2, "{sample}: 제어 문자 줄 + 표 줄");
    let second = para.line_segs[1].text_start;
    assert!(
        para.char_offsets.iter().all(|&offset| offset >= second),
        "{sample}: 글자는 표 줄 소속 {:?} ≥ {second}",
        para.char_offsets
    );
}

/// 한/글 저장 줄: 첫 줄 textpos 0(글자 높이), 표 줄 textpos 8.
fn assert_hancom_marker_row(core: &DocumentCore, sample: &str) {
    let para = &core.document().sections[0].paragraphs[HOST];
    assert_eq!(para.line_segs.len(), 2, "{sample}: 한/글 저장 줄 2개");
    let (first, table_row) = (&para.line_segs[0], &para.line_segs[1]);
    assert_eq!(
        (first.text_start, table_row.text_start),
        (0, 8),
        "{sample}: 제어 문자 줄 + 표 줄"
    );
    assert!(
        first.text_height < table_row.text_height
            && table_row.vertical_pos
                == first.vertical_pos + first.text_height + first.line_spacing,
        "{sample}: 첫 줄은 글자 높이 줄, 표 줄은 그 바로 아래"
    );
}

#[test]
fn bookmark_before_wide_table_reflows_to_the_next_row() {
    let sample = "samples/issue8684/bookmark_led_wide_table.hwpx";
    let core = assert_same_row_as_field_led(sample, is_bookmark);
    assert_reflowed_marker_row(&core, sample);
}

#[test]
fn textless_bookmark_and_index_mark_hosts_reflow_the_table_to_the_next_row() {
    for (sample, expected) in [
        (
            "samples/issue8684/bookmark_only_wide_table.hwpx",
            is_bookmark as fn(&Control) -> bool,
        ),
        (
            "samples/issue8684/indexmark_only_wide_table.hwpx",
            is_index_mark,
        ),
    ] {
        let core = assert_same_row_as_field_led(sample, expected);
        let para = &core.document().sections[0].paragraphs[HOST];
        assert!(para.text.is_empty(), "{sample}: 글자 없는 host");
        assert_eq!(para.line_segs.len(), 2, "{sample}: 제어 문자 줄 + 표 줄");
    }
}

#[test]
fn hancom_stored_marker_rows_keep_the_table_on_the_second_row() {
    for (sample, expected) in [
        (
            "samples/issue8684/bookmark_led_wide_table_hancom.hwp",
            is_bookmark as fn(&Control) -> bool,
        ),
        (
            "samples/issue8684/bookmark_only_wide_table_hancom.hwp",
            is_bookmark,
        ),
    ] {
        let core = assert_same_row_as_field_led(sample, expected);
        assert_hancom_marker_row(&core, sample);
    }
}
