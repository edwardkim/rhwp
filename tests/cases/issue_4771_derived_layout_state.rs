//! [#4771] 원본 IR과 renderer-only 조판 상태의 persistence 경계.

use rhwp::model::bin_data::BinDataContent;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::image::Picture;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo};
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::serializer::serialize_hwpx;
use rhwp::DocumentCore;
use std::path::Path;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xff, 0xff, 0x3f,
    0x00, 0x05, 0xfe, 0x02, 0xfe, 0xdc, 0xcc, 0x59, 0xe7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
    0x44, 0xae, 0x42, 0x60, 0x82,
];

#[test]
fn hwpx_does_not_persist_layout_only_fill_lines() {
    let mut paragraph = Paragraph::new_empty();
    paragraph.line_segs.push(LineSeg {
        text_start: 1,
        vertical_pos: 1_600,
        line_height: 1_000,
        text_height: 1_000,
        baseline_distance: 850,
        line_spacing: 600,
        segment_width: 42_520,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
        ..Default::default()
    });
    paragraph.layout_only_fill_lines = 1;

    let mut document = Document::default();
    document.sections.push(Section {
        paragraphs: vec![paragraph],
        ..Default::default()
    });

    let bytes = serialize_hwpx(&document).expect("HWPX 직렬화");
    let reparsed = parse_hwpx(&bytes).expect("HWPX 재파싱");
    let persisted = &reparsed.sections[0].paragraphs[0].line_segs;

    assert_eq!(
        persisted.len(),
        1,
        "renderer-only suffix가 HWPX 파일 데이터가 되면 안 된다: {persisted:?}"
    );
    assert_eq!(persisted[0].text_start, 0);
}

fn floating_picture(
    bin_data_id: u16,
    horizontal_offset: u32,
    vertical_offset: u32,
    horz_rel_to: HorzRelTo,
    width: u32,
    height: u32,
    horz_align: HorzAlign,
    vert_align: VertAlign,
) -> Control {
    let mut picture = Picture::default();
    picture.common = CommonObjAttr {
        horizontal_offset,
        vertical_offset,
        width,
        height,
        horz_rel_to,
        horz_align,
        vert_rel_to: VertRelTo::Para,
        vert_align,
        text_wrap: TextWrap::Square,
        allow_overlap: false,
        treat_as_char: false,
        ..Default::default()
    };
    picture.image_attr.bin_data_id = bin_data_id;
    Control::Picture(Box::new(picture))
}

fn floating_pair_core(second_x: u32, second_y: u32, second_frame: HorzRelTo) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    let mut document = Document::default();
    document.sections.push(Section {
        paragraphs: vec![Paragraph {
            char_count: 1,
            line_segs: vec![LineSeg {
                line_height: 1_000,
                text_height: 1_000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: 42_520,
                ..Default::default()
            }],
            controls: vec![
                floating_picture(
                    1,
                    0,
                    0,
                    HorzRelTo::Page,
                    10_000,
                    50_000,
                    HorzAlign::Left,
                    VertAlign::Top,
                ),
                floating_picture(
                    2,
                    second_x,
                    second_y,
                    second_frame,
                    10_000,
                    50_000,
                    HorzAlign::Left,
                    VertAlign::Top,
                ),
            ],
            ..Default::default()
        }],
        ..Default::default()
    });
    document.bin_data_content = vec![
        BinDataContent {
            id: 1,
            data: PNG_1X1.to_vec().into(),
            extension: "png".into(),
        },
        BinDataContent {
            id: 2,
            data: PNG_1X1.to_vec().into(),
            extension: "png".into(),
        },
    ];
    core.set_document(document);
    core
}

fn centered_unequal_pair_core() -> DocumentCore {
    let mut core = floating_pair_core(0, 0, HorzRelTo::Page);
    let mut document = core.document().clone();
    document.sections[0].paragraphs[0].controls = vec![
        floating_picture(
            1,
            0,
            0,
            HorzRelTo::Page,
            10_000,
            50_000,
            HorzAlign::Center,
            VertAlign::Top,
        ),
        floating_picture(
            2,
            6_000,
            0,
            HorzRelTo::Page,
            1_000,
            50_000,
            HorzAlign::Center,
            VertAlign::Top,
        ),
    ];
    core.set_document(document);
    core
}

fn collect_images<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect_images(child, out);
    }
}

fn rendered_pair(
    core: DocumentCore,
    reason: &str,
) -> Vec<(u16, rhwp::renderer::render_tree::BoundingBox)> {
    assert_eq!(core.page_count(), 1, "{reason}: projection이 쪽을 늘렸다");
    let page = core.build_page_render_tree(0).expect("page render tree");
    let mut images = Vec::new();
    collect_images(&page.root, &mut images);
    assert_eq!(
        images.len(),
        2,
        "{reason}: 두 source 그림 identity를 유지해야 한다"
    );
    images
        .iter()
        .map(|node| match &node.node_type {
            RenderNodeType::Image(image) => (image.bin_data_id, node.bbox),
            _ => unreachable!(),
        })
        .collect()
}

#[test]
fn floating_stack_admission_requires_same_frame_and_strict_2d_overlap() {
    let horizontal = rendered_pair(
        floating_pair_core(20_000, 0, HorzRelTo::Page),
        "가로 bounds가 분리된 그림",
    );
    assert!(
        horizontal[1].1.x >= horizontal[0].1.x + horizontal[0].1.width,
        "가로로 분리된 source bounds를 세로 projection으로 바꾸면 안 된다: {horizontal:?}"
    );
    assert!((horizontal[1].1.y - horizontal[0].1.y).abs() < 0.01);

    let centered = rendered_pair(
        centered_unequal_pair_core(),
        "가운데 정렬에서 폭이 다른 가로 bounds가 분리된 그림",
    );
    assert!(
        centered[1].1.x >= centered[0].1.x + centered[0].1.width,
        "alignment-relative bounds를 offset-only로 비교하면 안 된다: {centered:?}"
    );

    let frames = rendered_pair(
        floating_pair_core(0, 0, HorzRelTo::Column),
        "서로 다른 가로 reference frame",
    );
    assert!(
        (frames[1].1.y - frames[0].1.y).abs() < 0.01,
        "서로 다른 frame을 한 stack으로 합치면 안 된다: {frames:?}"
    );

    let boundary = rendered_pair(
        floating_pair_core(0, 50_000, HorzRelTo::Page),
        "세로 offset spread가 그림 높이와 같은 경계값",
    );
    let expected_offset_px = 50_000.0 / 75.0;
    assert!(
        ((boundary[1].1.y - boundary[0].1.y) - expected_offset_px).abs() < 0.01,
        "offset 경계값은 source 좌표를 유지해야 한다: {boundary:?}"
    );
}

#[test]
fn issue_2004_projection_preserves_each_picture_identity_and_final_bounds() {
    let expected = [
        (3, (82.4, 124.9, 601.5, 868.1)),
        (4, (82.4, 84.1, 579.3, 840.8)),
        (5, (82.4, 84.1, 592.1, 867.4)),
        (6, (82.4, 84.1, 580.1, 868.0)),
        (7, (82.4, 84.1, 604.9, 862.6)),
    ];

    for relative in [
        "samples/issue2004_cell_image_stack.hwp",
        "samples/issue2004_cell_image_stack.hwpx",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("fixture read"))
            .expect("fixture parse");
        assert_eq!(core.page_count(), 8, "{relative}: #2004 page count");

        let first_picture_id = if relative.ends_with(".hwpx") { 6 } else { 3 };
        for (page_index, (x, y, width, height)) in expected {
            let page = core
                .build_page_render_tree(page_index)
                .unwrap_or_else(|error| panic!("{relative} page {}: {error}", page_index + 1));
            let mut images = Vec::new();
            collect_images(&page.root, &mut images);
            assert_eq!(
                images.len(),
                1,
                "{relative} page {} must contain exactly one stack picture",
                page_index + 1
            );
            let RenderNodeType::Image(image) = &images[0].node_type else {
                unreachable!()
            };
            assert_eq!(
                image.bin_data_id,
                first_picture_id + (page_index - 3) as u16,
                "{relative} page {} picture identity",
                page_index + 1
            );
            let actual = images[0].bbox;
            for (name, actual, expected) in [
                ("x", actual.x, x),
                ("y", actual.y, y),
                ("width", actual.width, width),
                ("height", actual.height, height),
            ] {
                assert!(
                    (actual - expected).abs() <= 0.1,
                    "{relative} page {} {name}: expected {expected:.1}, got {actual:.3}",
                    page_index + 1
                );
            }
        }
    }
}

#[test]
fn renderer_cache_lifecycle_is_absent_from_source_models() {
    let paragraph_source = include_str!("../../src/model/paragraph.rs");
    let table_source = include_str!("../../src/model/table.rs");

    assert!(
        !paragraph_source.contains("pub single_line_overflow_memo"),
        "renderer memo must be owned by a renderer session cache"
    );
    assert!(
        !table_source.contains("pub dirty: bool"),
        "measurement validity must be owned by DocumentCore revisions"
    );
    assert!(
        !table_source.contains("text_reflowed_after_edit"),
        "pagination provenance must be owned by render normalization"
    );
    assert!(
        !table_source.contains("local_resize_"),
        "editor layout projection must not be stored on source Table"
    );
}
