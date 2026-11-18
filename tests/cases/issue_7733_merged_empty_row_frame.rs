//! 검사 유형: 관계형 — 병합 제약이 이미 충족된 미결 행에 별도 최소 띠를 만들지 않는다.
//! 주석 갱신: 2026-11-18 04:19 KST.
//! 화학 문서 85쪽 Native raw 98.46202%, 최신 fresh WASM은 미검증이다.
//! 같은 원본의 한컴 2020 Hancom PDF Print와 위·아래 정렬 대조에서 작은 표의
//! 정렬 대상은 앞의 두 행 병합 칸과 같은 위끝에서 시작한다. 저장 기준선 이동이 아니다.
//! 절대 PDF 좌표를 고정하지 않고 같은 행 구간의 시작 경계와 원래 내용 소속을 비교한다.

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn owner(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.para_index == Some(173) && meta.control_index == Some(0))
    {
        return Some(node);
    }
    node.children.iter().find_map(owner)
}

#[test]
fn merged_spans_that_already_fit_do_not_insert_an_empty_opening_band() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let Control::Table(source) = &core.document().sections[4].paragraphs[173].controls[0] else {
        panic!("같은 원본의 부모 표");
    };
    assert_eq!((source.row_count, source.col_count), (8, 9));
    assert!(source
        .cells
        .iter()
        .filter(|cell| cell.row == 6)
        .all(|cell| cell.row_span > 1));
    assert_eq!(core.page_count(), 103);
    let tree = core.build_page_render_tree(84).unwrap();
    let parent = owner(&tree.root).unwrap();
    let cell = |row, col| {
        parent
            .children
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::TableCell(meta)
            if meta.row == row && meta.col == col)
            })
            .unwrap()
    };
    let preceding = cell(6, 3);
    let label = cell(7, 0);
    assert!(
        format!("{label:?}").contains("어린이보호포장"),
        "작은 표의 내용 소속"
    );
    assert!(
        (label.bbox.y - preceding.bbox.y).abs() <= 96.0 / 7200.0,
        "병합 칸의 내용으로 앞 행 구간이 이미 닫혔으면 미결 행의 높이는 0이다: 앞={:?}, 라벨={:?}",
        preceding.bbox,
        label.bbox
    );
}
