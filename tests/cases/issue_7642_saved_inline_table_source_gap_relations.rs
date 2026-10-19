//! 독립 Print와 Native/fresh WASM 전체 15쪽을 대조한 언어 원본이다.
//! Stage190의 원본 소유·내용 순서·저장 상대 간격을 검사한다.
//! 실제 화면 좌표와 픽셀 영역으로 대상을 선택하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

type OwnedNode<'a> = (&'a RenderNode, u16, BoundingBox);

fn visit<'a>(
    node: &'a RenderNode,
    column: Option<(u16, BoundingBox)>,
    out: &mut Vec<OwnedNode<'a>>,
) {
    let column = match node.node_type {
        RenderNodeType::Column(index) => Some((index, node.bbox)),
        _ => column,
    };
    if let Some((index, bbox)) = column {
        out.push((node, index, bbox));
    }
    for child in &node.children {
        visit(child, column, out);
    }
}

#[test]
fn issue7642_saved_inline_tables_follow_their_owned_text_and_source_gap() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21_언어_기출_편집가능본.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("검증한 원본 읽기"))
        .expect("언어 원본 파싱");
    assert_eq!(core.page_count(), 15, "독립 Print의 전체 쪽수");
    let paragraphs = &core.document().sections[0].paragraphs;
    for (page, text_owner, table_owner) in [(2, 36, 37), (3, 71, 72), (8, 166, 167), (11, 228, 229)]
    {
        let source = &paragraphs[text_owner];
        let last_index = source
            .line_segs
            .len()
            .checked_sub(1)
            .expect("앞 문단의 글줄");
        let last = &source.line_segs[last_index];
        let next = paragraphs[table_owner]
            .line_segs
            .first()
            .expect("표의 저장 글줄");
        let source_gap = next.vertical_pos - last.vertical_pos - last.line_height;
        assert!(
            source_gap > 0 && last.line_height > 0,
            "독립 Print가 보존한 상대 간격"
        );
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("실제 페이지 출력");
        let mut nodes = Vec::new();
        visit(&tree.root, None, &mut nodes);
        let lines: Vec<_> = nodes.iter().filter(|(node, _, _)| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.para_index == Some(text_owner) && line.line_index == Some(last_index as u32))
        }).collect();
        let tables: Vec<_> = nodes
            .iter()
            .filter(|(node, _, _)| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(table_owner) && table.cell_context.is_none())
            })
            .collect();
        assert_eq!(
            lines.len(),
            1,
            "앞 문단의 마지막 글줄 소유는 한 번만 방출한다"
        );
        assert_eq!(tables.len(), 1, "뒤 표 소유는 한 번만 방출한다");
        let (line, line_column, _) = *lines[0];
        let (table, table_column, column) = *tables[0];
        assert_eq!(
            line_column, table_column,
            "앞 문단과 뒤 표는 같은 단에 속한다"
        );
        let gap = table.bbox.y - (line.bbox.y + line.bbox.height);
        assert!(gap > 0.0, "앞 글줄 뒤에 표를 놓아 겹침을 만들지 않는다");
        assert!(
            table.bbox.y + table.bbox.height <= column.y + column.height,
            "표가 자신의 단 본문 안에 포함된다"
        );
        let actual_ratio = gap / line.bbox.height;
        let source_ratio = f64::from(source_gap) / f64::from(last.line_height);
        // 같은 물리 이동에서 생기는 부동소수점 반올림만 허용한다.
        let roundoff = (table.bbox.y.abs() + line.bbox.y.abs() + line.bbox.height)
            / line.bbox.height
            * f64::EPSILON
            * 256.0;
        assert!((actual_ratio - source_ratio).abs() <= roundoff,
            "문단 {text_owner} → 표 {table_owner}: 원본 간격 비율 {source_ratio}를 보존한다, 실제 {actual_ratio}");
    }
}
