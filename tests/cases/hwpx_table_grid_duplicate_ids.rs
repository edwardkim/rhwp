//! 관계형·고정형: 중복 표 ID와 빈 논리 행의 저장·재열기·실패 원자성을 검사한다.
//! 주석 갱신: 2026-11-03T20:37:08+09:00; 직전 파일 커밋: 2026-10-26T23:08:07+09:00 e1012b59f6ed5e0e8823de2154d471852dfc0b7a.
//! 시각 측정 비해당: 합성 IR의 ID·행 수·직렬화 바이트·자원 정책을 검사하며 실제 배치 좌표를 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table};

fn table(rows: u16, id: u32) -> Table {
    let mut cell = Cell {
        row_span: rows,
        col_span: 1,
        width: 10_000,
        height: 1000 * u32::from(rows),
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    };
    cell.col = 0;
    cell.row = 0;
    let mut table = Table {
        row_count: rows,
        col_count: 1,
        cells: vec![cell],
        ..Default::default()
    };
    table.common.instance_id = id;
    table.common.width = 10_000;
    table.common.height = 1000 * u32::from(rows);
    table.common.treat_as_char = true;
    table.rebuild_grid();
    table
}

#[test]
fn duplicate_table_ids_keep_empty_logical_rows_in_hwpx() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    let host = |table: Table| Paragraph {
        char_count: 9,
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    };
    // 둘째 행에 셀 시작이 없는 표와, 같은 ID를 가진 일반 표.
    document.sections[0].paragraphs.push(host(table(2, 7)));
    document.sections[0].paragraphs.push(host(table(1, 7)));
    core.set_document(document);
    let bytes = core
        .export_hwpx_native()
        .expect("중복 ID 표도 HWPX로 저장한다");
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    let rows: Vec<u16> = reopened.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Table(table) => Some(table.row_count),
            _ => None,
        })
        .collect();
    assert_eq!(rows, vec![2, 1]);
}

fn core_with_tables(tables: Vec<Table>) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for table in tables {
        document.sections[0].paragraphs.push(Paragraph {
            char_count: 9,
            controls: vec![Control::Table(Box::new(table))],
            ..Default::default()
        });
    }
    core.set_document(document);
    core
}
fn table_rows_ids(core: &DocumentCore) -> Vec<(u16, u32)> {
    fn walk(paragraphs: &[Paragraph], out: &mut Vec<(u16, u32)>) {
        for p in paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    out.push((t.row_count, t.common.instance_id));
                    for cell in &t.cells {
                        walk(&cell.paragraphs, out);
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for s in &core.document().sections {
        walk(&s.paragraphs, &mut out);
    }
    out
}
#[test]
fn reassignment_preserves_later_original_ids_and_nested_grids() {
    let mut later = table(1, u32::MAX);
    later.cells[0].paragraphs[0]
        .controls
        .push(Control::Table(Box::new(table(2, 0))));
    later.cells[0].paragraphs[0].char_count = 9;
    let core = core_with_tables(vec![table(2, 7), table(1, 7), later]);
    let before = table_rows_ids(&core);
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    let actual = table_rows_ids(&reopened);
    assert_eq!(
        actual.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![2, 1, 1, 2]
    );
    assert_eq!(actual[0].1, 7);
    assert_eq!(actual[2].1, u32::MAX);
    assert_eq!(actual[3].1, 0);
    assert_ne!(actual[1].1, 7);
    assert_ne!(actual[1].1, u32::MAX);
    assert_eq!(
        table_rows_ids(&core),
        before,
        "저장은 live ID를 바꾸지 않는다"
    );
    let again = DocumentCore::from_bytes(&reopened.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(table_rows_ids(&again), actual);
}
#[test]
fn zero_ids_are_separated_only_where_row_metadata_needs_identity() {
    let core = core_with_tables(vec![table(2, 0), table(2, 0), table(1, 0)]);
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    let actual = table_rows_ids(&reopened);
    assert_eq!(
        actual.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![2, 2, 1]
    );
    assert_eq!(
        actual
            .iter()
            .map(|x| x.1)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3
    );
    let plain = core_with_tables(vec![table(1, 0), table(1, 0)]);
    let reopened = DocumentCore::from_bytes(&plain.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(table_rows_ids(&reopened), vec![(1, 0), (1, 0)]);
}
#[cfg(feature = "pdf-import")]
#[test]
fn duplicate_grid_export_obeys_request_budget_preserves_live_ir_and_recovers() {
    use rhwp::{parser::pdf::PdfImportPolicy, PdfImportSession};
    const PDF: &[u8] = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");
    fn attach(core: &mut DocumentCore) {
        for _ in 0..64 {
            core.document_mut().sections[0].paragraphs.push(Paragraph {
                char_count: 9,
                controls: vec![Control::Table(Box::new(table(2, 0)))],
                ..Default::default()
            });
        }
    }
    let mut healthy = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = healthy.open(PDF, &Default::default()).unwrap();
    attach(&mut core);
    let before = table_rows_ids(&core);
    let opened = healthy.resource_usage().decoded_bytes;
    let expected = core.export_hwpx_native().unwrap();
    assert_eq!(
        healthy.export_hwpx(&mut core, 1024 * 1024).unwrap(),
        expected
    );
    assert_eq!(table_rows_ids(&core), before);
    let used = healthy.resource_usage().decoded_bytes;
    assert!(used > opened);
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = opened + (used - opened) / 2;
    let mut limited = PdfImportSession::new(policy).unwrap();
    let mut next = limited.open(PDF, &Default::default()).unwrap();
    attach(&mut next);
    let original = table_rows_ids(&next);
    assert!(limited.export_hwpx(&mut next, 1024 * 1024).is_err());
    assert_eq!(table_rows_ids(&next), original);
    let failed = limited.resource_usage();
    assert!(failed.failure.is_some());
    assert!(limited.export_hwpx(&mut next, 1024 * 1024).is_err());
    assert_eq!(limited.resource_usage().decoded_bytes, failed.decoded_bytes);
    assert_eq!(next.export_hwpx_native().unwrap(), expected);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut recovered = fresh.open(PDF, &Default::default()).unwrap();
    attach(&mut recovered);
    assert_eq!(
        fresh.export_hwpx(&mut recovered, 1024 * 1024).unwrap(),
        expected
    );
}

#[cfg(feature = "pdf-import")]
#[test]
fn cancelled_grid_save_keeps_live_model_and_a_new_request_recovers() {
    use rhwp::{parser::pdf::PdfImportPolicy, PdfImportSession};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    const PDF: &[u8] = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");
    let allow = Arc::new(AtomicBool::new(true));
    let owner = Arc::clone(&allow);
    let mut request =
        PdfImportSession::new_with_work_guard(PdfImportPolicy::default(), move || {
            owner.load(Ordering::SeqCst)
        })
        .unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    for _ in 0..8 {
        core.document_mut().sections[0].paragraphs.push(Paragraph {
            char_count: 9,
            controls: vec![Control::Table(Box::new(table(2, 0)))],
            ..Default::default()
        });
    }
    let model = core.document().clone();
    let expected = core.export_hwpx_native().unwrap();
    let ids = table_rows_ids(&core);
    allow.store(false, Ordering::SeqCst);
    assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(table_rows_ids(&core), ids);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
    let used = request.resource_usage().decoded_bytes;
    allow.store(true, Ordering::SeqCst);
    assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, used);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut next = fresh.open(PDF, &Default::default()).unwrap();
    *next.document_mut() = model;
    assert_eq!(fresh.export_hwpx(&mut next, 1024 * 1024).unwrap(), expected);
}
