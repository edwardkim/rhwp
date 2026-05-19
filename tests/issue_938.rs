//! Issue #938: 복학원서 중앙 워터마크 JPEG 한컴 정답지 톤 보정 회귀
//!
//! 본질: `samples/복학원서.hwp` 의 중앙 워터마크는 alpha 없는 JPEG 이며,
//! 흰색 배경까지 필터/opacity/multiply 대상이 되어 옅은 사각 영역이 보였다.
//! 워터마크 JPEG 에 한정해 한컴 PDF 정답지에 가까운 회색 톤 opaque PNG 로
//! 선보정하고, 이후 런타임 필터를 중복 적용하지 않아야 한다.

use base64::Engine;
use image::GenericImageView;
use serde_json::Value;

fn decode_data_uri(uri: &str) -> Option<(&str, Vec<u8>)> {
    let (mime, data) = uri.strip_prefix("data:")?.split_once(";base64,")?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .ok()?;
    Some((mime, bytes))
}

fn extract_svg_image_data(svg: &str) -> Vec<(String, Vec<u8>)> {
    let mut images = Vec::new();
    let mut idx = 0usize;
    while let Some(found) = svg[idx..].find("data:image/") {
        let start = idx + found;
        let end = svg[start..]
            .find('"')
            .map(|rel| start + rel)
            .unwrap_or(svg.len());
        if let Some((mime, bytes)) = decode_data_uri(&svg[start..end]) {
            images.push((mime.to_string(), bytes));
        }
        idx = end.saturating_add(1);
    }
    images
}

#[derive(Debug)]
struct WatermarkToneStats {
    dims: (u32, u32),
    min_alpha: u8,
    max_alpha: u8,
    mean_gray: f64,
    visible_count: u64,
    visible_p10: u8,
    visible_p50: u8,
}

fn percentile(sorted: &[u8], p: f64) -> u8 {
    if sorted.is_empty() {
        return 255;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx]
}

fn watermark_tone_stats(bytes: &[u8]) -> WatermarkToneStats {
    let img = image::load_from_memory(bytes)
        .expect("decode image")
        .to_rgba8();
    let dims = img.dimensions();
    let mut min_alpha = u8::MAX;
    let mut max_alpha = 0u8;
    let mut gray_sum = 0u64;
    let mut visible = Vec::new();
    for px in img.pixels() {
        let a = px.0[3];
        min_alpha = min_alpha.min(a);
        max_alpha = max_alpha.max(a);
        let gray = ((px.0[0] as u16 + px.0[1] as u16 + px.0[2] as u16) / 3) as u8;
        gray_sum += gray as u64;
        if gray < 250 {
            visible.push(gray);
        }
    }
    visible.sort_unstable();
    let total = (dims.0 as f64) * (dims.1 as f64);
    WatermarkToneStats {
        dims,
        min_alpha,
        max_alpha,
        mean_gray: gray_sum as f64 / total,
        visible_count: visible.len() as u64,
        visible_p10: percentile(&visible, 0.10),
        visible_p50: percentile(&visible, 0.50),
    }
}

fn collect_image_ops<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if value.get("type").and_then(Value::as_str) == Some("image") {
                out.push(value);
            }
            for child in map.values() {
                collect_image_ops(child, out);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_image_ops(child, out);
            }
        }
        _ => {}
    }
}

#[test]
fn issue_938_svg_watermark_is_hancom_baked_png() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let svg = doc
        .render_page_svg_native(0)
        .expect("render 복학원서.hwp page 1");
    let images = extract_svg_image_data(&svg);
    assert!(
        images.len() >= 2,
        "복학원서 1쪽은 학교 로고 + 중앙 워터마크 이미지를 포함해야 함"
    );
    assert!(
        !images.iter().any(|(mime, _)| mime == "image/jpeg"),
        "중앙 워터마크 JPEG 는 한컴 정답지 톤 PNG 로 emit 되어야 함"
    );
    assert!(
        !svg.contains("opacity=\"0.17\""),
        "baked 워터마크에는 SVG opacity 를 중복 적용하지 않아야 함"
    );

    let stats = images
        .iter()
        .find_map(|(mime, bytes)| {
            let stats = watermark_tone_stats(bytes);
            if mime == "image/png" && stats.dims == (728, 729) {
                Some(stats)
            } else {
                None
            }
        })
        .expect("728x729 중앙 워터마크 PNG 를 찾아야 함");

    assert_eq!(
        stats.min_alpha, 255,
        "baked PNG 는 정답 PDF 처럼 opaque 여야 함"
    );
    assert_eq!(
        stats.max_alpha, 255,
        "baked PNG 는 정답 PDF 처럼 opaque 여야 함"
    );
    assert!(
        (236.0..=244.0).contains(&stats.mean_gray),
        "정답 PDF 워터마크 평균 회색값 근처여야 함: {:?}",
        stats
    );
    assert!(
        (230_000..=330_000).contains(&stats.visible_count),
        "정답 PDF 워터마크 가시 픽셀 수 근처여야 함: {:?}",
        stats
    );
    assert!(
        (190..=210).contains(&stats.visible_p10),
        "짙은 엠블럼 톤이 정답 PDF 근처여야 함: {:?}",
        stats
    );
    assert!(
        (232..=244).contains(&stats.visible_p50),
        "중앙 워터마크 중간 톤이 정답 PDF 근처여야 함: {:?}",
        stats
    );
}

#[test]
fn issue_938_layer_tree_watermark_is_resolved_hancom_baked_png() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc
        .get_page_layer_tree_native(0)
        .expect("layer tree page 1");
    let parsed: Value = serde_json::from_str(&json).expect("PageLayerTree JSON");
    let mut images = Vec::new();
    collect_image_ops(&parsed, &mut images);

    let watermark = images
        .iter()
        .copied()
        .find(|entry| entry.get("watermark").is_some())
        .expect("watermark PaintOp::Image");

    assert_eq!(watermark["mime"], "image/png");
    assert_eq!(watermark["effect"], "grayScale");
    assert_eq!(watermark["brightness"], -50);
    assert_eq!(watermark["contrast"], 70);
    assert_eq!(
        watermark["bakedWatermark"], true,
        "PageLayerTree image op must carry resolved baked watermark state"
    );
    assert!(
        !images
            .iter()
            .any(|entry| entry.get("watermark").is_some() && entry["mime"] == "image/jpeg"),
        "watermark PaintOp::Image should no longer expose the original JPEG payload"
    );

    let base64 = watermark["base64"].as_str().expect("base64");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64)
        .expect("decode layer tree image");
    let stats = watermark_tone_stats(&bytes);
    assert_eq!(stats.dims, (728, 729));
    assert_eq!(stats.min_alpha, 255);
    assert_eq!(stats.max_alpha, 255);
    assert!((236.0..=244.0).contains(&stats.mean_gray), "{:?}", stats);
    assert!(
        (230_000..=330_000).contains(&stats.visible_count),
        "{:?}",
        stats
    );
    assert!((190..=210).contains(&stats.visible_p10), "{:?}", stats);
    assert!((232..=244).contains(&stats.visible_p50), "{:?}", stats);
}

#[test]
fn issue_938_overlay_watermark_is_hancom_baked_png() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc
        .get_page_overlay_images_native(0)
        .expect("overlay images");
    let parsed: Value = serde_json::from_str(&json).expect("overlay JSON");
    let behind = parsed["behind"].as_array().expect("behind array");
    let watermark = behind
        .iter()
        .find(|entry| entry.get("watermark").is_some())
        .expect("watermark overlay");

    assert_eq!(
        watermark["mime"], "image/png",
        "Studio overlay 도 baked PNG 데이터를 받아야 함"
    );
    assert_eq!(watermark["effect"], "grayScale");
    assert_eq!(watermark["brightness"], -50);
    assert_eq!(watermark["contrast"], 70);
    assert_eq!(
        watermark["bakedWatermark"], true,
        "Studio 는 baked 워터마크에 CSS filter/opacity 를 중복 적용하지 않아야 함"
    );

    let base64 = watermark["base64"].as_str().expect("base64");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64)
        .expect("decode overlay image");
    let stats = watermark_tone_stats(&bytes);
    assert_eq!(stats.dims, (728, 729));
    assert_eq!(stats.min_alpha, 255);
    assert_eq!(stats.max_alpha, 255);
    assert!((236.0..=244.0).contains(&stats.mean_gray), "{:?}", stats);
    assert!(
        (230_000..=330_000).contains(&stats.visible_count),
        "{:?}",
        stats
    );
    assert!((190..=210).contains(&stats.visible_p10), "{:?}", stats);
    assert!((232..=244).contains(&stats.visible_p50), "{:?}", stats);
}
