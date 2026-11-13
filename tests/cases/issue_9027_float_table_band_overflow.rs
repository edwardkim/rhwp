//! [#9027] 관계형·고정형 회귀 — 앞 어울림 표 띠 아래로 밀려 쪽을 넘는 자리차지 표는 표만 다음 쪽으로 가고, 띠에 흡수된 줄 범위만 띠 바닥을 정한다.
//!
//! 주석 갱신: 2026-11-13 09:38 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue9027/1312000-201400081_2015_senior_policy_course.hwp`(인사혁신처 공개 문서
//!   「2015년 고위정책과정 교육운영 개요」, HWP5, 2쪽).
//! - 독립 기준: `pdf/issue9027/1312000-201400081_2015_senior_policy_course-hwp-2020.pdf`(한/글 2022
//!   Print, 로컬 오라클 engine 2020, 2쪽).
//! - 2쪽: 문단 20 의 어울림 표가 1쪽 69222HU 까지 차지해, 빈 문단 21 의 자리차지 제목 표(2165HU,
//!   바깥 여백 141·141, 쪽나눔 `RowBreak`, 칸 모두 1줄)는 그 띠 아래에서 쪽을 넘는다. 한/글은 제목 표만
//!   2쪽 맨 위에 두고(아래 괘선 125.0px), 문단 21 host 줄(저장 vpos 50958)과 빈 문단 22(53198)는 1쪽에
//!   남긴다. 문단 23 큰 표는 2쪽 저장 vpos 2447 = 141 + 2165 + 141 에서 시작한다(위 괘선 130.7px).
//!   수정 전 rhwp(devel `31a4225757`)는 host 줄·빈 문단까지 넘겨 큰 표가 40.7px 아래(171.4px)였다.
//! - 1쪽: 문단 15 의 빈 host 어울림 표 옆 차선에 문단 16 전체와 문단 17 첫 줄이 들어가고, 문단 17 둘째
//!   줄(저장 vpos 43278)은 전체 폭으로 이어진다. 수정 전 rhwp 는 문단 17 의 마지막 줄 끝까지 띠 바닥으로
//!   잡고 host 끝 줄간격을 다시 더해 그 줄부터 30.9px 아래였다.
//!
//! 측정: 2026-11-13, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣). 전체 2쪽 = 한/글 2쪽.
//! 수정 후(head 소스 `1983410f81`, devel `31a4225757` 기준) Native 1쪽 99.99%·2쪽 100.00%, fresh WASM 1쪽 99.99%·2쪽 100.00%
//! (전쪽 최저 Native 99.99%, fresh WASM 99.99%). 수정 전 Native 1쪽 74.42%·2쪽 44.93%.
//! 증거: `mydocs/pr/assets/issue_9027/{native,wasm,base-native}/`.
//!
//! 검사: 쪽수 2 = 한/글(고정형). 쪽 소속 — 제목 표는 2쪽, 문단 22 빈 줄은 1쪽(관계형). 2쪽 큰 표는 제목
//! 표 바로 아래(관계형), 1쪽 문단 17 둘째 줄·문단 18 은 저장 사다리 간격을 지킨다(관계형). 2쪽 제목 표
//! 아래·큰 표 위 괘선과 1쪽 문단 20 표 위 괘선 = 한/글 PDF ±2px(고정형, 양쪽 raw 98% 이상·확대 판독 완료).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9027/1312000-201400081_2015_senior_policy_course.hwp";
/// 한/글 Print PDF 의 괘선(96dpi px).
const PDF_P2_TITLE_BOTTOM: f64 = 125.0;
const PDF_P2_BIG_TABLE_TOP: f64 = 130.7;
const PDF_P1_PARA20_TABLE_TOP: f64 = 798.5;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn table_box(node: &RenderNode, para: usize) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.cell_context.is_none() {
            return Some((node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    node.children
        .iter()
        .find_map(|child| table_box(child, para))
}

/// 본문 문단 `para` 의 `line` 번째 줄 상자 위끝.
fn line_top(node: &RenderNode, para: usize, line: u32) -> Option<f64> {
    if let RenderNodeType::TextLine(text_line) = &node.node_type {
        if text_line.para_index == Some(para) && text_line.line_index == Some(line) {
            return Some(node.bbox.y);
        }
    }
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    node.children
        .iter()
        .find_map(|child| line_top(child, para, line))
}

#[test]
fn band_overflowing_float_table_moves_alone_and_band_ends_at_absorbed_lines() {
    let core = load();
    assert_eq!(core.page_count(), 2, "한/글 Print 와 같은 2쪽");

    let p1 = core.build_page_render_tree(0).expect("1쪽");
    let p2 = core.build_page_render_tree(1).expect("2쪽");

    // 쪽 소속: 제목 표는 2쪽만, 문단 22 빈 줄은 1쪽에 남는다.
    assert!(table_box(&p1.root, 21).is_none(), "제목 표는 1쪽에 없다");
    let (title_top, title_bottom) = table_box(&p2.root, 21).expect("2쪽 제목 표");
    assert!(
        line_top(&p1.root, 22, 0).is_some(),
        "문단 22 빈 줄은 1쪽에 남는다"
    );
    assert!(
        line_top(&p2.root, 22, 0).is_none(),
        "문단 22 빈 줄은 2쪽으로 넘어가지 않는다"
    );

    // 2쪽 큰 표는 제목 표 바로 아래에서 시작한다(문단 21 host 줄 29.9px 이월이 없다).
    let (big_top, _) = table_box(&p2.root, 23).expect("2쪽 큰 표");
    let host21 = &core.document().sections[0].paragraphs[21].line_segs[0];
    let host21_advance = hu_to_px(host21.line_height + host21.line_spacing);
    assert!(
        big_top > title_bottom && big_top - title_bottom < host21_advance,
        "큰 표 위 {big_top:.1} 는 제목 표 아래 {title_bottom:.1} 바로 아래(문단 21 host 줄 {host21_advance:.1}px 미만)"
    );
    assert!(title_top < title_bottom);
    assert!(
        (title_bottom - PDF_P2_TITLE_BOTTOM).abs() <= 2.0,
        "제목 표 아래 {title_bottom:.1} = 한/글 PDF {PDF_P2_TITLE_BOTTOM} ±2"
    );
    assert!(
        (big_top - PDF_P2_BIG_TABLE_TOP).abs() <= 2.0,
        "큰 표 위 {big_top:.1} = 한/글 PDF {PDF_P2_BIG_TABLE_TOP} ±2"
    );

    // 1쪽: 차선의 문단 17 첫 줄 → 전체 폭 둘째 줄 → 문단 18 은 저장 사다리 간격 그대로다.
    let doc = core.document();
    let para17 = &doc.sections[0].paragraphs[17];
    let para18 = &doc.sections[0].paragraphs[18];
    assert_eq!(
        para17.line_segs.len(),
        2,
        "문단 17 은 차선 1줄 + 전체 폭 1줄"
    );
    assert!(
        para17.line_segs[0].segment_width < para17.line_segs[1].segment_width,
        "첫 줄만 표 옆 차선"
    );
    let l0 = line_top(&p1.root, 17, 0).expect("문단 17 첫 줄");
    let l1 = line_top(&p1.root, 17, 1).expect("문단 17 둘째 줄");
    let n0 = line_top(&p1.root, 18, 0).expect("문단 18 첫 줄");
    let ladder_l1 = hu_to_px(para17.line_segs[1].vertical_pos - para17.line_segs[0].vertical_pos);
    let ladder_n0 = hu_to_px(para18.line_segs[0].vertical_pos - para17.line_segs[1].vertical_pos);
    assert!(
        (l1 - l0 - ladder_l1).abs() <= 0.5,
        "문단 17 둘째 줄 간격 {:.2} = 저장 {ladder_l1:.2}",
        l1 - l0
    );
    assert!(
        (n0 - l1 - ladder_n0).abs() <= 0.5,
        "문단 18 간격 {:.2} = 저장 {ladder_n0:.2}",
        n0 - l1
    );
    let (para20_top, _) = table_box(&p1.root, 20).expect("1쪽 문단 20 표");
    assert!(
        (para20_top - PDF_P1_PARA20_TABLE_TOP).abs() <= 2.0,
        "문단 20 표 위 {para20_top:.1} = 한/글 PDF {PDF_P1_PARA20_TABLE_TOP} ±2"
    );
}
