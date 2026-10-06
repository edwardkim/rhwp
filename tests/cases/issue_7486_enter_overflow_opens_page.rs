//! [#7486] 새 문서에서 Enter 만 눌러 본문이 1쪽을 넘치면 2쪽이 생겨야 한다.
//!
//! 끝 쪽의 빈 문단만 남은 쪽을 버리는 마무리(`discard_terminal_blank_only_page`)가
//! 편집 흐름이 넘쳐 연 쪽까지 버리면, 새 문단이 어느 쪽에도 속하지 않아 캐럿이
//! 2쪽으로 가지 못한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

#[test]
fn enter_past_page_end_opens_second_page() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native()
        .expect("create blank document");
    assert_eq!(doc.page_count(), 1);

    // 기본 A4 본문은 10pt 빈 줄 40여 개를 담는다. 60번이면 반드시 넘친다.
    const SPLITS: usize = 60;
    for para in 0..SPLITS {
        doc.split_paragraph_native(0, para, 0, None)
            .expect("split paragraph");
    }

    assert_eq!(doc.page_count(), 2);
    doc.get_cursor_rect_native(0, SPLITS, 0)
        .expect("넘친 새 문단은 2쪽에 놓여야 한다");
}

// A4 기본 본문 높이 약 65,760 HU에 대해 10pt(1,000 HU) 빈 줄의
// 200%/300% 전진은 각각 2,000/3,000 HU다. 33/22번째 새 줄의 시작은
// 66,000 HU이므로 본문 밖이며, 마지막 빈 문단 흡수로 소유를 잃으면 안 된다.
fn assert_spacing_boundary(spacing: u32, boundary: usize) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.apply_para_format_native(
        0,
        0,
        &format!(r#"{{"lineSpacing":{spacing},"lineSpacingType":"Percent"}}"#),
    )
    .unwrap();
    for para in 0..boundary {
        doc.split_paragraph_native(0, para, 0, None).unwrap();
        let rect: serde_json::Value =
            serde_json::from_str(&doc.get_cursor_rect_native(0, para + 1, 0).unwrap()).unwrap();
        if para + 1 == boundary {
            assert_eq!(doc.page_count(), 2, "spacing={spacing}, Enter={boundary}");
            assert_eq!(rect["pageIndex"].as_u64(), Some(1));
            // キャレットの行上端は新しい本文先頭(20mm + 15mm)に置く。
            let body_top = (20.0 + 15.0) * 96.0 / 25.4;
            assert!((rect["y"].as_f64().unwrap() - body_top).abs() < 1.0);
            assert!((rect["x"].as_f64().unwrap() - 30.0 * 96.0 / 25.4).abs() < 1.0);
        }
    }
    // 저장 후에도 같은 입력을 재사용한다. 세션 전용 플래그로만 고치지 않는다.
    let bytes = doc.export_hwpx_native().unwrap();
    let reopened = HwpDocument::from_bytes(&bytes).unwrap();
    assert_eq!(reopened.page_count(), 2, "저장 후에도 새 빈 쪽을 보존");
    for para in 0..=boundary {
        reopened.get_cursor_rect_native(0, para, 0).unwrap();
    }
}

#[test]
fn spacing_200_enter_boundary_keeps_caret_owner() {
    assert_spacing_boundary(200, 33);
}

#[test]
fn spacing_300_enter_boundary_keeps_caret_owner() {
    assert_spacing_boundary(300, 22);
}

#[test]
fn repeated_enter_preserves_line_boxes_across_spacing_and_pages() {
    for spacing in [100, 130, 160, 180, 200, 300] {
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().unwrap();
        doc.apply_para_format_native(
            0,
            0,
            &format!(r#"{{"lineSpacing":{spacing},"lineSpacingType":"Percent"}}"#),
        )
        .unwrap();
        for para in 0..90 {
            doc.split_paragraph_native(0, para, 0, None).unwrap();
            let rect: serde_json::Value = serde_json::from_str(
                &doc.get_cursor_rect_native(0, para + 1, 0)
                    .unwrap_or_else(|e| panic!("spacing={spacing}, Enter={}: {e}", para + 1)),
            )
            .unwrap();
            // 편집 가능한 빈 줄 상자 전체가 A4 본문 안에 들어가야 한다.
            let body_top = (20.0 + 15.0) * 96.0 / 25.4;
            let body_bottom = (297.0 - 15.0 - 15.0) * 96.0 / 25.4;
            let y = rect["y"].as_f64().unwrap();
            let height = rect["height"].as_f64().unwrap();
            assert!(
                y >= body_top - 1.0 && y + height <= body_bottom + 1.0,
                "spacing={spacing}, Enter={}, rect={rect}",
                para + 1
            );
        }
    }
}
