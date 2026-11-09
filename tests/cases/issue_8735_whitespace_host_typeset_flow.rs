//! [#8735] 관계형·고정형 회귀 — 공백 host 문단의 문단 기준 자리차지 표가 저장 첫 줄을 띠 아래로
//! 밀면, 조판 흐름은 표 상자와 그 줄만 예약한다(문단 앞 간격·host 줄간격을 표 예약에 얹지 않는다).
//!
//! 주석 갱신: 2026-11-09 07:41 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8735/whitespace_host_approval_table.hwpx` — 공개 결재문서 36325626(HWPX, 1쪽).
//!   문단 1 은 공백 한 칸 host, 문단 기준 자리차지 결재 표(오프셋 0)가 저장 첫 줄을 띠 끝
//!   13697 = 앞 문단 끝 6392 + 바깥 위 138 + 표 7029 + 바깥 아래 138 로 민다(앞 간격 852 없음).
//! - 독립 기준: `pdf/issue8735/whitespace_host_approval_table-2020.pdf`(한/글 2022 12.0.0.4547 Print, 1쪽).
//! - 수정 전 rhwp(PR #8758 rebase 본 `0b54ab1879`, 옛 head `a5f2d2b28b` 도 같음)는 배치는 저장 사다리대로 두지만 조판이 결재 표
//!   예약에 앞 간격·바깥 위 여백·host 줄간격을 얹고 host 줄을 따로 또 세어, 조판이 원점을 정하는
//!   문서번호 TAC 표가 17.5px 아래에 놓였다(문서번호 표 − host 줄 52.1px, 기대 34.6px).
//!
//! 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 PR #8758 rebase 본 `0b54ab1879` + 이 수정.
//! 1쪽 Native 99.99 / fresh WASM 100.00(수정 전 Native 58.08). 전체 1쪽 = PDF 1쪽, 전쪽 최저
//! Native 99.99(1쪽)·fresh WASM 100.00(1쪽).
//! 증거: `mydocs/pr/assets/issue_8735/typeset/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사: 원본 계약(공백 host·저장 줄 = 띠 끝·줄이 표를 품지 않음·앞 간격 양수), 쪽수 1 = 한/글
//! (고정형), host 줄 − 결재 표 아래끝 = 바깥 아래 여백, 문서번호 표 − host 줄 = 저장 vpos 차이 +
//! 바깥 위 여백(관계형, ±1px — 양쪽 raw 99.99% 이상 쪽).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8735/whitespace_host_approval_table.hwpx";
const DPI: f64 = 96.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 본문 흐름(표 칸 안 제외)의 문단별 첫 표 상자와 첫 글줄 위끝.
#[derive(Default)]
struct Flow {
    tables: Vec<(usize, f64, f64)>,
    lines: Vec<(usize, f64)>,
}

fn collect(node: &RenderNode, flow: &mut Flow) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index {
                if !flow.tables.iter().any(|(owner, _, _)| *owner == para) {
                    flow.tables
                        .push((para, node.bbox.y, node.bbox.y + node.bbox.height));
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

fn first_table(para: &Paragraph) -> &Table {
    para.controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표")
}

#[test]
fn whitespace_host_float_reserves_only_the_band_and_its_pushed_line() {
    let core = load();
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;

    // 원본 계약: 문단 1 은 공백 한 칸 host, 문단 기준 자리차지 표(오프셋 0)가 저장 첫 줄을
    // 띠 아래로 민다 — 줄 vpos = 앞 문단 끝 + 바깥 위 + 표 + 바깥 아래, 문단 앞 간격(양수)은 없다.
    let host = &paragraphs[1];
    assert_eq!(host.text, " ", "공백 host");
    let approval = first_table(host);
    assert!(!approval.common.treat_as_char);
    let before = doc.doc_info.para_shapes[host.para_shape_id as usize].spacing_before;
    assert!(before > 0, "문단 1 앞 간격");
    let prev = paragraphs[0].line_segs.last().unwrap();
    let prev_end = prev.vertical_pos + prev.line_height + prev.line_spacing;
    let band = i32::from(approval.outer_margin_top)
        + approval.common.height as i32
        + i32::from(approval.outer_margin_bottom);
    let host_line = &host.line_segs[0];
    assert_eq!(host_line.vertical_pos, prev_end + band, "저장 줄 = 띠 끝");
    assert!(host_line.line_height < band, "저장 줄이 표를 품지 않는다");
    assert_eq!(core.page_count(), 1, "전체 쪽수 = 한/글 1쪽");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut flow = Flow::default();
    collect(&page.root, &mut flow);
    let table = |para: usize| {
        flow.tables
            .iter()
            .find(|(owner, _, _)| *owner == para)
            .copied()
            .unwrap_or_else(|| panic!("문단 {para} 표"))
    };
    let line = |para: usize| {
        flow.lines
            .iter()
            .find(|(owner, _)| *owner == para)
            .map(|(_, y)| *y)
            .unwrap_or_else(|| panic!("문단 {para} 글줄"))
    };

    // 표가 민 host 줄은 결재 표 바깥 아래 끝에 붙는다.
    let (_, _, approval_bottom) = table(1);
    let host_top = line(1);
    let expected = px(i32::from(approval.outer_margin_bottom));
    assert!(
        (host_top - approval_bottom - expected).abs() <= 1.0,
        "host 줄 − 결재 표 아래끝 {:.1} = 바깥 아래 여백 {expected:.1}",
        host_top - approval_bottom
    );

    // 다음 문단(문서번호 TAC 표)은 host 줄과 저장 vpos 차이 + 바깥 위 여백만큼 아래다.
    let number = first_table(&paragraphs[2]);
    let (_, number_top, _) = table(2);
    let expected = px(paragraphs[2].line_segs[0].vertical_pos - host_line.vertical_pos)
        + px(i32::from(number.outer_margin_top));
    assert!(
        (number_top - host_top - expected).abs() <= 1.0,
        "문서번호 표 − host 줄 {:.1} = 저장 vpos 차이 + 바깥 위 여백 {expected:.1}",
        number_top - host_top
    );
}
