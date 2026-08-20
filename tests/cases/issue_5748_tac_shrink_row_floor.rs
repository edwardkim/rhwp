//! [Issue #5748] TAC 표 비례 축소가 '내용이 딱 맞는 행'까지 눌러 글자가 잘린다.
//!
//! 156682735 제목 표: rhwp 측정 합 14,608HU 가 선언 13,631HU 를 7.17% 넘어 축소가
//! 발동하는데, 종전 균일 배율은 내용이 정확히 9,286HU 인 0행까지 8,665HU 로 눌러
//! 제목 셋째 줄 baseline(322.76px)이 칸 클립 바닥(321.36px) 아래로 떨어졌다.
//! 한글 2022 는 저장 좌표에 여유가 있는 행(1행: 선언 4,345 > 내용 4,062)에서만
//! 부족분을 흡수한다 — 0행 9,286 / 1행 4,345.
//!
//! 수정: 행별 하한 = 저장 lineseg 내용 높이(pad + max(vertpos+vertsize)); 여유(slack)
//! 비례로만 줄이고, 하한 합이 선언을 넘으면 종전 균일 축소로 폴백.
//!
//! 픽스처 `samples/issue5748/tac_shrink_row_floor.hwpx` 는 원본에서 제목 표 문단까지만
//! 남긴 축소본 — 원본과 같은 행 기하(123.81/57.93px)를 재현한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5748/tac_shrink_row_floor.hwpx";

fn cell_clips(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for cap in svg.split("<clipPath id=\"cell-clip-").skip(1) {
        let Some(rect) = cap.find("<rect ") else {
            continue;
        };
        let seg = &cap[rect..cap[rect..]
            .find("/>")
            .map(|e| rect + e)
            .unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = seg.find(&key)? + key.len();
            let e = s + seg[s..].find('"')?;
            seg[s..e].parse().ok()
        };
        if let (Some(y), Some(h)) = (attr("y"), attr("height")) {
            out.push((y, h));
        }
    }
    out
}

#[test]
fn issue_5748_shrink_keeps_content_exact_row_unclipped() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    let clips = cell_clips(&svg);
    // 0행: 내용이 정확히 들어차 있으므로 눌리면 안 된다 — 한글 9,286HU = 123.81px.
    let title_row = clips
        .iter()
        .find(|(_, h)| (h - 123.81).abs() < 0.6)
        .unwrap_or_else(|| {
            panic!("제목 행 높이가 한글값(123.81px)이어야 한다 — 균일 축소 결함이면 115.5px. clips={clips:?}")
        });
    // 1행: 여유분이 부족분(13.03px)을 흡수해 한글 선언값 4,345HU = 57.93px 이 된다.
    assert!(
        clips.iter().any(|(_, h)| (h - 57.93).abs() < 0.6),
        "부제 행이 여유분 흡수 후 57.93px 이어야 한다 — 균일 축소 결함이면 66.2px. clips={clips:?}"
    );

    // 제목 셋째 줄 baseline 이 자기 행 클립 안에 있어야 한다 (종전 322.76 > 321.36 잘림).
    let mut title_baselines: Vec<f64> = Vec::new();
    for cap in svg.split("<text x=\"").skip(1) {
        if !cap.contains("font-size=\"33.3") {
            continue;
        }
        let Some(ys) = cap.find("y=\"") else { continue };
        let s = ys + 3;
        if let Some(e) = cap[s..].find('"') {
            if let Ok(y) = cap[s..s + e].parse::<f64>() {
                title_baselines.push(y);
            }
        }
    }
    let last_baseline = title_baselines.iter().cloned().fold(f64::MIN, f64::max);
    let row_bottom = title_row.0 + title_row.1;
    assert!(
        last_baseline < row_bottom - 1.0,
        "제목 마지막 줄 baseline({last_baseline:.2})이 행 클립 바닥({row_bottom:.2}) 안에 있어야 한다"
    );
}
