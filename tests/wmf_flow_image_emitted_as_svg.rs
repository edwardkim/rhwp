//! WMF 로 임베드된 본문 그림은 studio 가 지나는 좁은 질의에서 **SVG 로** 나가야 한다.
//!
//! 브라우저는 WMF 를 디코드하지 못한다. 그래서 `getPageFlowImageOps` 가 `image/x-wmf` 를
//! 실어 보내면 studio 는 그 바이트로 `Blob{type:"image/x-wmf"}` 를 만들고, `<img>` 는
//! `naturalWidth === 0` 인 깨진 그림이 된다.
//!
//! 이 결함이 실제로 있었다. `svg.rs`·`web_canvas.rs` 는 각자 내보내기 직전에
//! `convert_wmf_to_svg` 를 불러서 export-svg 와 canvas 백엔드는 멀쩡했는데, DOM `<img>`
//! 경로가 지나는 `emitted_image_bytes` 에는 BMP·PCX·TIFF·JPEG 분기만 있고 WMF 가 없었다.
//! 그래서 **한 경로만 깨졌고**, 다른 두 경로가 멀쩡해서 오래 눈에 띄지 않았다.
//!
//! `image_resolver` 의 단위 테스트(`wmf_is_emitted_as_svg_not_raw_wmf`)는 합성 WMF 로 변환
//! 분기를 고정한다. 이 테스트는 그 위에서 **실제 문서**로 같은 계약을 고정한다 — 합성
//! 바이트는 파서·레이아웃을 지나지 않으므로, 그림이 본문 flow 그림으로 분류되고 좁은
//! 질의에 실리기까지의 사슬은 실물 fixture 로만 확인된다.
//!
//! fixture: 관세청 "2024년 5월 월간 수출입 현황(확정치)" — 1쪽 하단 표 안에 WMF 차트 2장.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;

const SAMPLE: &str = "samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp";

/// 좁은 질의 JSON 에서 `"mime":"..."` 값만 뽑는다.
///
/// JSON 파서를 끌어오지 않는 이유는 이 테스트가 보는 것이 **문자열로 나가는 mime** 이기
/// 때문이다. 소비자(studio)도 이 필드를 그대로 읽어 `Blob` 타입으로 쓴다.
fn mimes_in(json: &str) -> Vec<String> {
    const KEY: &str = "\"mime\":\"";
    let mut out = Vec::new();
    let mut rest = json;
    while let Some(at) = rest.find(KEY) {
        rest = &rest[at + KEY.len()..];
        match rest.find('"') {
            Some(end) => {
                out.push(rest[..end].to_string());
                rest = &rest[end..];
            }
            None => break,
        }
    }
    out
}

#[test]
fn wmf_flow_images_are_emitted_as_svg_not_raw_wmf() {
    let bytes = std::fs::read(SAMPLE).expect("fixture 를 읽을 수 있어야 한다");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");

    let page_count = core.page_count();
    assert!(page_count > 0, "페이지가 없으면 아무것도 검증하지 못한다");

    let mut all = Vec::new();
    for page in 0..page_count {
        let json = core
            .get_page_flow_image_ops_native(page)
            .unwrap_or_else(|e| panic!("{page}쪽 좁은 질의 실패: {e}"));
        all.extend(mimes_in(&json));
    }

    // 이 fixture 에 WMF 그림이 있다는 전제를 먼저 못박는다. 문서가 바뀌어 WMF 가 사라지면
    // 아래 단언은 자동으로 통과해버리므로, 그때는 이 테스트가 아무것도 지키지 않는다.
    let svg = all.iter().filter(|m| *m == "image/svg+xml").count();
    assert!(
        svg >= 2,
        "fixture 의 WMF 차트 2장이 SVG 로 나와야 한다 — 관측된 mime: {all:?}"
    );

    assert!(
        !all.iter().any(|m| m == "image/x-wmf"),
        "원본 WMF 가 그대로 나가면 브라우저가 못 그린다 (naturalWidth 0) — 관측된 mime: {all:?}"
    );
}
