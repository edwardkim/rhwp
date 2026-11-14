//! 관계형·고정형 — [#9016] 글자 없는 문단의 제목 차례 표시 슬롯을 빼고 다중 TAC 표를 자기 저장 줄에 매핑한다.
//! 주석 갱신: 2026-11-14 12:30 KST (새 파일, 직전 파일 커밋 없음)
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
//! 측정(2026-11-14, 코드 커밋 475fc73f5e(base devel 5f78bb8d02),
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw = `silhouette_raw_match_percent`).
//! 글꼴 공급은 두 경로 같게 맑은 고딕(Regular·Bold)·바탕(Windows/Office 글꼴)과 저장소
//! `ttfs/opensource` 다. 표지의 HY울릉도M·HY헤드라인M·굴림체 등 문서 face 는 대체 글꼴이다.
//! - Native: 10/10쪽, 1쪽 93.46 · 9쪽 99.98 · 10쪽 99.62, 전쪽 최저 93.46(1쪽 표지).
//! - fresh WASM(wasm sha256 430504874f79c84c…): 10/10쪽, 1쪽 92.43 · 9쪽 99.95 · 10쪽 99.82,
//!   전쪽 최저 92.43(1쪽 표지).
//! - `ttfs/opensource` 만 공급하면 fresh WASM 1쪽이 89.91 이다(2026-11-13 측정). macOS Chrome 이
//!   대체 사다리의 Apple SD Gothic Neo 를 골라 Native 의 대체 글꼴과 갈린다. 1쪽 배치는 수정 전후 같다.
//! - 수정 전(devel 5f78bb8d02): 11쪽(한컴 10쪽)이라 쪽수 불일치.
//!
//! 증거: `mydocs/pr/assets/issue_9016/`(Native·fresh WASM TSV, 1·9·10쪽 review·overlay PNG).
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
