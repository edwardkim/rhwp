//! [Issue #5821] 표 안 제목 글자를 선언 크기 그대로 그려 괘선이 글자를 가로지른다
//! — 한글은 5% 작게 (156601658 제목 상자 등).
//!
//! 근인(오라클 확정): 한글 2022 는 **압축 장평(ratio<100%) 글자를 세로도 √r 로**
//! 줄인다 — COM 으로 뜬 원본 PDF 실측: 선언 25pt·ratio 90% → `Tf 23.706`
//! = 25×√0.90(오차 0.05%), 총 폭은 선언×0.90(한글 630.1px ↔ rhwp 634.0px)로
//! 종전과 같다. rhwp 는 세로를 선언 그대로(33.33px) 그려 5.4% 크고 3.8px 위라
//! 위/아래 괘선에 닿았다.
//!
//! 수정: glyph 크기 = fs×√r, 가로 스케일 = √r (폭 ×r 불변, advance 불변) —
//! SSOT `condensed_ratio_draw_params`, svg/web_canvas/skia 3벡엔드 공용.
//! 보도일시 칸 축(156513948·156560092, ratio=100%)은 별개 축으로 잔존.
//!
//! 픽스처는 원본 HWPX 구역0 문단 3..5(제목 상자 표) 절단 + BinData 스텁(69KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5821/condensed_ratio_title_box.hwpx";

#[test]
fn issue_5821_condensed_ratio_shrinks_glyph_height_sqrt_r() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 제목 글리프: 선언 2500HU(33.33px)·ratio 90% → 그리기 33.33×√0.90 = 31.62px
    // + scale(√0.90=0.9487). 결함 시 font-size 33.33 + scale(0.9).
    let shrunk = svg.matches("font-size=\"31.62").count();
    assert!(
        shrunk >= 40,
        "제목 글리프가 √r 축소 크기(31.62px)로 그려져야 한다 (결함 시 33.33): {shrunk}"
    );
    assert!(
        svg.contains("scale(0.9487"),
        "가로 스케일이 √0.90 이어야 총 폭이 선언×0.90 으로 유지된다"
    );
    assert!(
        !svg.contains("font-size=\"33.33"),
        "선언 크기 그대로 그려진 제목 글리프가 남아 있다"
    );
}
