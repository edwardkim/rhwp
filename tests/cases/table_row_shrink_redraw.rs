//! 관계형: 셀 높이를 줄이면 그린 행 높이가 새 선언을 따르고, 다른 행은 제 높이를 지킨다.
//! 주석 갱신: 2026-11-23 KST. 합성 계약 검사라 한컴 PDF 시각 측정은 해당하지 않는다.
//!
//! 입력은 공개 API로 만든 글자처럼 취급한 3×2 표다. 첫 행은 한컴 양식처럼 셀 높이를 0(내용 맞춤)으로
//! 두고 글자를 넣었으며, 표 높이는 한컴 저장본처럼 그린 행 높이의 합이다. HWP·HWPX로 저장해 다시 연
//! 문서에서 마지막 행(6000HU)을 2000HU로 줄인다.
//!
//! 고치기 전에는 두 결함이 있었다.
//! - HWP: 편집 재측정이 로드 때의 행 높이를 행별 하한으로 써서 줄인 행이 6000HU로 남았다.
//! - HWPX·다시 연 HWP: 셀 높이 편집이 표 높이를 셀 높이 합(0인 행은 400HU)으로 다시 적어 첫 행이
//!   내용으로 자란 몫을 잃었다. 글자처럼 취급한 표는 그 낮은 표 높이로 다른 행을 눌러 그렸다.
//! - 병합 셀의 높이를 키우면 행은 자라지만 표 높이는 병합하지 않은 셀만 보고 다시 적어 그대로였다.
//!   글자처럼 취급한 표 뒤 문단이 옛 자리에 남아 커진 표의 아래 선에 걸쳤다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;

const PX_TOLERANCE_HU: f64 = 75.0;

fn table(doc: &HwpDocument, para: usize, control: usize) -> &Table {
    match &doc.document().sections[0].paragraphs[para].controls[control] {
        Control::Table(table) => table,
        other => panic!("표가 아님: {other:?}"),
    }
}

/// 그린 행 높이(HU). 행마다 rowSpan 1인 셀의 높이를 읽는다.
fn drawn_rows(doc: &HwpDocument, para: usize, control: usize) -> Vec<f64> {
    let json = doc
        .get_table_cell_bboxes(0, para as u32, control as u32, None)
        .expect("셀 상자");
    let cells: serde_json::Value = serde_json::from_str(&json).unwrap();
    let rows = table(doc, para, control).row_count as u64;
    (0..rows)
        .map(|row| {
            let cell = cells
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["row"].as_u64() == Some(row) && c["rowSpan"].as_u64() == Some(1))
                .expect("행의 셀");
            cell["h"].as_f64().unwrap() * 75.0
        })
        .collect()
}

fn cells_in_row(doc: &HwpDocument, para: usize, control: usize, row: u16) -> Vec<usize> {
    table(doc, para, control)
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.row == row)
        .map(|(idx, _)| idx)
        .collect()
}

/// 첫 행이 내용 맞춤(0)이고 표 높이가 그린 행 합인 글자처럼 취급한 표를 HWP·HWPX로 저장한다.
fn saved_sources() -> (Vec<(&'static str, Vec<u8>)>, usize, usize) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let created: serde_json::Value = serde_json::from_str(
        &doc.create_table_ex_native(0, 0, 0, 3, 2, true, None, Some(&[1000, 2000, 6000]))
            .unwrap(),
    )
    .unwrap();
    let para = created["paraIdx"].as_u64().unwrap() as usize;
    let control = created["controlIdx"].as_u64().unwrap() as usize;
    for cell in cells_in_row(&doc, para, control, 0) {
        doc.insert_text_in_cell_native(0, para, control, cell, 0, 0, "제목 칸")
            .unwrap();
        doc.set_cell_properties_native(0, para, control, cell, r#"{"height":0}"#)
            .unwrap();
    }
    // 한컴 저장본처럼 표 높이는 내용으로 자란 첫 행과 나머지 행 선언의 합이다.
    let title_row = drawn_rows(&doc, para, control)[0];
    let height = {
        let table = match &mut doc.document_mut().sections[0].paragraphs[para].controls[control] {
            Control::Table(table) => table,
            _ => unreachable!(),
        };
        table.common.height = title_row.round() as u32 + 2000 + 6000;
        // HWP CommonObjAttr에서 높이는 16..20바이트다. 저장은 이 원본 헤더를 쓴다.
        if let Some(raw) = table.raw_ctrl_data.get_mut(16..20) {
            raw.copy_from_slice(&table.common.height.to_le_bytes());
        }
        table.common.height
    };
    assert!(
        height
            > table(&doc, para, control)
                .get_row_heights()
                .iter()
                .sum::<u32>(),
        "첫 행이 내용으로 자라 표 높이가 셀 높이 합보다 크다"
    );
    (
        vec![
            ("hwp", doc.export_hwp().unwrap()),
            ("hwpx", doc.export_hwpx().unwrap()),
        ],
        para,
        control,
    )
}

fn assert_rows(label: &str, actual: &[f64], expected: &[f64]) {
    for (row, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a - e).abs() <= PX_TOLERANCE_HU,
            "{label}: {row}행을 {e}HU로 그려야 하는데 {a}HU다 (전체 {actual:?})"
        );
    }
}

fn shrink_redraws(resize: fn(&mut HwpDocument, usize, usize, &[usize], u32)) {
    let (sources, para, control) = saved_sources();
    for (format, bytes) in sources {
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        let before = drawn_rows(&doc, para, control);
        assert_rows(&format!("{format} 열기"), &before[2..], &[6000.0]);
        let last = cells_in_row(&doc, para, control, 2);

        resize(&mut doc, para, control, &last, 2000);
        let expected = [before[0], before[1], 2000.0];
        assert_rows(
            &format!("{format} 줄인 뒤"),
            &drawn_rows(&doc, para, control),
            &expected,
        );

        let saved = if format == "hwp" {
            doc.export_hwp().unwrap()
        } else {
            doc.export_hwpx().unwrap()
        };
        let reopened = HwpDocument::from_bytes(&saved).unwrap();
        assert_rows(
            &format!("{format} 다시 연 뒤"),
            &drawn_rows(&reopened, para, control),
            &expected,
        );

        // 키우는 편집도 그대로 새 선언을 그린다.
        resize(&mut doc, para, control, &last, 8000);
        assert_rows(
            &format!("{format} 키운 뒤"),
            &drawn_rows(&doc, para, control),
            &[before[0], before[1], 8000.0],
        );
    }
}

#[test]
fn cell_properties_height_shrink_redraws_row() {
    shrink_redraws(|doc, para, control, cells, height| {
        for &cell in cells {
            doc.set_cell_properties_native(
                0,
                para,
                control,
                cell,
                &format!(r#"{{"height":{height}}}"#),
            )
            .unwrap();
        }
    });
}

#[test]
fn resize_table_cells_shrink_redraws_row() {
    shrink_redraws(|doc, para, control, cells, height| {
        let current = table(doc, para, control).cells[cells[0]].height;
        let delta = height as i64 - current as i64;
        let updates: Vec<String> = cells
            .iter()
            .map(|cell| format!(r#"{{"cellIdx":{cell},"heightDelta":{delta}}}"#))
            .collect();
        doc.resize_table_cells(
            0,
            para as u32,
            control as u32,
            &format!("[{}]", updates.join(",")),
        )
        .expect("셀 크기 조절");
    });
}

/// 표 아래 선과 뒤 문단 캐럿 위 끝 사이 거리(px).
fn gap_below_table(doc: &HwpDocument, para: usize) -> f64 {
    let bbox: serde_json::Value =
        serde_json::from_str(&doc.get_table_bbox(0, para as u32, 0).expect("표 상자")).unwrap();
    let next: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect_native(0, para + 1, 0).unwrap()).unwrap();
    next["y"].as_f64().unwrap() - bbox["y"].as_f64().unwrap() - bbox["height"].as_f64().unwrap()
}

#[test]
fn growing_a_merged_cell_moves_the_following_paragraph() {
    // 앞 문단 · 글자처럼 취급한 4×3 표 · 뒤 문단. 셋째 칸을 네 줄로 합쳐 사진 칸(40mm)으로 키운다.
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "앞 문단").unwrap();
    doc.split_paragraph_native(0, 0, 4, None).unwrap();
    doc.split_paragraph_native(0, 1, 0, None).unwrap();
    doc.insert_text_native(0, 2, 0, "뒤 문단").unwrap();
    doc.create_table_ex_native(0, 1, 0, 4, 3, true, None, None)
        .unwrap();
    doc.merge_table_cells_native(0, 1, 0, 0, 2, 3, 2).unwrap();
    let gap = gap_below_table(&doc, 1);
    let merged = table(&doc, 1, 0)
        .cells
        .iter()
        .position(|cell| cell.row == 0 && cell.col == 2)
        .unwrap();
    let before = drawn_rows(&doc, 1, 0).iter().sum::<f64>();

    doc.set_cell_properties_native(0, 1, 0, merged, r#"{"height":11339}"#)
        .unwrap();
    let grown = drawn_rows(&doc, 1, 0).iter().sum::<f64>();
    assert!(
        grown > before + 5000.0,
        "병합 셀이 걸친 행을 키운다: {before} → {grown}"
    );
    let live = gap_below_table(&doc, 1);
    assert!(
        (live - gap).abs() <= 1.0,
        "편집 직후: 뒤 문단은 커진 표 아래에 같은 거리({gap}px)로 놓여야 하는데 {live}px다"
    );
    for (label, bytes) in [
        ("HWP 다시 연 뒤", doc.export_hwp().unwrap()),
        ("HWPX 다시 연 뒤", doc.export_hwpx().unwrap()),
    ] {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap();
        let after = gap_below_table(&reopened, 1);
        assert!(
            (after - gap).abs() <= 1.0,
            "{label}: 뒤 문단은 커진 표 아래에 같은 거리({gap}px)로 놓여야 하는데 {after}px다"
        );
    }
}
