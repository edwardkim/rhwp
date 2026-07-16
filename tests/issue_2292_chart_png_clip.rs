//! Issue #2292: 차트(RawSvg) PNG 내보내기 전 유형 잘림 — 조각 좌표계 계약 불일치.
//!
//! 원인: skia `rasterize_svg_fragment_to_png` 가 조각을 로컬 좌표로 가정해
//! `viewBox="0 0 w h"` 로 감쌌으나, 차트 RawSvg 조각은 **페이지 절대 좌표**
//! 로 방출된다 (SVG 백엔드·web_canvas 는 페이지 좌표 계약 — skia 만 이탈).
//! viewBox 창 밖 콘텐츠가 전부 클리핑되어 차트의 좌상단 조각만 보였다.
//!
//! 수정: wrapper viewBox 를 조각의 페이지 좌표 창(`"x y w h"`)으로 —
//! 호출부(PaintOp::RawSvg)가 bbox 원점을 보유한다.
//!
//! 검증: 차트 bbox 의 좌측 1/3(축 라벨)·상단 1/3(제목/플롯 상단)에 잉크가
//! 존재해야 한다. 수정 전에는 viewBox(0,0) 클리핑 + bbox 재배치 이중
//! 오프셋으로 잉크가 우하단 스트립에만 몰려 둘 다 공백(FAILED 실증).
#![cfg(feature = "native-skia")]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLES: &[&str] = &[
    // 기존 유형 (C1d 라인) — 잘림이 stock 특이가 아님을 가드
    "samples/chart/라인/표식이있는꺽은선형.hwp",
    // 신규 stock (#2288)
    "samples/chart/기타/고가저가종가.hwp",
];

/// 페이지 0 렌더 트리에서 첫 RawSvg(차트) bbox 를 찾는다.
fn find_chart_bbox(node: &RenderNode) -> Option<(f64, f64, f64, f64)> {
    if let RenderNodeType::RawSvg(_) = &node.node_type {
        return Some((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    node.children.iter().find_map(find_chart_bbox)
}

/// 영역 내 잉크(비흰색·불투명) 픽셀 수.
fn ink_in_region(img: &image::RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
    let mut ink = 0;
    for y in y0..y1.min(img.height()) {
        for x in x0..x1.min(img.width()) {
            let p = img.get_pixel(x, y);
            if p.0[3] > 8 && (p.0[0] < 230 || p.0[1] < 230 || p.0[2] < 230) {
                ink += 1;
            }
        }
    }
    ink
}

#[test]
fn chart_png_renders_full_bbox_not_top_left_fragment() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    for sample in SAMPLES {
        let bytes = fs::read(repo.join(sample)).unwrap_or_else(|e| panic!("read {sample}: {e}"));
        let core =
            DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));

        let tree = core
            .build_page_render_tree(0)
            .unwrap_or_else(|e| panic!("{sample}: tree: {e:?}"));
        let (bx, by, bw, bh) =
            find_chart_bbox(&tree.root).unwrap_or_else(|| panic!("{sample}: RawSvg(차트) 부재"));

        let png = core
            .render_page_png_native(0)
            .unwrap_or_else(|e| panic!("{sample}: png: {e:?}"));
        let img = image::load_from_memory(&png)
            .unwrap_or_else(|e| panic!("{sample}: decode: {e}"))
            .to_rgba8();

        // PNG 픽셀 좌표 = 페이지 px × (이미지 폭 / 페이지 폭) 배율
        let page_w = tree.root.bbox.width;
        let scale = img.width() as f64 / page_w;
        let px = |v: f64| (v * scale).round() as u32;

        // 수정 전 실측(#2292): viewBox(0,0) 클리핑 + bbox 재배치의 이중
        // 오프셋으로 잉크가 우하단 스트립(x 291..537, y 322..382)에만 존재
        // — bbox 좌측 1/3(축 라벨 영역)과 상단 1/3(제목/플롯 상단)이 공백.
        let left_third = ink_in_region(&img, px(bx), px(by), px(bx + bw / 3.0), px(by + bh));
        let top_third = ink_in_region(&img, px(bx), px(by), px(bx + bw), px(by + bh / 3.0));
        let total = ink_in_region(&img, px(bx), px(by), px(bx + bw), px(by + bh));

        assert!(total > 0, "{sample}: 차트 bbox 잉크 0 — 차트 미렌더");
        assert!(
            left_third > 0,
            "{sample}: 차트 bbox 좌측 1/3 잉크 0 (total={total}) — #2292 이중 오프셋 잘림"
        );
        assert!(
            top_third > 0,
            "{sample}: 차트 bbox 상단 1/3 잉크 0 (total={total}) — #2292 이중 오프셋 잘림"
        );
    }
}
