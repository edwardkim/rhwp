//! 첫 저장 줄의 시작 밴드는 본문 paint와 다음 단으로 이월한 OLE가 함께 소유한다.
//! 전쪽 Native/fresh WASM 대조를 마친 양형식 원본과 정상 분리 저장본을 사용한다.
//! 절대 px 좌표 대신 용지 여백·저장 기준선·개체 바깥여백의 원본 관계를 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, shape::ShapeObject};
use rhwp::paint::RenderProfile;
use std::path::Path;

fn coordinate(node: roxmltree::Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name).expect("출력 좌표").parse().unwrap()
}

#[test]
fn stored_body_and_deferred_ole_share_the_first_column_band() {
    for relative in [
        "samples/143E433F503322BD33.hwp",
        "samples/hwpx/143E433F503322BD33.hwpx",
        "samples/hwpx/143E433F503322BD33-ole.hwp",
        "samples/hwpx/143E433F503322BD33-ole.hwpx",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(core.page_count(), 1, "전쪽 독립 Print는 1쪽: {relative}");
        let section = &core.document().sections[0];
        let paper = &section.section_def.page_def;
        let first = section.paragraphs[0].line_segs.first().unwrap();
        let objects: Vec<_> = section
            .paragraphs
            .iter()
            .flat_map(|para| &para.controls)
            .filter_map(|control| match control {
                Control::Shape(shape) if matches!(shape.as_ref(), ShapeObject::Ole(_)) => {
                    Some(shape.common())
                }
                _ => None,
            })
            .collect();
        let [object] = objects.as_slice() else {
            panic!("독립 Print와 대응하는 OLE 한 개: {relative}");
        };
        let units = 96.0 / 7200.0;
        let body_top = f64::from(paper.margin_top + paper.margin_header) * units;
        let band = f64::from(first.vertical_pos) * units;
        let expected_ole_top = body_top
            + band
            + f64::from(object.vertical_offset as i32) * units
            + f64::from(object.margin.top) * units;
        let svg = core
            .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
            .unwrap();
        let xml = roxmltree::Document::parse(&svg).unwrap();
        let charts: Vec<_> = xml
            .descendants()
            .filter(|n| {
                n.has_tag_name("g")
                    && n.attribute("class")
                        .is_some_and(|s| s.split_whitespace().any(|c| c == "hwp-ole-chart"))
            })
            .collect();
        assert_eq!(charts.len(), 1, "OLE 누락·중복: {relative}");
        let origin: Vec<f64> = charts[0]
            .attribute("transform")
            .unwrap()
            .strip_prefix("translate(")
            .unwrap()
            .trim_end_matches(')')
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(origin.len(), 2);
        assert!(
            (origin[1] - expected_ole_top).abs() <= units,
            "OLE 원점은 첫 저장 밴드와 자기 바깥여백을 각각 한 번 소비한다: {relative}"
        );
        assert!(origin[0] >= f64::from(paper.margin_left) * units);
        assert!(
            origin[0] + f64::from(object.width) * units
                <= f64::from(paper.width - paper.margin_right) * units
        );
        if first.vertical_pos > 0 {
            let body = &section.paragraphs[1];
            let line = &body.line_segs[0];
            let glyph = body
                .text
                .chars()
                .find(|c| !c.is_control() && !c.is_whitespace() && *c != '\u{fffc}')
                .unwrap()
                .to_string();
            let first_glyph = xml
                .descendants()
                .find(|n| n.has_tag_name("text") && n.text() == Some(glyph.as_str()))
                .expect("원본 첫 본문 글자");
            let expected_baseline =
                body_top + f64::from(line.vertical_pos + line.baseline_distance) * units;
            assert!(
                (coordinate(first_glyph, "y") - expected_baseline).abs() <= units,
                "본문 paint도 첫 저장 밴드를 잃거나 중복하지 않는다: {relative}"
            );
        }
    }
}
