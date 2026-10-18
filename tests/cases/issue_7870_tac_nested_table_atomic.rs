//! [#7870] 셀 안 글자처럼 취급(TAC) 중첩 표는 쪽 경계에서 행 사이로 나뉘지 않는다.
//!
//! TAC 중첩 표는 줄 안의 글자 하나다. 한/글은 그 줄이 쪽 남은 공간에 안 들어가면 줄째 다음
//! 쪽으로 넘긴다. 수정 전 rhwp 는 바깥 칸 문단의 중첩 표를 `pageBreak` 만 보고 행 유닛으로
//! 나눠, 한/글이 다음 쪽에 통째로 그린 4×4 표의 앞 두 행을 앞 쪽 바닥에 남겼다.
//!
//! 입력 `samples/issue7870/{tac,flow}_37.hwpx` 는 생성기 `gen_tac_nested_split.py` 의 출력이고,
//! 정본 `pdf/issue7870/*-2024.pdf` 는 한/글 2024(13.0.0.564) Print 다. 두 입력은 중첩 4×4 표의
//! `treatAsChar` 만 다르다(바깥 1×1 표·중첩 표 모두 `pageBreak=CELL`).
//!
//! | 입력 | 한/글 1쪽 | 한/글 2쪽 |
//! |---|---|---|
//! | `tac_37` (TAC) | 칸 앞 글줄 | 행 0~3 · 칸 뒤 글줄 · 표 뒤 문단 |
//! | `flow_37` (자리차지, 대조군) | 칸 앞 글줄 · 행 0~1 | 행 2~3 · 칸 뒤 글줄 · 표 뒤 문단 |
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

/// 쪽마다 표지 글자(BEFORE·R{n}C0·AFTER·TAIL)의 순서.
fn markers_by_page(sample: &str) -> Vec<Vec<String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    fn walk(node: &Value, out: &mut Vec<(f64, String)>) {
        if node["type"] == "TextRun" {
            let text = node["text"].as_str().unwrap_or_default();
            for marker in ["BEFORE", "AFTER", "TAIL"] {
                if text.contains(marker) {
                    out.push((node["bbox"]["y"].as_f64().unwrap(), marker.to_string()));
                }
            }
            if let Some(start) = text.find('R') {
                let tail = &text[start..];
                if let Some(end) = tail.find("C0") {
                    if tail[1..end].chars().all(|c| c.is_ascii_digit()) && end > 1 {
                        out.push((
                            node["bbox"]["y"].as_f64().unwrap(),
                            tail[..end + 2].to_string(),
                        ));
                    }
                }
            }
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, out);
        }
    }
    (0..doc.page_count())
        .map(|page| {
            let tree: Value =
                serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
            let mut markers = Vec::new();
            walk(&tree, &mut markers);
            markers.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            markers.into_iter().map(|(_, marker)| marker).collect()
        })
        .collect()
}

#[test]
fn tac_nested_table_moves_whole_line_to_next_page() {
    let pages = markers_by_page("samples/issue7870/tac_37.hwpx");
    assert_eq!(
        pages,
        vec![
            vec!["BEFORE"],
            vec!["R0C0", "R1C0", "R2C0", "R3C0", "AFTER", "TAIL"],
        ],
        "TAC 중첩 표는 줄째 다음 쪽으로 간다(한/글 2024) — 수정 전 rhwp 는 행 0~1 을 1쪽에 남겼다"
    );
}

#[test]
fn floating_nested_table_still_splits_between_rows() {
    let pages = markers_by_page("samples/issue7870/flow_37.hwpx");
    assert_eq!(
        pages,
        vec![
            vec!["BEFORE", "R0C0", "R1C0"],
            vec!["R2C0", "R3C0", "AFTER", "TAIL"],
        ],
        "자리차지 중첩 표는 한/글처럼 행 사이에서 나뉜다"
    );
}
