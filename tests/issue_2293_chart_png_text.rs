//! Issue #2293: skia RawSvg 조각 래스터의 텍스트 소실 — fontdb 폰트 해석 축.
//!
//! 원인: `svg_fontdb()` 가 ①프로젝트 `ttfs/` 미로딩 ②generic 폴백을 존재
//! 확인 없이 특정 배포판 폰트명에 하드 고정 — 매칭 실패 시 resvg 가
//! 텍스트를 드롭했다 (조각에는 `<text>` 12개 정상 포함, #2292 검증 중 발견).
//!
//! 수정: PDF 경로(`renderer/pdf.rs::create_fontdb`)와 동일 규약으로
//! `ttfs/`(재귀)·WSL 윈도우 폰트 로딩 + generic 폴백을 존재 확인 체인으로.
//!
//! 검증: 차트 PNG 의 제목·축 라벨 영역에 잉크가 존재해야 한다 (수정 전
//! FAILED). 환경별 폰트 차이를 허용하기 위해 잉크 존재만 단언하고 특정
//! 글리프 픽셀 비교는 하지 않는다. #2292 표적(기하)과 축 분리.
#![cfg(feature = "native-skia")]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/chart/라인/표식이있는꺽은선형.hwp";

fn find_chart_bbox(node: &RenderNode) -> Option<(f64, f64, f64, f64)> {
    if let RenderNodeType::RawSvg(_) = &node.node_type {
        return Some((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    node.children.iter().find_map(find_chart_bbox)
}

fn ink_in_region(img: &image::RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
    let mut ink = 0;
    for y in y0..y1.min(img.height()) {
        for x in x0..x1.min(img.width()) {
            let p = img.get_pixel(x, y);
            if p.0[3] > 8 && (p.0[0] < 200 || p.0[1] < 200 || p.0[2] < 200) {
                ink += 1;
            }
        }
    }
    ink
}

/// 차트 텍스트(제목 상단 밴드 + 카테고리 라벨 하단 밴드)가 래스터에 그려진다.
#[test]
fn chart_png_renders_text_labels() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(repo.join(SAMPLE)).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");

    let tree = core.build_page_render_tree(0).expect("tree");
    let (bx, by, bw, bh) = find_chart_bbox(&tree.root).expect("RawSvg(차트) 부재");

    let png = core.render_page_png_native(0).expect("png");
    let img = image::load_from_memory(&png).expect("decode").to_rgba8();
    let scale = img.width() as f64 / tree.root.bbox.width;
    let px = |v: f64| (v * scale).round() as u32;

    // 제목 밴드: bbox 상단 12% (조각 실측 — "차트 제목" y≈150/bbox y=132..382).
    // 기하(플롯 프레임 상단선)가 걸치지 않도록 좌우 25% 인셋의 중앙부만 본다.
    let title = ink_in_region(
        &img,
        px(bx + bw * 0.25),
        px(by + bh * 0.02),
        px(bx + bw * 0.75),
        px(by + bh * 0.12),
    );

    // 카테고리 라벨 밴드: 플롯 하단과 bbox 사이 (조각 실측 — "항목 N"
    // y≈374/bbox 하단 382). 하단 6% 밴드.
    let category = ink_in_region(
        &img,
        px(bx + bw * 0.1),
        px(by + bh * 0.94),
        px(bx + bw * 0.9),
        px(by + bh),
    );

    assert!(
        title > 0,
        "차트 제목 밴드 잉크 0 — 조각 래스터 텍스트 소실(#2293)"
    );
    assert!(
        category > 0,
        "카테고리 라벨 밴드 잉크 0 (title={title}) — 조각 래스터 텍스트 소실(#2293)"
    );
}
