//! #6812: 어울림 그림의 물리 영역에서 TAC 표의 줄 위치를 계산해야 한다.
//! 파일의 저장 vpos를 답으로 쓰지 않고, 공개 API가 만든 두 사각형을 비교한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;
use rhwp::model::shape::{HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo};
use rhwp::renderer::page_layout::PageLayoutInfo;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn sample() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6797/156160455-social-pig-farm-income.hwp");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("필수 원본 {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("원본 로드")
}

fn change_picture(core: &mut DocumentCore, change: impl FnOnce(&mut Picture)) {
    let mut doc = core.document().clone();
    let Control::Picture(picture) = &mut doc.sections[0].paragraphs[0].controls[3] else {
        panic!("원본 그림 컨트롤");
    };
    change(picture);
    core.set_document(doc);
}

fn collect(node: &RenderNode, pictures: &mut Vec<BoundingBox>, tables: &mut Vec<BoundingBox>) {
    match &node.node_type {
        RenderNodeType::Image(image)
            if image.para_index == Some(0) && image.control_index == Some(3) =>
        {
            pictures.push(node.bbox);
        }
        RenderNodeType::Table(table)
            if table.para_index == Some(0)
                && table.control_index == Some(4)
                && table.cell_context.is_none() =>
        {
            tables.push(node.bbox);
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, pictures, tables);
    }
}

fn boxes(core: &DocumentCore) -> (BoundingBox, BoundingBox) {
    let tree = core.build_page_render_tree(0).expect("첫 쪽 렌더 트리");
    let (mut pictures, mut tables) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut pictures, &mut tables);
    assert_eq!(pictures.len(), 1, "그림 누락/중복으로 통과하면 안 된다");
    assert_eq!(tables.len(), 1, "표 누락/중복으로 통과하면 안 된다");
    (pictures[0], tables[0])
}

fn assert_below_picture(core: &DocumentCore) {
    let (picture, table) = boxes(core);
    let horizontal_overlap =
        ((picture.x + picture.width).min(table.x + table.width) - picture.x.max(table.x)).max(0.0);
    let vertical_overlap = ((picture.y + picture.height).min(table.y + table.height)
        - picture.y.max(table.y))
    .max(0.0);
    assert!(
        horizontal_overlap > 0.5,
        "이 입력은 가로로 함께 놓을 수 없다"
    );
    assert!(
        vertical_overlap <= 0.5,
        "그림/표 교집합: 가로 {horizontal_overlap:.3}, 세로 {vertical_overlap:.3}; \
         그림 {picture:?}, 표 {table:?}"
    );
    // 교집합과 여백은 다른 축이다. 여백을 빼서 겹침 검사만 통과시키지 않는다.
    let Control::Table(source) = &core.document().sections[0].paragraphs[0].controls[4] else {
        panic!("원본 표 컨트롤");
    };
    let required_top = picture.y + picture.height + f64::from(source.outer_margin_top) / 75.0;
    assert!(
        table.y + 0.5 >= required_top,
        "바깥 위 여백: {table:?}, 필요 {required_top}"
    );
}

#[test]
fn issue_6812_original_paper_picture_precedes_tac_table_without_intersection() {
    let core = sample();
    let Control::Picture(picture) = &core.document().sections[0].paragraphs[0].controls[3] else {
        panic!("원본 그림");
    };
    assert!(!picture.common.treat_as_char);
    assert_eq!(picture.common.text_wrap, TextWrap::Square);
    assert_eq!(picture.common.horz_rel_to, HorzRelTo::Paper);
    assert_eq!(picture.common.vert_rel_to, VertRelTo::Paper);
    assert!(
        picture.common.allow_overlap,
        "原本 bit 14를 꺼서 회피 조건을 맞추지 않는다"
    );
    eprintln!(
        "#6812 원본 overlap={}, attr={:#x}, flow={:?}",
        picture.common.allow_overlap, picture.common.attr, picture.common.text_flow
    );
    assert_below_picture(&core);
}

/// 실제 paint의 Paper/Page 기준·정렬·여백은 공통 좌표 계산 분리 후에도 유지한다.
/// 기대값은 공개 PageDef/ColumnDef의 물리 영역에서 직접 구하고 내부 helper는 호출하지 않는다.
#[test]
fn issue_6812_reference_frame_geometry_preserves_alignment_and_outer_margins() {
    let mut core = sample();
    let page = &core.document().sections[0].section_def.page_def;
    let layout = PageLayoutInfo::from_page_def(page, &Default::default(), 96.0);
    for paper in [true, false] {
        for horizontal in [HorzAlign::Left, HorzAlign::Center, HorzAlign::Right] {
            for vertical in [VertAlign::Top, VertAlign::Center, VertAlign::Bottom] {
                change_picture(&mut core, |picture| {
                    picture.common.horz_rel_to = if paper {
                        HorzRelTo::Paper
                    } else {
                        HorzRelTo::Page
                    };
                    picture.common.vert_rel_to = if paper {
                        VertRelTo::Paper
                    } else {
                        VertRelTo::Page
                    };
                    picture.common.horz_align = horizontal;
                    picture.common.vert_align = vertical;
                    picture.common.horizontal_offset = 750;
                    picture.common.vertical_offset = 1500;
                    picture.common.margin.left = 75;
                    picture.common.margin.right = 150;
                    picture.common.margin.top = 225;
                    picture.common.margin.bottom = 300;
                });
                let (image, _) = boxes(&core);
                let (x, y, w, h) = if paper {
                    (0.0, 0.0, layout.page_width, layout.page_height)
                } else {
                    let body = &layout.body_area;
                    (body.x, body.y, body.width, body.height)
                };
                let box_width = image.width + 3.0;
                let box_height = image.height + 7.0;
                let expected_x = match horizontal {
                    HorzAlign::Left => x + 10.0,
                    HorzAlign::Center => x + (w - box_width) / 2.0 + 10.0,
                    HorzAlign::Right => x + w - box_width - 10.0,
                    _ => unreachable!(),
                } + 1.0;
                let expected_y = match vertical {
                    VertAlign::Top => y + 20.0,
                    VertAlign::Center => y + (h - box_height) / 2.0 + 20.0,
                    VertAlign::Bottom => y + h - box_height - 20.0,
                    _ => unreachable!(),
                } + 3.0;
                assert!(
                    (image.x - expected_x).abs() < 0.01,
                    "paper={paper}, {horizontal:?}: {image:?}, expected_x={expected_x}"
                );
                assert!(
                    (image.y - expected_y).abs() < 0.01,
                    "paper={paper}, {vertical:?}: {image:?}, expected_y={expected_y}"
                );
            }
        }
    }
}

#[test]
fn issue_6812_picture_height_change_recomputes_clearance_without_saved_vpos() {
    let mut core = sample();
    change_picture(&mut core, |picture| {
        picture.common.height += 1500;
        picture.shape_attr.current_height += 1500;
    });
    assert_below_picture(&core);
}

#[test]
fn issue_6812_picture_offset_change_recomputes_clearance_without_saved_vpos() {
    let mut core = sample();
    change_picture(&mut core, |picture| picture.common.vertical_offset += 750);
    assert_below_picture(&core);
}

#[test]
fn issue_6812_nonwrapping_picture_does_not_reserve_a_tac_line() {
    let mut core = sample();
    change_picture(&mut core, |picture| {
        picture.common.text_wrap = TextWrap::BehindText
    });
    let (_, before) = boxes(&core);
    change_picture(&mut core, |picture| picture.common.vertical_offset += 750);
    let (_, after) = boxes(&core);
    assert!(
        (before.y - after.y).abs() < 0.01,
        "배경 그림의 위치는 표 흐름을 바꾸지 않는다"
    );
}

#[test]
fn issue_6812_picture_outside_the_horizontal_frame_does_not_push_table_down() {
    let mut core = sample();
    change_picture(&mut core, |picture| {
        picture.common.horizontal_offset = 70000
    });
    let (_, before) = boxes(&core);
    change_picture(&mut core, |picture| picture.common.vertical_offset += 750);
    let (_, after) = boxes(&core);
    assert!(
        (before.y - after.y).abs() < 0.01,
        "가로로 만나지 않는 영역은 줄을 차지하지 않는다"
    );
}
