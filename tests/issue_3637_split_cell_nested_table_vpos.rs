//! #3637: 독립 한컴 Print 12쪽과 같은 문단·표 소속 및 빈 줄 점유를 검사한다.
//! 전쪽 Native/actual fresh 90% 이상 근거는 issue7445 resolution 증적에 있다.
//! 절대 쪽 좌표나 임의 픽셀 여유로 잠정 배치를 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue3637/press_release_split_cell_nested_table.hwpx";

fn nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        nodes(child, out);
    }
}

#[test]
fn nested_table_snap_stays_inside_the_split_cell() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("원본 보도자료");
    let source = rhwp::parse_document(&bytes).expect("원본 저장 정보");
    let doc = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(doc.page_count(), 12, "독립 한컴 Print와 같은 쪽 소속");

    let mut visible_runs = 0;
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("실제 페이지 tree");
        let mut all = Vec::new();
        nodes(&tree.root, &mut all);
        for node in all {
            if matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
            {
                visible_runs += 1;
                assert!(
                    node.bbox.y >= tree.root.bbox.y
                        && node.bbox.y + node.bbox.height
                            <= tree.root.bbox.y + tree.root.bbox.height,
                    "{}쪽 글줄이 실제 쪽 상자 밖으로 이송됐습니다: {:?}",
                    page + 1,
                    node.bbox
                );
            }
        }
    }
    assert!(visible_runs > 0, "본문을 실제로 검사해야 합니다");

    // 독립 Print에서 새 쪽을 시작하는 원본 빈 줄은 각각 한 번 공간을 소유한다.
    // 뒤 표의 바깥 여백은 빈 줄의 높이·뒤 간격을 소비한 다음 붙는다.
    let Control::Table(root) = &source.sections[0].paragraphs[5].controls[0] else {
        panic!("본문을 소유한 바깥 표");
    };
    for (page, empty_index, table_index) in [(2, 31, 32), (7, 135, 136), (10, 189, 190)] {
        let empty = &root.cells[1].paragraphs[empty_index];
        assert!(empty.text.is_empty() && empty.controls.is_empty());
        let Control::Table(next) = &root.cells[1].paragraphs[table_index].controls[0] else {
            panic!("빈 줄 뒤 원본 표");
        };
        let tree = doc
            .build_page_render_tree(page)
            .expect("대응 쪽의 실제 배치");
        let mut all = Vec::new();
        nodes(&tree.root, &mut all);
        let outer = all
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                if table.section_index == Some(0)
                    && table.para_index == Some(5)
                    && table.control_index == Some(0)
                    && table.cell_context.is_none())
            })
            .expect("같은 원본 바깥 표의 조각");
        let cell = outer
            .children
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::TableCell(cell)
                if cell.model_cell_index == Some(1))
            })
            .expect("본문 원본 셀");
        let mut owned = Vec::new();
        nodes(cell, &mut owned);
        let blanks: Vec<_> = owned
            .iter()
            .filter(|node| {
                node.visible
                    && !node.editor_only
                    && matches!(&node.node_type, RenderNodeType::TextLine(line)
                    if line.para_index == Some(empty_index))
            })
            .collect();
        assert_eq!(
            blanks.len(),
            1,
            "{}쪽 원본 빈 줄은 한 번 공간을 소유합니다",
            page + 1
        );
        let tables: Vec<_> = owned
            .iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(table_index)
                    && table.cell_context.as_ref().is_some_and(|context| {
                        context.parent_para_index == 5
                            && context.path.first().is_some_and(|entry| {
                                entry.control_index == 0
                                    && entry.cell_index == 1
                                    && entry.cell_para_index == table_index
                            })
                    }))
            })
            .collect();
        assert_eq!(
            tables.len(),
            1,
            "같은 쪽의 후속 표 소유는 중복되지 않습니다"
        );
        let blank = blanks[0];
        let table = tables[0];
        assert!(
            blank.bbox.y >= cell.bbox.y
                && blank.bbox.y + blank.bbox.height <= cell.bbox.y + cell.bbox.height,
            "빈 줄의 실제 점유는 자신의 셀 조각 안에 있습니다"
        );
        let source_gap = rhwp::renderer::hwpunit_to_px(
            empty.line_segs[0].line_spacing + i32::from(next.outer_margin_top),
            rhwp::renderer::DEFAULT_DPI,
        );
        let preceding_bottom = blank.bbox.y + blank.bbox.height + source_gap;
        assert!(
            table.bbox.y >= preceding_bottom - f64::EPSILON * preceding_bottom.abs(),
            "빈 줄의 끝·원본 뒤 간격·표 위 여백을 소비한 뒤에 표가 있어야 합니다: {:?}, {:?}",
            blank.bbox,
            table.bbox
        );
    }
}
