//! 관계형: 쪽 끝에서 선언 높이는 넘치지만 내용은 드는 0행이 병합 칸 묶음을 연 RowBreak 표에서, 0행이 앞 쪽 바닥에서 닫히고 걸친 병합 칸의 남은 선언 높이가 다음 조각 첫 행으로 이어지는지를 행 소속과 원본 저장 높이 관계로 검사한다.
//! 주석 갱신: 2026-11-10 08:05 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-10, `scripts/visual_sweep.py --silhouette-only --dpi 96` silhouette_raw_match_percent(색상 경계 보정 전 2px 내용 실루엣), Native(native-skia)·fresh WASM 각 235쪽 전쪽, 수정 전 = #8849 head d195bce988, 수정 후 = 0f24da33cc.
//! 원본 `samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp`(저장 제품 한/글 2010), 전체 235쪽 = 한/글 2020 Print 235쪽.
//! 18쪽: Native 99.93 / fresh WASM 99.90(수정 전 94.47 / 94.73). 19쪽: Native 99.95 / fresh WASM 99.95(수정 전 37.39 / 38.10). 전쪽 최저: Native 0.35 / fresh WASM 0.41 — 수정 전과 같은 값이며 이 범위 밖이다(76~172쪽 표 쪽 나눔 등 별도 축). 90% 미만 Native 102→101쪽, WASM 100→99쪽, 18·19쪽 밖 쪽 점수 변화 없음.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8875/{base,head}/{native,wasm}/hpv_cohort_final_report_silhouette.tsv`, `png/`.
//! 확대 판독: 18·19쪽 review·overlay 를 한/글 2020 Print 와 같은 배율로 대조 — 18쪽 바닥 「2018 | 혈청(Serum) | …」 0행 조각의 위아래 괘선과 19쪽 첫 행 「명 | vial」·이후 7행 괘선이 PDF 와 겹친다.
//! 같은 assertion 이 수정 전 FAIL(18쪽에 pi273 조각 없음, 표 전체가 19쪽) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8842/1351000-201800239_hpv_cohort_final_report-2020.pdf` 18쪽 — 표 0행(「2018 | 혈청(Serum) | …」)만 쪽 바닥에 2232HU(저장 표 높이 2229HU)로 닫히고, 19쪽 첫 행(「명 | vial | …」)은 2784HU = 병합 칸 「2018」 5000HU − 2229HU 다(300DPI 괘선 측정).
//! 저장 근거: 구역 0 pi273 9×9 RowBreak 표, 저장 높이 2229HU, 0행 선언 2500HU, 「2018」 칸 r=0 rs=2 h=5000HU, 남은 공간 32.9px < 0행 33.3px, 0행 내용 한 줄 1331HU + 안 여백 282HU.
//! 반례: #1145 표본(잔여 15.2px 에 rowspan 제목 18.7px 이 안 듦)과 task1753 pi=51(rowspan 내용이 0행을 넘음)은 묶음을 통째로 넘긴다 — `tests/cases/rowbreak_rowspan_opening_row_cut.rs` 가 그대로 통과한다.
//! 독립 절대 px 좌표 대신 행의 쪽 소속과 원본 저장 높이(HU)에서 계산한 조각·행 높이 관계를 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp";
const PARA: usize = 273;

fn px(hu: f64) -> f64 {
    hu * 96.0 / 7200.0
}

/// 구역 0 pi273 표 조각의 상자와 그 칸들(행, 병합 수, 상자).
fn fragment(node: &RenderNode) -> Option<(BoundingBox, Vec<(u16, u16, BoundingBox)>)> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(0)
            && table.para_index == Some(PARA)
            && table.cell_context.is_none()
        {
            let cells = node
                .children
                .iter()
                .filter_map(|child| match &child.node_type {
                    RenderNodeType::TableCell(cell) => Some((cell.row, cell.row_span, child.bbox)),
                    _ => None,
                })
                .collect();
            return Some((node.bbox, cells));
        }
    }
    node.children.iter().find_map(fragment)
}

#[test]
fn opening_row_closes_at_page_bottom_and_span_remainder_continues() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), 235, "한/글 2020 Print 와 같은 전체 쪽수");

    let first = core.build_page_render_tree(17).expect("18쪽 배치");
    let (first_box, first_cells) = fragment(&first.root).expect("18쪽에 pi273 첫 조각");
    let mut first_rows: Vec<u16> = first_cells.iter().map(|(row, _, _)| *row).collect();
    first_rows.dedup();
    assert_eq!(
        first_rows,
        vec![0],
        "18쪽 조각은 0행만 연다: {first_cells:?}"
    );
    assert!(
        (first_box.height - px(2229.0)).abs() <= 0.5,
        "18쪽 조각은 저장 표 높이 2229HU 로 닫힌다: {first_box:?}"
    );

    let next = core.build_page_render_tree(18).expect("19쪽 배치");
    let (_, next_cells) = fragment(&next.root).expect("19쪽에 pi273 이어받는 조각");
    let row_height = |row: u16| {
        next_cells
            .iter()
            .find(|(r, span, _)| *r == row && *span == 1)
            .map(|(_, _, bbox)| bbox.height)
            .unwrap_or_else(|| panic!("19쪽 {row}행 칸: {next_cells:?}"))
    };
    assert!(
        next_cells
            .iter()
            .all(|(row, span, _)| *row >= 1 || *span > 1),
        "19쪽 조각은 1행부터 이어받는다: {next_cells:?}"
    );
    assert!(
        next_cells.iter().any(|(row, _, _)| *row == 8),
        "19쪽 조각은 마지막 8행까지 담는다"
    );
    // 「2018」 병합 칸(5000HU)이 앞 쪽에서 2229HU 를 쓰고 남긴 2771HU 가 1행을 키운다.
    let carried = row_height(1) - row_height(2);
    assert!(
        (carried - px(5000.0 - 2229.0 - 2500.0)).abs() <= 0.5,
        "19쪽 1행은 병합 칸의 남은 선언만큼 크다: 차이 {carried:.2}px"
    );
}
