//! [#8836] 관계형·고정형 회귀 — 쪽 기준 자리차지 표가 있는 글 문단의 첫 줄에 문단 앞 간격을
//! 한 번만(표가 민 줄은 띠 위에 남는 몫만) 쓴다.
//!
//! 주석 갱신: 2026-11-09 14:48 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8836/page_float_host_spacing.hwpx` — 공개 결재문서 36346146(HWPX, 1쪽).
//!   문단 0 은 글 host, 쪽 기준 자리차지 머리 표(바깥 위 140 + 높이 15617 + 바깥 아래 852 = 16609)가
//!   첫 줄을 저장 vpos 16609(띠 끝)로 민다. 문단 10 은 쪽 아래 정렬 자리차지 표를 가진 글 host 다.
//!   두 문단의 앞 간격은 200HU 다.
//! - 독립 기준: `pdf/issue8836/page_float_host_spacing-2020.pdf`(한/글 2022 12.0.0.4547 Print, 1쪽).
//! - 수정 전 rhwp(devel `024d610633`)는 문단 0 첫 줄에 앞 간격을 두 번 더해(배치 「표 위 간격」
//!   분기 + 글줄 조각) 본문 전체가 5.3px 아래, 문단 10 은 +2.7px 였다(첫 줄 − 표 아래끝 16.7px,
//!   기대 11.4px).
//!
//! 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `024d610633` + 이 수정.
//! 1쪽 Native 99.92 / fresh WASM 99.92(수정 전 Native 81.49). 전체 1쪽 = PDF 1쪽, 전쪽 최저
//! Native 99.92(1쪽)·fresh WASM 99.92(1쪽).
//! 증거: `mydocs/pr/assets/issue_8836/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사: 원본 계약(쪽 기준 자리차지 표의 위·아래 정렬, 저장 첫 줄 = 띠 끝, 앞 간격 양수), 쪽수 1 =
//! 한/글(고정형), 문단 0 첫 줄 − 머리 표 아래끝 = 바깥 아래 여백, 문단 10 − 문단 9 = 저장 vpos 차이
//! (관계형, ±1px — 양쪽 raw 99.92% 쪽).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{TextWrap, VertAlign, VertRelTo};
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8836/page_float_host_spacing.hwpx";
const DPI: f64 = 96.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 본문 흐름(표 칸 안 제외)의 (문단, 줄) 위끝과 문단별 첫 표 상자.
#[derive(Default)]
struct Flow {
    lines: Vec<(usize, u32, f64)>,
    tables: Vec<(usize, usize, f64, f64)>,
}

fn collect(node: &RenderNode, flow: &mut Flow) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let (Some(para), Some(control)) = (table.para_index, table.control_index) {
                flow.tables
                    .push((para, control, node.bbox.y, node.bbox.y + node.bbox.height));
            }
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let (Some(para), Some(index)) = (line.para_index, line.line_index) {
                flow.lines.push((para, index, node.bbox.y));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, flow);
    }
}

fn page_float(para: &Paragraph) -> (usize, &Table) {
    para.controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::Table(table)
                if !table.common.treat_as_char
                    && table.common.text_wrap == TextWrap::TopAndBottom
                    && table.common.vert_rel_to == VertRelTo::Page =>
            {
                Some((index, table.as_ref()))
            }
            _ => None,
        })
        .expect("쪽 기준 자리차지 표")
}

#[test]
fn page_float_text_host_uses_spacing_before_once() {
    let core = load();
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;
    let before =
        |para: &Paragraph| doc.doc_info.para_shapes[para.para_shape_id as usize].spacing_before;

    // 원본 계약: 문단 0 은 글 host, 쪽 기준 자리차지 머리 표(위 정렬·오프셋 0)가 첫 줄을 띠 끝
    // (바깥 위 + 높이 + 바깥 아래)으로 민다. 문단 10 은 쪽 아래 정렬 표를 가진 글 host 다.
    // 두 문단 모두 앞 간격이 양수다.
    let host = &paragraphs[0];
    let (head_index, head) = page_float(host);
    assert_eq!(head.common.vert_align, VertAlign::Top);
    assert_eq!(head.common.vertical_offset, 0);
    let band = i32::from(head.outer_margin_top)
        + head.common.height as i32
        + i32::from(head.outer_margin_bottom);
    assert_eq!(host.line_segs[0].vertical_pos, band, "저장 첫 줄 = 띠 끝");
    assert!(before(host) > 0, "문단 0 앞 간격");
    let tail = &paragraphs[10];
    assert_eq!(page_float(tail).1.common.vert_align, VertAlign::Bottom);
    assert!(before(tail) > 0, "문단 10 앞 간격");
    assert_eq!(core.page_count(), 1, "전체 쪽수 = 한/글 1쪽");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut flow = Flow::default();
    collect(&page.root, &mut flow);
    let line = |para: usize, index: u32| {
        flow.lines
            .iter()
            .find(|(p, i, _)| *p == para && *i == index)
            .map(|(_, _, y)| *y)
            .unwrap_or_else(|| panic!("문단 {para} 줄 {index}"))
    };
    let (_, _, _, head_bottom) = flow
        .tables
        .iter()
        .copied()
        .find(|(p, c, _, _)| *p == 0 && *c == head_index)
        .expect("머리 표");

    // 표가 민 첫 줄은 띠 끝 — 표 아래끝 + 바깥 아래 여백(앞 간격은 띠 안에서 흡수).
    let expected = px(i32::from(head.outer_margin_bottom));
    let got = line(0, 0) - head_bottom;
    assert!(
        (got - expected).abs() <= 1.0,
        "문단 0 첫 줄 − 머리 표 아래끝 {got:.1} = 바깥 아래 여백 {expected:.1}"
    );

    // 밀리지 않은 문단 10 의 첫 줄은 앞 문단과 저장 vpos 차이만큼 아래(앞 간격 한 번).
    let previous = &paragraphs[9].line_segs[0];
    let expected = px(tail.line_segs[0].vertical_pos - previous.vertical_pos);
    let got = line(10, 0) - line(9, 0);
    assert!(
        (got - expected).abs() <= 1.0,
        "문단 10 − 문단 9 {got:.1} = 저장 vpos 차이 {expected:.1}"
    );
}
