//! 고정형(셀 기준 ±2px)·고정형(쪽수): 셀 문단의 자리차지 그림이 자기 앵커 줄을 그림 아래로 민 저장 형상에서, 그림을 밀린 줄이 아니라 문단 시작(셀 세로 정렬이 옮긴 저장 내용 묶음의 위)에 그리는지, 전체 쪽수가 독립 PDF 와 같은지 검사한다.
//! 주석 갱신: 2026-11-07T17:27:12+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-07 KST, base = upstream/devel b8dd67c500, head = base + 이 PR 의 src 변경. 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw.
//! 원본 `samples/issue8737/156667654_saemangeum_press_release.hwpx`(한/글 2024 저장), 전체 2쪽=2쪽.
//! 1쪽 수정 전 Native 96.86 / fresh WASM 97.15 → 수정 후 Native 99.90 / fresh WASM 99.94(양쪽 98% 이상이라 칸 기준 ±2px 를 고정한다). 2쪽 99.95 그대로.
//! 수정 후 전쪽 최저 Native 99.90·fresh WASM 99.94(90% 미만 0쪽).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8737/{base,head}/{native,wasm}/saemangeum_press_release_silhouette.tsv`, `png/`.
//! 확대 판독: 1쪽 머리 표 로고가 한/글과 같은 자리(overlay 일치). 수정 전에는 로고가 24.7px 아래, 셀 정렬 없이 원점만 옮기면 9.4px 위.
//! 같은 assertion 이 수정 전 FAIL(칸 위끝에서 44.11px) / 수정 후 PASS.
//! 독립 기대값: 한/글 2024 Print `pdf/issue8737/156667654_saemangeum_press_release-2024.pdf`(2쪽). 1쪽 머리 표 첫 칸의 로고 그림 위끝 98.8px, 칸 위끝 79.4px(96 DPI) — 칸 위끝에서 19.4px.
//! 저장 근거: 칸 문단의 첫 줄 vpos 3474 = 그림 높이 2863 + 세로 오프셋 611(줄이 그림에 밀림). 칸은 세로 가운데 정렬이고 저장 내용 끝은 4474 라 묶음이 (6171 − 282 − 4474)/2 = 707 HU 내려간다.
//! 독립 절대 px 좌표 대신 칸 위끝 기준 거리를 고정한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8737/156667654_saemangeum_press_release.hwpx";
const PDF_PAGES: u32 = 2;
/// 한/글 Print: 로고 위끝 98.8px − 칸 위끝 79.4px.
const LOGO_TOP_FROM_CELL_TOP: f64 = 19.4;

/// 첫 쪽 첫 번째 그림과 그 그림을 품은 칸의 상자.
fn first_cell_picture(
    node: &RenderNode,
    cell: Option<BoundingBox>,
) -> Option<(BoundingBox, BoundingBox)> {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node.bbox)
    } else {
        cell
    };
    if let (RenderNodeType::Image(_), Some(cell)) = (&node.node_type, cell) {
        return Some((node.bbox, cell));
    }
    node.children
        .iter()
        .find_map(|child| first_cell_picture(child, cell))
}

#[test]
fn cell_picture_that_pushed_its_own_line_starts_at_paragraph_origin() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    let tree = core.build_page_render_tree(0).expect("쪽 배치");
    let (logo, cell) = first_cell_picture(&tree.root, None).expect("머리 표 첫 칸의 로고");
    let from_cell_top = logo.y - cell.y;
    assert!(
        (from_cell_top - LOGO_TOP_FROM_CELL_TOP).abs() <= 2.0,
        "로고 위끝은 칸 위끝에서 {LOGO_TOP_FROM_CELL_TOP}px(한/글): 실제 {from_cell_top:.2}px, logo={logo:?} cell={cell:?}"
    );
    assert!(
        logo.y + logo.height <= cell.y + cell.height + 0.5,
        "로고가 칸 아래로 나가지 않는다: logo={logo:?} cell={cell:?}"
    );
}
