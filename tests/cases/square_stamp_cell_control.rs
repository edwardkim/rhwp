//! 그림 아래 이름표 내리기 규칙의 대조군: 결재 도장 칸과 글줄 띠 안 사진 칸.
//!
//! `square_floor_empty_lead_row_height` 의 규칙(칸 글줄 띠 밖 어울림 그림만 가진 빈 문단을
//! 그리기와 같은 em 줄박스로 세는 것)은 다음 두 칸에 걸리면 안 된다.
//! - 결재 표 도장 칸: 저장 줄 없는 빈 문단 하나 + 칸 글줄 띠 안의 어울림 도장 그림. 내릴 글줄이
//!   없다. 이 칸 높이는 안 여백 + 도장이다.
//! - 본문 표 사진 칸: 글줄 띠 안의 어울림 사진 문단 + 저장 줄 없는 빈 문단 둘 + 설명 글줄.
//!   사진이 글줄을 막으므로 설명 글줄 자리는 사진 바닥이 정하고, 칸 안에 담긴다.
//!
//! 재현물 `samples/square_floor_empty_lead/approval-stamp-cell.hwp` 는 생성기
//! `tools/fixture_generators/build_square_stamp_cell_fixture.rs` 가 공개 합성본
//! `samples/title_flow_photo/title-flow-photo-row.hwp` 의 문단만으로 만든다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/square_floor_empty_lead/approval-stamp-cell.hwp";
/// 칸 안 여백 141HU(px).
const PAD_PX: f64 = 141.0 * 96.0 / 7200.0;
/// 도장 그림 3131HU(px), 사진 그림 6000HU(px).
const STAMP_PX: f64 = 3131.0 * 96.0 / 7200.0;
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
struct CellBox {
    rows: (u16, u16),
    cell: Rect,
    images: Vec<Rect>,
    lines: Vec<(Rect, String)>,
}

fn cells() -> Vec<CellBox> {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            y: n.bbox.y,
            h: n.bbox.height,
        }
    }
    fn collect(n: &RenderNode, images: &mut Vec<Rect>, lines: &mut Vec<(Rect, String)>) {
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
                lines.push((rect(n), text));
            }
            _ => {}
        }
        for c in &n.children {
            collect(c, images, lines);
        }
    }
    fn walk(n: &RenderNode, out: &mut Vec<CellBox>) {
        if let RenderNodeType::TableCell(cell) = &n.node_type {
            let mut images = Vec::new();
            let mut lines = Vec::new();
            collect(n, &mut images, &mut lines);
            out.push(CellBox {
                rows: (cell.row, cell.col),
                cell: rect(n),
                images,
                lines,
            });
            return;
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        walk(&tree.root, &mut out);
    }
    out
}

/// 도장 칸 높이는 안 여백 + 도장이다(빈 줄 몫을 더하지 않는다). 도장은 칸 안에 있다.
#[test]
fn stamp_only_cell_closes_at_stamp() {
    let all = cells();
    let stamp = all
        .iter()
        .find(|c| {
            c.rows == (2, 1) && c.images.len() == 1 && (c.images[0].h - STAMP_PX).abs() <= TOL_PX
        })
        .unwrap_or_else(|| panic!("도장 칸: {all:?}"));
    let expected = PAD_PX * 2.0 + STAMP_PX;
    assert!(
        (stamp.cell.h - expected).abs() <= TOL_PX,
        "도장 칸 높이({:.2})가 안 여백 + 도장({expected:.2})이어야 한다: {stamp:?}",
        stamp.cell.h
    );
    let img = stamp.images[0];
    assert!(
        img.y >= stamp.cell.y - TOL_PX && img.bottom() <= stamp.cell.bottom() + TOL_PX,
        "도장이 칸 안에 있어야 한다: {stamp:?}"
    );
}

/// 글줄 띠 안 사진 칸: 설명 글줄은 사진 바닥 아래에 있고 칸 안에 담긴다.
#[test]
fn in_band_photo_cell_keeps_label_below_photo_inside_cell() {
    let all = cells();
    let photo = all
        .iter()
        .find(|c| c.images.iter().any(|r| (r.h - PHOTO_PX).abs() <= TOL_PX))
        .unwrap_or_else(|| panic!("사진 칸: {all:?}"));
    let img = photo
        .images
        .iter()
        .find(|r| (r.h - PHOTO_PX).abs() <= TOL_PX)
        .unwrap();
    let (label, _) = photo
        .lines
        .iter()
        .find(|(_, t)| t.contains("제목"))
        .unwrap_or_else(|| panic!("설명 글줄: {photo:?}"));
    assert!(
        label.y >= img.bottom() - TOL_PX,
        "설명 글줄 위({:.2})가 사진 바닥({:.2}) 아래여야 한다: {photo:?}",
        label.y,
        img.bottom()
    );
    assert!(
        label.bottom() <= photo.cell.bottom() + TOL_PX,
        "설명 글줄이 칸 안에 있어야 한다: {photo:?}"
    );
}
