//! 정상 한컴 저장본의 독립 Print로 확인한 닫힌 복합선의 소유·띠·빈 간격을 보호한다.
//! 전 1쪽 Native/fresh WASM 모두 100%이며 복합선의 띠·간격을 직접 판독했다.
//! 단선 래스터 농도는 미해결이므로 절대 좌표를 고정하지 않는다. 증거는 stage1900 기록.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, shape::ShapeObject};
use rhwp::renderer::render_tree::{PageRenderTree, RectangleNode, RenderNode, RenderNodeType};
use rhwp::renderer::{PathCommand, StrokeDash};
use rhwp::DocumentCore;

type Owner = (usize, usize, usize);

struct Source {
    owner: Owner,
    kind: u32,
    width: i32,
    color: u32,
}

fn load() -> (DocumentCore, Vec<Source>) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/shapes/hancom-2020-compound-rectangles.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    assert_eq!(core.page_count(), 1, "한컴 Print 전체 쪽수와 도형 보존");
    let mut sources = Vec::new();
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, paragraph) in section.paragraphs.iter().enumerate() {
            for (ci, control) in paragraph.controls.iter().enumerate() {
                if let Control::Shape(shape) = control {
                    if let ShapeObject::Rectangle(rect) = shape.as_ref() {
                        sources.push(Source {
                            owner: (si, pi, ci),
                            kind: rect.drawing.border_line.attr & 0x3f,
                            width: rect.drawing.border_line.width,
                            color: rect.drawing.border_line.color,
                        });
                    }
                }
            }
        }
    }
    assert_eq!(sources.len(), 26, "정상 저장·인쇄한 대조군의 원본 개체 수");
    (core, sources)
}

fn rectangles<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a RectangleNode)>) {
    if let RenderNodeType::Rectangle(rect) = &node.node_type {
        out.push((node, rect));
    }
    for child in &node.children {
        rectangles(child, out);
    }
}

fn placement(tree: &PageRenderTree, owner: Owner) -> (&RenderNode, &RectangleNode) {
    let mut nodes = Vec::new();
    rectangles(&tree.root, &mut nodes);
    let matches: Vec<_> = nodes
        .into_iter()
        .filter(|(_, r)| {
            (r.section_index, r.para_index, r.control_index)
                == (Some(owner.0), Some(owner.1), Some(owner.2))
        })
        .collect();
    assert_eq!(matches.len(), 1, "원본 소유 개체의 누락·중복: {owner:?}");
    matches[0]
}

// 곡선의 끝점 다각형도 안팎 윤곽의 진행 방향을 구별한다.
fn contour_areas(commands: &[PathCommand]) -> Vec<f64> {
    let mut areas = Vec::new();
    let mut points = Vec::new();
    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y)
            | PathCommand::LineTo(x, y)
            | PathCommand::CurveTo(_, _, _, _, x, y)
            | PathCommand::ArcTo(_, _, _, _, _, x, y) => points.push((x, y)),
            PathCommand::ClosePath => {
                assert!(points.len() >= 3, "닫힌 띠의 유효한 외곽");
                let area = points
                    .iter()
                    .zip(points.iter().cycle().skip(1))
                    .take(points.len())
                    .map(|(a, b)| a.0 * b.1 - b.0 * a.1)
                    .sum::<f64>()
                    / 2.0;
                areas.push(area);
                points.clear();
            }
        }
    }
    assert!(points.is_empty(), "열린 윤곽 없이 닫힌 띠로 종료");
    areas
}

#[test]
fn verified_closed_compound_borders_keep_owner_and_opposite_winding_holes() {
    let (core, sources) = load();
    let tree = core.build_page_render_tree(0).unwrap();
    for source in sources.iter().filter(|s| (8..=11).contains(&s.kind)) {
        let (node, rect) = placement(&tree, source.owner);
        assert!(rect.style.stroke_color.is_none(), "단선 stroke 중복 방지");
        let paths: Vec<_> = node
            .children
            .iter()
            .filter_map(|n| {
                if let RenderNodeType::Path(p) = &n.node_type {
                    Some((n, p))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(paths.len(), 1, "모든 띠·빈 간격이 하나의 경로에 소속");
        let (child, path) = paths[0];
        assert_eq!(
            (path.section_index, path.para_index, path.control_index),
            (rect.section_index, rect.para_index, rect.control_index)
        );
        assert_eq!(
            (
                child.bbox.x,
                child.bbox.y,
                child.bbox.width,
                child.bbox.height
            ),
            (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height)
        );
        assert!(path.style.stroke_color.is_none() && path.style.fill_color.is_some());
        assert_eq!(
            path.style.fill_color,
            Some(source.color),
            "원본 선 색상 보존"
        );
        assert_eq!(path.style.opacity, rect.style.opacity, "원본 투명도 보존");
        let areas = contour_areas(&path.commands);
        assert_eq!(areas.len(), if source.kind == 11 { 6 } else { 4 });
        for ring in areas.chunks_exact(2) {
            assert!(ring[0] * ring[1] < 0.0, "빈 간격은 반대 winding으로 뚫음");
            assert!(
                ring[0].abs() > ring[1].abs(),
                "안쪽 윤곽은 바깥 윤곽에 포함"
            );
        }
    }
}

#[test]
fn verified_closed_compound_controls_preserve_single_dash_and_absent_strokes() {
    let (core, sources) = load();
    let tree = core.build_page_render_tree(0).unwrap();
    let controls: Vec<_> = sources.iter().filter(|s| s.kind < 8).collect();
    assert_eq!(controls.len(), 5, "단선 세 폭·선 없음·파선 대조군");
    for source in controls {
        let (node, rect) = placement(&tree, source.owner);
        assert!(!node
            .children
            .iter()
            .any(|n| matches!(n.node_type, RenderNodeType::Path(_))));
        assert_eq!(rect.style.stroke_color.is_some(), source.kind != 0);
        if source.kind == 2 {
            assert_eq!(rect.style.stroke_dash, StrokeDash::HwpDash);
        }
    }
}

#[cfg(feature = "native-skia")]
fn raster(tree: &PageRenderTree) -> rhwp::renderer::raster_pixels::RasterPixels {
    use rhwp::paint::{LayerBuilder, RenderProfile};
    use rhwp::renderer::{layer_renderer::RasterRenderOptions, skia::SkiaLayerRenderer};
    let layers = LayerBuilder::new(RenderProfile::Print).build(tree);
    SkiaLayerRenderer::new()
        .render_pixels_with_options(
            &layers,
            RasterRenderOptions {
                transparent: false,
                ..RasterRenderOptions::default()
            },
        )
        .unwrap()
}

#[cfg(feature = "native-skia")]
fn scan(
    pixels: &rhwp::renderer::raster_pixels::RasterPixels,
    node: &RenderNode,
    width: i32,
) -> Vec<[u8; 3]> {
    let declared = f64::from(width) / 75.0;
    let left = (node.bbox.x - declared * 0.6).floor().max(0.0) as usize;
    let right = (node.bbox.x + node.bbox.width / 2.0).floor() as usize;
    let y = (node.bbox.y + node.bbox.height / 2.0).floor() as usize;
    assert!(right < pixels.width as usize && y < pixels.height as usize);
    (left..right)
        .map(|x| {
            let i = y * pixels.stride + x * 4;
            [pixels.bytes[i], pixels.bytes[i + 1], pixels.bytes[i + 2]]
        })
        .collect()
}

#[cfg(feature = "native-skia")]
fn ink_runs(row: &[[u8; 3]]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start = None;
    for (x, pixel) in row.iter().enumerate() {
        let ink = pixel.iter().all(|c| *c < 128);
        match (start, ink) {
            (None, true) => start = Some(x),
            (Some(left), false) => {
                runs.push((left, x));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(left) = start {
        runs.push((left, row.len()));
    }
    runs
}

#[cfg(feature = "native-skia")]
#[test]
fn verified_closed_compound_print_ink_keeps_band_count_order_and_empty_gaps() {
    let (core, sources) = load();
    let tree = core.build_page_render_tree(0).unwrap();
    let pixels = raster(&tree);
    let sources: Vec<_> = sources
        .iter()
        .filter(|s| (8..=11).contains(&s.kind) && s.width == 1600)
        .collect();
    assert_eq!(sources.len(), 4);
    for source in sources {
        let (node, _) = placement(&tree, source.owner);
        let row = scan(&pixels, node, source.width);
        let runs = ink_runs(&row);
        assert_eq!(
            runs.len(),
            if source.kind == 11 { 3 } else { 2 },
            "최종 출력의 분리된 선"
        );
        assert!(
            runs.windows(2).all(|p| p[0].1 < p[1].0),
            "선 사이의 빈 간격"
        );
        let widths: Vec<_> = runs.iter().map(|(a, b)| (b - a) as f64).collect();
        match source.kind {
            8 => assert!((widths[0] - widths[1]).abs() <= widths[0].max(widths[1]) * 0.25),
            9 => assert!(widths[1] > widths[0] * 1.5),
            10 => assert!(widths[0] > widths[1] * 1.5),
            11 => assert!(widths[1] > widths[0] * 2.0 && widths[1] > widths[2] * 2.0),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn verified_closed_compound_gap_exposes_original_blue_fill() {
    let (core, sources) = load();
    let tree = core.build_page_render_tree(0).unwrap();
    let pixels = raster(&tree);
    let blue: Vec<_> = sources
        .iter()
        .filter(|s| s.kind == 10)
        .filter_map(|s| {
            let (node, rect) = placement(&tree, s.owner);
            (rect.style.fill_color == Some(0x00ffcc99)).then_some((node, s))
        })
        .collect();
    assert_eq!(blue.len(), 1, "독립 Print의 파랑 채우기 굵은/가는 이중선");
    let (node, source) = blue[0];
    let row = scan(&pixels, node, source.width);
    let runs = ink_runs(&row);
    assert_eq!(runs.len(), 2, "채우기 위에서도 두 선은 분리됨");
    assert!(
        row[runs[0].1..runs[1].0]
            .iter()
            .any(|p| p[2] > p[1] && p[1] > p[0]),
        "간격을 흰색으로 덮지 않고 원래 파랑 채우기를 드러냄"
    );
}
