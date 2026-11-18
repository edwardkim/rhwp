//! 관계형: 저장 줄 없는 문단의 줄 머리 글자처럼 취급 표 뒤 말미 공백이 줄을 따로 만들지 않는다.
//! 주석 작성: 2026-11-04 16:42 KST (새 파일). 측정: 2026-11-04 KST, ai101.
//!
//! # 측정 근거
//!
//! - 방법: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//!   판정값 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 실루엣). 기준 PDF는
//!   `pdf/issue8574/tac_trailing_space_nols-2020.pdf`(한/글 2020 Print, 1쪽).
//! - 원본 `samples/issue8574/tac_trailing_space_nols.hwp` 1쪽(전체 1쪽, PDF 1쪽):
//!   수정 후(7bc812ea81) Native 100.00% · fresh WASM 100.00%, 전쪽 최저 동일.
//!   수정 전(base 1924d371d6) Native 86.21% · fresh WASM 86.01%.
//! - 증거: `mydocs/pr/assets/issue_8574/{native,wasm}/fixture_{base,fix}_silhouette.tsv`,
//!   `mydocs/pr/assets/issue_8574/png/`. 이 검사는 좌표를 고정하지 않고 표·글줄의 앞뒤
//!   관계와 저장본과의 동일 위치만 본다.
//!
//! # 무엇이 깨져 있었나
//!
//! 저장 `LINE_SEG` 가 없는 문단에 줄 머리의 글자처럼 취급 표와 문단 끝 공백 한 칸만 있고,
//! 표 폭 + 공백 폭이 줄 폭을 넘으면 렌더의 인라인 흐름 계획(`inline_flow`)이 그 공백을
//! 새 줄로 넘겼다. 구분 공백 흡수(`spaces.wrap`)는 뒤에 흐름 내용이 있는 공백만 받았기
//! 때문이다. 표 아래에 공백 줄 하나(줄높이 + 줄간격)가 생겨 뒤 문단이 그만큼 내려갔다.
//! 조판 장부도 같은 문단을 공백 줄로 쟀다(`try_compose_lines` 폴백).
//!
//! 한/글은 말미 공백에 줄상자를 주지 않는다(#7500·#7160 `absorb_whitespace_only_rows`).
//!
//! # 기대값의 출처
//!
//! - 합성 입력 `tac_trailing_space.hwpx`(생성기 `gen_tac_trailing_space.py`)를 한/글 2020 이
//!   HWP 로 저장한 것이 `tac_trailing_space-2020.hwp` 다. 한/글은 host 문단에 표 줄 하나
//!   (lh 8000HU, ls 600HU)만 저장한다 — 공백 줄이 없다.
//! - 그 저장본에서 host 문단의 `PARA_LINE_SEG` 만 지운 것이 `tac_trailing_space_nols.hwp` 다
//!   (나머지 문단과 개체는 그대로). 이 파일의 한/글 2020·2024 Print PDF 모두
//!   「표 뒤 첫 줄」 위끝 268.4px, 「끝」 289.6px(96dpi) 로 저장본과 같은 자리다.
//!
//! # 검사하는 것
//!
//! - 저장 줄 없는 host 의 본문 글줄은 표 아래로 내려가지 않는다(표 줄만 있다).
//! - 뒤 문단 첫 줄 위끝 = 표 아래끝 + host 저장 줄간격(600HU) — 저장본의 사다리와 같다.
//! - 뒤 문단 위치가 한/글 저장본(저장 줄 있음)의 같은 문단 위치와 같다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const STORED: &str = "samples/issue8574/tac_trailing_space-2020.hwp";
const NO_LINE_SEG: &str = "samples/issue8574/tac_trailing_space_nols.hwp";
/// 줄 머리 표 + 말미 공백 한 칸 문단.
const HOST_PARA: usize = 1;
/// host 뒤 첫 문단.
const NEXT_PARA: usize = 2;
/// host 저장 줄간격 600HU 를 96dpi 로 환산한 값.
const HOST_LINE_SPACING_PX: f64 = 600.0 * 96.0 / 7200.0;

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("재현물 {sample} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 본문 직속(표 안 제외) 글줄의 (문단, 위끝, 아래끝).
fn body_lines(node: &RenderNode, in_table: bool, out: &mut Vec<(usize, f64, f64)>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if !in_table {
            if let Some(para) = line.para_index {
                out.push((para, node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
    }
    let inside = in_table || matches!(node.node_type, RenderNodeType::Table(_));
    for child in &node.children {
        body_lines(child, inside, out);
    }
}

/// host 문단의 본문 직속 표 (위끝, 아래끝).
fn host_table(node: &RenderNode) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(HOST_PARA) {
            return Some((node.bbox.y, node.bbox.y + node.bbox.height));
        }
        return None;
    }
    node.children.iter().find_map(host_table)
}

struct Layout {
    table: (f64, f64),
    lines: Vec<(usize, f64, f64)>,
}

fn layout(sample: &str) -> Layout {
    let doc = load(sample);
    assert_eq!(doc.page_count(), 1, "{sample}: 한/글 Print 는 1쪽");
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let table = host_table(&tree.root).expect("host 표");
    let mut lines = Vec::new();
    body_lines(&tree.root, false, &mut lines);
    Layout { table, lines }
}

fn next_para_top(layout: &Layout, sample: &str) -> f64 {
    layout
        .lines
        .iter()
        .filter(|(para, ..)| *para == NEXT_PARA)
        .map(|(_, top, _)| *top)
        .fold(None, |acc: Option<f64>, top| {
            Some(acc.map_or(top, |a| a.min(top)))
        })
        .unwrap_or_else(|| panic!("{sample}: 뒤 문단 글줄이 없다"))
}

#[test]
fn trailing_space_after_leading_tac_table_has_no_own_line() {
    let no_ls = layout(NO_LINE_SEG);
    let (_, table_bottom) = no_ls.table;
    let below: Vec<_> = no_ls
        .lines
        .iter()
        .filter(|(para, top, _)| *para == HOST_PARA && *top + 0.5 >= table_bottom)
        .collect();
    assert!(
        below.is_empty(),
        "host 의 말미 공백이 표 아래(아래끝 {table_bottom:.1})에 자기 줄을 만든다: {below:?}"
    );
}

#[test]
fn next_paragraph_follows_table_line_spacing() {
    let no_ls = layout(NO_LINE_SEG);
    let (_, table_bottom) = no_ls.table;
    let next_top = next_para_top(&no_ls, NO_LINE_SEG);
    let gap = next_top - table_bottom;
    assert!(
        (gap - HOST_LINE_SPACING_PX).abs() <= 0.5,
        "뒤 문단은 표 아래끝 + host 줄간격({HOST_LINE_SPACING_PX:.1}px)에서 시작해야 한다: \
         표 아래끝 {table_bottom:.1}, 뒤 문단 위끝 {next_top:.1}, 간격 {gap:.1}"
    );
}

#[test]
fn missing_host_line_seg_lands_where_hancom_stored_it() {
    let stored = layout(STORED);
    let no_ls = layout(NO_LINE_SEG);
    let stored_top = next_para_top(&stored, STORED);
    let no_ls_top = next_para_top(&no_ls, NO_LINE_SEG);
    assert!(
        (stored_top - no_ls_top).abs() <= 0.5,
        "저장 줄 없는 host 뒤 문단({no_ls_top:.1})이 한/글 저장본({stored_top:.1})과 달라졌다"
    );
    assert!(
        (stored.table.0 - no_ls.table.0).abs() <= 0.5
            && (stored.table.1 - no_ls.table.1).abs() <= 0.5,
        "host 표 자리가 저장본과 다르다: 저장본 {:?}, 저장 줄 없음 {:?}",
        stored.table,
        no_ls.table
    );
}
