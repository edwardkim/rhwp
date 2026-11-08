#![cfg(not(target_arch = "wasm32"))]

//! [#8156] 저장 줄이 있는 글 host 의 양수 오프셋 자리차지 표는 문단 원점에서 잰다.
//!
//! 입력은 #7687 합성 K1~K4(`samples/issue7687/gen_titled_host_after_band.py` 의 HWPX)를
//! 한/글 2020 으로 HWP 저장한 것이다(저장 줄 있음). 글 host 마다 T3(오프셋 0)와
//! T4(오프셋 9000HU)가 있고 표마다 바깥 여백 283HU 이다. 한/글은 T3 가 흐름을 소비하므로
//! host 첫 줄을 T3 띠 아래에 저장한다. 같은 HWP 의 한/글 2020 독립 Print
//! `pdf/issue8156/titled-host-hancom2020-saved-2020.pdf` 와 대조한다.
//!
//! T4 의 세로 오프셋을 그 밀린 저장 줄에서 재면 T4 가 T3 띠만큼(87.5px) 내려가
//! 뒤 본문과 겹친다.

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue8156/titled-host-hancom2020-saved.hwp";
const OUTER: f64 = 283.0 * 96.0 / 7200.0;
const T4_OFFSET: f64 = 9000.0 * 96.0 / 7200.0;

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

fn near(what: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0,
        "{what}: 실제 {actual:.1}, 기대 {expected:.1} (±1px)"
    );
}

#[test]
fn saved_titled_host_measures_offset_float_from_paragraph_origin() {
    let doc = load(SAMPLE);
    assert_eq!(doc.page_count(), 4, "정본 PDF 와 같은 4쪽");

    // K2·K4: T3 와 T4 는 같은 문단 원점·같은 위 여백이라 두 윗변의 차이가 곧 오프셋이다.
    // host 첫 글줄은 저장 줄대로 T3 띠 끝에, 뒤 글줄은 T4 띠 끝에 선다.
    for (page, titled, after_t4) in [(1u32, 15u64, 16u64), (3, 27, 28)] {
        let t = tree(&doc, page);
        let (t3_top, t3_bottom) = table(&t, titled, 0);
        let (t4_top, t4_bottom) = table(&t, titled, 1);
        let k = page + 1;
        near(
            &format!("K{k} T4 윗변 − T3 윗변"),
            t4_top - t3_top,
            T4_OFFSET,
        );
        let (host_top, host_bottom) = line(&t, titled);
        near(&format!("K{k} host 첫 글줄"), host_top, t3_bottom + OUTER);
        assert!(
            host_bottom <= t4_top,
            "K{k} host 글줄 {host_top:.1}..{host_bottom:.1} 은 T4 윗변 {t4_top:.1} 위"
        );
        near(
            &format!("K{k} T4 아래 첫 글줄"),
            line(&t, after_t4).0,
            t4_bottom + OUTER,
        );
    }

    // K1·K3: T4 의 자연 위치가 앞 빈 host 의 T2 띠 안이라 띠 끝 아래에 쌓인다
    // (T4 윗변 = T2 아랫변 + T2 아래 여백 + T4 위 여백). host 글줄은 T4 띠 끝에 선다.
    for (page, empty_host, titled) in [(0u32, 2u64, 5u64), (2, 20, 21)] {
        let t = tree(&doc, page);
        let (_, t2_bottom) = table(&t, empty_host, 1);
        let (t3_top, _) = table(&t, titled, 0);
        let (t4_top, t4_bottom) = table(&t, titled, 1);
        let k = page + 1;
        assert!(t3_top < t4_top, "K{k} T3 는 T4 보다 위");
        near(&format!("K{k} T4 윗변"), t4_top, t2_bottom + OUTER + OUTER);
        near(
            &format!("K{k} T4 아래 host 글줄"),
            line(&t, titled).0,
            t4_bottom + OUTER,
        );
    }
}
