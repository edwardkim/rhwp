//! Native raster 의 장치 한 칸보다 가는 획은 한 칸 굵기로 그린다.
//!
//! 한컴 Print PDF 를 raster 로 읽으면 0.1 mm(약 0.38 px) 표 괘선도 장치 한 칸을 채운다.
//! Native Skia 가 선언 굵기 그대로 anti-alias 하면 괘선의 덮임이 38% 안팎에 그쳐 거의 보이지
//! 않는다. `samples/basic/calendar_year.hwp`(한 쪽, 기준 `pdf/basic/calendar_year-2022.pdf`)는
//! 이 차이로 Native 실루엣이 88.25 였고, 한 칸 굵기로 그리면 98.87(fresh WASM 93.80)이다.
//!
//! 조판 좌표·선언 굵기는 그대로이고 raster 획 굵기만 바뀌므로, render tree 의 실제 괘선 위치에서
//! 그 괘선을 가로지르는 화소 중 가장 진한 화소가 선 색 대비의 절반 이상인지 본다.

#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use rhwp::renderer::render_tree::{LineNode, RenderNode, RenderNodeType};

fn thin_horizontal_lines(node: &RenderNode, out: &mut Vec<LineNode>) {
    if let RenderNodeType::Line(line) = &node.node_type {
        let horizontal = (line.y1 - line.y2).abs() < 0.01 && (line.x2 - line.x1).abs() > 20.0;
        if horizontal && line.style.width > 0.0 && line.style.width < 1.0 {
            out.push(line.clone());
        }
    }
    for child in &node.children {
        thin_horizontal_lines(child, out);
    }
}

#[test]
fn subpixel_table_rules_cover_one_device_pixel() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/calendar_year.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
    let tree = core.build_page_render_tree(0).expect("page");
    let mut lines = Vec::new();
    thin_horizontal_lines(&tree.root, &mut lines);
    assert!(lines.len() >= 10, "가는 가로 괘선 — {}개", lines.len());

    let width = tree.root.bbox.width.round() as u32;
    let pixels = core
        .render_page_pixels_native_at_width(0, width, RenderProfile::Print, &[])
        .expect("raster");
    let scale = f64::from(pixels.width) / tree.root.bbox.width;
    let gray = |x: i32, y: i32| -> f64 {
        let at = y as usize * pixels.stride + x as usize * 4;
        let p = &pixels.bytes[at..at + 3];
        (f64::from(p[0]) + f64::from(p[1]) + f64::from(p[2])) / 3.0
    };

    let mut faint = Vec::new();
    for line in &lines {
        let color = line.style.color;
        let ink = (f64::from((color & 0xff) as u8)
            + f64::from(((color >> 8) & 0xff) as u8)
            + f64::from(((color >> 16) & 0xff) as u8))
            / 3.0;
        let contrast = 255.0 - ink;
        if contrast < 64.0 {
            continue;
        }
        // 괘선 양 끝(교차 괘선)을 피해 가운데 세 지점을 본다.
        for t in [0.3, 0.5, 0.7] {
            let x = ((line.x1 + (line.x2 - line.x1) * t) * scale) as i32;
            let cy = line.y1 * scale;
            let darkest = ((cy - 2.0) as i32..=(cy + 2.0) as i32)
                .filter(|y| *y >= 0 && *y < pixels.height)
                .map(|y| 255.0 - gray(x, y))
                .fold(0.0, f64::max);
            if darkest < contrast * 0.5 {
                faint.push((line.y1, line.style.width, darkest / contrast));
            }
        }
    }
    assert!(
        faint.is_empty(),
        "장치 한 칸을 덮지 못한 괘선 {}곳 (y, 굵기, 최대 덮임) — {:?}",
        faint.len(),
        &faint[..faint.len().min(6)]
    );
}
