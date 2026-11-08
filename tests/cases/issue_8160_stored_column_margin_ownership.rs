//! #8160: sample16 회귀 검사 통합. 이전 등록: tests/cases/issue_7642_stored_column_margin_owner_relations.rs
//! #7642: 단 첫 여백을 글줄 내용 하단과 후속 표가 같은 축으로 소비한다.
//! 독립 2024 Print·Native/fresh WASM 전쪽 검증 뒤 정식 회귀로 추가한다.
//! 절대 화면 좌표 대신 원본 줄의 간격을 줄/표 높이로 정규화해 소유·순서를 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5.hwp");
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 로드");
    core.set_hangul2024_compat(true);
    core
}

fn body_lines(node: &RenderNode, owner: usize, out: &mut Vec<BoundingBox>) {
    // 표 셀은 본문과 독립된 소유 공간이다. 숫자가 같은 셀 문단을 섞지 않는다.
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.section_index == Some(0) && line.para_index == Some(owner) {
            out.push(node.bbox);
        }
    }
    for child in &node.children {
        body_lines(child, owner, out);
    }
}

fn body_tables(node: &RenderNode, owner: usize, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(0)
            && table.para_index == Some(owner)
            && table.control_index == Some(0)
            && table.cell_context.is_none()
        {
            out.push(node.bbox);
        }
        return;
    }
    for child in &node.children {
        body_tables(child, owner, out);
    }
}

fn line(root: &RenderNode, owner: usize) -> BoundingBox {
    let mut found = Vec::new();
    body_lines(root, owner, &mut found);
    assert_eq!(found.len(), 1, "한 줄 원본의 본문 소유/누락/중복: {owner}");
    found[0]
}

fn table(root: &RenderNode, owner: usize) -> BoundingBox {
    let mut found = Vec::new();
    body_tables(root, owner, &mut found);
    assert_eq!(found.len(), 1, "전체 표의 본문 소유/누락/중복: {owner}");
    found[0]
}

fn equal_ratio(actual: f64, expected: f64, context: &str) {
    let roundoff = f64::EPSILON * actual.abs().max(expected.abs()).max(1.0) * 256.0;
    assert!(
        (actual - expected).abs() <= roundoff,
        "{context}: observed={actual}, source={expected}"
    );
}

fn assert_saved_table_axis(
    core: &DocumentCore,
    page: u32,
    title: usize,
    preceding: usize,
    owner: usize,
    following: Option<usize>,
) {
    let paragraphs = &core.document().sections[0].paragraphs;
    let previous = &paragraphs[preceding];
    let host = &paragraphs[owner];
    let [previous_source] = previous.line_segs.as_slice() else {
        panic!("직전 저장 한 줄");
    };
    let [host_source] = host.line_segs.as_slice() else {
        panic!("표 저장 한 줄");
    };
    let [Control::Table(source_table)] = host.controls.as_slice() else {
        panic!("표 소유");
    };
    assert!(source_table.common.treat_as_char);
    let tree = core
        .build_page_render_tree(page)
        .expect("본문 쪽 render tree");
    let heading = line(&tree.root, title);
    let before = line(&tree.root, preceding);
    let object = table(&tree.root, owner);
    assert!(
        heading.y < before.y && before.y + before.height <= object.y,
        "제목 → 앞 본문 → 표의 소유 순서/비겹침"
    );
    // 원본 저장 줄은 pen, Table bbox는 위 바깥여백을 소비한 테두리다.
    let source_gap = i64::from(host_source.vertical_pos) - i64::from(previous_source.vertical_pos)
        + i64::from(source_table.outer_margin_top);
    equal_ratio(
        (object.y - before.y) / before.height,
        source_gap as f64 / f64::from(previous_source.line_height),
        "앞 글줄과 표가 같은 저장 여백 축을 소비해야 함",
    );
    if let Some(following) = following {
        let after = line(&tree.root, following);
        assert!(
            after.y >= object.y + object.height,
            "표 뒤 문단이 표 밴드 뒤에 보존돼야 함"
        );
        let source = &paragraphs[following].line_segs[0];
        let delta = i64::from(source.vertical_pos)
            - i64::from(host_source.vertical_pos)
            - i64::from(source_table.outer_margin_top);
        equal_ratio(
            (after.y - object.y) / object.height,
            delta as f64 / f64::from(source_table.common.height),
            "표와 후속 본문의 원점 관계",
        );
    }
}

#[test]
fn issue_8160_stored_plain_column_margin_preserves_successor_table_axis() {
    let core = load();
    assert!(core.document().sections[0].paragraphs[488]
        .controls
        .is_empty());
    assert_saved_table_axis(&core, 25, 488, 489, 490, None);
}

#[test]
fn issue_8160_stored_bookmark_column_margin_preserves_table_and_following_body_axis() {
    let core = load();
    assert!(
        !core.document().sections[0].paragraphs[162]
            .controls
            .is_empty(),
        "책갈피가 있는 첫 글줄"
    );
    assert_saved_table_axis(&core, 6, 162, 167, 168, Some(170));
}
