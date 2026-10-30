//! [Issue #8382] HWP5 출처 HWPX 의 표 분할 예산 — 쪽 나눔 측정기도 저장 첫 조각 행 상자를 쓴다.
//!
//! 결함: HWP5 출처 HWPX 에서 쪽 나눔 측정기가 `native_hwp5_layout()`(HWP5 원본만 참)을 넘겨
//! 저장 첫 조각 행 상자를 쓰지 않았다. 배치 경로는 `hwp5_stored_pagination_layout()` 을 쓰므로
//! 원본 HWP 와 같은 문서인데도 쪽 끝 표 행을 행 안에서 자르거나 다음 쪽으로 넘겼다.
//!
//! 독립 기준: 각 원본의 한컴 Print PDF 쪽수와, 같은 문서인 원본 HWP 의 쪽 구성.
//! head 의 HWPX 저장본 Native Visual Sweep(2px 관용 실루엣, TSV 4열) 전쪽 최저:
//! headerrow-2020 31쪽 100.00% · reservefit-2020 65쪽 100.00% · rowfit-2020 33쪽 100.00% ·
//! 71352 48쪽 96.91%. base 는 각각 33.32% · 44.67% · 35.44% · 40.90% 였다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use serde_json::Value;

/// (원본 HWP, 한컴 Print PDF 쪽수)
const CASES: &[(&str, u32)] = &[
    ("samples/issue8149/headerrow-2020.hwp", 31),
    ("samples/issue8149/reservefit-2020.hwp", 65),
    ("samples/issue8149/rowfit-2020.hwp", 33),
    ("samples/issue8060/71352_lpg_regulatory_impact.hwp", 48),
];

fn open_hwp(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open hwp")
}

fn open_exported_hwpx(hwp: &DocumentCore) -> DocumentCore {
    let hwpx = hwp.export_hwpx_native().expect("export hwpx");
    DocumentCore::from_bytes(&hwpx).expect("reopen hwpx")
}

/// 쪽마다 (구역, 문단, 종류, 컨트롤, 시작/끝 행, 시작/끝 컷, 줄 범위) — 좌표 없는 쪽 구성.
fn page_structure(core: &DocumentCore) -> Vec<(u64, Vec<String>)> {
    let dump = core.dump_page_items_json(None);
    dump.as_array()
        .expect("pages")
        .iter()
        .map(|page| {
            let section = page
                .get("section")
                .and_then(Value::as_u64)
                .expect("section");
            let items = page
                .pointer("/columns")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .flat_map(|column| {
                    column
                        .get("items")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                })
                .map(|item| {
                    [
                        "kind",
                        "paraIndex",
                        "controlIndex",
                        "startRow",
                        "endRow",
                        "startCut",
                        "endCut",
                        "startLine",
                        "endLine",
                    ]
                    .iter()
                    .map(|key| item.get(*key).unwrap_or(&Value::Null).to_string())
                    .collect::<Vec<_>>()
                    .join(":")
                })
                .collect();
            (section, items)
        })
        .collect()
}

#[test]
fn hwp5_origin_hwpx_paginates_like_its_source_hwp() {
    for &(sample, hancom_pages) in CASES {
        let hwp = open_hwp(sample);
        let hwpx = open_exported_hwpx(&hwp);
        assert_eq!(hwp.page_count(), hancom_pages, "{sample}: HWP 쪽수");
        assert_eq!(hwpx.page_count(), hancom_pages, "{sample}: HWPX 쪽수");

        // 같은 문서다 — 쪽마다 항목·행·컷·줄 구성이 원본 HWP 와 같아야 한다.
        let source = page_structure(&hwp);
        let exported = page_structure(&hwpx);
        for (index, (a, b)) in source.iter().zip(&exported).enumerate() {
            assert_eq!(a, b, "{sample}: {}쪽 구성이 원본 HWP 와 다르다", index + 1);
        }
    }
}
