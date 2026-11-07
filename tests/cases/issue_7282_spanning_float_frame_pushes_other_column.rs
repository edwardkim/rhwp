//! 관계형(줄·표 겹침, 줄 순서)·고정형(쪽수): 2단 쪽에서 오른쪽 단 문단에 앵커한 두 단 걸침 자리차지 표가, 앵커가 없는 왼쪽 단 문단의 줄도 표 아래로 밀어내는지 검사한다.
//! 주석 갱신: 2026-11-08T05:40:00+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-08 KST, base = upstream/devel 5077b216cb, head = base + 이 PR 의 src 변경. 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw.
//! 원본 `samples/issue7282/wide_paper_table_two_columns.hwp`, 전체 2쪽=2쪽.
//! 1쪽 수정 전 Native 74.70 / fresh WASM 73.59 → 수정 후 Native 93.45 / fresh WASM 92.47. 2쪽 Native 96.35 / WASM 95.98 그대로.
//! 수정 후 전쪽 최저 Native 93.45·fresh WASM 92.47(90% 미만 0쪽). 98% 미만이라 px 좌표는 고정하지 않는다.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_7282/{base,head}/{native,wasm}/wide_paper_table_two_columns_silhouette.tsv`, `png/`.
//! 확대 판독: 1쪽 표와 두 단의 표 위·아래 줄이 한/글과 같은 자리(overlay 일치). 수정 전에는 왼쪽 단 줄이 표 위에 겹쳐 그려졌다. 오른쪽 단 마지막 문단이 약 5px 위에 놓이는 차이는 수정 전부터 있던 것이다.
//! 같은 assertion 이 수정 전 FAIL(왼쪽 단 문단 0.6 줄이 표와 겹침) / 수정 후 PASS.
//! 독립 기대값: 한/글 Print `pdf/issue7282/wide_paper_table_two_columns-2020.pdf`(2쪽). 1쪽 두 단 모두 표 위 줄과 표 아래 줄 사이에 표가 비켜 있고, 글자가 표와 겹치지 않는다.
//! 저장 근거: 문단 0.6(왼쪽 단)·0.12(오른쪽 단)의 줄 사다리 19200 → 32008 점프 = 줄 피치 1600 + 표 높이 8374 + 바깥 위·아래 1417×2. 표는 문단 0.13 의 종이 기준 자리차지 표다.
//! 독립 절대 px 좌표 대신 표 상자와 줄 상자의 겹침·순서 관계를 쓴다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue7282/wide_paper_table_two_columns.hwp";
const PDF_PAGES: u32 = 2;
const TABLE_PARA: usize = 13;
/// 저장 사다리에서 표 틀 아래로 점프하는 문단: 왼쪽 단(앵커 없음), 오른쪽 단.
const PUSHED_PARAS: [usize; 2] = [6, 12];

fn table_frame(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(TABLE_PARA) && table.cell_context.is_none() {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(table_frame)
}

fn para_lines(node: &RenderNode, para: usize, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) {
            out.push(node.bbox);
            return;
        }
    }
    for child in &node.children {
        para_lines(child, para, out);
    }
}

#[test]
fn spanning_paper_table_pushes_lines_in_both_columns() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽 배치");
    let table = table_frame(&tree.root).expect("1쪽 두 단 걸침 표");
    for para in PUSHED_PARAS {
        let mut lines = Vec::new();
        para_lines(&tree.root, para, &mut lines);
        assert!(lines.len() >= 3, "문단 0.{para} 의 줄: {lines:?}");
        assert!(
            lines.windows(2).all(|w| w[0].y < w[1].y),
            "문단 0.{para} 의 줄은 저장 순서대로 아래로 간다: {lines:?}"
        );
        for line in &lines {
            let overlaps =
                line.y + line.height > table.y + 0.5 && line.y < table.y + table.height - 0.5;
            assert!(
                !overlaps,
                "문단 0.{para} 의 줄이 표와 겹친다: line={line:?} table={table:?}"
            );
        }
        assert!(
            lines.iter().any(|l| l.y + l.height <= table.y + 0.5),
            "문단 0.{para} 의 앞 줄은 표 위에 남는다: {lines:?} table={table:?}"
        );
        assert!(
            lines.iter().any(|l| l.y >= table.y + table.height - 0.5),
            "문단 0.{para} 의 나머지 줄은 표 아래로 간다: {lines:?} table={table:?}"
        );
    }
}
