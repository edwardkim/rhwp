//! Issue #1611: 렌더링 −1쪽 갭 요인 B — footer(발신명의) PAGE-앵커 블록 page-fit 누적 과소.
//!
//! 발신명의 footer 블록(`VertRelTo::Page` + `valign=Bottom` + `wrap=TopAndBottom`, 비-TAC)을
//! TypesetEngine 이 stored vpos 에 동기화하지 않고 본문 흐름 위치(flowed `current_height`)에
//! 배치해, page-fit 판정이 ~30~62px 과소되어 footer 가 본문 페이지에 흡수된다.
//!
//! 대표 케이스 `36387725` (서울시 opengov 결재문서):
//! - 한글 정답지(통제셋 `render_page_controlset.tsv`): **2쪽** (page1 본문 / page2 발신명의).
//! - footer stored vpos=48053 HU(≈640.7px) + 선언 351.4px = ≈992.1px > body 990.2px → 분할.
//! - 수정 전(버그): footer 를 flowed cur_h(≈578px)에 배치 → 929.8px ≤ 990.2 → **1쪽**.
//!
//! 정정(typeset.rs): `VertRelTo::Page`+`Bottom` TopAndBottom 블록의 `current_height` 를
//! stored vpos 에 동기화 → fit 체크가 vpos 하단(992.1)으로 판정 → 분할(2쪽, 한글 일치).

use std::fs;
use std::path::Path;

#[test]
fn issue_1611_footer_page_bottom_splits_to_second_page() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwpx_path =
        Path::new(repo_root).join("samples/hwpx/opengov/36387725_footer_page_bottom.hwpx");
    let bytes =
        fs::read(&hwpx_path).unwrap_or_else(|e| panic!("read {}: {}", hwpx_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 36387725_footer_page_bottom.hwpx");

    // 한글 정답지 2쪽 — 발신명의 footer 가 본문 직후 흡수되지 않고 다음 쪽으로 분할되어야 함.
    assert_eq!(
        doc.page_count(),
        2,
        "발신명의 footer(Page+Bottom) page-fit 누적 과소로 1쪽에 흡수됨 (#1611 요인 B)"
    );
}
