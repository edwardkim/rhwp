//! 글자 없는 개체 문단의 저장 사다리가 줄을 겹쳐 적으면 한글은 저장 vpos 차이만큼만 전진한다.
//!
//! 통계 보고서 요약 쪽 첫 문단은 조판부호 줄과 글상자 줄이 둘 다 lh=3403 인데 vpos 가
//! 500→600 이다(다음 문단 vpos 4003). 두 줄 상자를 통째로 쌓으면 문단이 +44px 높아져
//! 뒤 표가 밀리고 쪽 아래 표가 꼬리말에 닿았다. 문단 줄 높이는 저장 사다리의 범위
//! (100 + 3403 HU)여야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::model::shape::{CommonObjAttr, RectangleShape, ShapeObject, TextWrap};
use rhwp::DocumentCore;
use serde_json::Value;

const OBJECT_HEIGHT: i32 = 3403;

fn seg(text_start: u32, vertical_pos: i32, text_height: i32) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos,
        line_height: OBJECT_HEIGHT,
        text_height,
        baseline_distance: text_height * 85 / 100,
        segment_width: 48_000,
        ..Default::default()
    }
}

fn first_paragraph_height(value: &Value) -> Option<&Value> {
    match value {
        Value::Object(map) => {
            if map.get("kind") == Some(&Value::from("fullParagraph"))
                && map.get("paraIndex") == Some(&Value::from(0))
            {
                return map.get("height");
            }
            map.values().find_map(first_paragraph_height)
        }
        Value::Array(items) => items.iter().find_map(first_paragraph_height),
        _ => None,
    }
}

#[test]
fn overlapping_object_lines_advance_by_the_stored_vpos_step() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    // 빈 문서 첫 문단(구역·단 정의)에 글자처럼 취급한 글상자를 붙인다 — 원본처럼
    // 조판부호만 든 줄 다음에 개체 줄이 온다.
    let para = &mut doc.sections[0].paragraphs[0];
    let banner_start = 8 * para.controls.len() as u32;
    para.controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(
            RectangleShape {
                common: CommonObjAttr {
                    width: 40_000,
                    height: OBJECT_HEIGHT as u32,
                    treat_as_char: true,
                    text_wrap: TextWrap::TopAndBottom,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))));
    para.text.clear();
    para.char_offsets.clear();
    para.char_count = banner_start + 9;
    para.line_segs = vec![seg(0, 0, 100), seg(banner_start, 100, OBJECT_HEIGHT)];
    core.set_document(doc);

    let dump = core.dump_page_items_json(Some(0));
    let height = first_paragraph_height(&dump).expect("첫 문단 높이");
    let lines =
        height["lineHeightSum"].as_f64().unwrap() + height["lineSpacingSum"].as_f64().unwrap();
    let stored_span = f64::from(100 + OBJECT_HEIGHT) / 75.0;
    assert!(
        (lines - stored_span).abs() < 0.5,
        "줄 높이 합 {lines} 은 저장 사다리 범위 {stored_span} 여야 한다"
    );
}
