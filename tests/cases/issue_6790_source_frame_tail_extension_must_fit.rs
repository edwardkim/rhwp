//! [Issue #6790] `#5584 ②` 저장 프레임 꼬리 확장이 **위아래(TopAndBottom)** 배치에서
//! 상한이 없어, `RowBreak` 표의 첫 조각이 쪽 예산을 **254px** 넘겨 배치됐다.
//!
//! `17544911` 실측 — 행 2 하나가 1,934.7px 라 인트라-셀 컷이 필요하다.
//!
//! ```text
//!   avail_for_rows = 1005.4          이 조각이 행에 쓸 수 있는 전부
//!   res.consumed_height =  789.6     확장 전 행 2 컷
//!   source_tail_cut     = 1178.5     확장 후 행 2 컷   → extension = 388.9
//!                                    ⚠ 이 컷 하나가 avail 1005.4 를 넘는다
//!   조각 합계 = consumed + cs + pad + tail = 1263.3   (avail 대비 +257.9)
//!   TABLE_SPLIT_RESULT: … fits=false                  ← 그런데 그대로 배치된다
//! ```
//!
//! ⚠ 두 수치를 구분한다 — 코드가 검사하는 `extension` 은 **388.9**(컷 자체의 증가량)
//! 이고, `257.9` 는 그로 인한 **조각 합계의 예산 초과량**이다. 초판 설명이 이 둘을
//! 섞어 적었다(PR #6792 검토 지적).
//!
//! `#6549`(PR #6559)가 **어울림(Square)** 에 상한을 달았지만 이 문서는
//! **위아래(TopAndBottom)** 라 그 갈래에 안 들어간다.
//!
//! ⭐⭐ **판정은 크기가 아니라 쪽 소유(page ownership)다.** 확장된 저장 프레임 컷이
//! **이 조각의 행 예산 자체**를 넘으면, 그 컷은 혼자서도 이 쪽에 못 들어간다 — 그
//! 프레임은 어차피 쪽을 넘기므로 이 쪽의 경계를 정할 자격이 없다. 문턱 상수가 없다.
//!
//! ```text
//!   문서                                  tail    avail   소유
//!   17544911 (누에 사육기준) r=2         1178.5  1005.4   ✗ 못 넘김 → 확장 기각
//!   편람 r=1                               232.3   240.7   ✔
//!   편람 r=4 (여섯 갈래)              82.9~386.9  108.1~458.2 ✔ 전부
//!   3232693 (#5584/#6025) r=7             162.7   906.6   ✔
//!   16418295 (#6549) r=6                   92.8  1009.1   ✔
//! ```
//!
//! ⚠ 크기·비율 축으로는 갈리지 않는다 — 편람 핀들의 확장은 15.3~107.4px, 예산
//! 초과량은 17.8~112.0px 로 흩어져 있다. 위 표에서 갈리는 것은 **부호 하나**다.
//! (초판은 `extension <= 120.0` 이라는 경험적 경계를 썼다 — 관측 최대 107.4px 바로
//! 위에 둔 값이라 근거가 없었고, PR #6792 검토 지적에 따라 제거했다.)
//!
//! ⚠ 초판에는 `|| consumed > avail_for_rows` 우회가 있었다. **제거했다** — 그 우회는
//! 위 쪽 수용 불변식을 무효화할 수 있고, `#5057` 두 시험은 이 `source_frame_tail`
//! 갈래를 실제로 실행하지 않는다(PR #6792 검토 실측). 우회 없이 선택 시험 19/19 통과.
//!
//! 결과: 쪽수 2 → **3**(= 한/글 2024), 넘침·용지밖 1·1 → **0·0**,
//! 공백 제거 글자 결손 105 → **26**자.
//!
//! ⚠ **남는 축** — 잔여 26자와 2쪽 중첩 표 안 글자겹침 2건은 **중첩 표 이어지는
//! 조각**의 별개 축이다. 이 시험은 쪽수와 넘침·용지밖만 계약한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6790/17544911-sericulture-training-criteria.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6790 정식 HWP fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

fn worst_table_overflow(node: &RenderNode, bottom: f64, out: &mut f64) {
    if matches!(node.node_type, RenderNodeType::Table { .. }) {
        *out = out.max(node.bbox.y + node.bbox.height - bottom);
    }
    for child in &node.children {
        worst_table_overflow(child, bottom, out);
    }
}

/// 한/글 2024 와 같은 3쪽이어야 한다.
///
/// 수정 전에는 첫 조각이 예산을 254px 넘겨 표가 안 쪼개지고 2쪽이 됐다.
#[test]
fn row_break_table_splits_into_three_pages() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        3,
        "한/글 2024 와 같은 3쪽이어야 한다 — #6790 회귀 (수정 전 2쪽)"
    );
}

/// 1쪽 표가 본문 하한을 넘으면 안 된다.
///
/// 수정 전 `+254.2px`(용지 밖 `+216.4px`).
#[test]
fn first_fragment_stays_inside_the_body() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");
    let bottom = body.bbox.y + body.bbox.height;

    let mut over = 0.0f64;
    worst_table_overflow(body, bottom, &mut over);

    assert!(
        over <= 0.5,
        "1쪽 표가 본문 하한을 넘으면 안 된다 — #6790 회귀          \
         (초과 {over:.1}px, 본문 하한 {bottom:.1}; 수정 전 +254.2px)"
    );
}

/// 반대 방향 — **예산 안에 들어가는 저장 프레임 확장은 그대로 적용된다**.
///
/// `#5584 ②`/`#4763` 의 확장은 `source_tail_cut` 이 조각의 행 예산 안에 들어가는
/// 형상이라 이 판정에 걸리지 않는다.
///
/// ```text
///   문서                          tail    avail   mid_ok  확장 적용
///   3232693 (#5584 / #6025) r=7   162.7   906.6   true    ✔  → 4쪽
///   17544911 (이 이슈)      r=2  1178.5  1005.4   —       ✗ 기각 → 3쪽
/// ```
///
/// ⚠ 초판은 여기에 `16418295`(#6549)도 양성 사례로 적었는데 **틀렸다**(PR #6792
/// 검토 실측). 그 문서는 `extension=25.6`, `mid_extension_ok=false` 라 애초에 확장이
/// **적용되지 않는다** — 이 판정과 무관하다. 아래 `..._is_unaffected` 로 분리했다.
#[test]
fn a_source_frame_that_fits_still_extends() {
    let path = "samples/issue5584/3232693_employment_support_criteria.hwpx";
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("{path} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        4,
        "예산 안에 들어가는 저장 프레임 확장은 유지돼야 한다 — {path}"
    );
}

/// `#6549`(어울림 상한) 문서는 이 판정과 **무관하다**.
///
/// `mid_extension_ok=false` 로 이미 `#6549` 상한에서 걸러지므로 새 조건이 켜지든
/// 꺼지든 결과가 같다. 그래도 쪽수를 핀으로 남겨 이 축이 그 문서를 건드리지 않음을
/// 고정한다.
#[test]
fn the_square_wrap_bound_document_is_unaffected() {
    let path = "samples/issue6549/16418295_square_rowbreak_table.hwp";
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("{path} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 2, "#6549 쪽수 핀 — {path}");
}
