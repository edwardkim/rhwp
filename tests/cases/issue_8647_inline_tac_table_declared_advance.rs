//! 관계형: 한 줄에 나란한 글자처럼 취급 표의 앞뒤 순서·비겹침·본문 폭 포함과 셀 소속을 검사한다.
//! 주석 갱신: 2026-11-06T04:35:18+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-06 KST, source = upstream/devel 170b886f16 + 이 수정(`layout_inline_table_paragraph`
//! 의 표 뒤 펜 전진을 선언 폭으로). 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw,
//! 독립 한컴 2020 Print(`pdf/issue8647/20650735_overpayment_refund_notice-2020.pdf`, 1쪽).
//! 입력 `samples/issue8647/20650735_overpayment_refund_notice.hwp`(10k 코퍼스 공개 조례 별지 서식), 전체 1쪽.
//! 수정 전 Native 39.76%(쪽수 1=1). 수정 후 Native 99.66% / fresh WASM 100.00% — 전쪽 최저도 같다.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8647/{native,wasm,png}/`.
//! 확대 판독: 표 네 개의 왼끝·괘선·칸 글자가 PDF 와 겹친다. `【` 글리프 차이는 수정 전과 같다.
//! 같은 assertion 이 수정 전 FAIL(표1 x=104.6 < 표0 오른쪽 끝 207.6) / 수정 후 PASS.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다(표 상자는 선언 폭에서 나온다).

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8647/20650735_overpayment_refund_notice.hwp";
/// 표 네 개를 한 줄에 담은 문단(`[표] 빈칸 U+2007` 반복, 저장 줄 1개).
const HOST_PARA: usize = 9;

fn open(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

/// 최외곽 표 노드와 그 표가 소유한 셀 상자들.
fn host_tables(node: &RenderNode, out: &mut Vec<(usize, BoundingBox, Vec<BoundingBox>)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(HOST_PARA) && table.cell_context.is_none() {
            let cells = node
                .children
                .iter()
                .filter(|child| matches!(child.node_type, RenderNodeType::TableCell(_)))
                .map(|child| child.bbox)
                .collect();
            out.push((
                table.control_index.expect("표 컨트롤 번호"),
                node.bbox,
                cells,
            ));
            return;
        }
    }
    for child in &node.children {
        host_tables(child, out);
    }
}

fn body_area(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(body_area)
}

#[test]
fn inline_tac_tables_on_one_line_advance_by_their_own_width() {
    let core = open(SOURCE);
    assert_eq!(core.page_count(), 1, "독립 Print 와 같은 1쪽");
    let tree = core.build_page_render_tree(0).expect("쪽 배치");
    let mut tables = Vec::new();
    host_tables(&tree.root, &mut tables);
    tables.sort_by_key(|(control, _, _)| *control);
    assert_eq!(
        tables
            .iter()
            .map(|(control, _, _)| *control)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3],
        "한 줄의 표 네 개가 누락·중복 없이 배치된다"
    );

    // 한/글 Print 는 표를 왼쪽부터 차례로 놓고 서로 겹치지 않는다. 병합 셀이 많은 표0 의
    // 열 폭 합(col_span==1 셀 최댓값 합)은 실제 폭보다 작아, 그 값으로 전진하면 다음 표가
    // 앞 표 위에 겹친다.
    for pair in tables.windows(2) {
        let (prev_control, prev, _) = &pair[0];
        let (next_control, next, _) = &pair[1];
        assert!(
            next.x + 0.5 >= prev.x + prev.width,
            "표{next_control} 은 표{prev_control} 오른쪽 끝 뒤에서 시작한다: prev={prev:?}, next={next:?}"
        );
    }

    // 표들은 본문 폭 안에 머문다(넘쳐 용지 밖으로 잘리지 않는다).
    let body = body_area(&tree.root).expect("본문 영역");
    let last = &tables.last().expect("마지막 표").1;
    assert!(
        last.x + last.width <= body.x + body.width + 1.0,
        "마지막 표가 본문 오른쪽 끝을 넘지 않는다: last={last:?}, body={body:?}"
    );

    // 각 표의 셀은 자기 표 상자 안에 있다 — 전진 폭이 표 자신의 폭과 같은 값이다.
    for (control, table, cells) in &tables {
        assert!(!cells.is_empty(), "표{control} 의 셀이 그려진다");
        for cell in cells {
            assert!(
                cell.x + 0.5 >= table.x && cell.x + cell.width <= table.x + table.width + 0.5,
                "표{control} 의 셀이 자기 표 가로 범위 안에 있다: table={table:?}, cell={cell:?}"
            );
        }
    }
}
