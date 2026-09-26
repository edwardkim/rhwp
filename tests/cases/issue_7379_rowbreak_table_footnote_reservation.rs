//! [#7379] 각주를 든 `RowBreak` 자리차지 표가 쪽 경계에서 쪼개진다.
//!
//! ## 무엇이 문제였나
//!
//! 통째/분할 **진입 판정**의 예산(`available`)에서 표 **자신의** 각주 높이를 미리 깎았다.
//! 그러면 각주를 든 표는 자기 각주 때문에 현재 쪽의 남은 자리가 음수가 되어, 분할을
//! 시도조차 못 하고 통째로 다음 쪽으로 간다. 쪼개졌다면 첫 조각은 그 조각이 실제로
//! 데려가는 각주만 필요한데, 판정은 **어느 조각도 쓰지 않을 예산**을 요구한 셈이다.
//!
//! ```text
//!   문단 0.728  7행×2열 · 쪽나눔=RowBreak · treat_as_char=false · wrap=자리차지
//!               표 안 각주 6건
//!     종전  total_footnote 294.0 (쪽 각주 43.4 + 표 각주 250.6)
//!           → available 622.2 < current_height 713.8   → 통째 이월
//!           → 그 쪽에 실제로 그려진 각주는 35.9px, 본문은 220px 공백
//! ```
//!
//! 같은 문서의 표 **58건** 중 표 안 각주가 없는 47건은 한 번도 이 자리에 막히지 않았고,
//! 각주를 든 11건에서만 막혔다 — 갈리는 것은 표 크기가 아니라 **자기 각주 유무**다.
//!
//! ## 기대값의 출처 — 한/글 출력 PDF
//!
//! `pdf/정책연구용역사업 중간진도보고서(…)-hwpx-2024.pdf`(215쪽)에서 이 표의 **행 안 각주
//! 77번**이 **66쪽**에 있다.
//!
//! ```text
//!   정본 p66 끝   … 77) CFR → Title 42(Public Health) → Chapter IV(CMS …) - 66 -
//!   정본 p67 앞   Stephanie Tubbs Jones … Policy OPTN policy 14.82)  표 23. …
//! ```
//!
//! 곧 한/글은 이 표를 **p66/p67 로 쪼개** 앞부분을 66쪽에 둔다. rhwp 는 머리행부터 통째로
//! 67쪽에 두었다(66쪽에는 표가 하나도 없었다).
//!
//! ## 비범위
//!
//! 이 문서의 총 쪽수는 이 수정만으로 정본(215)과 같아지지 않는다(220 → 217). 남은 3쪽은
//! 다른 삽입 지점이고 이 시험의 대상이 아니다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx";
/// 문제의 표를 든 host 문단.
const HOST_PARA: usize = 728;

fn core() -> DocumentCore {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("정식 원본");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 해당 쪽에서 `HOST_PARA` 가 소유한 최상위 표의 `(y, height)` 를 모은다.
fn host_tables(root: &RenderNode) -> Vec<(f64, f64)> {
    fn walk(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(HOST_PARA) {
                out.push((node.bbox.y, node.bbox.height));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

/// 각주를 든 RowBreak 자리차지 표가 쪽 경계에서 쪼개져 앞 조각이 66쪽에 남는다.
#[test]
fn rowbreak_table_with_own_footnotes_splits_at_the_page_boundary() {
    let core = core();

    // 66쪽(0-based 65) — 한/글이 앞 조각을 두는 쪽.
    let tree66 = core.build_page_render_tree(65).expect("66쪽 render tree");
    let on66 = host_tables(&tree66.root);
    assert_eq!(
        on66.len(),
        1,
        "66쪽에 문단 {HOST_PARA} 표 조각이 없다 — 표가 통째로 다음 쪽으로 밀렸다(수정 전 상태). \
         한/글 정본은 이 표의 각주 77번을 66쪽에 둔다. 실제: {on66:?}"
    );

    // 67쪽 — 이어받은 조각.
    let tree67 = core.build_page_render_tree(66).expect("67쪽 render tree");
    let on67 = host_tables(&tree67.root);
    assert_eq!(
        on67.len(),
        1,
        "67쪽에 이어받은 조각이 없다 — 분할이 두 쪽에 걸치지 않았다. 실제: {on67:?}"
    );

    // 앞 조각은 쪽 아래쪽에서 시작하고, 이어받은 조각은 본문 상단에서 시작한다.
    let (first_top, first_h) = on66[0];
    let (next_top, _) = on67[0];
    assert!(
        first_top > 400.0,
        "66쪽 조각이 쪽 위에서 시작한다 ({first_top:.1}) — 앞 본문 뒤에 이어 붙어야 한다"
    );
    assert!(
        next_top < 200.0,
        "67쪽 조각이 본문 상단에서 시작하지 않는다 ({next_top:.1})"
    );
    // 두 조각의 합이 통짜 높이(약 213.5px)를 넘지 않는다 — 같은 행을 두 번 그리지 않는다.
    let (_, next_h) = on67[0];
    assert!(
        first_h + next_h < 260.0,
        "두 조각 높이 합 {:.1}px 이 통짜 표보다 크다 — 행이 중복됐을 수 있다",
        first_h + next_h
    );
}

/// 반례 대조군 — 표 안 각주가 **없는** 같은 형상의 표는 종전대로 두 쪽에 걸쳐 쪼개진다.
///
/// 문단 0.866 은 host·`RowBreak`·`treat_as_char`·`wrap`·`vert`/`horz`·행 수가 위 표와 같고
/// **표 안 각주만 0건**이다. 이 수정이 각주 없는 표의 분할을 건드리지 않았음을 잠근다.
///
/// 쪽 번호는 이 수정으로 앞쪽이 줄면서 밀리므로(수정 전 77/78 · 수정 후 76/77) **절대
/// 번호로 고정하지 않는다.** 그래서 이 시험은 수정 전에도 통과한다 — 결함 검출 증거가
/// 아니라 회귀 잠금이다.
#[test]
fn footnote_free_rowbreak_table_keeps_splitting() {
    const FOOTNOTE_FREE_HOST: usize = 866;
    fn count_on(core: &DocumentCore, page: u32, para: usize) -> usize {
        fn walk(node: &RenderNode, para: usize, out: &mut usize) {
            if let RenderNodeType::Table(table) = &node.node_type {
                if table.para_index == Some(para) {
                    *out += 1;
                }
            }
            for child in &node.children {
                walk(child, para, out);
            }
        }
        let Ok(tree) = core.build_page_render_tree(page) else {
            return 0;
        };
        let mut n = 0;
        walk(&tree.root, para, &mut n);
        n
    }
    let core = core();
    // 70..85쪽 구간에서 이 표가 나타나는 쪽을 모은다(번호 고정 없이).
    let pages: Vec<u32> = (70..85)
        .filter(|&p| count_on(&core, p, FOOTNOTE_FREE_HOST) > 0)
        .collect();
    assert_eq!(
        pages.len(),
        2,
        "표 안 각주가 없는 표(문단 {FOOTNOTE_FREE_HOST})가 두 쪽에 걸쳐 쪼개지지 않는다. 실제 쪽: {pages:?}"
    );
    assert_eq!(
        pages[1],
        pages[0] + 1,
        "두 조각이 연속한 쪽에 있지 않다: {pages:?}"
    );
}
