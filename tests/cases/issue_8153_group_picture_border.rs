//! #8153: 묶음 자식 그림의 테두리는 이미지와 같은 확정 프레임을 소유한다.
//! 편람 독립 한컴 Print 94쪽의 image372·373 검은 외곽선이 독립 근거다.
//! 94쪽은 그림/외곽선의 관계·순서·crop 보존을, 163쪽은 독립 Print ±2px도 검사한다.
//! source 9f868694ef, Print/compat2024, 96dpi, 좌상단 CSS px.
//! 시각 증거·입력/PDF 해시는 mydocs/plans/issue_8153_handbook_raw100_20261022.md를 따른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::shape::{GroupShape, ShapeObject};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const OWNERS: [usize; 2] = [657, 661];

fn source_group(core: &DocumentCore, section: usize, para: usize) -> &GroupShape {
    match &core.document().sections[section].paragraphs[para].controls[0] {
        Control::Shape(shape) => match shape.as_ref() {
            ShapeObject::Group(group) => group,
            _ => panic!("원본 묶음 개체"),
        },
        _ => panic!("원본 문단 소유 묶음"),
    }
}

fn collect_groups<'a>(
    node: &'a RenderNode,
    section: usize,
    para: usize,
    found: &mut Vec<&'a RenderNode>,
) {
    if matches!(&node.node_type, RenderNodeType::Group(group)
        if group.section_index == Some(section) && group.para_index == Some(para)
            && group.control_index == Some(0))
    {
        found.push(node);
    }
    for child in &node.children {
        collect_groups(child, section, para, found);
    }
}

fn same_frame(actual: BoundingBox, image: BoundingBox) {
    for (actual, expected) in [
        (actual.x, image.x),
        (actual.y, image.y),
        (actual.width, image.width),
        (actual.height, image.height),
    ] {
        assert!(
            (actual - expected).abs() < 1e-8,
            "이미지와 외곽선의 공통 프레임"
        );
    }
}

fn open(ext: &str) -> DocumentCore {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{ext}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    core
}

fn verify_original(ext: &str) {
    let core = open(ext);
    let page = core.build_page_render_tree(93).expect("독립 Print 94쪽");
    let mut frames = Vec::new();
    for para in OWNERS {
        let source = source_group(&core, 2, para);
        let ShapeObject::Picture(picture) = &source.children[0] else {
            panic!("원본 묶음의 첫 자식 그림");
        };
        assert_eq!(picture.border_attr.attr & 0x3f, 1, "원본 실선 외곽선");
        assert_eq!(picture.border_color, 0, "독립 Print의 검은 외곽선");
        let mut groups = Vec::new();
        collect_groups(&page.root, 2, para, &mut groups);
        assert_eq!(groups.len(), 1, "94쪽의 원본 소유 묶음 누락·중복 금지");
        let group = groups[0];
        let images: Vec<_> = group
            .children
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                if let RenderNodeType::Image(image) = &node.node_type {
                    Some((i, node, image))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(images.len(), 1, "묶음당 원본 그림 한 개 보존");
        let (index, node, image) = images[0];
        assert_eq!(image.bin_data_id, picture.image_attr.bin_data_id);
        assert_eq!(image.crop, picture.render_crop_rect(), "원본 crop 보존");
        assert_eq!(image.original_size_hu, picture.crop_reference_size());
        let border = group.children.get(index + 1).expect("그림 바로 뒤 테두리");
        let RenderNodeType::Rectangle(rectangle) = &border.node_type else {
            panic!("그림 뒤의 외곽 사각형");
        };
        assert!(
            rectangle.section_index.is_none()
                && rectangle.para_index.is_none()
                && rectangle.control_index.is_none(),
            "외곽선은 독립 컨트롤이 아니며 원본 묶음의 선택 소유를 유지한다"
        );
        assert_eq!(
            rectangle.style.fill_color, None,
            "이미지 내부를 채우지 않는다"
        );
        same_frame(border.bbox, node.bbox);
        assert_eq!(rectangle.style.stroke_color, Some(picture.border_color));
        assert!(
            rectangle.style.stroke_width > 0.0,
            "저장 width=0도 Print에는 선이 있다"
        );
        assert_eq!(rectangle.transform.rotation, image.transform.rotation);
        assert_eq!(rectangle.transform.horz_flip, image.transform.horz_flip);
        assert_eq!(rectangle.transform.vert_flip, image.transform.vert_flip);
        assert_eq!(
            group.children.len(),
            source.children.len() + 1,
            "원래 주석 개체를 유지하고 테두리를 한 번만 추가"
        );
        frames.push(node.bbox);
    }
    assert!(
        frames[0].y + frames[0].height < frames[1].y,
        "위·아래 설명 그림의 순서·비겹침 보존"
    );
}

#[test]
fn hwpx_group_picture_border_shares_resolved_image_frame() {
    verify_original("hwpx");
}

#[test]
fn hwp_group_picture_border_shares_resolved_image_frame() {
    verify_original("hwp");
}

// 검은 외곽선 묶음 그림을 보호할 수 있는 완료 페이지 중 양쪽 raw 최솟값이 높은
// 163쪽을 선택한다(양쪽·두 형식 최저 99.35380%). 248쪽의 흰색 가림띠는 다른 동작이다.
// 입력/PDF SHA·TSV·직접 판독 근거는 issue_8153_handbook_raw100_20261022 계획에 연결한다.
// 독립 Print 두 형식의 163쪽 /Image(669x946) bbox, 72pt -> 96dpi CSS px.
const PRINT_163_IMAGE_EDGES: [f64; 4] = [206.656006, 312.929281, 621.408000, 886.633301];

fn verify_completed_frame(ext: &str) {
    let core = open(ext);
    let source = source_group(&core, 4, 67);
    let ShapeObject::Picture(picture) = &source.children[0] else {
        panic!("독립 Print의 신청서 그림을 소유한 첫 자식");
    };
    assert_eq!(picture.border_attr.attr & 0x3f, 1);
    assert_eq!(picture.border_color, 0);
    let page = core.build_page_render_tree(162).expect("독립 Print 163쪽");
    let mut groups = Vec::new();
    collect_groups(&page.root, 4, 67, &mut groups);
    let owners: Vec<_> = groups
        .into_iter()
        .filter(|group| {
            group.children.iter().any(|child| {
                matches!(&child.node_type,
            RenderNodeType::Image(image) if image.bin_data_id == picture.image_attr.bin_data_id)
            })
        })
        .collect();
    assert_eq!(owners.len(), 1, "원본 소유와 그림 식별자로 묶음을 찾는다");
    let group = owners[0];
    let pictures: Vec<_> = group
        .children
        .iter()
        .enumerate()
        .filter(|(_, child)| {
            matches!(&child.node_type, RenderNodeType::Image(image)
            if image.bin_data_id == picture.image_attr.bin_data_id)
        })
        .collect();
    assert_eq!(pictures.len(), 1, "163쪽 신청서 그림의 누락·중복 금지");
    let (index, image) = pictures[0];
    let border = group.children.get(index + 1).expect("그림의 외곽선");
    let RenderNodeType::Rectangle(rectangle) = &border.node_type else {
        panic!("신청서 그림 외곽 사각형");
    };
    assert_eq!(rectangle.style.stroke_color, Some(picture.border_color));
    same_frame(border.bbox, image.bbox);
    let actual = [
        image.bbox.x,
        image.bbox.y,
        image.bbox.x + image.bbox.width,
        image.bbox.y + image.bbox.height,
    ];
    for (actual, expected) in actual.into_iter().zip(PRINT_163_IMAGE_EDGES) {
        assert!(
            (actual - expected).abs() <= 2.0,
            "163쪽 원본 소유 그림: 실제 {actual}, 독립 Print {expected}, 허용 ±2px"
        );
    }
}

#[test]
fn hwpx_completed_group_picture_keeps_independent_print_frame_within_2px() {
    verify_completed_frame("hwpx");
}

#[test]
fn hwp_completed_group_picture_keeps_independent_print_frame_within_2px() {
    verify_completed_frame("hwp");
}
