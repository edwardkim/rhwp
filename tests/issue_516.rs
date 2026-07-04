//! Issue #516: rhwp-studio (web Canvas) 그림 워터마크 효과 미적용 + AI 메타정보
//!
//! 본질: web_canvas.rs 의 image render 가 image.effect / brightness / contrast 를
//! CSS filter 로 적용하지 않아 한컴 워터마크 효과가 안 보임. 또한 dump / PageLayerTree
//! JSON 에 워터마크 메타정보가 없어 AI 활용 불가.
//!
//! 정정:
//! - ImageAttr 헬퍼 (`is_watermark`, `watermark_preset`) — Issue #1156 에서
//!   밝기·대비 기반 판정으로 정정 (특정값 매칭 `is_hancom_watermark_preset` 폐기)
//! - rhwp dump 의 그림 출력에 [image_attr] 줄 추가 (effect/brightness/contrast/watermark)
//! - PaintOp::Image JSON 에 "watermark":{"preset":"..."} 조건부 추가
//! - web_canvas.rs 의 image render 시 CSS filter 적용 (Canvas 시각 정정, wasm 빌드 검증)
//!
//! 본 파일은 native cargo test 로 검증 가능한 항목 (헬퍼 + JSON) 만 포함.
//! Web Canvas CSS filter 적용은 Stage 5 시각 판정 게이트로 검증.

use rhwp::model::image::{ImageAttr, ImageEffect};

// [Issue #1156] 워터마크 판정 기준 확정:
// HWP/HWPX 에 워터마크 적용 비트는 존재하지 않는다 (한컴 공식 파일구조 3.0/5.0 +
// water-mark.hwp/.hwpx 두 그림 비교로 확정). 한컴 편집기는 "워터마크 효과" 해제 시
// 밝기·대비를 모두 0 으로 되돌리고, 적용 시 0 이 아닌 값을 부여한다 (기본 70/-50,
// 사용자 변경 가능). 따라서 워터마크 여부는 밝기·대비가 **둘 다 0 이 아닌 경우**로
// 판정한다 (effect 종류 무관, 한쪽이라도 0 이면 아님).
// 특정값 매칭(is_hancom_watermark_preset)은 폐기.

#[test]
fn issue_516_image_attr_helper_watermark_by_bright_contrast() {
    // 밝기·대비가 둘 다 0 이 아니면 워터마크 (effect 무관).
    // 한컴 기본 프리셋 (GrayScale 70/-50), 사용자 정의 (GrayScale -50/70),
    // RealPic 배경 워터마크 (143E: RealPic 70/-50) 모두 워터마크.
    for (b, c, e) in [
        (70i8, -50i8, ImageEffect::GrayScale),
        (-50, 70, ImageEffect::GrayScale),
        (70, -50, ImageEffect::RealPic),
        (30, 20, ImageEffect::RealPic),
        (-10, 5, ImageEffect::BlackWhite),
    ] {
        let attr = ImageAttr {
            brightness: b,
            contrast: c,
            effect: e,
            transparency: 0,
            bin_data_id: 1,
            external_path: None,
        };
        assert!(
            attr.is_watermark(),
            "밝기={b} 대비={c} effect={e:?} 는 워터마크 (둘 다 ≠0)"
        );
        assert_eq!(attr.watermark_preset(), Some("custom"));
    }
}

#[test]
fn issue_516_image_attr_helper_no_watermark() {
    // 밝기·대비 중 하나라도 0 이면 워터마크 아님 (effect 무관).
    // (0,0) 둘 다 0, (30,0) 대비만 0, (0,20) 밝기만 0 모두 워터마크 아님.
    for (b, c, e) in [
        (0i8, 0i8, ImageEffect::RealPic),
        (0, 0, ImageEffect::GrayScale),
        (0, 0, ImageEffect::BlackWhite),
        (30, 0, ImageEffect::RealPic),
        (0, 20, ImageEffect::GrayScale),
    ] {
        let attr = ImageAttr {
            brightness: b,
            contrast: c,
            effect: e,
            transparency: 0,
            bin_data_id: 0,
            external_path: None,
        };
        assert!(
            !attr.is_watermark(),
            "밝기={b} 대비={c} (effect={e:?}) 는 워터마크 아님 (한쪽이 0)"
        );
        assert_eq!(attr.watermark_preset(), None);
    }
}

#[test]
fn issue_516_layer_tree_json_includes_watermark_for_emblem() {
    // 복학원서.hwp 의 가운데 엠블렘 (bin_id=2, GrayScale + b=-50 c=70)
    // PageLayerTree JSON 의 PaintOp::Image 에 "watermark":{"preset":"custom"} 포함
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc
        .get_page_layer_tree_native(0)
        .expect("layer tree page 1");

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

    let json = doc
        .get_page_layer_tree_native(0)
        .expect("layer tree page 1");

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

    let json = doc
        .get_page_layer_tree_native(0)
        .expect("layer tree page 1");

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
    eprintln!(
        "image ops: {}, wrap=behindText: {}, mime png: {}, mime jpg: {}",
        image_count, wrap_behind, mime_png, mime_jpg
    );
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
        eprintln!(
            "--- image op #{} at pos {}: ...{}",
            count,
            abs,
            &json[start..abs.min(json.len())]
        );
        // 그리고 mime 까지 포함
        let end = (abs + 100).min(json.len());
        eprintln!("    after image: {}", &json[abs..end]);
        idx = abs + 14;
        count += 1;
    }
}
