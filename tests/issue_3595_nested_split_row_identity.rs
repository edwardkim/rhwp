//! [#3595] 텍스트와 2행 이상 중첩 표가 같은 문단에 있으면 그 중첩 표는 분할 가능해야 한다.
//!
//! `cell_units` 의 중첩 표 분해 경로가 세 갈래인데 그 사이에 구멍이 있다.
//!
//! | 문단 형태 | 분해 |
//! | --- | --- |
//! | 가시 텍스트 없음 + 중첩표 1개(2행+) | per-중첩행 유닛 |
//! | 가시 텍스트 없음 + 중첩표 1×1 | `nested_table_mixed_fragment_heights` fragment |
//! | **텍스트 + 중첩표(2행+)** | **없음 → 분할 불가 atom** |
//!
//! `nested_table_mixed_fragment_heights` 는 첫 줄에서 `row_count != 1` 이면 빈 벡터를
//! 돌려주고, per-중첩행 경로는 "가시 텍스트 없는" 문단만 받는다. 둘 다 못 타는 문단의
//! 중첩 표는 하나의 atom 이 되어, 남은 페이지 조각에 들어가지 못하면 **통째로 렌더에서
//! 탈락한다**(표 괘선만 남고 안이 빈다).
//!
//! 대상 — `samples/task2097/75544_pii_bunseok.hwpx`
//!
//! ```text
//! 호스트 셀 C[0](r0,c0)  문단 2개
//!   p0  vpos=0     ""
//!   p1  vpos=1920  "[별지 제15호의4 서식] 자동화평가"  [중첩표 2행x1열 h=51122HU]
//! ```
//!
//! 이 문서는 저장 vpos 사다리가 정상이다(p1 vpos=1920 > 0). 즉 `vertical_pos == 0`
//! 센티널 계열(a6b0aed3 / d8caba0a / 5385690d)과 무관하게 분해 경로 구멍만 격리한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/task2097/75544_pii_bunseok.hwpx";

/// 문제의 중첩 표 둘째 행(`C[1](r1,c0)`) 안에만 있는 문자열들.
/// 공백을 제거한 형태로 대조한다(렌더 런 분할과 무관하게).
const MARKERS: &[&str] = &[
    "※작성요령",
    "2.자동화평가결과의산출에유리하다고판단되는정보",
];

/// 같은 행의 **마지막** 문단. [#3658] 종료 조각(end_cut=[])의 셀 하단 드롭이
/// 이 꼬리 문단을 유실시키던 결함의 회귀 마커.
const TAIL_MARKER: &str = "○○금융회사대표이사△△△";

fn rendered_text_without_spaces() -> String {
    let bytes = std::fs::read(SAMPLE).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"));

    fn collect(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.extend(run.display_or_text().chars().filter(|c| !c.is_whitespace()));
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut text = String::new();
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("build page {page}: {e:?}"));
        collect(&tree.root, &mut text);
    }
    text
}

#[test]
fn nested_table_sharing_a_paragraph_with_text_is_not_dropped() {
    let rendered = rendered_text_without_spaces();

    let missing: Vec<&str> = MARKERS
        .iter()
        .copied()
        .filter(|m| !rendered.contains(m))
        .collect();

    assert!(
        missing.is_empty(),
        "연속 페이지가 중첩 표의 뒤 행을 이어받지 못했다 — 조각의 행 범위가 \
         `0..1` 로 고정되면 앞 행만 반복 렌더되고 뒤 행이 어느 페이지에도 나오지 \
         않는다. 누락 {}건: {:?}",
        missing.len(),
        missing
    );
}

/// 같은 표의 **마지막** 문단까지 렌더되어야 한다.
///
/// [#3658] 종전 결함: 컷이 `end_cut=[]` 로 완료를 선언한 종료 조각에서, 렌더가
/// start_row 를 처음부터 재적층해 행 그리드(747px)보다 커지고(766px+), 초과한
/// 꼬리 줄이 "셀 하단 초과 줄 드롭"(다음 쪽 소속 줄 제외용)에 걸려 어느 쪽에도
/// 렌더되지 않았다 — 이어받을 continuation 이 없어 영구 유실.
///
/// 수정: 종료 조각(`NestedTableSplit.terminal`)은 (1) 이전 쪽에 이미 보인
/// start_row 상단 밴드만큼 `offset_within_start` 를 부여해 잔여 콘텐츠가 행
/// 그리드 안에 들어가게 하고, (2) 셀 하단 초과 줄 드롭을 적용하지 않는다.
#[test]
fn nested_table_tail_paragraph_is_rendered() {
    let rendered = rendered_text_without_spaces();
    assert!(
        rendered.contains(TAIL_MARKER),
        "표의 마지막 문단이 렌더되지 않았다: {TAIL_MARKER:?}"
    );
}
