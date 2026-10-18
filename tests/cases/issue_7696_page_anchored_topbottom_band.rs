//! [#7696] 쪽·종이 기준 자리차지(TopAndBottom) 개체의 세로 밴드.
//!
//! 자리차지 개체는 그 세로 구간에서 본문을 받지 않는다. 밴드 안에서 시작하는 문단은 밴드
//! 아래로 가고, 밴드가 본문 아래까지 이어지면 다음 쪽으로 간다. 수정 전 rhwp 는 문단 기준
//! 개체만 흐름에 계상해, 쪽·종이 기준 개체의 밴드에는 줄이 그대로 들어갔다(재조판 입력
//! 2쪽 — 정본 4쪽). 배치는 앵커 바로 뒤 문단부터 밴드 아래로 점프해 밴드 위 줄을 잃었다.
//!
//! 입력 `samples/issue7696/page-anchored-band.hwpx` 는 생성기 `gen_page_anchored_band.py` 의
//! 저장 줄 없는 HWPX(재조판 경로)이고, `page-anchored-band-2024.hwp` 는 한/글 2024(13.0.0.564)가
//! 이를 HWP 로 저장한 것(저장 줄 경로)이다. 정본 `pdf/issue7696/page-anchored-band-2024.pdf` 는
//! 같은 한/글의 Print 출력이다.
//!
//! ```text
//!   정본 쪽   내용                                  밴드
//!   1        제목·사례 1·앵커·줄 1..24            종이 기준 50000HU~84000HU (본문 아래까지)
//!   2        사례 1 줄 25..40
//!   3        사례 2·앵커·줄 1..32 (줄 11 은 밴드 바로 아래)   쪽 기준 20000HU~35000HU
//!   4        사례 2 줄 33..40
//! ```
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

/// 문단 순번: 0 제목, 1 「사례 1」, 2 앵커, 3..=42 줄 1..40, 43 「사례 2」, 44 앵커, 45..=84 줄 1..40.
const PAGE_PARAS: [std::ops::RangeInclusive<u64>; 4] = [0..=26, 27..=42, 43..=76, 77..=84];
/// 사례 2 줄 11 — 밴드 바로 아래에서 시작한다.
const FIRST_LINE_BELOW_CASE2_BAND: u64 = 55;

struct PageGeometry {
    /// 본문 글줄 (문단, 위, 아래). 개체 안 글줄은 뺀다.
    lines: Vec<(u64, f64, f64)>,
    /// 자리차지 사각형 (위, 아래).
    bands: Vec<(f64, f64)>,
}

fn page_geometry(doc: &HwpDocument, page: u32) -> PageGeometry {
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
    let mut geometry = PageGeometry {
        lines: Vec::new(),
        bands: Vec::new(),
    };
    fn walk(node: &Value, in_shape: bool, out: &mut PageGeometry) {
        let kind = node["type"].as_str().unwrap_or_default();
        let y = node["bbox"]["y"].as_f64().unwrap_or(0.0);
        let bottom = y + node["bbox"]["h"].as_f64().unwrap_or(0.0);
        if kind == "Rect" && !in_shape {
            out.bands.push((y, bottom));
        } else if kind == "TextLine" && !in_shape {
            out.lines
                .push((node["pi"].as_u64().unwrap_or(u64::MAX), y, bottom));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, in_shape || kind == "Rect", out);
        }
    }
    walk(&tree, false, &mut geometry);
    geometry
}

fn check(sample: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    assert_eq!(doc.page_count(), 4, "{sample}: 정본 PDF 와 같은 4쪽");
    for (page, expected) in PAGE_PARAS.iter().enumerate() {
        let geometry = page_geometry(&doc, page as u32);
        let paras: Vec<u64> = geometry.lines.iter().map(|line| line.0).collect();
        assert_eq!(
            paras,
            expected.clone().collect::<Vec<_>>(),
            "{sample} {}쪽: 문단 소속·순서가 정본과 같아야 한다",
            page + 1
        );
        for &(top, bottom) in &geometry.bands {
            for &(pi, line_top, line_bottom) in &geometry.lines {
                assert!(
                    line_bottom <= top + 0.5 || line_top >= bottom - 0.5,
                    "{sample} {}쪽: 문단 {pi} 글줄 {line_top:.1}..{line_bottom:.1} 이 \
                     자리차지 밴드 {top:.1}..{bottom:.1} 에 들어갔다",
                    page + 1
                );
            }
        }
        if page == 2 {
            let &(band_top, band_bottom) = geometry.bands.first().expect("3쪽 밴드");
            let below = geometry
                .lines
                .iter()
                .find(|line| line.0 == FIRST_LINE_BELOW_CASE2_BAND)
                .expect("3쪽 줄 11");
            assert!(
                (below.1 - band_bottom).abs() <= 1.0,
                "{sample} 3쪽: 줄 11 은 밴드 하단 {band_bottom:.1} 에서 시작한다 (실제 {:.1})",
                below.1
            );
            let above = geometry
                .lines
                .iter()
                .find(|line| line.0 == FIRST_LINE_BELOW_CASE2_BAND - 1)
                .expect("3쪽 줄 10");
            assert!(
                above.2 <= band_top,
                "{sample} 3쪽: 줄 10 은 밴드 위 {band_top:.1} 에 남는다 (실제 하단 {:.1})",
                above.2
            );
        }
    }
}

#[test]
fn page_anchored_topbottom_band_reflowed_input() {
    check("samples/issue7696/page-anchored-band.hwpx");
}

#[test]
fn page_anchored_topbottom_band_hancom_saved_input() {
    check("samples/issue7696/page-anchored-band-2024.hwp");
}
