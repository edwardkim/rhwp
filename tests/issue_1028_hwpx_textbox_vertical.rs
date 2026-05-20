//! Task #1028 회귀 가드 — HWPX 글상자 세로쓰기 (`<hp:subList textDirection>`) 파싱
//!
//! HWPX `<hp:drawText>` 내부 `<hp:subList>` 의 `textDirection="VERTICAL"`
//! 또는 `"VERTICALALL"` 속성이 `text_box.list_attr` bit 0~2 (text_direction)
//! 으로 set 되어야 함. renderer (`shape_layout.rs:1652`) 가 이 비트로
//! 세로쓰기 분기 (`layout_vertical_textbox_text_with_paras`) 활성화.
//!
//! Refs #1028 — HWPX 글상자 세로 쓰기 미구현 (HWP5는 지원, HWPX 미적용).

use rhwp::parser::parse_document;

/// HWPX 글상자 `<hp:subList textDirection="VERTICALALL">` 가 list_attr
/// bit 0~2 에 set 되어야 함.
///
/// fixture: `samples/hwpx/tbox-v-flow-01.hwpx` — 윤동주 "서시" 세로쓰기 글상자.
#[test]
fn issue_1028_hwpx_textbox_vertical_direction_parsed() {
    use rhwp::model::control::Control;
    use rhwp::model::shape::ShapeObject;
    use std::fs;

    let bytes = fs::read("samples/hwpx/tbox-v-flow-01.hwpx")
        .expect("fixture samples/hwpx/tbox-v-flow-01.hwpx not found");
    let doc = parse_document(&bytes).expect("HWPX parse failed");

    // 세로쓰기 글상자 1개 보유 fixture
    let mut found_vertical_textbox = false;
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Shape(shape_obj) = ctrl {
                    if let ShapeObject::Rectangle(rect) = shape_obj.as_ref() {
                        if let Some(text_box) = &rect.drawing.text_box {
                            let text_direction = (text_box.list_attr & 0x07) as u8;
                            if text_direction != 0 {
                                found_vertical_textbox = true;
                                assert_eq!(
                                    text_direction, 1,
                                    "VERTICALALL 은 코드 1 (세로) 로 매핑되어야 함 \
                                     — list_attr=0x{:08x}",
                                    text_box.list_attr
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(
        found_vertical_textbox,
        "tbox-v-flow-01.hwpx 의 세로쓰기 글상자가 식별되지 않음 \
         — parse_draw_text 의 textDirection 분기 회귀 가능성"
    );
}

/// 가로 글상자 (textDirection 미설정 또는 HORIZONTAL) 는 list_attr
/// bit 0~2 가 0 으로 유지되어야 함 — 본 PR 변경의 case-specific 가드 검증.
///
/// fixture: `samples/hwpx/hy-001.hwpx` — 일반 글상자 보유 (세로쓰기 없음).
#[test]
fn issue_1028_hwpx_horizontal_textbox_unchanged() {
    use rhwp::model::control::Control;
    use rhwp::model::shape::ShapeObject;
    use std::fs;

    let bytes =
        fs::read("samples/hwpx/hy-001.hwpx").expect("fixture samples/hwpx/hy-001.hwpx not found");
    let doc = parse_document(&bytes).expect("HWPX parse failed");

    // 모든 글상자가 가로 (text_direction == 0) 여야 함
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Shape(shape_obj) = ctrl {
                    if let ShapeObject::Rectangle(rect) = shape_obj.as_ref() {
                        if let Some(text_box) = &rect.drawing.text_box {
                            let text_direction = (text_box.list_attr & 0x07) as u8;
                            assert_eq!(
                                text_direction, 0,
                                "hy-001.hwpx 의 글상자는 가로여야 함 \
                                 (textDirection 미설정 또는 HORIZONTAL) \
                                 — list_attr=0x{:08x}",
                                text_box.list_attr
                            );
                        }
                    }
                }
            }
        }
    }
}
