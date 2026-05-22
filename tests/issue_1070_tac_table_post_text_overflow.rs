//! Issue #1070: 거의 한 페이지 크기 treat_as_char(TAC) 표가 첫 줄에 있고 그 뒤에
//! 본문 줄이 있는 문단에서, 후속 본문 텍스트가 표 높이만큼 추가로 하강해 편집영역
//! 하단을 넘어 렌더링되는 결함의 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플):
//! - `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` (page 2, pi=25, 472px)
//! - `samples/hwpx/hwpx-h-02.hwpx` (page 2, pi=51, 348px)
//! - `samples/hwpx/2025년 2분기 해외직접투자 (최종).hwpx` (page 2, pi=51, 348px)
//!
//! 결함 본질: `place_table_with_text` (typeset.rs) 의 `post_table_start` 산식이
//! `attr & 0x01` (HWP5 TAC 비트) 에만 의존 → HWPX TAC 표(비트0=0)는 표줄(line0)을
//! post-text PartialParagraph 에 포함 → 후속 본문 줄이 표 높이만큼 하강.
//! 수정: `treat_as_char && total_lines > pre_end + 1` 일 때 표줄을 post-text 에서 제외
//! (HWP5 `pre_end.max(1)` 와 정합, 단일줄 TAC 표는 불변).

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

/// SVG 의 `height="..."` 속성값(물리 페이지 높이 px).
fn svg_height(svg: &str) -> f64 {
    let i = svg.find("height=\"").expect("svg height attr") + "height=\"".len();
    let rest = &svg[i..];
    let end = rest.find('"').expect("height close");
    rest[..end].parse().expect("height f64")
}

/// SVG 내 모든 `<text ... y="..."`> 의 최대 y 좌표(px). 없으면 0.
fn max_text_y(svg: &str) -> f64 {
    let mut max = 0.0_f64;
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let after = &rest[open..];
        let tag_end = after.find('>').map(|g| open + g).unwrap_or(rest.len());
        let tag = &rest[open..tag_end];
        if let Some(yi) = tag.find(" y=\"") {
            let yrest = &tag[yi + 4..];
            if let Some(ye) = yrest.find('"') {
                if let Ok(y) = yrest[..ye].parse::<f64>() {
                    max = max.max(y);
                }
            }
        }
        rest = &rest[tag_end..];
    }
    max
}

/// 지정 페이지에서 본문 텍스트가 물리 페이지 높이를 넘지 않음을 검증.
/// 결함 시 후속 본문이 y ≈ 1490px (페이지 높이 1122px 초과) 로 그려진다.
fn assert_no_text_below_page(rel: &str, page: u32) {
    let doc = load_doc(rel);
    let svg = doc
        .render_page_svg_native(page)
        .expect("render_page_svg_native");
    let h = svg_height(&svg);
    let max_y = max_text_y(&svg);
    assert!(
        max_y <= h,
        "{rel} page {page}: text max_y={max_y:.1} 가 페이지 높이 {h:.1} 초과 (TAC 표 post-text 하강 회귀)"
    );
}

#[test]
fn gibu_dalryepum_page2_no_text_overflow() {
    assert_no_text_below_page("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx", 2);
}

#[test]
fn hwpx_h_02_page2_no_text_overflow() {
    assert_no_text_below_page("samples/hwpx/hwpx-h-02.hwpx", 2);
}

#[test]
fn haewoe_jikjeop_tuja_page2_no_text_overflow() {
    assert_no_text_below_page("samples/hwpx/2025년 2분기 해외직접투자 (최종).hwpx", 2);
}
