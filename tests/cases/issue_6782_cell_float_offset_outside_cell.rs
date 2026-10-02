//! [Issue #6782] 칸 앵커 그림의 **음수** 세로 오프셋이 그림을 칸 내용 영역 **위로 통째로**
//! 밀어내면 그 오프셋을 버린다 — 종전에는 그대로 실려 그림이 용지 위쪽 밖(음수 y)으로
//! 나가 인쇄에서 소실됐다.
//!
//! ## 계약 — 좁게 적는다
//!
//! ```text
//!   v_off < 0  그리고  배치결과 y + 그림높이 <= content_top   →  오프셋을 버리고 0 으로 놓는다
//! ```
//!
//! ⚠ **"칸과 겹치는가"라는 일반 계약이 아니다.** Top/Center 의 아래쪽 이탈, Bottom 정렬에서
//! 양수 오프셋이 만드는 위쪽 이탈, 모든 정렬의 아래쪽 완전 이탈은 이 갈래가 다루지 않는다.
//! 오라클이 증명하는 것이 **Center + 음수 + 위쪽 이탈** 하나뿐이라 구현·이름·주석·시험을
//! 그 범위에 맞췄다.
//!
//! 기준선이 실제 칸 상단(`230.2`)이 아니라 padding 뒤 `content_top`(`232.1`)인 것도 의도한
//! 것이다 — 좌표를 만드는 `place()` 가 `content_top` 을 기준점으로 쓰므로 같은 기준으로
//! 판정해야 부호가 뒤집히지 않는다.
//!
//! ## 대상과 오라클
//!
//! `samples/issue6782/…-chemical-labeling-standards.hwp` 77쪽(0-based `76`),
//! 표 `pi=118 ci=0`(14행×4열)의 **`row=4 col=3`** 칸에 매달린 그림 `w=81.1 h=65.8`.
//!
//! ```text
//!   Cell row=4 col=3   x=608.0 y=230.2 w=110.1 h=82.9   content_top=232.1
//!
//!                  y (px)     판정
//!   한/글 2020      235.9     칸 안        ← pdf/…-chemical-labeling-standards-2020.pdf
//!   수정 전        -233.4     용지 밖·소실
//!   수정 후         238.7     칸 안 (한/글과 2.8px)
//!
//!   voff -70,819HU = -944.25px, valign=Center:
//!   232.1 + (79.05 - 65.8 - 944.25) / 2 = -233.4
//! ```
//!
//! ⚠ **음수라고 무조건 버리면 안 된다.** `#5734`(156684746 9쪽 왼쪽 칸)의 첫 그림도 저장
//! vpos 가 0이라 같은 폴백 갈래로 오는데 `-1,079HU`(14.4px)는 **적용되는 것이 정답**이고
//! `issue_5734_cell_float_stack_stored_vpos` 가 그 값을 잠근다. 여기서는 같은 쪽의 **다른
//! 그림 10장**이 자기 칸 안에 그대로 남는지로 그 축을 함께 잠근다.
//!
//! ## [#6761] 쪽 번호가 하나 밀렸다 — 기하 계약은 그대로다
//!
//! 이 fixture 는 `#6761` 이 다루는 바로 그 문서다. `#6761` 수정은 저장 사다리가 적어 둔
//! 쪽 경계 하나를 복원한다 — 한/글 정본 14쪽(`최종안 제시 및 보고 자료: Design B 최종 제안
//! 및 결정`)을 rhwp 가 13쪽에 얹고 있었다. 그 쪽이 제자리로 가면서 **뒤쪽 전부가 +1** 밀렸다.
//!
//! 그래서 이 파일의 `PAGE_INDEX` 를 76 → 77 로, `page_count` 를 104 → 105 로 옮긴다.
//! **검사 항목은 하나도 완화하지 않았다** — 칸 안 그림 11개, `row4/col3` 의 CCC, 한/글
//! 2020 기준 `y = 235.9 ± 3.0` 이 새 쪽 번호에서 그대로 성립한다(실측 `y = 238.7`).
//!
//! `page_count` 는 한컴 정본값이 아니다 — 이 문서의 정본은 **103쪽**이고(MCP engine 2024
//! 변환) rhwp 는 104 → 105 로 움직인다. 이 값은 그저 이 시험의 쪽 좌표 앵커다. 남은 두 쪽
//! 격차(표 제목행만 남는 빈 쪽 2건)는 `#6761` 범위 밖이다.
//!
//! ## [#6761 후속] 빈 조각 쪽이 사라져 쪽수가 105 → 104 다
//!
//! 같은 이슈의 개체 칸 회계 수정(`빈 개체 줄을 그림 위에 쌓지 않는다`)으로 이 문서의
//! `<표 4-1> 국내외 유사 마크 현황` 이 한 쪽에 들어간다. 수정 전에는 마지막 `덴마크` 행의
//! 그림만 이어받는 **여분 쪽**이 78쪽 뒤에 끼어 있었다. 정본은 그 표를 55쪽 한 장에 담는다.
//!
//! - `PAGE_INDEX`(77) 는 그대로다 — 없어진 쪽은 그 **뒤**(0-기반 78)였다.
//! - `page_count` 는 105 → **104**. 정본은 103쪽이므로 한 쪽 가까워진다.
//! - 그 쪽의 칸 안 그림은 11 → **12** 개. 정본 55쪽의 그림도 12개다(`pdfimages -list`).
//!   덴마크 행의 마크가 제 행으로 돌아온 몫이다.
//!
//! ## [#6761 잔여 축] 나란히 놓이는 그림을 더하지 않으면서 쪽수가 104 → 103 이다
//!
//! 한 문단의 개체를 가로 겹침과 무관하게 세로로 합산하던 측정을 배치와 맞췄다.
//! `<표 3-4>` 의 마지막 행이 제 쪽에 들어가면서 그 앞의 여분 쪽도 사라진다.
//!
//! - `page_count` 104 → **103** — 한컴 정본 쪽수와 같다.
//! - `PAGE_INDEX` 77 → **76**. 없어진 쪽이 이 쪽 **앞**(`<표 3-4>` 구간)이다.
//! - 그 쪽의 칸 안 그림은 12장 그대로다(정본 55쪽도 12장).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp";

/// 0-based — 대상 그림이 있는 물리 77쪽(한/글 2020 정본도 같은 인덱스).
const PAGE_INDEX: u32 = 76;
const PAGE_HEIGHT_PX: f64 = 1122.5;
const TOLERANCE_PX: f64 = 1.0;

/// 대상 그림을 치수로 집는다 — 이 쪽에서 유일하다.
const TARGET_W: f64 = 81.1;
const TARGET_H: f64 = 65.8;
/// 대상 칸.
const TARGET_ROW: u16 = 4;
const TARGET_COL: u16 = 3;
/// 한/글 2020 정본의 대상 그림 y(px). 정본은 `pdf/` 에 보존돼 있다.
const HANGUL_Y_PX: f64 = 235.9;
/// 회귀 시 값 — 가드를 되돌리면 정확히 여기로 간다.
const REGRESSION_Y_PX: f64 = -233.4;

/// `(row, col, (칸 y, 칸 높이), (그림 x, y, w, h))`
type CellImage = (u16, u16, (f64, f64), (f64, f64, f64, f64));

fn collect_cell_images<'a>(
    node: &'a RenderNode,
    cell: Option<&'a RenderNode>,
    out: &mut Vec<CellImage>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        if let Some(host) = cell {
            if let RenderNodeType::TableCell(info) = &host.node_type {
                out.push((
                    info.row,
                    info.col,
                    (host.bbox.y, host.bbox.height),
                    (node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height),
                ));
            }
        }
    }
    for child in &node.children {
        collect_cell_images(child, cell, out);
    }
}

fn page_cell_images() -> Vec<CellImage> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#6782 공개 fixture 읽기 {}: {error}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse 1480000-201900042");
    assert_eq!(
        document.page_count(),
        103,
        "쪽수는 103쪽이어야 한다 (#6761 잔여 축까지 닫혀 정본과 같다)"
    );
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p77");
    let mut out = Vec::new();
    collect_cell_images(&tree.root, None, &mut out);
    out
}

fn target(images: &[CellImage]) -> CellImage {
    let hits: Vec<&CellImage> = images
        .iter()
        .filter(|(.., (_, _, w, h))| {
            (w - TARGET_W).abs() < TOLERANCE_PX && (h - TARGET_H).abs() < TOLERANCE_PX
        })
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "대상 그림({TARGET_W}x{TARGET_H})은 77쪽에 정확히 하나여야 한다 — \
         그림이 사라졌거나 표본이 어긋났다. got {hits:?}"
    );
    *hits[0]
}

/// 표본 고정 — 대상이 통째로 사라지면 여기서 먼저 걸린다.
#[test]
fn the_page_still_holds_all_twelve_cell_images() {
    let images = page_cell_images();
    assert_eq!(
        images.len(),
        12,
        "77쪽 표의 칸 안 그림은 12장이어야 한다 (정본 55쪽도 12장) — 종전 시험은 `>= 10` \
         이라 대상 한 장이 없어져도 통과했다. got {}",
        images.len()
    );
}

/// 양성 계약 — 대상은 `row=4 col=3` 칸 안에 있고 한/글 좌표와 맞는다.
#[test]
fn the_target_image_sits_inside_its_own_cell_at_the_hangul_position() {
    let images = page_cell_images();
    let (row, col, (cell_y, cell_h), (_, image_y, _, image_h)) = target(&images);

    assert_eq!(
        (row, col),
        (TARGET_ROW, TARGET_COL),
        "대상 그림은 row={TARGET_ROW} col={TARGET_COL} 칸에 있어야 한다"
    );

    // 회귀 계약 — 가드를 되돌리면 정확히 여기로 간다.
    assert!(
        (image_y - REGRESSION_Y_PX).abs() > TOLERANCE_PX,
        "회귀: 대상 그림이 {REGRESSION_Y_PX}px 로 되돌아갔다 (용지 위쪽 밖, 인쇄에서 소실)"
    );
    assert!(
        image_y >= 0.0 && image_y + image_h <= PAGE_HEIGHT_PX,
        "대상 그림이 용지(0..{PAGE_HEIGHT_PX})를 벗어났다 — y={image_y:.1} h={image_h:.1}"
    );

    // 자기 칸과의 실제 교집합.
    let overlap = (image_y + image_h).min(cell_y + cell_h) - image_y.max(cell_y);
    assert!(
        overlap > 0.0,
        "대상 그림이 자기 칸과 겹치지 않는다 — 칸 {cell_y:.1}..{:.1}, 그림 {image_y:.1}..{:.1}",
        cell_y + cell_h,
        image_y + image_h
    );

    // 한/글 2020 정본(`pdf/…-chemical-labeling-standards-2020.pdf`, idx 76)은 235.9px.
    // `place(0.0)` 이 주는 238.7 과 2.8px 차이이므로 한/글 허용오차 안이다.
    assert!(
        (image_y - HANGUL_Y_PX).abs() <= 4.0,
        "대상 그림 y={image_y:.1}px 가 한/글 2020 정본 {HANGUL_Y_PX}px 에서 4px 넘게 벗어났다"
    );
}

/// 음성 통제군 — 나머지 10장은 이 가드가 건드리면 안 된다.
///
/// 같은 쪽·같은 표·같은 폴백 갈래인데 오프셋 결과가 칸 안에 남으므로 그대로 적용돼야 한다.
/// 「음수면 0」이나 「결과 바닥을 칸 상단으로」 같은 넓은 판으로 바꾸면 여기가 깨진다.
#[test]
fn the_other_eleven_images_keep_their_offsets() {
    let images = page_cell_images();
    let (.., (_, target_y, ..)) = target(&images);

    let others: Vec<&CellImage> = images
        .iter()
        .filter(|(.., (_, y, ..))| (y - target_y).abs() > f64::EPSILON)
        .collect();
    assert_eq!(others.len(), 11, "대상 외 그림은 11장이어야 한다");

    for (row, col, (cell_y, cell_h), (_, image_y, _, image_h)) in &others {
        assert!(
            *image_y >= 0.0 && image_y + image_h <= PAGE_HEIGHT_PX,
            "row={row} col={col} 그림이 용지 밖으로 나갔다 — y={image_y:.1}"
        );
        let overlap = (image_y + image_h).min(cell_y + cell_h) - image_y.max(*cell_y);
        assert!(
            overlap > 0.0,
            "row={row} col={col} 그림이 자기 칸과 겹치지 않는다 — \
             칸 {cell_y:.1}..{:.1}, 그림 {image_y:.1}..{:.1}",
            cell_y + cell_h,
            image_y + image_h
        );
    }
}

/// 일본 PS 두 마크는 같은 셀의 아래 경계를 넘어가면 안 된다.
///
/// 이 셀은 저장 `cell.height`가 행의 실제 높이보다 작고, 빈 문단의 `vpos`는
/// 행 하단 쪽에 저장돼 있다. 이를 두 부동 그림의 문단 기준점으로 그대로 쓰면
/// 두 번째 마크가 다음 행으로 밀린다. 한/글 2020 기준 PDF(물리 77쪽, 인쇄 쪽번호
/// 55)에서는 두 마크 모두 일본 행 안에 온전히 들어간다.
#[test]
fn japan_mixed_wrap_marks_stay_inside_their_cell() {
    let images = page_cell_images();
    let japan: Vec<&CellImage> = images
        .iter()
        .filter(|(row, col, _, _)| (*row, *col) == (5, 3))
        .collect();
    assert_eq!(
        japan.len(),
        2,
        "일본 인증마크 셀에는 그림 두 장이 있어야 한다"
    );

    for (_, _, (cell_y, cell_h), (_, image_y, _, image_h)) in japan {
        assert!(
            *image_y >= *cell_y - TOLERANCE_PX
                && image_y + image_h <= cell_y + cell_h + TOLERANCE_PX,
            "일본 PS 마크가 자기 셀을 벗어났다: 셀 {cell_y:.1}..{:.1}, 그림 {image_y:.1}..{:.1}",
            cell_y + cell_h,
            image_y + image_h,
        );
    }
}

/// 일본 PS 두 마크는 빈 문단의 마지막 글줄 위에 놓인다.
///
/// 아래쪽으로 내보내던 회귀를 막기 위해 셀 content bottom에 그림을 붙이면, 이번에는
/// 한/글이 남겨 둔 빈 문단 한 줄(1000 HU = 13.33px)을 덮어 PDF보다 아래로 내려간다.
/// 한/글 PDF(물리 77쪽, 인쇄 쪽번호 55)의 두 그림 frame top은 각각 318.2px,
/// 319.0px이다. 첫 그림과 둘째 그림의 세로 offset 차이(82 HU = 1.09px)를 보존한
/// 값이며, 원본 HWP와 축소 fixture 양쪽에서 고정한다.
#[test]
fn japan_mixed_wrap_marks_reserve_the_blank_line_at_cell_bottom() {
    let images = page_cell_images();
    let mut japan: Vec<&CellImage> = images
        .iter()
        .filter(|(row, col, _, _)| (*row, *col) == (5, 3))
        .collect();
    japan.sort_by(|a, b| a.3 .0.total_cmp(&b.3 .0));
    assert_eq!(
        japan.len(),
        2,
        "일본 인증마크 셀에는 그림 두 장이 있어야 한다"
    );

    let expected_tops = [318.2, 319.0];
    for (image, expected_top) in japan.iter().zip(expected_tops) {
        let (_, _, _, (_, image_y, _, _)) = image;
        assert!(
            (image_y - expected_top).abs() <= TOLERANCE_PX,
            "일본 PS 마크의 빈 글줄 예약 위치가 한/글 PDF와 다르다: y={image_y:.1}, expected={expected_top:.1}"
        );
    }
}

/// 축소 과정이 전체 원본의 칸과 그림 배치를 바꾸지 않는지 공개 입력끼리 대조한다.
#[test]
fn the_reduced_fixture_preserves_original_cell_image_geometry() {
    let original_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp");
    let bytes = std::fs::read(&original_path)
        .unwrap_or_else(|error| panic!("전체 원본 읽기 {}: {error}", original_path.display()));
    let original = HwpDocument::from_bytes(&bytes).expect("parse full original");
    assert_eq!(original.page_count(), 103);
    let tree = original
        .build_page_render_tree(PAGE_INDEX)
        .expect("render full original p77");
    let mut original_images = Vec::new();
    collect_cell_images(&tree.root, None, &mut original_images);

    let reduced_images = page_cell_images();
    assert_eq!(original_images.len(), 12);
    assert_eq!(reduced_images.len(), original_images.len());
    for (index, (reduced, full)) in reduced_images.iter().zip(&original_images).enumerate() {
        assert_eq!((reduced.0, reduced.1), (full.0, full.1), "image {index}");
        let reduced_geometry = [
            reduced.2 .0,
            reduced.2 .1,
            reduced.3 .0,
            reduced.3 .1,
            reduced.3 .2,
            reduced.3 .3,
        ];
        let full_geometry = [
            full.2 .0, full.2 .1, full.3 .0, full.3 .1, full.3 .2, full.3 .3,
        ];
        for (actual, expected) in reduced_geometry.into_iter().zip(full_geometry) {
            assert!(
                (actual - expected).abs() <= 1e-7,
                "image {index}: reduced={actual}, full={expected}"
            );
        }
    }
}

/// 수동 IR 반례: 실제 3..14행 이어받기 조각의 가운데 정렬을 흐름 상자로 검사한다.
/// 재저장한 한컴 문서의 출력 증거와 구분하며 원본 그림 유닛은 그대로 보존한다.
#[test]
fn continued_centered_image_uses_the_reserved_forward_space() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    for offset in [-187i32, 0, 187] {
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();
        let mut doc = core.document().clone();
        let Control::Table(table) = &mut doc.sections[4].paragraphs[118].controls[0] else {
            panic!("원본 이어받는 표");
        };
        let cell = table
            .cells
            .iter_mut()
            .find(|cell| (cell.row, cell.col) == (4, 3))
            .unwrap();
        let picture = cell
            .paragraphs
            .iter_mut()
            .flat_map(|p| &mut p.controls)
            .find_map(|c| {
                if let Control::Picture(picture) = c {
                    Some(picture)
                } else {
                    None
                }
            })
            .unwrap();
        picture.common.vertical_offset = offset as u32;
        core.set_document(doc);
        let tree = core.build_page_render_tree(PAGE_INDEX).unwrap();
        let mut images = Vec::new();
        collect_cell_images(&tree.root, None, &mut images);
        assert_eq!(images.len(), 12, "이어받는 조각의 그림 누락·중복 없음");
        let (_, _, (cell_y, cell_h), (_, image_y, _, image_h)) = images
            .iter()
            .find(|image| (image.0, image.1) == (4, 3))
            .unwrap();
        // 대칭 셀 여백의 중앙 정렬 불변식: 앞 공간은 흐름 상자에 포함되며 음수는 앞 공간이 아니다.
        let expected = cell_y + (cell_h - image_h + f64::from(offset.max(0)) / 75.0) / 2.0;
        assert!(
            (image_y - expected).abs() < 0.1,
            "이어받는 그림 오프셋{offset}의 물리 중심: {image_y}, 독립 정렬 불변식{expected}"
        );
        assert_eq!(core.page_count(), 103);
    }
}

fn japan_cell(node: &RenderNode, row: u16) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TableCell(cell)
        if cell.row == row && cell.col == 3)
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| japan_cell(child, row))
}

fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        descendants(child, out);
    }
}

fn assert_japan_frame(cell: &RenderNode, offsets: [i32; 2]) {
    let mut nodes = Vec::new();
    descendants(cell, &mut nodes);
    let mut images: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
        .copied()
        .collect();
    images.sort_by(|a, b| a.bbox.x.total_cmp(&b.bbox.x));
    assert_eq!(images.len(), 2, "그림 묶음의 소유 누락·중복 없음");
    let lines: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .collect();
    assert_eq!(lines.len(), 1, "원본 마지막 빈 줄 하나 보존");
    // 원본 저장 줄과 행 선언: 4728 + 1000 + 위/아래 141씩 = 6010HU.
    let line_y = cell.bbox.y + cell.bbox.height - (141.0 + 1000.0) / 75.0;
    assert!(
        (lines[0].bbox.y - line_y).abs() < 0.1,
        "마지막 빈 줄 위치: {:?}, {line_y}",
        lines[0].bbox
    );
    assert!((lines[0].bbox.height - 1000.0 / 75.0).abs() < 0.1);
    // 두 그림은 저장 마지막 줄 앞의 같은 띠 원점을 공유한다. 높이가 다른 그림의
    // 오프셋을 0으로 만든 한컴 대조군에서도 상단이 같다는 독립 결과를 검사한다.
    let band_end = (offsets[0] + 3806).max(offsets[1] + 3866);
    let band_origin = line_y - f64::from(band_end) / 75.0;
    for (image, offset) in images.iter().zip(offsets) {
        let expected = band_origin + f64::from(offset) / 75.0;
        assert!(
            (image.bbox.y - expected).abs() < 0.1,
            "같은 띠 원점: {:?}, {expected}",
            image.bbox
        );
        assert!(image.bbox.y >= cell.bbox.y - 0.1);
        assert!(image.bbox.y + image.bbox.height <= line_y + 0.1);
    }
}

/// 원본 77쪽의 마지막 빈 줄과 두 그림이 같은 프레임을 소비한다.
/// 0 오프셋은 원본 레코드 두 값만 바꾼 독립 한컴 PDF 대조와 연결한 수동 IR 검사다.
#[test]
fn japan_picture_band_and_final_empty_line_share_the_stored_frame() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    for offsets in [[780, 862], [0, 0]] {
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();
        let mut doc = core.document().clone();
        let Control::Table(table) = &mut doc.sections[4].paragraphs[118].controls[0] else {
            panic!("표");
        };
        let cell = table
            .cells
            .iter_mut()
            .find(|cell| (cell.row, cell.col) == (5, 3))
            .unwrap();
        for (control, offset) in cell.paragraphs[0].controls.iter_mut().zip(offsets) {
            let Control::Picture(picture) = control else {
                panic!("그림");
            };
            picture.common.vertical_offset = offset as u32;
        }
        core.set_document(doc);
        assert_eq!(core.page_count(), 103);
        let tree = core.build_page_render_tree(PAGE_INDEX).unwrap();
        let cell = japan_cell(&tree.root, 5).unwrap();
        assert_japan_frame(cell, offsets);
        let mut images = Vec::new();
        collect_cell_images(&tree.root, None, &mut images);
        assert_eq!(images.len(), 12);
        if offsets == [0, 0] {
            for image in images.iter().filter(|image| (image.0, image.1) == (5, 3)) {
                assert!((image.3 .1 - 319.011).abs() < 1.0, "독립 0 대조 PDF 상단");
            }
        }
    }
}

/// 원본 한 행만 분리한 수동 IR에서 온전한 셀 경로도 같은 저장 상대 좌표를 사용한다.
#[test]
fn whole_japan_cell_uses_the_same_picture_and_empty_line_frame() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    use rhwp::model::table::TablePageBreak;
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let mut doc = core.document().clone();
    let mut section = doc.sections[4].clone();
    let mut host = section.paragraphs[118].clone();
    host.line_segs[0].vertical_pos = 0;
    let Control::Table(table) = &mut host.controls[0] else {
        panic!("표");
    };
    table.cells.retain(|cell| cell.row == 5);
    for cell in &mut table.cells {
        cell.row = 0;
    }
    table.row_count = 1;
    table.page_break = TablePageBreak::None;
    table.common.height = 6010;
    table.common.vertical_offset = 0;
    section.paragraphs = vec![host];
    doc.sections = vec![section];
    core.set_document(doc);
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let cell = japan_cell(&tree.root, 0).unwrap();
    assert!(matches!(&cell.node_type, RenderNodeType::TableCell(data) if !data.page_fragment));
    assert_japan_frame(cell, [780, 862]);
}

/// 수동 IR의 좁은 본문 예산으로 실제 행 분할·이월 경로를 실행한다.
/// 마지막 줄만 먼저 담지 않고 두 그림과 같은 유닛으로 한 번씩 보존해야 한다.
#[test]
fn deferred_japan_picture_band_keeps_both_marks_and_final_line() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    use rhwp::model::table::TablePageBreak;
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    for body_height in [9000, 11000] {
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();
        let mut doc = core.document().clone();
        let mut section = doc.sections[4].clone();
        let mut host = section.paragraphs[118].clone();
        host.line_segs[0].vertical_pos = 4500;
        let mut prefix = host.clone();
        prefix.controls.clear();
        prefix.line_segs[0].vertical_pos = 0;
        prefix.line_segs[0].line_height = 4500;
        prefix.line_segs[0].text_height = 4500;
        prefix.line_segs[0].baseline_distance = 3500;
        prefix.line_segs[0].line_spacing = 0;
        let Control::Table(table) = &mut host.controls[0] else {
            panic!("표");
        };
        table.cells.retain(|cell| cell.row == 5);
        for cell in &mut table.cells {
            cell.row = 0;
        }
        table.row_count = 1;
        table.page_break = TablePageBreak::RowBreak;
        table.common.height = 6010;
        table.common.vertical_offset = 0;
        let mut tail = rhwp::model::paragraph::Paragraph::new_empty_like(&host);
        tail.insert_text_at(0, "보정33 뒤 문단");
        tail.invalidate_layout_inputs();
        section.paragraphs = vec![prefix, host, tail];
        let page = &mut section.section_def.page_def;
        // 본문 예산은 용지에서 위/아래 및 머리말/꼬리말 여백을 모두 뺀 공간이다.
        page.height = page.margin_top
            + page.margin_bottom
            + page.margin_header
            + page.margin_footer
            + body_height;
        doc.sections = vec![section];
        core.set_document(doc);
        let mut group_owners = 0;
        let mut partial_owners = 0;
        let mut tail_count = 0;
        let mut last_group_end = None;
        let mut tail_position = None;
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut all_nodes = Vec::new();
            descendants(&tree.root, &mut all_nodes);
            for node in &all_nodes {
                if matches!(node.node_type, RenderNodeType::TextLine(_)) {
                    let mut leaves = Vec::new();
                    descendants(node, &mut leaves);
                    let text: String = leaves
                        .iter()
                        .filter_map(|leaf| {
                            if let RenderNodeType::TextRun(run) = &leaf.node_type {
                                Some(run.text.as_str())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if text.contains("보정33 뒤 문단") {
                        tail_count += 1;
                        tail_position = Some((page, node.bbox.y));
                    }
                }
            }
            if let Some(cell) = japan_cell(&tree.root, 0) {
                let mut nodes = Vec::new();
                descendants(cell, &mut nodes);
                if nodes
                    .iter()
                    .any(|node| matches!(node.node_type, RenderNodeType::Image(_)))
                {
                    group_owners += 1;
                    partial_owners += usize::from(
                        matches!(&cell.node_type, RenderNodeType::TableCell(data) if data.page_fragment),
                    );
                    assert_japan_frame(cell, [780, 862]);
                    last_group_end = Some((page, cell.bbox.y + cell.bbox.height));
                    let body = all_nodes
                        .iter()
                        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
                        .unwrap();
                    assert!(
                        (body.bbox.height - f64::from(body_height) / 75.0).abs() < 0.1,
                        "지정한 실제 본문 예산: {:?}",
                        body.bbox
                    );
                    assert!(cell.bbox.y >= body.bbox.y - 0.5);
                    assert!(
                        cell.bbox.y + cell.bbox.height <= body.bbox.y + body.bbox.height + 0.5,
                        "본문 점유 하단 준수: {:?}, {:?}",
                        cell.bbox,
                        body.bbox
                    );
                }
            }
        }
        assert_eq!(tail_count, 1, "뒤 문단 누락·중복 없음");
        let (group_page, group_bottom) = last_group_end.unwrap();
        let (tail_page, tail_y) = tail_position.unwrap();
        assert!(tail_page > group_page || (tail_page == group_page && tail_y >= group_bottom - 0.5),
            "뒤 문단과의 점유 겹침 없음: 묶음({group_page}, {group_bottom}), 뒤 문단({tail_page}, {tail_y})");
        assert_eq!(
            group_owners, 1,
            "본문{body_height} 그림 묶음의 유일한 소유 쪽"
        );
        assert!(core.page_count() >= 2, "실제 이월 경계 실행");
        assert!(partial_owners > 0, "실제 부분 셀 경로 실행");
    }
}
