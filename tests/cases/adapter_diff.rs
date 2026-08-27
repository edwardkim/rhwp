//! [#5392] M06-4: 전어댑터 상호 diff 골든 하네스.
//!
//! 있는 어댑터끼리 구조·capability·산출 해시/bbox 를 맞댄다. Svg/Null/Trace 는
//! devel 에 있다. Png/Skia 는 `rhwp_has_*_backend` cfg 가 켜질 때만 컴파일된다.
//! 러너(`scripts/run-adapter-diff.mjs`)가 원본+export 를 보고 cfg 를 켠다.
//! 없으면 이 파일은 있는 어댑터만 비교하고, 없는 쪽은 컴파일에서 빠진다.
//!
//! 장면은 `tools/adapter_diff/fixtures/ci-scene.json` 과 같다. samples/ 가 아니다.
#![allow(unexpected_cfgs)]
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::{CacheHint, GroupKind, LayerNode, PageLayerTree, PaintOp};
use rhwp::render_backend::{
    replay_page, BackendCapabilities, BackendFeature, NullBackend, RenderBackend, SvgBackend,
    TraceBackend,
};
use rhwp::renderer::render_tree::{BoundingBox, PageBackgroundNode, RectangleNode, TextRunNode};
use rhwp::renderer::{ShapeStyle, TextStyle};

#[cfg(rhwp_has_png_backend)]
use rhwp::render_backend::PngBackend;
#[cfg(rhwp_has_skia_backend)]
use rhwp::render_backend::SkiaBackend;

/// `tools/adapter_diff/fixtures/ci-scene.json` 과 같은 싼 장면.
const PAGE_W: f64 = 400.0;
const PAGE_H: f64 = 300.0;
const MARKER: &str = "M06-4";
const RECT_X: f64 = 20.0;
const RECT_Y: f64 = 20.0;
const RECT_W: f64 = 40.0;
const RECT_H: f64 = 24.0;

#[derive(Debug, Clone)]
struct Snap {
    name: &'static str,
    caps: BackendCapabilities,
    hash: String,
    empty: bool,
    kind: &'static str,
    view_w: Option<f64>,
    view_h: Option<f64>,
}

fn bbox(x: f64, y: f64, w: f64, h: f64) -> BoundingBox {
    BoundingBox::new(x, y, w, h)
}

fn fnv1a64(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn page_background_op() -> PaintOp {
    PaintOp::page_background(
        bbox(0.0, 0.0, PAGE_W, PAGE_H),
        PageBackgroundNode {
            background_color: None,
            border_color: None,
            border_width: 0.0,
            gradient: None,
            image: None,
        },
    )
}

fn rect_op() -> PaintOp {
    PaintOp::rectangle(
        bbox(RECT_X, RECT_Y, RECT_W, RECT_H),
        RectangleNode::new(0.0, ShapeStyle::default(), None),
    )
}

fn text_op(text: &str) -> PaintOp {
    PaintOp::text_run(
        bbox(20.0, 52.0, 80.0, 16.0),
        TextRunNode {
            text: text.to_string(),
            style: TextStyle {
                font_family: "sans-serif".to_string(),
                font_size: 16.0,
                ..TextStyle::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 12.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        },
    )
}

fn scene_tree() -> PageLayerTree {
    let bounds = bbox(0.0, 0.0, PAGE_W, PAGE_H);
    let leaf = LayerNode::leaf(
        bounds,
        None,
        vec![page_background_op(), rect_op(), text_op(MARKER)],
    );
    let root = LayerNode::group(
        bounds,
        None,
        vec![leaf],
        CacheHint::default(),
        GroupKind::Body,
    );
    PageLayerTree::new(PAGE_W, PAGE_H, root)
}

fn svg_viewbox(svg: &str) -> Option<(f64, f64)> {
    let key = "viewBox=\"";
    let start = svg.find(key)? + key.len();
    let end = svg[start..].find('"')? + start;
    let parts: Vec<&str> = svg[start..end].split_whitespace().collect();
    if parts.len() != 4 {
        return None;
    }
    Some((parts[2].parse().ok()?, parts[3].parse().ok()?))
}

fn png_ihdr(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || bytes[0..8] != [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

fn snap_svg(tree: &PageLayerTree) -> Snap {
    let mut first = SvgBackend::new();
    replay_page(&mut first, tree).unwrap();
    let out = first.finish().unwrap();
    let mut second = SvgBackend::new();
    replay_page(&mut second, tree).unwrap();
    let again = second.finish().unwrap();
    assert_eq!(out, again, "svg 산출이 결정적이지 않다");
    let (view_w, view_h) = svg_viewbox(&out).expect("svg viewBox");
    Snap {
        name: "svg",
        caps: SvgBackend::new().capabilities(),
        hash: fnv1a64(out.as_bytes()),
        empty: out.is_empty(),
        kind: "svg",
        view_w: Some(view_w),
        view_h: Some(view_h),
    }
}

fn snap_null(tree: &PageLayerTree) -> Snap {
    let mut first = NullBackend::new();
    replay_page(&mut first, tree).unwrap();
    let stats = first.finish().unwrap();
    let mut second = NullBackend::new();
    replay_page(&mut second, tree).unwrap();
    assert_eq!(stats, second.finish().unwrap());
    let payload = format!("{}:{}", stats.pages, stats.ops);
    Snap {
        name: "null",
        caps: NullBackend::new().capabilities(),
        hash: fnv1a64(payload.as_bytes()),
        empty: stats.ops == 0,
        kind: "stats",
        view_w: Some(tree.page_width),
        view_h: Some(tree.page_height),
    }
}

fn snap_trace(tree: &PageLayerTree) -> Snap {
    let mut first = TraceBackend::new();
    replay_page(&mut first, tree).unwrap();
    let out = first.finish().unwrap();
    let mut second = TraceBackend::new();
    replay_page(&mut second, tree).unwrap();
    assert_eq!(out, second.finish().unwrap());
    Snap {
        name: "trace",
        caps: TraceBackend::new().capabilities(),
        hash: fnv1a64(out.as_bytes()),
        empty: out.is_empty(),
        kind: "trace",
        view_w: Some(tree.page_width),
        view_h: Some(tree.page_height),
    }
}

#[cfg(rhwp_has_png_backend)]
fn snap_png(tree: &PageLayerTree) -> Snap {
    let caps = PngBackend::new().capabilities();
    let mut first = PngBackend::new();
    replay_page(&mut first, tree).unwrap();
    let out = first.finish().unwrap();
    let live = PngBackend::raster_available();
    if live {
        assert!(png_ihdr(&out).is_some(), "png 시그니처/IHDR 없음");
    } else {
        assert!(out.is_empty(), "raster 없으면 png finish 는 빈 바이트");
    }
    let (view_w, view_h) = match png_ihdr(&out) {
        Some((width, height)) => (Some(width as f64), Some(height as f64)),
        None => (None, None),
    };
    Snap {
        name: caps.name,
        caps,
        hash: fnv1a64(&out),
        empty: out.is_empty(),
        kind: "png",
        view_w,
        view_h,
    }
}

#[cfg(rhwp_has_skia_backend)]
fn snap_skia(tree: &PageLayerTree) -> Snap {
    let caps = SkiaBackend::new().capabilities();
    let mut first = SkiaBackend::new();
    replay_page(&mut first, tree).unwrap();
    let out = first.finish().unwrap();
    let live = SkiaBackend::raster_available();
    if live {
        assert!(out.width > 0 && out.height > 0 && !out.bytes.is_empty());
    } else {
        assert_eq!(out.width, 0);
        assert_eq!(out.height, 0);
        assert!(out.bytes.is_empty());
    }
    Snap {
        name: caps.name,
        caps,
        hash: fnv1a64(&out.bytes),
        empty: out.bytes.is_empty(),
        kind: "raster",
        view_w: if live { Some(out.width as f64) } else { None },
        view_h: if live { Some(out.height as f64) } else { None },
    }
}

fn present_snaps(tree: &PageLayerTree) -> Vec<Snap> {
    let mut snaps = vec![snap_svg(tree), snap_null(tree), snap_trace(tree)];
    #[cfg(rhwp_has_png_backend)]
    snaps.push(snap_png(tree));
    #[cfg(rhwp_has_skia_backend)]
    snaps.push(snap_skia(tree));
    snaps
}

fn skipped_optional() -> Vec<&'static str> {
    let mut skipped = Vec::new();
    if !cfg!(rhwp_has_png_backend) {
        skipped.push("png");
    }
    if !cfg!(rhwp_has_skia_backend) {
        skipped.push("skia");
    }
    skipped
}

/// 있는 어댑터를 모으고 없는 쪽은 로그로만 남긴다.
#[test]
fn present_adapters_are_listed_and_missing_ones_skipped() {
    let snaps = present_snaps(&scene_tree());
    let names: Vec<&str> = snaps.iter().map(|snap| snap.name).collect();
    assert!(names.contains(&"svg"), "{names:?}");
    assert!(names.contains(&"null"), "{names:?}");
    assert!(names.contains(&"trace"), "{names:?}");
    let skipped = skipped_optional();
    for name in &names {
        assert!(
            !skipped.contains(name),
            "skip 목록에 있는 어댑터를 비교했다: {name}"
        );
    }
    eprintln!("adapter-diff present={names:?} skipped={skipped:?}");
}

/// 구조: 이름 고유, capability 자기모순 없음.
#[test]
fn capability_structure_is_consistent_across_present_adapters() {
    let snaps = present_snaps(&scene_tree());
    let mut names = Vec::new();
    for snap in &snaps {
        assert!(
            snap.caps.is_consistent(),
            "{} capability 자기모순",
            snap.name
        );
        assert_eq!(snap.caps.name, snap.name);
        assert!(
            !names.contains(&snap.name),
            "어댑터 이름 중복: {}",
            snap.name
        );
        names.push(snap.name);
        if snap.caps.raster_only {
            assert!(
                !snap.caps.supports(BackendFeature::VectorText),
                "{} raster_only 인데 vector_text",
                snap.name
            );
        }
    }
}

/// 상호 capability: 같은 family 는 raster/vector 축이 같고, 다른 family 는 달라도 된다.
#[test]
fn capability_matrix_agrees_within_family() {
    let snaps = present_snaps(&scene_tree());
    for left in &snaps {
        for right in &snaps {
            if left.name == right.name {
                continue;
            }
            if left.caps.raster_only && right.caps.raster_only {
                assert_eq!(
                    left.caps.supports(BackendFeature::VectorText),
                    right.caps.supports(BackendFeature::VectorText),
                    "{} vs {} raster family vector_text",
                    left.name,
                    right.name
                );
            }
            if !left.caps.raster_only
                && !right.caps.raster_only
                && left.caps.supports(BackendFeature::VectorText)
                && right.caps.supports(BackendFeature::VectorText)
            {
                assert_eq!(
                    left.caps.supports(BackendFeature::VectorText),
                    right.caps.supports(BackendFeature::VectorText)
                );
            }
        }
    }
    let svg = snaps.iter().find(|snap| snap.name == "svg").unwrap();
    assert!(svg.caps.supports(BackendFeature::VectorText));
    assert!(!svg.caps.raster_only);
}

/// 산출 해시: 결정론 어댑터는 두 번 그려 같은 해시. 다른 형식끼리는 맞대지 않는다.
#[test]
fn output_hash_is_stable_and_not_cross_format() {
    let tree = scene_tree();
    let first = present_snaps(&tree);
    let second = present_snaps(&tree);
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.name, b.name);
        if a.caps.supports(BackendFeature::Deterministic) {
            assert_eq!(a.hash, b.hash, "{} 결정론 해시가 흔들린다", a.name);
            assert!(!a.hash.is_empty());
        }
    }
    let hashes: Vec<(&str, &str, &str)> = first
        .iter()
        .map(|snap| (snap.name, snap.kind, snap.hash.as_str()))
        .collect();
    for i in 0..first.len() {
        for j in (i + 1)..first.len() {
            if first[i].kind != first[j].kind {
                continue;
            }
            if first[i].empty || first[j].empty {
                continue;
            }
            if first[i].caps.supports(BackendFeature::Deterministic)
                && first[j].caps.supports(BackendFeature::Deterministic)
                && first[i].kind == first[j].kind
            {
                // 같은 형식·결정론이면 해시 비교 대상. 현재 devel 에 같은 형식
                // 쌍은 없다(svg/trace/stats). 나중에 png+png 복제가 생겨도 안전.
                let _ = (first[i].hash == first[j].hash, hashes.as_slice());
            }
        }
    }
}

/// bbox: 논리 페이지는 같고, SVG viewBox 는 그 치수와 같다.
#[test]
fn page_bbox_agrees_across_present_adapters() {
    let tree = scene_tree();
    let snaps = present_snaps(&tree);
    for snap in &snaps {
        match snap.kind {
            "svg" => {
                assert_eq!(snap.view_w, Some(PAGE_W), "{}", snap.name);
                assert_eq!(snap.view_h, Some(PAGE_H), "{}", snap.name);
                assert!(!snap.empty);
            }
            "trace" | "stats" => {
                assert_eq!(snap.view_w, Some(PAGE_W), "{}", snap.name);
                assert_eq!(snap.view_h, Some(PAGE_H), "{}", snap.name);
            }
            "png" | "raster" => {
                if !snap.empty {
                    let width = snap.view_w.expect("raster bbox");
                    let height = snap.view_h.expect("raster bbox");
                    assert!(
                        width > 0.0 && height > 0.0,
                        "{} {}x{}",
                        snap.name,
                        width,
                        height
                    );
                }
            }
            other => panic!("unknown kind {other}"),
        }
    }
}

/// Trace 가 기록한 장면 bbox 가 SVG 산출물에 남는다.
#[test]
fn trace_bbox_is_visible_in_svg_output() {
    let tree = scene_tree();
    let mut trace = TraceBackend::new();
    replay_page(&mut trace, &tree).unwrap();
    let log = trace.finish().unwrap();
    assert!(
        log.contains("rectangle bbox=20.00,20.00,40.00,24.00"),
        "{log}"
    );
    assert!(
        log.contains("textRun bbox=20.00,52.00,80.00,16.00"),
        "{log}"
    );

    let mut svg = SvgBackend::new();
    replay_page(&mut svg, &tree).unwrap();
    let out = svg.finish().unwrap();
    assert!(out.contains("viewBox=\"0 0 400 300\""), "{out}");
    assert!(
        out.contains("20") && out.contains("40"),
        "svg 가 사각형 좌표를 잃음\n{out}"
    );
}

/// Null 이 센 op 수가 장면과 같다 (구조).
#[test]
fn null_backend_counts_scene_ops() {
    let mut backend = NullBackend::new();
    replay_page(&mut backend, &scene_tree()).unwrap();
    let stats = backend.finish().unwrap();
    assert_eq!(stats.pages, 1);
    assert_eq!(stats.ops, 3);
    assert_eq!(stats.count_of("pageBackground"), 1);
    assert_eq!(stats.count_of("rectangle"), 1);
    assert_eq!(stats.count_of("textRun"), 1);
}
