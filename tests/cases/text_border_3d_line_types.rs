//! 검사 유형: 관계형(글줄·문단 테두리의 유무와 정상 실선); 페이지 절대 좌표 핀 없음.
//! 주석 갱신: 2026-11-05T13:18:44+09:00; 직전 파일 변경: 2f44d7d0ab19c743089ae309a8ca64eee57efae2 2026-11-05T02:23:29+09:00.
//! 측정일 2026-11-05, source 6337fe4d10; 96 DPI·Print·2px raw 실루엣(Native Skia/fresh WASM Canvas).
//! borders 전1쪽 Native 최저 98.04% / fresh WASM 최저 97.73%; 누락 없음. 기존 ±8px 영역을 앞뒤 글줄 사이의 소유 band로 바꾼다.
//! 증거: mydocs/pr/assets/semanticist21-20261105-next10/validation/pre8622/candidate/{native,wasm}/borders.tsv 및 circle-print-geometry.md.
//! 독립 PDF와 변경 영역을 직접 확대 판독한다.
//!
//! 입체선(14~17)으로 지정한 글자·문단 테두리는 긋지 않는다.
//!
//! rhwp로 만든 시험 문서를 한컴 한글 12.30(Mac)에서 인쇄한 PDF
//! (`pdf/border_line_types/`)에는 3D, 3D 반전, 3D 가는선, 3D 가는선 반전 네 종류의
//! 글자 테두리와 문단 테두리가 하나도 없다. 같은 문서의 실선·점선·원형 점선·물결선
//! 테두리는 모두 그려져 있다. 수정 전 rhwp는 네 종류를 모두 실선으로 그렸다.
//! 표 칸·쪽 테두리의 입체선은 대조 근거가 없어 그대로 둔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/border_line_types/borders.hwp";

fn line_text(node: &RenderNode) -> String {
    node.children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn is_stroke(node: &RenderNode) -> bool {
    matches!(
        node.node_type,
        RenderNodeType::Line(_) | RenderNodeType::Rectangle(_) | RenderNodeType::Path(_)
    )
}

fn collect<'a>(
    node: &'a RenderNode,
    lines: &mut Vec<(String, &'a RenderNode)>,
    strokes: &mut Vec<BoundingBox>,
) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        lines.push((line_text(node), node));
    }
    if is_stroke(node) {
        strokes.push(node.bbox);
    }
    for child in &node.children {
        collect(child, lines, strokes);
    }
}

fn paragraph_band(lines: &[(String, &RenderNode)], line: &RenderNode) -> (f64, f64) {
    let top = line.bbox.y;
    let bottom = top + line.bbox.height;
    let previous = lines
        .iter()
        .map(|(_, n)| n.bbox.y + n.bbox.height)
        .filter(|y| *y < top)
        .max_by(f64::total_cmp);
    let next = lines
        .iter()
        .map(|(_, n)| n.bbox.y)
        .filter(|y| *y > bottom)
        .min_by(f64::total_cmp);
    let above = previous
        .map(|y| top - y)
        .or_else(|| next.map(|y| y - bottom))
        .expect("neighbor line");
    let below = next.map(|y| y - bottom).unwrap_or(above);
    (top - above / 2.0, bottom + below / 2.0)
}

fn owns_stroke(line: &RenderNode, band: (f64, f64), stroke: &BoundingBox) -> bool {
    stroke.y <= band.1
        && stroke.y + stroke.height >= band.0
        && stroke.x <= line.bbox.x + line.bbox.width
        && stroke.x + stroke.width >= line.bbox.x
}

#[test]
fn three_d_character_and_paragraph_borders_are_not_drawn() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let (mut lines, mut strokes) = (Vec::new(), Vec::new());
    collect(&page.root, &mut lines, &mut strokes);

    for kind in 14..=17 {
        // 글자 테두리: 테두리선은 그 글줄의 자식으로 들어간다.
        let label = format!("{kind} 3D");
        let (_, line) = lines
            .iter()
            .find(|(text, _)| text.starts_with(&label))
            .unwrap_or_else(|| panic!("{label} 글줄"));
        assert!(
            !line.children.iter().any(is_stroke),
            "{label} 글자 테두리를 그렸다"
        );

        // 문단 테두리: 글줄 둘레 1mm 간격에 놓일 선이 없어야 한다.
        let label = format!("paragraph border type {kind}");
        let (_, line) = lines
            .iter()
            .find(|(text, _)| text.contains(&label))
            .unwrap_or_else(|| panic!("{label} 글줄"));
        let band = paragraph_band(&lines, line);
        assert!(
            !strokes.iter().any(|s| owns_stroke(line, band, s)),
            "{label} 문단 테두리를 그렸다"
        );
    }

    // 대조: 같은 문서의 실선 문단 테두리는 그대로 그린다.
    let (_, solid) = lines
        .iter()
        .find(|(text, _)| text.contains("paragraph border type 1"))
        .expect("실선 문단");
    let band = paragraph_band(&lines, solid);
    assert!(
        strokes.iter().any(|s| owns_stroke(solid, band, s)),
        "실선 문단 테두리"
    );
}
