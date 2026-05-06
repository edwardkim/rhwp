//! Issue #595: exam_math.hwp 2페이지부터 수식 더블클릭 hitTest 오동작.
//!
//! 본질: `src/renderer/layout.rs::build_header` 의 `expand_bbox_to_children`
//! 호출이 머리말 자식 노드 (특히 단 구분선 line `paraIdx=0 ci=2`, h≈1227px) 의
//! bbox 까지 Header 영역으로 확장 → `hit_test_header_footer_native` 가 본문 좌표를
//! 머리말 hit 으로 잘못 인식 → `onDblClick` 의 머리말 분기가 picture selection
//! 분기보다 먼저 실행되어 수식 편집기 진입 차단.
//!
//! 발현: page 0 (1p) 정상 / page 1+ (2p~) 결함.
//!
//! 본 테스트는 정정 전 fail / 정정 후 pass — 회귀 차단 영구 가드.

use std::fs;
use std::path::Path;

fn load_exam_math() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/exam_math.hwp");
    let bytes = fs::read(&hwp_path).expect("read exam_math.hwp");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_math.hwp")
}

/// page 0 (1-based 1) 의 본문 좌표 (514, 200) 는 머리말 hit 이 아니어야 한다.
/// 정상 머리말 영역은 y=0~147 정도. 본 테스트는 baseline (정정 전에도 통과 예상).
#[test]
fn issue_595_page0_body_coord_not_header() {
    let doc = load_exam_math();
    let r = doc.hit_test_header_footer_native(0, 514.0, 200.0).unwrap();
    assert!(
        r.contains("\"hit\":false"),
        "page 0 본문 좌표 (514, 200) 가 머리말 hit 으로 잘못 인식됨: {}",
        r
    );
}

/// page 1 (1-based 2) 의 본문 좌표 (514, 200) 는 머리말 hit 이 아니어야 한다.
/// 정정 전: hit:true (Header bbox 가 본문 영역 60~1355 까지 침범)
/// 정정 후: hit:false (정상 머리말 영역 60~145 로 제한)
#[test]
fn issue_595_page1_body_coord_not_header_regression_guard() {
    let doc = load_exam_math();
    let r = doc.hit_test_header_footer_native(1, 514.0, 200.0).unwrap();
    assert!(
        r.contains("\"hit\":false"),
        "page 1 본문 좌표 (514, 200) 가 머리말 hit 으로 잘못 인식됨 (Issue #595 회귀): {}",
        r
    );
}

/// 이슈 명세 정확 좌표 — page 1 의 paraIdx=65 ci=0 수식 영역 (654.5, 209.7).
/// 이 좌표는 본문 영역의 수식 객체 위치이며 머리말 hit 이 아니어야 한다.
#[test]
fn issue_595_page1_equation_coord_not_header() {
    let doc = load_exam_math();
    let r = doc.hit_test_header_footer_native(1, 654.5, 209.7).unwrap();
    assert!(
        r.contains("\"hit\":false"),
        "page 1 수식 좌표 (654.5, 209.7) 가 머리말 hit 으로 잘못 인식됨 (Issue #595): {}",
        r
    );
}

/// page 1 의 페이지 중앙 본문 영역 (514, 800) 도 머리말 hit 이 아니어야 한다.
/// 본문 한가운데 — 명백히 머리말 영역 밖.
#[test]
fn issue_595_page1_body_center_not_header() {
    let doc = load_exam_math();
    let r = doc.hit_test_header_footer_native(1, 514.0, 800.0).unwrap();
    assert!(
        r.contains("\"hit\":false"),
        "page 1 본문 중앙 (514, 800) 이 머리말 hit 으로 잘못 인식됨 (Issue #595): {}",
        r
    );
}

/// page 1 의 머리말 영역 좌표 (514, 100) 은 정상적으로 머리말 hit 이어야 한다.
/// 정정으로 인해 머리말 영역 자체의 hit 이 사라지지 않아야 함을 보장.
#[test]
fn issue_595_page1_header_area_still_hits() {
    let doc = load_exam_math();
    let r = doc.hit_test_header_footer_native(1, 514.0, 100.0).unwrap();
    assert!(
        r.contains("\"hit\":true") && r.contains("\"isHeader\":true"),
        "page 1 머리말 영역 (514, 100) 이 머리말 hit 으로 인식되어야 함 (정정 회귀 가드): {}",
        r
    );
}
