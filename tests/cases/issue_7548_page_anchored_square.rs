//! [#7548 2단계] 쪽(PAGE) 기준 어울림 표는 host 문단에 본문이 있어도 절대 위치에 놓이고
//! 흐름을 밀지 않는다.
//!
//! 공개 합성 입력 `samples/page_anchored_square/page_anchored_square.hwp`: 생성기
//! (`make_page_anchored_square.py`) 출력 HWPX 를 한/글 2020(hwp2024Convert MCP engine 2020)으로
//! HWP 저장한 것이라 저장 LineSeg 는 한/글이 쓴 값이다. 정본
//! `pdf/page_anchored_square/page_anchored_square-2020.pdf` 에서 표(3x4, vertRelTo=PAGE
//! vertOffset=13000)는 본문 위에서 offset 만큼 아래에, host 뒤 pi=4~7 은 표 위에, 표 띠와
//! 겹치는 pi=8 부터는 띠 아래(저장 vpos 19992)에 놓인다.
//!
//! 수정 전 rhwp 는 표를 host 흐름 위치(158.5px)에 그리고 host 뒤 문단을 표 아래로 밀었다.
//! 검사는 절대 픽셀이 아니라 관계(본문 상단 + 오프셋, 저장 vpos 간격, 띠 위/아래)로 한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/page_anchored_square/page_anchored_square.hwp";
const HOST_PARA: usize = 3;
const VERT_OFFSET_HU: f64 = 13000.0;
const HU_PER_PX: f64 = 75.0;

#[derive(Debug, Default)]
struct Page {
    body_top: Option<f64>,
    table: Option<(f64, f64)>,
    /// (문단, y, 저장 vpos) — 본문 줄만(표 셀 안 줄 제외).
    lines: Vec<(usize, f64, i32)>,
}

fn collect(node: &RenderNode, in_table: bool, page: &mut Page) {
    let mut in_table = in_table;
    match &node.node_type {
        RenderNodeType::Body { .. } => {
            page.body_top.get_or_insert(node.bbox.y);
        }
        RenderNodeType::Table(_) => {
            if !in_table {
                page.table = Some((node.bbox.y, node.bbox.y + node.bbox.height));
            }
            in_table = true;
        }
        RenderNodeType::TextLine(tl) if !in_table => {
            if let (Some(para), Some(vpos)) = (tl.para_index, tl.vpos) {
                page.lines.push((para, node.bbox.y, vpos));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, in_table, page);
    }
}

fn page() -> Page {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read page_anchored_square fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse page_anchored_square fixture");
    let tree = core.build_page_render_tree(0).expect("render 1쪽");
    let mut page = Page::default();
    collect(&tree.root, false, &mut page);
    page
}

fn line(page: &Page, para: usize) -> (f64, i32) {
    page.lines
        .iter()
        .find(|(p, _, _)| *p == para)
        .map(|(_, y, vpos)| (*y, *vpos))
        .unwrap_or_else(|| panic!("pi={para} 본문 줄이 1쪽에 있어야 한다"))
}

#[test]
fn issue_7548_page_anchored_square_table_top_is_body_top_plus_offset() {
    let page = page();
    let body_top = page.body_top.expect("본문 영역");
    let (table_top, _) = page.table.expect("쪽 기준 어울림 표");
    let expected = body_top + VERT_OFFSET_HU / HU_PER_PX;
    assert!(
        (table_top - expected).abs() < 2.0,
        "PAGE 기준 표 상단 = 본문 상단 + vertOffset: table={table_top:.1}, expected={expected:.1}"
    );
}

#[test]
fn issue_7548_page_anchored_square_successors_above_band_keep_stored_positions() {
    let page = page();
    let (table_top, _) = page.table.expect("쪽 기준 어울림 표");
    let (host_y, host_vpos) = line(&page, HOST_PARA);
    for para in 4..=7 {
        let (y, vpos) = line(&page, para);
        let expected = host_y + f64::from(vpos - host_vpos) / HU_PER_PX;
        assert!(
            (y - expected).abs() < 0.5,
            "pi={para} 는 host 에서 저장 vpos 간격만큼 아래다(표로 밀리지 않는다): y={y:.1}, expected={expected:.1}"
        );
        assert!(
            y < table_top,
            "pi={para} 는 표 위에 놓인다: y={y:.1}, table top={table_top:.1}"
        );
    }
}

#[test]
fn issue_7548_page_anchored_square_band_overlapping_successor_goes_below_band() {
    let page = page();
    let (_, table_bottom) = page.table.expect("쪽 기준 어울림 표");
    let (host_y, host_vpos) = line(&page, HOST_PARA);
    let (y, vpos) = line(&page, 8);
    assert!(
        y >= table_bottom,
        "표 띠와 겹치는 pi=8 은 띠 아래에 놓인다: y={y:.1}, table bottom={table_bottom:.1}"
    );
    let expected = host_y + f64::from(vpos - host_vpos) / HU_PER_PX;
    assert!(
        (y - expected).abs() < 0.5,
        "pi=8 은 저장 vpos(띠 아래) 위치다: y={y:.1}, expected={expected:.1}"
    );
}
