//! [Issue #6912] 세로 가운데 정렬 칸의 자리차지 개체가 빈 글줄을 따라 420.6px
//! 내려가 칸 바닥과 쪽 밖으로 잘린다 (156564340 안내 포스터).
//!
//! 근인은 콘텐츠 높이 **계산**이 아니라 그 값을 덮는 **저장 흐름 신뢰 규칙**이다.
//! 재조판 스택은 자리차지(`textWrap=TOP_AND_BOTTOM`, `vertRelTo=PARA`) 개체의 띠를
//! 이미 세고 있었다(13.33 + 854.4 = 867.7px). 그런데 `#1486`(악보 셀) 계약이
//! "TopAndBottom 은 저장 vpos 에 흡수된다"를 **전제로** 깔고 그 개체를
//! `non_flow_object_extent` 에서 빼 두어, 저장 extent 13.33px 가 콘텐츠 높이를
//! 통째로 대신했다. 그래서 글자 없는 13.33px 짜리 줄 하나가 854px 칸 한가운데로
//! 내려가고((854.4 − 13.33)/2 = 420.5px), 그 줄에 매달린 개체가 따라갔다.
//!
//! 이 문서의 앵커 줄은 `vertpos=0` — **흡수가 일어나지 않았다.** 흡수는 결과이지
//! 전제가 아니므로, 개체 띠를 `non_flow_object_extent` 에 넣으면 비교식이 흡수
//! 여부를 스스로 판정한다(흡수했으면 `띠 ≤ extent` 로 악보 셀 계약 유지).
//!
//! 한/글 자신의 저장값이 정답지다 — 그 띠는 행 높이 안에 있다.
//!
//! ```text
//!   tc  cellSz height =   282   = cellMargin.top 141 + bottom 141  → 글 내용 0
//!   tbl sz     height = 64362   = 63356(개체 높이) + 724(vertOffset) + 282
//! ```
//!
//! 그래서 가운데 정렬이 옮길 여백이 남지 않고 개체는 칸 윗변에서
//! `cellMargin.top` 과 `vertOffset` 을 더한 **+11.5px** 에 앉는다. 정본
//! `pdf/156564340-ip-dispute-mediation-poster-slice-2020.pdf`(engine 2020) 2쪽이
//! 칸 `116.8..974.0` 에 개체 `y=128.3`(= +11.5) 로 그린다.
//!
//! ⚠ **쪽 절대 좌표는 계약이 아니다.** rhwp 의 칸 상단(`109.4`)은 정본(`116.8`)과
//! 7.4px 다른 별개 축이라, 판정은 칸 기준 상대 위치로 한다.
//!
//! fixture 유래와 정본 산출 근거는 `samples/issue6912/README.md`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6912/156564340-ip-dispute-mediation-poster-slice.hwpx";
/// 개체가 있는 1×1 표의 쪽 (0-based).
const PAGE: u32 = 1;
/// 정본이 재는 칸 윗변 기준 개체 상단 = `cellMargin.top` 1.88px + `vertOffset` 9.65px.
const ORACLE_OFFSET_FROM_CELL_TOP: f64 = 11.53;
/// 허용 오차. 수정 후 실측은 `+10.60px` 로 정본과 **0.93px** 다르다 — rhwp 가 이 칸의
/// 안쪽 패딩을 `0.96px`(정본 `1.88px`)로 잡는 별개 축이고, 이 이슈가 고친 어긋남
/// (`420.6px`)과는 두 자릿수 차이다. 그 축까지 이 시험으로 잠그지 않는다.
const TOLERANCE_PX: f64 = 2.0;

#[test]
fn issue_6912_cell_float_sits_at_cell_top_not_centered_line() {
    let root = tree();
    let (cell, float) = poster_cell_and_float(&root);

    let offset = float.bbox.y - cell.bbox.y;
    assert!(
        (offset - ORACLE_OFFSET_FROM_CELL_TOP).abs() <= TOLERANCE_PX,
        "자리차지 개체가 칸 윗변 기준 +{offset:.1}px 에 있다 (정본 +{ORACLE_OFFSET_FROM_CELL_TOP:.1}px). \
         칸 y={:.1} h={:.1}, 개체 y={:.1} h={:.1}",
        cell.bbox.y,
        cell.bbox.height,
        float.bbox.y,
        float.bbox.height,
    );

    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let float_bottom = float.bbox.y + float.bbox.height;
    assert!(
        float_bottom <= cell_bottom + 1.0,
        "개체 아래끝 {float_bottom:.1} 이 칸 바닥 {cell_bottom:.1} 을 넘었다",
    );
}

/// 개체를 매단 **빈 글줄**도 같은 칸 상단 띠에 남는다 — 개체가 따라가는 대상이 이 줄이다.
#[test]
fn issue_6912_anchor_line_stays_at_cell_top() {
    let root = tree();
    let (cell, _) = poster_cell_and_float(&root);
    let line = cell_text_line(cell).expect("칸 안 앵커 글줄");

    let offset = line.bbox.y - cell.bbox.y;
    assert!(
        offset <= 4.0,
        "앵커 글줄이 칸 윗변 기준 +{offset:.1}px 로 내려갔다 \
         (칸 h={:.1} — 가운데 정렬이 (h - 줄높이)/2 만큼 옮긴 자리)",
        cell.bbox.height,
    );
}

fn tree() -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .expect("문서 로드")
        .build_page_render_tree(PAGE)
        .unwrap_or_else(|error| panic!("쪽 idx {PAGE} render tree: {error:?}"))
        .root
}

/// 포스터를 담은 1×1 표의 칸과 그 안의 자리차지 개체.
///
/// 이 쪽에는 머리 표(3칸, 높이 37.8px)도 있으므로 **칸 높이 800px 이상**으로 가른다.
/// 개체는 칸 배경 사각형과 같은 `Rectangle` 노드라 크기로 가른다(개체 602×845px).
fn poster_cell_and_float(root: &RenderNode) -> (&RenderNode, &RenderNode) {
    let cell = find(root, &|n| {
        matches!(n.node_type, RenderNodeType::TableCell(_)) && n.bbox.height > 800.0
    })
    .expect("포스터를 담은 세로 가운데 정렬 칸");
    let cell_width = cell.bbox.width;
    let float = find(cell, &|n| {
        matches!(n.node_type, RenderNodeType::Rectangle(_))
            && n.bbox.height > 800.0
            // 칸 배경 사각형과 가른다 — 개체는 602.4px 로 칸(635.0px)보다 좁다.
            && n.bbox.width < cell_width - 10.0
    })
    .expect("칸 안 자리차지 개체");
    (cell, float)
}

fn cell_text_line(cell: &RenderNode) -> Option<&RenderNode> {
    find(cell, &|n| {
        matches!(n.node_type, RenderNodeType::TextLine(_))
    })
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}
