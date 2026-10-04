//! [#7418] 저장 줄 없는 문서에서 표 host 글·칸 조각의 기하를 한/글 출력에 맞춘다.
//!
//! 모든 기대값은 같은 원본의 한/글 PDF(`pdf/` 정본)에서 잰 좌표다(px@96dpi, 허용 2px —
//! Visual Sweep 실루엣 판정과 같은 관용). 두 문서 모두 HWP5 저장본이지만 본문·칸 문단에
//! `LINE_SEG` 가 없어 rhwp 가 직접 조판한다.
//!
//! | 검사 | 원인 | 수정 전 |
//! |---|---|---|
//! | 78494 9쪽 표 위끝 | 문단 기준 오프셋의 기준점이 host 글줄 **뒤** | 229.5 |
//! | 78494 20쪽 줄 끝 | 글자 단위 모드에서 숫자→한글 경계를 끊지 못함 | `…평균연봉은` |
//! | 78494 20쪽·76076 34쪽 괘선 위끝 | 2열 표 이어진 조각이 바깥 위 여백을 안 연다 | 75.6 |
//! | 70833 12쪽 행 5·6 위끝 | 선언 높이 축소 가드·TAC host 줄간격 누락 | 596.3 / 967.0 |
//! | 70833 12쪽 4행 첫 줄 x | 칸 여백 축소가 마지막 줄간격까지 세었다 | 241.0 |
//! | 70833 13쪽 이어진 줄 x | 이어진 조각이 목록 마커 영역을 잃었다 | 246.8 |
//! | 36384689 칸 안 표 뒤 문단 | 합성 줄이 품은 host 줄간격을 한 번 더 더했다 | 408.5 |
//!
//! 70833 의 쪽 수·본문 없는 쪽은 `issue_6854_empty_para_orphan_page` 가 본다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const DOC_78494: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
const DOC_70833: &str = "samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp";
const DOC_76076: &str = "samples/76076_regulatory_analysis.hwp";
const DOC_36384689: &str =
    "samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx";

fn load(doc: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(doc);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{doc} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{doc} 로드: {e:?}"))
}

fn page(core: &DocumentCore, page: u32) -> RenderNode {
    core.build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1))
        .root
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

/// 최상위 표(`para_index`)의 bbox.
fn table_top(root: &RenderNode, para_index: usize) -> f64 {
    nodes(root)
        .into_iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Table(t) if t.para_index == Some(para_index) => Some(n.bbox.y),
            _ => None,
        })
        .unwrap_or_else(|| panic!("표 pi={para_index} 가 이 쪽에 없다"))
}

/// 최상위 표의 행 위끝 — 첫 열 칸의 bbox.
fn row_top(root: &RenderNode, para_index: usize, row: u16) -> f64 {
    let all = nodes(root);
    let table = all
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(para_index)))
        .unwrap_or_else(|| panic!("표 pi={para_index} 가 이 쪽에 없다"));
    table
        .children
        .iter()
        .find_map(|c| match &c.node_type {
            RenderNodeType::TableCell(cell) if cell.row == row && cell.col == 0 => Some(c.bbox.y),
            _ => None,
        })
        .unwrap_or_else(|| panic!("행 {row} 의 첫 칸이 없다"))
}

/// 글자 run 을 y 로 묶은 줄: (y, 줄 첫 run x, 공백 뺀 글).
fn lines(root: &RenderNode) -> Vec<(f64, f64, String)> {
    let mut runs: Vec<(f64, f64, String)> = nodes(root)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some((n.bbox.y, n.bbox.x, run.text.clone())),
            _ => None,
        })
        .collect();
    runs.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());
    let mut out: Vec<(f64, f64, String)> = Vec::new();
    for (y, x, text) in runs {
        match out.last_mut() {
            Some((ly, lx, buf)) if (*ly - y).abs() < 0.5 => {
                buf.push_str(&text);
                *lx = lx.min(x);
            }
            _ => out.push((y, x, text)),
        }
    }
    out.into_iter()
        .map(|(y, x, t)| (y, x, t.chars().filter(|c| !c.is_whitespace()).collect()))
        .collect()
}

fn near(actual: f64, expected: f64, tol: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{what}: rhwp {actual:.1} · 한/글 {expected:.1} (허용 {tol})"
    );
}

/// host 글줄(공백 한 칸) 뒤의 문단 기준 자리차지 표는 **문단 상단**에서 오프셋을 잰다.
/// 한/글: host 줄 175.9..195.9(줄간격 12px), 표 위끝 199.3 = 175.9 + 21.65 + 바깥 여백 1.9.
#[test]
fn para_float_after_host_text_measures_offset_from_the_paragraph_top() {
    let core = load(DOC_78494);
    let root = page(&core, 8);
    near(table_top(&root, 90), 199.3, 2.0, "78494 9쪽 표 pi=90 위끝");
}

/// 글자 단위 한글 문단은 숫자 토큰과 뒤 한글 사이에서도 끊는다(`3,588|만원이며,`).
#[test]
fn hangul_char_mode_breaks_between_a_number_and_the_following_hangul() {
    let core = load(DOC_78494);
    let root = page(&core, 19);
    let all = lines(&root);
    let line = all
        .iter()
        .find(|(_, _, t)| t.starts_with("-「임금직무정보시스템」"))
        .unwrap_or_else(|| {
            panic!(
                "대상 줄이 없다: {:?}",
                all.iter().map(|l| &l.2).collect::<Vec<_>>()
            )
        });
    assert_eq!(
        line.2, "-「임금직무정보시스템」기타사무직의평균연봉은3,588",
        "한/글 20쪽의 이 줄은 `3,588` 로 끝나고 다음 줄이 `만원이며,` 로 시작한다"
    );
}

/// 마지막 행 안에서 이어지는 2열 표 조각은 바깥 위 여백(141 HU)을 연다.
#[test]
fn two_column_terminal_row_continuation_reopens_the_outer_top_margin() {
    let core = load(DOC_78494);
    near(
        table_top(&page(&core, 19), 193),
        77.5,
        0.6,
        "78494 20쪽 표 pi=193 괘선 위끝",
    );

    let core = load(DOC_76076);
    near(
        table_top(&page(&core, 33), 325),
        77.4,
        0.6,
        "76076 34쪽 표 pi=325 괘선 위끝",
    );
}

/// 칸 안 중첩 표를 품은 행이 선언 높이 축소에 눌리지 않고, TAC 표 host 줄 뒤 줄간격까지
/// 행에 넣어 한/글처럼 12쪽 바닥에서 나뉜다.
#[test]
fn nested_table_row_keeps_its_content_height_and_host_line_spacing() {
    let core = load(DOC_70833);
    let root = page(&core, 11);
    near(
        row_top(&root, 83, 5),
        601.3,
        2.0,
        "70833 12쪽 표 pi=83 5행 위끝",
    );
    near(
        row_top(&root, 83, 6),
        989.0,
        2.0,
        "70833 12쪽 표 pi=83 6행 위끝",
    );
}

/// 줄바꿈 결과가 칸에 들어가면 칸 안 여백을 깎지 않는다 — 마지막 줄의 줄간격은 칸을
/// 채우지 않는다. 한/글 4행 첫 줄(`☞ 업종과 무관하게…`) x = 246.7.
#[test]
fn cell_padding_is_kept_when_wrapped_lines_fit_without_the_last_line_spacing() {
    let core = load(DOC_70833);
    let root = page(&core, 11);
    let all = lines(&root);
    let line = all
        .iter()
        .find(|(_, _, t)| t.contains("업종과무관하게"))
        .expect("4행 첫 줄");
    near(line.1, 246.7, 1.0, "70833 12쪽 4행 첫 줄 x");
}

/// 쪽을 넘어 이어지는 칸 문단의 줄은 글머리표 영역만큼 들어간 자리에서 시작한다.
#[test]
fn continued_list_paragraph_lines_keep_the_marker_area() {
    let core = load(DOC_70833);
    let root = page(&core, 12);
    let all = lines(&root);
    let line = all
        .iter()
        .find(|(_, _, t)| t.starts_with("확인이필요한"))
        .expect("13쪽 이어진 줄");
    near(line.1, 266.7, 1.5, "70833 13쪽 이어진 줄 x");
}

/// host 줄 뒤 줄간격은 한 번만 붙는다. 36384689 칸[2] 의 TAC 표 host 문단은 저장 줄이 없어
/// rhwp 가 줄을 합성하고, 그 합성 줄의 줄간격(600 HU)을 배치가 이미 전진시킨다. 뒤 문단은
/// 한/글 저장 사다리 그대로 host 줄 위끝 + vertpos 30040 HU(400.5px) 에 선다 — 저장 줄이
/// **아예 없는** host(70833)에만 줄간격을 따로 더한다.
#[test]
fn host_line_spacing_is_not_added_again_over_a_synthesized_host_line() {
    let core = load(DOC_36384689);
    let root = page(&core, 0);
    let host_top = nodes(&root)
        .into_iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Table(t) if t.row_count == 8 && t.col_count == 3 => Some(n.bbox.y),
            _ => None,
        })
        .expect("칸[2] 의 8×3 중첩 표");
    let all = lines(&root);
    let line = all
        .iter()
        .find(|(_, _, t)| t.starts_with("붙임"))
        .expect("`붙임` 줄");
    near(
        line.0 - host_top,
        30040.0 * 96.0 / 7200.0,
        1.0,
        "36384689 `붙임` 줄 − host 줄 위끝",
    );

    // 셀 글줄을 새로 계산한 표도 옛 host 줄높이로 다시 압축해서는 안 된다.
    // 같은 원본의 독립 한컴 PDF는 2쪽이며, 표 뒤 문단은 두 번째 쪽에만 있다.
    // Native/fresh WASM 전체 두 쪽의 최소 일치율 90.86083%를 확인한 입력이다.
    let ladder = load("samples/task2070/hy_ladder3.hwpx");
    assert_eq!(
        ladder.page_count(),
        2,
        "실측 표 뒤 문단은 독립 PDF처럼 새 쪽이어야 한다"
    );
    let first = page(&ladder, 0);
    let second = page(&ladder, 1);
    assert!(
        lines(&first)
            .iter()
            .all(|(_, _, text)| !text.contains("NEXTPARAGRAPH")),
        "표 뒤 문단이 첫 쪽의 표와 함께 쪽 밖으로 밀렸다"
    );
    assert_eq!(
        lines(&second)
            .iter()
            .filter(|(_, _, text)| text == "NEXTPARAGRAPH")
            .count(),
        1,
        "표 뒤 문단을 두 번째 쪽에 한 번 보존해야 한다"
    );
    assert!(
        nodes(&first)
            .iter()
            .any(|node| matches!(node.node_type, RenderNodeType::Table(_))),
        "표 본체는 첫 쪽에 남아야 한다"
    );
    assert!(
        nodes(&second)
            .iter()
            .all(|node| !matches!(node.node_type, RenderNodeType::Table(_))),
        "원본의 글자처럼 취급 표를 임의로 분할하지 않는다"
    );
}
