//! 관계형·고정형: 그림의 독립 Print 색·마스크, 원본 소유와 물리 경계를 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 255쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 255쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 원본 255쪽은 Native b2b / fresh DEV WASM 88d에서 raw 100%·미대응 0이다.
//! 원본/PDF SHA·96dpi·확대 판독·표본 경로는
//! `mydocs/plans/issue_8153_print_alpha_mask_20261026.md`에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    paint::{ClipKind, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImagePayload},
    renderer::render_tree::{BoundingBox, ImageNode},
    DocumentCore,
};

struct Figure<'a> {
    bounds: &'a BoundingBox,
    source: &'a ImageNode,
    payload: &'a ResolvedImagePayload,
    body_clip: bool,
}

fn collect<'a>(node: &'a LayerNode, body_clip: bool, found: &mut Vec<Figure<'a>>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect(child, body_clip, found);
            }
        }
        LayerNodeKind::ClipRect {
            clip_kind, child, ..
        } => {
            collect(child, body_clip || *clip_kind == ClipKind::Body, found);
        }
        LayerNodeKind::ClipPath { child, .. } => collect(child, body_clip, found),
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image {
                    bbox,
                    image,
                    resolved,
                } = op
                {
                    if image.bin_data_id == 409 {
                        found.push(Figure {
                            bounds: bbox,
                            source: image,
                            payload: resolved.as_deref().expect("그림의 독립 Print 전달 자원"),
                            body_clip,
                        });
                    }
                }
            }
        }
    }
}

fn verify(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let original = core.export_hwp_native().unwrap();
    let tree = core
        .build_page_layer_tree_with_profile(254, RenderProfile::Print)
        .unwrap();
    let mut figures = Vec::new();
    collect(&tree.root, false, &mut figures);
    assert_eq!(figures.len(), 1, "원본 인계·인수 그림의 누락·중복 금지");
    let figure = &figures[0];
    assert_eq!(
        (figure.source.section_index, figure.source.para_index),
        (Some(9), Some(38))
    );
    assert!(figure.body_clip, "그림의 상위 본문 clip 소유 보존");
    let source = image::load_from_memory(figure.source.data.as_deref().unwrap())
        .unwrap()
        .into_rgba8();
    let actual = image::load_from_memory(&figure.payload.data)
        .unwrap()
        .into_rgba8();
    assert_eq!(source.dimensions(), (768, 401));
    assert_eq!(actual.dimensions(), source.dimensions());
    assert!(
        source.pixels().any(|pixel| pixel[3] > 0 && pixel[3] < 255),
        "실제 반투명 원본"
    );
    assert!(
        actual
            .pixels()
            .all(|pixel| pixel[3] == 0 || pixel[3] == 255),
        "독립 Print의 1비트 마스크를 연속 alpha나 흰 배경 합성으로 바꾸지 않는다"
    );
    // 독립 PDF 255쪽의 JPEG/1비트 마스크에서 읽은 원본 자원 좌표다.
    // alpha 0 아래 JPEG 색도 별도 평면 표본에서 필요하므로 검정으로 버리지 않는다.
    for (x, y, expected) in [
        (50, 50, [0, 97, 168, 255]),
        (400, 200, [198, 198, 198, 255]),
        (550, 350, [42, 75, 106, 255]),
        (765, 4, [194, 194, 194, 0]),
        (760, 400, [254, 254, 254, 0]),
    ] {
        assert_eq!(
            actual.get_pixel(x, y).0,
            expected,
            "독립 색·마스크 표본 ({x},{y})"
        );
    }
    let frame = figure.bounds;
    // 동일 PDF 자원의 표시 사각형을 72pt→96dpi로 환산했다. 코드 출력에서 복사하지 않는다.
    for (actual, expected) in [
        (frame.x, 170.3759969075521),
        (frame.y, 658.4959716796875),
        (frame.x + frame.width, 626.6826985677084),
        (frame.y + frame.height, 865.0359700520834),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print 그림 경계: {actual} vs {expected}"
        );
    }
    assert_eq!(
        original,
        core.export_hwp_native().unwrap(),
        "출력 변환은 원본을 수정하지 않는다"
    );
}

#[test]
fn hwp_print_alpha_preserves_hidden_color_source_owner_and_bounds() {
    verify("hwp");
}

#[test]
fn hwpx_print_alpha_preserves_hidden_color_source_owner_and_bounds() {
    verify("hwpx");
}
