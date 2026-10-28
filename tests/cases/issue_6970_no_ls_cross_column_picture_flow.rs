//! #7104 메인터너 보정: 앞 단에 연결되어 다음 단에 놓이는 Square 그림.
//! 동일 원본의 한컴 PDF는 그림 오른쪽으로 본문을 감싼다. 페이지 수나
//! 흰색 이미지의 픽셀 대신 실제 그림/글줄의 교집합으로 계약을 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, shape::TextWrap};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn source() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn collect(node: &RenderNode, pictures: &mut Vec<BoundingBox>, lines: &mut Vec<BoundingBox>) {
    match &node.node_type {
        RenderNodeType::Image(image) if matches!(image.para_index, Some(6 | 12 | 19)) =>
            pictures.push(node.bbox),
        RenderNodeType::TextLine(_) if node.children.iter().any(|child|
            matches!(&child.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty())) =>
            lines.push(node.bbox),
        _ => {}
    }
    for child in &node.children {
        collect(child, pictures, lines);
    }
}

fn intersections(core: &DocumentCore) -> Vec<(BoundingBox, BoundingBox)> {
    let tree = core.build_page_render_tree(0).unwrap();
    let (mut pictures, mut lines) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut pictures, &mut lines);
    assert_eq!(pictures.len(), 3, "그림을 누락시켜 겹침을 없애면 안 된다");
    assert!(lines.len() > 30, "본문을 누락시켜 통과하면 안 된다");
    let mut overlaps = Vec::new();
    for image in pictures {
        for line in &lines {
            let x = (image.x + image.width).min(line.x + line.width) - image.x.max(line.x);
            let y = (image.y + image.height).min(line.y + line.height) - image.y.max(line.y);
            if x > 0.5 && y > 0.5 {
                overlaps.push((image, *line));
            }
        }
    }
    overlaps
}

#[test]
fn issue_6970_no_ls_text_avoids_previous_column_pictures_on_same_page() {
    let core = source();
    assert_eq!(core.page_count(), 3);
    assert!(core.document().sections[0].paragraphs[39]
        .line_segs
        .is_empty());
    assert!(
        intersections(&core).is_empty(),
        "실제 그림과 본문이 교차함: {:?}",
        intersections(&core)
    );
    assert!(
        core.document().sections[0].paragraphs[39]
            .line_segs
            .is_empty(),
        "렌더 전용 행을 원본 저장 LineSeg로 바꾸지 않는다"
    );
}

#[test]
fn issue_6970_behind_text_pictures_do_not_reserve_a_side_lane() {
    let mut core = source();
    let mut document = core.document().clone();
    for index in [6, 12, 19] {
        for control in &mut document.sections[0].paragraphs[index].controls {
            if let Control::Picture(picture) = control {
                picture.common.text_wrap = TextWrap::BehindText;
            }
        }
    }
    core.set_document(document);
    assert!(
        !intersections(&core).is_empty(),
        "BehindText는 본문 뒤에 겹칠 수 있다"
    );
}

/// 같은 행의 빈 우측 lane을 다음 글줄로 세어 마지막 제목을 벌리면 안 된다.
#[test]
fn issue_6970_empty_wrap_lane_does_not_justify_final_heading() {
    let core = source();
    let tree = core.build_page_render_tree(1).unwrap();
    fn collect(node: &RenderNode, found: &mut Vec<(usize, f64)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if let Some(index @ (88 | 93 | 99)) = run.para_index {
                if !run.text.is_empty() {
                    found.push((index, node.bbox.width));
                }
            }
        }
        for child in &node.children {
            collect(child, found);
        }
    }
    let mut found = Vec::new();
    collect(&tree.root, &mut found);
    for (index, width) in [(88, 64.0), (93, 32.0), (99, 48.0)] {
        assert!(
            found
                .iter()
                .any(|(i, w)| *i == index && (*w - width).abs() < 0.5),
            "12pt 전각 제목의 자연 폭을 보존해야 한다: {found:?}"
        );
    }
}
