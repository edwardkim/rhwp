//! [#7678] 자리차지(PARA·TOP_AND_BOTTOM) 표 뒤 host 글이 쪽 끝에 걸리면 줄 단위로 나눈다.
//!
//! 한/글은 들어가는 줄까지 이 쪽에 두고 나머지 줄을 다음 쪽 맨 위로 넘긴다(넘긴 줄은 문단 안의
//! 줄이라 앞 간격이 없다). 수정 전 rhwp 는 표 뒤 글 전체를 한 항목(`PartialParagraph lines=0..4`)으로
//! 현재 쪽에 붙여 넷째 줄이 본문 하단 아래(1035.7~1049.0, 본문 하단 1028.1)에 그려졌다.
//!
//! 입력은 저장소 생성기 `make_float_push_spacing_before_boundaries.py --continuation` 의 HWPX(저장 줄
//! 없음)와, 그것을 한/글 2024(13.0.0.564)로 HWP 저장한 파일(저장 줄 있음)이다. 정본
//! `pdf/issue7678/float_push_spacing_before_continuation-2024.pdf` 는 같은 한/글의 Print 출력이다.
//!
//! ```text
//!   C 쪽 (96dpi)            정본                 수정 전 rhwp
//!   표 뒤 글 1~3줄          7쪽 970.6/991.8/1013.1  7쪽 971.7/993.0/1014.4
//!   표 뒤 글 4줄            8쪽 94.2               7쪽 1035.7 (본문 하단 밖)
//!   「C 뒤 문단이다.」       8쪽 115.5              8쪽 94.5
//! ```
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

/// host 문단(「C. …」)과 다음 문단의 구역 내 문단 번호.
const HOST_PI: u64 = 127;
const NEXT_PI: u64 = 128;

fn host_lines_by_page(rel: &str) -> (Vec<(f64, f64)>, Vec<(u64, f64)>) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    assert_eq!(doc.page_count(), 8, "{rel}: 정본 PDF 와 같은 8쪽");
    let lines = |page: u32| {
        let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
        let mut out = Vec::new();
        fn walk(node: &Value, in_table: bool, out: &mut Vec<(u64, f64, f64)>) {
            let kind = node["type"].as_str().unwrap_or_default();
            if kind == "TextLine" && !in_table {
                let y = node["bbox"]["y"].as_f64().unwrap();
                out.push((
                    node["pi"].as_u64().unwrap_or(u64::MAX),
                    y,
                    y + node["bbox"]["h"].as_f64().unwrap(),
                ));
            }
            for child in node["children"].as_array().into_iter().flatten() {
                walk(child, in_table || kind == "Table", out);
            }
        }
        walk(&tree, false, &mut out);
        out
    };
    let p7: Vec<(f64, f64)> = lines(6)
        .into_iter()
        .filter(|l| l.0 == HOST_PI)
        .map(|l| (l.1, l.2))
        .collect();
    let p8: Vec<(u64, f64)> = lines(7).into_iter().map(|l| (l.0, l.1)).collect();
    (p7, p8)
}

fn assert_split(rel: &str) {
    let (p7, p8) = host_lines_by_page(rel);
    assert_eq!(
        p7.len(),
        3,
        "{rel}: 7쪽에는 표 뒤 글 3줄만 남아야 한다(정본) — {p7:?}"
    );
    assert!(
        p7.windows(2).all(|w| w[1].0 >= w[0].1 - 0.5),
        "{rel}: 7쪽 글줄 순서 {p7:?}"
    );
    let first = p8.first().copied();
    assert_eq!(
        first.map(|l| l.0),
        Some(HOST_PI),
        "{rel}: 8쪽 첫 글줄은 이어받은 host 넷째 줄이어야 한다 — {p8:?}"
    );
    assert_eq!(
        p8.iter().filter(|l| l.0 == HOST_PI).count(),
        1,
        "{rel}: 8쪽에 넘어온 host 줄은 하나다 — {p8:?}"
    );
    let next = p8
        .iter()
        .find(|l| l.0 == NEXT_PI)
        .unwrap_or_else(|| panic!("{rel}: 8쪽에 다음 문단이 있어야 한다 — {p8:?}"));
    assert!(
        next.1 > first.unwrap().1,
        "{rel}: 다음 문단은 넘어온 host 줄 뒤다 — {p8:?}"
    );
}

#[test]
fn host_text_after_float_table_splits_at_the_page_end_saved_hwp() {
    assert_split("mydocs/pr/assets/issue7445/deferred-20261020/added-issue7678-float_push_spacing_before_continuation-2024-hwp/float_push_spacing_before_continuation-2024.hwp");
}

#[test]
fn host_text_after_float_table_splits_at_the_page_end_without_stored_lines() {
    assert_split("mydocs/pr/assets/issue7445/deferred-20261020/added-issue7678-float_push_spacing_before_continuation-hwpx/float_push_spacing_before_continuation.hwpx");
}
