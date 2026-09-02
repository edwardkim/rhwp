//! [#5679] 배분 정렬(Justify)의 여분이 줄-말미 공백에도 붙어 run 이 칸을
//! 벗어나던 폭 발산 가드.
//!
//! 10857(위임전결규정, 105쪽 HWP5) — 배분 몫(extra_word_spacing)은 **내부
//! 공백 수**로 나누고 est 의 effective_used 도 말미 공백을 제외하는데,
//! char_width_decision 은 모든 ' ' 에 여분을 붙였다. 그래서 말미 공백이
//! 여분까지 얹어 그려져 run bbox·x 전진이 줄 상자를 여분×말미공백수만큼
//! 넘었다 — p11 '외부 평가전문위원 ': run 144.0px vs 줄 122.1px(칸 우측
//! +20.1px), 문서 전체 텍스트 run 칸-초과 76건 중 53건이 자연 말미공백
//! 폭을 넘는 초과였다. 수정 후 그 53건이 0 — 잔여 초과는 전부 자연 말미
//! 공백 overhang(한글도 동일하게 허용하는 침범)이다. 가시 글리프는 수정
//! 전에도 줄 끝(avail)에서 정확히 끝났으므로 이 가드는 bbox/전진 좌표
//! 계약이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5679/10857_delegation_rules.hwp";

fn find_run_with_line<'a>(
    node: &'a RenderNode,
    needle: &str,
    line: Option<&'a RenderNode>,
) -> Option<(&'a RenderNode, &'a RenderNode)> {
    let line = if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
        Some(node)
    } else {
        line
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return line.map(|l| (node, l));
        }
    }
    node.children
        .iter()
        .find_map(|child| find_run_with_line(child, needle, line))
}

#[test]
fn issue_5679_justified_trailing_space_does_not_take_word_distribution() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5679 fixture");

    let p11 = core.build_page_render_tree(10).expect("render p11");
    let (run, line) = find_run_with_line(&p11.root, "외부 평가전문위원", None)
        .expect("p11 must contain the 외부 평가전문위원 run");

    // 여분이 말미 공백에 붙으면 run 폭이 줄 상자를 여분×2 만큼 넘는다
    // (수정 전 144.0 vs 줄 122.1). 자연 말미 공백 overhang 만 허용한다
    // (공백 2개 × 자간 반영 자연폭 ≈ 11.5px, 상한 14px).
    let over = run.bbox.width - line.bbox.width;
    assert!(
        over <= 14.0,
        "run w={:.1} line w={:.1} — 말미 공백이 배분 여분을 얹어 그려짐 (초과 {over:.1}px)",
        run.bbox.width,
        line.bbox.width,
    );

    // '단, ' 사례 — 앞 run 들의 배분 여분 누적 위에서 마지막 run 의 말미 공백
    // 까지 여분을 받으면 칸 우측을 23.1px 넘었다. 자연 공백 overhang(≤8px)만 허용.
    let (dan_run, _) =
        find_run_with_line(&p11.root, "단,", None).expect("p11 must contain the 단, run");
    let cell_right = 451.8;
    let dan_right = dan_run.bbox.x + dan_run.bbox.width;
    assert!(
        dan_right <= cell_right + 8.0,
        "'단, ' 우측 {dan_right:.1} — 칸 우측({cell_right}) 초과가 자연 공백 폭을 넘음",
    );
}
