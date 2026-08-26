//! [Issue #5780] flow 그림이 있는 쪽은 studio 에서 쪽 배경색이 통째로 사라진다.
//!
//! 성북구 자원순환집행계획 표지: 구역 배경 `#1c3d62` + flow 그림 1장. studio 의
//! flow-static 분리에서 flow 그림 갈래는 canvas 가 아니라 DOM DIV 인데, 그 DIV 는
//! `var(--doc-paper)`(종이색)를 하드코딩해 Background plane 이 어느 평면에도 실리지
//! 않았다 — 표지가 통째로 흰 쪽이 된다(글자도 흰색이라 함께 사라짐).
//!
//! 수정: `get_page_overlay_images_native` 요약에 쪽 배경을 실어
//! (`pageBackgroundCss` 단색 / `pageBackgroundComplex` 그라데이션·이미지),
//! studio 가 단색은 DIV background 로 지고 복합 배경은 flow-static canvas 갈래로
//! 폴백한다(page-renderer.ts).
//!
//! 픽스처는 원본 표지 1쪽을 extract-pages 로 떼고 대형 BinData 를 1×1 스텁으로
//! 바꾼 marker-HWPX(67KB) — 남색 배경과 flow 그림 1장의 결함 유발 조합을 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5780/flow_image_page_background.hwpx";

#[test]
fn issue_5780_overlay_summary_carries_page_background() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let json = core
        .get_page_overlay_images_native(0)
        .expect("overlay summary");

    // 결함 유발 조합: flow 그림이 있어 DOM 갈래를 타는 쪽.
    assert!(
        json.contains("\"flowImageCount\":1"),
        "표지에 flow 그림 1장이 있어야 한다: {json}"
    );
    // 쪽 배경이 요약에 실려야 studio DIV 갈래가 배경을 잃지 않는다.
    assert!(
        json.contains("\"pageBackgroundCss\":\"#1c3d62\""),
        "쪽 배경색 #1c3d62 가 요약에 있어야 한다: {json}"
    );
    assert!(
        json.contains("\"pageBackgroundComplex\":false"),
        "단색 배경은 complex=false 여야 한다(캔버스 폴백 불필요): {json}"
    );
}
