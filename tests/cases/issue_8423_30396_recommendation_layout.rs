//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8431의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:38+09:00 (`44fd851032ba50ee356103f71d9139512567292b`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8423/30396-childcare-leave-recommendation.hwp`; 독립 Print `pdf/issue8423/30396-childcare-leave-recommendation-2022.pdf`.
//! 전체 25쪽, 전쪽 최저 Native 98.66% / fresh WASM 98.68%.
//! 주장 9쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8431_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-01T18:21:58+09:00 (새 파일).
//! 2026-11-01 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준 pdf/issue8423/30396-childcare-leave-recommendation-2022.pdf(한컴 2022 12.0.0.4547 Print, 25쪽):
//! - 9쪽 Native 98.43% / fresh WASM 98.61%(수정 전 84.78% / 84.47%).
//! - 18쪽 Native 99.90% / fresh WASM 96.98%(수정 전 85.04% / 82.85%).
//! - 전 25쪽 최저 Native 98.43% / fresh WASM 87.13%(1쪽, 수정 전과 같은 값 — WASM 가짜 굵게 #8430).
//!
//! 증거: mydocs/pr/assets/issue_8423/{native,wasm}/*_silhouette.tsv.
//! 취소선 판정은 한/글 2020 의 HWP→HWPX 변환 결과(5.0.2.4 → NONE)와 독립 Print 를 근거로 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue8423/30396-childcare-leave-recommendation.hwp";

fn hu_to_px(hu: i64) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn load() -> (Vec<u8>, DocumentCore) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 25, "독립 한/글 2022 Print 전체 쪽수");
    (bytes, core)
}

/// 표 안으로 내려가지 않고 본문 흐름의 노드만 모은다.
fn flow_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    for child in &node.children {
        flow_nodes(child, out);
    }
}

#[test]
fn v5024_strike_bit_one_does_not_draw_strikethrough() {
    let (_, core) = load();
    // 9쪽 표 숫자(50·94·17·80)를 소유한 글자 모양은 비트 1·모양 0 이다.
    let shapes = &core.document().doc_info.char_shapes;
    for id in [75usize, 215, 227, 229] {
        assert_eq!((shapes[id].attr >> 18) & 0x07, 1, "원본 attr 보존: {id}");
        assert!(!shapes[id].strikethrough, "5.0.2.4 의 비트 값 1: {id}");
    }
}

/// 반례: 같은 비트·모양 조합이라도 5.1.1.0 파일(한/글 HWPX 변환 `SOLID`)은 실제 취소선이다.
#[test]
fn later_version_strike_bit_one_keeps_strikethrough() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue1949_giant_cell_nested_tables_perf.hwp"),
    )
    .expect("5.1.1.0 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 파싱");
    let shapes = &core.document().doc_info.char_shapes;
    for id in [33usize, 36] {
        assert_eq!((shapes[id].attr >> 18) & 0x07, 1);
        assert_eq!((shapes[id].attr >> 26) & 0x0F, 0);
        assert!(shapes[id].strikethrough, "5.1.1.0 의 실선 취소선: {id}");
    }
}

#[test]
fn host_line_below_non_tac_table_advances_once() {
    let (bytes, core) = load();
    let source = rhwp::parser::parse_document(&bytes).expect("저장 줄 파싱");
    let paragraphs = &source.sections[0].paragraphs;
    let host = 159;
    let Some(Control::Table(table)) = paragraphs[host].controls.first() else {
        panic!("문단 159 의 상자 표");
    };
    assert!(!table.common.treat_as_char);
    let previous = paragraphs[host - 1].line_segs.last().expect("앞 문단 줄");
    let previous_end =
        i64::from(previous.vertical_pos + previous.line_height + previous.line_spacing);
    let host_line = &paragraphs[host].line_segs[0];
    assert!(
        i64::from(host_line.vertical_pos) - previous_end >= i64::from(table.common.height),
        "전제: host 줄은 표 아래에 저장돼 있다"
    );
    let follower = &paragraphs[host + 1];
    let expected = hu_to_px(i64::from(follower.line_segs[0].vertical_pos) - previous_end);

    let page = core.build_page_render_tree(8).expect("9쪽");
    let mut nodes = Vec::new();
    flow_nodes(&page.root, &mut nodes);
    let table_node = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(host) && t.control_index == Some(0))
        })
        .expect("같은 쪽의 상자 표");
    let follower_line = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.para_index == Some(host + 1) && line.line_index == Some(0))
        })
        .expect("표 뒤 다음 문단 첫 줄");
    // 표 원점은 문단 시작(앞 문단 저장 끝)이며 바깥 여백만큼 차이가 날 수 있다.
    let actual = follower_line.bbox.y - table_node.bbox.y;
    assert!(
        (actual - expected).abs() <= 2.5,
        "표 원점→다음 문단 {actual:.2}px, 저장 사다리 {expected:.2}px"
    );
}

fn find_image_cell<'a>(
    node: &'a RenderNode,
    cell: Option<&'a RenderNode>,
    out: &mut Vec<(&'a RenderNode, &'a RenderNode)>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if let (RenderNodeType::Image(_), Some(cell)) = (&node.node_type, cell) {
        out.push((node, cell));
    }
    for child in &node.children {
        find_image_cell(child, cell, out);
    }
}

#[test]
fn para_relative_cell_picture_starts_before_paragraph_spacing() {
    let (bytes, core) = load();
    let source = rhwp::parser::parse_document(&bytes).expect("저장 줄 파싱");
    let host = &source.sections[0].paragraphs[328];
    let Some(Control::Table(table)) = host.controls.first() else {
        panic!("문단 328 의 그림 표");
    };
    let cell = table.cells.last().expect("그림 셀");
    let cell_para = &cell.paragraphs[0];
    let Some(Control::Picture(picture)) = cell_para.controls.first() else {
        panic!("셀 문단의 그림");
    };
    assert!(!picture.common.treat_as_char);
    assert!(
        cell_para.line_segs[0].vertical_pos > 0,
        "전제: 셀 문단은 앞 간격 뒤에 줄을 둔다"
    );

    let page = core.build_page_render_tree(17).expect("18쪽");
    let mut found = Vec::new();
    find_image_cell(&page.root, None, &mut found);
    let (image, cell_node) = found
        .iter()
        .find(|(image, _)| {
            (image.bbox.height - hu_to_px(i64::from(picture.common.height))).abs() <= 1.0
        })
        .expect("셀 안 그림");
    // 셀은 세로 가운데 정렬이고 내용은 그림 하나다. 문단 앞 간격이 그림 위에 끼지 않으면
    // 그림 위·아래의 남는 공간이 같다(앞 간격이 끼면 위가 그만큼 크다).
    assert_eq!(
        cell.vertical_align,
        rhwp::model::table::VerticalAlign::Center
    );
    let above = image.bbox.y - (cell_node.bbox.y + hu_to_px(i64::from(cell.padding.top)));
    let below = (cell_node.bbox.y + cell_node.bbox.height
        - hu_to_px(i64::from(cell.padding.bottom)))
        - (image.bbox.y + image.bbox.height);
    assert!(
        (above - below).abs() <= 1.0,
        "그림 위 여유 {above:.2}px, 아래 여유 {below:.2}px"
    );
    // 행은 그림 흐름 높이만 차지하고 문단 앞 간격으로 늘지 않는다 — 표 높이는 선언값과 같다.
    let mut nodes = Vec::new();
    flow_nodes(&page.root, &mut nodes);
    let table_node = nodes
        .iter()
        .find(
            |node| matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(328)),
        )
        .expect("같은 쪽의 표");
    assert!(
        (table_node.bbox.height - hu_to_px(i64::from(table.common.height))).abs() <= 1.0,
        "표 높이 {:.2}px, 선언 {:.2}px",
        table_node.bbox.height,
        hu_to_px(i64::from(table.common.height))
    );
}
