//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8422의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:30+09:00 (`fe79988fa1ee006243d70b248682638eb99e9f3e`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8418/30026-safety-reform-toc.hwp`; 독립 Print `pdf/issue8418/30026-safety-reform-toc-2022.pdf`.
//! 전체 37쪽, 전쪽 최저 Native 97.65% / fresh WASM 97.52%.
//! 주장 3쪽 raw: Native 99.79% / fresh WASM 99.83%.
//! 주장 28쪽 raw: Native 97.65% / fresh WASM 97.52%.
//! 입력 `samples/issue6737/156487948-marine-pollution-law.hwp`; 독립 Print `pdf/issue6737-156487948-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 98.89% / fresh WASM 98.89%.
//! 주장 1쪽 raw: Native 98.89% / fresh WASM 98.89%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8422_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-01T12:47:06+09:00 (새 파일).
//! 2026-11-01 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`):
//! - 30026-safety-reform-toc.hwp(독립 한컴 2022 12.0.0.4547 Print, 37쪽): 3쪽 Native 99.66% /
//!   fresh WASM 98.01%. 전 37쪽 최저 Native 97.80% / fresh WASM 93.02%. 수정 전 3쪽 Native 34.08%.
//! - 156487948-marine-pollution-law.hwp(독립 한컴 2020 Print, 2쪽): 1쪽 Native 98.83% /
//!   fresh WASM 98.56%. 전 2쪽 최저 Native 92.75% / fresh WASM 98.56%. 수정 전 1쪽 Native 60.03%.
//!
//! 증거: mydocs/pr/assets/issue_8418/{native,wasm}/*_silhouette.tsv.
//! 근거: 저장 LINE_SEG 의 표 줄 원점과 다음 문단 첫 줄 원점의 차(vpos)를 그대로 쓴다.
//! 절대 px 좌표는 고정하지 않는다. 실제 위치는 독립 한컴 Print 와 Visual Sweep 으로 확인했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// 본문 흐름의 노드만 모은다. 셀 문단도 같은 `para_index` 를 쓰므로 표 안으로 내려가지 않는다.
fn collect<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    for child in &node.children {
        collect(child, nodes);
    }
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

struct Case {
    sample: &'static str,
    page_count: u32,
    page: u32,
    host: usize,
    table_control: usize,
    follower: usize,
    follower_text: &'static str,
}

fn verify(case: &Case) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(case.sample);
    let bytes = std::fs::read(&path).expect("시각 검증한 원본");
    let source = rhwp::parser::parse_document(&bytes).expect("원본 저장 줄 파싱");
    let paragraphs = &source.sections[0].paragraphs;
    let host = &paragraphs[case.host];
    let Some(Control::Table(table)) = host.controls.get(case.table_control) else {
        panic!("{}: 문단 {}의 TAC 표", case.sample, case.host);
    };
    assert!(table.common.treat_as_char);
    // 표가 놓인 저장 줄은 문단의 마지막 줄이고, 그 앞 줄에는 다른 글자처럼 취급 개체가 있다.
    let table_line = host.line_segs.last().expect("표 줄");
    assert!(
        host.line_segs.len() >= 2,
        "앞 저장 줄의 개체와 표 줄이 분리돼 있어야 한다"
    );
    assert!(table_line.line_spacing > 0);
    assert!(
        host.controls[..case.table_control]
            .iter()
            .any(|control| match control {
                Control::Table(t) => t.common.treat_as_char,
                Control::Picture(p) => p.common.treat_as_char,
                Control::Shape(s) => s.common().treat_as_char,
                _ => false,
            }),
        "앞 줄의 가시 TAC 개체가 이 반례의 전제다"
    );
    let follower = &paragraphs[case.follower];
    let follower_vpos = follower.line_segs[0].vertical_pos;
    let expected = hu_to_px(follower_vpos - table_line.vertical_pos);
    assert!(
        expected > hu_to_px(table_line.line_height),
        "다음 문단은 표 줄 높이에 줄간격까지 지난 뒤 시작한다"
    );

    let mut core = DocumentCore::from_bytes(&bytes).expect("동일 원본 배치");
    assert_eq!(
        core.page_count(),
        case.page_count,
        "{}: 독립 Print 전체 쪽수",
        case.sample
    );
    let page = core.build_page_render_tree(case.page).expect("대상 쪽");
    let mut nodes = Vec::new();
    collect(&page.root, &mut nodes);
    let table_node = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(case.host) && t.control_index == Some(case.table_control))
        })
        .expect("같은 쪽의 표");
    // 표 줄 상자의 위쪽은 표 위 바깥 여백만큼 앞이다.
    let table_line_top = table_node.bbox.y - hu_to_px(i32::from(table.outer_margin_top));
    let follower_line = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.para_index == Some(case.follower) && line.line_index == Some(0))
        })
        .expect("같은 쪽의 다음 본문 첫 줄");
    let text: String = follower_line
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        text.contains(case.follower_text),
        "{}: 다음 본문 내용 보존 ({text})",
        case.sample
    );
    let actual = follower_line.bbox.y - table_line_top;
    assert!(
        (actual - expected).abs() <= 1.0,
        "{}: 표 줄 원점→다음 문단 첫 줄 {actual:.2}px, 저장 사다리 {expected:.2}px",
        case.sample
    );
}

#[test]
fn tac_table_line_after_prior_line_object_keeps_its_line_spacing() {
    for case in [
        // 앞 표(컨트롤 0)는 2쪽 끝 줄 0, 제목 표(컨트롤 2)는 3쪽 줄 1.
        Case {
            sample: "samples/issue8418/30026-safety-reform-toc.hwp",
            page_count: 37,
            page: 2,
            host: 30,
            table_control: 2,
            follower: 32,
            follower_text: "추진",
        },
        // 같은 문단 줄 0의 TAC 그림 두 개, 줄 1의 TAC 표.
        Case {
            sample: "samples/issue6737/156487948-marine-pollution-law.hwp",
            page_count: 2,
            page: 0,
            host: 0,
            table_control: 4,
            follower: 1,
            follower_text: "해양경찰청",
        },
    ] {
        verify(&case);
    }
}
