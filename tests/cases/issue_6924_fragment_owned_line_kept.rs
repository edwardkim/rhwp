//! [Issue #6924] 쪽 조각이 **소유한** 글줄은 clip 바닥을 스쳐도 지우지 않는다.
//!
//! `suppress_bottom_clipped_text_residue`(#2007)는 clip 바닥에 6px 미만으로 걸친 글줄을
//! "다음 조각이 온전히 그릴 것" 이라 보고 `visible = false` 로 끈다. 그 전제는 조각 하나만
//! 봐서는 확인할 수 없었고, 실측에서 **소유자가 없는 줄까지 꺼져** 문서에서 통째로
//! 사라졌다 — 렌더 트리에는 정상 좌표로 남는데 SVG 에 한 자도 나가지 않는다.
//!
//! 분할 표 경로는 `partial_table_page_contains_cell_position` 으로 줄 단위 소속을 이미
//! 계산할 수 있다. 이 시험은 그 판정이 clip 보정까지 도달해, 조각이 소유한 줄이 살아남는지
//! 고정한다.
//!
//! 픽스처는 저장소 표본이다. `1490000-200800034_vietnam_labor_report.hwp` 114쪽의
//! `⑦ 물류 ⑧ 기타 ( )` 줄이 종전에는 **어느 쪽에도** 그려지지 않았다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue5714/1490000-200800034_vietnam_labor_report.hwp";
/// 조각이 소유하지만 종전에 소실되던 줄 (공백 제거 비교).
const LOST_LINE: &str = "⑦물류⑧기타";
/// 0-based 페이지 번호 (SVG 파일명 `_114` 는 1-based).
const PAGE: u32 = 113;

/// SVG 는 글자마다 `<text>` 를 따로 낸다 — x 순으로 이어 붙여야 줄 내용이 된다.
fn page_text_without_spaces(svg: &str) -> String {
    let mut glyphs: Vec<(i64, i64, String)> = Vec::new();
    for caps in svg.split("<text ").skip(1) {
        let Some(head_end) = caps.find('>') else {
            continue;
        };
        let (head, rest) = caps.split_at(head_end);
        let Some(body_end) = rest.find("</text>") else {
            continue;
        };
        let body = &rest[1..body_end];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let start = head.find(&key)? + key.len();
            let end = head[start..].find('"')? + start;
            head[start..end].parse().ok()
        };
        let (Some(x), Some(y)) = (attr("x"), attr("y")) else {
            continue;
        };
        glyphs.push(((y * 10.0) as i64, (x * 10.0) as i64, body.to_string()));
    }
    glyphs.sort();
    glyphs
        .into_iter()
        .map(|(_, _, t)| t)
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn issue6924_fragment_owned_bottom_line_is_painted() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue5714 sample");
    let svg = document.render_page_svg(PAGE).expect("render page");
    let text = page_text_without_spaces(&svg);
    assert!(
        text.contains(LOST_LINE),
        "조각이 소유한 clip 하단 글줄이 방출되지 않았다 (#6924): {LOST_LINE:?} 없음"
    );
}
