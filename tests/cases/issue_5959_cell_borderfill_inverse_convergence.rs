//! [#5959] 셀 테두리/배경 속성쌍 역연산의 저장 바이트 수렴 계약.
//!
//! 셀 테두리·배경 적용(`set_cell_properties` / `set_cell_zone_properties`)의 변경
//! 실체는 대상+이웃 집단의 `border_fill_id` 재배정과 스타일 테이블 append 다
//! (mydocs/working/task_m100_5959_cell_border_bg.md 판정). 이 시험은 TS 커맨드가
//! 수행하는 것과 같은 3단 복구 — ① `apply_cell_border_fill_ids` 로 id 직접 대입,
//! ② `remove_border_fill_tails` 로 push 분 절단(dirty 원복), ③ 구역 raw 저널 복원 —
//! 을 네이티브에서 그대로 재현하고, 원본 파일과의 저장 바이트 왕복 동일성을 단언한다.
//!
//! 수렴이 깨지면 역연산화는 스냅샷(전체 복원) 대체 자격을 잃는다 — 고아 BorderFill,
//! dirty 플래그 잔존, 이웃 id 미복원 중 무엇이 원인인지 first_diff 로 좁힐 수 있게
//! 스타일 테이블 길이도 함께 단정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

/// 표가 있는 .hwp 표본 후보 — 순서대로 훑어 첫 본문 최상위 표(4셀 이상)를 쓴다.
const CANDIDATES: &[&str] = &[
    "samples/156457617_240617 2024년 5월 월간 수출입 현황(확정치).hwp",
    "samples/156457624_210622 7월부터 해외직구 구매대행업체 등록제 시행.hwp",
    "samples/20250130-hongbo-no.hwp",
];

struct BodyTable {
    section: usize,
    paragraph: usize,
    control: usize,
    rows: u32,
    cols: u32,
}

fn load_with_body_table() -> (Vec<u8>, HwpDocument, BodyTable) {
    for rel in CANDIDATES {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(doc) = HwpDocument::from_bytes(&bytes) else {
            continue;
        };
        for g in extract_tables(doc.document()) {
            if !g.container_path.is_empty() {
                continue;
            }
            let Some(Control::Table(table)) = doc.document().sections[g.section].paragraphs
                [g.paragraph]
                .controls
                .get(g.control)
            else {
                continue;
            };
            if table.cells.len() < 4 || table.row_count < 2 || table.col_count < 2 {
                continue;
            }
            let info = BodyTable {
                section: g.section,
                paragraph: g.paragraph,
                control: g.control,
                rows: table.row_count as u32,
                cols: table.col_count as u32,
            };
            return (bytes, doc, info);
        }
    }
    panic!("표가 있는 본문 표본을 찾지 못했다 — CANDIDATES 를 갱신하라");
}

fn first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    let n = a.len().min(b.len());
    (0..n)
        .find(|&i| a[i] != b[i])
        .or((a.len() != b.len()).then_some(n))
}

/// 다이얼로그가 보내는 것과 같은 모양의 테두리+배경 변경 JSON.
/// 색상 값의 `"#` 이 raw string 종결자와 겹치지 않게 ## 을 쓴다.
fn border_bg_json() -> &'static str {
    r##"{"fillType":"solid","fillColor":"#FFFF00","patternColor":"#000000","patternType":0,"diagonalLine":0,"diagonalSlash":false,"diagonalBackSlash":false,"diagonalWidth":0,"diagonalColor":"#000000","centerLine":"NONE","borderLeft":{"type":1,"width":1,"color":"#000000"},"borderRight":{"type":1,"width":1,"color":"#000000"},"borderTop":{"type":1,"width":1,"color":"#000000"},"borderBottom":{"type":1,"width":1,"color":"#000000"}}"##
}

/// undo payload 조립 — execute 의 changes 를 cellIdx/beforeId 쌍으로 변환한다.
fn undo_cells(resp: &serde_json::Value) -> Vec<serde_json::Value> {
    resp["changes"]
        .as_array()
        .expect("changes 배열")
        .iter()
        .map(|c| serde_json::json!({ "cellIdx": c["cellIdx"], "id": c["beforeId"] }))
        .collect()
}

#[test]
fn cell_border_apply_undo_converges_to_original_bytes() {
    let (_original, mut doc, grid) = load_with_body_table();
    // 수렴 기준은 원본 파일 바이트가 아니라 같은 문서의 무변경 export 다 —
    // 헤더 필드 재계산 등 로드↔저장 고정 델타와 게이트를 섞지 않는다.
    let baseline = doc.export_hwp().expect("변경 전 export");
    let style_len_before = doc.document().doc_info.border_fills.len();
    // 0. 적용 **직전** 구역 raw 캡처 — TS 커맨드의 execute 첫 걸음이다.
    let capture = doc
        .capture_section_raw(grid.section as usize)
        .expect("캡처");

    // 1. 적용 — 응답의 self-describing 기록을 받는다.
    let resp: serde_json::Value = serde_json::from_str(
        &doc.set_cell_properties(
            grid.section as u32,
            grid.paragraph as u32,
            grid.control as u32,
            0,
            border_bg_json(),
        )
        .expect("적용 성공"),
    )
    .expect("응답 JSON");

    let len_before = resp["borderFillLenBefore"]
        .as_u64()
        .expect("borderFillLenBefore 가 응답에 없다 — 구버전 경로로 돌아갔다")
        as usize;
    let dirty_before = resp["docInfoDirtyBefore"].as_bool().expect("dirty 플래그");
    assert!(
        !resp["changes"].as_array().expect("changes 배열").is_empty(),
        "변경 기록이 비었다 — dedupe 수렴으로 실질 변경이 없던 표본이다"
    );
    assert!(
        doc.document().doc_info.border_fills.len() > style_len_before,
        "새 스타일이 push 됐어야 한다(기존에 동일 스타일이 있었다면 표본·JSON 을 바꿔라)"
    );

    // 2. undo ① — id 직접 대입(스타일 테이블 무손상).
    doc.apply_cell_border_fill_ids(
        grid.section as u32,
        grid.paragraph as u32,
        grid.control as u32,
        &serde_json::to_string(&serde_json::json!({ "cells": undo_cells(&resp) })).unwrap(),
    )
    .expect("id 직접 대입이 성공해야 함");

    // 3. undo ② — push 분 꼬리 절단 + dirty 원복.
    doc.remove_border_fill_tails(
        &serde_json::to_string(&serde_json::json!({
            "fromLen": len_before, "dirtyWas": dirty_before
        }))
        .unwrap(),
    )
    .expect("꼬리 절단이 성공해야 함");
    assert_eq!(
        doc.document().doc_info.border_fills.len(),
        style_len_before,
        "스타일 테이블 길이가 원복돼야 한다 — 고아 BorderFill 이 남았다"
    );

    // 4. undo ③ — 구역 raw 저널 복원(TS 커맨드와 같은 순서).
    doc.restore_section_raw(capture)
        .expect("raw 는 applyIds 가 무효화한 상태여야 한다");

    // 5. 저장 바이트 왕복 동일성.
    let after = doc.export_hwp().expect("복원 후 export");
    match first_diff(&baseline, &after) {
        None => {}
        Some(pos) => panic!(
            "undo 후 저장 바이트가 원본과 어긋난다(first_diff @{pos}) — 고아 BorderFill·dirty 잔존·이웃 미복원 중 무엇인지 확인하라"
        ),
    }
}

#[test]
fn cell_zone_apply_undo_converges_to_original_bytes() {
    let (_original, mut doc, grid) = load_with_body_table();
    let baseline = doc.export_hwp().expect("변경 전 export");
    // 적용 직전 캡처 — 위와 동일 계약.
    let capture = doc
        .capture_section_raw(grid.section as usize)
        .expect("캡처");

    // 2×2 zone 에 배경만 다른 스타일을 asOne 적용한다.
    let resp: serde_json::Value = serde_json::from_str(
        &doc.set_cell_zone_properties(
            grid.section as u32,
            grid.paragraph as u32,
            grid.control as u32,
            0,
            0,
            1,
            1,
            r##"{"fillType":"solid","fillColor":"#00FF00","patternColor":"#000000","patternType":0}"##,
        )
        .expect("zone 적용 성공"),
    )
    .expect("응답 JSON");

    let len_before = resp["borderFillLenBefore"].as_u64().expect("길이 기록") as usize;
    let dirty_before = resp["docInfoDirtyBefore"].as_bool().expect("dirty 플래그");
    let before_id = resp["zoneBeforeId"].as_u64();
    let after_id = resp["borderFillId"].as_u64().expect("적용 id");
    if before_id == Some(after_id) {
        panic!("동일 스타일 재적용이라 실질 변경이 없다 — JSON 을 바꿔라");
    }

    // undo — 신설(null)이면 zone 제거, 교체면 id 원복.
    doc.apply_cell_border_fill_ids(
        grid.section as u32,
        grid.paragraph as u32,
        grid.control as u32,
        &serde_json::to_string(&serde_json::json!({
            "zones": [{
                "startRow": 0, "startCol": 0, "endRow": 1, "endCol": 1,
                "id": before_id.map(|v| v as u32),
            }]
        }))
        .unwrap(),
    )
    .expect("zone id 원복이 성공해야 함");
    doc.remove_border_fill_tails(
        &serde_json::to_string(&serde_json::json!({
            "fromLen": len_before, "dirtyWas": dirty_before
        }))
        .unwrap(),
    )
    .expect("꼬리 절단이 성공해야 함");
    doc.restore_section_raw(capture).expect("raw 복원");

    let after = doc.export_hwp().expect("복원 후 export");
    match first_diff(&baseline, &after) {
        None => {}
        Some(pos) => panic!(
            "zone undo 후 저장 바이트가 원본과 어긋난다(first_diff @{pos}) — zone 제거·고아·raw 잔존 순으로 확인하라"
        ),
    }
}
