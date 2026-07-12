//! Issue #1695: HWP3 LINE_SEG vpos reset/rewind를 문단 내부 페이지 경계로 반영한다.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/SO-SUEOP.hwp";
const EXPECTED_PAGE_COUNT: u32 = 46;

fn load_doc() -> HwpDocument {
    let bytes = std::fs::read(SAMPLE).unwrap_or_else(|err| panic!("read {SAMPLE}: {err}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|err| panic!("parse {SAMPLE}: {err:?}"))
}

fn body_bottom(root: &RenderNode) -> Option<f64> {
    let own = match &root.node_type {
        RenderNodeType::Body { .. } => Some(root.bbox.y + root.bbox.height),
        _ => None,
    };
    own.or_else(|| root.children.iter().find_map(body_bottom))
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
fn issue_1695_so_sueop_hwp3_page_count_matches_pdf() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        EXPECTED_PAGE_COUNT,
        "SO-SUEOP.hwp는 기준 PDF와 같은 46페이지를 유지해야 한다"
    );
}

#[test]
fn issue_1695_internal_vpos_rewind_splits_at_page_bottom() {
    let doc = load_doc();
    let cases = [
        (
            5,
            6,
            179,
            "PartialParagraph  pi=179  lines=0..1",
            "PartialParagraph  pi=179  lines=1..2",
        ),
        (
            23,
            24,
            634,
            "PartialParagraph  pi=634  lines=0..3",
            "PartialParagraph  pi=634  lines=3..5",
        ),
        (
            28,
            29,
            748,
            "PartialParagraph  pi=748  lines=0..3",
            "PartialParagraph  pi=748  lines=3..6",
        ),
    ];

    for (page_index, next_page_index, para_index, head, tail) in cases {
        let page = doc.dump_page_items(Some(page_index));
        assert!(
            page.contains(head),
            "page {} must split paragraph {para_index} before the rewind line\n{page}",
            page_index + 1
        );

        let next_page = doc.dump_page_items(Some(next_page_index));
        assert!(
            next_page.contains(tail),
            "page {} must continue paragraph {para_index} from the rewind line\n{next_page}",
            next_page_index + 1
        );

        let tree = doc
            .build_page_render_tree(page_index)
            .unwrap_or_else(|err| panic!("render page {}: {err}", page_index + 1));
        let bottom = body_bottom(&tree.root).expect("body bottom");
        let para_bottom = max_para_text_line_bottom(&tree.root, para_index)
            .unwrap_or_else(|| panic!("paragraph {para_index} on page {}", page_index + 1));
        assert!(
            para_bottom <= bottom + 0.5,
            "paragraph {para_index} must stay inside body on page {}: {para_bottom:.1} > {bottom:.1}",
            page_index + 1
        );

        let next_tree = doc
            .build_page_render_tree(next_page_index)
            .unwrap_or_else(|err| panic!("render page {}: {err}", next_page_index + 1));
        let tail_first =
            min_para_text_line_bbox(&next_tree.root, para_index).unwrap_or_else(|| {
                panic!(
                    "paragraph {para_index} continuation on page {}",
                    next_page_index + 1
                )
            });
        assert!(
            tail_first.y < 220.0,
            "paragraph {para_index} continuation should restart near page top, got {:?}",
            tail_first
        );
    }
}
