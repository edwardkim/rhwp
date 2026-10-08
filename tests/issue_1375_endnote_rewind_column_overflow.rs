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
fn issue_1375_sep2020_page17_rewind_paragraph_splits_at_stored_rewind() {
    // [#6574] 기준 PDF(`pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf` Hwp 2024
    // 13.0.0.3622 · `-hwpx-2024.pdf`)는 pi=894 의 앞 두 줄('(ⅰ) …가 포함되는 경우' 1050.8px,
    // '두 집합 …에서 한 원소씩을 택' 1068.7px)을 17쪽 왼쪽 단 하단에 두고 셋째 줄부터 오른쪽
    // 단 맨 위에서 잇는다. 저장 사다리도 셋째 줄에서 되감긴다(789203 → 770559). 종전 기대
    // (문단 통째로 오른쪽 단)는 이 PDF 와 다르다.
    let doc = load_doc();
    let page17 = doc.dump_page_items(Some(16));

    let right_col = page17.find("  단 1").expect("page 17 right column dump");
    let head = page17
        .find("PartialParagraph[미주]  pi=894  lines=0..2")
        .expect("pi=894 head on page 17 left column");
    let tail = page17
        .find("PartialParagraph[미주]  pi=894  lines=2..5")
        .expect("pi=894 tail on page 17 right column");
    assert!(
        head < right_col && right_col < tail,
        "pi=894 should keep its first two lines in the left column and continue in the right column\n{page17}"
    );

    // 문단이 두 단에 걸치면 쪽에서 가장 위 줄은 오른쪽 단 맨 위 줄이다. 왼쪽 단의 첫 줄을 따로 잰다.
    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let first_line = min_para_text_line_bbox_in_x_range(&tree.root, 894, 0.0, 395.0)
        .expect("pi=894 first text line in the left column");
    assert!(
        (1043.0..=1059.0).contains(&first_line.y),
        "pi=894 first line should sit at the left-column bottom like the PDF (1050.8px), got {:?}",
        first_line
    );
}

fn min_para_text_line_bbox_in_x_range(
    node: &RenderNode,
    para_index: usize,
    x_min: f64,
    x_max: f64,
) -> Option<BoundingBox> {
    let own = match &node.node_type {
        RenderNodeType::TextLine(line)
            if line.para_index == Some(para_index)
                && node.bbox.x >= x_min
                && node.bbox.x < x_max =>
        {
            Some(node.bbox.clone())
        }
        _ => None,
    };
    own.into_iter()
        .chain(node.children.iter().filter_map(|child| {
            min_para_text_line_bbox_in_x_range(child, para_index, x_min, x_max)
        }))
        .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
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
