//! Issue #418: hwpspec.hwp 20 페이지의 빈 문단 + TAC Picture 가
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
//! 정정 후 기대: <image> 3 개 (pi=83, 86, 89 각 1회).

use std::fs;
use std::path::Path;

#[test]
fn hwpspec_page20_no_duplicate_image_emit() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/hwpspec.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse hwpspec.hwp");

    // 페이지 20 = index 19
    let svg = doc
        .render_page_svg_native(19)
        .expect("render hwpspec.hwp page 20");

    // <image> 요소 개수 검증
    let image_count = svg.matches("<image").count();
    assert_eq!(
        image_count, 3,
        "회귀: 빈 문단 + TAC Picture 이중 emit (Task #376 정정 누락 회귀). \
        기대 3 (pi=83/86/89 각 1회), 실제 {image_count}"
    );
}
