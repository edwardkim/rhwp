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
    let placement = ParagraphFloatPlacement { anchor_y: 20.0, table_top: 50.0, occupied_bottom: 100.0 };
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
            let tree = core.build_page_render_tree(page).expect("영역 변경 후 조판");
            fn body_bottom(node: &RenderNode) -> Option<f64> {
                if matches!(node.node_type, RenderNodeType::Body { .. }) {
                    return Some(node.bbox.y + node.bbox.height);
                }
                node.children.iter().find_map(body_bottom)
            }
            let body_bottom = body_bottom(&tree.root).expect("본문 영역");
            let mut items = Vec::new();
            body_items(&tree.root, &mut items);
            let host_bottom = items.iter().filter_map(|n| match &n.node_type {
                RenderNodeType::TextLine(line) if line.para_index == Some(1) =>
                    Some(n.bbox.y + n.bbox.height),
                _ => None,
            }).fold(0.0_f64, f64::max);
            for node in &items {
                if matches!(&node.node_type, RenderNodeType::Table(t)
                    if t.para_index == Some(1) && t.control_index == Some(0)) {
                    table_fragments += 1;
                    assert!(node.bbox.y + node.bbox.height <= body_bottom + 0.5,
                        "영역 축소 {reduction}, 쪽 {page}: 표 하단 {}, 본문 하단 {body_bottom}",
                        node.bbox.y + node.bbox.height);
                    assert!(node.bbox.y >= host_bottom - 0.1,
                        "영역 축소 {reduction}, 쪽 {page}: 본문 끝 {host_bottom}, 표 {}", node.bbox.y);
                }
            }
        }
        assert!(table_fragments > 0, "영역 축소로 표가 사라지면 안 된다");
    }
}

#[test]
fn reflowed_host_does_not_use_stale_stored_line_coordinates() {
    // 저장 줄 캐시가 없는 편집 상태를 직접 구성한다. 파일을 변조해 저장하지 않는다.
    let mut core = core();
    let mut document = core.document().clone();
    document.sections[0].paragraphs[1].invalidate_layout_inputs();
    core.set_document(document);
    let tree = core.build_page_render_tree(0).expect("호스트 재조판");
    let mut items = Vec::new();
    body_items(&tree.root, &mut items);
    let host: Vec<_> = items.iter().filter(|n| matches!(&n.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(1))).collect();
    assert!(!host.is_empty());
    let (top, _) = table(&items, 1, 0);
    let bottom = host.iter().map(|n| n.bbox.y + n.bbox.height).fold(0.0_f64, f64::max);
    assert!(top >= bottom, "재조판된 본문 끝 {bottom}, 표 상단 {top}");
}
