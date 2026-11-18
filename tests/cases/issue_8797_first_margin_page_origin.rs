//! [#8797] 관계형·고정형 회귀 — 쪽 첫 저장 줄의 vpos 가 그 문단 앞 간격을 담지 않고 저장 사다리가
//! 다음 문단으로 정확히 닫히면, 쪽 원점은 그 간격을 뺀 값이다. 뒤 저장 줄은 vpos 차이대로 놓인다.
//!
//! 주석 갱신: 2026-11-09 04:22 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8797/first_margin_page_origin.hwpx` — 공개 결재문서 36392221(HWPX, 1쪽).
//!   문단 0(제목 표)의 저장 vpos 0 이 앞 간격 852 HU 를 담지 않고, 문단 0 끝 + 뒤 간격 + 문단 1 앞
//!   간격이 문단 1 의 저장 vpos 와 정확히 같다(사다리가 닫힘).
//! - 독립 기준: `pdf/issue8797/first_margin_page_origin-2020.pdf`(한/글 2022 12.0.0.4547 Print, 1쪽).
//! - 수정 전 rhwp(devel `06181802e5`)는 쪽 원점을 첫 vpos 그대로 두어, 단일 저장 개체 줄인 문단 2
//!   (문서번호 표)부터 앞 간격만큼(약 11px) 위로 당겨 그렸다(결재 표 → 문서번호 표 130.1px, 저장
//!   vpos 차이 141.5px).
//!
//! 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `06181802e5` + 이 수정.
//! 1쪽 Native 100.00 / fresh WASM 100.00(수정 전 Native 70.17). 전체 1쪽 = PDF 1쪽, 전쪽 최저
//! Native 100.00(1쪽)·fresh WASM 100.00(1쪽).
//! 증거: `mydocs/pr/assets/issue_8797/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사: 원본 계약(문단 0 저장 줄 vpos 0·앞 간격 양수, 문단 1~3 저장 줄), 쪽수 1 = 한/글(고정형),
//! 본문 위 → 제목 표, 결재 표 → 문서번호 표, 문서번호 표 → 근무지 글줄 간격 = 앞 간격·저장 vpos
//! 차이·바깥 여백(관계형, 양쪽 raw 100.00% 쪽이므로 ±1px).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8797/first_margin_page_origin.hwpx";
const DPI: f64 = 96.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 본문 흐름(표 칸 안 제외)의 문단별 첫 표 위끝과 첫 글줄 위끝, 본문 위끝.
#[derive(Default)]
struct Flow {
    body_top: Option<f64>,
    tables: Vec<(usize, f64)>,
    lines: Vec<(usize, f64)>,
}

fn collect(node: &RenderNode, flow: &mut Flow) {
    match &node.node_type {
        RenderNodeType::Body { .. } if flow.body_top.is_none() => flow.body_top = Some(node.bbox.y),
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index {
                if !flow.tables.iter().any(|(owner, _)| *owner == para) {
                    flow.tables.push((para, node.bbox.y));
                }
            }
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                if !flow.lines.iter().any(|(owner, _)| *owner == para) {
                    flow.lines.push((para, node.bbox.y));
                }
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, flow);
    }
}

fn find(items: &[(usize, f64)], para: usize, what: &str) -> f64 {
    items
        .iter()
        .find(|(owner, _)| *owner == para)
        .map(|(_, y)| *y)
        .unwrap_or_else(|| panic!("문단 {para} {what}"))
}

/// 글자처럼 취급 표 하나의 바깥 위 여백.
fn tac_table_outer_top(para: &Paragraph) -> i32 {
    para.controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.common.treat_as_char => {
                Some(i32::from(table.outer_margin_top))
            }
            _ => None,
        })
        .expect("글자처럼 취급 표")
}

#[test]
fn table_after_unstored_first_margin_keeps_the_stored_ladder() {
    let core = load();
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;
    // 문단 여백은 짝수 저장값이면 2배 HWPUNIT 단위다(홀수는 글자 단위).
    let before = |para: &Paragraph| {
        let stored = doc.doc_info.para_shapes[para.para_shape_id as usize].spacing_before;
        assert_eq!(stored % 2, 0, "HWPUNIT 단위 앞 간격");
        stored / 2
    };

    // 원본 계약: 문단 0 은 한/글 저장 줄이고 vpos 0 이 앞 간격(양수)을 담지 못한다.
    let first = &paragraphs[0].line_segs[0];
    assert_eq!(first.tag & (1 << 31), 0, "문단 0 은 한/글 저장 줄");
    assert_eq!(first.vertical_pos, 0);
    assert!(before(&paragraphs[0]) > 0, "문단 0 앞 간격");
    for index in [1, 2, 3] {
        assert_eq!(paragraphs[index].line_segs[0].tag & (1 << 31), 0);
    }
    assert_eq!(core.page_count(), 1, "전체 쪽수 = 한/글 1쪽");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut flow = Flow::default();
    collect(&page.root, &mut flow);
    let vpos = |index: usize| paragraphs[index].line_segs[0].vertical_pos;

    // 흐름은 문단 0 의 앞 간격을 쪽 맨 위에 둔다(수정 전후 공통, 한/글 Print 와 같음).
    let body = flow.body_top.expect("본문");
    let title = find(&flow.tables, 0, "제목 표");
    let expected = px(before(&paragraphs[0])) + px(tac_table_outer_top(&paragraphs[0]));
    assert!(
        (title - body - expected).abs() <= 1.0,
        "제목 표 위끝 − 본문 위 {:.1} = 앞 간격 + 바깥 위 여백 {expected:.1}",
        title - body
    );

    // 결재 표(문단 1) → 문서번호 표(문단 2): 두 저장 줄 vpos 차이 + 바깥 위 여백 차이.
    let approval = find(&flow.tables, 1, "결재 표");
    let number = find(&flow.tables, 2, "문서번호 표");
    let expected = px(vpos(2) - vpos(1)) + px(tac_table_outer_top(&paragraphs[2]))
        - px(tac_table_outer_top(&paragraphs[1]));
    assert!(
        (number - approval - expected).abs() <= 1.0,
        "문서번호 표 − 결재 표 {:.1} = 저장 vpos 차이 {expected:.1}",
        number - approval
    );

    // 문서번호 표(문단 2) → 「(근무지 …)」 글줄(문단 3): 저장 vpos 차이 − 표 바깥 위 여백.
    let place = find(&flow.lines, 3, "근무지 글줄");
    let expected = px(vpos(3) - vpos(2)) - px(tac_table_outer_top(&paragraphs[2]));
    assert!(
        (place - number - expected).abs() <= 1.0,
        "근무지 글줄 − 문서번호 표 {:.1} = 저장 vpos 차이 {expected:.1}",
        place - number
    );
}
