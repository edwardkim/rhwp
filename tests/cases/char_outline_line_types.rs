//! 글자 외곽선 종류(1~6)를 한컴 인쇄와 같은 pen과 반복 구간으로 긋는다.
//!
//! rhwp로 만든 시험 문서를 한컴 한글 12.30(Mac)에서 인쇄한 PDF
//! (`pdf/char_outline_types/`)는 외곽선 글자를 채우지 않고 윤곽만 긋는다. pen은
//! 글자 크기(24pt)와 무관한 1/600 inch 한 칸이고 굵은 실선(3)만 두 칸이다. 끝과 꺾임은
//! 둥글다. 반복 구간은 칸 단위로 점선(2) [3 3], 파선(4) [14 6], 일점쇄선(5) [10 6 3 6],
//! 이점쇄선(6) [11 3 3 3 3 3]이다. 수정 전 rhwp SVG는 외곽선을 무시하고 글자를 채워
//! 그렸고, Canvas는 종류와 상관없이 글자 크기 1/25 굵기의 실선을 그렸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;

const SAMPLE: &str = "tests/fixtures/char_outline_types/outline.hwp";

/// 24pt(32px) 글자 `가`의 SVG `<text>` 속성을 문서 순서대로 모은다.
fn outline_rows(svg: &str) -> Vec<&str> {
    svg.split("<text ")
        .skip(1)
        .filter(|tail| tail.contains("font-size=\"32\"") && tail.contains(">가<"))
        .map(|tail| tail.split('>').next().unwrap())
        .collect()
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    Some(&tag[start..start + tag[start..].find('"')?])
}

#[test]
fn outline_types_use_hancom_print_pen_and_dash() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let unit = 96.0 / 600.0;
    let expected: [Option<(f64, &[f64])>; 7] = [
        None,
        Some((1.0, &[])),
        Some((1.0, &[3.0, 3.0])),
        Some((2.0, &[])),
        Some((1.0, &[14.0, 6.0])),
        Some((1.0, &[10.0, 6.0, 3.0, 6.0])),
        Some((1.0, &[11.0, 3.0, 3.0, 3.0, 3.0, 3.0])),
    ];
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let svg = core
            .render_page_svg_layer_with_profile_native(0, profile)
            .expect("page 1 svg");
        let rows = outline_rows(&svg);
        assert_eq!(rows.len(), expected.len(), "{profile:?} 외곽선 시험 줄");
        for (kind, (tag, want)) in rows.iter().zip(expected).enumerate() {
            let Some((width, dash)) = want else {
                assert_eq!(attr(tag, "stroke"), None, "{profile:?} 외곽선 없음");
                assert_eq!(attr(tag, "fill"), Some("#000000"));
                continue;
            };
            assert_eq!(
                attr(tag, "fill"),
                Some("none"),
                "{profile:?} {kind}번 채우지 않음"
            );
            assert_eq!(attr(tag, "stroke"), Some("#000000"));
            assert_eq!(attr(tag, "stroke-linecap"), Some("round"));
            assert_eq!(attr(tag, "stroke-linejoin"), Some("round"));
            let stroke: f64 = attr(tag, "stroke-width").unwrap().parse().unwrap();
            assert!(
                (stroke - width * unit).abs() < 1e-9,
                "{kind}번 굵기 {stroke}"
            );
            let got: Vec<f64> = attr(tag, "stroke-dasharray")
                .map(|v| v.split(' ').map(|n| n.parse().unwrap()).collect())
                .unwrap_or_default();
            let want: Vec<f64> = dash.iter().map(|v| v * unit).collect();
            assert_eq!(got.len(), want.len(), "{kind}번 반복 구간 {got:?}");
            assert!(got.iter().zip(&want).all(|(a, b)| (a - b).abs() < 1e-9));
        }
    }
}
