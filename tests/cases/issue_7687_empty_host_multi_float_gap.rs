#![cfg(not(target_arch = "wasm32"))]

//! [#7687 · #7680] 빈(또는 공백뿐인) host 문단의 다중 자리차지 표와 글 있는 host 의 둘째 양수
//! 오프셋 표.
//!
//! 한/글 2020 독립 Print `pdf/issue7687/empty-host-2020.pdf` H1~H6와 대조한다.
//! 빈 host의 표 사이 틈, 줄 상자의 배제 띠 회피와 후속 본문 순서를 보호한다.
//!
//! 전체 쪽수·시각 선행 조건을 충족하지 못한 실제 형식 명세 문서의 재활성화 검사는
//! mydocs/pr/assets/issue_7687/reactivated_guard/에 원문·공용 helper와 함께 보관했다.
//! 공유 원본·독립 PDF·다른 정상 검사는 유지한다.

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load(rel: &str) -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("표본 파싱")
}

fn tree(doc: &HwpDocument, page: u32) -> Value {
    serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap()
}

/// 본문 직속 Table 노드 (pi, ci, 윗변, 아랫변). 표 안은 내려가지 않는다.
fn tables(tree: &Value) -> Vec<(u64, u64, f64, f64)> {
    fn walk(node: &Value, out: &mut Vec<(u64, u64, f64, f64)>) {
        if node["type"].as_str() == Some("Table") {
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

fn table(tree: &Value, pi: u64, ci: u64) -> (f64, f64) {
    tables(tree)
        .into_iter()
        .find(|t| t.0 == pi && t.1 == ci)
        .map(|t| (t.2, t.3))
        .unwrap_or_else(|| panic!("표 pi={pi} ci={ci}: {:?}", tables(tree)))
}

/// 본문 직속 글줄 (pi, 윗변, 아랫변). 표 안은 내려가지 않는다.
fn lines(tree: &Value) -> Vec<(u64, f64, f64)> {
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

fn line(tree: &Value, pi: u64) -> (f64, f64) {
    lines(tree)
        .into_iter()
        .find(|l| l.0 == pi)
        .map(|l| (l.1, l.2))
        .unwrap_or_else(|| panic!("글줄 pi={pi}: {:?}", lines(tree)))
}

/// 글줄이 두 표 사이 틈 안에 있다(첫 표 아래, 둘째 표 위).
fn assert_in_gap(what: &str, (y, end): (f64, f64), first_bottom: f64, second_top: f64) {
    assert!(
        y >= first_bottom - 0.5 && end <= second_top + 0.5,
        "{what} 글줄 {y:.1}~{end:.1} 은 첫 표 아래 {first_bottom:.1} 와 둘째 표 위 {second_top:.1} 사이여야 한다"
    );
}

fn assert_below(what: &str, (y, _): (f64, f64), bottom: f64) {
    assert!(
        y >= bottom - 0.5,
        "{what} 글줄 {y:.1} 은 표 아래 {bottom:.1} 밑이어야 한다"
    );
}

/// 합성 H1~H6 (`samples/issue7687/gen_empty_host.py`, 모든 문단에 저장 줄 없음).
/// 쪽마다 빈 host + T1(오프셋 0) + T2(오프셋 15000HU) 뒤에 사례별 문단이 온다.
#[test]
fn synthetic_empty_host_cases_follow_hancom_gap_rule() {
    let doc = load("samples/issue7687/empty-host.hwpx");
    assert_eq!(doc.page_count(), 6, "정본 PDF 와 같은 6쪽");
    let host_line_advance = |t: &Value, text_pi: u64, t1_bottom: f64| {
        // 첫 표 띠 끝(아래 바깥 여백 283HU = 3.8px) 뒤에 host 줄 상자(10pt × 160% = 21.3px)가
        // 놓이고 그 뒤에 첫 글줄이 온다 — 글자모양에서 계산한 줄 상자 크기와의 관계.
        let (y, _) = line(t, text_pi);
        let gap = y - t1_bottom;
        assert!(
            (gap - (3.8 + 21.3)).abs() <= 1.5,
            "pi={text_pi} 첫 글줄은 첫 표 아래 바깥여백 + host 줄 상자 뒤에 와야 한다 (gap {gap:.1})"
        );
    };

    // H1: 글 8줄 — 1·2·3 은 틈, 4 부터 T2 아래.
    let t = tree(&doc, 0);
    let (_, t1_bottom) = table(&t, 2, 0);
    let (t2_top, t2_bottom) = table(&t, 2, 1);
    host_line_advance(&t, 3, t1_bottom);
    for pi in [3u64, 4, 5] {
        assert_in_gap(&format!("H1 pi={pi}"), line(&t, pi), t1_bottom, t2_top);
    }
    assert_below("H1 pi=6", line(&t, 6), t2_bottom);
    let mut prev = line(&t, 6).1;
    for pi in [7u64, 8, 9, 10] {
        let l = line(&t, pi);
        assert!(l.0 >= prev - 0.5, "H1 pi={pi} 순서");
        prev = l.1;
    }

    // H2: 빈 문단 2 + 글 → 글은 틈.  H3: 빈 문단 1 + 글 → 틈.
    let t = tree(&doc, 1);
    let (_, t1_bottom) = table(&t, 12, 0);
    let (t2_top, _) = table(&t, 12, 1);
    host_line_advance(&t, 13, t1_bottom);
    assert_in_gap("H2 글", line(&t, 15), t1_bottom, t2_top);
    let t = tree(&doc, 2);
    let (_, t1_bottom) = table(&t, 17, 0);
    let (t2_top, _) = table(&t, 17, 1);
    assert_in_gap("H3 글", line(&t, 19), t1_bottom, t2_top);

    // H4: 빈 문단 4 + 글 — 넷째 빈 문단이 T2 띠에 걸려 띠 아래로, 글도 그 뒤.
    let t = tree(&doc, 3);
    let (_, t1_bottom) = table(&t, 21, 0);
    let (t2_top, t2_bottom) = table(&t, 21, 1);
    for pi in [22u64, 23, 24] {
        assert_in_gap(&format!("H4 pi={pi}"), line(&t, pi), t1_bottom, t2_top);
    }
    assert_below("H4 넷째 빈 문단", line(&t, 25), t2_bottom);
    assert_below("H4 글", line(&t, 26), t2_bottom);
    assert!(
        line(&t, 26).0 >= line(&t, 25).1 - 0.5,
        "H4 글은 넷째 빈 문단 뒤"
    );

    // H5: 공백 host — 빈 host 와 같다(host 줄은 틈, 빈 문단 2, 글은 틈).
    let t = tree(&doc, 4);
    let (_, t1_bottom) = table(&t, 28, 0);
    let (t2_top, _) = table(&t, 28, 1);
    assert_in_gap("H5 host", line(&t, 28), t1_bottom, t2_top);
    assert_in_gap("H5 글", line(&t, 31), t1_bottom, t2_top);

    // H6: 앞 간격 있는 문단들 — 셋째 문단이 띠에 걸려 T2 아래로 가고 앞 간격은 띠 안에서
    // 흡수된다(글줄 윗변 = 띠 끝 = 표 아랫변 + 아래 바깥여백).
    let t = tree(&doc, 5);
    let (_, t2_bottom) = table(&t, 33, 1);
    let p36 = line(&t, 36);
    assert_below("H6 글", p36, t2_bottom);
    assert!(
        p36.0 - t2_bottom <= 3.8 + 1.0,
        "H6 글 {:.1} 은 앞 간격 없이 띠 끝(표 {:.1} + 여백 3.8) 에 바로 선다",
        p36.0,
        t2_bottom
    );
}
