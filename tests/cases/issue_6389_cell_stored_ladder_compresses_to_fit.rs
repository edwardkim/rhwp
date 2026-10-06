//! [Issue #6389] 편람 p68 본문 표 셀의 `○` 문단 줄들이 셀 우측 테두리를 +72~85px 넘는다.
//!
//! 이 셀 문단들의 저장 사다리(2~4줄, 각 `segment_width=37560HU`=셀 안쪽 폭)는
//! kopub/no-ttf 오라클 PDF 와 문자 단위로 일치한다 — 한글이 이 내용을 이 폭에
//! 담았다는 증언이다. 그런데 내장 KoPub돋움체 한글 진행폭(1.0em)이 오라클
//! 실측(≈0.83em)보다 넓어 자연 폭이 줄 상자의 1.12~1.17배가 되고, 억제 임계
//! (1.15)를 넘은 줄은 압축이 꺼져 자연 폭 그대로 칸 밖에 그려졌다.
//!
//! #6196 이 한 줄 사다리에 연 증언 예외를 다줄 사다리로 일반화한다: 조합이
//! 저장 줄 수를 그대로 따랐고 모든 저장 줄폭이 셀 열폭 이내면 억제하지 않고
//! 압축해 칸 안에 맞춘다. 열폭보다 넓게 기록된 사다리([sw>w] 낡은 캐시)는
//! 증언이 성립하지 않아 종전대로 클리핑된다.
//!
//! 현행 face 폭은 #6484에서 임베드 CIDFont 실측 0.872em으로 수정되었다.
//! 위 ≈0.83em은 장평·자간 적용 후 유효 폭이다. p68 KoPub 설치 환경 PDF의
//! 줄 경계도 함께 검증해, 셀 안에 들어오더라도 재조판으로 +1줄이 생기거나
//! 글자가 누락되는 경우를 잡는다. 명시적 no-ttf 환경에서도 PDF p69의 같은
//! 저장 줄 경계를 유지해야 한다. 저장 정보가 없는 재조판은 companion 환경 테스트가 검증한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
/// 대상 셀의 식별 텍스트 — p68(0기준 67) 본문 표 r1c1 첫 문단.
const CELL_MARKER: &str = "모든 기록물";
/// 줄 끝 공백 한 칸은 한글도 칸 밖으로 걸치므로 허용한다(반각 ≈ 6.7px + 여유).
const TRAILING_SPACE_ALLOWANCE_PX: f64 = 8.0;

#[test]
fn issue_6389_manual_p68_stored_ladder_cell_stays_inside_cell() {
    check_manual_cell(None);
}

#[test]
fn issue_6389_manual_no_ttf_preserves_pdf_line_boundaries() {
    // 독립 기준: 기존 -2010-no-ttf.pdf p69의 Haansoft Batang 및 대상 셀 16줄.
    // 원본의 저장 줄 경계는 KoPub PDF p68과 같다. 대체 세션은 IR을 바꾸지 않는다.
    let environment = FontEnvironment::from_json(
        r#"{"id":"hancom-2010-no-kopub","substitutions":{
            "KoPub돋움체 Light":"바탕", "KoPub돋움체 Medium":"바탕", "KoPub돋움체 Bold":"바탕",
            "KoPub바탕체 Light":"바탕", "KoPub바탕체 Medium":"바탕", "KoPub바탕체 Bold":"바탕"
        }}"#,
    )
    .expect("no-ttf environment");
    check_manual_cell(Some(environment));
}

fn check_manual_cell(environment: Option<FontEnvironment>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    core.set_font_environment(environment)
        .expect("set environment");
    let page = core.build_page_render_tree(67).expect("p68 render tree");

    let mut overflow = Vec::new();
    let mut checked = 0usize;
    walk(&page.root, None, false, &mut checked, &mut overflow);

    assert!(
        checked > 0,
        "대상 셀({CELL_MARKER})의 텍스트를 찾지 못했다 — 쪽 배분이 바뀌었는지 확인하라"
    );
    assert!(
        overflow.is_empty(),
        "저장 사다리 셀의 줄이 칸 밖으로 나간다 — {}건 (칸 우변 초과 px, 텍스트): {:?}",
        overflow.len(),
        overflow
    );

    // 독립 기대값: pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf p68.
    // pdftotext -bbox-layout 추출과 Visual Sweep 직접 비교로 확인한 대상 셀 16줄.
    // PDF 단어 추출이 삽입하는 공백만 제외하고 글자·구두점·줄 경계를 모두 보존한다.
    // 특히 세 번째(※) 문단은 종전 메트릭에서 4줄이 되었던 재조판 사례다.
    let expected = [
        "○ 모든 기록물은 전자적 관리를 원칙으로 하며, 전자적 형태로 생산되지 아니한 기록물도 전자적",
        "으로 관리되도록 노력하여야 함(공공기록물법 제6조)",
        "○ (첨부문서 분리 등록 방법) 기록물의 본문과 첨부물의 규격의 차이가 심하거나 서로 다른 기록",
        "매체로 구성된 기록물은 전자기록생산시스템의‘분리등록’기능을 이용하여 첨부물(붙임물) 등록·",
        "관리",
        "※ 분리등록한 첨부물의 경우 「공공기록물법 시행규칙」[별표 1]의 서식에 따라 등록번호를 표기",
        "하고, ‘등록번호’에는 본문의 생산(접수)등록번호에 ‘-분리연번’을 추가하여 기재(‘분리연번’은",
        "문서등록대장 해당 기록물 건의 ‘분리등록현황’에서 확인 가능)",
        "<예시> 해당 기록물 건의 등록번호 : 행정지원과-925",
        "분리등록 첨부물의 등록번호 : 행정지원과-925-1",
        "○ (붙임 파일의 압축 조건부 허용) 윈도우에 내장된 공개 압축 SW 이용 가능(반디집 7-zip 등),",
        "단 특정 압축 포맷(.egg, .alz)은 타기관에서 열람시 제약이 발생할 수 있어 사용금지",
        "○ (비전자기록물 관리) 전자문서에 분리등록한 비전자 첨부물(붙임물)이 있는 경우 「공공기록물법",
        "시행규칙」 [별표 1]의 서식에 따라 등록번호를 표기하고, 관련 문서의 문서관리카드 문서",
        "정보 또는 문서를 출력(업무관리시스템), 기록물등록대장의 해당 문서(전자문서시스템)를",
        "출력하여 분리등록한 첨부물과 함께 보관 [출력한 문서는 원본대조필을 날인]",
    ];
    let target = find_target_cell(&page.root).expect("대상 셀");
    let mut lines = Vec::new();
    collect_lines(target, &mut lines);
    assert_eq!(
        lines.iter().map(|s| without_space(s)).collect::<Vec<_>>(),
        expected
            .iter()
            .map(|s| without_space(s))
            .collect::<Vec<_>>(),
        "셀 내부 표시뿐 아니라 한컴 PDF의 줄 수·줄 경계·전체 텍스트를 유지해야 한다"
    );
}

fn find_target_cell(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TableCell(_))
        && subtree_contains(node, CELL_MARKER)
    {
        return Some(node);
    }
    node.children.iter().find_map(find_target_cell)
}

fn collect_lines(node: &RenderNode, lines: &mut Vec<String>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        collect_text(node, &mut text);
        if !text.trim().is_empty() {
            lines.push(text);
        }
        return;
    }
    for child in &node.children {
        collect_lines(child, lines);
    }
}

fn collect_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, text);
    }
}

fn without_space(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 대상 셀(marker 텍스트를 담은 셀) 안 TextRun 중 칸 우변을 넘는 것을 모은다.
fn walk(
    node: &RenderNode,
    cell_right: Option<f64>,
    in_target_cell: bool,
    checked: &mut usize,
    out: &mut Vec<(String, String)>,
) {
    let (cell_right, in_target_cell) = match &node.node_type {
        RenderNodeType::TableCell(_) => {
            let holds_marker = subtree_contains(node, CELL_MARKER);
            (
                holds_marker.then_some(node.bbox.x + node.bbox.width),
                holds_marker,
            )
        }
        _ => (cell_right, in_target_cell),
    };
    if let (Some(right), true, RenderNodeType::TextRun(run)) =
        (cell_right, in_target_cell, &node.node_type)
    {
        if !run.text.trim().is_empty() {
            *checked += 1;
            let run_right = node.bbox.x + node.bbox.width;
            if run_right > right + TRAILING_SPACE_ALLOWANCE_PX {
                out.push((format!("+{:.1}", run_right - right), run.text.clone()));
            }
        }
    }
    for child in &node.children {
        walk(child, cell_right, in_target_cell, checked, out);
    }
}

fn subtree_contains(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| subtree_contains(c, needle))
}
