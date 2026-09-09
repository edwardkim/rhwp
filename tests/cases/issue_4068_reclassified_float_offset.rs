//! [Issue #4068] `#2004` 정규화가 셀 안 부동 그림 스택을 인라인으로 재분류하면서
//! **배치 계약을 통째로 버리던** 결함의 가드.
//!
//! `reclassify_cell_floating_stacks`(`document_core/queries/rendering.rs`)는 겹치는
//! 부동 그림 스택을 "그림 1장짜리 인라인 문단 N개" 로 쪼개 `treat_as_char` 를 켠다.
//! 그 순간 `horzOffset` 이 버려져 그림 다섯 장이 전부 칸 왼쪽에 붙는다.
//!
//! ```text
//! rhwp dump          bin_id=3..7  tac=false wrap=Square horz=Para(0/1580/2092/1550/1432)
//! layout_picture 도착  tac=true (5장 전부)  → x 가 전부 inner_area.x
//! ```
//!
//! 한/글 2024 실측(표 왼쪽 세로 괘선 기준, pt):
//!
//! ```text
//!        horzOffset       한/글    rhwp 종전   교정 후
//!  4쪽      0HU            5.04      5.10       5.10
//!  5쪽   1580HU (15.80)   20.87      5.10      20.90
//!  6쪽   2092HU (20.92)   25.90      5.10      26.02
//!  7쪽   1550HU (15.50)   20.51      5.10      20.60
//!  8쪽   1432HU (14.32)   19.31      5.10      19.42
//! ```
//!
//! ⚠ 표 틀 자체가 `om`(283HU = 3.77px) 만큼 왼쪽·위로 밀려 있는 것은 **별개 축**이다.
//! 그래서 이 시험은 절대 x 가 아니라 **표 왼쪽 괘선으로부터의 상대 x** 를 잰다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue2004_cell_image_stack.hwp";

/// (표 왼쪽 세로 괘선 x, 가장 위 그림의 x)
fn page_geometry(core: &DocumentCore, page: u32) -> Option<(f64, f64)> {
    let tree = core.build_page_render_tree(page).ok()?;
    let mut vlines: Vec<f64> = Vec::new();
    let mut images: Vec<(f64, f64)> = Vec::new();
    fn walk(n: &RenderNode, vlines: &mut Vec<f64>, images: &mut Vec<(f64, f64)>) {
        match &n.node_type {
            RenderNodeType::Line(l) if (l.x1 - l.x2).abs() < 0.5 && (l.y2 - l.y1).abs() > 100.0 => {
                vlines.push(l.x1)
            }
            RenderNodeType::Image(_) => images.push((n.bbox.y, n.bbox.x)),
            _ => {}
        }
        for c in &n.children {
            walk(c, vlines, images);
        }
    }
    walk(&tree.root, &mut vlines, &mut images);
    vlines.sort_by(|a, b| a.partial_cmp(b).unwrap());
    images.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some((*vlines.first()?, images.first()?.1))
}

/// 재분류된 부동 그림은 저장된 `horzOffset` 만큼 오른쪽에서 시작해야 한다.
#[test]
fn reclassified_floating_pictures_keep_their_horizontal_offset() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    // (쪽 index, horzOffset HU, 한/글 실측 상대 x[pt])
    let cases = [
        (3usize, 0i32, 5.04f64),
        (4, 1580, 20.87),
        (5, 2092, 25.90),
        (6, 1550, 20.51),
        (7, 1432, 19.31),
    ];
    let mut checked = 0;
    for (page, offset_hu, hangul_pt) in cases {
        let Some((line_x, img_x)) = page_geometry(&core, page as u32) else {
            continue;
        };
        let rel_pt = (img_x - line_x) * 0.75;
        assert!(
            (rel_pt - hangul_pt).abs() <= 1.0,
            "{}쪽 그림이 표 왼쪽 괘선에서 {hangul_pt:.2}pt 떨어져야 한다(한/글 2024, \
             horzOffset {offset_hu}HU) — #4068 회귀. got {rel_pt:.2}pt \
             (오프셋을 버리면 5.10)",
            page + 1
        );
        checked += 1;
    }
    assert_eq!(checked, 5, "5쪽 전부를 재야 한다 — 시험 설정 오류");
}

/// 저작 단계에서 인라인으로 놓인 그림은 건드리지 않는다 — 4쪽(offset 0)이 그 증인이고,
/// 같은 문서 1쪽의 머리 그림(`wrap=TopAndBottom`)도 종전 자리를 지킨다.
#[test]
fn genuine_inline_pictures_are_untouched() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let Some((line_x, img_x)) = page_geometry(&core, 3) else {
        panic!("4쪽 기하를 못 읽었다 — 시험 설정 오류");
    };
    let rel_pt = (img_x - line_x) * 0.75;
    assert!(
        (rel_pt - 5.10).abs() <= 0.6,
        "offset 0 인 그림은 종전 자리(5.10pt)를 지켜야 한다 — got {rel_pt:.2}pt"
    );
}
