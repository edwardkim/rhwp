//! Native 글꼴 공급의 스타일 계약. 한컴 문서 배치나 픽셀 golden이 아니다.
//! 같은 OFL family의 독립적인 400/700 프로그램으로 같은 디렉터리의 파일 순서에 무관한
//! 실제 glyph 출력을 비교한다. 정확한 좌표나 현재 구현의 점수는 기대값으로 쓰지 않는다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;
use std::path::{Path, PathBuf};

struct FontDir(PathBuf);
impl FontDir {
    fn new(label: &str, bytes: &[u8]) -> Self {
        Self::with_faces(label, &[("face.ttf", bytes)])
    }
    fn with_faces(label: &str, faces: &[(&str, &[u8])]) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("rhwp-style-{}-{stamp}-{label}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        for (name, bytes) in faces {
            std::fs::write(dir.join(name), bytes).unwrap();
        }
        Self(dir)
    }
}
impl Drop for FontDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn tree(bold: bool, profile: RenderProfile) -> PageLayerTree {
    let bbox = BoundingBox::new(10.0, 10.0, 200.0, 40.0);
    let run = TextRunNode {
        text: "AW 총투자".into(),
        style: TextStyle {
            font_family: "RHWP Supplied Style Contract".into(),
            font_size: 24.0,
            bold,
            shade_color: 0x00FF_FFFF,
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
        baseline: 28.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    };
    PageLayerTree::with_profile(
        240.0,
        80.0,
        LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]),
        profile,
    )
}
fn pixels(tree: &PageLayerTree, paths: &[&Path]) -> image::RgbaImage {
    let dirs: Vec<_> = paths.iter().map(|p| p.to_path_buf()).collect();
    let png = SkiaLayerRenderer::new()
        .with_font_paths(&dirs)
        .render_png(tree)
        .unwrap();
    image::load_from_memory(&png).unwrap().to_rgba8()
}
#[test]
fn supplied_family_matches_style_within_the_first_directory() {
    const REGULAR: &[u8] =
        include_bytes!("../fixtures/native_font_styles/RHWPSuppliedStyle-Regular.ttf");
    const BOLD: &[u8] = include_bytes!("../fixtures/native_font_styles/RHWPSuppliedStyle-Bold.ttf");
    let regular = FontDir::new("regular", REGULAR);
    let heavy = FontDir::new("bold", BOLD);
    let mixed = [
        FontDir::with_faces("bold-first", &[("a.ttf", BOLD), ("b.ttf", REGULAR)]),
        FontDir::with_faces("regular-first", &[("a.ttf", REGULAR), ("b.ttf", BOLD)]),
    ];
    assert_eq!(
        ttf_parser::Face::parse(REGULAR, 0)
            .unwrap()
            .weight()
            .to_number(),
        400
    );
    assert_eq!(
        ttf_parser::Face::parse(BOLD, 0)
            .unwrap()
            .weight()
            .to_number(),
        700
    );
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        for bold in [false, true] {
            let tree = tree(bold, profile);
            let reference = pixels(&tree, &[if bold { &heavy.0 } else { &regular.0 }]);
            let other = pixels(&tree, &[if bold { &regular.0 } else { &heavy.0 }]);
            assert!(reference != other, "독립 프로그램의 윤곽이 실제로 다르다");
            for folder in &mixed {
                assert!(pixels(&tree, &[&folder.0]) == reference,
                    "같은 디렉터리의 400/700 공급은 파일 순서와 관계없이 요청한 프로그램을 선택한다");
            }
            for (first, second) in [(&regular.0, &heavy.0), (&heavy.0, &regular.0)] {
                assert!(
                    pixels(&tree, &[first, second]) == pixels(&tree, &[first]),
                    "별도 경로에서는 먼저 지정한 family 공급의 우선순위를 보존한다"
                );
            }
        }
    }
}
