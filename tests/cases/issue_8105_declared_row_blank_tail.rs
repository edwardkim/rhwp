//! [#8105] 내용이 이 쪽에서 끝난 분할 행의 남은 선언 높이.
//!
//! 한/글 2024(13.0.0.3901) 합성 사다리(`mydocs/tech/investigations/issue-8105/probes/
//! make_declared_row_band.py`)에서 시험 행(라벨 칸 선언 D, 내용 5줄)은 쪽 아래까지 그린 뒤
//! 남은 선언(D − 그린 높이)이 1282HU(17.09px) 를 넘으면 다음 쪽 첫머리에 빈 띠로 넘기고, 그 이하면
//! 쪽 경계에서 버린다(행을 덮는 병합 칸이 자기 마지막 행에서 그 몫을 원장으로 이어받는다).
//! 17.0px 흡수 · 17.2px 이월이며, 칸 여백·글자 크기·저장 줄·병합 칸 유무와 무관했다.
//!
//! 픽스처 `samples/issue8105/declared_row_band.hwpx` 의 블록(블록마다 새 쪽, 시험 행 1 은 쪽 끝에서 나뉨):
//! - 병합 칸 + 저장 줄(재조판 없음): D 15862 · 14738 · 15412 HU, 1–6쪽.
//! - 병합 칸 + 줄 없음(재조판): 같은 D, 7–12쪽.
//!
//! 기준 `pdf/issue8105/declared_row_band-2024.pdf` 의 이어지는 쪽 첫 행: 51.2 · 흡수 · 24.0 · 29.9 · 흡수 · 24.0px.
//! 수정 전 rhwp 는 저장 줄 블록의 띠를 버렸고(2·6쪽), 재조판 블록의 15px 꼬리를 띠로 넘겼다(10쪽).
//! 1342000 58→59쪽 행 9(띠 25.9px) 와 같은 형상이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8105/declared_row_band.hwpx";
const ABSORB_LIMIT_HU: f64 = 1282.0;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {SAMPLE}: {error:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn text_of(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 쪽 `page`(0-기준)의 라벨 열(열 1) 칸: (행, 윗변, 높이, 글자).
fn label_column(core: &DocumentCore, page: u32) -> Vec<(u16, f64, f64, String)> {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let mut cells: Vec<_> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) if cell.col == 1 => {
                Some((cell.row, node.bbox.y, node.bbox.height, text_of(node)))
            }
            _ => None,
        })
        .collect();
    cells.sort_by(|a, b| a.1.total_cmp(&b.1));
    cells
}

/// 앞 쪽에서 그린 시험 행(행 1) 높이와 이어지는 쪽의 첫 라벨 열 칸.
fn split(core: &DocumentCore, page: u32) -> (f64, (u16, f64, f64, String)) {
    let painted = label_column(core, page - 1)
        .into_iter()
        .find(|cell| cell.0 == 1 && cell.3 == "라벨")
        .unwrap_or_else(|| panic!("{}쪽에 시험 행 라벨 칸이 없다", page))
        .2;
    let first = label_column(core, page)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{}쪽에 표 칸이 없다", page + 1));
    (painted, first)
}

#[test]
fn blank_tail_above_the_limit_opens_the_next_page() {
    let core = core();
    // (이어지는 쪽 0-기준, 선언 HU)
    for (page, declared) in [(1, 15862.0), (5, 15412.0), (7, 15862.0), (11, 15412.0)] {
        let (painted, (row, _, height, text)) = split(&core, page);
        let tail = declared / 75.0 - painted;
        assert!(
            tail > ABSORB_LIMIT_HU / 75.0,
            "{}쪽 대조 조건: 꼬리 {tail:.2}px",
            page + 1
        );
        assert_eq!(row, 1, "{}쪽 첫 행은 시험 행의 빈 띠", page + 1);
        assert!(text.is_empty(), "{}쪽 빈 띠에 글자 «{text}»", page + 1);
        assert!(
            (height - tail).abs() <= 0.5,
            "{}쪽 빈 띠: 기대 {tail:.2}px(선언 − 앞 쪽 {painted:.2}), 실제 {height:.2}px",
            page + 1
        );
    }
}

#[test]
fn blank_tail_within_the_limit_is_absorbed_at_the_page_end() {
    let core = core();
    for (page, declared) in [(3, 14738.0), (9, 14738.0)] {
        let (painted, (row, _, _, text)) = split(&core, page);
        let tail = declared / 75.0 - painted;
        assert!(
            tail > 0.5 && tail <= ABSORB_LIMIT_HU / 75.0,
            "{}쪽 대조 조건: 꼬리 {tail:.2}px",
            page + 1
        );
        assert_eq!(row, 2, "{}쪽은 빈 띠 없이 다음 행으로 시작", page + 1);
        assert_eq!(text, "다음", "{}쪽 첫 칸", page + 1);
    }
}

const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";

fn core_1342000() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_1342000);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse 1342000")
}

/// 쪽 `page`(0-기준)의 표 칸 (행, 열, 행 병합, 윗변, 높이, 글자).
fn cells(core: &DocumentCore, page: u32) -> Vec<(u16, u16, u16, f64, f64, String)> {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) => Some((
                cell.row,
                cell.col,
                cell.row_span,
                node.bbox.y,
                node.bbox.height,
                text_of(node),
            )),
            _ => None,
        })
        .collect()
}

/// 1342000 구역 1 다문화 표(한/글 2014 저장, 저장 줄 칸)의 한/글 2020 Print PDF:
/// - 58쪽 끝 행 9(«부산», 라벨 칸 선언 9366HU)는 내용을 모두 담고 닫는 선까지 98.9px, 59쪽 첫머리에
///   빈 띠 25.9px — 둘의 합이 선언이다. 수정 전 rhwp 는 띠를 버려 59쪽 전체가 19.4pt 위였다.
/// - 61쪽 첫 빈 띠 51.6px 는 행 17 꼬리(30.8)가 아니라 행 8–17 병합 칸(열 1, 선언 103310HU)의
///   남은 선언이다(58–60쪽에 그린 몫을 뺀 값).
#[test]
fn stored_rows_carry_declared_and_ending_span_remainders() {
    let core = core_1342000();
    let label = |page: u32, row: u16| {
        cells(&core, page)
            .into_iter()
            .find(|cell| cell.0 == row && cell.1 == 2)
            .unwrap_or_else(|| panic!("{}쪽에 행 {row} 라벨 칸이 없다", page + 1))
    };
    let painted = label(57, 9);
    assert_eq!(painted.5, "부산");
    let band = label(58, 9);
    assert!(band.5.is_empty(), "59쪽 띠에 글자 «{}»", band.5);
    let expected = 9366.0 / 75.0 - painted.4;
    assert!(expected > ABSORB_LIMIT_HU / 75.0);
    assert!(
        (band.4 - expected).abs() <= 0.5,
        "59쪽 띠: 기대 {expected:.2}px, 실제 {:.2}px",
        band.4
    );

    let span_drawn: f64 = (57..=59)
        .flat_map(|page| cells(&core, page))
        .filter(|cell| cell.0 == 8 && cell.1 == 1 && cell.2 == 10)
        .map(|cell| cell.4)
        .sum();
    let first = cells(&core, 60)
        .into_iter()
        .filter(|cell| cell.1 == 2)
        .min_by(|a, b| a.3.total_cmp(&b.3))
        .expect("61쪽 라벨 열");
    assert_eq!(first.0, 17, "61쪽은 행 17 의 빈 띠로 시작");
    assert!(first.5.is_empty(), "61쪽 띠에 글자 «{}»", first.5);
    let expected = 103310.0 / 75.0 - span_drawn;
    assert!(
        (first.4 - expected).abs() <= 0.5,
        "61쪽 띠: 기대 {expected:.2}px(병합 칸 선언 − 58–60쪽 {span_drawn:.2}), 실제 {:.2}px",
        first.4
    );
}
