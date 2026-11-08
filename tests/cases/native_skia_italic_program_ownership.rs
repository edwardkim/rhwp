//! 공개 Native paint에서 요청 기울임과 실제 face 기울임의 소유를 구별한다.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

fn render_face(face: &str, italic: bool) -> image::RgbaImage {
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "rhwp-italic-program-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fonts")
        .join(format!("RHWPHostFixture-{face}.ttf"));
    std::fs::copy(source, directory.join("only-face.ttf")).unwrap();
    let bounds = BoundingBox::new(30.0, 0.0, 100.0, 100.0);
    let run = TextRunNode {
        text: "A".into(),
        style: TextStyle {
            font_family: "RHWP Host Fixture".into(),
            font_size: 64.0,
            italic,
            ..Default::default()
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
        baseline: 78.0,
        field_marker: Default::default(),
        layout_positions: Some(vec![0.0, 64.0]),
        display_text: None,
    };
    let tree = PageLayerTree::new(
        160.0,
        110.0,
        LayerNode::leaf(bounds, None, vec![PaintOp::text_run(bounds, run)]),
    );
    let output = SkiaLayerRenderer::new()
        .with_font_paths(std::slice::from_ref(&directory))
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .expect("공개 Native PNG");
    std::fs::remove_dir_all(directory).unwrap();
    image::load_from_memory(&output.bytes).unwrap().to_rgba8()
}

fn upper_minus_lower_ink_x(image: &image::RgbaImage) -> f64 {
    let ink: Vec<_> = image
        .enumerate_pixels()
        .filter(|(_, _, pixel)| pixel[3] > 128)
        .map(|(x, y, _)| (x, y))
        .collect();
    let first_y = ink.iter().map(|(_, y)| *y).min().expect("가시 글리프");
    let last_y = ink.iter().map(|(_, y)| *y).max().unwrap();
    let middle_y = (first_y + last_y) / 2;
    let mean = |upper| {
        let xs: Vec<_> = ink
            .iter()
            .filter(|(_, y)| (*y <= middle_y) == upper)
            .map(|(x, _)| f64::from(*x))
            .collect();
        xs.iter().sum::<f64>() / xs.len() as f64
    };
    mean(true) - mean(false)
}

#[test]
fn upright_only_program_honors_the_requested_italic_in_actual_png() {
    let regular = render_face("Regular", false);
    let italic = render_face("Regular", true);
    assert!(
        upper_minus_lower_ink_x(&italic) > upper_minus_lower_ink_x(&regular),
        "동일한 직립 face·원점·advance에서 기울임은 위쪽 잉크를 오른쪽으로 옮긴다"
    );
}

#[test]
fn actual_italic_or_oblique_program_is_not_sheared_twice() {
    for face in ["Italic", "Oblique"] {
        assert_eq!(
            render_face(face, false),
            render_face(face, true),
            "이미 기울어진 {face} 프로그램에 요청 기울임을 다시 합성하지 않는다"
        );
    }
}
