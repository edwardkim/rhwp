use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::renderer::composer::compose_paragraph;

/// LineSeg 없는 텍스트 문단
#[test]
fn test_compose_no_line_segs() {
    let para = Paragraph {
        text: "텍스트만 있음".to_string(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 7,
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 1);
    assert_eq!(composed.lines[0].runs[0].text, "텍스트만 있음");
    assert_eq!(composed.lines[0].runs[0].char_style_id, 7);
}
