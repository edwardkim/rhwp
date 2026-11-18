//! 관계형·고정형 — [#8925] 쪽나누기로 새 쪽을 여는 저장 줄 없는 TAC 표 문단 뒤 간격 전량.
//! 주석 갱신: 2026-11-11 09:30 KST (새 파일)
//!
//! 입력: 공개 결재문서 36415413(한컴오피스 2020 저장) 원본. 2쪽 첫 문단 `pi=3` 은 쪽나누기가 있는
//! 빈 문단으로 글자처럼 취급 제목 표(lh 8574 = 표 8008 + 바깥 위아래 여백 283·283, ls 1360) 하나를
//! 가진다. 원본에 저장 줄이 없어 로드 때 줄을 합성하며 그 vpos 는 앞 쪽 누적값(66142)이다.
//! 다음 원본 문단 `pi=4`(상자 표)의 저장 vpos 9934 = 새 쪽 영점 0 + lh 8574 + ls 1360 이므로
//! 한/글은 제목 표 줄 뒤 간격을 전량 남겼다.
//!
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8925/36415413-page-break-tac-full-gap-2020.pdf`(3쪽).
//! - 고정형: 전체 쪽수 3.
//! - 관계형: 2쪽 상자 표 위끝 − 제목 표 위끝 = `pi=4` 저장 vpos(두 표의 바깥 위 여백은 같다).
//!   두 표와 `pi=5` 는 2쪽에 이 순서로 있고 겹치지 않는다.
//!
//! 측정(2026-11-11, 코드 head 사슬 #8921 위(base devel e118de8aba),
//! `scripts/visual_sweep.py --silhouette-only --dpi 96`, raw = `silhouette_raw_match_percent`,
//! 문서 face 만 모은 글꼴 폴더를 `RHWP_FONT_PATH`·`--font-path` 로 공급, H2mjsM 은 미보유):
//! - Native: 3/3쪽, 1쪽 100.00 · 2쪽 99.52 · 3쪽 95.62, 전쪽 최저 95.62.
//! - fresh WASM(같은 head, wasm sha256 6df776c1…): 3/3쪽, 1쪽 100.00 · 2쪽 99.49 · 3쪽 95.45,
//!   전쪽 최저 95.45.
//! - 수정 전 Native(같은 사슬 끝 ba94215e1b): 2쪽 65.30(상자 표 위끝 223.2px, 한컴 232.2px).
//!   수정 전 fresh WASM(wasm sha256 dfc9ecb6…): 1쪽 100.00 · 2쪽 65.43 · 3쪽 95.45.
//! - 증거: `mydocs/pr/assets/issue_8925/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8925/36415413-page-break-tac-full-gap.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn document() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 쪽의 본문 표(문단 번호, 위끝, 아래끝)를 위에서부터 돌려준다.
fn body_tables(core: &DocumentCore, page: u32) -> Vec<(usize, f64, f64)> {
    let tree = core.build_page_render_tree(page).expect("쪽");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let mut tables: Vec<(usize, f64, f64)> = nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Table(table) if table.section_index == Some(0) => table
                .para_index
                .map(|para| (para, node.bbox.y, node.bbox.y + node.bbox.height)),
            _ => None,
        })
        .collect();
    tables.sort_by(|a, b| a.1.total_cmp(&b.1));
    tables
}

/// 쪽의 본문(표 칸 밖) 글줄 중 주어진 문단의 첫 위끝.
fn body_line_top(core: &DocumentCore, page: u32, para: usize) -> Option<f64> {
    let tree = core.build_page_render_tree(page).expect("쪽");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.section_index == Some(0)
                    && run.cell_context.is_none()
                    && run.para_index == Some(para) =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .min_by(|a, b| a.total_cmp(b))
}

#[test]
fn table_line_opening_page_by_break_keeps_full_trailing_spacing() {
    let core = document();
    assert_eq!(core.page_count(), 3, "한컴 engine 2020 Print PDF 는 3쪽");

    let paragraphs = &core.document().sections[0].paragraphs;
    let next_vpos = paragraphs[4].line_segs[0].vertical_pos;
    let title = &paragraphs[3].line_segs[0];
    assert_eq!(
        next_vpos,
        title.line_height + title.line_spacing,
        "원본 전제: pi=4 저장 vpos 는 새 쪽 영점에서 제목 표 줄 lh + ls 뒤"
    );

    let tables = body_tables(&core, 1);
    let find = |para: usize| {
        tables
            .iter()
            .find(|(p, _, _)| *p == para)
            .copied()
            .unwrap_or_else(|| panic!("2쪽 pi={para} 표: {tables:?}"))
    };
    let (_, title_top, title_bottom) = find(3);
    let (_, box_top, box_bottom) = find(4);
    assert_eq!(
        tables.first().map(|t| t.0),
        Some(3),
        "2쪽은 제목 표로 시작한다"
    );
    assert!(title_bottom <= box_top, "제목 표와 상자 표는 겹치지 않는다");
    let gap = box_top - title_top;
    assert!(
        (gap - hu_to_px(next_vpos)).abs() <= TOLERANCE_PX,
        "상자 표 위끝 − 제목 표 위끝 {gap:.2}px 는 저장 vpos {:.2}px 와 같아야 한다(간격 전량)",
        hu_to_px(next_vpos)
    );

    let heading_top = body_line_top(&core, 1, 5).expect("2쪽 pi=5 글줄");
    assert!(
        box_bottom <= heading_top,
        "pi=5 「추진개요」는 상자 표 아래에서 시작한다"
    );
}
