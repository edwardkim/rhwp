//! 어울림 그림 아래로 내려간 이름표가 칸 안에 담기는가(빈 그림 문단이 앞에 있는 칸).
//!
//! 저장 줄 없는 칸에 `p0 = 칸 글줄 띠 밖에 앵커한 어울림 그림만 가진 빈 문단`,
//! `p1 = 사진 어울림 그림 문단`, `p2 = 이름표` 가 놓인다. 그리기는 p0 을 글자 모양의 em
//! 줄박스(줄 높이 + 줄 간격)로 놓고, 사진 문단 원점을 그 아래에 두며, 이름표를 사진 바닥
//! 아래로 내린다. 행 높이 측정이 p0 을 400HU 자리 줄로 세면 사진 바닥이 그만큼 위에 잡혀
//! 행이 그린 이름표보다 짧아지고, 이름표가 칸 아래 괘선에서 잘려 보이지 않는다(실문서 활동
//! 사진 칸의 반 이름표가 사라진 증상).
//!
//! 재현물 `samples/square_floor_empty_lead/cell-label-below-picture-after-empty-anchor.hwp` 는
//! 생성기 `tools/fixture_generators/build_square_floor_empty_lead_fixture.rs` 가 공개 합성본
//! `samples/title_flow_photo/title-flow-photo-row.hwp` 의 문단만으로 만든다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str =
    "samples/square_floor_empty_lead/cell-label-below-picture-after-empty-anchor.hwp";
/// 사진 그림 높이 6000HU(px).
const PHOTO_PX: f64 = 6000.0 * 96.0 / 7200.0;
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Rect {
    y: f64,
    h: f64,
}

impl Rect {
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

#[derive(Debug)]
struct PhotoCell {
    cell: Rect,
    photo: Rect,
    label: Rect,
}

fn photo_cell() -> PhotoCell {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            y: n.bbox.y,
            h: n.bbox.height,
        }
    }
    fn collect(n: &RenderNode, images: &mut Vec<Rect>, label: &mut Option<Rect>) {
        match &n.node_type {
            RenderNodeType::Image(_) => images.push(rect(n)),
            RenderNodeType::TextLine(_) => {
                let text: String = n
                    .children
                    .iter()
                    .filter_map(|c| match &c.node_type {
                        RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                        _ => None,
                    })
                    .collect();
                if text.contains("제목") {
                    *label = Some(rect(n));
                }
            }
            _ => {}
        }
        for c in &n.children {
            collect(c, images, label);
        }
    }
    fn find(n: &RenderNode) -> Option<PhotoCell> {
        if let RenderNodeType::TableCell(cell) = &n.node_type {
            if cell.row == 0 && cell.col == 0 {
                let mut images = Vec::new();
                let mut label = None;
                collect(n, &mut images, &mut label);
                let photo = images
                    .into_iter()
                    .find(|r| (r.h - PHOTO_PX).abs() <= TOL_PX)
                    .expect("사진 그림");
                return Some(PhotoCell {
                    cell: rect(n),
                    photo,
                    label: label.expect("이름표 글줄"),
                });
            }
        }
        n.children.iter().find_map(find)
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("render tree");
    find(&tree.root).expect("사진 칸")
}

/// 이름표는 사진 바닥 아래에 놓인다.
#[test]
fn label_sits_below_square_picture() {
    let c = photo_cell();
    assert!(
        c.label.y >= c.photo.bottom() - TOL_PX,
        "이름표 위({:.2})가 사진 바닥({:.2}) 아래여야 한다: {c:?}",
        c.label.y,
        c.photo.bottom()
    );
}

/// 행 높이는 그린 이름표를 담는다(칸 아래 괘선에서 잘리지 않는다).
#[test]
fn row_height_contains_label_below_picture() {
    let c = photo_cell();
    assert!(
        c.label.bottom() <= c.cell.bottom() + TOL_PX,
        "이름표 아래({:.2})가 칸 아래({:.2}) 안에 있어야 한다: {c:?}",
        c.label.bottom(),
        c.cell.bottom()
    );
}
