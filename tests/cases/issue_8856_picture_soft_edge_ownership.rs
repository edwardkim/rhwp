//! 검사 유형: 관계형.
//! 주석 갱신: 2026-11-10T04:30:37+09:00.
//! 원본 효과 반경의 모델·저장 소유 검사이며 절대 화면 좌표를 고정하지 않는다.
//! 기존 원본 시각 근거: mydocs/pr/assets/issue8856/stage4-policy-fields/nine-groups-final-comparison.tsv.
//! interview 전 2쪽 Native 최저 97.19457%, fresh WASM 최저 97.32378%.
//! 측정: 2026-11-10, 96dpi·PDFium·색상 보정 전 2px raw, 최소 글꼴 환경.
//! 이 수치는 효과 보정 전 결과다. 효과 출력의 최종 시각 검증은 아직 완료하지 않았다.
//! 독립 한컴 2020 HWPX는 원본 첫 그림의 softEdge radius=1000을 보존한다.
//! HWP5 표 108·111의 플래그 4·반경 f32와 대조하며 원본 자르기·이미지 바이트는 유지한다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn radii(core: &DocumentCore) -> Vec<f64> {
    fn visit(value: &Value, out: &mut Vec<f64>) {
        match value {
            Value::Object(fields) => {
                if fields.contains_key("raw_picture_extra") {
                    if let Some(radius) = fields["effects"]["soft_edge_radius"].as_f64() {
                        out.push(radius);
                    }
                }
                for child in fields.values() {
                    visit(child, out);
                }
            }
            Value::Array(children) => {
                for child in children {
                    visit(child, out);
                }
            }
            _ => {}
        }
    }
    let mut result = Vec::new();
    for section in &core.document().sections {
        for paragraph in &section.paragraphs {
            visit(
                &serde_json::to_value(&paragraph.controls).unwrap(),
                &mut result,
            );
        }
    }
    result
}

#[test]
fn picture_radius_survives_render_and_layer_tree_transport() {
    use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn image_radius(image: &rhwp::renderer::render_tree::ImageNode, out: &mut Vec<f64>) {
        let value = serde_json::to_value(image).unwrap();
        if let Some(radius) = value["soft_edge_radius_hu"].as_f64() {
            out.push(radius);
        }
    }
    fn render_radii(node: &RenderNode, out: &mut Vec<f64>) {
        if let RenderNodeType::Image(image) = &node.node_type {
            image_radius(image, out);
        }
        for child in &node.children {
            render_radii(child, out);
        }
    }
    fn layer_radii(node: &LayerNode, out: &mut Vec<f64>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    layer_radii(child, out);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                layer_radii(child, out);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::Image { image, .. } = op {
                        image_radius(image, out);
                    }
                }
            }
        }
    }
    for path in [
        "samples/basic/interview.hwp",
        "samples/issue8593/picture_soft_edge.hwp",
    ] {
        let mut core = load(path);
        let expected = radii(&core);
        assert!(!expected.is_empty(), "반경을 소유한 실제 그림 대조군");
        let mut render = Vec::new();
        let mut layer = Vec::new();
        for page in 0..core.page_count() {
            render_radii(
                &core.build_page_render_tree(page).unwrap().root,
                &mut render,
            );
            layer_radii(
                &core
                    .build_page_layer_tree_with_profile(page, RenderProfile::Print)
                    .unwrap()
                    .root,
                &mut layer,
            );
        }
        assert_eq!(
            render, expected,
            "{path}: 조판 트리가 원본 반경을 버리지 않는다"
        );
        assert_eq!(
            layer, expected,
            "{path}: 공통 paint가 원본 반경을 버리지 않는다"
        );
    }
}

#[test]
fn source_soft_edge_radius_is_owned_by_the_picture_and_survives_hwpx() {
    for (path, expected) in [
        ("samples/basic/interview.hwp", vec![1000.0]),
        (
            "samples/issue8593/picture_soft_edge.hwp",
            vec![300.0, 500.0],
        ),
    ] {
        let core = load(path);
        assert_eq!(
            radii(&core),
            expected,
            "{path}: 소스 효과 반경을 버리지 않는다"
        );
        let saved = core.export_hwpx_native().unwrap();
        let reopened = DocumentCore::from_bytes(&saved).unwrap();
        assert_eq!(
            radii(&reopened),
            expected,
            "{path}: HWPX 효과 저장·읽기 보존"
        );
        let hwp = reopened.export_hwp_native().unwrap();
        assert_eq!(
            radii(&DocumentCore::from_bytes(&hwp).unwrap()),
            expected,
            "{path}: HWPX에서 HWP5로 저장할 때도 반경 보존"
        );
        assert_eq!(
            reopened.page_count(),
            core.page_count(),
            "효과 저장은 쪽 소유를 바꾸지 않는다"
        );
    }
}

#[test]
fn picture_without_effect_does_not_gain_a_soft_edge() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn check(node: &RenderNode) -> usize {
        let own = if let RenderNodeType::Image(image) = &node.node_type {
            assert!(
                serde_json::to_value(image)
                    .unwrap()
                    .get("soft_edge_radius_hu")
                    .is_none(),
                "무효과 원본의 이미지 JSON에 null 효과 필드를 추가하지 않는다"
            );
            1
        } else {
            0
        };
        own + node.children.iter().map(check).sum::<usize>()
    }
    let core = load("samples/ta-pic-001-r.hwp");
    assert!(radii(&core).is_empty());
    let images = (0..core.page_count())
        .map(|page| check(&core.build_page_render_tree(page).unwrap().root))
        .sum::<usize>();
    assert!(images > 0, "무효과 실물 그림도 실제로 검사한다");
    let saved = core.export_hwpx_native().unwrap();
    assert!(radii(&DocumentCore::from_bytes(&saved).unwrap()).is_empty());
}

#[test]
fn editing_the_known_effect_keeps_alpha_and_can_remove_the_effect() {
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    type CropFrame = (Option<(u32, u32)>, Option<(i32, i32, i32, i32)>);
    fn crop_frames(core: &DocumentCore) -> Vec<CropFrame> {
        fn collect(node: &RenderNode, result: &mut Vec<CropFrame>) {
            if let RenderNodeType::Image(image) = &node.node_type {
                result.push((image.original_size_hu, image.crop));
            }
            for child in &node.children {
                collect(child, result);
            }
        }
        let mut result = Vec::new();
        for page in 0..core.page_count() {
            collect(
                &core.build_page_render_tree(page).unwrap().root,
                &mut result,
            );
        }
        result
    }
    for (radius, transparency) in [(Some(2000.0), 50), (Some(0.0), 100), (None, 0)] {
        let mut core = load("samples/basic/interview.hwp");
        let source_frames = crop_frames(&core);
        let mut doc = core.document().clone();
        let picture = doc.sections[0]
            .paragraphs
            .iter_mut()
            .flat_map(|p| &mut p.controls)
            .find_map(|control| match control {
                Control::Picture(p) => Some(p),
                _ => None,
            })
            .expect("원본 첫 그림");
        picture.effects.soft_edge_radius = radius;
        picture.image_attr.transparency = transparency;
        core.set_document(doc);
        let saved = core.export_hwp_native().unwrap();
        let reopened = DocumentCore::from_bytes(&saved).unwrap();
        assert_eq!(
            crop_frames(&reopened),
            source_frames,
            "효과·투명도 편집이 누락된 imgDim을 crop 끝값으로 추정해 자르기 기준을 바꾸지 않는다"
        );
        assert_eq!(radii(&reopened), radius.into_iter().collect::<Vec<_>>());
        let picture = reopened.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|control| match control {
                Control::Picture(p) => Some(p),
                _ => None,
            })
            .unwrap();
        assert!(
            (i16::from(picture.image_attr.transparency) - i16::from(transparency)).abs() <= 1,
            "효과 꼬리의 alpha를 반경이나 크기로 오독하지 않는다"
        );
    }
}
