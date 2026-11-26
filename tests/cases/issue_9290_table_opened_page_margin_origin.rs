//! 관계형·고정형 — [#9290] 쪽을 표로 열어도 첫 저장 줄 vpos 가 문단 앞 간격이고 간격 사다리가 닫히면 쪽 저장 원점은 0 이다.
//! 주석 갱신: 2026-11-22 09:40 KST (새 파일)
//!
//! 입력: 고용노동부 보도자료 공개 문서(「2025 우리 노동부 타운홀 미팅 개최」, HWPX,
//! `appVersion="12, 0, 0, 4204"`, 3쪽) `samples/issue9290/labor-townhall-table-opened-page.hwpx`.
//! 기준: 한컴 Print PDF `pdf/issue9290/labor-townhall-table-opened-page-2020.pdf`(3쪽).
//!
//! 3쪽은 글 없는 host 문단(pi=25, 문단 앞 간격 1500HU)의 문단 기준 자리차지 표(1×3, 「붙임」)로
//! 시작한다. host 저장 줄 vpos 1500 = 문단 앞 간격, 다음 문단 첫 줄 vpos 5400 = 1500 + 줄 높이
//! 1500 + 줄간격 900 + 다음 문단 앞 간격 1500 이므로 저장 쪽 원점은 0 이다. 수정 전에는 표로 여는
//! 쪽의 원점 조회가 이 규칙을 보지 않고 첫 줄 vpos 1500 을 원점으로 써서, 표 아래 본문이 모두
//! 20px 위에 그려졌다(한/글 Print 「일시」 줄은 저장 vpos 그대로의 자리).
//! - 고정형: 전체 쪽수 3(= PDF).
//! - 관계형: 3쪽 표 아래 저장 본문 줄(pi=26..=33)의 윗변 − 본문 윗변 = 각 줄의 저장 vpos.
//! - 관계형: 표 윗변 − 본문 윗변 = 표의 바깥 위 여백(원점 0, 세로 오프셋 0).
//!
//! 측정(2026-11-22, 이 파일과 같은 커밋의 코드 — upstream/devel `5665e53829` 위, base 수치는
//! 수정 전 upstream/devel `d7b5286cbf`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 3쪽 / 3쪽, 3쪽 96.19(base 30.75), 전쪽 최저 96.19(3쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 3쪽 / 3쪽, 3쪽 96.69, 전쪽 최저 96.69(3쪽)
//! - 3쪽 98% 미만은 글꼴 차이만 남은 쪽이다(PDF 글꼴 H2hdrM·HYwulM·한컴바탕·HCR돋움·맑은 고딕·Type3
//!   HFT, 표 칸 글머리 ⦁ U+2981 자형). 줄·표 괘선 위치는 PDF 와 겹친다(review·overlay 확대 판독).
//! - 증거: `mydocs/pr/assets/issue_9290/`(native·wasm·base 전쪽 TSV, 3쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9290/labor-townhall-table-opened-page.hwpx";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 3;
/// 3쪽을 여는 표의 host 문단.
const HOST: usize = 25;
/// 표 아래 저장 본문 줄을 가진 문단(다음 표 host 전까지).
const BODY: std::ops::RangeInclusive<usize> = 26..=33;

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn px(hu: i32) -> f64 {
    rhwp::renderer::hwpunit_to_px(hu, rhwp::renderer::DEFAULT_DPI)
}

fn body_line_top(node: &RenderNode, para: usize) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.para_index == Some(para) && line.line_index == Some(0))
    {
        return Some(node.bbox.y);
    }
    node.children
        .iter()
        .find_map(|child| body_line_top(child, para))
}

fn host_table_top(node: &RenderNode) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(HOST) && table.cell_context.is_none())
    {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(host_table_top)
}

fn body_top(node: &RenderNode) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(body_top)
}

#[test]
fn body_lines_below_the_opening_table_keep_their_stored_page_positions() {
    let core = load();
    assert_eq!(core.page_count(), PDF_PAGES, "한/글 Print 와 같은 쪽수");
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;
    let host = &paragraphs[HOST];
    let Some(Control::Table(table)) = host.controls.first() else {
        panic!("3쪽을 여는 문단은 자리차지 표 host");
    };
    assert!(!table.common.treat_as_char && table.common.vertical_offset == 0);
    let shape = &doc.doc_info.para_shapes[host.para_shape_id as usize];
    // HWP ParaShape 간격은 실제 HWPUNIT 의 두 배로 저장된다.
    assert_eq!(
        host.line_segs[0].vertical_pos,
        shape.spacing_before / 2,
        "host 첫 저장 줄 vpos = 문단 앞 간격"
    );

    let tree = core.build_page_render_tree(2).expect("3쪽 렌더 트리");
    let top = body_top(&tree.root).expect("본문 영역");
    let table_top = host_table_top(&tree.root).expect("3쪽 여는 표");
    let margin = px(i32::from(table.outer_margin_top));
    assert!(
        (table_top - top - margin).abs() < 0.5,
        "표 윗변 {table_top:.2}px 은 본문 위 {top:.2}px + 바깥 위 여백 {margin:.2}px"
    );
    for para in BODY {
        let stored = px(paragraphs[para].line_segs[0].vertical_pos);
        let line = body_line_top(&tree.root, para).expect("3쪽 본문 줄");
        assert!(
            (line - top - stored).abs() < 0.5,
            "pi={para}: 줄 윗변 {line:.2}px − 본문 위 {top:.2}px 은 저장 vpos {stored:.2}px 이어야 한다"
        );
    }
}
