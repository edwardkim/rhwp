//! [#8464] 관계형 회귀 — 셀 단위로 나누는 표(`pageBreak="CELL"`, RowBreak)의 쪽 끝 행은 선언 높이가
//! 아니라 **내용(글줄 상자 + 칸 안 여백)이 남은 공간보다 작으면** 이 쪽에 두고 쪽 바닥에서 닫는다.
//! 남는 선언 몫은 1282HU 이하이면 버리고(#8105), 넘으면 다음 쪽 첫머리 빈 띠가 된다.
//!
//! - 갱신: 2026-11-02 (task/8464-rowbreak-content-fit)
//! - 입력: `samples/issue8464/*.hwpx` — `samples/issue8464/gen_row_content_fit.py` 로 생성(저장 줄 없음).
//! - 독립 기준: 한/글 2022(12.0.0.4547) 파일 → 인쇄 → Hancom PDF(한 쪽씩)
//!   `pdf/issue8464_rowbreak_content_fit_*-2020.pdf`. 2쪽 첫 행 라벨로 수용 여부를 읽었다.
//! - 측정(Native / fresh WASM `silhouette_raw_match_percent`, 쪽별·최저): 아래 `EVIDENCE` 참고.
//!   증거 경로 `mydocs/pr/assets/issue_8464/{native,wasm}/`.
//!
//! 한/글 실측(12pt 한 줄 행, 내용 1200 + 칸 여백 282 = 1482HU): 남은 공간이 내용보다 2HU 크면
//! 행을 두고, 같으면(0HU) 넘긴다. 10pt(1282HU)에서도 남은 16.5px 넘김 · 17.6px 둠으로 같은 경계다.
//! 수정 전 rhwp 는 남은 공간 25px 하한(`MIN_TOP_KEEP_PX`) 때문에 h2570·h3180·d14 의 행을
//! 통째로 다음 쪽에 넘겼다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 쪽 `page`(0-기준) 표 칸: (행, 윗변, 아랫변, 칸 안 글자).
fn cells(core: &DocumentCore, page: u32) -> Vec<(u16, f64, f64, String)> {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut out = Vec::new();
    collect_cells(&tree.root, &mut out);
    out
}

fn collect_cells(node: &RenderNode, out: &mut Vec<(u16, f64, f64, String)>) {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        let mut nodes = Vec::new();
        walk(node, &mut nodes);
        let text = nodes
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect::<String>()
            .replace(' ', "");
        out.push((cell.row, node.bbox.y, node.bbox.y + node.bbox.height, text));
        return;
    }
    for child in &node.children {
        collect_cells(child, out);
    }
}

/// 라벨 칸 «라벨 NN» 이 그려진 쪽들.
fn label_pages(core: &DocumentCore, row: u16) -> Vec<u32> {
    let label = format!("라벨{row:02}");
    (0..core.page_count())
        .filter(|page| {
            cells(core, *page)
                .iter()
                .any(|(_, _, _, text)| *text == label)
        })
        .collect()
}

/// 표가 이어지는 둘째 쪽에서 가장 먼저 그려진 행.
fn first_row_on(core: &DocumentCore, page: u32) -> u16 {
    cells(core, page)
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|cell| cell.0)
        .expect("이어지는 쪽에 표 칸이 있어야 한다")
}

fn assert_kept_on_first_page(sample: &str, row: u16) {
    let core = core(sample);
    assert_eq!(core.page_count(), 3, "{sample}: 한/글 Print 쪽수 3");
    assert_eq!(
        label_pages(&core, row),
        vec![0],
        "{sample}: 행 {row} 내용은 1쪽"
    );
    assert_eq!(
        label_pages(&core, row + 1),
        vec![1],
        "{sample}: 행 {} 은 2쪽",
        row + 1
    );
}

fn assert_deferred_to_second_page(sample: &str, row: u16) {
    let core = core(sample);
    assert_eq!(core.page_count(), 3, "{sample}: 한/글 Print 쪽수 3");
    assert_eq!(
        label_pages(&core, row - 1),
        vec![0],
        "{sample}: 행 {} 은 1쪽",
        row - 1
    );
    assert_eq!(
        label_pages(&core, row),
        vec![1],
        "{sample}: 행 {row} 은 통째로 2쪽"
    );
    assert_eq!(
        first_row_on(&core, 1),
        row,
        "{sample}: 2쪽은 행 {row} 로 시작"
    );
}

/// 남은 20.0px · 내용 17.1px · 선언 34.3px: 행을 두고 남는 14.3px(≤ 1282HU)는 버린다.
#[test]
fn content_fitting_row_closes_at_page_bottom_and_drops_small_tail() {
    let sample = "samples/issue8464/rowbreak_content_fit_h2570.hwpx";
    assert_kept_on_first_page(sample, 26);
    let core = core(sample);
    assert_eq!(
        first_row_on(&core, 1),
        27,
        "버린 꼬리는 2쪽 띠가 되지 않는다"
    );
}

/// 남은 20.5px · 선언 42.4px: 남는 21.9px(> 1282HU)는 2쪽 첫머리 빈 띠다.
#[test]
fn content_fitting_row_carries_large_tail_as_blank_band() {
    let sample = "samples/issue8464/rowbreak_content_fit_h3180.hwpx";
    assert_kept_on_first_page(sample, 21);
    let core = core(sample);
    let page = cells(&core, 1);
    let band = page.iter().filter(|cell| cell.0 == 21).collect::<Vec<_>>();
    assert!(!band.is_empty(), "2쪽 첫머리에 행 21 의 남은 띠");
    assert!(
        band.iter().all(|cell| cell.3.is_empty()),
        "띠에는 글자가 없다: {band:?}"
    );
    let next_top = page
        .iter()
        .filter(|cell| cell.0 == 22)
        .map(|cell| cell.1)
        .fold(f64::INFINITY, f64::min);
    assert!(
        band.iter().all(|cell| cell.2 <= next_top + 0.01),
        "띠는 행 22 위에 있다"
    );
    assert_eq!(first_row_on(&core, 1), 21);
}

/// 남은 9.1px < 내용 17.1px: 행을 통째로 다음 쪽에 넘긴다(반례).
#[test]
fn row_whose_content_does_not_fit_moves_whole() {
    assert_deferred_to_second_page("samples/issue8464/rowbreak_content_fit_h2505.hwpx", 27);
}

/// 12pt 한 줄 행의 HU 경계: 남은 공간 = 내용 + 2HU 이면 두고, = 내용이면 넘긴다.
#[test]
fn hu_boundary_is_strict() {
    assert_kept_on_first_page(
        "samples/issue8464/rowbreak_content_fit_f1200_h2660_d14.hwpx",
        25,
    );
    assert_deferred_to_second_page(
        "samples/issue8464/rowbreak_content_fit_f1200_h2660_d16.hwpx",
        25,
    );
}
