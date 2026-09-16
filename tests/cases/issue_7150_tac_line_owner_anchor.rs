#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7150] 한 줄에 글자처럼 취급되는 표가 둘 이상이면, **줄 높이를 정한 표**까지
//! 저장 기준선에 앉아 그 줄 전체가 1.4px 내려앉는다.
//!
//! `#7049` 가 밴드 분기(`y + om_top`)를 「줄을 독점한 표」로 좁히면서 `line_tac_table_count
//! <= 1` 을 걸었는데, 그 술어는 **줄을 소유한 표까지** 배제한다. 소유자는 저장
//! `lh == h + om_top + om_bottom` 이 정확히 성립하는 표이고, 한/글은 그것을
//! `줄상단 + om_top` 에 고정한다. rhwp 는 대신 저장 기준선에 앉혀
//! `0.85×(om_top+om_bottom) − om_top` 만큼 내려놓았다.
//!
//! 기준선이 저장값이라는 것은 변조로 확인했다 — `issue2470` 의 해당 lineseg
//! `baseline="9841"` 을 `8000` 으로 바꾸면 `RHWP_DEBUG_PARA_TAC` 의 `baseline=131.2` 가
//! `106.7`(=8000/75) 이 된다. `0.85 × 11578 = 9841.3` 이라 값만으로는 계산·저장을
//! 구분할 수 없다.
//!
//! 한/글 2022 실측(px, 정본 PDF 의 글자 기준선 `origin` × 96/72):
//!
//! ```text
//!   글자                한/글     devel    이 수정
//!   시 민             159.19    160.60    159.30
//!   주무관·운영과장     188.91    190.30    189.00
//!   문서번호           195.95    197.40    196.10
//!   결재일자 값        223.12    224.60    223.30
//! ```
//!
//! 소유자만 고치는 것으로는 부족하다 — `line_tac_table_count` 술어만 떼면 큰 표는
//! 제자리로 오지만 작은 표가 저장 기준선에 남아 두 표의 하단차가 `6.20 → 4.90` 으로
//! 벌어진다(한/글 `6.23`). 같은 줄의 다른 표도 소유자 앵커에서 유도한 기준선에
//! 앉혀야 한다.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn page_tables(rel: &str, page: u32) -> Vec<(f64, f64, f64, f64)> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let core = DocumentCore::from_bytes(&std::fs::read(p).expect("표본 읽기")).expect("문서 로드");
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = Vec::new();
    fn walk(n: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
        if matches!(n.node_type, RenderNodeType::Table { .. }) {
            out.push((n.bbox.x, n.bbox.y, n.bbox.width, n.bbox.height));
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    walk(&tree.root, &mut out);
    out
}

/// 폭이 `[lo, hi]` 인 표들을 x 순으로.
fn by_width(rel: &str, page: u32, lo: f64, hi: f64) -> Vec<(f64, f64, f64, f64)> {
    let mut v: Vec<_> = page_tables(rel, page)
        .into_iter()
        .filter(|t| (lo..=hi).contains(&t.2))
        .collect();
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    v
}

const SAMPLE: &str = "samples/issue2470/36382471_masked.hwpx";

/// 줄을 소유한 표(우 7×4, 높이 150.60)는 `줄상단 + om_top` 에 앉는다.
///
/// 줄 상단 `140.50` + `om_top 1.867` = `142.37`. 수정 전에는 저장 기준선에 앉아
/// `143.70` 이었다.
#[test]
fn issue_7150_line_owner_sits_at_its_outer_margin() {
    let boxes = by_width(SAMPLE, 0, 340.0, 380.0);
    assert_eq!(boxes.len(), 1, "우측 결재표 하나를 기대했다: {boxes:?}");
    let (_, y, _, h) = boxes[0];
    assert!(
        (h - 150.60).abs() < 0.2,
        "표본 전제: 소유 표 높이 150.60 이어야 판정이 의미를 갖는다 (got {h:.2})"
    );
    assert!(
        (y - 142.37).abs() < 0.3,
        "줄을 소유한 표는 줄상단+om_top(142.37)에 앉아야 한다 — got {y:.2} \
         (수정 전 143.70: 저장 기준선에 앉아 0.85×(om_top+om_bottom)−om_top 만큼 낮음)"
    );
}

/// 같은 줄의 다른 표(좌 4×2, 높이 109.00)도 소유자 앵커에서 유도한 기준선에 앉는다.
///
/// 공유 기준선 = `줄상단 + owner_om_top + 0.85×owner_h`. 거기에 앉히면
/// `142.37 + 0.85×(150.60 − 109.00) = 177.73`.
#[test]
fn issue_7150_line_companion_follows_the_owner_anchor() {
    let boxes = by_width(SAMPLE, 0, 240.0, 270.0);
    assert_eq!(boxes.len(), 1, "좌측 결재표 하나를 기대했다: {boxes:?}");
    let (_, y, _, h) = boxes[0];
    assert!(
        (h - 109.00).abs() < 0.2,
        "표본 전제: 동반 표 높이 109.00 (got {h:.2})"
    );
    assert!(
        (y - 177.73).abs() < 0.3,
        "동반 표는 소유자 앵커 기준선에 앉아야 한다(177.73) — got {y:.2} \
         (수정 전 179.10)"
    );
}

/// #7049 가 세운 하단차 계약은 그대로다 — 높이차의 0.15 배.
///
/// `0.15 × (150.60 − 109.00) = 6.24`, 한/글 실측 `6.23`. **이 시험은 수정 전에도
/// 통과한다** — devel 이 이미 `6.20` 이기 때문이다. 목적은 회귀 가드다: 소유자만
/// 올리고 동반 표를 저장 기준선에 두면 `4.90` 으로 깨진다(그 오답을 여기서 막는다).
#[test]
fn issue_7150_bottom_gap_keeps_the_issue_7049_contract() {
    let left = by_width(SAMPLE, 0, 240.0, 270.0)[0];
    let right = by_width(SAMPLE, 0, 340.0, 380.0)[0];
    let gap = (right.1 + right.3) - (left.1 + left.3);
    assert!(
        (gap - 6.24).abs() < 0.3,
        "두 표의 하단차는 높이차의 0.15 배(6.24)여야 한다 — got {gap:.2} \
         (한/글 6.23 · 소유자만 고치면 4.90)"
    );
}

/// 같은 쪽의 본문 칸은 건드리지 않는다 — 바깥 표의 나머지 세 행.
#[test]
fn issue_7150_body_rows_are_untouched() {
    let outer: Vec<_> = page_tables(SAMPLE, 0)
        .into_iter()
        .filter(|t| t.2 > 640.0)
        .collect();
    assert_eq!(outer.len(), 1, "바깥 표 하나를 기대했다: {outer:?}");
    let (_, y, _, h) = outer[0];
    assert!(
        (y - 117.10).abs() < 0.2 && (h - 914.20).abs() < 0.3,
        "바깥 표 기하는 불변이어야 한다 — got y={y:.2} h={h:.2} (기대 117.10 / 914.20)"
    );
}
