//! 선 종류가 "없음"(HWPX `lineShape style="NONE"`, HWP 선 속성 bit 0-5 = 0)인 직선은
//! 그리지 않는다. 종전에는 0 을 실선으로 바꿔 바탕쪽의 보이지 않는 선이 모든 쪽에 그려졌다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;

const SAMPLE: &[u8] = include_bytes!("../../samples/issue5789/tac_line_shape_baseline.hwpx");

fn line_count(svg: &str) -> usize {
    svg.matches("<line ").count()
}

fn set_line_type(core: &mut DocumentCore, line_type: u32) {
    let mut found = false;
    for para in &mut core.document_mut().sections[0].paragraphs {
        for control in &mut para.controls {
            if let Control::Shape(shape) = control {
                if let ShapeObject::Line(line) = shape.as_mut() {
                    let attr = &mut line.drawing.border_line.attr;
                    *attr = (*attr & !0x3F) | line_type;
                    found = true;
                }
            }
        }
    }
    assert!(found, "sample must contain a line shape");
}

#[test]
fn line_with_none_type_is_not_painted() {
    let mut solid = DocumentCore::from_bytes(SAMPLE).unwrap();
    set_line_type(&mut solid, 1);
    let solid_svg = solid.render_page_svg_native(0).unwrap();
    assert!(line_count(&solid_svg) > 0, "solid line must be painted");

    let mut none = DocumentCore::from_bytes(SAMPLE).unwrap();
    set_line_type(&mut none, 0);
    let none_svg = none.render_page_svg_native(0).unwrap();
    assert_eq!(
        line_count(&none_svg),
        line_count(&solid_svg) - 1,
        "NONE line must not be painted"
    );
}
