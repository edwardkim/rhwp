//! 관계형·고정형 — [#9291] 빈 줄 감추기 구역에서도 저장 사다리가 쪽 첫 빈 줄의 자리를 남기면 그 줄을 지우지 않는다.
//! 주석 갱신: 2026-11-22 07:48 KST (새 파일)
//!
//! 입력: 부산광역시 금정구 공개 과업지시서(「금정구청사 중장기 기본구상 용역」, HWP,
//! `lastSavedWith.product = hancom-office-2018`, 14쪽) `samples/issue9291/geumjeong-task-order-page-start-empty-line.hwp`.
//! 기준: 한컴 Print PDF `pdf/issue9291/geumjeong-task-order-page-start-empty-line-2020.pdf`(14쪽).
//!
//! 구역 정의는 빈 줄 감추기(SectionDef bit 19)를 켠다. 2쪽은 빈 문단 pi=21(저장 vpos 0,
//! 줄 높이 1800, 줄간격 1080, 문단 뒤 간격 0)로 시작하고 다음 문단 pi=22(「목 차」)의 첫 저장 줄
//! vpos 2880 = 0 + 1800 + 1080 + 다음 문단 앞 간격 0 이다. 한/글은 쪽 첫 빈 줄을 지웠다면
//! 다음 줄을 0 에 저장했을 것이므로, 이 사다리는 빈 줄이 자리를 차지했다는 저장 증거다.
//! 수정 전에는 사다리를 보지 않고 빈 줄을 지워 2쪽 본문 전체가 38.4px 위에 그려졌다.
//! - 고정형: 전체 쪽수 14(= PDF).
//! - 관계형: 2쪽 「목 차」 줄 윗변 − 본문 윗변 = pi=22 저장 vpos.
//! - 관계형 반례: pi=22 저장 vpos 를 빈 줄과 같은 0 으로 바꾸면(사다리가 빈 줄 자리를 남기지
//!   않음) 빈 줄은 감춰져 「목 차」 줄 윗변 = 본문 윗변.
//!
//! 측정(2026-11-22, 이 파일과 같은 커밋의 코드, base = upstream/devel `d7b5286cbf` 바이너리
//! (수정 전 시험은 head 의 base `5665e53829` 에서 0.00px 로 실패),
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 14쪽 / 14쪽, 2쪽 97.68(base 28.19), 3쪽 98.15(base 12.66), 전쪽 최저 97.16(1쪽, base 와 같은 값)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 14쪽 / 14쪽, 2쪽 99.03, 3쪽 99.14, 전쪽 최저 94.32(1쪽)
//! - 1쪽·2쪽의 98% 미만은 글꼴 차이뿐이다: 1쪽 SVG 는 base 와 바이트가 같고(이 수정이 닿지 않음),
//!   PDF 1쪽 글꼴 H2hdrM·H2gtrE·Dotum 중 Dotum(gulim.ttc)을 공급해도 값이 같으며 overlay 에서 줄 위치가 일치한다.
//!   2쪽은 모든 줄 위치가 PDF 와 같고 글자 모양만 다르다(WASM 99.03).
//! - 증거: `mydocs/pr/assets/issue_9291/`(native·wasm·base 전쪽 TSV, 2쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9291/geumjeong-task-order-page-start-empty-line.hwp";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 14;
/// 2쪽을 여는 빈 문단.
const EMPTY: usize = 21;
/// 빈 문단 뒤 「목 차」 문단.
const TITLE: usize = 22;

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn px(hu: i32) -> f64 {
    rhwp::renderer::hwpunit_to_px(hu, rhwp::renderer::DEFAULT_DPI)
}

fn line_top(node: &RenderNode, para: usize) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.para_index == Some(para) && line.line_index == Some(0))
    {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(|child| line_top(child, para))
}

fn body_top(node: &RenderNode) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(body_top)
}

/// 2쪽 「목 차」 줄 윗변 − 본문 윗변(px).
fn title_offset(core: &DocumentCore) -> f64 {
    let tree = core.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let top = body_top(&tree.root).expect("본문 영역");
    let title = line_top(&tree.root, TITLE).expect("2쪽 「목 차」 줄");
    title - top
}

#[test]
fn page_start_empty_line_kept_by_the_stored_ladder_holds_its_place() {
    let core = load();
    assert_eq!(core.page_count(), PDF_PAGES, "한/글 Print 와 같은 쪽수");
    let paragraphs = &core.document().sections[0].paragraphs;
    let empty = &paragraphs[EMPTY];
    let title = &paragraphs[TITLE];
    assert!(
        empty.text.is_empty() && empty.line_segs.len() == 1,
        "pi=21 은 한 줄 빈 문단"
    );
    let seg = &empty.line_segs[0];
    let stored = title.line_segs[0].vertical_pos;
    assert_eq!(
        stored,
        seg.vertical_pos + seg.line_height + seg.line_spacing,
        "다음 문단 저장 vpos 는 빈 줄 자리를 남긴 사다리"
    );
    let offset = title_offset(&core);
    assert!(
        (offset - px(stored)).abs() < 0.5,
        "「목 차」 줄 윗변 − 본문 윗변 {offset:.2}px 은 저장 vpos {:.2}px 이어야 한다",
        px(stored)
    );
}

#[test]
fn page_start_empty_line_without_a_stored_place_is_still_hidden() {
    let mut core = load();
    {
        let paragraphs = &mut core.document_mut().sections[0].paragraphs;
        let empty_vpos = paragraphs[EMPTY].line_segs[0].vertical_pos;
        // 사다리가 빈 줄 자리를 남기지 않은 저장본: 다음 문단이 빈 줄과 같은 vpos 에서 시작한다.
        paragraphs[TITLE].line_segs[0].vertical_pos = empty_vpos;
    }
    let document = core.document().clone();
    core.set_document(document);
    let offset = title_offset(&core);
    assert!(
        offset.abs() < 0.5,
        "감춘 빈 줄 뒤 「목 차」 줄 윗변은 본문 윗변이어야 한다(차이 {offset:.2}px)"
    );
}
