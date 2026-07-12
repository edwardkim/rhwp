//! Issue #2071: 셀 앵커 floating 그림(restrict-ON, TopAndBottom+Para)이 셀
//! vertical_align 을 존중해 세로 정렬되는지 검증한다.
//!
//! 배경: rhwp 는 셀 안 자리차지 그림을 항상 셀 콘텐츠 상단(top)에 앵커해,
//! valign=Center/Bottom 셀에서 그림이 위로 떴다(한컴은 셀 정렬을 존중). PR #2033
//! (off-page 소실) 머지 후 저자 visual sweep 에서 드러난 잔여 결함.
//!
//! 오라클: 한글 2024 편집기 COM 자동화로 ta-pic-001-r-쪽영역안제한 의 오른쪽 셀
//! (valign=Center) 작은 그림 실측 = small_top 362.5px (rhwp 수정 전 153.5px).
//! valign 변형(TOP/BOTTOM)도 편집기 실측으로 153.8 / 571.1 확정.
//! 정렬 기준은 셀 콘텐츠 box(테두리 − 패딩), vertOffset 은 추가 하향 이동:
//!   TOP    = content_top + vertOffset
//!   CENTER = content_top + (content_h − img_h + vertOffset)/2
//!   BOTTOM = content_bottom − img_h − vertOffset

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

#[derive(Debug, Clone, Copy)]
struct Img {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn collect(node: &RenderNode, out: &mut Vec<Img>) {
    if let RenderNodeType::Image(_) = &node.node_type {
        out.push(Img {
            x: node.bbox.x,
            y: node.bbox.y,
            w: node.bbox.width,
            h: node.bbox.height,
        });
    }
    for c in &node.children {
        collect(c, out);
    }
}

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("load {name}: {e}"))
}

/// 오른쪽 셀(x>374)의 작은 그림(폭 ~304px)을 페이지 0 렌더트리에서 찾는다.
fn right_cell_small_image(core: &mut DocumentCore) -> Img {
    let tree = core.build_page_render_tree(0).expect("render tree page 0");
    let mut imgs = Vec::new();
    collect(&tree.root, &mut imgs);
    imgs.into_iter()
        .find(|i| i.x > 374.0 && i.w > 280.0 && i.w < 330.0)
        .expect("오른쪽 셀 작은 그림 ImageNode")
}

/// CENTER 셀(기존 커밋 샘플): 작은 그림이 셀 상단이 아니라 세로 중앙에 배치돼야 한다.
/// 한컴 편집기 실측 small_top=362.5px. 수정 전 rhwp=153.5px(상단 고정).
#[test]
fn center_cell_anchored_picture_is_vertically_centered() {
    let mut core = load("samples/ta-pic-001-r-쪽영역안제한.hwpx");
    let img = right_cell_small_image(&mut core);
    // 한컴 오라클 362.5 ±6px (서브픽셀/패딩 반올림 허용)
    assert!(
        (img.y - 362.5).abs() < 6.0,
        "CENTER 셀 그림이 한컴 실측(362.5) 근처여야 함: 실제 y={:.1} (수정 전 버그값 ~153.5=상단고정)",
        img.y
    );
    // 명시적으로 '상단 고정' 회귀가 아님을 단언
    assert!(
        img.y > 300.0,
        "CENTER 셀 그림이 여전히 셀 상단에 고정됨(valign 미반영): y={:.1}",
        img.y
    );
}

/// 한컴은 셀 앵커 그림을 **셀 valign 으로만** 배치하고 그림 자체 pos vert_align 은
/// 무시한다. CENTER 셀 + 그림 pos vertAlign=Bottom 이어도 셀 중앙(362.5)에 배치돼야
/// 한다(그림 vert_align 을 따르면 ~571=하단). 한글 2024 편집기 오라클.
#[test]
fn pic_pos_valign_ignored_in_center_cell() {
    let mut core = load("samples/ta-pic-cell-center-pos-bottom.hwpx");
    let img = right_cell_small_image(&mut core);
    assert!(
        (img.y - 362.5).abs() < 6.0,
        "그림 pos vertAlign=Bottom 이어도 셀(Center) 중앙(362.5)에 와야 함: y={:.1} \
         (그림 vert_align 을 따르는 버그면 ~571)",
        img.y
    );
}

/// TOP 셀 + 그림 pos vertAlign=Center 이어도 셀 상단(153.8)에 배치돼야 한다
/// (그림 vert_align 을 따르면 ~362=중앙). 셀 valign 이 항상 이긴다.
#[test]
fn pic_pos_valign_ignored_in_top_cell() {
    let mut core = load("samples/ta-pic-cell-top-pos-center.hwpx");
    let img = right_cell_small_image(&mut core);
    assert!(
        (img.y - 153.8).abs() < 6.0,
        "그림 pos vertAlign=Center 이어도 셀(Top) 상단(153.8)에 와야 함: y={:.1} \
         (그림 vert_align 을 따르는 버그면 ~362)",
        img.y
    );
}
