//! [#7874] 저장 줄 위치가 문단 아래 간격을 빼먹은 사다리.
//!
//! 저장 사다리가 다음 문단 첫 줄을 앞 문단 마지막 줄 끝에 바로 두면(아래 간격 누락), 흐름이
//! 그 아래 간격만큼 앞서 있는 것은 드리프트가 아니라 간격을 소비한 것이다. 수정 전 rhwp 는
//! 아래 간격(568HU = 7.57px)이 후방 스냅 허용(8px) 안이라 낡은 저장 위치로 문단마다 되감아,
//! 문단이 한/글보다 7.57px 씩 위로 붙고 그 차가 쪽 아래로 쌓였다.
//!
//! 입력 `samples/issue7874/omitted-spacing-after.hwpx` 는 생성기 `gen_omitted_spacing_after.py` 의
//! 출력이고, 정본 `pdf/issue7874/omitted-spacing-after-2024.pdf` 는 한/글 2024(13.0.0.564)가 이 HWPX
//! 를 Print 한 것이다. 한/글은 1쪽(저장 위치가 간격을 뺌)과 2쪽(담음)을 같은 피치
//! 28.9px = (줄 높이 1000 + 줄간격 600 + 아래 간격 568)HU 로 그린다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue7874/omitted-spacing-after.hwpx";
const EXPECTED_PITCH_PX: f64 = (1000.0 + 600.0 + 568.0) * 96.0 / 7200.0;

/// 쪽의 본문 글줄 (문단, 위끝).
fn body_lines(doc: &HwpDocument, page: u32) -> Vec<(u64, f64)> {
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
    let mut lines = Vec::new();
    fn walk(node: &Value, in_body: bool, out: &mut Vec<(u64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        let in_body = in_body || kind == "Body";
        if in_body && kind == "TextLine" {
            out.push((
                node["pi"].as_u64().unwrap_or(u64::MAX),
                node["bbox"]["y"].as_f64().unwrap_or(0.0),
            ));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, in_body, out);
        }
    }
    walk(&tree, false, &mut lines);
    lines
}

#[test]
fn paragraphs_keep_spacing_after_omitted_by_stored_ladder() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    assert_eq!(doc.page_count(), 2, "정본 PDF 와 같은 2쪽");
    for (page, paragraphs) in [(0u32, 1u64..=30), (1, 31..=60)] {
        let lines = body_lines(&doc, page);
        let owned: Vec<u64> = lines.iter().map(|line| line.0).collect();
        for pi in paragraphs.clone() {
            assert!(
                owned.contains(&pi),
                "{}쪽에 문단 {pi} 가 있어야 한다",
                page + 1
            );
        }
        let tops: Vec<(u64, f64)> = lines
            .into_iter()
            .filter(|(pi, _)| paragraphs.contains(pi))
            .collect();
        for pair in tops.windows(2) {
            let ((a, ya), (b, yb)) = (pair[0], pair[1]);
            assert!(
                ((yb - ya) - EXPECTED_PITCH_PX).abs() <= 1.0,
                "{}쪽 문단 {a}→{b}: 피치는 줄 높이 + 줄간격 + 아래 간격 {EXPECTED_PITCH_PX:.1}px \
                 (실제 {:.1}; 수정 전 1쪽은 아래 간격이 되감겨 21.3)",
                page + 1,
                yb - ya
            );
        }
    }
}
