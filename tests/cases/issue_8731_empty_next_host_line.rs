//! 관계형.
//! 주석 갱신: MEASURE_PLACEHOLDER
//!
//! [Issue #8731] 자리차지 개체를 매단 빈 앵커 문단 뒤가 **개체 없는 빈 문단**이면 띠 아래
//! host 줄이 흐름에서 빠져 뒤 본문 전체가 그 줄만큼 위로 붙는다. 저장 사다리
//! (`다음 vpos - host vpos = lh + ls`)는 뒤 문단의 글자 유무와 관계없이 같은 증거다.
//! 개체를 가진 빈 뒤 문단(앵커 스택)은 개체-개체 간격이라 계속 제외한다.
//!
//! 가드하는 축:
//!   ① 실제 보도자료 2건: 첫 본문 줄 = 띠 아래끝 + 바깥 여백 + host 줄 + 빈 문단 줄
//!   ② 줄 폭이 남은 host 는 음수 오프셋에서도 띠 아래 줄을 더하지만, 폭 0 host 는 더하지 않는다
//!   ③ 뒤 문단이 개체를 가지면(앵커 스택) 같은 사다리여도 띠 아래 줄을 더하지 않는다
//!   ④ 리셋 앞 TAC 줄은 줄 높이까지만 쪽에 들어가면 남는다(마지막 줄간격을 세지 않는다)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 렌더 트리 소수 반올림과 저장 HWPUNIT 정수를 덮는 관계 허용치(px).
const TOL_PX: f64 = 0.6;

fn hu_px(hu: i32) -> f64 {
    f64::from(hu) / 75.0
}

fn open(path: &str) -> DocumentCore {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(&full).expect("read sample")).expect("open")
}

fn table_bottom(node: &RenderNode, para: usize) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Table(table) if table.para_index == Some(para) => {
            Some(node.bbox.y + node.bbox.height)
        }
        _ => None,
    };
    node.children
        .iter()
        .filter_map(|child| table_bottom(child, para))
        .chain(own)
        .reduce(f64::max)
}

fn table_top(node: &RenderNode, para: usize) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Table(table) if table.para_index == Some(para) => Some(node.bbox.y),
        _ => None,
    };
    node.children
        .iter()
        .filter_map(|child| table_top(child, para))
        .chain(own)
        .reduce(f64::min)
}

fn line_top(node: &RenderNode, para: usize) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None; // 셀 안의 줄은 본문 문단이 아니다
    }
    let own = match &node.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para) => Some(node.bbox.y),
        _ => None,
    };
    node.children
        .iter()
        .filter_map(|child| line_top(child, para))
        .chain(own)
        .reduce(f64::min)
}

/// 앵커 문단 0 · 빈 문단 1 · 첫 본문 문단 2 구성의 실제 문서.
fn assert_body_keeps_host_line(path: &str) {
    let core = open(path);
    let paras = &core.document().sections[0].paragraphs;
    assert!(paras[0].text.is_empty(), "{path}: 전제 — 빈 앵커 문단");
    assert!(
        paras[1].text.is_empty() && paras[1].controls.is_empty(),
        "{path}: 전제 — 개체 없는 빈 문단"
    );
    let (host, empty) = (&paras[0].line_segs[0], &paras[1].line_segs[0]);
    let host_advance = host.line_height + host.line_spacing;
    assert_eq!(
        empty.vertical_pos - host.vertical_pos,
        host_advance,
        "{path}: 전제 — 저장 사다리가 host 줄 하나를 증언한다"
    );
    let empty_advance = paras[2].line_segs[0].vertical_pos - empty.vertical_pos;
    let Control::Table(last) = paras[0].controls.last().expect("앵커 표") else {
        panic!("{path}: 전제 — 마지막 개체는 표");
    };

    let page = core.build_page_render_tree(0).expect("page");
    let band = table_bottom(&page.root, 0).expect("띠");
    let body = line_top(&page.root, 2).expect("첫 본문 줄");
    let expected = band
        + hu_px(i32::from(last.outer_margin_bottom))
        + hu_px(host_advance)
        + hu_px(empty_advance);
    assert!(
        (body - expected).abs() <= TOL_PX,
        "{path}: 첫 본문 줄 {body:.1}px 이 띠 아래 host 줄·빈 문단 줄 뒤 {expected:.1}px 과 다르다"
    );
}

#[test]
fn issue_8731_hwpx_body_keeps_host_line_after_empty_next() {
    assert_body_keeps_host_line("samples/issue8731/156641865_press_release_band.hwpx");
}

#[test]
fn issue_8731_hwp_body_keeps_host_line_after_empty_next() {
    assert_body_keeps_host_line("samples/issue8731/156713639_press_release_band.hwp");
}

const HOST: usize = 1;

/// 음수 오프셋 앵커의 사다리를 `lh + ls` 로 맞춘 합성 변형. 한컴 저장 문서가 아니라 알고리즘
/// 경계 검사다. 한컴 저장 형상의 근거는 413쪽 보고서의 폭 0·오프셋 -189 앵커(6쪽 raw 99.9)다.
/// 반환값은 띠 아래끝 + 바깥 여백에서 뒤 빈 문단 줄 윗변까지의 거리(px).
fn gap_below_band_with_host_width(segment_width: u32) -> f64 {
    let mut core = open("samples/issue7621/control-float.hwp");
    let mut doc = core.document().clone();
    let paras = &mut doc.sections[0].paragraphs;
    let host = &mut paras[HOST];
    host.line_segs[0].segment_width = segment_width as i32;
    let Control::Table(table) = &mut host.controls[0] else {
        panic!("전제: 표 문단");
    };
    table.common.vertical_offset = (-189i32) as u32;
    let margin = hu_px(i32::from(table.outer_margin_bottom));
    let seg = host.line_segs[0].clone();
    assert_eq!(
        paras[HOST + 1].line_segs[0].vertical_pos - seg.vertical_pos,
        seg.line_height + seg.line_spacing,
        "전제 — 저장 사다리가 host 줄 하나를 증언한다"
    );
    core.set_document(doc);
    let page = core.build_page_render_tree(0).expect("page");
    let band = table_bottom(&page.root, HOST).expect("띠");
    line_top(&page.root, HOST + 1).expect("뒤 빈 문단 줄") - (band + margin)
}

/// 뒤 문단에 원본 표를 복제해 앵커 스택을 만든 합성 변형. 반환값은 뒤 앵커 표의 윗변(px).
fn anchor_stack_next_top(host_width: u32) -> f64 {
    let mut core = open("samples/issue7621/control-float.hwp");
    let mut doc = core.document().clone();
    let paras = &mut doc.sections[0].paragraphs;
    let next_table = paras[HOST].controls[0].clone();
    paras[HOST + 1].controls = vec![next_table];
    let host = &mut paras[HOST];
    host.line_segs[0].segment_width = host_width as i32;
    let Control::Table(table) = &mut host.controls[0] else {
        panic!("전제: 표 문단");
    };
    table.common.vertical_offset = (-189i32) as u32;
    let seg = host.line_segs[0].clone();
    assert_eq!(
        paras[HOST + 1].line_segs[0].vertical_pos - seg.vertical_pos,
        seg.line_height + seg.line_spacing,
        "전제 — 저장 사다리가 host 줄 하나 높이와 같다"
    );
    core.set_document(doc);
    let page = core.build_page_render_tree(0).expect("page");
    table_top(&page.root, HOST + 1).expect("뒤 앵커 표")
}

#[test]
fn issue_8731_anchor_stack_next_adds_no_line_below_band() {
    // 폭 0 host 는 띠 아래 줄이 없는 기준이다. 뒤가 앵커 스택이면 폭이 남은 host 도 사다리를
    // 띠 아래 줄의 증거로 쓰지 않아 뒤 표가 같은 자리에 와야 한다.
    let blocked = anchor_stack_next_top(0);
    let open_host = anchor_stack_next_top(1000);
    assert!(
        (open_host - blocked).abs() <= TOL_PX,
        "앵커 스택의 사다리는 띠 아래 줄의 증거가 아니다: 뒤 표 {blocked:.1} → {open_host:.1}px"
    );
}

#[test]
fn issue_8731_zero_width_host_adds_no_line_below_band() {
    let seg = open("samples/issue7621/control-float.hwp")
        .document()
        .sections[0]
        .paragraphs[HOST]
        .line_segs[0]
        .clone();
    let line = hu_px(seg.line_height + seg.line_spacing);
    let blocked = gap_below_band_with_host_width(0);
    let open_host = gap_below_band_with_host_width(1000);
    assert!(
        blocked <= TOL_PX,
        "폭 0 host 는 띠 아래 줄을 더하지 않아야 한다: 간격 {blocked:.1}px"
    );
    assert!(
        (open_host - line).abs() <= TOL_PX,
        "폭이 남은 host 는 띠 아래 줄 {line:.1}px 을 더해야 한다: 간격 {open_host:.1}px"
    );
}

/// 글자처럼 그림 줄 · 저장 쪽 리셋 · 글자처럼 표 줄 문단을 쪽 끝에 둔 합성 변형(#2311 형상).
/// 그림 줄 아래끝은 본문 안이고 줄간격까지 더해야 본문을 넘는다.
fn reset_paragraph_at_page_bottom() -> DocumentCore {
    let mut core = open("samples/issue7621/control-float.hwp");
    let mut doc = core.document().clone();
    let body_hu = {
        let def = &doc.sections[0].section_def.page_def;
        (def.height - def.margin_top - def.margin_bottom - def.margin_header - def.margin_footer)
            as i32
    };
    let (line_height, line_spacing) = (3721, 376);
    let paras = &mut doc.sections[0].paragraphs;
    paras.truncate(HOST + 1);
    let Control::Table(source) = paras[HOST].controls[0].clone() else {
        panic!("전제: 표 문단");
    };
    let mut table = source;
    table.common.treat_as_char = true;
    table.attr |= 0x01;
    let mut picture = Picture::default();
    picture.common.treat_as_char = true;
    picture.common.width = 30000;
    picture.common.height = line_height as u32;

    let mut filler = paras[0].clone();
    let filler_height = body_hu - line_height - line_spacing / 2;
    filler.line_segs.truncate(1);
    filler.line_segs[0].vertical_pos = 0;
    filler.line_segs[0].line_height = filler_height;
    filler.line_segs[0].text_height = filler_height;
    filler.line_segs[0].line_spacing = 0;

    let mut host = paras[HOST].clone();
    let height = table.common.height as i32;
    host.controls = vec![Control::Picture(Box::new(picture)), Control::Table(table)];
    host.ctrl_data_records = vec![None, None];
    let segment = |vertical_pos: i32, line_height: i32| LineSeg {
        vertical_pos,
        line_height,
        text_height: line_height,
        line_spacing,
        ..host.line_segs[0].clone()
    };
    host.line_segs = vec![segment(filler_height, line_height), segment(0, height)];
    *paras = vec![filler, host];
    core.set_document(doc);
    core
}

fn has_image(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Image(_)) || node.children.iter().any(has_image)
}

#[test]
fn issue_8731_tac_line_before_reset_fits_by_line_height() {
    let core = reset_paragraph_at_page_bottom();
    assert_eq!(
        core.page_count(),
        2,
        "그림 줄 쪽과 리셋 뒤 표 쪽만 있어야 한다"
    );
    let first = core.build_page_render_tree(0).expect("page");
    assert!(
        has_image(&first.root),
        "리셋 앞 그림 줄은 앞 쪽에 남아야 한다"
    );
    let second = core.build_page_render_tree(1).expect("page");
    assert!(
        table_bottom(&second.root, HOST).is_some(),
        "리셋 뒤 표는 다음 쪽 소속이어야 한다"
    );
}
