//! Issue #768: shortcut.hwp 페이지 3 끝 column-break 행이 페이지 4 첫 줄로 밀림
//!
//! `samples/basic/shortcut.hwp` 페이지 3 끝의 다단 영역에 있어야 할
//! pi=94 ("<편집 화면 분할에서>") + pi=95 ("화면 이동 Ctrl+W,N") 가
//! 페이지 4 의 첫 zone 으로 밀리는 결함.
//!
//! Root cause: typeset.rs:advance_column_or_new_page 가 다단 영역의 마지막
//! 단 (current_column+1 == col_count) 에서 column-break 만나면 무조건
//! push_new_page. PDF 권위(한글 2022) 는 같은 다단 영역에서 col 0 으로
//! wrap-around 하여 좌단 7행 / 우단 7행 으로 표시.
//!
//! 권위 자료: `pdf/basic/shortcut-2022.pdf` 페이지 3 끝.

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/basic/shortcut.hwp";
const TARGET_PI: usize = 94;
const PDF_AUTHORITY_PAGE_INDEX: u32 = 2; // 페이지 3 (0-based)

fn has_para(node: &RenderNode, target_pi: usize) -> bool {
    if let RenderNodeType::TextLine(tl) = &node.node_type {
        if tl.para_index == Some(target_pi) {
            return true;
        }
    }
    node.children.iter().any(|c| has_para(c, target_pi))
}

#[test]
fn issue_768_pi94_appears_on_page3_not_page4() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    let page_count = doc.page_count();
    let mut found_page: Option<u32> = None;
    for pn in 0..page_count {
        let tree = doc.build_page_render_tree(pn).expect("build_page_render_tree");
        if has_para(&tree.root, TARGET_PI) {
            found_page = Some(pn);
            break;
        }
    }

    let page_idx = found_page
        .unwrap_or_else(|| panic!("pi={} 가 어떤 페이지에도 등장하지 않음", TARGET_PI));

    eprintln!(
        "[issue_768] pi={} 등장 페이지 인덱스 = {} (page_count={}), PDF 권위 = {}",
        TARGET_PI, page_idx, page_count, PDF_AUTHORITY_PAGE_INDEX,
    );

    assert_eq!(
        page_idx, PDF_AUTHORITY_PAGE_INDEX,
        "pi={} 가 page_index={} 에 등장. PDF 권위(한글 2022) 정합 = {} (3쪽). \
         column-break 가 다단 영역 마지막 단에서 발생할 때 wrap-around 안 되고 \
         페이지 break 강제하는 결함.",
        TARGET_PI, page_idx, PDF_AUTHORITY_PAGE_INDEX,
    );
}
