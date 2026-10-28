//! [Issue #6913] 테두리 굵기 축 — 한/글은 **1/600 inch 격자에 반올림**해서 그린다.
//!
//! 정본 PDF 의 stroke width 는 언제나 `units × 0.12 pt` 로 떨어진다. 격자 계산은 두
//! 단계다 — 선언 mm 를 HWPUNIT(1/7200 inch)으로 반올림한 뒤 그것을 600dpi 로
//! **half-up** 반올림한다.
//!
//! ```text
//!   정본 PDF(engine 2020, 한컴 12.0.0.4605) 1쪽 머리 표
//!     '국가보훈부' 칸  borderFill 14   0.2mm  → 0.60 pt =  5 units = 0.80 px
//!     가운데·오른쪽 칸 borderFill 13·10 0.12mm → 0.36 pt =  3 units = 0.48 px
//! ```
//!
//! 굵기가 칸마다 다른 것 **자체는 원본 선언**이라 결함이 아니다. 이 시험이 잠그는 것은
//! 그 두 값의 절대 크기, 곧 격자 축이다.
//!
//! 16단계 전부를 이 문서의 `borderFill 14` 굵기만 바꾼 단일 변수 실험으로 쟀고
//! 16/16 이 격자와 일치했다 (`samples/issue6913/README.md`). 종전 표
//! (`mm × 96/25.4` 를 소수 첫째자리로 반올림)는 격자를 못 맞췄고, 가장 많이 쓰이는
//! 0.1mm 를 0.4px 로 **25% 두껍게** 그렸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const SAMPLE: &str = "samples/issue6913/156591199-veterans-joint-burial-press-release.hwpx";

/// 정본 0.60 pt.
const THICK_PX: f64 = 0.80;
/// 정본 0.36 pt.
const THIN_PX: f64 = 0.48;

fn page_svg(page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 읽기 ({}): {e}", path.display()));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("문서 로드: {e:?}"));
    doc.render_page_svg(page)
        .unwrap_or_else(|e| panic!("쪽 {page} SVG: {e:?}"))
}

/// `<line …>` 중 y 가 `y_max` 위인 것들의 `stroke-width` 값.
fn stroke_widths_above(svg: &str, y_max: f64) -> Vec<f64> {
    let mut out = Vec::new();
    for tag in svg.split('<').filter(|t| t.starts_with("line ")) {
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let i = tag.find(&key)? + key.len();
            let rest = &tag[i..];
            let end = rest.find('"')?;
            rest[..end].parse::<f64>().ok()
        };
        let Some(w) = attr("stroke-width") else {
            continue;
        };
        let ys: Vec<f64> = ["y1", "y2"].iter().filter_map(|n| attr(n)).collect();
        if !ys.is_empty() && ys.iter().cloned().fold(f64::MIN, f64::max) <= y_max {
            out.push(w);
        }
    }
    out
}

/// 1쪽 머리 표의 두 굵기가 정본 값이다.
#[test]
fn header_table_borders_match_the_2020_oracle() {
    let svg = page_svg(0);
    let widths = stroke_widths_above(&svg, 200.0);
    assert!(
        !widths.is_empty(),
        "1쪽 상단에서 테두리 선을 하나도 못 찾았다 — 선택자가 깨졌다"
    );

    let thick = widths.iter().filter(|w| **w > 0.6).count();
    let thin = widths.iter().filter(|w| **w <= 0.6).count();
    assert!(
        thick > 0 && thin > 0,
        "머리 표는 0.2mm 와 0.12mm 를 같이 쓴다 — got {widths:?}"
    );

    for w in &widths {
        let target = if *w > 0.6 { THICK_PX } else { THIN_PX };
        assert!(
            (w - target).abs() < 1e-6,
            "테두리 굵기가 정본 축(1mm = 4px)에서 벗어났다 — got {w}, want {target}. \
             전체 {widths:?}"
        );
    }
}

/// 굵은 선은 '국가보훈부' 칸 자기 변 3개뿐이고 나머지 4개는 얇다 — 정본과 같은 갈림.
///
/// 정본 1쪽 상단의 선 집합(중복 제외): 0.60pt 3개(왼쪽 세로 · 왼쪽 위 가로 · 왼쪽 아래
/// 가로)와 0.36pt 4개(오른쪽 세로 · 오른쪽 위 가로 · 오른쪽 아래 가로 · y≈169.4 밑줄).
#[test]
fn only_the_first_cell_edges_are_thick() {
    let svg = page_svg(0);
    let widths = stroke_widths_above(&svg, 200.0);
    let thick = widths
        .iter()
        .filter(|w| (*w - THICK_PX).abs() < 1e-6)
        .count();
    let thin = widths
        .iter()
        .filter(|w| (*w - THIN_PX).abs() < 1e-6)
        .count();
    assert_eq!(
        (thick, thin),
        (3, 4),
        "정본은 0.2mm 3개 · 0.12mm 4개로 갈린다 — got {widths:?}"
    );
}
