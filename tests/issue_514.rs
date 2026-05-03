//! Issue #514: 복학원서.hwp s0:pi=0 BehindText PCX 그림 출력 누락 회귀
//!
//! 본질: HWP 파일이 학교 로고 등을 PCX 포맷 (`BIN0001.PCX`, ZSoft Paintbrush v3.0+)
//! 으로 임베드하는 경우 rhwp 의 `detect_image_mime_type` 가 PCX 를 인식하지 못해
//! `application/octet-stream` 으로 emit → 브라우저가 이미지로 렌더링 불가.
//!
//! 정정: `detect_image_mime_type` 에 PCX 시그니처 (0x0A 0x05) 분기 추가 +
//! `pcx_bytes_to_png_bytes` 변환 함수로 PCX → PNG (RGBA, 흰색 → 투명) 변환 후 emit.
//!
//! 회귀 검증 (samples/복학원서.hwp):
//! - SVG 의 학교 로고 image href 가 `data:image/png;base64,iVBORw0K` 로 시작
//!   (PNG magic 의 base64 prefix)
//! - octet-stream 폴백이 발생하지 않음

use std::fs;
use std::path::Path;

#[test]
fn issue_514_pcx_logo_converted_to_png() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 복학원서.hwp");

    let svg = doc
        .render_page_svg_native(0)
        .expect("render 복학원서.hwp page 1");

    // 정정 후: 학교 로고 (PCX) → PNG 변환되어 image/png 로 emit
    assert!(
        svg.contains("data:image/png;base64,"),
        "PCX 로고가 PNG 로 변환되어 SVG 에 emit 되어야 함"
    );

    // 회귀 가드: octet-stream 폴백이 발생하지 않음
    assert!(
        !svg.contains("data:application/octet-stream"),
        "PCX 변환 실패 시 octet-stream 폴백이 발생하면 안 됨 (브라우저 렌더링 불가)"
    );

    // PNG magic prefix 검증: PNG 의 첫 8 바이트 (89 50 4E 47 0D 0A 1A 0A) 의
    // base64 인코딩은 'iVBORw0KGgo' 로 시작
    assert!(
        svg.contains("data:image/png;base64,iVBORw0KGgo"),
        "PNG 데이터가 정상 PNG magic 으로 시작해야 함"
    );
}

#[test]
fn issue_514_jpeg_watermark_unchanged() {
    // 회귀 가드: PCX 변환 분기가 다른 포맷 (JPEG 워터마크) 에 영향 없음
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = fs::read(&hwp_path).expect("read 복학원서.hwp");

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 복학원서.hwp");

    let svg = doc
        .render_page_svg_native(0)
        .expect("render 복학원서.hwp page 1");

    // 워터마크 (JPEG) 는 변경 없이 유지
    assert!(
        svg.contains("data:image/jpeg;base64,"),
        "JPEG 워터마크는 변경 없이 emit 되어야 함"
    );
}

#[test]
fn issue_514_pcx_to_png_conversion_unit() {
    // 직접 변환 함수 단위 검증 (svg.rs::pcx_bytes_to_png_bytes 가 pub(crate) 라
    // 외부 접근 불가하므로, HWP 파일 단계에서 종합 검증으로 충분).
    //
    // 본 테스트는 `pcx_bytes_to_png_bytes` 가 paletted PCX (BIN0001.PCX) 를
    // 변환할 수 있는지 확인하는 종합 테스트와 중복. issue_514_pcx_logo_converted_to_png
    // 가 이미 변환 결과를 검증하므로 별도 단위 테스트는 생략.
    //
    // 추후 RGB PCX fixture 추가 시 단위 테스트 신설 권장.
}
