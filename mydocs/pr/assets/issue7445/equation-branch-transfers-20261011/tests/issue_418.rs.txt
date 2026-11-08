//! Issue #418: hwpspec.hwp 20/21 페이지의 빈 문단 + TAC Picture 가
//! paragraph_layout 와 layout_shape_item 양쪽에서 emit 되어 SVG 에 두 번
//! 그려지는 회귀.
//!
//! 정황:
//! - pi=83 / pi=86 / pi=89 가 각각 빈 문단 + TAC=true Picture (bin_id=35,36,37)
//! - paragraph_layout.rs 의 빈 runs + TAC offsets 분기 (line 2008-) 가 emit
//! - layout.rs::layout_shape_item 의 Task #347 분기 (line 2554-) 가 또 emit
//! - 결과: <image> 6 개 (3 쌍 × 2.67px y 어긋남)
//!
//! Task #376 이 정정한 결함이지만 commit (45419a2) 이 devel 에 머지되지 않은
//! 정황. 본 task #418 에서 정확히 재적용 — paragraph_layout 가 emit 후
//! set_inline_shape_position 호출, layout_shape_item 은 등록된 경우 push 스킵.
//!
//! Task #1086 이후 Hancom 기준 페이지 경계에 맞게 pi=89 는 21 페이지로 이동한다.
//! 정정 후 기대: 20 페이지 <image> 2 개(pi=83, 86), 21 페이지 <image> 1 개(pi=89).

use std::fs;
use std::path::Path;

fn load_hwpspec() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/hwpspec.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse hwpspec.hwp")
}

#[test]
fn hwpspec_split_control_figures_no_duplicate_image_emit() {
    let doc = load_hwpspec();

    let page20_dump = doc.dump_page_items(Some(19));
    assert!(page20_dump.contains("pi=83"));
    assert!(page20_dump.contains("pi=86"));
    assert!(
        !page20_dump.contains("pi=89"),
        "Task #1086: pi=89 extended-control figure must start on page 21"
    );

    let page21_dump = doc.dump_page_items(Some(20));
    assert!(
        page21_dump.contains("pi=89"),
        "Task #1086: pi=89 extended-control figure must be on page 21"
    );

    // 페이지 20 = index 19, 페이지 21 = index 20
    let page20_svg = doc
        .render_page_svg_native(19)
        .expect("render hwpspec.hwp page 20");
    let page21_svg = doc
        .render_page_svg_native(20)
        .expect("render hwpspec.hwp page 21");

    let page20_image_count = page20_svg.matches("<image").count();
    let page21_image_count = page21_svg.matches("<image").count();
    assert_eq!(
        page20_image_count, 2,
        "회귀: 빈 문단 + TAC Picture 이중 emit (Task #376 정정 누락 회귀). \
        20 페이지 기대 2 (pi=83/86 각 1회), 실제 {page20_image_count}"
    );
    assert_eq!(
        page21_image_count, 1,
        "회귀: 빈 문단 + TAC Picture 이중 emit (Task #376 정정 누락 회귀). \
        21 페이지 기대 1 (pi=89 1회), 실제 {page21_image_count}"
    );
}
