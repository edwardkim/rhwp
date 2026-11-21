//! 관계형 회귀 검사. 도형 캡션의 편집 주소, 캐럿·히트 주소와 저장 후 문구 보존을 대조한다.
//! 주석 갱신: 2026-11-20T12:50:49+09:00
//! 원래 실패한 경로는 도형 캡션을 글상자로 찾아 셀 주소 65534로 접근하지 못했다.
//! 같은 크기의 합성 도형과 캡션을 저장한 한 쪽은 한컴 2020 Print와 대조했다.
//! Visual Sweep 96 DPI, 색상 경계 보정 전 2px raw 실루엣은 Native·fresh WASM에서 각각
//! 100.00000%였다. 이 검사는 주소·소유·재열기 관계를 고정한다.
//! 기준 입력, 전쪽 TSV와 이미지 해시는 `mydocs/pr/assets/pr-9201/evidence.json`에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn json_usize(json: &str, key: &str) -> usize {
    let parsed: Value = serde_json::from_str(json).expect("JSON 파싱");
    parsed[key].as_u64().expect("usize 값") as usize
}

#[test]
fn shape_caption_text_uses_caption_cell_65534() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("빈 문서");
    let created = doc
        .create_shape_control_native(
            0,
            0,
            0,
            7200,
            7200,
            0,
            0,
            false,
            "TopAndBottom",
            "rectangle",
            false,
            false,
            &[],
        )
        .expect("사각형");
    let para = json_usize(&created, "paraIdx");
    let ctrl = json_usize(&created, "controlIdx");
    doc.set_shape_properties_native(
        0,
        para,
        ctrl,
        r#"{"hasCaption":true,"captionDirection":"Bottom","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850,"captionIncludeMargin":false}"#,
    )
    .expect("캡션 넣기");
    let start = doc
        .get_cell_paragraph_length_native(0, para, ctrl, 65534, 0)
        .expect("캡션 문단 길이");
    doc.insert_text_in_cell_native(0, para, ctrl, 65534, 0, start, "검토 시작")
        .expect("캡션에 글 넣기");
    let text = doc
        .get_text_in_cell_native(0, para, ctrl, 65534, 0, 0, 100)
        .expect("캡션 글 읽기");
    assert!(text.ends_with("검토 시작"), "{text}");

    // 캐럿과 누르기가 같은 캡션 셀 주소로 이어진다.
    let rect: Value = serde_json::from_str(
        &doc.get_cursor_rect_in_cell_native(0, para, ctrl, 65534, 0, start + 2)
            .expect("캡션 캐럿"),
    )
    .unwrap();
    let (x, y, h) = (
        rect["x"].as_f64().unwrap(),
        rect["y"].as_f64().unwrap(),
        rect["height"].as_f64().unwrap(),
    );
    let hit: Value = serde_json::from_str(
        &doc.hit_test_native(
            rect["pageIndex"].as_u64().unwrap() as u32,
            x + 1.0,
            y + h / 2.0,
        )
        .expect("캡션 누르기"),
    )
    .unwrap();
    assert_eq!(hit["controlIndex"].as_u64(), Some(ctrl as u64), "{hit}");
    assert_eq!(hit["cellIndex"].as_u64(), Some(65534), "{hit}");

    // Enter·Backspace로 캡션 문단을 나누고 합친다.
    doc.split_paragraph_in_cell_native(0, para, ctrl, 65534, 0, start, None)
        .expect("캡션 문단 나누기");
    assert_eq!(
        doc.get_cell_paragraph_count_native(0, para, ctrl, 65534)
            .unwrap(),
        2
    );
    doc.merge_paragraph_in_cell_native(0, para, ctrl, 65534, 1)
        .expect("캡션 문단 합치기");
    assert_eq!(
        doc.get_text_in_cell_native(0, para, ctrl, 65534, 0, 0, 100)
            .unwrap(),
        text
    );

    // 저장 후 다시 열어도 남는다.
    let reopened = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    assert_eq!(
        reopened
            .get_text_in_cell_native(0, para, ctrl, 65534, 0, 0, 100)
            .unwrap(),
        text
    );
}
