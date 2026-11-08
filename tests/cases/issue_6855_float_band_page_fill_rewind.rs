//! 관계형·고정형: #6855 되감김의 원문 밴드·제목·후속 표 순서와 #7727 독립 Print ±2px.
//! 주석 갱신: 2026-11-06T08:16:20+09:00; 직전 파일 커밋: 2026-10-04T19:03:36+09:00 (731de9e1b4).
//! 2026-11-06 측정: canonical 96DPI·compat2022·PDFium·2px raw, 원본 전204쪽.
//! 전쪽 최저 Native 92.05232% / fresh WASM 92.16419%(11쪽), 90% 미만·쪽 누락 0.
//! px 대상 185쪽 N/W 99.92888/99.93373%, 186쪽 99.93606/99.93833%.
//! 고정 base 170b886f / code head fc378076: 각 backend 전204쪽 base/후보 PNG 동일, raw 하락0.
//! 근거: mydocs/pr/assets/issue7727/verification-20261106/README.md.
//! PDF: pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf, SHA-256 0db1eb55eda0f1a4985eeaf187656c23499616b7dccddcbb7c874c0937e1b2e0.
//! 원 해결 26812ee08d·cabcd9a07c는 base 조상이다. 당시 용지 밖으로 빠진 제목의 검출 의미를 유지한다.
//! 기존 181 인덱스는 현재 landscape 247×7 표를 가리킨다. 소속 사전조건을 넣은 실제 Native 검사 1FAIL.
//! 원문 구역2·문단1의 29×3 표를 찾아 실제 용지 높이와 SVG paint를 검사하도록 교정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp";

fn open_sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("CBTA 원문 읽기"))
        .expect("CBTA 원문 파싱")
}

fn table(node: &RenderNode, paragraph: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t)
        if t.section_index == Some(2) && t.para_index == Some(paragraph)
            && t.control_index == Some(0) && t.cell_context.is_none())
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table(child, paragraph))
}

fn title(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.section_index == Some(2) && line.para_index == Some(3))
    {
        return Some(node);
    }
    // 표 셀 안의 같은 이름·번호 문단은 본문 제목이 아니다.
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    node.children.iter().find_map(title)
}

/// SVG paint가 실제 출력한 `<text>`의 baseline을 읽는다.
fn text_baselines(svg: &str) -> Vec<f64> {
    let mut out = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let Some(ys) = head.find("y=\"") else {
            continue;
        };
        let s = ys + 3;
        if let Some(e) = head[s..].find('"') {
            if let Ok(y) = head[s..s + e].parse::<f64>() {
                out.push(y);
            }
        }
    }
    out
}

#[test]
fn issue_6855_band_page_paints_nothing_below_the_paper() {
    let core = open_sample();
    let mut band_pages = Vec::new();
    let mut title_pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page as u32).expect("전쪽 트리");
        if table(&tree.root, 1).is_some() {
            band_pages.push(page + 1);
            let svg = core
                .render_page_svg_native(page as u32)
                .expect("밴드 쪽 SVG paint");
            let baselines = text_baselines(&svg);
            assert!(!baselines.is_empty(), "밴드 쪽의 실제 글자를 검사한다");
            let bottom = tree.root.bbox.y + tree.root.bbox.height;
            assert!(
                baselines
                    .iter()
                    .all(|y| *y >= tree.root.bbox.y && *y <= bottom),
                "{page}번 쪽의 실제 글자 baseline은 해당 용지 안이어야 한다"
            );
        }
        if let Some(heading) = title(&tree.root) {
            title_pages.push(page + 1);
            let painted: String = heading
                .children
                .iter()
                .filter_map(|n| match &n.node_type {
                    RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(
                painted,
                core.document().sections[2].paragraphs[3].text,
                "되감김 다음 제목의 글자·번호를 빠뜨리지 않는다"
            );
            let previous = table(&tree.root, 1).expect("앞 표의 실제 마지막 조각");
            let following = table(&tree.root, 4).expect("후속 46×3 표의 첫 조각");
            assert!(previous.bbox.y + previous.bbox.height <= heading.bbox.y);
            assert!(heading.bbox.y + heading.bbox.height <= following.bbox.y);
            assert!(
                following.bbox.y + following.bbox.height
                    <= tree.root.bbox.y + tree.root.bbox.height
            );
        }
    }
    assert_eq!(band_pages, [185, 186], "독립 Print의 29×3 밴드 소유");
    assert_eq!(
        title_pages,
        [186],
        "제목의 누락·중복 또는 밴드 아래 용지 밖 배치 금지"
    );
}

fn pdf_px(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 한컴2020 Print ±2px: {label}, actual={actual}, expected={expected}"
    );
}

#[test]
fn issue_7727_rewind_band_and_heading_keep_independent_print_edges() {
    let core = open_sample();
    // 독립 PDF 벡터 괘선 중심과 PDF 단어 bbox다. 현재 rhwp 출력을 기대값으로 쓰지 않는다.
    for (page, top, bottom) in [
        (184, 259.55598958, 1035.02800496),
        (185, 79.27335612, 165.89868164),
    ] {
        let tree = core.build_page_render_tree(page).unwrap();
        let band = table(&tree.root, 1).expect("원문 구역2 문단1의 29×3 표");
        pdf_px(band.bbox.y, top, "자리차지 밴드 상단");
        pdf_px(band.bbox.y + band.bbox.height, bottom, "자리차지 밴드 하단");
        if page == 185 {
            pdf_px(
                title(&tree.root).expect("밴드 다음 본문 제목").bbox.y,
                257.632,
                "과목2 제목 상단",
            );
        }
    }
}
