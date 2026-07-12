//! Issue #2207: 셀 내부 앵커 그림(글앞으로)의 vert=Para 기준점 오류.
//!
//! `samples/basic/issue1994_behindtext_table_20200830.hwp` p1 헤더 표 제목 셀의
//! 전화 금지 픽토그램(wrap=InFrontOfText, vert=Para off=91, 12.0×11.3mm)이
//! compose 후 전진된 para_y 를 앵커로 사용해 앵커 문단 줄 높이(1800 HU ≈ 24px)만큼
//! 아래로 배치되고 셀 클립에서 하단이 잘렸다.
//!
//! 정정: #577 의 앵커-시점 기준(첫 LINE_SEG vpos)을 오버레이 wrap(글뒤로/글앞으로)
//! + Para 조합으로 확장 (table_layout.rs, overlay_para).
//!
//! 기대값: 한컴 2022 PDF 실측 잉크 top 65px ↔ 이미지 요소 y ≈ 63.6px.
//! 수정 전 y = 87.56px.

use std::fs;
use std::path::Path;

#[test]
fn issue_2207_cell_overlay_picture_anchors_at_paragraph_top() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue1994_behindtext_table_20200830.hwp");
    let svg = doc.render_page_svg_native(0).expect("render page 1 SVG");

    // 픽토그램: 헤더 영역(x>400, y<150)의 중첩 <svg ...><image ...> (12.0×11.3mm
    // ≈ 45.5×42.8px). 파서 없이 좌표 속성만 추출한다.
    let mut found = None;
    for frag in svg.split("<svg ").skip(2) {
        // skip(2): 루트 <svg 와 split 선두 조각 이후부터
        let head = &frag[..frag.find('>').unwrap_or(frag.len())];
        let attr = |k: &str| -> Option<f64> {
            let pat = format!("{k}=\"");
            let s = head.find(&pat)? + pat.len();
            head[s..].split('"').next()?.parse().ok()
        };
        let (Some(x), Some(y), Some(w), Some(h)) =
            (attr("x"), attr("y"), attr("width"), attr("height"))
        else {
            continue;
        };
        if x > 400.0 && y < 150.0 && (44.0..47.0).contains(&w) && (41.0..44.0).contains(&h) {
            found = Some((x, y, w, h));
            break;
        }
    }
    let (_, y, _, h) = found.expect("헤더 셀 픽토그램 이미지 요소를 찾지 못함");

    // 앵커 문단 시작 기준이면 y ≈ 63.6 (한컴 잉크 top 65). 회귀(compose 후
    // para_y 기준)면 y = 87.6 — 한 줄 높이만큼 하강 + 셀 클립 절단.
    assert!(
        y < 70.0,
        "픽토그램이 앵커 문단 시작이 아니라 아래로 밀림: y={y:.1} (기대 ≈63.6, 회귀 시 87.6) — #2207"
    );
    // 하단도 헤더 제목 셀 영역(≈ y 61~112px) 안에 있어야 클립 절단이 없다.
    assert!(
        y + h < 115.0,
        "픽토그램 하단이 셀 영역을 벗어남: bottom={:.1} — #2207 클리핑 회귀",
        y + h
    );
}
