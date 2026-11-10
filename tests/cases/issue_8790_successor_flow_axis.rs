//! [#8790] 관계형 회귀 — 다음 문단 저장 위치로 문단 끝을 정할 때, 원본 저장 축의 프레임을 현재
//! 줄 축 기준점으로 투영하지 않는다. 표 문단은 바로 앞 글줄의 저장 전진 바로 아래에서 시작한다.
//!
//! 주석 갱신: 2026-11-09 01:03 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8790/successor_flow_axis.hwpx` — 공개 결재문서 36302427(HWPX, 2쪽).
//!   문단 0 의 줄은 합성 줄(tag 최상위 비트)이라 rhwp 는 열 때 뒤 문단 vpos 를 다시 계산한다(이 문서는
//!   3840HU 당겨짐). 문단 3·5·7 은 글줄, 바로 뒤 문단 4·6·8 은 글자처럼 취급 표다.
//! - 독립 기준: `pdf/issue8790/successor_flow_axis-2020.pdf`(한/글 2022 Print, 2쪽). 표 문단은 바로 앞
//!   글줄의 저장 전진(줄 높이 + 간격) 바로 아래에서 바깥 위 여백만큼 안쪽에 시작한다.
//! - 수정 전 rhwp(devel `80fc0c6b44`)는 원본 축 흐름 끝을 현재 축 기준점에 투영해 표 문단마다 51.2px
//!   (3840HU) 아래에 놓았다(문단 4: 간격 78.7px, 기대 27.5px).
//!
//! 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `80fc0c6b44` + 이 수정.
//! 1쪽 Native 100.00 / fresh WASM 100.00(수정 전 Native 38.94), 2쪽 99.97 / 100.00(수정 전 99.97).
//! 전체 2쪽 = PDF 2쪽, 전쪽 최저 Native 99.97(2쪽)·fresh WASM 100.00.
//! 증거: `mydocs/pr/assets/issue_8790/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사(관계형): 원본 계약(문단 0 합성 줄), 표 위끝 − 앞 글줄 위끝 = 저장 전진 + 바깥 위 여백(±1px,
//! 문단 쌍 3·4 / 5·6 / 7·8), 문단 3~9 의 1쪽 소속.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8790/successor_flow_axis.hwpx";
const DPI: f64 = 96.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 본문 흐름(표 칸 안 제외)의 문단별 첫 글줄 위끝과 표 위끝.
fn collect(node: &RenderNode, lines: &mut Vec<(usize, f64)>, tables: &mut Vec<(usize, f64)>) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index {
                tables.push((para, node.bbox.y));
            }
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                if !lines.iter().any(|(owner, _)| *owner == para) {
                    lines.push((para, node.bbox.y));
                }
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, lines, tables);
    }
}

#[test]
fn table_paragraph_starts_right_after_the_preceding_text_line() {
    let core = load();
    let paragraphs = &core.document().sections[0].paragraphs;
    // 원본 계약: 첫 문단 줄은 합성 줄이고, 뒤 문단은 한/글 저장 줄이다.
    assert!(paragraphs[0]
        .line_segs
        .iter()
        .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0));

    let page = core.build_page_render_tree(0).expect("1쪽");
    let (mut lines, mut tables) = (Vec::new(), Vec::new());
    collect(&page.root, &mut lines, &mut tables);
    let line_top = |para: usize| {
        lines
            .iter()
            .find(|(owner, _)| *owner == para)
            .map(|(_, y)| *y)
            .unwrap_or_else(|| panic!("문단 {para} 글줄"))
    };
    let table_top = |para: usize| {
        tables
            .iter()
            .find(|(owner, _)| *owner == para)
            .map(|(_, y)| *y)
            .unwrap_or_else(|| panic!("문단 {para} 표"))
    };
    // 글줄 문단 3·5·7 바로 뒤의 TAC 표 문단 4·6·8.
    for (text, table_para) in [(3usize, 4usize), (5, 6), (7, 8)] {
        let seg = &paragraphs[text].line_segs[0];
        let Some(Control::Table(table)) = paragraphs[table_para].controls.first() else {
            panic!("문단 {table_para} 표");
        };
        let expected =
            px(seg.line_height + seg.line_spacing) + px(i32::from(table.outer_margin_top));
        let gap = table_top(table_para) - line_top(text);
        assert!(
            (gap - expected).abs() <= 1.0,
            "문단 {table_para} 표 위끝 − 문단 {text} 글줄 {gap:.1} = 저장 전진 + 바깥 위 여백 {expected:.1}"
        );
    }
    // 내용 순서와 소속: 문단 3~9 가 모두 1쪽에 있다.
    for para in [3usize, 5, 7, 9] {
        line_top(para);
    }
}
