//! 관계형·고정형 — [#9217] 평행 그림자는 본문 clip 안에 있다.
//! 주석 갱신: 2026-11-20 09:18 KST (새 파일)
//!
//! 입력: 공개 연구용역 과업지시서(경찰청, 정책연구관리시스템 공개, hwp5, 14쪽)
//! `samples/issue9217/research-task-order-toc-shadow.hwp`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue9217/research-task-order-toc-shadow-2020.pdf`(14쪽).
//! 2쪽 `pi=15` 는 글자처럼 취급한 묶음 개체이고, 그 안의 목차 사각형(47591HU 폭, 본문 폭 45128HU 보다
//! 넓다)이 오른쪽 아래 평행 그림자를 가진다. 평행 그림자는 기본 이동량 600HU(8px)에 저장 offset(0)을
//! 더한 만큼 옮긴 도형으로 재생한다. 수정 전 rhwp 는 잉크 경계에 저장 offset 만 반영해 본문 clip 이
//! 사각형 오른쪽 끝에서 끝났고, 그림자의 오른쪽 띠 8px 가 잘렸다.
//! - 고정형: 전체 쪽수 14(= PDF).
//! - 관계형: 그림자를 가진 사각형을 그림자 이동량만큼 옮긴 상자가 본문 clip 안에 있다.
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = upstream/devel `0a437a7eef`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 14쪽 / 14쪽, 2쪽 99.77(base 87.46), 전쪽 최저 90.16(14쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 14쪽 / 14쪽, 2쪽 99.82, 전쪽 최저 90.20(14쪽)
//! - 증거: `mydocs/pr/assets/issue_9217/`(native·wasm 전쪽 TSV, 2쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9217/research-task-order-toc-shadow.hwp";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 14;
/// 대상 쪽(0 기반, 2쪽).
const PAGE: u32 = 1;
/// 오른쪽 아래 평행 그림자의 기본 이동량 600HU.
const PRESET_PX: f64 = 600.0 * 96.0 / 7200.0;

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

#[test]
fn parallel_shadow_stays_inside_body_clip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let tree = core.build_page_render_tree(PAGE).expect("2쪽");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let clip: BoundingBox = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Body { clip_rect } => *clip_rect,
            _ => None,
        })
        .expect("2쪽 본문 clip");

    // 오른쪽 아래 평행 그림자(종류 4)를 가진 사각형과 그 이동량.
    let shadowed: Vec<(BoundingBox, f64, f64)> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Rectangle(rect) => rect
                .style
                .shadow
                .as_ref()
                .filter(|shadow| shadow.shadow_type == 4)
                .map(|shadow| {
                    (
                        node.bbox,
                        PRESET_PX + shadow.offset_x,
                        PRESET_PX + shadow.offset_y,
                    )
                }),
            _ => None,
        })
        .collect();
    let (frame, dx, dy) = shadowed
        .iter()
        .copied()
        .max_by(|a, b| a.0.width.total_cmp(&b.0.width))
        .expect("2쪽에 오른쪽 아래 그림자를 가진 사각형이 있어야 한다");
    // 목차 사각형은 본문 폭보다 넓은 프레임이다(clip 이 자를 수 있는 자리).
    assert!(
        frame.width > 600.0 && dx > 0.0 && dy > 0.0,
        "목차 사각형과 그림자 이동량: {frame:?} ({dx}, {dy})"
    );

    let shadow_right = frame.x + frame.width + dx;
    let shadow_bottom = frame.y + frame.height + dy;
    assert!(
        shadow_right <= clip.x + clip.width + 0.01,
        "그림자 오른쪽 끝 {shadow_right:.2} 이 본문 clip 오른쪽 {:.2} 안에 있어야 한다",
        clip.x + clip.width
    );
    assert!(
        shadow_bottom <= clip.y + clip.height + 0.01,
        "그림자 아래 끝 {shadow_bottom:.2} 이 본문 clip 아래 {:.2} 안에 있어야 한다",
        clip.y + clip.height
    );
}
