use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SEP2020_SAMPLE: &str = "samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp";

fn load_doc() -> HwpDocument {
    let bytes = std::fs::read(SEP2020_SAMPLE).expect("sep2020 sample");
    HwpDocument::from_bytes(&bytes).expect("parse sep2020 sample")
}

fn min_para_text_line_bbox(node: &RenderNode, para_index: usize) -> Option<BoundingBox> {
    let own = match &node.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => {
            Some(node.bbox.clone())
        }
        _ => None,
    };
    own.into_iter()
        .chain(
            node.children
                .iter()
                .filter_map(|child| min_para_text_line_bbox(child, para_index)),
        )
        .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
}

fn max_para_text_line_bottom(node: &RenderNode, para_index: usize) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => {
            Some(node.bbox.y + node.bbox.height)
        }
        _ => None,
    };
    own.into_iter()
        .chain(
            node.children
                .iter()
                .filter_map(|child| max_para_text_line_bottom(child, para_index)),
        )
        .max_by(|a, b| a.partial_cmp(b).unwrap())
}

#[test]
fn issue_1375_sep2020_page17_rewind_paragraph_advances_whole_to_right_column() {
    let doc = load_doc();
    let page17 = doc.dump_page_items(Some(16));

    assert!(
        !page17.contains("PartialParagraph[미주]  pi=894"),
        "pi=894 rewind paragraph must not be split into the nearly-full left column\n{page17}"
    );

    let right_col = page17.find("  단 1").expect("page 17 right column dump");
    let para894 = page17
        .find("FullParagraph[미주]  pi=894")
        .expect("pi=894 whole paragraph on page 17");
    assert!(
        para894 > right_col,
        "pi=894 should start as a whole paragraph in the right column\n{page17}"
    );

    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let first_line = min_para_text_line_bbox(&tree.root, 894).expect("pi=894 first text line");
    assert!(
        first_line.x > 390.0 && (84.0..=110.0).contains(&first_line.y),
        "pi=894 should render at the right-column top band, got {:?}",
        first_line
    );
}

#[test]
fn issue_1375_sep2020_page22_rewind_tail_stays_inside_body_frame() {
    let doc = load_doc();
    let page22 = doc.dump_page_items(Some(21));
    let page23 = doc.dump_page_items(Some(22));

    assert!(
        page22.contains("PartialParagraph[미주]  pi=1175  lines=0..10"),
        "page 22 should keep only the render-safe head of pi=1175\n{page22}"
    );
    assert!(
        page23.contains("PartialParagraph[미주]  pi=1175  lines=10..13"),
        "page 23 should continue pi=1175 after the page 22 tail split\n{page23}"
    );

    let tree = doc.build_page_render_tree(21).expect("page 22 render tree");
    let tail_bottom =
        max_para_text_line_bottom(&tree.root, 1175).expect("pi=1175 page 22 text bottom");
    assert!(
        tail_bottom <= 1092.3,
        "pi=1175 page 22 tail should render inside the body frame, bottom={tail_bottom}"
    );
}
