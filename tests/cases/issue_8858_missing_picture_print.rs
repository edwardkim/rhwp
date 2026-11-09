//! [#8858] 관계형 회귀 — 데이터 없는 그림(그림 미지정) 이미지 노드는 `#2225` 의 대체 노드와 같은
//! 계약을 따른다: 편집 프로필은 대체 표시, 인쇄 등가 프로필·SVG 내보내기는 생략.
//!
//! 주석 갱신: 2026-11-09 23:57 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8858/missing_picture_print.hwpx` — 공개 결재문서 36481989(HWPX, 2쪽).
//!   1쪽 그림 하나가 BinData 없이 참조만 남은 「그림 미지정」이고 외부 경로도 없다. 이 그림은
//!   `#2225` 의 대체 노드가 아니라 데이터 없는 이미지 노드로 렌더 트리에 온다.
//! - 독립 기준: `pdf/issue8858/missing_picture_print-2020.pdf`(한/글 2022 12.0.0.4547 Print, 2쪽) —
//!   그림 자리는 비어 있다.
//! - 수정 전 rhwp(devel `b768bee1c4`)는 인쇄 프로필에서도 그 자리에 회색 대체 상자를 그렸다
//!   (인쇄 레이어 이미지 수 = 편집 레이어 이미지 수).
//!
//! 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `b768bee1c4` + 이 수정.
//! 1쪽 Native 100.00 / fresh WASM 99.99(수정 전 Native 81.97), 2쪽 99.84 / 99.83. 전체 2쪽 = PDF
//! 2쪽, 전쪽 최저 Native 99.84(2쪽)·fresh WASM 99.83(2쪽).
//! 증거: `mydocs/pr/assets/issue_8858/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사: 원본 계약(1쪽 그림 미지정 이미지 노드 존재), 쪽수 2 = 한/글(고정형), 편집 레이어 이미지 수
//! − 인쇄 레이어 이미지 수 = 그림 미지정 수, 편집 SVG 에는 대체 사각형·인쇄 SVG·레거시 SVG 에는
//! 없음(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8858/missing_picture_print.hwpx";

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn missing_images(node: &RenderNode, out: &mut usize) {
    // 그림 컨트롤(채우기 아님)·외부 경로 없음·원본 데이터 없음.
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.fill_mode.is_none()
            && image.external_path.is_none()
            && image.data.as_deref().is_none_or(<[u8]>::is_empty)
        {
            *out += 1;
        }
    }
    for child in &node.children {
        missing_images(child, out);
    }
}

fn image_ops(json: &str) -> usize {
    json.matches("\"type\":\"image\"").count()
}

#[test]
fn missing_picture_image_is_omitted_from_print_output() {
    let core = load();
    assert_eq!(core.page_count(), 2, "전체 쪽수 = 한/글 2쪽");

    // 원본 계약: 1쪽에 데이터 없는 그림(BinData 없음·외부 경로 없음)이 이미지 노드로 온다.
    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut missing = 0;
    missing_images(&tree.root, &mut missing);
    assert!(missing > 0, "1쪽의 그림 미지정 이미지 노드");

    // 편집 프로필은 대체 표시를 남기고, 인쇄 프로필은 그만큼 생략한다.
    let screen = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Screen)
        .expect("Screen 레이어")
        .to_json();
    let print = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .expect("Print 레이어")
        .to_json();
    assert_eq!(
        image_ops(&screen) - image_ops(&print),
        missing,
        "인쇄 레이어는 그림 미지정 이미지 {missing}개를 생략한다"
    );

    // SVG 도 같다 — 편집 프로필은 빈 데이터 대체 사각형을 그리고, 인쇄 프로필과 대체 표시를 끈
    // 레거시 SVG 경로는 그리지 않는다.
    let placeholder = "fill=\"#f0f0f0\" stroke=\"#999999\" stroke-dasharray=\"4\"";
    let screen_svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Screen)
        .expect("Screen SVG");
    assert!(screen_svg.contains(placeholder), "편집 SVG 의 대체 사각형");
    let print_svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .expect("Print SVG");
    assert!(
        !print_svg.contains(placeholder),
        "인쇄 SVG 에 대체 사각형이 없다"
    );
    let legacy_svg = core.render_page_svg_legacy_native(0).expect("레거시 SVG");
    assert!(
        !legacy_svg.contains(placeholder),
        "대체 표시를 끈 레거시 SVG 에 대체 사각형이 없다"
    );
}
