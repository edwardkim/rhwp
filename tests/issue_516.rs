//! Issue #516: rhwp-studio (web Canvas) 그림 워터마크 효과 미적용 + AI 메타정보
//!
//! 본질: web_canvas.rs 의 image render 가 image.effect / brightness / contrast 를
//! CSS filter 로 적용하지 않아 한컴 워터마크 효과가 안 보임. 또한 dump / PageLayerTree
//! JSON 에 워터마크 메타정보가 없어 AI 활용 불가.
//!
//! 정정:
//! - ImageAttr 헬퍼 (`is_watermark`, `is_hancom_watermark_preset`, `watermark_preset`)
//! - rhwp dump 의 그림 출력에 [image_attr] 줄 추가 (effect/brightness/contrast/watermark)
//! - PaintOp::Image JSON 에 "watermark":{"preset":"..."} 조건부 추가
//! - web_canvas.rs 의 image render 시 CSS filter 적용 (Canvas 시각 정정, wasm 빌드 검증)
//!
//! 본 파일은 native cargo test 로 검증 가능한 항목 (헬퍼 + JSON) 만 포함.
//! Web Canvas CSS filter 적용은 Stage 5 시각 판정 게이트로 검증.

use rhwp::model::image::{ImageAttr, ImageEffect};

#[test]
fn issue_516_image_attr_helper_hancom_preset() {
    // 한컴 자동 프리셋: effect=GrayScale, brightness=70, contrast=-50
    let attr = ImageAttr {
        brightness: 70,
        contrast: -50,
        effect: ImageEffect::GrayScale,
        bin_data_id: 1,
    };
    assert!(attr.is_watermark(), "한컴 프리셋은 워터마크");
    assert!(attr.is_hancom_watermark_preset(), "한컴 자동 프리셋 정합");
    assert_eq!(attr.watermark_preset(), Some("hancom-watermark"));
}

#[test]
fn issue_516_image_attr_helper_custom_watermark() {
    // 복학원서.hwp 의 엠블렘: effect=GrayScale, brightness=-50, contrast=70
    // (편집자 의도적 사용자 정의 — 회색조 + 워터마크 후 슬라이더 추가 조정)
    let attr = ImageAttr {
        brightness: -50,
        contrast: 70,
        effect: ImageEffect::GrayScale,
        bin_data_id: 2,
    };
    assert!(attr.is_watermark(), "사용자 정의도 워터마크");
    assert!(!attr.is_hancom_watermark_preset(), "한컴 자동 프리셋 미정합");
    assert_eq!(attr.watermark_preset(), Some("custom"));
}

#[test]
fn issue_516_image_attr_helper_no_watermark() {
    // RealPic + b=c=0: 워터마크 아님
    let plain = ImageAttr::default();
    assert!(!plain.is_watermark());
    assert_eq!(plain.watermark_preset(), None);

    // GrayScale + b=c=0: 단순 흑백 변환, 워터마크 아님
    let gray_only = ImageAttr {
        effect: ImageEffect::GrayScale,
        ..ImageAttr::default()
    };
    assert!(!gray_only.is_watermark(), "effect-only 는 워터마크 아님");
    assert_eq!(gray_only.watermark_preset(), None);

    // RealPic + b/c 변경: bc-only 는 워터마크 아님 (effect 가 RealPic)
    let bc_only = ImageAttr {
        brightness: 30,
        contrast: 20,
        effect: ImageEffect::RealPic,
        bin_data_id: 0,
    };
    assert!(!bc_only.is_watermark(), "bc-only 는 워터마크 아님");
    assert_eq!(bc_only.watermark_preset(), None);
}

#[test]
fn issue_516_layer_tree_json_includes_watermark_for_emblem() {
    // 복학원서.hwp 의 가운데 엠블렘 (bin_id=2, GrayScale + b=-50 c=70)
    // PageLayerTree JSON 의 PaintOp::Image 에 "watermark":{"preset":"custom"} 포함
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc.get_page_layer_tree_native(0).expect("layer tree page 1");

    // 본 fixture 의 엠블렘은 custom 워터마크
    assert!(
        json.contains("\"watermark\":{\"preset\":\"custom\"}"),
        "복학원서.hwp 엠블렘이 PageLayerTree JSON 에 watermark.preset=custom 으로 직렬화"
    );
}

#[test]
fn issue_516_layer_tree_json_no_watermark_for_normal_image() {
    // 워터마크 미적용 그림 (예: pic-in-table-01.hwp 또는 다른 fixture) 의 JSON 에는
    // "watermark" 필드 부재. 본 테스트는 복학원서.hwp 의 학교 로고 (bin_id=1, RealPic)
    // 가 워터마크 미적용임을 활용 — JSON 에 학교 로고 image op 가 watermark 필드 없이
    // 직렬화됨.
    //
    // 정확히는 JSON 에 image op 가 2 개 (학교 로고 + 엠블렘) 있고, 그중 watermark 필드가
    // 정확히 1 개 (엠블렘) 만 출현해야 한다.
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc.get_page_layer_tree_native(0).expect("layer tree page 1");

    // watermark 필드 출현 횟수 = 1 (엠블렘만)
    let watermark_count = json.matches("\"watermark\":").count();
    assert_eq!(
        watermark_count, 1,
        "복학원서.hwp 의 페이지 1 에는 엠블렘 1개만 워터마크 필드를 가져야 함 (학교 로고는 RealPic 이라 watermark 필드 부재)"
    );
}

#[test]
fn issue_516_layer_tree_json_includes_wrap_for_behind_text() {
    // Stage 5.1: PaintOp::Image JSON 에 "wrap" 필드 노출.
    // 복학원서.hwp 의 학교 로고 (bin_id=1) + 엠블렘 (bin_id=2) 모두
    // text_wrap = BehindText 로 IR 에 저장되어 있다.
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc.get_page_layer_tree_native(0).expect("layer tree page 1");

    // 두 그림 모두 BehindText → JSON 에 "wrap":"behindText" 출현
    assert!(
        json.contains("\"wrap\":\"behindText\""),
        "복학원서.hwp 의 BehindText 그림에 wrap 필드가 직렬화되어야 함"
    );
}


#[test]
fn issue_516_diag_count_image_ops() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let json = doc.get_page_layer_tree_native(0).expect("layer tree");
    let image_count = json.matches("\"type\":\"image\"").count();
    let wrap_behind = json.matches("\"wrap\":\"behindText\"").count();
    let mime_png = json.matches("\"mime\":\"image/png\"").count();
    let mime_jpg = json.matches("\"mime\":\"image/jpeg\"").count();
    eprintln!("image ops: {}, wrap=behindText: {}, mime png: {}, mime jpg: {}",
              image_count, wrap_behind, mime_png, mime_jpg);
}

#[test]
fn issue_516_diag_image_op_locations() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let json = doc.get_page_layer_tree_native(0).expect("layer tree");

    // 모든 image op 위치의 직전 100 byte 표시
    let mut idx = 0;
    let mut count = 0;
    while let Some(found) = json[idx..].find("\"type\":\"image\"") {
        let abs = idx + found;
        let start = abs.saturating_sub(80);
        eprintln!("--- image op #{} at pos {}: ...{}", count, abs, &json[start..abs.min(json.len())]);
        // 그리고 mime 까지 포함
        let end = (abs + 100).min(json.len());
        eprintln!("    after image: {}", &json[abs..end]);
        idx = abs + 14;
        count += 1;
    }
}
