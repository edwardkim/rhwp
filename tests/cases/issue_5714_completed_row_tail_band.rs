//! [#5714] 완결된 행의 빈 tail 밴드가 다음 조각의 새 행에 씌워지던 겹침 가드.
//!
//! 1490000-200800034(베트남 노동시장 연구보고서, 124쪽 HWP5) — 19×2 RowBreak
//! 표(rowspan 없음)가 p12→p13 으로 나뉠 때, p12 끝행(제9장)이 쪽 경계에서
//! 31.0px 로 압축되면 cut 높이(41.9px)와의 차 11.0px 가
//! `next_start_row_height_override` 로 다음 조각에 넘어가 **새 행**(제10장,
//! 3줄 59.3px)의 높이로 씌워졌다 — 셀 11.0px 에 3줄이 들어가 아래 행(제11장)
//! 위로 38.8px 포개짐. 한컴 2024 PDF 실측: p13 은 밴드 없이 제10장을 표 상단
//! 에서 전체 높이로 시작하고(62.4→93.9pt = 59.3px 정확 일치), p12 끝행 압축
//! (31px)은 rhwp 와 동일하다. tail 밴드 이월은 물리적으로 이어지는 것이 있을
//! 때만 옳다 — intra-row 컷(같은 행 재개) 또는 경계를 가로지르는 rowspan 셀
//! (76076 p36 의 24.1px 밴드, r8 rs=7). 이 표는 둘 다 없다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5714/1490000-200800034_vietnam_labor_report.hwp";

fn all_text(node: &RenderNode) -> String {
    let own = if let RenderNodeType::TextRun(run) = &node.node_type {
        run.text.as_str()
    } else {
        ""
    };
    let mut text = String::from(own);
    for child in &node.children {
        text.push_str(&all_text(child));
    }
    text
}

fn find_cell_containing<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TableCell(_)) && all_text(node).contains(needle) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_cell_containing(child, needle))
}

fn max_text_line_bottom(node: &RenderNode) -> f64 {
    let own = if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
        node.bbox.y + node.bbox.height
    } else {
        f64::MIN
    };
    node.children
        .iter()
        .map(max_text_line_bottom)
        .fold(own, f64::max)
}

/// [#2097→#5714] 1741000 p2 의 기하 핀 — 쪽수(2)만으로는 "앞 조각의 유령 tail
/// 밴드가 p2 첫 행을 압축"하는 우연 정답과 "말미 행 압축 수용"의 진짜 정답을
/// 못 가른다. 한글 2024 PDF 실측: p2 첫 행(국민 의견수렴)은 전체 높이
/// 56.4px(42.2pt), 말미 행(연구결과 활용방안)은 하단에서 압축(선언 80.3 →
/// 밴드 ~70pt급). 둘 다 핀한다.
#[test]
fn issue_5714_task2097_p2_top_row_full_height_bottom_row_squeezed() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task2097/1741000_project_application.hwp");
    let bytes = fs::read(&path).expect("read task2097 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse task2097 fixture");
    assert_eq!(core.page_count(), 2, "한글 2024 COM 재실측 쪽수");

    let p2 = core.build_page_render_tree(1).expect("render p2");
    let top = find_cell_containing(&p2.root, "국민 의견").expect("p2 must start at 국민 의견수렴 row");
    assert!(
        (52.0..=60.0).contains(&top.bbox.height),
        "p2 첫 행은 전체 높이(한글 56.3px)여야 한다 — h={:.1} (유령 tail 밴드 압축이면 ~10px)",
        top.bbox.height,
    );
    let bottom = find_cell_containing(&p2.root, "활용방안").expect("p2 must end at 활용방안 row");
    assert!(
        bottom.bbox.height < 80.0,
        "p2 말미 행은 압축 수용(선언 80.3 미만 밴드)이어야 한다 — h={:.1}",
        bottom.bbox.height,
    );
}

#[test]
fn issue_5714_new_row_after_compressed_row_keeps_full_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5714 fixture");

    assert_eq!(core.page_count(), 124, "흐름 전체 쪽수 고정");

    let p13 = core.build_page_render_tree(12).expect("render p13");
    let cell = find_cell_containing(&p13.root, "제10장")
        .expect("p13 must own the 제10장 row (fragment first row)");

    // 유령 tail 밴드(11.0px)가 씌워지면 셀 높이가 3줄 내용(59.3px)에 못 미치고
    // 아래 행(제11장) 위로 글자가 포개진다.
    assert!(
        cell.bbox.height >= 55.0,
        "p13 제10장 행 셀 h={:.1}px — 압축 행의 빈 tail 이 새 행에 씌워짐 (기대 59.3)",
        cell.bbox.height,
    );
    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let content_bottom = max_text_line_bottom(cell);
    assert!(
        content_bottom <= cell_bottom + 1.0,
        "p13 제10장 셀 내용({content_bottom:.1})이 셀 하단({cell_bottom:.1})을 넘어 아래 행과 겹침",
    );
}
