//! 관계형: 밀도와 imgDim이 없는 BMP의 전체 픽셀 영역과 저장 crop 창을 Print paint까지 보존한다.
//! 주석 갱신: 2026-11-19T16:55:38+0900 (Asia/Seoul).

//! 2026-11-19, `d158a24298`, 한컴 2022 Print, 96 DPI, compat2022, raw 2px 실루엣.
//! `1480000-200900112-hci-tulip-table.hwp` 전 16쪽 최저 Native 99.57% / fresh WASM 99.57%.
//! 검사 대상 7쪽 Native 99.92% / fresh WASM 99.92%. 기대값의 단위는 원본 픽셀과 HWPUNIT이다.

use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::ImageNode;
use rhwp::DocumentCore;

fn collect_graphs<'a>(layer: &'a LayerNode, graphs: &mut Vec<&'a ImageNode>) {
    match &layer.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_graphs(child, graphs);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect_graphs(child, graphs);
        }
        LayerNodeKind::Leaf { ops } => {
            graphs.extend(ops.iter().filter_map(|op| match op {
                PaintOp::Image { image, .. } if image.bin_data_id == 3 => Some(image.as_ref()),
                _ => None,
            }));
        }
    }
}

#[test]
fn print_paint_keeps_the_bmp_pixel_extent_and_stored_crop() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue677/1480000-200900112-hci-tulip-table.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(source).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let page = core
        .build_page_layer_tree_with_profile(6, RenderProfile::Print)
        .expect("7쪽 Print paint");
    let mut graphs = Vec::new();
    collect_graphs(&page.root, &mut graphs);
    let [graph] = graphs.as_slice() else {
        panic!(
            "7쪽에 원본 BMP 그래프가 하나 있어야 함. 실제 {}개",
            graphs.len()
        );
    };

    // BIN0003.bmp의 헤더는 758×527 픽셀, 가로·세로 밀도 0을 선언한다.
    // 그림 제어에 저장된 자르기 창은 전체 bitmap보다 작다.
    assert_eq!(graph.original_size_hu, Some((758 * 75, 527 * 75)));
    assert_eq!(graph.crop, Some((0, 5669, 54612, 39480)));
}
