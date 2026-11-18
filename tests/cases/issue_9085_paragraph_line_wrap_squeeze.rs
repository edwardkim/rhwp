//! [#9085] 관계형·고정형 계약 — 저장 줄 없는 문단 「한 줄로 입력」은 줄을 나누지 않고 자간을 좁힌다.
//!
//! 주석 갱신: 2026-11-15 10:00 KST(+09:00). 새 파일.
//!
//! 유형: 쪽수(1쪽)는 고정형(독립 한컴 Print 기대값). 「한 줄로 입력」 문단마다 글줄이 하나이고 그
//! 글자가 칸 안쪽에 있다는 검사는 관계형이다. 절대 px 좌표는 고정하지 않는다.
//!
//! 대상: `samples/issue9085/facility_fee_reduction_criteria.hwpx`(전남광주통합특별시
//! 코리아인터내셔널서킷 관리ㆍ운영 조례 [별표 4] 부대시설 사용료의 감면기준, 공개 자치법규 별표,
//! 저장 제품 hancom-office-2018). 저장 줄(`linesegarray`)이 없고, 글자처럼 취급 표의 항목 문단이
//! `breakSetting lineWrap="SQUEEZE"`(ParaShape attr2 bits 0..1 = 1)이다. 한컴은 항목마다 한 줄로
//! 두고 넘치는 줄의 자간을 좁혀 칸 안에 담는다. 표 선언 높이는 60350HU 이고 한 쪽에 들어간다.
//!
//! 수정 전에는 긴 항목을 칸 폭에서 낱말 단위로 나눠 두 줄이 됐고, 표가 선언보다 높아져(1,092px)
//! 2쪽으로 넘어갔다. 한 줄로만 두면 자연 폭이 가용 폭의 1.15배를 넘는 줄은 칸 경계에서 잘렸다.
//!
//! 시각 측정(2026-11-15, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/hwp`·`ttfs/windows`. 기준
//! `pdf/issue9085/facility_fee_reduction_criteria-print-2022.pdf` — 한컴오피스 2022 12.0.0.4547
//! Print → Hancom PDF):
//! - 1쪽 = 한컴 1쪽. Native 99.96, fresh WASM 99.97(전쪽 최저도 같은 값). 수정 전은 2쪽
//!   (쪽수 불일치).
//! - 확대 판독: 넘치는 줄(2·3·6·7·8·9·13·14번 항목)이 한컴과 같이 칸 오른쪽 안에서 끝난다. 한컴은 줄
//!   안에서 자간을 고르게 좁히지 않는 곳이 있어(3번 항목 법령 이름 구간은 덜 좁힘) 낱자 위치가
//!   1~3px 다르다. 이 배분 규칙은 확인하지 않았다.
//! - 증거: `mydocs/pr/assets/issue_9085/`(전쪽 Native·fresh WASM TSV, 대표 review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::style::PARA_ATTR2_LINE_WRAP_MASK;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::BTreeMap;
use std::path::PathBuf;

const SOURCE: &str = "samples/issue9085/facility_fee_reduction_criteria.hwpx";

fn core() -> DocumentCore {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).expect(SOURCE);
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

/// (행, 열) → 그 칸의 글자가 있는 「한 줄로 입력」 문단 수. 원본 문단 모양에서 센다.
fn squeeze_paragraphs_by_cell(core: &DocumentCore) -> BTreeMap<(u16, u16), usize> {
    let document = core.document();
    let squeeze = |shape_id: u16| {
        document
            .doc_info
            .para_shapes
            .get(shape_id as usize)
            .is_some_and(|shape| shape.attr2 & PARA_ATTR2_LINE_WRAP_MASK == 1)
    };
    let table = document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("별표의 표");
    let mut cells = BTreeMap::new();
    for cell in &table.cells {
        let count = cell
            .paragraphs
            .iter()
            .filter(|paragraph| {
                squeeze(paragraph.para_shape_id) && !paragraph.text.trim().is_empty()
            })
            .count();
        let all_squeeze = cell
            .paragraphs
            .iter()
            .filter(|paragraph| !paragraph.text.trim().is_empty())
            .all(|paragraph| squeeze(paragraph.para_shape_id));
        if count > 0 && all_squeeze {
            cells.insert((cell.row, cell.col), count);
        }
    }
    cells
}

/// 표는 한컴과 같이 한 쪽에 들어간다.
///
/// 고정: 전체 1쪽(한컴 2022 Print). 관계: 「한 줄로 입력」 문단만 있는 칸은 글자 있는 문단 수와
/// 글자 있는 글줄 수가 같다(문단마다 한 줄). 수정 전에는 긴 항목이 두 줄이 돼 표가 2쪽으로 갔다.
#[test]
fn squeeze_paragraphs_stay_on_one_line_and_the_table_fits_the_page() {
    let core = core();
    let expected = squeeze_paragraphs_by_cell(&core);
    assert!(
        expected.values().any(|&count| count >= 10),
        "항목 문단이 모인 「한 줄로 입력」 칸이 있다: {expected:?}"
    );
    assert_eq!(core.page_count(), 1, "한컴 2022 Print 의 전체 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut cells = Vec::new();
    collect(
        &tree.root,
        &|node| matches!(node.node_type, RenderNodeType::TableCell(_)),
        &mut cells,
    );
    let mut seen = 0;
    for cell in cells {
        let RenderNodeType::TableCell(info) = &cell.node_type else {
            continue;
        };
        let Some(&paragraphs) = expected.get(&(info.row, info.col)) else {
            continue;
        };
        let mut lines = Vec::new();
        collect(
            cell,
            &|node| {
                matches!(node.node_type, RenderNodeType::TextLine(_))
                    && node.children.iter().any(|run| {
                        matches!(&run.node_type, RenderNodeType::TextRun(run)
                            if !run.text.trim().is_empty())
                    })
            },
            &mut lines,
        );
        assert_eq!(
            lines.len(),
            paragraphs,
            "{}행 {}열: 「한 줄로 입력」 문단마다 글줄이 하나다",
            info.row,
            info.col
        );
        seen += 1;
    }
    assert_eq!(seen, expected.len(), "모든 대상 칸이 1쪽에 있다");
}

/// 넘치는 줄은 자간을 좁혀 칸 안에 담는다.
///
/// 관계: 「한 줄로 입력」 칸의 글자 런 오른쪽 끝은 칸 오른쪽 괘선을 넘지 않는다. 수정 전에는 줄을
/// 나눴고, 한 줄로만 두면 자연 폭이 가용 폭의 1.15배를 넘는 줄이 칸 밖으로 나가 잘렸다.
#[test]
fn overfull_squeeze_lines_are_tightened_inside_the_cell() {
    let core = core();
    let expected = squeeze_paragraphs_by_cell(&core);
    let mut checked = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽");
        let mut cells = Vec::new();
        collect(
            &tree.root,
            &|node| matches!(node.node_type, RenderNodeType::TableCell(_)),
            &mut cells,
        );
        for cell in cells {
            let RenderNodeType::TableCell(info) = &cell.node_type else {
                continue;
            };
            if !expected.contains_key(&(info.row, info.col)) {
                continue;
            }
            let right = cell.bbox.x + cell.bbox.width;
            let mut runs = Vec::new();
            collect(
                cell,
                &|node| {
                    matches!(&node.node_type, RenderNodeType::TextRun(run)
                        if !run.text.trim().is_empty())
                },
                &mut runs,
            );
            for run in runs {
                let end = run.bbox.x + run.bbox.width;
                assert!(
                    end <= right + 0.5,
                    "{}행 {}열: 글자 런이 칸 안에 있다: 런 끝 {end:.2}px, 칸 끝 {right:.2}px",
                    info.row,
                    info.col
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 14, "항목 글자 런을 검사했다: {checked}");
}
