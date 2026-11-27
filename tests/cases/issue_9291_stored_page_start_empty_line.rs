//! 관계형·고정형 — [#9291] 빈 줄 감추기 구역에서도 저장 사다리가 쪽 첫 빈 줄의 자리를 남기면 그 줄을 지우지 않는다.
//! 주석 갱신: 2026-11-23T22:58:04+0900
//!
//! 입력: 통계청 공개 보도자료(「2012년 6월 소비자물가동향」, HWP, `lastSavedWith.product = null`,
//! version 6.0.5.771, 18쪽) `samples/issue9291/consumer-price-trend-page-start-empty-line.hwp`.
//! 기준: 한컴 Print PDF `pdf/issue9291/consumer-price-trend-page-start-empty-line-2024.pdf`(18쪽).
//!
//! 구역 정의는 빈 줄 감추기(SectionDef bit 19)를 켠다. 12쪽은 빈 문단 pi=164(저장 vpos 0,
//! 줄 높이 1400, 줄간격 −280)로 시작하고 다음 문단 pi=165(「2. 소비자물가 추이 (2-2)」)의 첫 저장 줄
//! vpos 1120 = 0 + 1400 − 280 이다. 18쪽도 빈 문단 pi=210(vpos 0, 1800, 1080) 뒤 pi=211
//! (「2012년 소비자물가 동향 공표일정」) vpos 2880 = 0 + 1800 + 1080 이다. 두 문단 모두 앞뒤 간격은 0 이다.
//! 한/글은 쪽 첫 빈 줄을 지웠다면 다음 줄을 0 에 저장했을 것이므로, 이 사다리는 빈 줄이 자리를
//! 차지했다는 저장 증거다. 수정 전(upstream/devel `17b17c934b`)에는 사다리를 보지 않고 빈 줄을 지워
//! 12쪽 본문이 14.93px, 18쪽 본문이 38.40px 위에 그려졌다. 앞 쪽 끝에 이미 감춘 빈 줄 둘이
//! 있는 경우(#9350 `has_two_suppressed_before`)와 달리 이 두 쪽은 그 조건에 해당하지 않는다.
//! - 고정형: 전체 쪽수 18(= PDF).
//! - 관계형: 12쪽·18쪽 제목 줄 윗변 − 본문 윗변 = 다음 문단 저장 vpos.
//! - 관계형 반례: pi=165 저장 vpos 를 빈 줄과 같은 0 으로 바꾸면(사다리가 빈 줄 자리를 남기지
//!   않음) 빈 줄은 감춰져 제목 줄 윗변 = 본문 윗변.
//!
//! 원 PR의 이력 측정(2026-11-23, 한컴 2022 인쇄본, base = upstream/devel `17b17c934b` 바이너리,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 맑은 고딕(Regular·Bold)·바탕 + `ttfs/opensource` —
//! 문서 본 글꼴 없는 환경):
//! - Native: 18쪽 / 18쪽, 12쪽 99.96(base 62.93), 18쪽 99.47(base 31.25), 전쪽 최저 90.36(base 31.25)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 18쪽 / 18쪽, 12쪽 99.93, 18쪽 100.00, 전쪽 최저 93.05(1쪽)
//! - 98% 미만 쪽(Native 1·3·9·16·17쪽, WASM 같은 쪽)은 base 와 값이 같고 이 수정이 바꾸지 않는다(10k A/B 의
//!   SVG 변경 쪽은 12·18쪽뿐).
//! - 증거: `mydocs/pr/assets/issue_9291/`(native·wasm·base 전쪽 TSV, 12쪽·18쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9291/consumer-price-trend-page-start-empty-line.hwp";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 18;
/// (0 기반 쪽, 쪽을 여는 빈 문단, 그 뒤 제목 문단)
const CASES: [(u32, usize, usize); 2] = [(11, 164, 165), (17, 210, 211)];

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

/// 쪽의 제목 줄 윗변 − 본문 윗변(px).
fn title_offset(core: &DocumentCore, page: u32, title: usize) -> f64 {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|_| panic!("{}쪽 렌더 트리", page + 1));
    let top = body_top(&tree.root).expect("본문 영역");
    let line =
        line_top(&tree.root, title).unwrap_or_else(|| panic!("{}쪽 pi={title} 첫 줄", page + 1));
    line - top
}

#[test]
fn page_start_empty_line_kept_by_the_stored_ladder_holds_its_place() {
    let core = load();
    assert_eq!(core.page_count(), PDF_PAGES, "한/글 Print 와 같은 쪽수");
    let paragraphs = &core.document().sections[0].paragraphs;
    for (page, empty, title) in CASES {
        let empty_para = &paragraphs[empty];
        assert!(
            empty_para.text.is_empty() && empty_para.line_segs.len() == 1,
            "pi={empty} 는 한 줄 빈 문단"
        );
        let seg = &empty_para.line_segs[0];
        assert_eq!(seg.vertical_pos, 0, "pi={empty} 는 쪽 첫 저장 줄");
        let stored = paragraphs[title].line_segs[0].vertical_pos;
        assert_eq!(
            stored,
            seg.vertical_pos + seg.line_height + seg.line_spacing,
            "pi={title} 저장 vpos 는 빈 줄 자리를 남긴 사다리"
        );
        let offset = title_offset(&core, page, title);
        assert!(
            (offset - px(stored)).abs() < 0.5,
            "{}쪽 pi={title} 줄 윗변 − 본문 윗변 {offset:.2}px 은 저장 vpos {:.2}px 이어야 한다",
            page + 1,
            px(stored)
        );
    }
}

#[test]
fn page_start_empty_line_without_a_stored_place_is_still_hidden() {
    let (page, empty, title) = CASES[0];
    let mut core = load();
    {
        let paragraphs = &mut core.document_mut().sections[0].paragraphs;
        let empty_vpos = paragraphs[empty].line_segs[0].vertical_pos;
        // 사다리가 빈 줄 자리를 남기지 않은 저장본: 다음 문단이 빈 줄과 같은 vpos 에서 시작한다.
        paragraphs[title].line_segs[0].vertical_pos = empty_vpos;
    }
    let document = core.document().clone();
    core.set_document(document);
    let offset = title_offset(&core, page, title);
    assert!(
        offset.abs() < 0.5,
        "감춘 빈 줄 뒤 pi={title} 줄 윗변은 본문 윗변이어야 한다(차이 {offset:.2}px)"
    );
}
