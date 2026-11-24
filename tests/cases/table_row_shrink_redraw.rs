//! 관계형 검사. 셀 높이 편집 뒤 행의 점유 높이와 표 뒤 문단의 간격을 두 편집 경로와 두 저장 형식에서 대조한다.
//! 2026-11-24 15:55 KST 갱신. 직전 파일 변경은 2026-11-23 09:51:59 KST의 a6c7d1da185741b940e3dccf221376ff47c3c171이다.
//! 2026-11-24 한컴 Print, 96 DPI의 보정 전 2px 실루엣으로 측정했다. 병합 셀 편집본 4종은 각각 1쪽이며
//! 전쪽 Native와 fresh WASM 모두 100.00%였다. 용역표준계약서 원본 및 HWP·HWPX 편집본은 각각 2쪽이며
//! 세 문서 각각의 전쪽 최저는 Native 99.38%, fresh WASM 99.35%다. 표를 줄인 2쪽은 두 경로 모두 100.00%다.
//! 쪽별 수치와 입력·인쇄 해시는 `mydocs/pr/assets/semanticist21-9337-9354-9377/focus-pages.tsv`와 같은 디렉터리의 `sources-and-print.tsv`에 있다.
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

type HeightEdit = fn(&mut HwpDocument, usize, usize, &[usize], u32);

fn set_cell_height(
    doc: &mut HwpDocument,
    para: usize,
    control: usize,
    cells: &[usize],
    height: u32,
) {
    let value = serde_json::json!({"height": height}).to_string();
    for &index in cells {
        doc.set_cell_properties_native(0, para, control, index, &value)
            .unwrap();
    }
}

fn resize_cell_height(
    doc: &mut HwpDocument,
    para: usize,
    control: usize,
    cells: &[usize],
    height: u32,
) {
    let table = table(doc, para, control);
    let updates: Vec<_> = cells
        .iter()
        .map(|&index| {
            let delta = i64::from(height) - i64::from(table.cells[index].height);
            serde_json::json!({"cellIdx": index, "heightDelta": delta})
        })
        .collect();
    doc.resize_table_cells(
        0,
        para as u32,
        control as u32,
        &serde_json::to_string(&updates).unwrap(),
    )
    .expect("셀 크기 조절");
}

const HEIGHT_EDITS: [HeightEdit; 2] = [set_cell_height, resize_cell_height];

fn shrink_redraws(resize: HeightEdit) {
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
    shrink_redraws(set_cell_height);
}

#[test]
fn resize_table_cells_shrink_redraws_row() {
    shrink_redraws(resize_cell_height);
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

    let source = doc.export_hwpx().unwrap();
    for edit in HEIGHT_EDITS {
        let mut doc = HwpDocument::from_bytes(&source).unwrap();
        edit(&mut doc, 1, 0, &[merged], 11_339);
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
}

#[test]
fn shrinking_contract_rows_keeps_the_table_at_its_host_line() {
    let source = include_bytes!("../fixtures/issue9337/contract-appendix9.hwp");
    for edit in HEIGHT_EDITS {
        let mut doc = HwpDocument::from_bytes(source).expect("용역표준계약서");
        let (para, control) = (1usize, 0usize);
        let row = 3;
        let changed = cells_in_row(&doc, para, control, row);
        assert_eq!(changed.len(), 3);
        let bounds = |doc: &HwpDocument| {
            let bbox: serde_json::Value = serde_json::from_str(
                &doc.get_table_bbox(0, para as u32, control as u32)
                    .expect("표 상자"),
            )
            .unwrap();
            (
                bbox["pageIndex"].as_u64().unwrap(),
                bbox["y"].as_f64().unwrap(),
                bbox["height"].as_f64().unwrap(),
            )
        };
        let (page, top, old_height) = bounds(&doc);
        assert!(changed
            .iter()
            .all(|&index| table(&doc, para, control).cells[index].height > 2_000));
        edit(&mut doc, para, control, &changed, 2_000);
        let check_position = |doc: &HwpDocument| {
            let (after_page, new_top, height) = bounds(doc);
            assert_eq!(after_page, page);
            assert!(
                (top - new_top).abs() <= 1.0,
                "표가 놓인 글줄 {top} → {new_top}"
            );
            assert!(height < old_height, "편집한 표는 줄어야 한다");
            assert!(
                (drawn_rows(doc, para, control)[usize::from(row)] - 2000.0).abs()
                    <= PX_TOLERANCE_HU
            );
        };
        check_position(&doc);
        for saved in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
            check_position(&HwpDocument::from_bytes(&saved).unwrap());
        }
    }
}
