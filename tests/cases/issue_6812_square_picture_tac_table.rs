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
        "원본 bit 14를 꺼서 회피 조건을 맞추지 않는다"
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

#[test]
fn issue_6812_small_table_uses_the_available_side_lane() {
    let mut core = sample();
    let mut doc = core.document().clone();
    let para = &mut doc.sections[0].paragraphs[0];
    let Control::Picture(picture) = &mut para.controls[3] else {
        panic!("그림");
    };
    picture.common.width = 12000;
    picture.shape_attr.current_width = 12000;
    let Control::Table(table) = &mut para.controls[4] else {
        panic!("표");
    };
    let mut cell = table.cells[9].clone();
    cell.row = 0;
    cell.col = 0;
    cell.row_span = 1;
    cell.col_span = 1;
    cell.width = 10000;
    cell.height = 2500;
    table.row_count = 1;
    table.col_count = 1;
    table.cells = vec![cell];
    table.cell_grid = vec![Some(0)];
    table.common.width = 10000;
    table.common.height = 2500;
    table.row_sizes = vec![1];
    core.set_document(doc);
    let (picture, table) = boxes(&core);
    assert!(
        table.x >= picture.x + picture.width - 0.5,
        "옆 구간: {picture:?}, {table:?}"
    );
    assert!(
        table.y < picture.y + picture.height - 1.0,
        "옆에 공간이 있으면 내려가지 않는다"
    );
}

#[test]
fn issue_6812_already_below_picture_does_not_add_another_clearance() {
    let mut core = sample();
    change_picture(&mut core, |picture| picture.common.vertical_offset = 0);
    let (_, before) = boxes(&core);
    change_picture(&mut core, |picture| picture.common.height += 750);
    let (picture, after) = boxes(&core);
    assert!(picture.y + picture.height < after.y);
    assert!((before.y - after.y).abs() < 0.01);
}

#[test]
fn issue_6812_missing_lineseg_does_not_disable_physical_clearance() {
    let mut core = sample();
    let mut doc = core.document().clone();
    doc.sections[0].paragraphs[0].line_segs.clear();
    core.set_document(doc);
    assert_below_picture(&core);
}

#[test]
fn issue_6812_equivalent_paper_page_and_paragraph_anchors_share_clearance() {
    let mut core = sample();
    let (original_picture, original_table) = boxes(&core);
    let page = &core.document().sections[0].section_def.page_def;
    let layout = PageLayoutInfo::from_page_def(page, &Default::default(), 96.0);
    for paragraph in [false, true] {
        change_picture(&mut core, |picture| {
            picture.common.horz_rel_to = HorzRelTo::Column;
            picture.common.vert_rel_to = if paragraph {
                VertRelTo::Para
            } else {
                VertRelTo::Page
            };
            picture.common.horizontal_offset =
                ((original_picture.x - layout.body_area.x) * 75.0).round() as u32;
            picture.common.vertical_offset =
                ((original_picture.y - layout.body_area.y) * 75.0).round() as u32;
        });
        let (picture, table) = boxes(&core);
        assert!((picture.x - original_picture.x).abs() < 0.02);
        assert!((picture.y - original_picture.y).abs() < 0.02);
        assert!(
            (table.y - original_table.y).abs() < 0.02,
            "동일 물리 배치의 줄 위치"
        );
        assert_below_picture(&core);
    }
}

fn table_box_for_control(node: &RenderNode, control_index: usize, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(0)
            && table.control_index == Some(control_index)
            && table.cell_context.is_none()
        {
            out.push(node.bbox);
        }
    }
    for child in &node.children {
        table_box_for_control(child, control_index, out);
    }
}

#[test]
fn issue_6812_multiple_picture_boundaries_are_consumed_before_placing_table() {
    let mut core = sample();
    let mut doc = core.document().clone();
    let para = &mut doc.sections[0].paragraphs[0];
    let Control::Picture(mut second) = para.controls[3].clone() else {
        panic!("그림");
    };
    second.common.vertical_offset += 4500;
    let required_top =
        (f64::from(second.common.vertical_offset + second.common.height) + 141.0) / 75.0;
    para.controls.insert(4, Control::Picture(second));
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut tables = Vec::new();
    table_box_for_control(&tree.root, 5, &mut tables);
    assert_eq!(tables.len(), 1);
    assert!(
        tables[0].y + 0.5 >= required_top,
        "두 경계를 모두 지나야 한다: {:?}, {required_top}",
        tables[0]
    );
}

#[test]
fn issue_6812_future_picture_does_not_reposition_an_already_placed_table() {
    let mut core = sample();
    let mut doc = core.document().clone();
    doc.sections[0].paragraphs[0].controls.swap(3, 4);
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut tables = Vec::new();
    table_box_for_control(&tree.root, 3, &mut tables);
    assert_eq!(tables.len(), 1);
    assert!(
        (tables[0].y - 81.24).abs() < 0.5,
        "뒤에서 등장한 그림을 미리 예약하면 안 된다"
    );
}

#[test]
fn issue_6812_clearance_participates_in_page_fit_and_expires_at_page_boundary() {
    let mut core = sample();
    let mut doc = core.document().clone();
    doc.sections.truncate(1);
    doc.sections[0].paragraphs.truncate(1);
    let page = &mut doc.sections[0].section_def.page_def;
    let layout = PageLayoutInfo::from_page_def(page, &Default::default(), 96.0);
    // 표 단독은 들어가지만 그림을 피한 표는 들어가지 않는 200px 본문이다.
    page.height = page.height - (layout.body_area.height * 75.0).round() as u32 + 15000;
    core.set_document(doc);
    assert_eq!(core.page_count(), 2, "추가 줄 이동량도 쪽 예산을 소비한다");
    let first = core.build_page_render_tree(0).unwrap();
    let second = core.build_page_render_tree(1).unwrap();
    let (mut old, mut moved) = (Vec::new(), Vec::new());
    table_box_for_control(&first.root, 4, &mut old);
    table_box_for_control(&second.root, 4, &mut moved);
    assert!(old.is_empty(), "표가 첫 쪽에 중복/넘침 배치되면 안 된다");
    assert_eq!(moved.len(), 1);
    assert!(
        (moved[0].y - layout.body_area.y - 141.0 / 75.0).abs() < 0.5,
        "앞 쪽의 그림 회피를 새 쪽에 중복 가산하면 안 된다: {:?}",
        moved[0]
    );
}

#[test]
fn issue_6812_previous_paragraph_picture_reserves_space_until_its_bottom() {
    let mut core = sample();
    let mut doc = core.document().clone();
    doc.sections.truncate(1);
    doc.sections[0].paragraphs.truncate(1);
    let picture_host = &mut doc.sections[0].paragraphs[0];
    let table = picture_host.controls.remove(4);
    let mut table_host = picture_host.clone();
    table_host.controls = vec![table];
    // 앞 문단은 그림의 앵커이며 짧은 실제 줄이다. 표를 포함했던 저장 줄높이를
    // 남겨서 우연히 이미 그림 아래에 놓이는 가짜 대조를 만들지 않는다.
    picture_host.line_segs.clear();
    table_host.line_segs.clear();
    doc.sections[0].paragraphs.push(table_host);
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut tables = Vec::new();
    collect_top_level_tables(&tree.root, &mut tables);
    let matching: Vec<_> = tables.iter().filter(|(pi, ci, _)| (*pi, *ci) == (1, 0)).collect();
    assert_eq!(matching.len(), 1);
    let (mut pictures, mut unused) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut pictures, &mut unused);
    assert_eq!(pictures.len(), 1);
    let required_y = pictures[0].y + pictures[0].height + 141.0 / 75.0;
    assert!((matching[0].2.y - required_y).abs() < 0.5,
        "다른 문단의 유효한 선행 점유 영역도 소비한다: {:?}, {required_y}", matching[0]);
}

fn collect_top_level_tables(node: &RenderNode, out: &mut Vec<(usize, usize, BoundingBox)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.cell_context.is_none() {
            if let (Some(pi), Some(ci)) = (table.para_index, table.control_index) {
                out.push((pi, ci, node.bbox));
            }
        }
    }
    for child in &node.children {
        collect_top_level_tables(child, out);
    }
}
