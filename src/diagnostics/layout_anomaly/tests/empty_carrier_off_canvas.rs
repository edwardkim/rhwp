//! 빈 줄의 점유 상자와 실제 그리기 범위를 구분하는 진단 계약.

use super::*;

fn carrier(text: &str) -> RenderNode {
    let mut line = text_line(20.0, 195.0, 40.0, 20.0);
    let mut run = text_run_at(text, 20.0, 195.0, 40.0, 20.0);
    if let RenderNodeType::TextRun(run) = &mut run.node_type {
        run.baseline = 16.0;
    }
    line.children.push(run);
    line
}

fn off_canvas(line: RenderNode) -> PageAnomalies {
    let frame = table(10.0, 10.0, 100.0, 100.0, vec![line]);
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 200.0), vec![frame]);
    let root = page_root(200.0, 200.0, body);
    let result = scan_page(0, &root, 1, &AnomalyOptions::default());
    let source_line = &root.children[0].children[0].children[0];
    assert_eq!(
        source_line.bbox.y, 195.0,
        "진단이 원본 줄 원점을 바꾸면 안 된다"
    );
    assert_eq!(
        source_line.bbox.height, 20.0,
        "빈 줄의 점유 높이를 없애지 않는다"
    );
    result
}

#[test]
fn unpainted_carrier_does_not_expand_ancestor_drawing_bounds() {
    let result = off_canvas(carrier(""));
    assert!(
        result.off_canvas.is_empty(),
        "빈 커서 상자의 세로 합집합 오탐: {result:?}"
    );
}

#[test]
fn whitespace_display_decoration_and_object_children_remain_checked() {
    for text in [" ", "보이는 글자"] {
        assert_eq!(off_canvas(carrier(text)).off_canvas.len(), 1);
    }
    for decoration in ["display", "border", "shade"] {
        let mut line = carrier("");
        let RenderNodeType::TextRun(run) = &mut line.children[0].node_type else {
            unreachable!()
        };
        match decoration {
            "display" => run.display_text = Some("★".into()),
            "border" => run.border_fill_id = 1,
            "shade" => run.style.shade_color = 0x0000_ff00,
            _ => unreachable!(),
        }
        assert_eq!(off_canvas(line).off_canvas.len(), 1, "{decoration}");
    }
    let mut line = carrier("");
    line.children
        .push(floating_shape(20.0, 195.0, 20.0, 20.0, TextWrap::Square));
    assert_eq!(
        off_canvas(line).off_canvas.len(),
        1,
        "빈 호스트의 실제 개체"
    );
}
