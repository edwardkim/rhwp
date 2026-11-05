//! 관계형·고정형(쪽수·독립 Print 2쪽 그림 경계 ±1px): PR #8477의 정식 회귀.
//! 주석 갱신: 2026-11-04T16:28:23+09:00; 직전 파일 변경: 2026-11-03T19:40:21+09:00 (`76ceb2574cb09aa510b04759a7fddc8a92f10298`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `40778b67d6f69ccd3ee86cde84818dc2deea2cff`.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8472/cell_picture_row_split.hwp`; 독립 Print `pdf/issue8472_cell_picture_row_split-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 100.00% / fresh WASM 99.99%.
//! 1쪽 raw: Native 100.00% / fresh WASM 99.99%; 2쪽은 양쪽 100.00%.
//! 두 쪽의 라벨·그림·외곽선을 직접 확대 판독했다. 관계 검사를 유지하며 2쪽 그림만 ±1 CSSpx로 보호한다.
//! 원본·PDF SHA, 전체 TSV·review/overlay·실제 빌드 해시: `mydocs/pr/pr_8477_review.md`.
//! 최소 글꼴에서도 같은 이미지 경계를 확인했다. 설치 OTF 또는 특정 본문 글꼴에 의존하지 않는다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//!
//! 1. 본문 직속 비-TAC 표의 칸 높이는 저장 사다리(다음 문단 vpos = 앵커 vpos + 그림 높이)를 따른다.
//!    그림을 단 빈 문단의 줄 전진을 그림 위에 한 번 더 쌓지 않는다.
//! 2. 그런 행을 쪽 끝에서 나눌 때 그림을 단 문단의 글줄과 그림 몫은 한 조각에 함께 간다.
//!
//! - 갱신: 2026-11-02 (task/8472-cell-picture-anchor)
//! - 입력: `samples/issue8472/cell_picture_row_split.hwp` — 1382000-202100018 종합보고서를 한/글 2022 로
//!   HWPX 저장 → 문단 0·178·179·179·180·182·183 만 남김 → 마지막 차트 표 두 칸의 그림 앞에 라벨
//!   줄 2개 복제(`samples/issue8472/gen_picrow.py`, fills=0 before=2 after=0) → 한/글 2022 가 HWP 로
//!   다시 저장(저장 줄은 한/글이 조판).
//! - 독립 기준: 한/글 2022(12.0.0.4547) Print `pdf/issue8472_cell_picture_row_split-2020.pdf` —
//!   1쪽 끝에 둘째 표 라벨 «[ 남성 ]»·«[ 여성 ]» 각 3줄, 2쪽 첫머리에 그 두 그림.
//! - 측정(code 06cbf26801, `silhouette_raw_match_percent` 1/2쪽): Native 90.21/100 · fresh WASM
//!   90.55/100 — 최저 90.21(1쪽). 1쪽 남은 차이: 한/글은 뒤 표 첫 조각 상자를 쪽 바닥까지 닫고
//!   라벨을 그 안에서 세로 가운데 정렬하는데, rhwp 는 라벨 높이로만 닫는다(미해결, #8472 비범위).
//!   수정 전 Native 92.00/70.61(2쪽 그림이 조각 밖).
//!   증거 `mydocs/pr/assets/issue_8472/{native,wasm}/fixture.tsv`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8472/cell_picture_row_split.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 쪽 `page` 에서 문단 `para` 가 소유한 표 노드들.
fn tables_of<'a>(nodes: &[&'a RenderNode], para: usize) -> Vec<&'a RenderNode> {
    nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(para) && t.cell_context.is_none())
        })
        .collect()
}

fn subtree_text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn subtree_images(node: &RenderNode) -> usize {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .count()
}

/// 본문의 1×2 차트 표를 단 문단 번호(앞 표, 뒤 표).
fn chart_paras(core: &DocumentCore) -> (usize, usize) {
    let paras: Vec<usize> = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.controls.iter().any(|c| {
                matches!(c, Control::Table(t) if t.row_count == 1 && t.col_count == 2
                    && !t.common.treat_as_char)
            })
        })
        .map(|(i, _)| i)
        .collect();
    assert_eq!(paras.len(), 2, "차트 표 두 개: {paras:?}");
    (paras[0], paras[1])
}

#[test]
fn opening_picture_row_frame_preserves_its_declared_cell_height() {
    let core = core();
    let (_, last) = chart_paras(&core);
    let source = core.document().sections[0].paragraphs[last]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("뒤 차트 표 원본");
    let declared = source.cells.iter().map(|cell| cell.height).max().unwrap() as f64 / 75.0;
    let tree = core.build_page_render_tree(0).expect("첫 조각 쪽");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let table = tables_of(&nodes, last);
    assert_eq!(table.len(), 1, "1쪽 끝에 뒤 차트 표의 첫 조각");
    let table = table[0];
    let body = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문 상자");
    let remaining = body.bbox.y + body.bbox.height - table.bbox.y;
    let expected = declared.min(remaining);
    assert!(
        (table.bbox.height - expected).abs() <= 1.0,
        "독립 Print의 첫 분할 상자는 원본 선언을 남은 본문 한도에서 보존한다: 실제 {}, 원본 {}, 남은 본문 {}",
        table.bbox.height,
        declared,
        remaining
    );
    let text = subtree_text(table);
    assert_eq!(text.matches("남성").count(), 3, "첫 조각 남성 라벨 보존");
    assert_eq!(text.matches("여성").count(), 3, "첫 조각 여성 라벨 보존");
    assert_eq!(subtree_images(table), 0, "그림 유닛은 다음 조각이 소유");
    let mut selected = Vec::new();
    walk(table, &mut selected);
    for label in ["남성", "여성"] {
        let labels: Vec<_> = selected
            .iter()
            .filter(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(label)))
            .collect();
        assert_eq!(labels.len(), 3, "각 칸의 앞 내용만 정렬한다: {label}");
        let top = labels
            .iter()
            .map(|node| node.bbox.y)
            .fold(f64::INFINITY, f64::min);
        let bottom = labels
            .iter()
            .map(|node| node.bbox.y + node.bbox.height)
            .fold(f64::NEG_INFINITY, f64::max);
        let line = labels
            .iter()
            .map(|node| node.bbox.height)
            .fold(0.0, f64::max);
        let before = top - table.bbox.y;
        let after = table.bbox.y + table.bbox.height - bottom;
        assert!(
            before > 0.0 && after > 0.0,
            "라벨은 첫 조각 안에 있다: {label}"
        );
        let owner = source
            .cells
            .iter()
            .find(|cell| {
                cell.paragraphs
                    .iter()
                    .any(|paragraph| paragraph.text.contains(label))
            })
            .expect("라벨 원본 셀");
        assert_eq!(
            owner.vertical_align,
            rhwp::model::table::VerticalAlign::Center
        );
        let label_lines: Vec<_> = owner
            .paragraphs
            .iter()
            .filter(|paragraph| paragraph.text.contains(label))
            .flat_map(|paragraph| &paragraph.line_segs)
            .collect();
        let anchor = owner
            .paragraphs
            .iter()
            .find(|paragraph| {
                paragraph
                    .controls
                    .iter()
                    .any(|control| matches!(control, Control::Picture(_)))
            })
            .expect("이월된 원본 그림 앵커");
        let anchor_line = anchor.line_segs.last().expect("저장된 빈 앵커 줄");
        let first_line = label_lines.first().expect("원본 첫 라벨 줄");
        let last_line = label_lines.last().expect("원본 마지막 라벨 줄");
        // 독립 Print는 첫 물리 상자를 닫을 때 그림 앞 빈 앵커 줄까지 정렬한다.
        // 원본 저장 좌표 차이를 쓰며 특정 쪽의 절대 픽셀 위치를 고정하지 않는다.
        let source_offset = f64::from(
            first_line.vertical_pos + last_line.vertical_pos + last_line.line_height
                - anchor_line.vertical_pos
                - anchor_line.line_height,
        ) / 150.0;
        let actual_offset = (top + bottom) / 2.0 - (table.bbox.y + table.bbox.height / 2.0);
        assert!(
            (actual_offset - source_offset).abs() <= line / 10.0,
            "원본 줄 상자의 가운데 정렬 관계: {label}, 실제 중심차 {actual_offset}, 저장 중심차 {source_offset}"
        );
    }
}

#[test]
fn whole_picture_cells_center_their_own_saved_content_frame() {
    let core = core();
    let (first, _) = chart_paras(&core);
    let table = core.document().sections[0].paragraphs[first]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table)
            } else {
                None
            }
        })
        .expect("온전한 앞 차트 표");
    let mut source: Vec<_> = table
        .cells
        .iter()
        .map(|cell| {
            let anchor = cell
                .paragraphs
                .iter()
                .find(|paragraph| {
                    paragraph
                        .controls
                        .iter()
                        .any(|control| matches!(control, Control::Picture(_)))
                })
                .expect("저장된 그림 앵커");
            let top = anchor.line_segs.first().expect("그림 원본 줄").vertical_pos;
            let end = cell
                .paragraphs
                .iter()
                .flat_map(|paragraph| &paragraph.line_segs)
                .map(|line| line.vertical_pos + line.line_height)
                .max()
                .expect("저장 내용 끝");
            (cell.col, top, end)
        })
        .collect();
    source.sort_by_key(|&(column, _, _)| column);
    let tree = core.build_page_render_tree(0).expect("온전한 표의 쪽");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let rendered = tables_of(&nodes, first);
    assert_eq!(rendered.len(), 1);
    let mut children = Vec::new();
    walk(rendered[0], &mut children);
    let mut images: Vec<_> = children
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
        .collect();
    images.sort_by(|left, right| left.bbox.x.total_cmp(&right.bbox.x));
    assert_eq!(images.len(), 2);
    // 독립 Print에서 두 그림의 위변은 같은 물리 행의 서로 다른 셀 점유를
    // 가운데 정렬한 관계다. 공통 행 높이가 각 칸의 내용 높이를 대신하지 않는다.
    let expected_delta =
        f64::from(source[1].1 - source[0].1) / 75.0 - f64::from(source[1].2 - source[0].2) / 150.0;
    let actual_delta = images[1].bbox.y - images[0].bbox.y;
    assert!((actual_delta - expected_delta).abs() <= 2.0 / 75.0,
        "각 셀의 저장 프레임으로 그림 정렬: 실제 상대차 {actual_delta}, 원본 상대차 {expected_delta}");
}

#[test]
fn continuation_pictures_center_their_owned_content_without_readding_anchor_lines() {
    let core = core();
    let (_, last) = chart_paras(&core);
    let table = core.document().sections[0].paragraphs[last]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table)
            } else {
                None
            }
        })
        .expect("이어받는 원본 차트 표");
    let mut heights: Vec<_> = table
        .cells
        .iter()
        .map(|cell| {
            let picture = cell
                .paragraphs
                .iter()
                .flat_map(|paragraph| &paragraph.controls)
                .find_map(|control| {
                    if let Control::Picture(picture) = control {
                        Some(picture)
                    } else {
                        None
                    }
                })
                .expect("각 칸이 소유한 원본 그림");
            (cell.col, picture.common.height)
        })
        .collect();
    heights.sort_by_key(|&(column, _)| column);
    let tree = core
        .build_page_render_tree(1)
        .expect("그림을 소유한 이어받기 쪽");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let rendered = tables_of(&nodes, last);
    assert_eq!(rendered.len(), 1);
    let mut children = Vec::new();
    walk(rendered[0], &mut children);
    let mut images: Vec<_> = children
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
        .collect();
    images.sort_by(|left, right| left.bbox.x.total_cmp(&right.bbox.x));
    assert_eq!(images.len(), 2);
    // 독립 Print에서 높이가 작은 왼쪽 그림이 같은 행 안에서 더 아래에 있다.
    // 소유한 그림 높이 차이의 절반이 위변 차이다. 빈 앵커 줄은 그림과 겹친다.
    let expected = -(f64::from(heights[1].1) - f64::from(heights[0].1)) / 150.0;
    let actual = images[1].bbox.y - images[0].bbox.y;
    assert!(
        (actual - expected).abs() <= 2.0 / 75.0,
        "이어받기 그림의 원본 높이·정렬 관계: 실제 상대차 {actual}, 원본 상대차 {expected}"
    );
}

#[test]
fn continuation_picture_bounds_match_independent_print_on_completed_page() {
    let core = core();
    let (_, last) = chart_paras(&core);
    let tree = core.build_page_render_tree(1).expect("독립 Print 2쪽");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables = tables_of(&nodes, last);
    assert_eq!(tables.len(), 1, "이어받은 표가 그림을 소유한다");
    let mut children = Vec::new();
    walk(tables[0], &mut children);
    let mut images: Vec<_> = children
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
        .collect();
    images.sort_by(|left, right| left.bbox.x.total_cmp(&right.bbox.x));
    assert_eq!(images.len(), 2);
    // 같은 PDF의 실제 이미지 사각형을 points × 96/72로 환산했다.
    // Native/fresh WASM 2쪽 모두 100.00%이며 그림의 소유·상대 정렬 검사도 유지한다.
    let print_bounds = [
        [84.1146647, 91.5800374, 387.7919922, 245.9707031],
        [401.5439860, 90.4613444, 705.2213542, 247.4093424],
    ];
    for (image, expected) in images.into_iter().zip(print_bounds) {
        let bounds = [
            image.bbox.x,
            image.bbox.y,
            image.bbox.x + image.bbox.width,
            image.bbox.y + image.bbox.height,
        ];
        for (axis, (actual, reference)) in bounds.into_iter().zip(expected).enumerate() {
            assert!(
                (actual - reference).abs() <= 1.0,
                "완료 페이지의 독립 Print 그림 경계 ±1 CSSpx: 축={axis}, 실제={actual}, 기준={reference}"
            );
        }
    }
}

#[test]
fn chart_cell_height_follows_stored_ladder() {
    let core = core();
    let (first, _) = chart_paras(&core);
    let para = &core.document().sections[0].paragraphs[first];
    let table = para
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .unwrap();
    // 저장 사다리 끝 + 칸 위아래 여백(HU) — 한/글이 저장한 내용 상자.
    let stored_hu = table
        .cells
        .iter()
        .map(|cell| {
            let end = cell
                .paragraphs
                .iter()
                .flat_map(|p| p.line_segs.iter())
                .map(|seg| seg.vertical_pos + seg.line_height)
                .max()
                .unwrap_or(0);
            end + i32::from(cell.padding.top) + i32::from(cell.padding.bottom)
        })
        .max()
        .unwrap();
    let stored_px = stored_hu as f64 / 75.0;
    let tree = core.build_page_render_tree(0).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables = tables_of(&nodes, first);
    assert_eq!(tables.len(), 1, "앞 차트 표는 1쪽에 통째로");
    let painted = tables[0].bbox.height;
    assert!(
        (painted - stored_px).abs() <= 1.0,
        "앞 차트 표 높이 {painted:.1}px 는 저장 내용 상자 {stored_px:.1}px 와 같아야 한다(앵커 줄 이중 계상 금지)"
    );
}

#[test]
fn picture_stays_with_its_anchor_paragraph_across_page_split() {
    let core = core();
    let (_, second) = chart_paras(&core);
    assert_eq!(core.page_count(), 2, "한/글 Print 쪽수 2");
    let page_tables = |page: u32| {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        tables_of(&nodes, second)
            .iter()
            .map(|t| (subtree_text(t), subtree_images(t)))
            .collect::<Vec<_>>()
    };
    let first_page = page_tables(0);
    let second_page = page_tables(1);
    assert_eq!(first_page.len(), 1, "뒤 차트 표의 첫 조각은 1쪽 끝");
    assert_eq!(second_page.len(), 1, "이어지는 조각은 2쪽");
    let (text0, images0) = &first_page[0];
    let (text1, images1) = &second_page[0];
    assert_eq!(
        text0.matches("남성").count(),
        3,
        "1쪽에 라벨 «[ 남성 ]» 3줄: {text0}"
    );
    assert_eq!(
        text0.matches("여성").count(),
        3,
        "1쪽에 라벨 «[ 여성 ]» 3줄: {text0}"
    );
    assert_eq!(
        *images0, 0,
        "그림은 앵커 문단과 함께 2쪽으로 간다(1쪽에 그림 없음)"
    );
    assert_eq!(*images1, 2, "2쪽 조각이 두 그림을 소유한다");
    assert!(
        !text1.contains("남성") && !text1.contains("여성"),
        "라벨 줄은 다시 그리지 않는다: {text1}"
    );

    // 그림은 앵커 문단과 같은 조각의 칸 상자 안에 있다 — 앵커 줄을 앞 조각에 두면 그림이
    // 이어지는 조각의 위변 밖으로 올라가고 조각 상자가 그림보다 작아진다.
    let tree = core.build_page_render_tree(1).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let table = tables_of(&nodes, second)[0];
    let mut inside = Vec::new();
    walk(table, &mut inside);
    let top = table.bbox.y;
    let bottom = table.bbox.y + table.bbox.height;
    for image in inside
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
    {
        let (y0, y1) = (image.bbox.y, image.bbox.y + image.bbox.height);
        assert!(
            y0 >= top - 0.01 && y1 <= bottom + 0.01,
            "그림 {y0:.1}..{y1:.1} 은 이어지는 표 조각 {top:.1}..{bottom:.1} 안에 있어야 한다"
        );
    }
}
