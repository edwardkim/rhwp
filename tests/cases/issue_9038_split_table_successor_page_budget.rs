//! 관계형·고정형 — [#9038] 같은 문단에서 쪽을 넘겨 나뉜 앞 표 뒤의 둘째 표는 앞 표 조각 끝에서 쪽 예산을 잰다.
//!
//! 주석 갱신: 2026-11-13 KST(+09:00), 새 파일.
//!
//! - 입력: 공개 문서 `samples/issue9038/sme_announcement_split_table_successor.hwpx`
//!   (중소벤처기업부 사업공고 첨부 HWPX, edwardkim/rhwp#9038). `pi=99` 는 글 없는 문단에 18행 표
//!   (어울림, 4쪽에 걸쳐 분할)와 「2. 장비의 범위」로 시작하는 9행 표(자리차지)를 함께 가진다.
//! - 기준: `pdf/issue9038/sme_announcement_split_table_successor-hancom-2018.pdf`
//!   (같은 게시글의 한컴 PDF, Hwp 2018 10.0.0.14515, Hancom PDF 1.3.0.550) 46쪽. 23쪽은 18행 표의
//!   끝 조각으로 끝나고 24쪽이 「2. 장비의 범위」로 시작한다.
//! - 수정 전 rhwp(devel `180a4ec972`)는 둘째 표의 쪽 예산을 첫 표가 시작한 쪽의 문단 시작 높이로
//!   계산해 23쪽 남은 약 21px 를 627px 로 보고 0~5행을 담았다. 23쪽 글이 용지 아래로 넘치고
//!   전체가 45쪽이 됐다.
//!
//! 검사: 전체 쪽수 = 한컴 46쪽, 23쪽에는 「2. 장비의 범위」가 없고 24쪽에 있으며, 23·24쪽 글이 모두
//! 쪽 상자 안에 놓인다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9038/sme_announcement_split_table_successor.hwpx";
/// 한컴 PDF 의 전체 쪽수.
const HANCOM_PAGES: u32 = 46;
const HEADING: &str = "2.장비의범위";

fn ink(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// (위끝, 아래끝, 글) 묶음을 모은다.
fn runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((
            node.bbox.y,
            node.bbox.y + node.bbox.height,
            run.text.clone(),
        ));
    }
    for child in &node.children {
        runs(child, out);
    }
}

fn page_runs(core: &DocumentCore, page: u32) -> (f64, Vec<(f64, f64, String)>) {
    let tree = core.build_page_render_tree(page).expect("쪽");
    let mut out = Vec::new();
    runs(&tree.root, &mut out);
    (tree.root.bbox.y + tree.root.bbox.height, out)
}

fn page_ink(all: &[(f64, f64, String)]) -> String {
    all.iter().map(|(_, _, text)| ink(text)).collect()
}

#[test]
fn successor_table_after_split_table_starts_on_next_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"));
    assert_eq!(core.page_count(), HANCOM_PAGES, "한컴 PDF 의 전체 쪽수");

    let (bottom_23, runs_23) = page_runs(&core, 22);
    let (bottom_24, runs_24) = page_runs(&core, 23);
    assert!(
        !page_ink(&runs_23).contains(HEADING),
        "23쪽은 앞 표의 끝 조각으로 끝나고 둘째 표를 담지 않는다"
    );
    assert!(
        page_ink(&runs_24).contains(HEADING),
        "24쪽이 둘째 표 「2. 장비의 범위」로 시작한다"
    );
    for (page, bottom, all) in [(23, bottom_23, &runs_23), (24, bottom_24, &runs_24)] {
        let outside: Vec<_> = all
            .iter()
            .filter(|(_, end, _)| *end > bottom + 0.5)
            .collect();
        assert!(
            outside.is_empty(),
            "{page}쪽 글이 쪽 상자 아래로 넘친다: {}개",
            outside.len()
        );
    }
}
