//! 관계형: RowBreak 표 중간에서 병합 칸이 시작하는 행이 쪽 끝에 걸릴 때, 그 칸의 저장 글줄이 칸 안에서 쪽 프레임을 다시 시작하면 행을 앞 쪽 바닥에 두고, 그런 기록이 없으면 행을 통째로 다음 쪽으로 넘기는지를 행의 쪽 소속으로 검사한다.
//! 주석 갱신: MEASURED_AT
//! MEASUREMENT
//! 독립 기대값: 한/글 2020 Print — HPV `pdf/issue8842/1351000-201800239_hpv_cohort_final_report-2020.pdf` 77쪽 바닥에 구역 2 표 20행(「지난 1년간 | 없음 | 127 (24.8) …」)이 있고 78쪽은 「구강성교 횟수 (한 달 평균)」·21행(「3회 미만」)부터다. 반례 `pdf/issue8891/1480000-201900510_facility_pollution_study-2020.pdf` 113쪽은 27행(「유틸리티 시설 | 스팀생산 | … 보일러 운영 방안」)으로 시작한다.
//! 저장 근거: HPV 구역 2 pi23 34×5 표 20행 선언 1650HU, 남은 공간 20.9px. 20행에서 시작하는 병합 칸(r=20, rs=4) p[0] 의 ls[1] 이 vpos=0 으로 되감긴다. 반례 구역 0 pi920 38×4 표 27행 선언 20.9px, 남은 공간 14.5px, 일반 칸 내용은 들지만 27행에서 시작하는 병합 칸(rs=7·4·3)은 모두 한 줄이고 되감김이 없다.
//! 독립 절대 px 좌표 대신 행의 쪽 소속(어느 쪽 표 조각에 어느 행 칸이 있는가)을 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HPV: &str = "samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp";
const FACILITY: &str = "samples/issue8891/1480000-201900510_facility_pollution_study.hwp";

/// 쪽의 (구역, 문단) 최상위 표 조각에 있는 칸들의 (행, 병합 수).
fn fragment_cells(core: &DocumentCore, page: u32, section: usize, para: usize) -> Vec<(u16, u16)> {
    fn walk(node: &RenderNode, section: usize, para: usize, out: &mut Vec<(u16, u16)>) {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.section_index == Some(section)
                && table.para_index == Some(para)
                && table.cell_context.is_none()
            {
                for child in &node.children {
                    if let RenderNodeType::TableCell(cell) = &child.node_type {
                        out.push((cell.row, cell.row_span));
                    }
                }
                return;
            }
        }
        for child in &node.children {
            walk(child, section, para, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("쪽 배치");
    let mut out = Vec::new();
    walk(&tree.root, section, para, &mut out);
    out
}

fn ordinary_rows(cells: &[(u16, u16)]) -> Vec<u16> {
    let mut rows: Vec<u16> = cells
        .iter()
        .filter(|(_, span)| *span == 1)
        .map(|(row, _)| *row)
        .collect();
    rows.sort_unstable();
    rows.dedup();
    rows
}

#[test]
fn span_cell_with_stored_page_reset_keeps_its_opening_row_on_the_page() {
    let core =
        DocumentCore::from_bytes(&std::fs::read(HPV).expect("HPV 입력")).expect("HPV 문서 열기");
    // 77쪽(0 기준 76) 조각은 20행까지, 78쪽 조각은 21행부터 일반 칸을 담는다.
    let first = ordinary_rows(&fragment_cells(&core, 76, 2, 23));
    assert_eq!(
        first.last(),
        Some(&20),
        "77쪽 조각의 마지막 일반 행은 20행: {first:?}"
    );
    let next_cells = fragment_cells(&core, 77, 2, 23);
    let next = ordinary_rows(&next_cells);
    let body_rows: Vec<u16> = next.iter().copied().filter(|row| *row >= 2).collect();
    assert_eq!(
        body_rows.first(),
        Some(&21),
        "78쪽 조각은 21행부터 잇는다: {next:?}"
    );
    assert!(
        next_cells
            .iter()
            .any(|(row, span)| *row == 20 && *span == 4),
        "20행에서 시작한 병합 칸은 78쪽에서도 이어진다: {next_cells:?}"
    );
}

#[test]
fn span_cell_without_stored_reset_moves_its_opening_row_whole() {
    let core = DocumentCore::from_bytes(&std::fs::read(FACILITY).expect("반례 입력"))
        .expect("반례 문서 열기");
    // 113쪽(0 기준 112) 조각은 27행으로 시작한다 — 27행이 앞 쪽에 남으면 28행으로 시작한다.
    // 112쪽은 이 수정과 무관한 차이로 90% 미만이라 검사 범위에 넣지 않는다.
    let next = ordinary_rows(&fragment_cells(&core, 112, 0, 920));
    assert_eq!(
        next.first(),
        Some(&27),
        "113쪽 조각은 27행으로 시작한다: {next:?}"
    );
}
