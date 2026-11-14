//! [#9073] 관계형·고정형 계약 — 글자처럼 취급 표가 혼자 소유한 저장 줄은 그 줄의 전진을 따른다.
//!
//! 주석 갱신: 2026-11-14 15:29 KST(+09:00). 새 파일.
//!
//! 유형: 쪽수(1쪽·2쪽)는 고정형(독립 한컴 Print 기대값). 표의 쪽 소속과, 표 뒤 문단의 줄 위치가
//! 원본 저장 vpos 와 같다는 검사는 관계형이다. 절대 px 좌표는 고정하지 않는다.
//!
//! 대상: `treatAsChar="1" flowWithText="0"` 표가 문단의 유일한 저장 줄을 혼자 소유한다(줄 높이 =
//! 표 높이 + 위·아래 바깥 여백, 보이는 글과 다른 표 없음). HWPX 는 두 속성이 모두 참일 때만 표를
//! 글자처럼 취급 흐름으로 보내므로(#3930) 수정 전에는 이 표가 블록 경로로 가서 표 본체 높이만큼만
//! 전진하고, 저장 줄의 전진(줄 높이 + 줄간격)과 문단 끝 줄간격 복원을 받지 못했다.
//!
//! 시각 측정(2026-11-14, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/hwp`·`ttfs/windows`, 기준 한컴오피스 2022
//! 12.0.0.4547 Print → Hancom PDF):
//! - `samples/issue9073/veterans_summer_support_press_release.hwpx`(국가보훈부 보도자료 156712161)
//!   2쪽 = 한컴 2쪽. Native 99.37·99.92(최저 99.37), fresh WASM 99.88·99.99(최저 99.88). 수정 전 Native 1쪽 68.64.
//! - `samples/issue9073/proficiency_test_result_form.hwpx`(행정규칙 [별지 4] 숙련도 시험 결과서
//!   2685463) 1쪽 = 한컴 1쪽. Native 99.94, fresh WASM 99.97. 수정 전 rhwp 2쪽(쪽수 불일치).
//! - 증거: `mydocs/pr/assets/issue_9073/`(전쪽 Native·fresh WASM TSV, 대표 review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::path::PathBuf;

const PRESS: &str = "samples/issue9073/veterans_summer_support_press_release.hwpx";
const FORM: &str = "samples/issue9073/proficiency_test_result_form.hwpx";

fn load(rel: &str) -> DocumentCore {
    let bytes = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)).expect(rel);
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn collect<'a>(
    node: &'a RenderNode,
    pick: &dyn Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if pick(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, pick, out);
    }
}

fn has_table(paragraph: &rhwp::model::paragraph::Paragraph) -> bool {
    paragraph
        .controls
        .iter()
        .any(|control| matches!(control, Control::Table(_)))
}

/// 표 줄 뒤 문단은 저장 vpos 자리에서 시작한다.
///
/// 머리 표 문단 0·1 의 저장 줄은 줄간격이 −600HU 다(전진 = 줄 높이 − 600). 문단 0 의 표는
/// `flowWithText="0"` 이다. 한컴 2022 Print 는 뒤 문단들을 저장 vpos 그대로 놓는다. 수정 전에는
/// 문단 0 이 표 본체 높이만큼 전진해 뒤 내용이 저장 위치보다 내려갔다(쪽 끝에서 +4.7px).
///
/// 관계: 같은 쪽의 본문 문단 두 개에서 (줄 위끝 − 단 위끝) = 저장 vpos. 기대값은 원본 저장값이다.
#[test]
fn paragraphs_after_a_sole_stored_table_line_start_at_their_stored_positions() {
    let core = load(PRESS);
    assert_eq!(core.page_count(), 2, "한컴 2022 Print 의 전체 쪽수");
    let paragraphs = &core.document().sections[0].paragraphs;
    assert!(has_table(&paragraphs[0]) && paragraphs[0].line_segs.len() == 1);
    assert!(
        paragraphs[0].line_segs[0].line_spacing < 0,
        "머리 표 줄은 음수 줄간격으로 저장되어 있다"
    );

    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut columns = Vec::new();
    collect(
        &tree.root,
        &|node| matches!(node.node_type, RenderNodeType::Column(_)),
        &mut columns,
    );
    let column_top = columns.first().expect("본문 단").bbox.y;

    let mut checked = 0;
    for (index, paragraph) in paragraphs.iter().enumerate().skip(2) {
        if has_table(paragraph)
            || paragraph.text.trim().is_empty()
            || !paragraph.controls.is_empty()
        {
            continue;
        }
        let Some(first) = paragraph.line_segs.first() else {
            continue;
        };
        let mut lines = Vec::new();
        collect(
            &tree.root,
            &|node| {
                matches!(&node.node_type, RenderNodeType::TextLine(line)
                    if line.section_index == Some(0)
                        && line.para_index == Some(index)
                        && line.line_index == Some(0))
            },
            &mut lines,
        );
        let Some(line) = lines.first() else {
            // 다음 쪽 문단이거나 칸 안 문단이다.
            continue;
        };
        let expected = rhwp::renderer::hwpunit_to_px(first.vertical_pos, 96.0);
        let actual = line.bbox.y - column_top;
        assert!(
            (actual - expected).abs() <= 1.0,
            "문단 {index} 의 첫 줄은 저장 vpos 자리에 놓인다: actual={actual:.2}px expected={expected:.2}px"
        );
        checked += 1;
        if checked == 2 {
            break;
        }
    }
    assert_eq!(checked, 2, "표 뒤 본문 문단 두 개를 검사한다");
}

/// 혼자 저장 줄을 소유한 글자처럼 취급 표는 쪽을 넘겨 나누지 않는다.
///
/// 20×8 표의 저장 줄 높이(73457HU)는 표 높이 + 바깥 여백과 같고 본문보다 조금 크다. 한컴 2022
/// Print 는 이 표를 1쪽에 통째로 두어 문서가 1쪽이다. 수정 전에는 블록 표로 나눠 마지막 행이
/// 2쪽으로 넘어갔다.
///
/// 관계: 표의 모든 칸이 1쪽에 있고 조각으로 나뉘지 않는다.
#[test]
fn a_sole_stored_line_table_stays_whole_on_its_page() {
    let core = load(FORM);
    assert_eq!(core.page_count(), 1, "한컴 2022 Print 의 전체 쪽수");
    let paragraphs = &core.document().sections[0].paragraphs;
    let table = paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("본문 표");
    assert!(table.common.treat_as_char && !table.common.flow_with_text);

    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut cells = Vec::new();
    collect(
        &tree.root,
        &|node| matches!(&node.node_type, RenderNodeType::TableCell(_)),
        &mut cells,
    );
    let top_level = cells
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell)
                if cell.model_cell_index.is_some() && !cell.page_fragment)
        })
        .count();
    assert!(
        top_level >= table.cells.len(),
        "표의 모든 칸이 1쪽에 온전히 있다: {top_level} < {}",
        table.cells.len()
    );
}
