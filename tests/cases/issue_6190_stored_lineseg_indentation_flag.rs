//! [Issue #6190] 저장 LINE_SEG 가 "이 줄엔 들여쓰기 없음"이라고 적어 둔 줄에 문단
//! `indent` 를 얹어, `경 력 사 항` 이하가 통째로 오른쪽으로 밀리고 마지막 표가 용지
//! 밖으로 나간다.
//!
//! **정답지는 `LineSeg.tag` 의 `TAG_INDENTATION`(bit 20)이다** — 한글이 줄마다 남기는
//! "이 줄에 들여쓰기가 적용됐다"는 기록이다. 한글 통제 실험(같은 문단의 `indent` 만
//! 바꿔 한글로 PDF 를 떠서 실측):
//!
//! | 문서 | `ls[0].tag` | `indent` 스윕 | 한글 x |
//! |---|---|---|---|
//! | 156458354 pi=28 | `0x60000` (bit20 꺼짐) | 0 · 2000 · 6000 · 10000 · 20445 | **전부 345.60 (불변)** |
//! | 36313646 pi=2 | `0x160000` (bit20 켜짐) | 0 · 660 · 4000 · 10000 · 20445 | 352.77 → 420.89 (**선형**) |
//!
//! 즉 한글은 들여쓰기를 일반적으로 **적용하며**(둘째 행), 이 문단만 안 하는 이유가
//! 저장 비트에 적혀 있다. 내어쓰기 문단이 `ls[0]=0x60000, ls[1..]=0x160000` 인 것도
//! 같은 의미다 — 내어쓰기는 둘째 줄부터 적용된다.
//!
//! | | rhwp(수정 전) | rhwp(수정 후) | 한글 (1차 오라클) |
//! |---|---|---|---|
//! | `학 력 사 항` x | 345.4 | 345.4 | 345.60 |
//! | `경 력 사 항` x | **413.5** | **345.4** | 345.60 |
//! | `발표 논문` x | **423.0** | **354.9** | 354.88 |
//! | 마지막 표 우변 | **829.8** (용지 793.7 밖) | **695.4** | 본문 안 |
//!
//! 편차 68.1px 는 `indent/2`(136.3/2)와 같다 — 가운데 정렬에서 상자 좌단이 밀리면
//! 중심은 그 절반만 움직이기 때문이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6190/center_align_first_line_indent.hwp";
/// 본문 우단(px).
const BODY_RIGHT_PX: f64 = 699.2;
/// 본문 폭(px).
const BODY_WIDTH_PX: f64 = 604.7;

#[test]
fn issue_6190_stored_lineseg_without_indentation_flag_keeps_box() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let control = run_x(&page.root, "학 력 사 항").expect("indent=0 통제 문단");
    let subject = run_x(&page.root, "경 력 사 항").expect("indent=20445 문단");

    assert!(
        (subject - control).abs() <= 1.0,
        "저장 LINE_SEG 의 TAG_INDENTATION(bit 20)이 꺼진 줄에는 들여쓰기를 얹지 않는다 — \
         `학 력 사 항` x={control:.1}, `경 력 사 항` x={subject:.1} \
         (한글 1차 오라클은 둘 다 345.60). 반영하면 indent/2 = 68.1px 밀린다."
    );

    // 본문 폭에 **들어가는** 표만 본다 — 본문보다 넓은 표가 넘치는 것은 별개 축이다.
    let mut tables = Vec::new();
    collect_tables(&page.root, &mut tables);
    let escaping: Vec<_> = tables
        .iter()
        .filter(|(x, w)| *w <= BODY_WIDTH_PX + 1.0 && x + w > BODY_RIGHT_PX + 1.0)
        .map(|(x, w)| format!("x={x:.1} w={w:.1} 우변={:.1}", x + w))
        .collect();
    assert!(
        escaping.is_empty(),
        "들여쓰기가 반영되면 이 문단이 호스트하는 표까지 밀려 용지 밖으로 나간다 — \
         본문(우단 {BODY_RIGHT_PX})에 들어갈 폭인데 넘는 표: {escaping:?}"
    );
}

/// 주어진 텍스트를 가진 TextRun 의 x.
fn run_x(node: &RenderNode, want: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.trim() == want {
            return Some(node.bbox.x);
        }
    }
    node.children.iter().find_map(|c| run_x(c, want))
}

/// 쪽 안 모든 표의 `(x, width)`.
fn collect_tables(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push((node.bbox.x, node.bbox.width));
    }
    for child in &node.children {
        collect_tables(child, out);
    }
}
