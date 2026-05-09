//! Issue #702: shortcut.hwp Distribute 다단 + Page/Column break 시 ColumnDef
//! 후속 갱신 누락 회귀.
//!
//! 본질:
//! 1. Distribute (배분) 다단의 짧은 컬럼 (3 items 등) 에서 inter-paragraph
//!    vpos-reset 임계값 (`pv > 5000`) 미달로 column-advance 미발동.
//!    → 6항목이 col 0 에 적층 (PDF 는 3+3 분배 기대).
//!
//! 2. [쪽나누기] / [단나누기] 가 새 ColumnDef 를 동반할 때 (shortcut.hwp p2
//!    파일/미리보기/편집 sections) 새 ColumnDef 미적용. col_count 가 이전
//!    zone 값 유지 → 페이지 분기 폭주.
//!
//! 정정 후 검증:
//! - 페이지 1: 지우기 섹션 이 단 5 (items=3) + 단 6 (items=3) 로 분할
//! - 페이지 2: 파일 + 미리보기 + 편집 섹션 모두 동일 페이지 (각 2단 분배)
//! - 총 페이지 수 ≤ 8 (기존 10 → 8 또는 7 로 감축, PDF 7쪽 정합)

use std::fs;
use std::path::Path;

#[test]
fn shortcut_distribute_short_column_split() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/basic/shortcut.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse shortcut.hwp");

    let page_count = doc.page_count();
    assert!(
        page_count <= 8,
        "회귀: shortcut.hwp 페이지 수 폭주. 기대 ≤ 8 (PDF 7쪽 정합 목표), 실제 {page_count}",
    );
}

#[test]
fn shortcut_page2_has_three_sections() {
    // 페이지 2 SVG 에 파일/편집 섹션 헤더 모두 존재해야 함.
    // (회귀 시 파일 만 표시되고 편집은 페이지 3 으로 밀려남)
    //
    // SVG renderer 는 글자별로 `<text x=".." y="..">char</text>` emit 하므로
    // 직접 substring 검색 대신 글자 수와 동일 y 좌표 클러스터링을 사용한다.
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/basic/shortcut.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse shortcut.hwp");

    let svg = doc
        .render_page_svg_native(1)
        .expect("render shortcut.hwp page 2");

    // SVG 에서 모든 <text> 의 y 좌표 + 단일 글자 추출 후 동일 y 로 묶어 라인 텍스트 복원.
    use std::collections::BTreeMap;
    let mut by_y: BTreeMap<i32, Vec<(f64, String)>> = BTreeMap::new();
    let mut i = 0;
    while i < svg.len() {
        let Some(rel) = svg[i..].find("<text ") else { break };
        let abs = i + rel;
        let after = &svg[abs + 6..];
        let Some(close) = after.find('>') else { i = abs + 6; continue };
        let attrs = &after[..close];
        let content_start = abs + 6 + close + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else { i = abs + 6; continue };
        let content = &svg[content_start..content_start + end_rel];
        // SVG renderer 는 `transform="translate(x,y) scale(..)"` 로 좌표 인코딩.
        let xy = attrs.find("translate(").and_then(|p| {
            let s = p + "translate(".len();
            let e = attrs[s..].find(')')? + s;
            let inner = &attrs[s..e];
            let mut parts = inner.split(',');
            let x = parts.next()?.trim().parse::<f64>().ok()?;
            let y = parts.next()?.trim().parse::<f64>().ok()?;
            Some((x, y))
        });
        if let Some((x, y)) = xy {
            let y_key = (y * 10.0).round() as i32;
            by_y.entry(y_key).or_default().push((x, content.to_string()));
        }
        i = content_start + end_rel + 7;
    }

    let mut has_file = false;
    let mut has_edit = false;
    for (_y, mut chars) in by_y {
        chars.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let line: String = chars.iter().map(|(_, s)| s.as_str()).collect();
        if line.contains("파일") { has_file = true; }
        if line.contains("편집") { has_edit = true; }
    }

    assert!(
        has_file && has_edit,
        "회귀: 페이지 2 에 파일/편집 섹션 모두 미존재. \
        Page/Column break + 새 ColumnDef 미적용 회귀 가능성. \
        파일={has_file} 편집={has_edit}"
    );
}
