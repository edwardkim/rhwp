//! 표 셀 내 문단 자동 번호 (head_type=Number/Outline) 정합 회귀 가드.
//!
//! 본문 paragraph path 는 `apply_paragraph_numbering` 을 호출하여
//! head_type=Number/Outline paragraph 의 "1.", "2." 등 자동 번호를
//! 생성하지만, 표 셀 paragraph path 3곳은 호출하지 않아 셀 안 paragraph 의
//! 번호가 누락된다. 본 fix 는 셀 path 3곳 (`table_cell_content.rs`,
//! `table_layout.rs`, `table_partial.rs`) 에 동일 호출을 추가한다.
//!
//! 권위 자료: `pdf/k-water-rfp-2024.pdf` (한글 2024 편집기, 정답지 등급 ★★).
//! 같은 자료 페이지 18 (파일상 page 20) 예정공정표 28×13 표에서
//! 한컴은 "1. 서버 클라우드 환경 구축" / "2. 전사 데이터 허브 구축" /
//! "3. SaaS" 모두 번호 prefix 를 표시한다 (작업지시자 PDF 직접 확인).
//!
//! 가드 대상: `samples/hwpx/k-water-rfp.hwpx` 페이지 20 의 셀 안 paragraph
//! 번호 prefix 존재 여부 — fix 적용 후 SVG 에 "1." / "2." 가 셀 안 line
//! 시작부에 나타나야 한다.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

fn render_page_svg(rel: &str, page_idx: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.render_page_svg_native(page_idx)
        .unwrap_or_else(|e| panic!("svg {}: {:?}", rel, e))
}

/// SVG 안에서 단일 문자 `ch` 를 그리는 `<text x="..." y="...">ch</text>` 의 모든
/// y 좌표를 오름차순으로 반환한다. `<text transform="translate(...)">ch</text>`
/// 형식(회전·스케일 텍스트)은 명시적 y 속성이 없어 본 가드에서 제외한다.
fn all_text_y_for(svg: &str, ch: char) -> Vec<f64> {
    let needle = format!(">{}</text>", ch);
    let mut ys = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = svg[cursor..].find(&needle) {
        let end = cursor + rel;
        let tag_start = svg[..end]
            .rfind("<text ")
            .unwrap_or_else(|| panic!("'{}' 매치 직전 <text 태그 없음", ch));
        let tag = &svg[tag_start..end];
        cursor = end + needle.len();
        // transform 기반 텍스트는 y 속성이 없음 → 본 가드 대상 외, skip.
        if let Some(y_off) = tag.find(" y=\"") {
            let y_start = y_off + 4;
            let y_end = tag[y_start..]
                .find('"')
                .unwrap_or_else(|| panic!("'{}' y 속성 종료 따옴표 없음", ch));
            if let Ok(y) = tag[y_start..y_start + y_end].parse::<f64>() {
                ys.push(y);
            }
        }
    }
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys
}

#[test]
fn cell_paragraph_number_prefix_appears_in_k_water_rfp_p20() {
    let svg = render_page_svg("samples/hwpx/k-water-rfp.hwpx", 19);

    // 페이지 20 예정공정표 셀 안 헤딩의 y 좌표(작업지시자 PDF 확인):
    //   "1. 서버 클라우드 환경 구축"  ≈ y=270
    //   "2. 전사 데이터 허브 구축"   ≈ y=558
    // fix 미적용 시 "1." "2." 가 누락되어 셀 안에 단 한 개의 "1." 도 없을 수 있다.
    // 정합된 fix 적용 후에는 셀 안 두 y 위치(≈270, ≈558) 에 "1." / "2." 가 존재.
    let dot_ys = all_text_y_for(&svg, '1');
    let near_270 = dot_ys.iter().any(|&y| (y - 270.0).abs() < 5.0);
    let two_ys = all_text_y_for(&svg, '2');
    let near_558 = two_ys.iter().any(|&y| (y - 558.0).abs() < 5.0);

    assert!(
        near_270,
        "k-water-rfp.hwpx p20: 표 셀 안 '1.' 헤딩이 y≈270 에 출현해야 한다. \
         미출현 시 head_type=Number 셀 paragraph 의 자동번호 미적용 결함. \
         (작업지시자 한컴 2024 PDF 확인 — 한컴 정답에 '1. 서버 클라우드 환경 구축' 존재)"
    );
    assert!(
        near_558,
        "k-water-rfp.hwpx p20: 표 셀 안 '2.' 헤딩이 y≈558 에 출현해야 한다. \
         미출현 시 head_type=Number 셀 paragraph 의 자동번호 미적용 결함."
    );
}
