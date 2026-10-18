//! [#7679] 글줄 뒤 같은 문단 둘째 줄의 글자처럼 취급(TAC) 자리차지 표.
//!
//! 같은 문단의 글줄이 표 줄보다 먼저 놓였으면 문단 앞 간격은 그 글줄이 이미 소비했다. 표 줄은
//! 문단 안의 다음 줄이라 앞 간격을 다시 받지 않는다. 수정 전 rhwp 는 표 앞에서 앞 간격을 한 번
//! 더 더해 표와 뒤 본문이 그만큼 내려갔다.
//!
//! 입력 `samples/issue7679/tac-after-text-2024.hwp` 는 생성기 `gen_tac_after_text.py` 의 HWPX(동봉)를
//! 한/글 2024(13.0.0.564)로 HWP 저장한 것이고, 정본 `pdf/issue7679/tac-after-text-2024.pdf` 는 같은
//! 한/글의 Print 출력이다.
//!
//! ```text
//!   (96dpi)        글줄 상단   표 상단(정본)   표 − 글줄   수정 전 rhwp 표 상단
//!   1쪽 앞 간격 4000HU  184.9    210.0           25.1        236.9 (+26.7 = 앞 간격)
//!   2쪽 앞 간격 0       136.9    162.1           25.2        162.2 (대조군)
//!   3쪽 앞 간격 4000HU  163.5    188.6           25.1        215.6
//! ```
//! 25.1 = 글줄 전진(줄 높이 1000 + 줄간격 600HU = 21.3) + 표 바깥 위 여백 283HU(3.8).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue7679/tac-after-text-2024.hwp";
/// 글줄 전진 1600HU + 바깥 위 여백 283HU.
const EXPECTED_GAP_PX: f64 = (1600.0 + 283.0) * 96.0 / 7200.0;

/// 쪽의 (host 글줄 상단, 표 상단, 표 하단, 다음 글줄 상단).
fn page_items(doc: &HwpDocument, page: u32, host_pi: u64) -> (f64, f64, f64, f64) {
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
    let mut items = Vec::new();
    fn walk(node: &Value, in_table: bool, out: &mut Vec<(&'static str, u64, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        let y = node["bbox"]["y"].as_f64().unwrap_or(0.0);
        let bottom = y + node["bbox"]["h"].as_f64().unwrap_or(0.0);
        let pi = node["pi"].as_u64().unwrap_or(u64::MAX);
        if kind == "Table" && !in_table {
            out.push(("Table", pi, y, bottom));
        } else if kind == "TextLine" && !in_table {
            out.push(("Line", pi, y, bottom));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, in_table || kind == "Table", out);
        }
    }
    walk(&tree, false, &mut items);
    let host = items
        .iter()
        .find(|i| i.0 == "Line" && i.1 == host_pi)
        .unwrap_or_else(|| panic!("{}쪽 host 글줄: {items:?}", page + 1));
    let table = items
        .iter()
        .find(|i| i.0 == "Table" && i.1 == host_pi)
        .unwrap_or_else(|| panic!("{}쪽 표: {items:?}", page + 1));
    let next = items
        .iter()
        .find(|i| i.0 == "Line" && i.1 == host_pi + 1)
        .unwrap_or_else(|| panic!("{}쪽 다음 문단: {items:?}", page + 1));
    (host.2, table.2, table.3, next.2)
}

#[test]
fn tac_table_after_host_line_does_not_take_spacing_before_again() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    assert_eq!(doc.page_count(), 3, "정본 PDF 와 같은 3쪽");
    // 사례마다 문단 5개(제목·앞 문단·host·「나.」·뒤 문단) — host 는 3, 8, 13.
    for (page, host_pi) in [(0u32, 3u64), (1, 8), (2, 13)] {
        let (host, table_top, table_bottom, next) = page_items(&doc, page, host_pi);
        let gap = table_top - host;
        assert!(
            (gap - EXPECTED_GAP_PX).abs() <= 1.0,
            "{}쪽: 표 상단 − host 글줄 상단은 글줄 전진 + 바깥 위 여백 {EXPECTED_GAP_PX:.1}px 이어야 한다 \
             (gap {gap:.1}; 수정 전 앞 간격이 다시 더해져 +26.7)",
            page + 1
        );
        assert!(
            next > table_bottom,
            "{}쪽: 다음 문단 {next:.1} 은 표 하단 {table_bottom:.1} 아래다",
            page + 1
        );
    }
}
