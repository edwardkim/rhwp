//! 관계형·고정형 — [#9009] 앞 쪽에서 이어진 표 조각만 있는 단의 [단 나누기] 문단은 새 쪽을 열지 않는다.
//! 주석 갱신: 2026-11-12 20:10 KST (새 파일)
//!
//! 입력: 공개 연구보고서 「진도보고서(특허청) 산업재산권」(한국법제연구원, HWP, 8쪽)
//! `samples/issue9009/1430000-column-break-after-table-continuation.hwp`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue9009/1430000-column-break-after-table-continuation-2020.pdf`(8쪽).
//! pi=14 표(116행×3열, 비-TAC 자리차지)는 5쪽에서 시작해 7쪽까지 이어지며, 7쪽에는 그 끝 조각(65~116행)만
//! 있다. 뒤따르는 pi=15 는 [단 나누기] 빈 문단으로 저장 vpos 69148HU(921.97px)가 표 조각 끝(922.0px)
//! 바로 아래다. pi=16 「나. 연구의 진척도」는 [쪽 나누기]로 8쪽을 시작한다. 한/글 PDF 는 8쪽이다.
//! 수정 전 rhwp 는 pi=15 의 단 나누기로 새 쪽을 열어 8쪽에 빈 문단만 두고 9쪽이 됐다.
//! - 고정형: 전체 쪽수 8(= PDF).
//! - 관계형: 7쪽은 표 끝 행 「제60조(과태료)」를 담고, 8쪽은 「나. 연구의 진척도」로 시작하며 표 내용을
//!   담지 않는다. 모든 쪽에 글자가 있다(빈 쪽 없음).
//!
//! 측정(2026-11-12, 이 파일과 같은 커밋의 코드, base = upstream/devel 80b1c354c2,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 8쪽 / 8쪽, 8쪽 99.73(base 0.00, base 9쪽), 최저 MEASURE_NATIVE
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 8쪽 / 8쪽, 8쪽 MEASURE_WASM8, 최저 MEASURE_WASM
//! - 증거: `mydocs/pr/assets/issue_9009/`(native·wasm TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9009/1430000-column-break-after-table-continuation.hwp";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 8;
/// 표 끝 조각만 있는 쪽(0 기반, 7쪽).
const TABLE_TAIL_PAGE: u32 = 6;

fn page_text(core: &DocumentCore, page: u32) -> String {
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut text = String::new();
    walk(&tree.root, &mut text);
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn column_break_after_table_continuation_stays_below_the_fragment() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    for page in 0..core.page_count() {
        assert!(
            !page_text(&core, page).is_empty(),
            "{}쪽에 글자가 없다 — 단 나누기 빈 문단이 홀로 쪽을 차지했다",
            page + 1
        );
    }

    let tail = page_text(&core, TABLE_TAIL_PAGE);
    let next = page_text(&core, TABLE_TAIL_PAGE + 1);
    assert!(
        tail.contains("제60조(과태료)"),
        "7쪽은 표 끝 행 「제60조(과태료)」를 담아야 한다: {tail}"
    );
    assert!(
        next.starts_with("나.연구의진척도"),
        "8쪽은 「나. 연구의 진척도」로 시작해야 한다: {next}"
    );
    assert!(
        !next.contains("제60조(과태료)"),
        "표 내용이 8쪽으로 넘어가면 안 된다"
    );
}
