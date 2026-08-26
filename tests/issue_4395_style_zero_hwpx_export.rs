//! Issue #4395: HWPX 문서에 `<hh:styles>` 블록이 없으면(스타일 목록이 비어 있으면) 문단이
//! 참조하는 암묵적 기본 `styleIDRef="0"` 이 재직렬화 자기검증에서 "미등록 ID 참조"로 하드
//! 실패했다 — HWP5 를 거치지 않는 순수 HWPX 자기 왕복(`parse_hwpx` → `serialize_hwpx`)에서도,
//! HWPX→HWP→HWPX 왕복에서도 동일하게 실패했다.
//!
//! `src/serializer/hwpx/header.rs`의 `write_styles` 는 `doc_info.styles` 가 비어 있으면
//! `<hh:styles>` 블록 자체를 생략한다(정상 동작). 하지만
//! `src/serializer/hwpx/context.rs`의 `SerializeContext::collect_from_document` 는 그 목록을
//! 순회해서만 style id 를 등록했으므로, 목록이 비면 0 도 등록되지 않았다.
//! `effective_style_id`(같은 파일)의 주석은 "0(항상 등록됨)"이라고 명시하는데 그 불변식이
//! 실제로 보장되지 않았던 것 — `samples/task2156/width_ladder.hwpx` 등 스타일 목록이 없는
//! 최소 픽스처가 전부 이 경로로 저장 자체가 불가능했다.

use std::fs;

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

/// 스타일 목록이 없는 실제 픽스처가 순수 HWPX 자기 왕복(HWP5 미경유)에서도 저장돼야 한다.
#[test]
fn hwpx_without_style_list_serializes_without_error() {
    let path = "samples/task2156/width_ladder.hwpx";
    let data = fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    let doc = parse_hwpx(&data).expect("HWPX 파싱 실패");

    assert!(
        doc.doc_info.styles.is_empty(),
        "이 픽스처는 <hh:styles> 가 없다는 전제로 작성된 회귀 테스트다"
    );

    serialize_hwpx(&doc).unwrap_or_else(|e| {
        panic!("스타일 목록이 비어 있어도 styleIDRef=0 참조는 항상 유효해야 한다 (#4395): {e}")
    });
}

/// 같은 픽스처 3종에서 재확인 — 전부 같은 근본 원인(#4395)으로 실패했었다.
#[test]
fn other_style_zero_fixtures_serialize_without_error() {
    for path in [
        "samples/task2169/anchor_ladder.hwpx",
        "samples/task2070/hy_ladder3.hwpx",
        "samples/task2169/empty_ladder.hwpx",
    ] {
        let data = fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
        let doc = parse_hwpx(&data).unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"));
        serialize_hwpx(&doc)
            .unwrap_or_else(|e| panic!("{path}: 미등록 styleIDRef=0 회귀 (#4395): {e}"));
    }
}
