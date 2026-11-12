//! 관계형·고정형 — [#9016] 글자 없는 문단의 제목 차례 표시 슬롯을 빼고 다중 TAC 표를 자기 저장 줄에 매핑한다.
//! 주석 갱신: 2026-11-13 04:00 KST (새 파일)
//!
//! 입력: 공개 과업수행계획서(새만금개발청, HWP, 10쪽) `samples/issue9016/1730000-title-mark-empty-stream.hwp`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue9016/1730000-title-mark-empty-stream-2020.pdf`(10쪽).
//! pi=168 은 글자 없는 문단으로 원시 슬롯이 [제목 차례 표시 0x0008 `titM`][18×29 표][1×3 「Ⅳ」 제목 표]
//! 순이고, 저장 줄은 제목 표시 줄(th 2000)·큰 표 줄(th 58262)·새 쪽 제목 표 줄(vpos 0, lh 3546)이다.
//! 수정 전 rhwp 는 제목 표시 슬롯을 빼지 않아 「Ⅳ」 제목 표를 큰 표 줄로 계상(776.8px)해 10쪽 예산표를
//! 11쪽으로 밀었고, 9쪽 큰 표를 제목 표시 줄 원점에 그렸다.
//! - 고정형: 전체 쪽수 10(= PDF).
//! - 관계형: 9쪽은 「과업 추진일정」 표를 담고 「과업 예산계획」을 담지 않는다. 10쪽은 「과업 예산계획」과
//!   예산표 끝 「최종보고」를 함께 담는다.
//!
//! 측정(2026-11-13, 이 파일과 같은 커밋의 코드, base = upstream/devel 80b1c354c2,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 10쪽 / 10쪽, 9쪽 99.98 · 10쪽 90.95(base 69.21 · 8.23, base 11쪽), 최저 90.95(10쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): MEASURE_WASM
//! - 증거: `mydocs/pr/assets/issue_9016/`(native·wasm TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9016/1730000-title-mark-empty-stream.hwp";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 10;
/// 「Ⅲ 과업 추진일정」 큰 표가 있는 쪽(0 기반, 9쪽).
const SCHEDULE_PAGE: u32 = 8;

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
fn title_mark_slot_keeps_multi_tac_tables_on_their_stored_lines() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let schedule = page_text(&core, SCHEDULE_PAGE);
    let budget = page_text(&core, SCHEDULE_PAGE + 1);
    assert!(
        schedule.contains("과업추진일정") && schedule.contains("착수보고회"),
        "9쪽은 「과업 추진일정」 표를 담아야 한다: {schedule}"
    );
    assert!(
        !schedule.contains("과업예산계획"),
        "「Ⅳ 과업 예산계획」 제목 표는 10쪽에 있어야 한다"
    );
    assert!(
        budget.contains("과업예산계획") && budget.contains("최종보고"),
        "10쪽은 「Ⅳ 과업 예산계획」 제목 표와 예산표를 함께 담아야 한다: {budget}"
    );
}
