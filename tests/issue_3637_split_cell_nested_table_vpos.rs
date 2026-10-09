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

    // 8쪽 첫 빈 줄도 공간을 소유한다. 다음 표의 바깥 여백은 그 뒤에 붙는다.
    let Control::Table(root) = &source.sections[0].paragraphs[5].controls[0] else {
        panic!("본문을 소유한 바깥 표");
    };
    let empty = &root.cells[1].paragraphs[135];
    assert!(empty.text.is_empty() && empty.controls.is_empty());
    let Control::Table(next) = &root.cells[1].paragraphs[136].controls[0] else {
        panic!("빈 줄 뒤 포트폴리오 표");
    };
    let tree = doc.build_page_render_tree(7).expect("8쪽 실제 배치");
    let mut all = Vec::new();
    nodes(&tree.root, &mut all);
    let blank_line = all.iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(135))
    }).expect("8쪽 앞 빈 줄의 실제 점유");
    let table = all.iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(136))
    }).expect("같은 8쪽의 후속 표");
    let source_gap = rhwp::renderer::hwpunit_to_px(
        empty.line_segs[0].line_spacing + i32::from(next.outer_margin_top),
        rhwp::renderer::DEFAULT_DPI,
    );
    let preceding_bottom = blank_line.bbox.y + blank_line.bbox.height + source_gap;
    assert!(
        table.bbox.y >= preceding_bottom - f64::EPSILON * preceding_bottom.abs(),
        "빈 줄의 끝·원본 뒤 간격·표 위 여백을 소비한 뒤에 표가 있어야 합니다: {:?}, {:?}",
        blank_line.bbox,
        table.bbox
    );
}
