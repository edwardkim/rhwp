//! 관계형·고정형 — [#8993] 저장 프레임이 본문을 넘는 RowBreak 표는 남은 칸에서 나뉘어 앞 쪽을 비우지 않는다.
//! 주석 갱신: 2026-11-12 17:20 KST (새 파일)
//!
//! 입력: 행정안전부령 「지방자치단체 회계기준에 관한 규칙」(부령 제124호, 2009.12.31, HWP, 19쪽)
//! `samples/issue8993/31675-rowbreak-overflowing-frame.hwp`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8993/31675-rowbreak-overflowing-frame-2020.pdf`(19쪽).
//! 14쪽은 빈 문단 넷 뒤에 [별지 제1호 서식] 재정상태보고서 표(52행×13열, 비-TAC 자리차지,
//! 쪽나눔 RowBreak, 선언 높이 791.6px)다. 이 표의 저장 프레임은 14쪽 130.1px 에서 시작해
//! 921.7px 로 본문(914.6px)을 넘는다 — 저장본도 그 자리에서 행을 나눴다. 한/글 PDF 는 14쪽에
//! 「Ⅱ. 장기차입부채」 묶음까지, 15쪽에 「Ⅲ. 기타비유동부채」부터 「부채와 순자산 총계」까지 둔다.
//! 수정 전 rhwp 는 저장 프레임을 통째 이월의 근거로 써 표 전체를 15쪽에 넘기고 14쪽을 비웠다.
//! - 고정형: 전체 쪽수 19(= PDF).
//! - 관계형: 14쪽은 표 머리(「별지 제1호 서식」)와 「장기차입부채」를 담고 「기타비유동부채」를
//!   담지 않는다. 15쪽은 「기타비유동부채」·「부채와 순자산 총계」를 담고 표 머리를 다시 담지 않는다.
//!
//! 측정(2026-11-12, 이 파일과 같은 커밋의 코드, base = upstream/devel 80b1c354c2,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 19쪽 / 19쪽, 14쪽 99.83 · 15쪽 99.80(base 0.00 · 14.63), 최저 96.44(13쪽, base 와 같음)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 19쪽 / 19쪽, 14쪽 99.85 · 15쪽 99.78, 최저 95.69(13쪽)
//! - 증거: `mydocs/pr/assets/issue_8993/`(native·wasm TSV, review·overlay PNG)
//! - 표 외곽·행 경계·쪽 나뉨 위치는 PDF 와 겹친다. 남은 차이는 공개 대체 글꼴의 글자 모양이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue8993/31675-rowbreak-overflowing-frame.hwp";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 19;
/// 표가 시작하는 쪽(0 기반, 14쪽).
const TABLE_PAGE: u32 = 13;

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
fn overflowing_saved_rowbreak_frame_splits_on_the_current_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let first = page_text(&core, TABLE_PAGE);
    let second = page_text(&core, TABLE_PAGE + 1);
    assert!(
        first.contains("별지제1호서식") && first.contains("장기차입부채"),
        "14쪽은 표 머리와 「Ⅱ. 장기차입부채」 묶음까지 담아야 한다: {first}"
    );
    assert!(
        !first.contains("기타비유동부채"),
        "「Ⅲ. 기타비유동부채」는 15쪽으로 넘어가야 한다"
    );
    assert!(
        second.contains("기타비유동부채") && second.contains("부채와순자산총계"),
        "15쪽은 「Ⅲ. 기타비유동부채」부터 표 끝까지 담아야 한다: {second}"
    );
    assert!(
        !second.contains("별지제1호서식"),
        "표 머리는 14쪽에 한 번만 있어야 한다"
    );
}
