//! [#7412] 셀 내용 폭은 한/글처럼 4 HWPUNIT 격자에 내려서 줄을 나눈다.
//!
//! ## 독립 근거
//!
//! - 저장본: `samples/**/*.hwpx` 56문서의 `hp:lineseg/@horzsize` 19,837개 중 19,766개
//!   (99.64%)가 4의 배수다. 셀 안 줄도 예외가 아니다.
//! - 정본 출력: `pdf/80168_regulatory_analysis-2022.pdf`(한/글 2022 12.0.0.4547)와
//!   `pdf/80168_regulatory_analysis-hwp-2024.pdf`(한/글 2024) 121쪽의 「7.규제내용」 셀은
//!
//!   ```text
//!   업의 경우 100분의 30 이상 100분의 50 이하의 범위에서 시ㆍ도조
//!   례로 정하는 비율로 하고, 공공분양주택 공급 비율은 100분의 60 이
//!   ```
//!
//!   로 끊긴다. 이 셀 문단의 저장 줄은 `horzsize=0` 자리표시 한 줄뿐이라 rhwp 가 새로
//!   줄을 나눈다. 셀 폭 35076 HU 에서 안 여백 225×2 를 빼면 내용 폭이 34626 HU 인데,
//!   격자 없이 34626 으로 나누면 「례」가 2 HU 안에 들어가 「…시ㆍ도조례 / 로…」가 된다.
//!   격자로 내린 34624 HU 에서는 정본처럼 「조」 뒤에서 끊긴다.
//!
//! 이 검사는 줄 소속(어느 줄이 어느 글자로 끝나고 시작하는지)만 본다. 위치·모양은
//! 같은 쪽의 Visual Sweep 으로 따로 확인한다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 정본 `pdf/80168_regulatory_analysis-2022.pdf`·`-hwp-2024.pdf` 121쪽(0-기반 120).
const PAGE: u32 = 120;

/// 쪽의 글자를 기준선(y)별 줄로 모은다. 공백은 버린다(정본 텍스트 추출과 같은 비교 키).
fn page_lines(sample: &str) -> Vec<(f64, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(PAGE)
            .expect("공개 text-layout"),
    )
    .expect("text-layout JSON");
    let mut rows: BTreeMap<i64, Vec<(f64, String)>> = BTreeMap::new();
    for run in layout["runs"].as_array().expect("runs") {
        let y = run["y"].as_f64().expect("run y");
        let x = run["x"].as_f64().expect("run x");
        let text: String = run["text"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        rows.entry((y * 10.0).round() as i64)
            .or_default()
            .push((x, text));
    }
    rows.into_values()
        .map(|mut runs| {
            runs.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("유한값"));
            (runs[0].0, runs.into_iter().map(|(_, t)| t).collect())
        })
        .collect()
}

fn assert_hancom_break(sample: &str) {
    let lines = page_lines(sample);
    let first = lines
        .iter()
        .position(|(_, line)| line.contains("100분의30이상100분의50이하의범위에서시ㆍ도"))
        .unwrap_or_else(|| {
            panic!("{sample}: 「7.규제내용」 둘째 줄이 121쪽에 있어야 한다 — {lines:#?}")
        });
    let (left, line) = &lines[first];
    // 같은 기준선 사이에 왼쪽 열의 셀 제목(「7.규제내용」)이 끼므로, 같은 셀 왼쪽 끝에서
    // 시작하는 다음 줄을 고른다.
    let next = lines[first + 1..]
        .iter()
        .find(|(x, _)| (x - left).abs() < 1.0)
        .map(|(_, text)| text)
        .expect("같은 셀의 다음 줄");
    assert!(
        line.ends_with("시ㆍ도조") && next.starts_with("례로정하는"),
        "{sample}: 한/글 2022·2024 PDF 는 「…시ㆍ도조 / 례로…」에서 끊는다(셀 내용 폭 34626→34624 HU).\n  줄: {line}\n  다음: {next}"
    );
}

#[test]
fn issue_7412_cell_reflow_breaks_on_the_4hu_grid_hwp() {
    assert_hancom_break("samples/80168_regulatory_analysis.hwp");
}

#[test]
fn issue_7412_cell_reflow_breaks_on_the_4hu_grid_hwpx() {
    assert_hancom_break("samples/issue1891/80168_regulatory_analysis.hwpx");
}
