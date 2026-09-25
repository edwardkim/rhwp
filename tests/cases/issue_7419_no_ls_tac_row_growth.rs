//! Issue #7419 — 저장 LINE_SEG 가 없는 글자처럼 취급(TAC) 표가 선언 높이로 눌려 아래 표와 겹친다.
//!
//! 생성기가 쓴 hwpx(셀 문단에 `linesegarray` 없음, `hp:tbl/hp:sz height` = 행별 `cellSz` 합)에서
//! 셀 글이 두 줄로 나뉘면, 측정기의 TAC 비례 축소가 행을 줄여 합을 선언(64px)에 맞추고
//! layout 의 선언 신뢰(`trust_declared_row_heights` viewtext 갈래)가 행을 셀 선언 [32, 32] 로 되돌린다.
//! 둘째 줄이 칸 밖으로 나가 다음 표 제목과 포개졌다. 한글은 행을 키운다(재저장 시 표 4800 → 5284HU,
//! 행 32.0 + 38.5px).
//!
//! - `samples/issue7419/tac_two_line_cell_no_lineseg.hwpx`: 1열 2행 TAC 표 둘, 첫 표 둘째 칸이 두 줄
//! - `samples/issue7419/tac_nested_two_line_cell_no_lineseg.hwpx`: 위 첫 표를 둘째 표 둘째 칸 안에 넣은 판
//!   (안쪽 표가 자란 만큼 바깥 칸도 커져야 한다 — 합성 줄이 선언 높이만 담던 결함)

use std::fs;
use std::path::Path;

/// SVG 의 `<rect x y width height>` 가운데 셀 클립 사각형(폭 500px 이상, 쪽 크기 제외)을 y 순으로.
fn cell_rects(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for frag in svg.split("<rect ").skip(1) {
        let head = &frag[..frag.find('>').unwrap_or(frag.len())];
        let attr = |k: &str| -> Option<f64> {
            let pat = format!(" {k}=\"");
            let s = format!(" {head}").find(&pat)? + pat.len() - 1;
            head[s..].split('"').next()?.parse().ok()
        };
        let (Some(y), Some(w), Some(h)) = (attr("y"), attr("width"), attr("height")) else {
            continue;
        };
        if w >= 500.0 && w < 700.0 && h < 800.0 {
            out.push((y, h));
        }
    }
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    out
}

fn render(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7419")
        .join(name);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse sample");
    doc.render_page_svg_native(0).expect("render page 1 SVG")
}

#[test]
fn issue_7419_two_line_cell_grows_row_and_next_table_follows() {
    let rects = cell_rects(&render("tac_two_line_cell_no_lineseg.hwpx"));
    assert!(
        rects.len() >= 4,
        "셀 사각형 4개(표 둘 × 행 둘)를 기대: {rects:?}"
    );
    let (row1_y, row1_h) = rects[1];
    // 한글 38.5px(2줄 + 상하 여백). 회귀 시 셀 선언 32.0px 로 되돌아간다.
    assert!(
        row1_h >= 38.0,
        "첫 표 둘째 행이 두 줄만큼 자라야 한다: h={row1_h:.1} (한글 38.5, 회귀 32.0)"
    );
    let next_top = rects[2].0;
    assert!(
        next_top >= row1_y + row1_h - 0.5,
        "둘째 표({next_top:.1})가 첫 표 끝({:.1}) 아래에서 시작해야 한다",
        row1_y + row1_h
    );
}

#[test]
fn issue_7419_nested_tac_table_growth_expands_host_cell() {
    let rects = cell_rects(&render("tac_nested_two_line_cell_no_lineseg.hwpx"));
    // 바깥 둘째 표의 둘째 칸이 안쪽 표(32.0 + 38.5 = 70.5px)와 상하 여백을 담아야 한다.
    // 회귀 시 안쪽 표 선언 높이 64.0px + 여백 = 67.8px 에 머물러 안쪽 표 둘째 줄이 칸 밖으로 나간다.
    let host = rects
        .iter()
        .copied()
        .filter(|(_, h)| *h > 60.0)
        .fold(0.0f64, |m, (_, h)| m.max(h));
    assert!(
        host >= 72.0,
        "바깥 칸이 자란 안쪽 표를 담아야 한다: h={host:.1} (기대 ≥72, 회귀 67.8)"
    );
    // 안쪽 표 둘째 행도 두 줄만큼 자라야 한다(바깥 칸 안쪽의 38px 대 사각형).
    assert!(
        rects
            .iter()
            .filter(|(_, h)| *h >= 38.0 && *h < 40.0)
            .count()
            >= 2,
        "첫 표와 안쪽 표의 둘째 행이 모두 38.5px 로 자라야 한다: {rects:?}"
    );
}
