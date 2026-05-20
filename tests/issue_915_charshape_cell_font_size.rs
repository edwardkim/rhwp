//! Issue #915: CharShape 적용 영역 결함 — 표 셀 글자 크기 이상 회귀 가드.
//!
//! 재현 문서: `samples/table-in-tbox.hwp` page 2 (0-indexed page=1).
//! 대상: Shape.TextBox > Table > 셀[1](r=0,c=1) 의 텍스트 "충남중부권지사".
//!
//! 본질: `CharShapeRef.start_pos` 의 의미를 Task #884 가 "visible 글자 인덱스"로
//! 바꿨으나(`composer.rs`), 그 변경이 일부 렌더/측정 경로에 미반영되어
//! 인라인 제어자(이미지/표 등)가 있는 문단·셀에서 char_shape 적용 영역이
//! 어긋난다. 결과: "충남중부권지사" 7글자가 올바른 텍스트 char_shape 대신
//! 인접 char_shape(HY수평선 계열, 극소 크기)를 받아 ≈1.33px 로 렌더됨.
//!
//! 회귀 가드 본질: 한글 음절 글리프가 5px 미만으로 렌더되면 char_shape 오적용.
//! 정상 본문 텍스트는 어떤 폰트·문맥에서도 5px 이상이다 (1.33px = 명백한 결함).
//!
//! 좌표/크기 측정: `render_page_svg_native` SVG 의 `<text font-size>` (96 DPI).

use std::fs;
use std::path::Path;

/// 렌더된 SVG 에서 `<text>` 요소를 (내용, font-size) 목록으로 추출한다.
fn text_glyphs(svg: &str) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = svg[from..].find("<text ") {
        let tag = from + rel;
        from = tag + 6;

        let after = &svg[tag..];
        let Some(gt) = after.find('>') else { break };
        let attrs = &after[..gt];
        let Some(end_rel) = after[gt + 1..].find("</text>") else {
            break;
        };
        let content = &after[gt + 1..gt + 1 + end_rel];

        let fs = {
            let Some(p) = attrs.find("font-size=\"") else {
                continue;
            };
            let s = p + 11;
            let Some(e) = attrs[s..].find('"') else {
                continue;
            };
            match attrs[s..s + e].parse::<f64>() {
                Ok(v) => v,
                Err(_) => continue,
            }
        };

        out.push((content.to_string(), fs));
    }
    out
}

#[test]
fn issue_915_cell_text_not_rendered_microscopic() {
    let repo = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo).join("samples/table-in-tbox.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse table-in-tbox.hwp");

    // 페이지 2 (0-indexed page=1) — 텍스트박스 안 표가 있는 페이지.
    let svg = doc
        .render_page_svg_native(1)
        .expect("render table-in-tbox.hwp page 2");

    let glyphs = text_glyphs(&svg);
    assert!(
        !glyphs.is_empty(),
        "table-in-tbox.hwp page 2 에 <text> 요소가 있어야 함"
    );

    // 한글 음절(U+AC00..U+D7A3) 글리프 중 5px 미만으로 렌더된 것 수집.
    // 정상 텍스트는 5px 이상 — 5px 미만 = CharShape 오적용(#915) ("충남중부권지사" ≈1.33px).
    let micro: Vec<&(String, f64)> = glyphs
        .iter()
        .filter(|(c, fs)| *fs < 5.0 && c.chars().any(|ch| ('\u{AC00}'..='\u{D7A3}').contains(&ch)))
        .collect();

    assert!(
        micro.is_empty(),
        "한글 글리프가 5px 미만으로 렌더됨 = CharShape 적용 영역 결함(#915).\n  \
         극소 렌더 {}건 (정상 ≥5px, 결함 시 ≈1.33px): {:?}\n  \
         원인: Task #884 의 start_pos=visible-index 해석이 렌더/측정 경로에 \
         미반영 — composer.rs 와 paragraph_layout.rs/find_active_char_shape 의 \
         start_pos 해석 일치 여부를 확인할 것.",
        micro.len(),
        micro,
    );
}
