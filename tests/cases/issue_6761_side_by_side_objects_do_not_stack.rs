//! [#6761] 한 문단의 개체 여럿을 측정이 세로로 합산해 행이 배치보다 커진다.
//!
//! ## 무엇이 문제였나
//!
//! 문단 하나가 `TopAndBottom` 개체를 여럿 달면 그 개체들은 저장 `horz` 오프셋대로
//! 놓인다 — 가로로 떨어져 있으면 **나란히**, 겹치면 **세로로** 쌓인다. 배치
//! (`table_layout`)는 저장 오프셋대로 그리는데, 측정
//! (`height_measurer::measure_non_inline_controls_height`)만 **무조건 더했다.**
//!
//! ```text
//!   1480000-201900042 <표 3-4> 셀[86] r=11,c=3  (선언 9531HU = 127.1px)
//!     그림 2장  w=7620 h=8049  horz=Column(off=-100)
//!               w=6311 h=7620  horz=Column(off=9064)   ← 가로로 떨어져 있다
//!     배치      x 499.8 / 621.9 나란히
//!     측정 전   107.3 + 101.6 = 208.9px  →  행 212.7px
//!     측정 후   max(107.3, 101.6) = 107.3px → 행 127.1px
//! ```
//!
//! 이 부풀림이 뒤 행을 밀어 `<표 3-4>` 의 마지막 행이 다음 쪽으로 넘어갔고, 그 쪽이
//! 이 문서의 마지막 여분 쪽이었다.
//!
//! ## 겹치는 개체는 여전히 쌓는다 — 반례
//!
//! 같은 문서 `<표 4-3> 기타 국내 인증 마크` 의 패널 칸은 그림 **세 장이 전부**
//! `horz=Column(off=123)` 로 **같은 가로 자리**에 있고 실제로 세로로 쌓인다. 여기서
//! 높이를 `max` 로 접으면 칸이 284.8px 로 줄어 그림이 쪽 밖으로 122.7px 나간다
//! (`layout-anomaly` off-canvas 0 → 1). 그래서 판정은 단순 `max` 가 아니라
//! **가로 구간이 겹치는 묶음 안에서만 더하고, 떨어진 묶음 사이에서는 가장 높은 묶음**
//! 을 쓴다.
//!
//! ## 기대값의 독립 근거
//!
//! - 저장 선언 `cell.height` = 9531HU = **127.1px**
//! - 한컴 정본 `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` 의
//!   해당 쪽을 96dpi 로 래스터해 가로 괘선을 찾은 실측 행 높이 **127px**
//! - 같은 정본의 패널 쪽에서 그림 세 장은 세로로 쌓여 있다(쪽을 넘어 이어진다)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::HorzAlign;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";

/// `<표 3-4>` 의 그 행에만 있는 글자(`1개 / 포장없음`). TextRun 이 쪼개지므로
/// 한 run 안에 온전히 들어가는 조각으로 찾는다.
const ROW_LABEL: &str = "포장없음";
/// 저장 선언 9531HU(127.1px) = 정본 괘선 실측 127px.
const ROW_HEIGHT: std::ops::RangeInclusive<f64> = 125.0..=129.0;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 셀 글자 → 그 셀의 높이.
fn cell_texts(node: &RenderNode, current: Option<f64>, out: &mut Vec<(String, f64)>) {
    let height = match node.node_type {
        RenderNodeType::TableCell(_) => Some(node.bbox.height),
        _ => current,
    };
    if let RenderNodeType::TextRun(ref run) = node.node_type {
        if let Some(h) = height {
            out.push((run.text.clone(), h));
        }
    }
    for child in &node.children {
        cell_texts(child, height, out);
    }
}

/// 페이지 상자를 벗어나는 노드의 최대 이탈량.
fn max_off_canvas(node: &RenderNode, page_bottom: f64) -> f64 {
    let own = (node.bbox.y + node.bbox.height - page_bottom).max(0.0);
    node.children
        .iter()
        .map(|child| max_off_canvas(child, page_bottom))
        .fold(own, f64::max)
}

fn target_cell_height(core: &DocumentCore) -> (u32, f64) {
    for page in 0..core.page_count() as u32 {
        let text = core.extract_page_text_native(page).unwrap_or_default();
        if !flat(&text).contains(&flat(ROW_LABEL)) {
            continue;
        }
        let tree = core.build_page_render_tree(page).expect("render tree");
        let mut cells = Vec::new();
        cell_texts(&tree.root, None, &mut cells);
        if let Some((_, height)) = cells
            .iter()
            .find(|(text, _)| flat(text).contains(&flat(ROW_LABEL)))
        {
            return (page, *height);
        }
    }
    panic!("`{ROW_LABEL}` 칸을 찾지 못했다");
}

/// 저장 Left offset을 같은 물리 x의 Right offset으로 바꾼 문서.
/// 그림 좌표는 같으므로 높이 측정도 원본과 같아야 한다.
fn right_aligned_equivalent() -> DocumentCore {
    let mut core = open();
    let mut doc = core.document().clone();
    let mut changed = 0;
    for section in &mut doc.sections {
        for paragraph in &mut section.paragraphs {
            for control in &mut paragraph.controls {
                let Control::Table(table) = control else {
                    continue;
                };
                let table_padding = table.padding;
                for cell in &mut table.cells {
                    if cell.row != 11 || cell.col != 3 {
                        continue;
                    }
                    let padding = if cell.apply_inner_margin {
                        cell.padding
                    } else {
                        cell.effective_padding(&table_padding)
                    };
                    let reference_width =
                        i64::from(cell.width) - i64::from(padding.left) - i64::from(padding.right);
                    let mut picture_index = 0;
                    for para in &mut cell.paragraphs {
                        for child in &mut para.controls {
                            let Control::Picture(picture) = child else {
                                continue;
                            };
                            if picture_index == 1 {
                                assert!(matches!(picture.common.horz_align, HorzAlign::Left));
                                let left_offset =
                                    i64::from(picture.common.horizontal_offset as i32);
                                let right_offset = reference_width
                                    - i64::from(picture.common.width as i32)
                                    - left_offset;
                                picture.common.horizontal_offset =
                                    u32::try_from(right_offset).expect("오른쪽 offset 범위");
                                picture.common.horz_align = HorzAlign::Right;
                                changed += 1;
                            }
                            picture_index += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(changed, 1, "대상 두 번째 그림을 찾지 못했다");
    core.set_document(doc);
    core
}

/// 가로로 떨어진 개체는 세로로 쌓지 않는다 — 행이 저장 선언·정본과 같아진다.
#[test]
fn horizontally_separated_objects_share_one_band() {
    let core = open();
    let pages = core.page_count() as u32;
    let mut found = None;
    for page in 0..pages {
        let text = core.extract_page_text_native(page).unwrap_or_default();
        if !flat(&text).contains(&flat(ROW_LABEL)) {
            continue;
        }
        let tree = core.build_page_render_tree(page).expect("render tree");
        let mut cells = Vec::new();
        cell_texts(&tree.root, None, &mut cells);
        if let Some((_, height)) = cells
            .iter()
            .find(|(text, _)| flat(text).contains(&flat(ROW_LABEL)))
        {
            found = Some((page, *height));
            break;
        }
    }
    let (page, height) = found.unwrap_or_else(|| panic!("`{ROW_LABEL}` 칸을 찾지 못했다"));
    assert!(
        ROW_HEIGHT.contains(&height),
        "`{ROW_LABEL}` 행({}쪽)이 {height:.1}px 이다 — 저장 선언 127.1px · 정본 실측 127px \
         기준 {ROW_HEIGHT:?} 안이어야 한다. 나란히 놓이는 그림 2장을 세로로 더하면 \
         212.7px 로 커진다 (#6761)",
        page + 1
    );
}

/// 정렬 표현만 Left에서 Right로 바꿔도 물리 구간이 같으면 같은 높이를 예약한다.
/// offset만 비교하면 두 그림을 떨어진 band로 보고 행을 212.7px로 부풀린다.
#[test]
fn equivalent_right_alignment_keeps_the_same_row_height() {
    let left = open();
    let right = right_aligned_equivalent();
    let (left_page, left_height) = target_cell_height(&left);
    let (right_page, right_height) = target_cell_height(&right);
    assert_eq!(
        right.page_count(),
        left.page_count(),
        "동일 위치 그림이 쪽 수를 바꿨다"
    );
    assert_eq!(
        right_page, left_page,
        "동일 위치 그림의 대상 행 쪽이 바뀌었다"
    );
    assert!(
        (right_height - left_height).abs() <= 0.5,
        "같은 물리 위치의 Right 정렬 그림 때문에 행 높이가 {left_height:.1}px -> \
         {right_height:.1}px 로 달라졌다"
    );
}

/// 반례 — 가로로 겹치는 개체는 그대로 쌓는다. 접으면 그림이 쪽 밖으로 나간다.
#[test]
fn horizontally_overlapping_objects_still_stack() {
    let core = open();
    let pages = core.page_count() as u32;
    let page_bottom = 1122.5;
    let mut worst: f64 = 0.0;
    for page in 0..pages {
        let tree = core.build_page_render_tree(page).expect("render tree");
        worst = worst.max(max_off_canvas(&tree.root, page_bottom));
    }
    assert!(
        worst <= 1.0,
        "쪽 상자 밖으로 {worst:.1}px 나간 노드가 있다. 같은 가로 자리에 쌓인 그림 \
         (`<표 4-3>` 패널 3장)의 높이를 `max` 로 접으면 칸이 284.8px 로 줄어 \
         122.7px 가 쪽 밖으로 나간다 (#6761 반례)"
    );
}
