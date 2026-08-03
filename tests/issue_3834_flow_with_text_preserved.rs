//! Issue #3834: HWPX→HWP5 변환이 표 `flowWithText` 를 무조건 켜 원본 `0` 을 파괴했다.
//!
//! `hwpx_to_hwp.rs::materialize_table_ctrl_header_attr` 가 `pack_common_attr_bits` 결과
//! 위에 bit 13 을 무조건 OR 했다. 파서·직렬화기는 정상이라 IR 안에서만 값이 뒤집혔고,
//! 왕복 검증(`convert --verify`)에서 `ObjectFlowWithText` 불일치로 나타났다 — HWPX
//! 1,500건 중 87건이 불일치였고 그중 80%가 이 한 필드였다.
//!
//! 판정 근거는 한글 2022 다. 같은 변환(HWPX 열어 HWP 로 저장)에서 한글은 `0` 을 그대로
//! 둔다 — 13문서 표 84개 전부 보존, 정규화 0건
//! (`tools/hangul_flowwithtext_oracle.py`). HWPX 는 이 속성을 항상
//! 명시하므로(코퍼스 119문서 표 560개, 누락 0) 파서 기본값을 메울 필요도 없다.
//!
//! 같은 서명이 #1637 cause B 였다 — HWPX 직렬화기가 `flowWithText="1"` 을 하드코딩해
//! 표 partial-split 임계를 바꿔 페이지네이션을 흔들었다. 이번 것은 그 HWP5 변환 경로판.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;

/// 공통 개체 속성의 자리차지(`flowWithText`) 비트.
const FLOW_WITH_TEXT_BIT: u32 = 0x0000_2000;

fn tables_after_convert(path: &str) -> Vec<Table> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("샘플 로드 실패 {path}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let mut doc = core.document().clone();
    convert_hwpx_to_hwp_ir(&mut doc);
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter_map(|ctrl| match ctrl {
            Control::Table(t) => Some((**t).clone()),
            _ => None,
        })
        .collect()
}

/// 원본이 `flowWithText="0"` 이면 변환 뒤에도 꺼져 있어야 한다.
#[test]
fn hwpx_to_hwp_keeps_flow_with_text_off() {
    let tables = tables_after_convert("samples/issue3834/flow_with_text_zero.hwpx");
    assert!(!tables.is_empty(), "재현본에 표가 없다");

    for (i, t) in tables.iter().enumerate() {
        assert!(
            !t.common.flow_with_text,
            "표 {i} 의 IR flow_with_text 가 켜졌다 — 재현본은 전부 자리차지 해제다"
        );
        assert_eq!(
            t.common.attr & FLOW_WITH_TEXT_BIT,
            0,
            "표 {i} 의 공통 속성 bit 13 이 켜졌다 (attr=0x{:08x}). 무조건 OR 이 살아 있으면 \
             원본 flowWithText=0 이 파괴되고, 한글이 보존하는 값과 어긋난다.",
            t.common.attr
        );
    }
}

/// 켜진 원본은 그대로 켜져 있어야 한다 — 수정이 비트를 통째로 없애지 않았음을 못박는다.
#[test]
fn hwpx_to_hwp_keeps_flow_with_text_on() {
    let tables = tables_after_convert("samples/hwpx/basic-table-01.hwpx");
    assert!(!tables.is_empty(), "표본에 표가 없다");

    for (i, t) in tables.iter().enumerate() {
        assert!(
            t.common.flow_with_text,
            "표 {i} 의 IR flow_with_text 가 꺼졌다"
        );
        assert_eq!(
            t.common.attr & FLOW_WITH_TEXT_BIT,
            FLOW_WITH_TEXT_BIT,
            "표 {i} 의 공통 속성 bit 13 이 꺼졌다 (attr=0x{:08x})",
            t.common.attr
        );
    }
}
