//! 빈 host 문단에 co-anchored 된 자리차지(TopAndBottom, vert=문단) RowBreak float 표가 둘
//! 이상일 때, 선행 표 A 가 페이지를 채운 뒤의 *후속* 표 B 를 행 단위로 쪼개 머리만 현재
//! 페이지에 남기지 않고(orphan control) 통째로 다음 페이지에 둔다. 또한 후속 표 B 가 페이지를
//! 채운 뒤의 문서 말미 빈 문단은 빈 페이지를 만들지 않고 흡수한다.
//!
//! 한컴은 페이지에 들어가는(= 한 페이지보다 작은) 자리차지 표를 분할하지 않고 통째로 다음
//! 페이지에 두며, 그 뒤의 말미 빈 문단을 trailing overflow 로 흡수한다. 한 자리차지 표 안에
//! 머리(표제/결재) 행과 본문 행이 함께 든 실문서(비공개 점검표 양식)에서, 수정 전에는 머리
//! 행만 앞 페이지에 남고 본문이 다음 페이지로 분리되며 표 뒤 빈 문단으로 빈 페이지가 더 생겼다.
//!
//! 단독 anchored 자리차지 표(host 의 첫/유일 float)는 본문 흐름대로 행 단위 분할되어야
//! 하므로 orphan control 대상이 아니다 — 선행 co-anchored float 가 있는 후속 표로 한정한다.
//!
//! fixture: samples/issue1663_coanchored_float_orphan.hwpx
//!   = issue1639 구조(빈 host + co-anchored float 표)에서 표 A(2행, 큰 cellSz≈800px)·표 B(21행
//!     ≈784px)를 자리차지 RowBreak·양수 offset 으로 narrow. 표 A 가 page0 를 채우고 표 B 는
//!     page0 잔여 초과·fresh 페이지 적합. 말미 빈 문단 3개.
//!   수정 전(clean): 표 B 가 page0 에서 행 분할(rows 0..1) + 빈 page = 3 페이지.
//!   수정 후: 표 B 통째 page1 + 말미 빈 문단 흡수 = 2 페이지.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue1663_coanchored_float_orphan.hwpx";
const TARGET_PI: usize = 0;
const TABLE_A_CI: usize = 2;
const TABLE_B_CI: usize = 3;

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn has_table(root: &RenderNode, control_index: usize) -> bool {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(TARGET_PI) && table.control_index == Some(control_index) {
            return true;
        }
    }
    root.children.iter().any(|c| has_table(c, control_index))
}

/// 선행 co-anchored 자리차지 float 표가 페이지를 채운 뒤의 후속 표는, 현재 페이지 잔여엔 안
/// 들어가지만 fresh 페이지엔 통째로 들어가면 행 단위로 분할되지 않고 통째로 다음 페이지에
/// 배치되어야 한다(orphan control). 후속 표의 머리 행이 앞 페이지에 남으면 회귀다.
#[test]
fn coanchored_following_float_table_defers_whole_to_next_page() {
    let doc = load_doc(SAMPLE);
    let page0 = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");
    let page1 = doc
        .build_page_render_tree(1)
        .expect("build_page_render_tree(1)");

    assert!(
        has_table(&page0.root, TABLE_A_CI),
        "preceding co-anchored float table A (ci={TABLE_A_CI}) must fill page 0",
    );
    assert!(
        !has_table(&page0.root, TABLE_B_CI),
        "following co-anchored float table B (ci={TABLE_B_CI}) must NOT be row-split onto \
         page 0 — orphan control defers a page-fitting float table whole to the next page",
    );
    assert!(
        has_table(&page1.root, TABLE_B_CI),
        "following co-anchored float table B (ci={TABLE_B_CI}) must render on page 1",
    );
}

/// 후속 자리차지 표가 페이지를 채운 뒤의 문서 말미 빈 문단은 빈 페이지를 만들지 않고
/// 흡수되어야 한다. 빈 page 가 추가되면(페이지 수 증가) 회귀다. (orphan defer 가 정상이라는
/// 전제는 위 테스트가 보장하므로 두 테스트는 상보적이다 — 수정 전 clean 에서는 표 B 분할 +
/// 빈 page 로 3페이지가 되어 본 단언이 실패한다.)
#[test]
fn trailing_empty_paragraphs_after_float_table_do_not_add_blank_page() {
    let doc = load_doc(SAMPLE);
    assert_eq!(
        doc.page_count(),
        2,
        "trailing empty paragraphs after a page-filling co-anchored float table must be \
         absorbed, not pushed onto a new blank page",
    );
}
