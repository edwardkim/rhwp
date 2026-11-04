#![cfg(not(target_arch = "wasm32"))]

//! #7687의 제목 host와 뒤 본문 소유 관계를 독립 한컴 Print와 대조한다.
//! 동일 입력의 전4쪽이 Native246/fresh WASM069에서 각각 raw100%였다.
//! source: 7e0be68ddc200bde3a29414deb3c0e870b8db357, 96 DPI, print, compat2022.
//! 선행 시각 검증과 문단16/28의 독립 근거는
//! mydocs/working/task_semanticist21_20261025_accepted_paragraph_origin.md에 기록한다.

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

fn body_text(tree: &Value, pi: u64) -> String {
    fn walk(node: &Value, pi: u64, out: &mut String) {
        if node["type"].as_str() == Some("Table") {
            return;
        }
        if node["type"].as_str() == Some("TextRun") && node["pi"].as_u64() == Some(pi) {
            out.push_str(node["text"].as_str().unwrap_or_default());
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, pi, out);
        }
    }
    let mut out = String::new();
    walk(tree, pi, &mut out);
    out
}

/// 합성 K1~K4 (`samples/issue7687/gen_titled_host_after_band.py`, 모든 문단에 저장 줄 없음).
/// 빈 host(T1 오프셋 0 / T2 오프셋 15000HU) 의 띠 뒤에 글 있는 host(T3 오프셋 0 / T4 오프셋
/// 9000HU)가 온다. 표마다 바깥 여백 283HU(3.77px). 한/글 2020 독립 Print
/// `pdf/issue7687/titled-host-2020.pdf` 와 대조한다.
///
/// 빈 host 표의 배제 띠 끝을 바깥 여백만큼 더 세면 뒤 글 host 의 원점이 내려가, 흐름이 놓는
/// T3 는 그대로인데 그 원점에서 재는 T4 와 T4 아래 글만 어긋난다(한글문서파일형식 20쪽 pi=133).
#[test]
fn verified_titled_host_offsets_and_first_body_ownership_follow_print() {
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
    for (page, titled, after_t4) in [(1u32, 15u64, 16u64), (3, 27, 28)] {
        let t = tree(&doc, page);
        let (t3_top, _) = table(&t, titled, 0);
        let (t4_top, t4_bottom) = table(&t, titled, 1);
        // Print의 첫 본문은 K2/K4 글 1이다. 글 2를 첫 줄로 잘못 고정하지 않는다.
        assert_eq!(
            body_text(&t, after_t4).trim(),
            format!("K{} 글 1", page + 1)
        );
        assert_eq!(
            body_text(&t, after_t4 + 1).trim(),
            format!("K{} 글 2", page + 1)
        );
        assert!(line(&t, after_t4).1 <= line(&t, after_t4 + 1).0);
        assert!(line(&t, after_t4 + 1).1 <= line(&t, after_t4 + 2).0);
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
