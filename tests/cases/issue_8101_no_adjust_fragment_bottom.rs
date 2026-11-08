//! [#8101] `noAdjust` 표의 비끝 조각은 본문 아래 − 바깥 아래 여백에서 닫는다.
//!
//! 1342000 교육과정 지도(한/글 2014 저장, 가로 쪽 본문 아래 = 59528 − 4252 − 1417 = 53859HU)의
//! 한/글 2020 Print PDF 에서 쪽 끝 표 아래 괘선(쪽 척도 1.00047 을 걷은 값):
//! - 구역 5 `pi=8`(244×10, 표 속성 `0x0600000e` — bit 3 `noAdjust`): 121–125쪽 716.11px
//!   = (53859 − 바깥 아래 141) HU − 0.13px.
//! - 구역 5 `pi=14`(139×10, `0x06000006`, `noAdjust` 아님): 157쪽 714.8px
//!   = (53859 − 141 − 100) HU − 0.11px — 종전 inset(#7095) 그대로.
//!
//! 수정 전 rhwp 는 `noAdjust` 표도 100HU 위에서 닫아, 쪽마다 걸침 칸 «생활안전»(1–60행, 247264HU)의
//! 몫을 덜 소비했다. 남은 490HU 가 그 칸의 끝 행 60(선언 2982HU)에 몰려 126쪽 행이 4.9pt 커졌고,
//! 쪽 끝 «자전거를 탈 때» 줄이 127쪽으로 밀려 127–129쪽이 한 줄씩 어긋났다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";
const BODY_BOTTOM_HU: f64 = 59528.0 - 4252.0 - 1417.0;
const OUTER_BOTTOM_HU: f64 = 141.0;

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn px(hu: f64) -> f64 {
    hu / 75.0
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 쪽 `page`(0-기준)의 표 칸 상자 (행, 윗변, 아랫변).
fn cell_boxes(core: &DocumentCore, page: u32) -> Vec<(u16, f64, f64)> {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) => {
                Some((cell.row, node.bbox.y, node.bbox.y + node.bbox.height))
            }
            _ => None,
        })
        .collect()
}

fn fragment_bottom(core: &DocumentCore, page: u32) -> f64 {
    cell_boxes(core, page)
        .into_iter()
        .map(|(_, _, bottom)| bottom)
        .fold(f64::MIN, f64::max)
}

fn page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>()
        .replace(' ', "")
}

#[test]
fn no_adjust_table_fragment_closes_at_the_outer_bottom_margin() {
    let core = core(SAMPLE_1342000);
    let line = px(BODY_BOTTOM_HU - OUTER_BOTTOM_HU);
    for page in 120..=124 {
        let bottom = fragment_bottom(&core, page);
        assert!(
            (bottom - line).abs() <= 0.5,
            "{}쪽 noAdjust 표 조각은 {line:.2}px 에서 닫혀야 한다: {bottom:.2}",
            page + 1
        );
    }
}

/// 반례: 같은 문서의 `noAdjust` 아닌 표는 종전처럼 100HU 위에서 닫는다.
#[test]
fn ordinary_table_fragment_keeps_the_bottom_inset() {
    let core = core(SAMPLE_1342000);
    let line = px(BODY_BOTTOM_HU - OUTER_BOTTOM_HU - 100.0);
    let bottom = fragment_bottom(&core, 156);
    assert!(
        (bottom - line).abs() <= 0.5,
        "157쪽 표 조각은 {line:.2}px 에서 닫혀야 한다: {bottom:.2}"
    );
}

/// 걸침 칸의 끝 행 60 은 선언 높이 그대로이고, 126쪽이 «자전거를 탈 때» 줄까지 담는다.
#[test]
fn spanning_cell_end_row_keeps_its_declared_height_and_page_owns_the_line() {
    let core = core(SAMPLE_1342000);
    let (top, bottom) = cell_boxes(&core, 125)
        .into_iter()
        .filter(|(row, _, _)| *row == 60)
        .map(|(_, top, bottom)| (top, bottom))
        .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
        .expect("126쪽 행 60");
    let declared = px(2982.0);
    assert!(
        (bottom - top - declared).abs() <= 0.5,
        "126쪽 행 60 은 선언 {declared:.2}px 여야 한다: {:.2}",
        bottom - top
    );
    let p126 = page_text(&core, 125);
    let p127 = page_text(&core, 126);
    let line = "건강한신체를위해자전거를탈때주의할점말하기";
    assert!(
        p126.contains(line),
        "126쪽이 «자전거를 탈 때» 줄을 담아야 한다"
    );
    assert!(
        !p127.contains(line),
        "127쪽에 «자전거를 탈 때» 줄이 다시 나오면 안 된다"
    );
    assert!(
        p127.contains("현장체험학습시버스이용에관한"),
        "127쪽은 «현장체험학습 시 버스 이용» 줄을 담는다"
    );
}
