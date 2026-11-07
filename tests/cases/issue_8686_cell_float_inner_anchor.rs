//! [#8686] 관계형·고정형 회귀 — 칸 안 문단 기준 자리차지(글 위아래) 표의 제어문자가 글
//! 중간에 있어도 세로 오프셋은 제어문자가 속한 저장 줄에서 잰다. 저장 호스트 줄이 모두 표
//! 띠 위에서 끝나면 표는 그 줄들 아래에 놓이고 호스트 글줄과 겹치지 않는다.
//!
//! 주석 갱신: 2026-11-07 09:30 (KST) — 측정값은 아래 표
//!
//! - 입력: `samples/issue8686/cell_float_inner_anchor.hwp` — 합천군 교복구입비 지원 신청서
//!   (공개 조례 서식, HWP, 1쪽) 그대로. 동의서 칸(r=14)의 문단 5 는 저장 줄 둘(「… 교복구입비」,
//!   「수집·이용하는 데 동의하십니까?」)과, 글 중간 제어문자로 묶인 1×5 「동의/미동의」 표
//!   (글자처럼 취급 아님, 글 위아래, 문단 기준, 위 정렬)를 담는다.
//! - 독립 기준: 같은 입력의 한/글 2022 Print PDF `pdf/issue8686_cell_float_inner_anchor-2020.pdf`
//!   (1쪽). 「동의/미동의」 표가 「동의하십니까?」 줄 아래에 놓인다.
//! - 수정 전 rhwp(task/8686 1단계, 칸 안 저장 프레임 인증이 글 앞·뒤 제어문자만 받음): 표를 호스트
//!   첫 줄 아래에 두어 「동의하십니까?」 줄 위에 겹쳐 그렸다.
//!
//! 측정(2026-11-07, 코드 6641965c69, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | 쪽 | Native | fresh WASM | 수정 전 Native |
//! |---|---|---|---|---|---|
//! | cell_float_inner_anchor | 1 | 1 | 99.95 | 99.97 | 99.00 |
//!
//! 두 경로 전쪽 최저: Native 99.95, fresh WASM 99.97.
//!
//! 증거: `mydocs/pr/assets/issue_8686/{native,native_base,wasm}/cell_float_inner_anchor_silhouette.tsv`,
//! `review/cell_float_inner_anchor_p1_*.png`. 검사는 제어문자가 글 중간에 있고 저장 호스트 줄이
//! 표 띠 위에서 끝난다는 원본 관계(전제), 그린 표가 「동의하십니까?」 줄 아래에 있고 겹치지
//! 않는 관계(관계형), 독립 PDF 전체 쪽수(고정형)다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8686/cell_float_inner_anchor.hwp";

fn node_text(node: &RenderNode) -> String {
    let mut out = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        out.push_str(&node_text(child));
    }
    out
}

fn collect(
    node: &RenderNode,
    lines: &mut Vec<(String, BoundingBox)>,
    tables: &mut Vec<(String, BoundingBox)>,
) {
    match &node.node_type {
        RenderNodeType::TextLine(_) => lines.push((node_text(node), node.bbox.clone())),
        RenderNodeType::Table(_) => tables.push((node_text(node), node.bbox.clone())),
        _ => {}
    }
    for child in &node.children {
        collect(child, lines, tables);
    }
}

/// 「동의하십니까」 문단과 그 문단을 품은 칸.
fn host_paragraph(core: &DocumentCore) -> &Paragraph {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Table(table) => table
                .cells
                .iter()
                .flat_map(|cell| cell.paragraphs.iter())
                .find(|para| para.text.contains("동의하십니까")),
            _ => None,
        })
        .expect("동의 문단")
}

#[test]
fn inner_control_float_table_sits_below_its_host_lines() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), 1, "한컴 Print PDF 1쪽");

    // 전제: 제어문자가 글 중간에 있고, 저장 호스트 줄이 모두 표 띠 위에서 끝난다.
    let host = host_paragraph(&core);
    let (control_index, table) = host
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::Table(table) => Some((index, table.as_ref())),
            _ => None,
        })
        .expect("호스트 문단의 표");
    assert!(!table.common.treat_as_char, "글자처럼 취급하지 않는다");
    assert_eq!(table.common.text_wrap, TextWrap::TopAndBottom, "글 위아래");
    assert_eq!(table.common.vert_rel_to, VertRelTo::Para, "문단 기준");
    let position = host.control_text_positions()[control_index];
    let text_len = host.text.chars().count();
    assert!(
        position > 0 && position < text_len,
        "제어문자 {position} 은 글({text_len}자) 중간에 있다"
    );
    assert!(host.line_segs.len() >= 2, "저장 호스트 줄 둘 이상");
    let anchor_line = host
        .line_segs
        .iter()
        .rev()
        .find(|line| line.text_start as usize <= position)
        .expect("제어문자가 속한 저장 줄");
    let band_top = anchor_line.vertical_pos + table.common.vertical_offset as i32;
    let host_end = host
        .line_segs
        .iter()
        .map(|line| line.vertical_pos + line.line_height)
        .max()
        .unwrap();
    assert!(
        host_end <= band_top,
        "저장 호스트 줄 끝 {host_end} 은 표 띠 위끝 {band_top} 위"
    );

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("1쪽 렌더 트리: {e:?}"));
    let mut lines = Vec::new();
    let mut tables = Vec::new();
    collect(&tree.root, &mut lines, &mut tables);
    let question: Vec<_> = lines
        .iter()
        .filter(|(text, _)| text.contains("동의하십니까"))
        .collect();
    assert_eq!(
        question.len(),
        1,
        "「동의하십니까」 줄은 한 번 그린다: {question:?}"
    );
    let question = &question[0].1;
    let consent = tables
        .iter()
        .filter(|(text, _)| text.contains("미동의"))
        .map(|(_, bbox)| bbox)
        .min_by(|a, b| a.height.total_cmp(&b.height))
        .expect("동의/미동의 표");

    assert!(
        consent.y >= question.y + question.height - 0.5,
        "표 위끝 {:.2} 은 「동의하십니까」 줄 아래끝 {:.2} 아래",
        consent.y,
        question.y + question.height
    );
}
