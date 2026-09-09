//! 실제 한컴 fixture의 배치와 공통 앵커 계약을 분리해서 검증한다.
//! 직접 바꾼 IR은 알고리즘 경계 검사용이지 한컴 저장 문서/정답지가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::LineSeg, shape::TextWrap};
use rhwp::renderer::float_placement::ParagraphFloatPlacement;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/hwpx/20260909-para-table.hwpx"),
    )
    .expect("#6950 실제 fixture");
    DocumentCore::from_bytes(&bytes).expect("원본 로드")
}

fn body_items<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    match &node.node_type {
        RenderNodeType::Table(_) | RenderNodeType::TextLine(_) => {
            out.push(node);
            return;
        }
        _ => {}
    }
    for child in &node.children {
        body_items(child, out);
    }
}

fn table(items: &[&RenderNode], pi: usize, ci: usize) -> (f64, f64) {
    let nodes: Vec<_> = items
        .iter()
        .filter(|n| {
            matches!(&n.node_type,
        RenderNodeType::Table(t) if t.para_index == Some(pi) && t.control_index == Some(ci))
        })
        .collect();
    assert_eq!(nodes.len(), 1, "표의 유일성: {pi}/{ci}");
    (nodes[0].bbox.y, nodes[0].bbox.y + nodes[0].bbox.height)
}

#[test]
fn paragraph_text_table_and_following_text_do_not_overlap() {
    let core = core();
    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut items = Vec::new();
    body_items(&tree.root, &mut items);
    let host: Vec<_> = items
        .iter()
        .filter(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(1))
        })
        .collect();
    assert_eq!(host.len(), 4, "호스트 네 줄 보존");
    let (top, bottom) = table(&items, 1, 0);
    let last_text_bottom = host
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(0.0_f64, f64::max);
    assert!(
        top >= last_text_bottom,
        "본문 끝 {last_text_bottom} 위로 표 {top}가 올라오면 안 된다"
    );
    let following: Vec<_> = items
        .iter()
        .filter(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(3))
        })
        .collect();
    assert!(!following.is_empty(), "후속 본문 누락 금지");
    assert!(
        following.iter().all(|n| n.bbox.y >= bottom),
        "후속 본문은 예약된 표 아래에 위치해야 한다"
    );
}

#[test]
fn preceding_tables_and_page_count_are_preserved() {
    let core = core();
    assert_eq!(core.page_count(), 3, "한컴 PDF와 같은 3쪽");
    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut items = Vec::new();
    body_items(&tree.root, &mut items);
    let a = table(&items, 0, 2);
    let b = table(&items, 0, 3);
    let c = table(&items, 0, 4);
    assert!(
        a.1 <= b.0 + 0.1 && b.1 <= c.0 + 0.1,
        "앞선 세 표의 순서/비겹침"
    );
}

#[test]
fn anchor_origin_translation_is_applied_once() {
    let core = core();
    let mut para = core.document().sections[0].paragraphs[1].clone();
    let Control::Table(table) = para.controls[0].clone() else {
        panic!("표")
    };
    let first =
        ParagraphFloatPlacement::from_stored_host(&para, &table, 0, 200.0, 100.0, 96.0).unwrap();
    let shifted =
        ParagraphFloatPlacement::from_stored_host(&para, &table, 0, 320.0, 100.0, 96.0).unwrap();
    assert!((shifted.table_top - first.table_top - 120.0).abs() < 1e-9);
    assert!((shifted.occupied_bottom - first.occupied_bottom - 120.0).abs() < 1e-9);
    for line in &mut para.line_segs {
        line.vertical_pos += 6000;
    }
    let rebased =
        ParagraphFloatPlacement::from_stored_host(&para, &table, 0, 200.0, 100.0, 96.0).unwrap();
    assert_eq!(
        first, rebased,
        "저장 원점의 절대 수치가 줄 간격을 바꾸지 않는다"
    );
}

#[test]
fn occupied_bands_move_the_box_not_the_anchor_and_are_order_independent() {
    let placement = ParagraphFloatPlacement {
        anchor_y: 20.0,
        table_top: 50.0,
        occupied_bottom: 100.0,
    };
    let bands = [80.0..120.0, 125.0..180.0];
    let forward = placement.clear_occupied_bands(bands.clone());
    let reverse = placement.clear_occupied_bands(bands.into_iter().rev());
    assert_eq!(forward, reverse);
    assert_eq!(forward.anchor_y, placement.anchor_y);
    assert_eq!(forward.table_top, 180.0);
    assert_eq!(forward.occupied_bottom - forward.table_top, 50.0);
}

#[test]
fn wrap_semantics_are_not_reclassified_as_topbottom() {
    let core = core();
    let para = &core.document().sections[0].paragraphs[1];
    let Control::Table(mut table) = para.controls[0].clone() else {
        panic!("표")
    };
    for wrap in [
        TextWrap::Square,
        TextWrap::BehindText,
        TextWrap::InFrontOfText,
    ] {
        table.common.text_wrap = wrap;
        assert!(
            ParagraphFloatPlacement::from_stored_host(para, &table, 0, 0.0, 100.0, 96.0).is_none()
        );
    }
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.treat_as_char = true;
    assert!(ParagraphFloatPlacement::from_stored_host(para, &table, 0, 0.0, 100.0, 96.0).is_none());
}

#[test]
fn unproven_source_coordinates_do_not_supply_a_stored_plan() {
    let core = core();
    let mut para = core.document().sections[0].paragraphs[1].clone();
    let Control::Table(table) = para.controls[0].clone() else {
        panic!("표")
    };
    assert!(ParagraphFloatPlacement::from_stored_host(&para, &table, 0, 0.0, 100.0, 0.0).is_none());
    let mut dirty = para.clone();
    dirty.invalidate_layout_inputs();
    assert!(
        ParagraphFloatPlacement::from_stored_host(&dirty, &table, 0, 0.0, 100.0, 96.0).is_none(),
        "저장 줄이 남아 있어도 무효화됐다면 저장 배치에 사용하지 않는다"
    );
    para.line_segs[1].tag |= LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    assert!(
        ParagraphFloatPlacement::from_stored_host(&para, &table, 0, 0.0, 100.0, 96.0).is_none()
    );
    para.line_segs[1].tag &= !LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    para.line_segs[1].vertical_pos = 0;
    assert!(
        ParagraphFloatPlacement::from_stored_host(&para, &table, 0, 0.0, 100.0, 96.0).is_none()
    );
}

#[test]
fn shorter_body_does_not_hide_table_overflow_by_moving_it_over_host_text() {
    // 직접 구성한 IR의 영역 경계 검사다. 한컴에서 저장한 별도 샘플이 아니다.
    for reduction in [6000, 12000, 18000] {
        let mut core = core();
        let mut document = core.document().clone();
        document.sections[0].section_def.page_def.margin_bottom += reduction;
        core.set_document(document);
        let mut table_fragments = 0;
        for page in 0..core.page_count() {
            let tree = core
                .build_page_render_tree(page)
                .expect("영역 변경 후 조판");
            fn body_bottom(node: &RenderNode) -> Option<f64> {
                if matches!(node.node_type, RenderNodeType::Body { .. }) {
                    return Some(node.bbox.y + node.bbox.height);
                }
                node.children.iter().find_map(body_bottom)
            }
            let body_bottom = body_bottom(&tree.root).expect("본문 영역");
            let mut items = Vec::new();
            body_items(&tree.root, &mut items);
            let host_bottom = items
                .iter()
                .filter_map(|n| match &n.node_type {
                    RenderNodeType::TextLine(line) if line.para_index == Some(1) => {
                        Some(n.bbox.y + n.bbox.height)
                    }
                    _ => None,
                })
                .fold(0.0_f64, f64::max);
            for node in &items {
                if matches!(&node.node_type, RenderNodeType::Table(t)
                    if t.para_index == Some(1) && t.control_index == Some(0))
                {
                    table_fragments += 1;
                    assert!(
                        node.bbox.y + node.bbox.height <= body_bottom + 0.5,
                        "영역 축소 {reduction}, 쪽 {page}: 표 하단 {}, 본문 하단 {body_bottom}",
                        node.bbox.y + node.bbox.height
                    );
                    assert!(
                        node.bbox.y >= host_bottom - 0.1,
                        "영역 축소 {reduction}, 쪽 {page}: 본문 끝 {host_bottom}, 표 {}",
                        node.bbox.y
                    );
                }
            }
        }
        assert!(table_fragments > 0, "영역 축소로 표가 사라지면 안 된다");
    }
}

#[test]
fn reflowed_host_does_not_use_stale_stored_line_coordinates() {
    // 저장 줄을 재조판용 템플릿으로 남기되 무효화한 편집 상태다. 파일로 저장하지 않는다.
    let mut core = core();
    let mut document = core.document().clone();
    document.sections[0].paragraphs[1].invalidate_layout_inputs();
    core.set_document(document);
    let tree = core.build_page_render_tree(0).expect("호스트 재조판");
    let mut items = Vec::new();
    body_items(&tree.root, &mut items);
    let host: Vec<_> = items
        .iter()
        .filter(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(1))
        })
        .collect();
    assert!(!host.is_empty());
    let (top, _) = table(&items, 1, 0);
    let bottom = host
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(0.0_f64, f64::max);
    assert!(top >= bottom, "재조판된 본문 끝 {bottom}, 표 상단 {top}");
    let anchor_top = host.iter().map(|n| n.bbox.y).fold(0.0_f64, f64::max);
    let Control::Table(target) = &core.document().sections[0].paragraphs[1].controls[0] else {
        panic!("표")
    };
    let expected = anchor_top
        + rhwp::renderer::hwpunit_to_px(
            target.common.vertical_offset as i32 + i32::from(target.outer_margin_top),
            96.0,
        );
    assert!(
        (top - expected).abs() < 0.1,
        "재조판된 마지막 앵커 줄 {anchor_top}와 위치 속성으로 결정한 {expected}, 출력 {top}"
    );
}

#[test]
fn top_caption_is_inside_the_reserved_box_before_the_table_body() {
    use rhwp::model::{
        paragraph::Paragraph,
        shape::{Caption, CaptionDirection},
    };
    let baseline = core();
    let tree = baseline.build_page_render_tree(0).unwrap();
    let mut items = Vec::new();
    body_items(&tree.root, &mut items);
    let (without_caption, _) = table(&items, 1, 0);
    let mut document = baseline.document().clone();
    let Control::Table(target) = &mut document.sections[0].paragraphs[1].controls[0] else {
        panic!("표")
    };
    target.caption = Some(Caption {
        direction: CaptionDirection::Top,
        spacing: 300,
        paragraphs: vec![Paragraph {
            text: "검증용 캡션".into(),
            line_segs: vec![LineSeg {
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 800,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
    let caption_extra = rhwp::renderer::composer::caption_height_px(&target.caption, 96.0) + 4.0;
    let mut core = core();
    core.set_document(document);
    let tree = core.build_page_render_tree(0).unwrap();
    items.clear();
    body_items(&tree.root, &mut items);
    let (with_caption, bottom) = table(&items, 1, 0);
    assert!((with_caption - without_caption - caption_extra).abs() < 0.1,
        "위 캡션은 예약 상자 안에서 한 번만 반영: {without_caption} → {with_caption}, 캡션 {caption_extra}");
    assert!(items
        .iter()
        .filter(|n| matches!(&n.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(3)))
        .all(|n| n.bbox.y >= bottom));
}

#[test]
fn computed_anchor_uses_scalar_positions_and_not_stored_geometry() {
    use rhwp::renderer::float_placement::ParagraphHostLine;
    let core = core();
    let mut para = core.document().sections[0].paragraphs[1].clone();
    let Control::Table(mut target) = para.controls[0].clone() else {
        panic!("표")
    };
    // 한글·surrogate pair와 뒤 control의 UTF-16 간격을 포함한 직접 IR이다.
    para.text = "가😀나다".into();
    para.char_offsets = vec![0, 1, 3, 4];
    target.common.vertical_offset = 1500;
    let lines = [
        ParagraphHostLine {
            char_start: 0,
            top: 0.0,
            height: 10.0,
        },
        ParagraphHostLine {
            char_start: 2,
            top: 30.0,
            height: 10.0,
        },
    ];
    let place = |p: &rhwp::model::paragraph::Paragraph, origin| {
        ParagraphFloatPlacement::from_computed_host(p, &target, 0, origin, &lines, 100.0, 96.0)
            .unwrap()
    };
    let a = place(&para, 200.0);
    assert_eq!(
        a.anchor_y, 230.0,
        "끝 control은 scalar 4에 있어 둘째 줄에 속한다"
    );
    for ls in &mut para.line_segs {
        ls.vertical_pos = 90000;
        ls.line_height = 50000;
    }
    assert_eq!(
        a,
        place(&para, 200.0),
        "오래된 source 좌표를 재사용하지 않는다"
    );
    let b = place(&para, 320.0);
    assert!((b.table_top - a.table_top - 120.0).abs() < 1e-9);
    assert!((b.occupied_bottom - a.occupied_bottom - 120.0).abs() < 1e-9);
    // 첫 문자와 둘째 문자 사이의 8 UTF-16 단위 control을 첫 줄에 대응한다.
    para.char_offsets = vec![0, 9, 11, 12];
    assert!(
        ParagraphFloatPlacement::from_computed_host(&para, &target, 0, 200.0, &lines, 100.0, 96.0)
            .is_none(),
        "첫 줄 앵커의 표 영역에 뒤 호스트 줄이 걸리면 선행 호스트 계약이 아니다"
    );
}

#[test]
fn computed_host_rejects_invalid_rows_and_other_wrap_owners() {
    use rhwp::renderer::float_placement::ParagraphHostLine;
    let core = core();
    let para = &core.document().sections[0].paragraphs[1];
    let Control::Table(mut target) = para.controls[0].clone() else {
        panic!("표")
    };
    let row = ParagraphHostLine {
        char_start: 0,
        top: 0.0,
        height: 10.0,
    };
    for rows in [
        vec![],
        vec![ParagraphHostLine {
            height: f64::NAN,
            ..row
        }],
        vec![row, ParagraphHostLine { top: -1.0, ..row }],
        vec![
            row,
            ParagraphHostLine {
                char_start: usize::MAX,
                top: 10.0,
                ..row
            },
        ],
    ] {
        assert!(ParagraphFloatPlacement::from_computed_host(
            para, &target, 0, 0.0, &rows, 100.0, 96.0
        )
        .is_none());
    }
    for wrap in [
        TextWrap::Square,
        TextWrap::BehindText,
        TextWrap::InFrontOfText,
    ] {
        target.common.text_wrap = wrap;
        assert!(ParagraphFloatPlacement::from_computed_host(
            para,
            &target,
            0,
            0.0,
            &[row],
            100.0,
            96.0
        )
        .is_none());
    }
    target.common.text_wrap = TextWrap::TopAndBottom;
    target.common.treat_as_char = true;
    assert!(ParagraphFloatPlacement::from_computed_host(
        para,
        &target,
        0,
        0.0,
        &[row],
        100.0,
        96.0
    )
    .is_none());
}

#[test]
fn typeset_publishes_a_computed_placement_for_the_current_frame() {
    use rhwp::renderer::{
        composer::compose_section, height_measurer::HeightMeasurer, style_resolver::resolve_styles,
        typeset::TypesetEngine,
    };
    let core = core();
    let doc = core.document();
    let styles = resolve_styles(&doc.doc_info, 96.0);
    let mut section = doc.sections[0].clone();
    section.paragraphs = vec![section.paragraphs[1].clone()];
    section.paragraphs[0].invalidate_layout_inputs();
    for without_source_rows in [false, true] {
        if without_source_rows {
            section.paragraphs[0].line_segs.clear();
        }
        let mut anchors = Vec::new();
        for reduction in [0, 6000] {
            let mut page = section.section_def.page_def.clone();
            page.margin_right += reduction;
            let width = rhwp::renderer::hwpunit_to_px(
                (page.width - page.margin_left - page.margin_right) as i32,
                96.0,
            );
            let composed = compose_section(&section);
            let measured = HeightMeasurer::new(96.0).measure_section(
                &section.paragraphs,
                &composed,
                &styles,
                Some(width),
            );
            let pages = TypesetEngine::new(96.0).typeset_section(
                &section.paragraphs,
                &composed,
                &styles,
                &page,
                &Default::default(),
                0,
                &measured.tables,
                false,
                &Default::default(),
            );
            let placements: Vec<_> = pages
                .pages
                .iter()
                .flat_map(|p| &p.column_contents)
                .filter_map(|c| c.paragraph_float_placements.get(&(0, 0)))
                .collect();
            assert_eq!(
                placements.len(),
                1,
                "현재 단에 확정 배치를 한 번 전달해야 한다"
            );
            let p = placements[0];
            assert!(
                p.anchor_y.is_finite()
                    && p.table_top > p.anchor_y
                    && p.occupied_bottom > p.table_top
            );
            anchors.push(p.anchor_y);
        }
        assert!(
            anchors[1] > anchors[0],
            "폭 축소에 따른 실제 줄바꿈이 앵커에 반영되어야 한다: {anchors:?}"
        );
    }
}

#[test]
fn computed_frame_placement_reaches_paint_and_following_flow() {
    for reduction in [0, 6000] {
        let mut core = core();
        let mut doc = core.document().clone();
        let mut host = doc.sections[0].paragraphs[1].clone();
        host.line_segs.clear();
        host.invalidate_layout_inputs();
        let mut following = doc.sections[0].paragraphs[3].clone();
        following.line_segs.clear();
        following.invalidate_layout_inputs();
        doc.sections[0].paragraphs = vec![host, following];
        doc.sections[0].section_def.page_def.margin_right += reduction;
        core.set_document(doc);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut items = Vec::new();
        body_items(&tree.root, &mut items);
        let (top, bottom) = table(&items, 0, 0);
        let host_lines: Vec<_> = items
            .iter()
            .filter(|n| {
                matches!(&n.node_type,
            RenderNodeType::TextLine(line) if line.para_index == Some(0))
            })
            .collect();
        assert!(!host_lines.is_empty());
        let anchor = host_lines.iter().map(|n| n.bbox.y).fold(0.0_f64, f64::max);
        let Control::Table(target) = &core.document().sections[0].paragraphs[0].controls[0] else {
            panic!("표")
        };
        let expected = anchor
            + rhwp::renderer::hwpunit_to_px(
                target.common.vertical_offset as i32 + i32::from(target.outer_margin_top),
                96.0,
            );
        assert!(
            (top - expected).abs() < 0.1,
            "폭 축소 {reduction}: 실제 앵커 기반 {expected}, 표 출력 {top}"
        );
        let next: Vec<_> = items
            .iter()
            .filter(|n| {
                matches!(&n.node_type,
            RenderNodeType::TextLine(line) if line.para_index == Some(1))
            })
            .collect();
        assert!(!next.is_empty());
        assert!(
            next.iter().all(|n| n.bbox.y >= bottom - 0.1),
            "폭 축소 {reduction}: 예약한 표 뒤에 다음 문단이 와야 한다"
        );
    }
}

#[test]
fn split_computed_host_publishes_fragment_local_placements() {
    use rhwp::renderer::{
        composer::compose_section, height_measurer::HeightMeasurer,
        pagination::PageItem, style_resolver::resolve_styles, typeset::TypesetEngine,
    };
    let core = core();
    let doc = core.document();
    let styles = resolve_styles(&doc.doc_info, 96.0);
    for without_source_rows in [false, true] {
        let mut section = doc.sections[0].clone();
        section.paragraphs = vec![section.paragraphs[1].clone()];
        section.paragraphs[0].invalidate_layout_inputs();
        if without_source_rows {
            section.paragraphs[0].line_segs.clear();
        }
        let mut page = section.section_def.page_def.clone();
        page.height = page.margin_top + page.margin_bottom + 18000;
        let width = rhwp::renderer::hwpunit_to_px(
            (page.width - page.margin_left - page.margin_right) as i32, 96.0,
        );
        let composed = compose_section(&section);
        let measured = HeightMeasurer::new(96.0).measure_section(
            &section.paragraphs, &composed, &styles, Some(width),
        );
        let pages = TypesetEngine::new(96.0).typeset_section(
            &section.paragraphs, &composed, &styles, &page, &Default::default(),
            0, &measured.tables, false, &Default::default(),
        );
        let mut fragments = 0;
        for column in pages.pages.iter().flat_map(|p| &p.column_contents) {
            for item in &column.items {
                if let PageItem::PartialTable { para_index: 0, control_index: 0,
                    is_continuation, .. } = item {
                    fragments += 1;
                    let placement = column.paragraph_float_placements.get(&(0, 0))
                        .expect("분할 표도 현재 단의 확정 배치를 전달해야 한다");
                    assert!(placement.occupied_bottom > placement.table_top);
                    if *is_continuation {
                        assert!(placement.table_top.abs() < 0.1,
                            "다음 단에 이전 앵커 거리 재적용 금지: {placement:?}");
                    } else {
                        assert!(placement.anchor_y > 0.0);
                        assert!(placement.table_top > placement.anchor_y);
                    }
                }
            }
        }
        assert!(fragments >= 2, "실제 분할 경로를 검증해야 한다: {fragments}, {pages:?}");
    }
}
