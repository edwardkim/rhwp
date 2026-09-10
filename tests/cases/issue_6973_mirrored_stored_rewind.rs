//! [Issue #6973] 파생 문단 꼬리 확장이 **저장된 물리 쪽 경계**를 넘어 마지막 표 조각이
//! 본문 바닥을 108.9px 넘고 쪽 하나가 사라졌다 (`rowSpan` 없는 축).
//!
//! `83818` 실측 — `section3` 15행 × 2열 신·구조문대비표(`pageBreak="CELL"`), 행 13.
//!
//! ```text
//!   저장 lineseg (두 셀 모두 12줄)
//!     li 0..8   vertpos 0 → 2520 → … → 20160      ← 8쪽
//!     li 9      vertpos 0                          ← 되감김 = 물리 쪽 경계
//!     li 10..11 vertpos 2520 · 5040                ← 9쪽
//!
//!   보통 컷    pre_cut  = [8, 8]     되감김 한 줄 앞에서 멈춘다
//!   파생 확장  tail_cut = [12, 12]   되감김을 3줄 넘어 문단 끝까지 당긴다
//!   결과       잔여 299.8px 에 388.3px → 본문 +88.5px, 뒤 spacer 행 5.1px 포함 108.9px
//! ```
//!
//! ⭐⭐ **판정은 크기가 아니라 저장 증거의 일치다.** 가시 셀이 둘 이상이고 전부 **같은 줄
//! 인덱스**에서 되감기면 그 자리는 물리 경계다 — 독립된 두 셀이 같은 자리를 적었다는 것이
//! writer-local 커서와 갈리는 증거이고, 기존 `row_has_single_visible_source_cell`
//! (가시 셀 1개) 계약의 **정확한 여집합**이라 `#3930`·`#3931`·`#5584`·`#5801`·`#6025`·
//! `#6549`·`#6790` 핀은 이 분기에 들어오지 않는다.
//!
//! ⚠ 크기·비율 축으로는 갈리지 않는다 — 편람 r=4(정상)의 잔여 초과 `+81.8px` 와 이 문서
//! r=13(결함)의 `+88.5px` 는 6.7px 차이다. 잔여 기준 수용 판정(지정 시험 10/54 실패)·
//! 이월 분기(12/54 실패)·되감김 무조건 클립(편람 384→385)이 모두 그래서 기각됐다.
//!
//! ⚠ **남는 축** — 수정 후에도 8쪽에 17.7px 이 남는다. 한 쪽 앞 행 7 도 같은 형상
//! (li=5 되감김)인데 rhwp 의 7쪽 예산이 5줄(168.0px)에 5.6px 부족해 4/4 로 끊고, 그 한 줄
//! (33.6px)이 8쪽 잔여를 잡아먹는다. 예산 대 실측 어긋남(`#6923`·`#6976` 단계 3)의 몫이라
//! 이 시험은 쪽수와 초과 상한만 계약한다.
//!
//! 기준: 한/글 2020(저장 버전) **9쪽** — `pdf/83818-appraisal-rules-amendment-2020.pdf`
//! (`hwp2024Convert` engine 2020, `pdf_page_count=9`)와 한/글 2020 COM 두 경로가 일치한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parse_document;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6973/83818-appraisal-rules-amendment.hwpx";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6973 정식 HWPX fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

fn worst_table_overflow(node: &RenderNode, bottom: f64, out: &mut f64) {
    if matches!(node.node_type, RenderNodeType::Table { .. }) {
        *out = out.max(node.bbox.y + node.bbox.height - bottom);
    }
    for child in &node.children {
        worst_table_overflow(child, bottom, out);
    }
}

/// 한/글 2020 과 같은 9쪽이어야 한다 — 수정 전 8쪽.
#[test]
fn mirrored_stored_rewind_restores_the_lost_page() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        9,
        "한/글 2020 과 같은 9쪽이어야 한다 — #6973 회귀 (수정 전 8쪽)"
    );
}

/// 마지막 조각이 본문 바닥을 크게 넘으면 안 된다 — 수정 전 `+108.9px`.
///
/// 상한 30px 은 위 모듈 주석의 잔여 축(행 7 한 줄 33.6px 중 17.7px)을 담되 수정 전
/// 108.9px 은 확실히 거르는 자리다. 잔여가 닫히면 이 상한을 조인다.
#[test]
fn the_last_fragment_stops_at_the_stored_boundary() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(7).expect("8쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");
    let bottom = body.bbox.y + body.bbox.height;

    let mut over = 0.0f64;
    worst_table_overflow(body, bottom, &mut over);

    assert!(
        over < 30.0,
        "8쪽 표가 본문 바닥을 크게 넘으면 안 된다 — #6973 회귀 \
         (초과 {over:.1}px, 본문 하한 {bottom:.1}; 수정 전 +108.9px)"
    );
}

/// 정답지 자체를 고정한다 — 행 13 의 두 가시 셀이 **같은 줄 인덱스**에서 되감긴다.
///
/// 이 형상이 사라지면 위 두 계약은 다른 이유로 통과할 수 있다. 조판을 거치지 않고
/// 저장 데이터만 본다.
#[test]
fn both_cells_record_the_same_frame_rewind_line() {
    let bytes = sample();
    let document = parse_document(&bytes).expect("문서 파싱");
    let table = document
        .sections
        .iter()
        .flat_map(|section| section.paragraphs.iter())
        .flat_map(|paragraph| paragraph.controls.iter())
        .filter_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .find(|table| table.row_count == 15 && table.col_count == 2)
        .expect("15행 × 2열 신·구조문대비표");

    let rewinds: Vec<Option<usize>> = table
        .cells
        .iter()
        .filter(|cell| cell.row == 13 && cell.row_span == 1)
        .map(|cell| {
            let mut line_index = 0usize;
            let mut found = None;
            for paragraph in &cell.paragraphs {
                for (li, pair) in paragraph.line_segs.windows(2).enumerate() {
                    if found.is_none() && pair[0].vertical_pos > 0 && pair[1].vertical_pos == 0 {
                        found = Some(line_index + li + 1);
                    }
                }
                line_index += paragraph.line_segs.len();
            }
            found
        })
        .collect();

    assert_eq!(
        rewinds,
        vec![Some(9), Some(9)],
        "행 13 의 두 셀이 같은 줄(li=9)에서 되감겨야 한다 — #6973 정답지"
    );
}
