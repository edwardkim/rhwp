//! 펼친 1×1 부모의 표시 영역과 자식 표의 독립 앵커를 실제 인쇄 SVG에서 검사한다.
//!
//! 원본·한 속성만 바꾼 수동 대조군·정상 한컴 HWP 저장 대조군은 같은 입력의 한컴2020
//! Print와 Native/fresh 전쪽 100% 및 확대 판독 후 등록했다. 입력 생성 방식과 PDF 해시는
//! mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/README.md에 있다.
//! 흘림/글자 취급 자식은 부모 괘선 안에서 잘리고, 흐름 제한이 없는 자식은 자기 상자를
//! 유지한다. 클립 ID나 특정 화면 좌표 대신 실제 괘선·클립의 포함 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

mod output {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/wrapper_note_render_output.rs"
    ));
}

use roxmltree::{Document, Node};

const ORIGINAL: &str = "mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/clickhere-wrapper-current-v175.hwpx";
const COUNTERS: &str = "mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289";

#[derive(Clone, Copy, Debug)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

fn number(node: Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name)
        .expect("실제 도형 좌표")
        .parse()
        .expect("유한 수치 좌표")
}

// SVG의 십진 문자열을 읽은 뒤 생기는 부동소수점 연산 오차만 비교한다.
// 실물 위치 공차나 절대 px 영역이 아니다.
fn equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= 64.0 * f64::EPSILON * left.abs().max(right.abs()).max(1.0)
}

fn clip_rects(text: Node<'_, '_>) -> Vec<Rect> {
    text.ancestors()
        .filter_map(|node| node.attribute("clip-path"))
        .map(|reference| {
            let id = reference
                .strip_prefix("url(#")
                .and_then(|id| id.strip_suffix(')'))
                .expect("실제 클립 참조");
            let definition = text
                .document()
                .descendants()
                .find(|node| node.attribute("id") == Some(id))
                .expect("참조한 클립 정의");
            let rect = definition
                .children()
                .find(|node| node.has_tag_name("rect"))
                .expect("셀/본문의 직사각 표시 영역");
            let left = number(rect, "x");
            let top = number(rect, "y");
            Rect {
                left,
                top,
                right: left + number(rect, "width"),
                bottom: top + number(rect, "height"),
            }
        })
        .collect()
}

/// 자식의 왼쪽보다 앞에서 시작하는 같은 폭의 부모 괘선을 찾는다.
/// 두 대상 셀은 각각 자기 글줄을 포함하는 가장 가까운 부모 괘선을 소유한다.
fn enclosing_border(document: &Document<'_>, text: Node<'_, '_>, child: Rect) -> Rect {
    let lines: Vec<_> = document
        .descendants()
        .filter(|node| node.has_tag_name("line"))
        .map(|node| {
            (
                number(node, "x1"),
                number(node, "y1"),
                number(node, "x2"),
                number(node, "y2"),
            )
        })
        .collect();
    let baseline = number(text, "y");
    lines
        .iter()
        .filter(|&&(left, top, right, y)| {
            equal(top, y)
                && left < child.left
                && equal(right - left, child.right - child.left)
                && top < baseline
        })
        .filter_map(|&(left, top, right, _)| {
            lines.iter().find_map(|&(x1, y1, x2, bottom)| {
                (equal(x1, left)
                    && equal(x2, left)
                    && equal(y1, top)
                    && bottom > baseline
                    && lines.iter().any(|&(bx1, by1, bx2, by2)| {
                        equal(bx1, left)
                            && equal(bx2, right)
                            && equal(by1, bottom)
                            && equal(by2, bottom)
                    }))
                .then_some(Rect {
                    left,
                    top,
                    right,
                    bottom,
                })
            })
        })
        .max_by(|a, b| a.top.total_cmp(&b.top))
        .expect("같은 자식 글줄을 소유하는 부모 괘선")
}

fn child_texts<'a, 'input>(document: &'a Document<'input>) -> Vec<Node<'a, 'input>> {
    let nodes: Vec<_> = document
        .descendants()
        .filter(|node| node.has_tag_name("text") && node.text() == Some("뒤"))
        .collect();
    assert_eq!(nodes.len(), 2, "두 래퍼의 뒤 글자는 각각 한 번씩 보존된다");
    nodes
}

fn assert_flow_clip(input: &str) {
    let pages = output::pages(input);
    assert_eq!(pages.len(), 1, "독립 Print의 전체 쪽수");
    let document = Document::parse(&pages[0]).expect("최종 인쇄 SVG");
    let mut previous_bottom = None;
    for text in child_texts(&document) {
        let clips = clip_rects(text);
        let child = *clips.first().expect("자식 셀 표시 영역");
        let parent = enclosing_border(&document, text, child);
        assert!(child.right > parent.right, "자식 저장 폭은 축소되지 않는다");
        let effective_right = clips
            .iter()
            .map(|clip| clip.right)
            .fold(f64::INFINITY, f64::min);
        let effective_bottom = clips
            .iter()
            .map(|clip| clip.bottom)
            .fold(f64::INFINITY, f64::min);
        assert!(
            effective_right <= parent.right || equal(effective_right, parent.right),
            "자식 괘선/글자는 부모 오른쪽 밖에 그리지 않는다: {child:?}, {parent:?}, {clips:?}"
        );
        assert!(effective_bottom <= parent.bottom || equal(effective_bottom, parent.bottom));
        assert!(number(text, "x") >= parent.left && number(text, "y") <= parent.bottom);
        if let Some(bottom) = previous_bottom {
            assert!(parent.top > bottom, "앞뒤 래퍼의 물리 상자는 겹치지 않는다");
        }
        previous_bottom = Some(parent.bottom);
    }
}

#[test]
fn inline_child_keeps_its_width_and_clips_at_the_parent_border() {
    assert_flow_clip(&format!("{COUNTERS}/child-inline-fda0d3d71ef2.hwpx"));
}

#[test]
fn flowing_child_keeps_its_width_and_clips_at_the_parent_border() {
    assert_flow_clip(&format!("{COUNTERS}/child-flow-faeb7331e8fb.hwpx"));
}

#[test]
fn unrestricted_children_keep_their_own_viewport_in_all_independent_controls() {
    let controls = [
        ORIGINAL.to_owned(),
        format!("{COUNTERS}/child-offset-8b1ec78415f1.hwpx"),
        format!("{COUNTERS}/parent-padding-61e9cbd25806.hwpx"),
        format!("{COUNTERS}/parent-height-96f2cd69236d.hwpx"),
        format!("{COUNTERS}/normal-save-hwp-bebf8d7399c2.hwp"),
    ];
    for input in controls {
        let pages = output::pages(&input);
        assert_eq!(pages.len(), 1, "{input}: 독립 Print의 전체 쪽수");
        let document = Document::parse(&pages[0]).expect("최종 인쇄 SVG");
        for text in child_texts(&document) {
            let clips = clip_rects(text);
            let own = *clips.first().expect("자식 셀 표시 영역");
            let right = clips
                .iter()
                .map(|clip| clip.right)
                .fold(f64::INFINITY, f64::min);
            let bottom = clips
                .iter()
                .map(|clip| clip.bottom)
                .fold(f64::INFINITY, f64::min);
            assert!(
                equal(right, own.right) && equal(bottom, own.bottom),
                "{input}: 흐름 제한 없는 자식에 부모의 작은 표시 영역을 덧씌우지 않는다"
            );
            assert!(number(text, "x") >= own.left && number(text, "y") <= own.bottom);
        }
        assert!(!output::text(&document).is_empty());
    }
}
