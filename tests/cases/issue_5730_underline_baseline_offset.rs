//! [Issue #5730] 밑줄은 기준선 아래 **0.17em** 에 온다 — 고정 2.0px 이 아니다.
//!
//! 한글 2022 COM 프로브(9/12/15/18/24/36pt 실선 밑줄, PDF 드로잉 좌표 실측):
//! 밑줄-기준선 간격은 전 크기에서 0.167~0.170em 으로 선형이다. 고정 2.0px 은
//! 11.8px 글꼴에서만 우연히 맞고, 제목처럼 큰 글꼴(24px)에서는 한글 4.2px vs
//! rhwp 0.8px 로 밑줄이 디센더를 가로질렀다(156467175 실측).
//!
//! 픽스처 `samples/issue5730/underline_probe.hwp` 는 그 프로브 문서 자체다 —
//! 한글 2022 가 만든 6개 크기의 실선 밑줄 문단.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5730/underline_probe.hwp";

/// SVG 에서 (font-size, 기준선 y, x시작) 텍스트와 <line> (x1, y) 목록을 뽑는다.
fn extract(svg: &str) -> (Vec<(f64, f64, f64)>, Vec<(f64, f64)>) {
    let mut texts = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let start = cap.find(&key)? + key.len();
            let end = start + cap[start..].find('"')?;
            cap[start..end].parse().ok()
        };
        if let (Some(x), Some(y), Some(fs)) = (attr("x"), attr("y"), attr("font-size")) {
            if cap.contains(">H<") || cap.contains(">Hx") {
                texts.push((fs, y, x));
            }
        }
    }
    let mut lines = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let start = cap.find(&key)? + key.len();
            let end = start + cap[start..].find('"')?;
            cap[start..end].parse().ok()
        };
        if let (Some(x1), Some(y1)) = (attr("x1"), attr("y1")) {
            lines.push((x1, y1));
        }
    }
    (texts, lines)
}

#[test]
fn issue_5730_underline_sits_at_017em_below_baseline() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("read underline probe");
    let core = DocumentCore::from_bytes(&bytes).expect("open probe");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    let (texts, lines) = extract(&svg);
    // 글자마다 <text> 가 따로 나오므로 run 대표는 (크기, 기준선)별 최소 x 다 —
    // 밑줄 <line> 의 x1 은 run 시작 x 에서만 출발한다.
    let mut runs: Vec<(f64, f64, f64)> = Vec::new();
    for &(fs, y, x) in &texts {
        if let Some(run) = runs
            .iter_mut()
            .find(|(rfs, ry, _)| (*rfs - fs).abs() < 0.5 && (*ry - y).abs() < 0.5)
        {
            if x < run.2 {
                run.2 = x;
            }
        } else {
            runs.push((fs, y, x));
        }
    }
    // 프로브는 9~36pt 6개 크기 — 최소 4개 크기가 잡혀야 검증이 유효하다.
    let mut sizes: Vec<f64> = runs.iter().map(|t| t.0).collect();
    sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sizes.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    assert!(sizes.len() >= 4, "프로브 글꼴 크기 표본 부족: {sizes:?}");

    for &(fs, baseline, x) in &runs {
        // 그 run 의 시작 x 에서 출발하는 밑줄 선을 찾는다.
        let expected = fs * 0.17;
        let hit = lines
            .iter()
            .filter(|(x1, _)| (x1 - x).abs() < 1.0)
            .map(|(_, y1)| y1 - baseline)
            .filter(|dy| *dy > 0.0 && *dy < fs)
            .min_by(|a, b| {
                (a - expected)
                    .abs()
                    .partial_cmp(&(b - expected).abs())
                    .unwrap()
            });
        let dy = hit.unwrap_or_else(|| panic!("font-size {fs} run 의 밑줄이 없다"));
        assert!(
            (dy - expected).abs() <= fs * 0.02 + 0.1,
            "font-size {fs}: 밑줄 간격 {dy:.2} ≠ 기대 {expected:.2} (0.17em, 한글 2022 실측)"
        );
    }
}
