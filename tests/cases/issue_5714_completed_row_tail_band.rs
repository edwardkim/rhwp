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

/// 정본 1쪽의 검토 결과와 2쪽의 첫/마지막 행 소유를 보존한다.
/// 실제 전쪽 Native/fresh 수용은 보정215의 참조 폰트 환경과 PDF로 확인했다.
/// 관측 px 높이 대신 완전한 라벨·본문의 셀 내부 포함과 앞뒤 행 순서를 검사한다.
#[test]
fn issue_5714_task2097_p2_top_row_full_height_bottom_row_squeezed() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task2097/1741000_project_application.hwp");
    let bytes = fs::read(&path).expect("사업신청서 원본 읽기");
    let mut core = DocumentCore::from_bytes(&bytes).expect("사업신청서 원본 파싱");
    let environment = rhwp::renderer::font_environment::FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/project2097-reference-font-environment.json"
    ))
    .expect("독립 PDF의 참조 폰트 환경");
    core.set_font_environment(Some(environment))
        .expect("측정과 paint에 같은 참조 환경 적용");
    assert_eq!(core.page_count(), 2, "정본 전체 쪽수와 마지막 행 소유");

    let p1 = core.build_page_render_tree(0).expect("첫 쪽 배치");
    let p2 = core.build_page_render_tree(1).expect("둘째 쪽 배치");
    let first_text = all_text(&p1.root);
    let second_text = all_text(&p2.root);
    assert!(first_text.contains("중복검토 방법") && first_text.contains("유사ㆍ중복 여부"));
    assert!(!first_text.contains("국민 의견"));
    assert!(!second_text.contains("중복검토 방법"));

    let top =
        find_cell_containing(&p2.root, "국민 의견").expect("둘째 쪽 첫 행의 국민 의견수렴 라벨");
    let middle =
        find_cell_containing(&p2.root, "연구내용").expect("같은 쪽 중간 행의 연구내용 라벨");
    let bottom =
        find_cell_containing(&p2.root, "활용방안").expect("같은 쪽 마지막 행의 활용방안 라벨");
    let epsilon = (top.bbox.width + middle.bbox.height + bottom.bbox.height) * f64::EPSILON * 128.0;
    assert!(top.bbox.y + top.bbox.height <= middle.bbox.y + epsilon);
    assert!(middle.bbox.y + middle.bbox.height <= bottom.bbox.y + epsilon);

    fn collect_lines<'a>(node: &'a RenderNode, lines: &mut Vec<&'a RenderNode>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            lines.push(node);
        }
        for child in &node.children {
            collect_lines(child, lines);
        }
    }
    for (cell, labels) in [
        (top, ["국민 의견", "수렴 여부"]),
        (bottom, ["연구결과", "활용방안"]),
    ] {
        let mut lines = Vec::new();
        collect_lines(cell, &mut lines);
        assert_eq!(lines.len(), labels.len(), "라벨 글줄 누락·중복 금지");
        for (line, label) in lines.into_iter().zip(labels) {
            assert!(all_text(line).contains(label), "원본 라벨 순서: {label}");
            assert!(line.bbox.y + epsilon >= cell.bbox.y);
            assert!(line.bbox.y + line.bbox.height <= cell.bbox.y + cell.bbox.height + epsilon);
        }
    }
    let use_plan =
        find_cell_containing(&p2.root, "現 일원화 모델").expect("둘째 쪽의 첫 활용방안 문단");
    let use_text = all_text(use_plan);
    let first = use_text.find("일원화 모델").expect("첫 활용방안");
    let last = use_text.find("중장기 발전방안").expect("마지막 활용방안");
    assert!(first < last, "마지막 두 문단의 원본 순서");
    assert!(max_text_line_bottom(use_plan) <= use_plan.bbox.y + use_plan.bbox.height + epsilon);
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
