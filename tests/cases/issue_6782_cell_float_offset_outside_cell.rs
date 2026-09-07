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
    assert_eq!(document.page_count(), 104, "쪽수는 104쪽이어야 한다");
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
fn the_page_still_holds_all_eleven_cell_images() {
    let images = page_cell_images();
    assert_eq!(
        images.len(),
        11,
        "77쪽 표의 칸 안 그림은 11장이어야 한다 — 종전 시험은 `>= 10` 이라 \
         대상 한 장이 없어져도 통과했다. got {}",
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
fn the_other_ten_images_keep_their_offsets() {
    let images = page_cell_images();
    let (.., (_, target_y, ..)) = target(&images);

    let others: Vec<&CellImage> = images
        .iter()
        .filter(|(.., (_, y, ..))| (y - target_y).abs() > f64::EPSILON)
        .collect();
    assert_eq!(others.len(), 10, "대상 외 그림은 10장이어야 한다");

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

/// 축소 과정이 전체 원본의 칸과 그림 배치를 바꾸지 않는지 공개 입력끼리 대조한다.
#[test]
fn the_reduced_fixture_preserves_original_cell_image_geometry() {
    let original_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp");
    let bytes = std::fs::read(&original_path)
        .unwrap_or_else(|error| panic!("전체 원본 읽기 {}: {error}", original_path.display()));
    let original = HwpDocument::from_bytes(&bytes).expect("parse full original");
    assert_eq!(original.page_count(), 104);
    let tree = original
        .build_page_render_tree(PAGE_INDEX)
        .expect("render full original p77");
    let mut original_images = Vec::new();
    collect_cell_images(&tree.root, None, &mut original_images);

    let reduced_images = page_cell_images();
    assert_eq!(original_images.len(), 11);
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
