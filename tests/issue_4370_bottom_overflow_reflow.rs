//! Issue #4370 후속: 페이지 하단 공간 부족 문단의 다음 쪽 리플로 회귀 가드.
//!
//! v0.7.6 은 페이지 하단 공간이 부족한 문단(연속 2개 문단, 그리고 페이지에 걸친
//! 문단의 꼬리 줄)을 다음 쪽으로 리플로하지 않고 페이지 높이 밖 y 좌표에 그대로
//! 방출해 내용이 시각적으로 소실됐고, v0.8.2 에서 수정이 확인됐다(이슈 리포트
//! 실측: y=1137~1157 > 페이지 높이 1122.5). 재현 문서가 비공개라 합성 문서
//! (빈 문서 + 여러 쪽을 채우는 장문 문단들)로 같은 불변식을 고정한다:
//!
//! 1. 어떤 페이지에서도 본문 텍스트의 y 좌표가 물리 페이지 높이를 넘지 않는다.
//! 2. 넘치는 내용은 소실되지 않고 다음 쪽에 실제로 배치된다(2쪽 이상 + 마지막
//!    쪽에도 텍스트 존재).

use rhwp::wasm_api::HwpDocument;

/// SVG 의 `height="..."` 속성값(물리 페이지 높이 px).
fn svg_height(svg: &str) -> f64 {
    let i = svg.find("height=\"").expect("svg height attr") + "height=\"".len();
    let rest = &svg[i..];
    let end = rest.find('"').expect("height close");
    rest[..end].parse().expect("height f64")
}

/// SVG 내 모든 `<text ... y="...">` 의 최대 y 좌표(px). 없으면 None.
fn max_text_y(svg: &str) -> Option<f64> {
    let mut max: Option<f64> = None;
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let after = &rest[open..];
        let tag_end = after.find('>').map(|g| open + g).unwrap_or(rest.len());
        let tag = &rest[open..tag_end];
        if let Some(yi) = tag.find(" y=\"") {
            let yrest = &tag[yi + 4..];
            if let Some(ye) = yrest.find('"') {
                if let Ok(y) = yrest[..ye].parse::<f64>() {
                    max = Some(max.map_or(y, |m: f64| m.max(y)));
                }
            }
        }
        rest = &rest[tag_end..];
    }
    max
}

/// 빈 문서에 장문 문단 `para_count` 개를 만든다. 각 문단은 여러 줄로 wrap 되는
/// 길이라 전체가 한 페이지를 확실히 넘긴다.
fn build_multi_page_doc(para_count: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    for _ in 1..para_count {
        doc.split_paragraph(0, 0, 0, None).expect("split paragraph");
    }
    let line = "하단 리플로 가드 문단 본문 텍스트 ".repeat(8);
    for i in 0..para_count {
        doc.insert_text(0, i as u32, 0, &line)
            .unwrap_or_else(|e| panic!("insert_text para {i}: {e:?}"));
    }
    doc
}

#[test]
fn bottom_short_paragraphs_reflow_to_next_page_not_clipped() {
    let doc = build_multi_page_doc(30);
    let pages = doc.page_count();
    assert!(
        pages >= 2,
        "합성 문서가 2쪽 이상이어야 리플로 경로를 검증한다: {pages}쪽"
    );

    for page in 0..pages {
        let svg = doc
            .render_page_svg_native(page)
            .unwrap_or_else(|e| panic!("render page {page}: {e:?}"));
        let h = svg_height(&svg);
        if let Some(max_y) = max_text_y(&svg) {
            assert!(
                max_y <= h,
                "page {page}: text max_y={max_y:.1} 가 페이지 높이 {h:.1} 초과 — \
                 하단 공간 부족 문단이 리플로되지 않고 페이지 밖으로 방출됨 (v0.7.6 회귀)"
            );
            // 마지막 쪽 이전 페이지는 하단 근처까지 실제로 채워져 있어야
            // 이 가드가 공허하지 않다(리플로 압력이 걸린 상태의 검증임을 보장).
            if page + 1 < pages {
                assert!(
                    max_y >= h * 0.6,
                    "page {page}: max_y={max_y:.1} 로 페이지가 하단({:.1} 이상)까지 \
                     채워지지 않아 리플로 경로가 검증되지 않음",
                    h * 0.6
                );
            }
        }
    }

    // 넘친 내용이 소실되지 않고 마지막 쪽까지 실제 배치됐는지 확인한다.
    let last_svg = doc
        .render_page_svg_native(pages - 1)
        .expect("render last page");
    assert!(
        max_text_y(&last_svg).is_some(),
        "마지막 쪽({pages})에 텍스트가 없다 — 넘친 문단이 다음 쪽으로 이월되지 않음"
    );
}
