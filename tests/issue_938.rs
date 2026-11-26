//! 관계형·고정형: 독립 Print의 내용·소유·배치와 기존 회귀 역할을 보호한다.
//! 주석 갱신: 2026-11-22T02:16:55+09:00; 직전 파일 변경 커밋: 2026-10-17T08:37:43+09:00 d1c1f8d8300ad3f9cf5abe00b41300263186163b.
//! 측정: 2026-11-22 KST, Print profile, 96DPI, PDFium, 색상 경계 보정 전2px raw.
//! 원본 복학원서 HWP 전1쪽, Native 99.00% / fresh WASM 98.99% (전쪽 최저도 동일).
//! 근거: mydocs/pr/assets/issues7754_7765/source32/README.md와 문서별 Native/WASM TSV.
//! 원본·PDF·생산 소스·명령은 qualified-documents.json에 연결한다.
//! 설치 글꼴을 요구하지 않으며 직접 확대 판독을 완료했다.
//!
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
    assert_eq!(watermark["wrap"], "behindText");
    assert_eq!(
        watermark["effect"], "realPic",
        "이미 bake한 PNG에 필터를 중복 적용하지 않는다"
    );
    assert_eq!(watermark["brightness"], 0);
    assert_eq!(watermark["contrast"], 0);
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
    let svg = doc.render_page_svg_native(0).expect("동일 원본의 SVG");
    let svg_images = extract_svg_image_data(&svg);
    assert!(
        svg_images
            .iter()
            .any(|(mime, payload)| mime == "image/png" && payload == &bytes),
        "SVG와 layer 경로는 같은 bake 결과를 사용한다"
    );
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
fn issue_677_bokhak_receipt_preserves_table_and_host_order() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn compact(node: &RenderNode) -> String {
        let mut text = match &node.node_type {
            RenderNodeType::TextRun(run) => run.text.clone(),
            _ => String::new(),
        };
        for child in &node.children {
            text.push_str(&compact(child));
        }
        text.chars()
            .filter(|c| !c.is_whitespace() && !c.is_control())
            .collect()
    }
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            collect(child, out);
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/복학원서.hwp");
    let core = rhwp::DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        core.page_count(),
        1,
        "독립 Print의 한 쪽에 원서와 접수증을 보존한다"
    );
    let page = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&page.root, &mut nodes);
    let table = |pi| {
        let found: Vec<_> = nodes
            .iter()
            .copied()
            .filter(|n| {
                matches!(
                    &n.node_type, RenderNodeType::Table(t) if t.para_index == Some(pi)
                        && t.section_index == Some(0) && t.control_index == Some(0)
                )
            })
            .collect();
        assert_eq!(found.len(), 1, "원본 표 {pi}를 한 번만 소유한다");
        found[0]
    };
    let application = table(2);
    let signature = table(12);
    let receipt = table(16);
    let first_body = nodes
        .iter()
        .find(|n| {
            matches!(
                &n.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(4)
            ) && compact(n).contains("본인은휴학으로")
        })
        .expect("원서 뒤의 실제 신청 문장");
    let separator = nodes
        .iter()
        .find(|n| {
            matches!(
                &n.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(16)
            ) && compact(n).contains("---")
        })
        .expect("접수증 앞 host의 점선");
    let addressee = nodes
        .iter()
        .find(|n| {
            matches!(
                &n.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(14)
            ) && compact(n).contains("고려대학교총장귀하")
        })
        .expect("서명 뒤 수신인");
    let bottom = |n: &RenderNode| n.bbox.y + n.bbox.height;
    assert!(bottom(application) <= first_body.bbox.y);
    assert!(bottom(signature) <= addressee.bbox.y);
    assert!(bottom(addressee) <= separator.bbox.y);
    assert!(
        bottom(separator) <= receipt.bbox.y,
        "host의 점선과 접수증 표는 각각 자기 줄을 소유한다"
    );
    let receipt_text = compact(receipt);
    for phrase in [
        "복학원서접수증",
        "FilingReceipt",
        "위학생의복학원서를접수함.",
    ] {
        assert_eq!(
            receipt_text.matches(phrase).count(),
            1,
            "접수증의 {phrase}를 한 번 보존한다"
        );
    }
    for owner in [application, signature, receipt] {
        let mut contents = Vec::new();
        collect(owner, &mut contents);
        for line in contents.into_iter().filter(|n| {
            matches!(n.node_type, RenderNodeType::TextLine(_)) && !compact(n).is_empty()
        }) {
            assert!(
                line.bbox.y >= owner.bbox.y && bottom(line) <= bottom(owner),
                "표의 실제 글줄은 소유 상자 안에 있어야 한다: {}",
                compact(line)
            );
        }
    }
}
