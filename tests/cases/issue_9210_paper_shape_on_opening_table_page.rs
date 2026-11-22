//! 관계형·고정형 — [#9210] 쪽을 넘겨 나뉜 표를 가진 문단의 용지 기준 글 앞 도형은 표의 첫 조각이 있는 쪽(앵커 쪽)에 한 번만 그린다.
//! 주석 갱신: 2026-11-23 01:25 KST(+09:00). 직전 파일 변경 커밋 `2449c3ebab`(2026-11-22 17:03:12 +09:00,
//! rebase 후 `9cfee854b8`)에 이어 기준 PDF 를 한컴 2020 Print 인쇄본으로 바꾸고 측정값을 다시 적음.
//!
//! 입력: 공개 문서 `samples/issue9038/sme_announcement_split_table_successor.hwpx`
//! (중소벤처기업부 사업공고 첨부 HWPX, edwardkim/rhwp#9038).
//! 기준: `pdf/issue9210/sme_announcement_split_table_successor-2020.pdf`(한컴 2020 Hwp.exe 11.0.0.1623,
//! `Hancom PDF` Print, `hancom2020_pdf_driver_one_up`, 46쪽, SHA-256 `e68b80c8…446e7`).
//!
//! `pi=251` 은 25행 1열 RowBreak 표(`ci=0`, 43~46쪽에 나뉨)와 빨간 점선 사각형 두 개
//! (`ci=1`·`ci=2`, 글 앞으로, 글자처럼 취급 아님, 가로·세로 기준 용지, 세로 오프셋 27245HU·20538HU)를
//! 함께 가진다. 한컴 PDF 는 두 사각형을 표 첫 조각이 있는 43쪽에 그린다.
//! 수정 전 rhwp(devel `2480af27df`)는 두 사각형을 표 끝 조각 뒤 46쪽에 실었다.
//! 수정은 devel `d7b5286cbf`(#9283 통합)의 `place_paper_overlay_on_opening_table_fragment` 다 —
//! 이 파일은 그 동작을 지키는 회귀 검사다.
//!
//! - 고정형: 전체 쪽수 46(= PDF).
//! - 관계형: 두 사각형은 문단 251 표의 첫 조각과 같은 쪽(43쪽)에 각각 한 번씩 그려지고, 표의 이어진
//!   조각이 있는 44~46쪽에는 없다. 세로 오프셋이 큰 `ci=1` 이 `ci=2` 보다 아래에 놓이며 둘 다 쪽 상자 안이다.
//!
//! 측정: 2026-11-23, source devel `622af208ee`(이 파일 외 코드 변경 없음),
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, `silhouette_raw_match_percent`,
//! 글꼴은 스크립트 기본 공급. 전체 46쪽(= PDF), 90% 미만 0쪽.
//!
//! - 43쪽(대상): Native 97.79%, fresh WASM 98.15%. 두 사각형 위치를 PDF 와 확대 대조함.
//! - 46쪽(표 끝 조각): Native 99.75%, fresh WASM 99.77%. 사각형이 없음을 확대 대조함.
//! - 전쪽 최저: Native 92.37%(17쪽), fresh WASM 96.86%(22쪽).
//! - 43쪽 Native raw 가 98% 미만이므로 절대 px 좌표는 고정하지 않는다.
//!
//! 증거: `mydocs/pr/assets/issue_9210/`(native·wasm 전쪽 TSV, 43·46쪽 review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9038/sme_announcement_split_table_successor.hwpx";
/// 한컴 PDF 의 쪽수.
const PDF_PAGES: u32 = 46;
/// 나뉜 표와 용지 기준 도형을 함께 가진 문단.
const HOST_PARA: usize = 251;
/// 표 첫 조각이 있는 쪽(0 기반 42 = 43쪽).
const OPENING_PAGE: u32 = 42;
/// 표의 이어진 조각이 있는 쪽(44~46쪽).
const CONTINUATION_PAGES: std::ops::RangeInclusive<u32> = 43..=45;
/// 용지 기준 글 앞 사각형(세로 오프셋 27245HU, 20538HU).
const LOWER_SHAPE: usize = 1;
const UPPER_SHAPE: usize = 2;

#[derive(Default)]
struct PageBoxes {
    host_tables: usize,
    host_shapes: Vec<(usize, BoundingBox)>,
}

/// 본문 문단 251 이 직접 가진 표·사각형을 모은다(표 칸 안 개체는 제외).
fn collect(node: &RenderNode, out: &mut PageBoxes) {
    match &node.node_type {
        RenderNodeType::Table(table) if table.para_index == Some(HOST_PARA) => {
            out.host_tables += 1;
            return;
        }
        RenderNodeType::Rectangle(rect)
            if rect.para_index == Some(HOST_PARA) && rect.cell_index.is_none() =>
        {
            if let Some(control) = rect.control_index {
                out.host_shapes.push((control, node.bbox));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );
    core
}

fn page_boxes(core: &DocumentCore, page: u32) -> (BoundingBox, PageBoxes) {
    let tree = core.build_page_render_tree(page).expect("쪽 렌더 트리");
    let mut out = PageBoxes::default();
    collect(&tree.root, &mut out);
    (tree.root.bbox, out)
}

#[test]
fn paper_shapes_are_drawn_once_on_the_opening_table_fragment_page() {
    let core = open();
    let (page_box, opening) = page_boxes(&core, OPENING_PAGE);
    assert!(
        opening.host_tables > 0,
        "43쪽에 문단 251 표의 첫 조각이 있어야 한다"
    );
    let shape = |control: usize| {
        let owned: Vec<_> = opening
            .host_shapes
            .iter()
            .filter(|(c, _)| *c == control)
            .collect();
        assert_eq!(
            owned.len(),
            1,
            "문단 251 의 사각형 ci={control} 은 표 첫 조각 쪽(43쪽)에 한 번 그려져야 한다"
        );
        owned[0].1
    };
    let lower = shape(LOWER_SHAPE);
    let upper = shape(UPPER_SHAPE);
    for (control, bbox) in [(LOWER_SHAPE, lower), (UPPER_SHAPE, upper)] {
        assert!(
            bbox.y >= page_box.y && bbox.y + bbox.height <= page_box.y + page_box.height,
            "사각형 ci={control} 이 43쪽 쪽 상자 밖이다"
        );
    }
    assert!(
        lower.y > upper.y,
        "세로 오프셋이 큰 ci=1({:.2}) 이 ci=2({:.2}) 보다 아래여야 한다",
        lower.y,
        upper.y
    );
}

#[test]
fn continuation_pages_of_the_table_do_not_repeat_or_receive_the_paper_shapes() {
    let core = open();
    for page in CONTINUATION_PAGES {
        let (_, boxes) = page_boxes(&core, page);
        assert!(
            boxes.host_tables > 0,
            "{}쪽에 문단 251 표의 이어진 조각이 있어야 한다",
            page + 1
        );
        assert!(
            boxes.host_shapes.is_empty(),
            "{}쪽에 문단 251 의 용지 기준 사각형 {:?} 이 그려졌다 — 앵커 쪽(43쪽)에만 있어야 한다",
            page + 1,
            boxes
                .host_shapes
                .iter()
                .map(|(c, _)| *c)
                .collect::<Vec<_>>()
        );
    }
}
