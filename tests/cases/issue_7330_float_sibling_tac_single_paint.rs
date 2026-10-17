#![cfg(not(target_arch = "wasm32"))]

//! [#7330] 자리차지 표 항목이 같은 문단의 글자처럼 취급(TAC) 표를 한 번 더 그리지 않는다.
//!
//! `36374873_결재문서본문_야간방호일지` 는 한 문단(pi=3)에 결재란 TAC 3×4 표와 본문 13×8
//! 자리차지 표가 함께 있고, 조판기는 TAC 표를 독립 `PageItem::Table` 로 내보낸다. 종전에는
//! 자리차지 표 항목의 «같은 문단의 인라인 TAC 표 렌더링» 경로가 그 TAC 표를 다시 그려
//! render tree 에 같은 bbox 의 Table 이 둘 생기고(글자 겹침 10건), 글자 추출이 결재란을 두 번
//! 내보냈다. 수정 전에는 첫 검사(TAC 표 노드 1개)에서 실패한다.
//!
//! 한/글 2020 Print(`pdf/36374873_night_guard_log-2020.pdf`)에는 결재란이 한 번뿐이다.

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load() -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue4599/36374873_night_guard_log.hwpx");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("표본 파싱")
}

/// 1쪽 render tree 의 본문 직속 Table 노드 (pi, ci, y, 아래끝). 표 안은 내려가지 않는다.
fn top_level_tables(tree: &Value) -> Vec<(u64, u64, f64, f64)> {
    fn walk(node: &Value, out: &mut Vec<(u64, u64, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        if kind == "Table" {
            let y = node["bbox"]["y"].as_f64().unwrap();
            let h = node["bbox"]["h"].as_f64().unwrap();
            out.push((
                node["pi"].as_u64().unwrap(),
                node["ci"].as_u64().unwrap(),
                y,
                y + h,
            ));
            return;
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(tree, &mut out);
    out
}

fn body_bottom(tree: &Value) -> f64 {
    fn find(node: &Value) -> Option<f64> {
        if node["type"].as_str() == Some("Body") {
            return Some(node["bbox"]["y"].as_f64()? + node["bbox"]["h"].as_f64()?);
        }
        node["children"]
            .as_array()
            .into_iter()
            .flatten()
            .find_map(find)
    }
    find(tree).expect("Body 노드")
}

fn text_lines(tree: &Value) -> Vec<(u64, f64, f64)> {
    fn walk(node: &Value, out: &mut Vec<(u64, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        if kind == "Table" {
            return;
        }
        if kind == "TextLine" {
            let y = node["bbox"]["y"].as_f64().unwrap();
            let h = node["bbox"]["h"].as_f64().unwrap();
            out.push((node["pi"].as_u64().unwrap(), y, y + h));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(tree, &mut out);
    out
}

#[test]
fn approval_box_tac_table_is_painted_once() {
    let doc = load();
    assert_eq!(doc.page_count(), 1);
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let tables = top_level_tables(&tree);
    let tac: Vec<_> = tables.iter().filter(|t| t.0 == 3 && t.1 == 1).collect();
    assert_eq!(
        tac.len(),
        1,
        "pi=3 ci=1 결재란 TAC 표는 한 번만 그려야 한다: {tables:?}"
    );
    // 같은 (pi, ci) 를 가진 본문 직속 Table 은 하나뿐이다.
    let mut keys: Vec<(u64, u64)> = tables.iter().map(|t| (t.0, t.1)).collect();
    keys.sort_unstable();
    let unique = keys.len();
    keys.dedup();
    assert_eq!(unique, keys.len(), "같은 표가 두 번 그려졌다: {tables:?}");
}

#[test]
fn host_paragraph_order_and_following_body_stay_in_place() {
    let doc = load();
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let tables = top_level_tables(&tree);
    let tac = tables
        .iter()
        .find(|t| t.0 == 3 && t.1 == 1)
        .expect("결재란 TAC 표");
    let float = tables
        .iter()
        .find(|t| t.0 == 3 && t.1 == 0)
        .expect("본문 자리차지 표");
    assert!(
        tac.3 <= float.2 + 0.5,
        "결재란({:.1}~{:.1})은 본문 표({:.1}~) 위에 있어야 한다",
        tac.2,
        tac.3,
        float.2
    );
    // 표 뒤 문단(pi 5·6·7)은 표 아래에 차례대로 놓이고, 글이 있는 pi=6(「붙임: … 끝.」)은
    // 본문 안에 있다(#7341 이 되돌린 자리). 끝의 빈 문단 pi=7 은 본문 아래 변을 7.5px 넘는
    // 기존 차이가 있어 순서만 본다.
    let bottom = body_bottom(&tree);
    let lines = text_lines(&tree);
    let mut prev = float.3;
    for pi in [5u64, 6, 7] {
        let (_, y, end) = lines
            .iter()
            .copied()
            .find(|l| l.0 == pi)
            .unwrap_or_else(|| panic!("pi={pi} 글줄: {lines:?}"));
        assert!(
            y >= prev - 0.5,
            "pi={pi} 글줄 {y:.1} 이 앞 항목 끝 {prev:.1} 위다"
        );
        if pi == 6 {
            assert!(
                end <= bottom + 0.5,
                "pi={pi} 글줄 끝 {end:.1} 이 본문 아래 {bottom:.1} 를 넘는다"
            );
        }
        prev = y;
    }
}
