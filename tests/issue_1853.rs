//! [Task #1853] 같은 문단 float 스택 이월 규칙(#1831/PR #1844) 회귀 게이트.
//!
//! PR #1844 의 `preceded_by_same_para_float` 술어가 `para_index` 만 비교하고 `tac`·앵커를
//! 구분하지 않아, 표 자신의 `tac=true` 캡션 상자까지 "선행 형제 float" 로 오분류했다. 그 결과
//! 같은 문단의 본체 자리차지 표가 현재 쪽에서 분할 시작하지 못하고 통째로 다음 쪽으로 이월되어
//! 페이지 수가 +1 되는 회귀가 났다(문서 5,200건 서베이에서 3건 확인, `mydocs/pr/pr_1844_review.md`).
//!
//! 수정: 이월 그룹을 같은 문단의 **진짜 flow 스택 float**(`is_para_topbottom_float` =
//! `!tac && TopAndBottom && vert=Para`)로 한정. tac 캡션·페이지-절대(vert=용지) 앵커는 제외한다.
//!
//! fixture: samples/issue1853_caption_precedes_body_split.hwpx
//!   = 실문서 78842 (조문별 제개정 이유서, 데이터기반행정 활성화 법률 일부개정안). pi=371 문단이
//!     `tac=true` 캡션 상자(ci=0, 1×3) + 본체 자리차지 표(ci=1, 3×2)를 함께 앵커한다. 캡션이 쪽
//!     하단 근처에 오면 본체가 현재 쪽에서 분할 시작해야 한다(한글 정답지 52쪽).
//!   수정 전(회귀): 캡션(tac)이 선행 float 로 잡혀 본체가 통째 다음 쪽 이월 → 캡션 쪽에 본체 없음,
//!     문서 전체 53쪽.
//!   수정 후: 본체가 캡션과 같은 쪽에서 분할 시작 → 52쪽.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue1853_caption_precedes_body_split.hwpx";
const ANCHOR_PI: usize = 371;
const CAPTION_CI: usize = 0; // tac=true 캡션 상자 (선행)
const BODY_CI: usize = 1; // 자리차지 본체 표 (분할 대상)
const EXPECTED_PAGES: u32 = 52;

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn has_table(root: &RenderNode, pi: usize, ci: usize) -> bool {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(pi) && table.control_index == Some(ci) {
            return true;
        }
    }
    root.children.iter().any(|c| has_table(c, pi, ci))
}

/// tac 캡션 상자가 선행하는 본체 자리차지 표는, 캡션과 같은 쪽에서 분할 시작해야 한다.
/// 캡션(tac)을 "선행 float" 로 오인해 본체를 통째 다음 쪽으로 이월하면 회귀다.
#[test]
fn body_float_splits_on_caption_page_not_deferred_whole() {
    let doc = load_doc(SAMPLE);
    let n = doc.page_count();

    // 캡션(ci=0)이 렌더된 쪽을 찾는다.
    let mut caption_page = None;
    for p in 0..n {
        let tree = doc
            .build_page_render_tree(p)
            .unwrap_or_else(|e| panic!("build_page_render_tree({p}): {e}"));
        if has_table(&tree.root, ANCHOR_PI, CAPTION_CI) {
            caption_page = Some((p, tree));
            break;
        }
    }
    let (page_idx, tree) = caption_page.expect("caption table pi=371 ci=0 (tac) must render");

    assert!(
        has_table(&tree.root, ANCHOR_PI, BODY_CI),
        "body float table (pi={ANCHOR_PI} ci={BODY_CI}) must split-start on the same page \
         ({page_idx}) as its tac caption (ci={CAPTION_CI}); deferring it whole to the next page \
         is the #1831/#1844 same-para float over-capture regression",
    );
}

/// tac 캡션 과잉 이월 회귀 시 본체가 한 쪽 밀려 전체 페이지 수가 +1(53쪽) 된다. 수정 후 52쪽.
#[test]
fn caption_over_deferral_does_not_add_a_page() {
    let doc = load_doc(SAMPLE);
    assert_eq!(
        doc.page_count(),
        EXPECTED_PAGES,
        "tac 캡션을 선행 float 로 오인해 본체를 통째 이월하면 53쪽으로 회귀한다",
    );
}
