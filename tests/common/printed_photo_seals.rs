// 독립 한컴 Print로 확인한 응시원서의 사진 날인 소유 계약.
// 실제 본문 충돌은 계속 검사하고, 같은 사진 셀 안의 원본 날인 교차만 구분한다.

use rhwp::diagnostics::layout_anomaly::DocAnomalies;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{ShapeObject, TextWrap};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

type Owner = (usize, usize);

fn visible_text(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text(),
        _ => "",
    };
    let mut text = own.to_string();
    for child in &node.children {
        text.push_str(&visible_text(child));
    }
    text
}

fn contains(outer: BoundingBox, inner: BoundingBox) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

fn collect(
    node: &RenderNode,
    doc: &DocumentCore,
    table_owner: Option<Owner>,
    photo: Option<(Owner, BoundingBox)>,
    seal: Option<Owner>,
    seals: &mut Vec<(Owner, BoundingBox)>,
    photo_runs: &mut Vec<(Owner, BoundingBox, BoundingBox)>,
) {
    let mut table_owner = table_owner;
    let mut photo = photo;
    let mut seal = seal;
    match &node.node_type {
        RenderNodeType::Table(table) => {
            table_owner = table.section_index.zip(table.para_index);
            photo = None;
        }
        RenderNodeType::TableCell(_) => {
            let caption = visible_text(node);
            if caption.contains("사진") && caption.contains("3.5cm×4.5cm") {
                photo = table_owner.map(|owner| (owner, node.bbox));
            }
        }
        RenderNodeType::Group(group) if visible_text(node).trim() == "㊞" => {
            let (si, pi, ci) = (
                group.section_index.expect("날인 구역 소유"),
                group.para_index.expect("날인 문단 소유"),
                group.control_index.expect("날인 개체 소유"),
            );
            let Control::Shape(shape) = &doc.document().sections[si].paragraphs[pi].controls[ci]
            else {
                panic!("날인은 원본 묶음 도형이다");
            };
            let ShapeObject::Group(source) = shape.as_ref() else {
                panic!("날인 묶음 도형의 원본 종류");
            };
            assert_eq!(source.common.text_wrap, TextWrap::InFrontOfText);
            seal = Some((si, pi));
        }
        RenderNodeType::TextRun(run) => {
            if let Some(owner) = seal {
                assert_eq!(run.display_or_text().trim(), "㊞");
                seals.push((owner, node.bbox));
            }
            if let Some((owner, cell)) = photo {
                if matches!(run.display_or_text().trim(), "2" | "매") {
                    photo_runs.push((owner, cell, node.bbox));
                }
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, doc, table_owner, photo, seal, seals, photo_runs);
    }
}

pub fn count_unintended_overlaps(doc: &DocumentCore, anomalies: &DocAnomalies) -> u64 {
    // 독립 Print의 페이지·날인 소유다. 좌표/픽셀/교차 건수는 기대값으로 고정하지 않는다.
    assert_eq!(doc.page_count(), 4);
    let mut remaining = 0;
    for page in 0..4 {
        let tree = doc.build_page_render_tree(page).expect("응시원서 페이지");
        let mut seals = Vec::new();
        let mut photo_runs = Vec::new();
        collect(
            &tree.root,
            doc,
            None,
            None,
            None,
            &mut seals,
            &mut photo_runs,
        );
        assert_eq!(seals.len(), [3, 0, 1, 0][page as usize]);
        for &(owner, cell, _) in &photo_runs {
            assert!(
                seals
                    .iter()
                    .any(|&(seal_owner, bbox)| { seal_owner == owner && contains(cell, bbox) }),
                "사진 설명과 날인은 같은 원본 문단의 사진 셀 안에 있어야 한다"
            );
        }
        if let Some(anomaly_page) = anomalies.pages.iter().find(|p| p.page == page) {
            remaining += anomaly_page
                .text_overlap
                .iter()
                .filter(|overlap| {
                    let intended_pair = |a: BoundingBox, b: BoundingBox| {
                        seals.iter().any(|&(owner, seal_bbox)| {
                            contains(seal_bbox, a)
                                && photo_runs.iter().any(|&(photo_owner, cell, run_bbox)| {
                                    photo_owner == owner
                                        && contains(cell, seal_bbox)
                                        && contains(run_bbox, b)
                                })
                        })
                    };
                    !intended_pair(overlap.bbox_a, overlap.bbox_b)
                        && !intended_pair(overlap.bbox_b, overlap.bbox_a)
                })
                .count() as u64;
        }
    }
    remaining
}
