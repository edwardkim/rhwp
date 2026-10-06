//! [#7548 3단계] 저장 LineSeg 가 없는(재조판) 본문은 쪽 기준 어울림(Square) 표 상자를
//! 배제 영역으로 피한다 — 옆 공간이 있으면 띠와 겹치는 줄만 표 옆 차선으로 좁히고,
//! 쓸 수 있는 폭이 없으면 띠 아래로 넘긴다.
//!
//! 공개 합성 입력은 결정적 생성기 `samples/page_anchored_square/make_page_anchored_square.py`
//! 의 출력이며 linesegarray 가 없다. 정본은 같은 입력을 한/글 2020(hwp2024Convert MCP
//! engine 2020, Hancom 11.0.0.9136)으로 출력한 PDF 다.
//! - `page_anchored_square_reflow.hwpx` (정본 `page_anchored_square-2020.pdf`, 같은 입력의
//!   재출력과 래스터 동일): 표 폭 = 본문 폭. pi=3 host·pi=4~7 은 표 위, 띠와 겹치는 pi=8 부터
//!   표 아래.
//! - `..._narrow.hwpx` (`page_anchored_square_reflow_narrow-2020.pdf`): 표 폭 = 본문 절반,
//!   왼쪽 정렬. 띠와 겹치는 pi=8~12 는 표 오른쪽 차선, pi=13 은 표 아래 전폭.
//! - `..._allow_overlap.hwpx` (`page_anchored_square_reflow_allow_overlap-2020.pdf`):
//!   allowOverlap=1 이어도 한/글 배치는 기본형과 같다(개체끼리의 겹침 허용은 글 배제를
//!   취소하지 않는다).
//!
//! - `samples/para_square_lane/para_square_lane_reflow.hwpx` (`pdf/para_square_lane/
//!   para_square_lane_reflow-2020.pdf`, 21_언어 14쪽 형상): 문단 기준 좁은 어울림 표의 host 뒤
//!   문단 pi=3 은 표 띠와 겹치는 세 줄 모두 표 오른쪽 차선, pi=4 는 표 아래 전폭.
//!   (생성기 `full_fontfaces` 인자 — 뼈대의 LATIN fontface 미선언으로 한/글이 라틴을
//!   Haansoft Batang 으로 그리는 변수를 없앤 입력이다.)
//!
//! 수정 전 rhwp 는 host 글자를 표 아래로 보내 한 줄 비우고(pi=3), 뒤 문단의 합성 줄을
//! 저장 증거로 여겨 표 위에 겹쳐 그렸다. 검사는 절대 좌표가 아니라 관계(표 위/아래·옆
//! 차선·순서·겹침 없음)로 한다. 저장 LineSeg 입력(`page_anchored_square.hwp`)은
//! `issue_7548_page_anchored_square` 가 무변화를 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const BASE: &str = "samples/page_anchored_square/page_anchored_square_reflow.hwpx";
const NARROW: &str = "samples/page_anchored_square/page_anchored_square_reflow_narrow.hwpx";
const ALLOW_OVERLAP: &str =
    "samples/page_anchored_square/page_anchored_square_reflow_allow_overlap.hwpx";
const PARA_LANE: &str = "samples/para_square_lane/para_square_lane_reflow.hwpx";
const HOST_PARA: usize = 3;
const LAST_PARA: usize = 13;

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Debug, Default)]
struct Page {
    body_x: Option<f64>,
    table: Option<Rect>,
    /// (문단, 줄 상자) — 본문 줄만(표 셀 안 줄 제외).
    lines: Vec<(usize, Rect)>,
}

fn collect(node: &RenderNode, in_table: bool, page: &mut Page) {
    let mut in_table = in_table;
    let rect = Rect {
        x: node.bbox.x,
        y: node.bbox.y,
        w: node.bbox.width,
        h: node.bbox.height,
    };
    match &node.node_type {
        RenderNodeType::Body { .. } => {
            page.body_x.get_or_insert(node.bbox.x);
        }
        RenderNodeType::Table(_) => {
            if !in_table {
                page.table = Some(rect);
            }
            in_table = true;
        }
        RenderNodeType::TextLine(tl) if !in_table => {
            if let Some(para) = tl.para_index {
                page.lines.push((para, rect));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, in_table, page);
    }
}

fn render(sample: &str) -> Page {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e}"));
    assert_eq!(core.page_count(), 1, "{sample}: 한/글 정본은 1쪽이다");
    let tree = core.build_page_render_tree(0).expect("render 1쪽");
    let mut page = Page::default();
    collect(&tree.root, false, &mut page);
    page
}

fn page(sample: &str) -> Page {
    let page = render(sample);
    let paras: Vec<usize> = page.lines.iter().map(|(p, _)| *p).collect();
    assert_eq!(
        paras,
        (0..=LAST_PARA).collect::<Vec<_>>(),
        "{sample}: 모든 본문 문단이 한 줄씩 순서대로 한 번만 그려진다"
    );
    page
}

fn line(page: &Page, para: usize) -> Rect {
    page.lines
        .iter()
        .find(|(p, _)| *p == para)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("pi={para} 본문 줄"))
}

fn assert_flow_order(sample: &str, page: &Page) {
    for pair in page.lines.windows(2) {
        let ((pa, a), (pb, b)) = (pair[0], pair[1]);
        assert!(
            b.y >= a.y + a.h - 0.5,
            "{sample}: pi={pb} 는 pi={pa} 아래에서 시작한다(겹침 없음): {:.1} < {:.1}",
            b.y,
            a.y + a.h
        );
    }
}

fn assert_full_width_band_goes_below(sample: &str) {
    let page = page(sample);
    let table = page.table.expect("쪽 기준 어울림 표");
    let body_x = page.body_x.expect("본문 영역");
    assert_flow_order(sample, &page);
    for para in HOST_PARA..=7 {
        let r = line(&page, para);
        assert!(
            r.y + r.h <= table.y + 0.5,
            "{sample}: pi={para} 는 표 띠 위에 놓인다: line bottom {:.1}, table top {:.1}",
            r.y + r.h,
            table.y
        );
    }
    // host 글자는 표 아래로 가지 않는다: host 는 앞 문단 바로 다음 줄이다.
    let prev = line(&page, HOST_PARA - 1);
    let host = line(&page, HOST_PARA);
    let next = line(&page, HOST_PARA + 1);
    assert!(
        ((host.y - prev.y) - (next.y - host.y)).abs() < 0.5,
        "{sample}: host 줄은 빈 줄 없이 앞뒤 문단과 같은 간격이다: {:.1} / {:.1}",
        host.y - prev.y,
        next.y - host.y
    );
    for para in 8..=LAST_PARA {
        let r = line(&page, para);
        assert!(
            r.y >= table.y + table.h - 0.5,
            "{sample}: 띠와 겹치는 pi={para} 는 표 아래로 넘어간다: line top {:.1}, table bottom {:.1}",
            r.y,
            table.y + table.h
        );
        assert!(
            (r.x - body_x).abs() < 0.5,
            "{sample}: 표 아래 pi={para} 는 전폭(본문 왼쪽)에서 시작한다"
        );
    }
}

#[test]
fn issue_7548_reflow_full_width_page_square_band_moves_lines_below() {
    assert_full_width_band_goes_below(BASE);
}

#[test]
fn issue_7548_reflow_allow_overlap_still_excludes_text() {
    assert_full_width_band_goes_below(ALLOW_OVERLAP);
}

#[test]
fn issue_7548_reflow_narrow_page_square_band_narrows_overlapping_lines() {
    let page = page(NARROW);
    let table = page.table.expect("쪽 기준 어울림 표");
    let body_x = page.body_x.expect("본문 영역");
    assert_flow_order(NARROW, &page);
    for para in HOST_PARA..=7 {
        let r = line(&page, para);
        assert!(
            r.y + r.h <= table.y + 0.5 && (r.x - body_x).abs() < 0.5,
            "{NARROW}: pi={para} 는 표 위 전폭 줄이다: {r:?}, table {table:?}"
        );
    }
    let mut beside = 0;
    for para in 8..LAST_PARA {
        let r = line(&page, para);
        assert!(
            r.x >= table.x + table.w,
            "{NARROW}: 띠와 겹치는 pi={para} 는 표 오른쪽 차선에서 시작한다: x {:.1}, table right {:.1}",
            r.x,
            table.x + table.w
        );
        beside += 1;
    }
    assert_eq!(beside, LAST_PARA - 8);
    let first_beside = line(&page, 8);
    assert!(
        first_beside.y < table.y + table.h,
        "{NARROW}: 옆 공간이 있으면 pi=8 은 띠 아래로 밀리지 않고 표 옆에 남는다: {first_beside:?}, table {table:?}"
    );
    let last = line(&page, LAST_PARA);
    assert!(
        last.y >= table.y + table.h - 0.5 && (last.x - body_x).abs() < 0.5,
        "{NARROW}: 띠 아래 pi={LAST_PARA} 는 전폭으로 돌아온다: {last:?}, table {table:?}"
    );
}

#[test]
fn issue_7548_reflow_para_square_successor_narrows_beside_table() {
    let page = render(PARA_LANE);
    let table = page.table.expect("문단 기준 어울림 표");
    let body_x = page.body_x.expect("본문 영역");
    assert_flow_order(PARA_LANE, &page);
    let paras: Vec<usize> = page.lines.iter().map(|(p, _)| *p).collect();
    assert_eq!(
        paras,
        vec![0, 1, 2, 2, 3, 3, 3, 4],
        "{PARA_LANE}: 한/글 정본과 같은 줄 수(host 2줄, 다음 문단 3줄)로 순서대로 그린다"
    );
    let table_right = table.x + table.w;
    for (para, r) in page.lines.iter().filter(|(p, _)| matches!(p, 2 | 3)) {
        assert!(
            r.y < table.y + table.h && r.x >= table_right,
            "{PARA_LANE}: 띠와 겹치는 pi={para} 줄은 표 오른쪽 차선이다: {r:?}, table {table:?}"
        );
    }
    let last = line(&page, 4);
    assert!(
        last.y >= table.y + table.h - 0.5 && (last.x - body_x).abs() < 0.5,
        "{PARA_LANE}: 띠 아래 pi=4 는 전폭으로 돌아온다: {last:?}, table {table:?}"
    );
}
