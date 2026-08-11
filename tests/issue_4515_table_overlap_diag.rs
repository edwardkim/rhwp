//! [#4515] `LAYOUT_TABLE_OVERLAP` 진단의 회귀 가드.
//!
//! `LAYOUT_OVERFLOW` 는 본문 하단 초과만 잡는다 — #4514 처럼 표 하단이 본문 하단으로
//! clamp 된 겹침은 초과량 0 이라 한 건도 못 잡았다(경고 페이지 16·42 vs 실제 결함
//! 페이지 8·12·13·22·25·29, 교집합 0). 이 테스트는 진단이 render tree 의 실제 최상위
//! 표 bbox 와 자기일관인지 검증한다 — #4514 가 수정돼 겹침 자체가 사라져도 양쪽이
//! 함께 0 이 되므로 픽스처 갱신 없이 성립한다. 검출 로직 자체의 동작(임계·정렬·수집
//! 도메인)은 `src/renderer/layout/tests.rs` 의 단위 테스트가 가드한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;

/// render tree JSON 에서 진단과 같은 도메인(Page 직계 + Body→Column 직계 Table)의
/// (pi, y0, y1) 을 독립 수집한다.
fn top_level_table_spans(root: &serde_json::Value) -> Vec<(i64, f64, f64)> {
    fn span_of(node: &serde_json::Value) -> Option<(i64, f64, f64)> {
        if node.get("type").and_then(|t| t.as_str()) != Some("Table") {
            return None;
        }
        let b = node.get("bbox")?;
        let y = b.get("y")?.as_f64()?;
        let h = b.get("h")?.as_f64()?;
        Some((
            node.get("pi").and_then(|p| p.as_i64()).unwrap_or(-1),
            y,
            y + h,
        ))
    }
    let empty = Vec::new();
    let children = |n: &serde_json::Value| -> Vec<serde_json::Value> {
        n.get("children")
            .and_then(|c| c.as_array())
            .unwrap_or(&empty)
            .clone()
    };
    let mut out = Vec::new();
    for child in children(root) {
        if let Some(s) = span_of(&child) {
            out.push(s);
        }
        if child.get("type").and_then(|t| t.as_str()) == Some("Body") {
            for col in children(&child) {
                if col.get("type").and_then(|t| t.as_str()) != Some("Column") {
                    continue;
                }
                for item in children(&col) {
                    if let Some(s) = span_of(&item) {
                        out.push(s);
                    }
                }
            }
        }
    }
    out
}

/// 독립 수집한 span 으로 겹침 쌍을 계산한다 (진단과 같은 규칙: y 정렬 후 인접 쌍,
/// 임계 2px 초과).
fn expected_overlap_pairs(mut spans: Vec<(i64, f64, f64)>) -> Vec<(i64, i64)> {
    spans.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    spans
        .windows(2)
        .filter(|w| w[0].2 - w[1].1 > 2.0)
        .map(|w| (w[0].0, w[1].0))
        .collect()
}

/// 47쪽 전 페이지에서 `take_table_overlaps()` 가 render tree 실측 겹침과 페이지·쌍
/// 단위로 일치해야 한다. #4514 재현 상태에서는 8·12·13·22·25·29쪽(1-based)에서
/// 겹침이 보고되고, 20쪽의 1.7px 접합 오차는 임계에 걸러진다.
#[test]
fn issue_4515_diag_matches_render_tree_ground_truth() {
    let bytes = std::fs::read("samples/issue4514/sample1-repro.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");
    let total = core.page_count();
    assert!(total > 0, "페이지가 있어야 한다");

    let mut diag_pages: Vec<(u32, Vec<(i64, i64)>)> = Vec::new();
    let mut truth_pages: Vec<(u32, Vec<(i64, i64)>)> = Vec::new();
    let mut any_table_seen = false;

    for page in 0..total {
        let tree = core
            .build_page_render_tree(page)
            .expect("render tree 를 얻을 수 있어야 한다");
        let overlaps = core.take_table_overlaps();
        let json: serde_json::Value =
            serde_json::from_str(&tree.root.to_json()).expect("render tree JSON");
        let root = json.get("root").unwrap_or(&json);

        let spans = top_level_table_spans(root);
        if !spans.is_empty() {
            any_table_seen = true;
        }
        let expected = expected_overlap_pairs(spans);
        if !expected.is_empty() {
            truth_pages.push((page, expected));
        }
        if !overlaps.is_empty() {
            for o in &overlaps {
                assert_eq!(o.page_index, page, "겹침은 렌더한 페이지에 귀속돼야 한다");
                assert!(
                    o.overlap_px > 2.0,
                    "임계 2px 이하는 보고하지 않아야 한다 (page={} overlap={:.1})",
                    page,
                    o.overlap_px
                );
            }
            diag_pages.push((
                page,
                overlaps
                    .iter()
                    .map(|o| (o.para_a as i64, o.para_b as i64))
                    .collect(),
            ));
        }
    }

    assert!(
        any_table_seen,
        "픽스처에 최상위 표가 있어야 검증이 의미 있다"
    );
    assert_eq!(
        diag_pages, truth_pages,
        "LAYOUT_TABLE_OVERLAP 진단은 render tree 실측 겹침과 일치해야 한다"
    );
}
