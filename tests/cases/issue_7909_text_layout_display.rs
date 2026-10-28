//! #7909: 텍스트 레이아웃의 표시 문자열은 모델 오프셋을 대체하지 않는다.
//! 조판/paint 기대값이 아닌 읽기 전용 export 계약 검사다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn layout(core: &DocumentCore) -> Vec<Value> {
    let value: Value = serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    value["runs"].as_array().unwrap().clone()
}

fn document(field: u8) -> HwpDocument {
    let mut core = HwpDocument::create_empty();
    core.create_blank_document_native().unwrap();
    core.set_file_name("보고서\"초안\\.hwp");
    core.create_header_footer_native(0, true, 0).unwrap();
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "AB")
        .unwrap();
    core.insert_field_in_hf_native(0, true, 0, 0, 0, field)
        .unwrap();
    core
}

#[test]
fn generated_display_is_separate_from_model_text_and_positions() {
    for (field, display, marker) in [
        (1, "1", '\u{0015}'),
        (2, "1", '\u{0016}'),
        (3, "보고서\"초안\\.hwp", '\u{0017}'),
    ] {
        let core = document(field);
        let runs = layout(&core);
        let run = runs
            .iter()
            .find(|run| run["text"] == marker.to_string())
            .expect("모델 필드 런");
        assert_eq!(run["displayText"], display);
        assert_eq!(run["charX"].as_array().unwrap().len(), 2, "모델 1자 경계");
        let positions = run["displayCharX"].as_array().expect("표시 경계");
        assert_eq!(positions.len(), display.chars().count() + 1);
        assert_eq!(positions[0], 0.0);
        assert!(positions.last().unwrap().as_f64().unwrap() > 0.0);
        assert_eq!(run["charStart"], 0);
        assert!(run["x"].as_f64().unwrap().is_finite());
        assert!(run["y"].as_f64().unwrap().is_finite());
        let plain = runs
            .iter()
            .find(|run| run["text"] == "AB")
            .expect("필드 뒤 문구");
        assert_eq!(plain["charStart"], 1);
        assert!(plain.get("displayText").is_none());
        assert!(plain.get("displayCharX").is_none());
    }
}

#[test]
fn export_preserves_selection_cursor_and_edit_offsets() {
    let mut core = document(3);
    let selection = core
        .get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 0, 0, 1)
        .unwrap();
    let cursor = core
        .get_cursor_rect_in_header_footer_native(0, true, 0, 0, 1, 0)
        .unwrap();
    let hit = core
        .hit_test_in_header_footer_native(0, true, 5000.0, 0.0)
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&hit).unwrap()["charOffset"],
        3
    );
    let _ = layout(&core);
    assert_eq!(
        core.get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 0, 0, 1)
            .unwrap(),
        selection
    );
    assert_eq!(
        core.get_cursor_rect_in_header_footer_native(0, true, 0, 0, 1, 0)
            .unwrap(),
        cursor
    );
    assert_eq!(
        core.hit_test_in_header_footer_native(0, true, 5000.0, 0.0)
            .unwrap(),
        hit
    );
    core.insert_text_in_header_footer_native(0, true, 0, 0, 1, "X")
        .unwrap();
    let content: Value =
        serde_json::from_str(&core.get_header_footer_native(0, true, 0).unwrap()).unwrap();
    assert_eq!(content["text"], "\u{0017}XAB");
}

#[test]
fn auto_number_hwpx_and_hwp_keep_display_and_model_spaces() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6986/cell-page-and-total-page-in-one-run.hwpx"),
    )
    .unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    for bytes in [bytes, core.export_hwp_with_adapter().unwrap()] {
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        let runs = layout(&core);
        let displays: String = runs
            .iter()
            .map(|run| {
                run["displayText"]
                    .as_str()
                    .or_else(|| run["text"].as_str())
                    .unwrap()
            })
            .collect();
        assert!(
            displays.replace(' ', "").starts_with("-1/1-"),
            "{displays:?}"
        );
        assert!(
            runs.iter().any(|run| run.get("displayText").is_some()),
            "자동번호 표시값"
        );
    }
}

#[test]
fn paragraph_without_auto_number_keeps_legacy_fields() {
    let mut core = HwpDocument::create_empty();
    core.create_blank_document_native().unwrap();
    core.create_header_footer_native(0, true, 0).unwrap();
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "일반 문구 AB")
        .unwrap();
    let runs = layout(&core);
    let text: String = runs
        .iter()
        .map(|run| run["text"].as_str().unwrap())
        .collect();
    assert!(text.contains("일반 문구 AB"));
    assert!(runs
        .iter()
        .all(|run| run.get("displayText").is_none() && run.get("displayCharX").is_none()));
}

fn assert_public_footer(name: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue7909")
        .join(name);
    let original = std::fs::read(path).expect("공개 원본 필수");
    let core = DocumentCore::from_bytes(&original).unwrap();
    // 한컴 원본 HWP와 구분하는 rhwp 생성 HWPX 대조다.
    for bytes in [original, core.export_hwpx_native().unwrap()] {
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        let runs = layout(&core);
        let display: String = runs
            .iter()
            .map(|r| {
                r["displayText"]
                    .as_str()
                    .or_else(|| r["text"].as_str())
                    .unwrap()
            })
            .collect();
        assert!(
            display.contains("ABCDEF양식 Version 0.0[0000-00-05]"),
            "{name}: 일반 꼬리말 문구 보존"
        );
        let generated: Vec<_> = runs
            .iter()
            .filter(|r| {
                r.get("displayText")
                    .is_some_and(|v| v.as_str().unwrap().contains('1'))
            })
            .collect();
        assert!(!generated.is_empty(), "{name}: 자동 쪽번호 표시 누락");
        for run in generated {
            assert_eq!(
                run["displayCharX"].as_array().unwrap().len(),
                run["displayText"].as_str().unwrap().chars().count() + 1
            );
            assert!(run["x"].as_f64().unwrap().is_finite());
            assert!(run["y"].as_f64().unwrap().is_finite());
        }
    }
}

#[test]
fn public_footer_with_picture_exports_page_number() {
    assert_public_footer("repro_footer_picture.hwp");
}

#[test]
fn public_no_picture_control_has_no_auto_number_and_keeps_plain_text() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue7909/control_footer_nopicture.hwp");
    let bytes = std::fs::read(path).unwrap();
    let parsed = rhwp::parser::parse_document(&bytes).unwrap();
    let footer = parsed.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Footer(hf) = c {
                Some(hf)
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        footer.paragraphs[0].controls.is_empty(),
        "공개 대조본에는 atno도 없다"
    );
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let runs = layout(&core);
    assert!(runs.iter().all(|r| r.get("displayText").is_none()));
    let text: String = runs.iter().map(|r| r["text"].as_str().unwrap()).collect();
    assert!(text.contains("ABCDEF양식 Version 0.0[0000-00-05]"));
}

/// 공개 원본의 선행 그림 2개만 제거한다. HWP 확장 컨트롤은 각각 8 UTF-16
/// 유닛이므로 선행 슬롯도 거둔다. atno 위치와 일반 문자열은 유지한다.
/// 한컴 생성 원본이 아닌, 생성 절차가 명시된 최소 HWPX 대조다.
fn no_picture_with_auto_number() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue7909/repro_footer_picture.hwp");
    let mut document = rhwp::parser::parse_document(&std::fs::read(path).unwrap()).unwrap();
    let footer = document.sections[0].paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|c| {
            if let Control::Footer(hf) = c {
                Some(hf)
            } else {
                None
            }
        })
        .unwrap();
    let para = &mut footer.paragraphs[0];
    let positions = para.control_text_positions();
    let removed = para
        .controls
        .iter()
        .take_while(|c| matches!(c, Control::Shape(_) | Control::Picture(_)))
        .count();
    assert_eq!(removed, 2, "공개 원본의 선행 도형·그림");
    assert!(positions[..removed].iter().all(|p| *p == 0));
    para.controls.drain(..removed);
    assert_eq!(para.controls.len(), 1);
    assert!(matches!(para.controls[0], Control::AutoNumber(_)));
    para.ctrl_data_records = vec![None];
    let released = u32::try_from(removed).unwrap() * rhwp::model::control::CTRL_CHAR_CODE_UNITS;
    for offset in &mut para.char_offsets {
        *offset = offset.checked_sub(released).unwrap();
    }
    let first_kept = para
        .char_shapes
        .iter()
        .rposition(|cs| cs.start_pos <= released)
        .unwrap();
    para.char_shapes.drain(..first_kept);
    for cs in &mut para.char_shapes {
        cs.start_pos = cs.start_pos.saturating_sub(released);
    }
    para.char_count = para.char_count.checked_sub(released).unwrap();
    para.line_segs.clear();
    para.source_line_seg_vertical_pos = None;
    para.layout_only_fill_lines = 0;
    para.invalidate_layout_inputs();
    rhwp::serializer::serialize_hwpx(&document).unwrap()
}

#[test]
fn generated_no_picture_keeps_auto_number_in_hwpx_and_hwp() {
    let hwpx = no_picture_with_auto_number();
    let core = DocumentCore::from_bytes(&hwpx).unwrap();
    for bytes in [hwpx, core.export_hwp_with_adapter().unwrap()] {
        let parsed = rhwp::parser::parse_document(&bytes).unwrap();
        let footer = parsed.sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|c| {
                if let Control::Footer(hf) = c {
                    Some(hf)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(footer.paragraphs[0].controls.len(), 1);
        assert!(matches!(
            footer.paragraphs[0].controls[0],
            Control::AutoNumber(_)
        ));
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        let runs = layout(&core);
        let run = runs
            .iter()
            .find(|r| r["text"].as_str().unwrap().contains("ABCDEF"))
            .unwrap();
        assert!(
            run["displayText"].as_str().is_some_and(|t| t.contains('1')),
            "그림 없는 atno 표시"
        );
        assert_eq!(
            run["displayCharX"].as_array().unwrap().len(),
            run["displayText"].as_str().unwrap().chars().count() + 1
        );
    }
}
