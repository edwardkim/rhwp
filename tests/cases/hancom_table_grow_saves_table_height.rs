//! 관계형·고정형: hancom_table_grow_saves_table_height의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-04T00:23:31+09:00; 직전 파일 변경 커밋: 2026-11-03T21:45:03+09:00 39b4f81f6aab4cc08c6145ed6911db2a8fe5125e.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. grow3-before/after 및 grow-nested 무편집/편집 각 1쪽 Native/fresh WASM 최저 100.00%. 저장 높이 HU는 독립 한컴 AFTER 원본에서 읽으며 live/재열기 내용과 셀 높이 관계를 검사한다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/validation.md 및 전쪽 TSV·확대 PNG.
//! 셀 내용이 넘친 표는 한컴처럼 칸 높이는 두고 표 높이(sz)만 키워 저장한다.
//! 기준 파일은 한컴에서 만든 3행 표(행 높이 10mm)와, 가운데 행에 긴 글을 넣어 저장한 결과다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

const BEFORE: &[u8] = include_bytes!("../fixtures/hancom_table_grow/grow3-before.hwpx");
const AFTER: &[u8] = include_bytes!("../fixtures/hancom_table_grow/grow3-after.hwpx");
const TEXT: &str = "가나다라마바사아자차카타파하가나다라마바사아자차카타파하";

fn table(core: &DocumentCore) -> (usize, u32, Vec<u32>) {
    let paragraph = &core.document().sections[0].paragraphs[0];
    let (index, table) = paragraph
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::Table(table) => Some((index, table)),
            _ => None,
        })
        .unwrap();
    (
        index,
        table.common.height,
        table.cells.iter().map(|cell| cell.height).collect(),
    )
}

/// 첫 행 "위"와 마지막 행 "아래"의 y 차이 = 첫 행 + 둘째 행 높이.
fn first_to_last_row(core: &DocumentCore) -> f64 {
    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    let y = |text: &str| {
        layout["runs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|run| run["text"] == text)
            .unwrap()["y"]
            .as_f64()
            .unwrap()
    };
    y("아래") - y("위")
}

#[test]
fn overflowing_cell_text_saves_the_grown_table_height_and_keeps_cell_heights() {
    let hancom = DocumentCore::from_bytes(AFTER).unwrap();
    let (_, hancom_height, hancom_cells) = table(&hancom);
    assert_eq!(hancom_height, 11748);
    assert_eq!(hancom_cells, vec![2834, 2834, 2834]);

    let mut core = DocumentCore::from_bytes(BEFORE).unwrap();
    let (control, ..) = table(&core);
    core.insert_text_in_cell_native(0, 0, control, 1, 0, 0, TEXT)
        .unwrap();
    for saved in [
        core.export_hwpx_native().unwrap(),
        core.export_hwp_with_adapter_snapshot().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&saved).unwrap();
        let (_, height, cells) = table(&reopened);
        assert!(
            height.abs_diff(hancom_height) <= 2,
            "저장 표 높이는 한컴처럼 행 높이 합이다: {height}"
        );
        assert_eq!(cells, hancom_cells, "칸 높이는 선언값을 지킨다");
        assert!((first_to_last_row(&reopened) - first_to_last_row(&hancom)).abs() < 0.2);
    }
}

#[test]
fn grown_table_reopens_with_only_the_overflowing_row_taller() {
    // 표 높이가 행 합보다 커도 넘친 행만 커지고 첫 행은 선언 높이 10mm를 지킨다.
    let before = DocumentCore::from_bytes(BEFORE).unwrap();
    let hancom = DocumentCore::from_bytes(AFTER).unwrap();
    let row = first_to_last_row(&before) / 2.0;
    let (_, total, cells) = table(&hancom);
    let second_row = f64::from(total - cells[0] - cells[2]) * 96.0 / 7200.0;
    assert!((first_to_last_row(&hancom) - (row + second_row)).abs() < 0.2);
}

#[test]
fn unchanged_table_keeps_its_saved_height() {
    let core = DocumentCore::from_bytes(BEFORE).unwrap();
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(table(&reopened).1, 8502);
}

fn nested_document() -> (DocumentCore, usize) {
    let mut core = DocumentCore::from_bytes(BEFORE).unwrap();
    let mut model = core.document().clone();
    let ci = table(&core).0;
    let Control::Table(mut child) = model.sections[0].paragraphs[0].controls[ci].clone() else {
        unreachable!()
    };
    child.common.treat_as_char = true;
    let mut parent = rhwp::model::table::Table {
        row_count: 1,
        col_count: 1,
        cells: vec![rhwp::model::table::Cell {
            width: child.common.width,
            height: child.common.height,
            paragraphs: vec![rhwp::model::paragraph::Paragraph {
                char_count: 9,
                controls: vec![Control::Table(child.clone())],
                has_para_text: true,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    parent.common.width = child.common.width;
    parent.common.height = child.common.height;
    parent.common.treat_as_char = true;
    parent.common.instance_id = child.common.instance_id.wrapping_add(1);
    parent.rebuild_grid();
    model.sections[0].paragraphs[0].controls[ci] = Control::Table(Box::new(parent));
    model.sections[0].paragraphs[0].line_segs.clear();
    model.sections[0].raw_stream = None;
    core.set_document(model);
    (core, ci)
}
fn nested_height(core: &DocumentCore, ci: usize) -> (u32, Vec<u32>) {
    let Control::Table(parent) = &core.document().sections[0].paragraphs[0].controls[ci] else {
        panic!("parent")
    };
    let Control::Table(child) = &parent.cells[0].paragraphs[0].controls[0] else {
        panic!("child")
    };
    (
        child.common.height,
        child.cells.iter().map(|c| c.height).collect(),
    )
}
#[test]
fn nested_save_keeps_unedited_height_and_only_grows_edited_content() {
    let expected = DocumentCore::from_bytes(AFTER).unwrap();
    let (_, height, cells) = table(&expected);
    for edited in [false, true] {
        let (mut core, ci) = nested_document();
        if edited {
            core.insert_text_in_cell_by_path(0, 0, &[(ci, 0, 0), (0, 1, 0)], 0, TEXT)
                .unwrap();
        }
        let original = nested_height(&core, ci);
        for bytes in [
            core.export_hwpx_native().unwrap(),
            core.export_hwp_with_adapter_snapshot().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            let actual = nested_height(&reopened, ci);
            assert_eq!(actual.1, cells, "중첩 셀의 선언 높이를 보존한다");
            if edited {
                assert!(
                    actual.0.abs_diff(height) <= 2,
                    "성장 높이 {} / 한컴 {height}",
                    actual.0
                );
            } else {
                assert_eq!(actual, original);
            }
        }
        assert_eq!(
            nested_height(&core, ci),
            original,
            "저장은 live IR을 바꾸지 않는다"
        );
    }
}

#[cfg(feature = "pdf-import")]
#[test]
fn grown_snapshot_budget_and_cancellation_preserve_live_content_and_fresh_requests_recover() {
    use rhwp::{parser::pdf::PdfImportPolicy, PdfImportSession};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    const PDF: &[u8] = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");
    let attach = |core: &mut DocumentCore| {
        *core.document_mut() = DocumentCore::from_bytes(BEFORE).unwrap().document().clone();
        let (ci, ..) = table(core);
        core.insert_text_in_cell_native(0, 0, ci, 1, 0, 0, TEXT)
            .unwrap();
    };
    let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let opened = request.resource_usage().decoded_bytes;
    attach(&mut core);
    let original = format!("{:?}", core.document());
    let expected = request.export_hwpx(&mut core, 1024 * 1024).unwrap();
    assert_eq!(format!("{:?}", core.document()), original);
    let used = request.resource_usage().decoded_bytes;
    assert!(used > opened);
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = opened + (used - opened) / 2;
    let mut limited = PdfImportSession::new(policy).unwrap();
    let mut next = limited.open(PDF, &Default::default()).unwrap();
    attach(&mut next);
    let before = format!("{:?}", next.document());
    assert!(limited.export_hwpx(&mut next, 1024 * 1024).is_err());
    assert_eq!(format!("{:?}", next.document()), before);
    let allow = Arc::new(AtomicBool::new(true));
    let flag = Arc::clone(&allow);
    let mut cancelled =
        PdfImportSession::new_with_work_guard(PdfImportPolicy::default(), move || {
            flag.load(Ordering::SeqCst)
        })
        .unwrap();
    let mut last = cancelled.open(PDF, &Default::default()).unwrap();
    attach(&mut last);
    let before = format!("{:?}", last.document());
    allow.store(false, Ordering::SeqCst);
    assert!(cancelled.export_hwpx(&mut last, 1024 * 1024).is_err());
    assert_eq!(format!("{:?}", last.document()), before);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut recovered = fresh.open(PDF, &Default::default()).unwrap();
    attach(&mut recovered);
    assert_eq!(
        fresh.export_hwpx(&mut recovered, 1024 * 1024).unwrap(),
        expected
    );
}
