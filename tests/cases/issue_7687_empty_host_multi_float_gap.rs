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

/// 합성 K1~K4 (`samples/issue7687/gen_titled_host_after_band.py`, 모든 문단에 저장 줄 없음).
/// 빈 host(T1 오프셋 0 / T2 오프셋 15000HU) 의 띠 뒤에 글 있는 host(T3 오프셋 0 / T4 오프셋
/// 9000HU)가 온다. 표마다 바깥 여백 283HU(3.77px). 한/글 2020 독립 Print
/// `pdf/issue7687/titled-host-2020.pdf` 와 대조한다.
///
/// 빈 host 표의 배제 띠 끝을 바깥 여백만큼 더 세면 뒤 글 host 의 원점이 내려가, 흐름이 놓는
/// T3 는 그대로인데 그 원점에서 재는 T4 와 T4 아래 글만 어긋난다(한글문서파일형식 20쪽 pi=133).
#[test]
fn titled_host_after_empty_host_band_keeps_its_float_offsets() {
    const OUTER: f64 = 283.0 * 96.0 / 7200.0;
    const T4_OFFSET: f64 = 9000.0 * 96.0 / 7200.0;
    let doc = load("samples/issue7687/titled-host.hwpx");
    assert_eq!(doc.page_count(), 4, "정본 PDF 와 같은 4쪽");
    let near = |what: &str, actual: f64, expected: f64| {
        assert!(
            (actual - expected).abs() <= 1.0,
            "{what}: 실제 {actual:.1}, 기대 {expected:.1} (±1px)"
        );
    };

    // K1·K3: T4 의 자연 위치가 T2 띠 안이라 띠 끝 아래에 쌓인다
    // (T4 윗변 = T2 아랫변 + T2 아래 여백 + T4 위 여백). 뒤 글줄은 T4 띠 끝에 선다.
    for (page, empty_host, titled, next_text) in [(0u32, 2u64, 5u64, 5u64), (2, 20, 21, 21)] {
        let t = tree(&doc, page);
        let (_, t2_bottom) = table(&t, empty_host, 1);
        let (t3_top, _) = table(&t, titled, 0);
        let (t4_top, t4_bottom) = table(&t, titled, 1);
        assert!(t3_top < t4_top, "K{} T3 는 T4 보다 위", page + 1);
        near(
            &format!("K{} T4 윗변", page + 1),
            t4_top,
            t2_bottom + OUTER + OUTER,
        );
        near(
            &format!("K{} T4 아래 첫 글줄", page + 1),
            line(&t, next_text).0,
            t4_bottom + OUTER,
        );
    }

    // K2·K4: T4 는 자기 host 원점에서 오프셋만큼 아래다. T3 와 T4 는 같은 host·같은 위 여백이라
    // 두 윗변의 차이가 곧 오프셋 차이다. T4 아래 첫 글줄은 T4 띠 끝에 선다.
    for (page, titled, after_t4) in [(1u32, 15u64, 17u64), (3, 27, 29)] {
        let t = tree(&doc, page);
        let (t3_top, _) = table(&t, titled, 0);
        let (t4_top, t4_bottom) = table(&t, titled, 1);
        near(
            &format!("K{} T4 윗변 − T3 윗변", page + 1),
            t4_top - t3_top,
            T4_OFFSET,
        );
        near(
            &format!("K{} T4 아래 첫 글줄", page + 1),
            line(&t, after_t4).0,
            t4_bottom + OUTER,
        );
    }
}
