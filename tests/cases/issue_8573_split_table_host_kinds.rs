//! [#8573] 쪽을 넘기는 자리차지 표의 첫 조각은 host 글자 종류(공백·"X"·빈)와 관계없이 같다.
//!
//! 재현물(공개 샘플 변형, 저장 LINE_SEG 없음, 세 문서는 host 글자만 다르다):
//!
//! ```text
//! 문단 0: "Q"
//! 문단 1: host 글자(공백 7칸 / "X" / 없음) + 자리차지 표(문단 기준 세로 1100HU, 바깥 여백 141HU,
//!         70행 × 1282HU, 쪽 경계에서 행 단위로 나눔)
//! ```
//!
//! 웹한글기안기 인쇄본(`pdf/issue8573/*-host-split-table-webhwp-print.pdf`)은 세 문서 모두 1쪽에
//! 1~48행을 둔다(세로 오프셋을 950→1000HU 로 바꾸면 세 문서가 함께 49행→48행). 공백 host 만
//! 표 아래 바깥 여백을 조각 예산에서 빼지 않으면 한 행을 더 받아 다른 두 문서와 갈린다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const WHITESPACE: &str = "samples/issue8573/whitespace-host-split-table.hwp";
const TEXT: &str = "samples/issue8573/text-host-split-table.hwp";
const EMPTY: &str = "samples/issue8573/empty-host-split-table.hwp";

fn first_page_rows(sample: &str) -> BTreeSet<u16> {
    fn walk(n: &RenderNode, rows: &mut BTreeSet<u16>) {
        if let RenderNodeType::TableCell(cell) = &n.node_type {
            rows.insert(cell.row);
            return;
        }
        for c in &n.children {
            walk(c, rows);
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert!(core.page_count() >= 2, "{sample}: 표가 쪽을 넘긴다");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut rows = BTreeSet::new();
    walk(&tree.root, &mut rows);
    rows
}

#[test]
fn whitespace_host_first_fragment_matches_text_and_empty_hosts() {
    let ws = first_page_rows(WHITESPACE);
    let tx = first_page_rows(TEXT);
    let em = first_page_rows(EMPTY);
    assert!(!ws.is_empty(), "공백 host 1쪽에 표 행이 있다");
    assert_eq!(
        tx,
        em,
        "글자 host 와 빈 host 의 1쪽 행이 같다(대조군): {:?} / {:?}",
        tx.last(),
        em.last()
    );
    assert_eq!(
        ws,
        tx,
        "공백 host 1쪽 마지막 행({:?})은 글자 host({:?})와 같다",
        ws.last(),
        tx.last()
    );
}
