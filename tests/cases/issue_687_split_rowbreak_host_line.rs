//! [planet6897/rhwp-issue-docs#687] 관계형·고정형 회귀 — 여러 쪽으로 나뉜 빈 host RowBreak 자리차지 표가 끝난 뒤 host 줄 상자를 흐름에 남긴다.
//!
//! 주석 갱신: 2026-11-26T21:55:00+09:00 (devel `3fdaa04258` rebase 뒤 재측정). 첫 작성 2026-11-25T02:21:00+09:00, 새 파일.
//!
//! - 입력: `samples/issue687/gochang_wetland_final_report.hwp` — 공개 문서
//!   「제2차 고창 운곡습지보호지역 보전계획 수립 연구용역 최종보고회 결과」(HWP5, 4쪽, 10k 코퍼스 #7287).
//!   문단 24 는 공백 둘(`"  "`)만 있는 host 에 8×2 자리차지(TopAndBottom)·RowBreak·비-TAC 표 하나를
//!   매단다(vert=문단, 세로 오프셋 0). 표는 2~4쪽에 나뉜다. 저장 줄: host `vpos 46200, lh 1500,
//!   ls 752`, 다음 문단 25 「향후 계획」 `vpos 48452` — `48452 − 46200 = 1500 + 752`(접힌 사다리).
//!   문단 25 에는 글앞으로 TAC 도형(□ 기호)이 있다.
//! - 독립 기준: `pdf/issue687/gochang_wetland_final_report-2022.pdf`(한컴오피스 2022 12.0.0.4547
//!   Print → Hancom PDF, 4쪽). 4쪽 표 끝 아래에 host 줄 한 줄(≈30px)을 두고 「향후 계획」이 이어진다.
//! - 수정 전 rhwp(devel `3fdaa04258`)는 끝 조각 뒤 host 줄 전진(#2439 계약)을 양수 오프셋·개체 없는
//!   다음 문단에서만 계상해 「향후 계획」 이하가 표에 붙었다(약 30px 위).
//!
//! 측정: 2026-11-26 KST(base `3fdaa04258` 위 rebase head), `scripts/visual_sweep.py --silhouette-only`, compat 2022·96 DPI·보정 전 2px raw
//! 실루엣(`silhouette_raw_match_percent`). 전체 4쪽 = 한컴 Print 4쪽. 대상 4쪽(표 끝 조각·「향후 계획」)
//! Native 95.67% / fresh WASM 95.97%. 전쪽 최저 Native 92.72%(3쪽) / fresh WASM 92.87%(3쪽).
//! 수정 전(base `3fdaa04258`) Native 4쪽 86.66%(1~3쪽은 수정 후와 같음). 4쪽 표 끝·「향후 계획」 경계를 독립 PDF와 같은 배율로 확대 판독했다.
//! 증거: `mydocs/pr/assets/issue_687/{native,wasm,base}/`(전쪽 TSV·review·overlay PNG, 확대 비교).
//!
//! 검사: 원본 계약(빈 host, 자리차지 RowBreak 표 하나, 오프셋 0, 접힌 사다리), 쪽수 4 = 한컴(고정형),
//! 4쪽 표 끝 → 문단 25 첫 줄 간격 ≥ host 줄 전진(관계형), 표가 3개 쪽에 나뉨(관계형).
//!
//! 반례(두 번째 검사): 같은 원본에서 다음 문단의 저장 vpos 를 표 높이만큼 더 내린 물리 사다리 변형.
//! 사다리가 host 줄만 증언하지 않으면 끝 조각 뒤에 host 줄을 더하지 않는다(#2808 판별자 유지).
//! 합성 변형이며 한컴 출력 일치 증거가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use rhwp::model::table::TablePageBreak;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue687/gochang_wetland_final_report.hwp";
const HOST_PARA: usize = 24;
const NEXT_PARA: usize = 25;
const LAST_PAGE: u32 = 3;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn collect<'a>(
    node: &'a RenderNode,
    pred: &dyn Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if pred(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, pred, out);
    }
}

fn host_table_fragment_bottom(core: &DocumentCore, page: u32) -> Option<f64> {
    let tree = core.build_page_render_tree(page).expect("쪽 render tree");
    let mut tables = Vec::new();
    collect(
        &tree.root,
        &|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(HOST_PARA) && t.cell_context.is_none())
        },
        &mut tables,
    );
    tables
        .iter()
        .map(|node| node.bbox.y + node.bbox.height)
        .reduce(f64::max)
}

fn next_para_first_line_top(core: &DocumentCore, page: u32) -> f64 {
    let tree = core.build_page_render_tree(page).expect("쪽 render tree");
    let mut lines = Vec::new();
    collect(
        &tree.root,
        &|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.para_index == Some(NEXT_PARA) && line.line_index == Some(0))
        },
        &mut lines,
    );
    lines.first().expect("문단 25 첫 줄").bbox.y
}

/// (host 줄 전진 px, 표 바깥 아래 여백 px)
fn host_contract(core: &DocumentCore) -> (f64, f64) {
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[HOST_PARA];
    let next = &doc.sections[0].paragraphs[NEXT_PARA];
    assert!(
        para.text.chars().all(char::is_whitespace),
        "문단 {HOST_PARA} host 는 공백뿐: {:?}",
        para.text
    );
    let tables: Vec<_> = para
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 1, "host 의 표는 하나");
    let table = tables[0];
    assert!(!table.common.treat_as_char, "비-TAC 표");
    assert!(
        matches!(table.common.text_wrap, TextWrap::TopAndBottom),
        "자리차지 표"
    );
    assert!(
        matches!(table.layout_page_break(), TablePageBreak::RowBreak),
        "RowBreak 나눔"
    );
    assert_eq!(table.common.vertical_offset, 0, "세로 오프셋 0");

    let host = para.line_segs.first().expect("host 저장 줄");
    let next_line = next.line_segs.first().expect("다음 문단 저장 줄");
    let advance = host.line_height + host.line_spacing;
    assert_eq!(
        next_line.vertical_pos - host.vertical_pos,
        advance,
        "접힌 사다리: 다음 vpos − host vpos = lh + ls"
    );
    (
        hu_to_px(advance),
        hu_to_px(i32::from(table.outer_margin_bottom)),
    )
}

#[test]
fn split_rowbreak_empty_host_keeps_host_line_after_last_fragment() {
    let core = load();
    assert_eq!(core.page_count(), 4, "한컴 Print 와 같은 4쪽");
    let (advance, outer_bottom) = host_contract(&core);

    let fragment_pages: Vec<u32> = (0..core.page_count())
        .filter(|page| host_table_fragment_bottom(&core, *page).is_some())
        .collect();
    assert_eq!(fragment_pages, vec![1, 2, 3], "표는 2~4쪽에 나뉜다");

    let table_bottom = host_table_fragment_bottom(&core, LAST_PAGE).expect("4쪽 끝 조각");
    let next_top = next_para_first_line_top(&core, LAST_PAGE);
    let gap = next_top - table_bottom;
    assert!(
        gap + 0.5 >= advance,
        "끝 조각 아래에 host 줄({advance:.2}px)이 남는다: 표 끝 {table_bottom:.2} → 「향후 계획」 {next_top:.2} (간격 {gap:.2})"
    );
    assert!(
        gap <= advance + outer_bottom + 1.0,
        "host 줄과 바깥 아래 여백({outer_bottom:.2}px)만큼만 띄운다: 간격 {gap:.2}"
    );
}

/// 반례: 사다리가 표 높이를 품으면(물리 사다리) 끝 조각 뒤에 host 줄을 더하지 않는다.
#[test]
fn physical_ladder_does_not_add_host_line_after_last_fragment() {
    let mut core = load();
    let mut doc = core.document().clone();
    let table_height = match &doc.sections[0].paragraphs[HOST_PARA].controls[0] {
        Control::Table(table) => table.common.height as i32,
        _ => panic!("host 첫 컨트롤은 표"),
    };
    let next = &mut doc.sections[0].paragraphs[NEXT_PARA];
    for seg in &mut next.line_segs {
        seg.vertical_pos += table_height;
    }
    if let Some(source) = next.source_line_seg_vertical_pos.as_mut() {
        for vpos in source.iter_mut() {
            *vpos += table_height;
        }
    }
    core.set_document(doc);

    let host = &core.document().sections[0].paragraphs[HOST_PARA].line_segs[0];
    let advance = hu_to_px(host.line_height + host.line_spacing);
    let last_page = (0..core.page_count())
        .filter(|page| host_table_fragment_bottom(&core, *page).is_some())
        .max()
        .expect("표 끝 조각");
    let table_bottom = host_table_fragment_bottom(&core, last_page).expect("끝 조각");
    let next_top = next_para_first_line_top(&core, last_page);
    assert!(
        next_top - table_bottom < advance - 0.5,
        "물리 사다리에서는 host 줄 전진({advance:.2}px)을 더하지 않는다: 간격 {:.2}",
        next_top - table_bottom
    );
}
