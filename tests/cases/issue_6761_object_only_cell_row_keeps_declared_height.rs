//! [#6761] 개체만 든 칸의 빈 글줄을 개체 높이에 더해 행이 부풀고, 마지막 행이
//! 다음 쪽으로 밀려 여분 쪽이 생긴다.
//!
//! ## 무엇이 문제였나
//!
//! `#6660` 이 세운 계약 — *"글자가 하나도 없는 문단의 줄은 그 개체를 담는 자리이지
//! 개체 아래에 따로 놓이는 글줄이 아니다"* — 은 **개체가 선언 칸보다 클 때만**
//! 적용됐다(`non_inline_h > declared_cell_h`). 개체가 칸 안에 들어가면 줄과 개체를
//! 둘 다 세어, 칸이 선언보다 커지고 그 행이 행 전체를 밀어 올렸다.
//!
//! `<표 4-1> 국내외 유사 마크 현황`(`pi=118`)의 `인증마크` 칸들이 그 모양이다 —
//! 문단 하나, 글자 없음, `TopAndBottom` 그림 하나.
//!
//! ```text
//!   r=4 (중국 CCC마크)  선언 5547HU = 74.0px   그림 65.8px   빈 글줄 13.3px
//!     수정 전  content 13.3 + 65.8 = 79.1  → +pad 3.8 → 행 82.9px  (선언 초과)
//!     수정 후  content        65.8         → +pad 3.8 → 69.6 ≤ 선언 → 행 74.0px
//! ```
//!
//! 이 부풀림이 행마다 쌓여(11행에 약 60px) 마지막 `덴마크` 행이 쪽 바닥을 넘었고,
//! 그 행의 그림만 이어받은 **여분 쪽**이 생겼다(반복 제목행 + 그림 한 장).
//!
//! ## 기대값의 독립 근거 — 저장 선언값과 한컴 정본, 둘 다 같은 값을 말한다
//!
//! 저장 `cell.height`(HWPUNIT, `rhwp dump`):
//!
//! ```text
//!   r=3 유럽 5191HU = 69.2px   r=4 중국 5547HU = 74.0px   r=13 덴마크 4215HU = 56.2px
//! ```
//!
//! 저장소 추적 정본 `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf`
//! 의 55쪽(문서 77쪽)을 96dpi 로 래스터해 가로 괘선 픽셀 행을 찾은 실측:
//!
//! ```text
//!   괘선 y  134 162 231 305 385 462 535 600 679 758 826 900 969
//!   행 높이      28  69  74  80  77  73  65  79  79  68  74  69
//!               제목 유럽 중국 일본 캐나다 멕시코 터키 베트남 아랍 러시아 아르헨티나 덴마크
//! ```
//!
//! 중국 행은 저장 선언(74.0)과 정본 실측(74)이 같다. rhwp 는 82.9px 이었다.
//! 덴마크 행은 정본 69px 이고 수정 전에는 56.6px 만 담고 나머지를 다음 쪽으로 넘겼다.
//!
//! ## 이 시험이 닫지 않는 것
//!
//! 이 문서는 정본 103쪽 / 수정 전 105쪽 → 수정 후 **104쪽**이다. 남은 한 쪽은
//! `제품사진` 칸에 **그림 두 장**이 든 행(정본 127px, rhwp 212.7px)이 세로로 합산되는
//! 별개 축(`#2226` 계열)이라 여기서 다루지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// `#6761`·`#6782` 가 쓰는 것과 같은 실물 원본이다.
const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";

/// `<표 4-1>` 의 마지막 행(덴마크)에만 있는 글자. 이 표를 담은 쪽을 고른다.
const LAST_ROW_MARK: &str = "DEMKO";
/// 그림이 가장 큰 행 중 하나. 저장 선언 5547HU = 74.0px.
const PICTURE_ROW_LABEL: &str = "중국";
/// 표 바로 뒤 본문. 여분 쪽이 없으면 표 쪽 **바로 다음** 쪽에 온다.
const NEXT_BODY_HEAD: &str = "그외제품인증마크에대한";

/// 저장 선언 5547HU(74.0px) = 정본 실측 74px.
const PICTURE_ROW_HEIGHT: std::ops::RangeInclusive<f64> = 72.5..=75.5;
/// 정본 실측 69px(괘선 900→969). 수정 전에는 56.6px 만 담고 잘렸다.
const LAST_ROW_HEIGHT: std::ops::RangeInclusive<f64> = 67.5..=70.5;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// `<표 4-1>` 을 담은 쪽(0-기반).
fn mark_table_page(core: &DocumentCore) -> u32 {
    (0..core.page_count() as u32)
        .find(|page| {
            core.extract_page_text_native(*page)
                .unwrap_or_default()
                .contains(LAST_ROW_MARK)
        })
        .expect("`DEMKO` 가 있는 쪽")
}

/// 렌더 트리에서 (칸 글자, 칸 높이) 를 모은다.
fn cell_texts(node: &RenderNode, current: Option<f64>, out: &mut Vec<(String, f64)>) {
    let height = match node.node_type {
        RenderNodeType::TableCell(_) => {
            out.push((String::new(), node.bbox.height));
            Some(node.bbox.height)
        }
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

fn row_height(core: &DocumentCore, page: u32, label: &str) -> f64 {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut cells = Vec::new();
    cell_texts(&tree.root, None, &mut cells);
    cells
        .iter()
        .find(|(text, _)| flat(text) == flat(label))
        .map(|(_, h)| *h)
        .unwrap_or_else(|| panic!("{page}쪽에서 `{label}` 칸을 찾지 못했다"))
}

/// 개체가 칸 안에 들어가도 빈 글줄을 개체 위에 더 쌓지 않는다.
#[test]
fn object_only_cell_row_matches_declared_and_hancom_height() {
    let core = open();
    let page = mark_table_page(&core);
    let height = row_height(&core, page, PICTURE_ROW_LABEL);
    assert!(
        PICTURE_ROW_HEIGHT.contains(&height),
        "`{PICTURE_ROW_LABEL}` 행이 {height:.1}px 이다 — 저장 선언 74.0px · 정본 실측 74px \
         기준 {PICTURE_ROW_HEIGHT:?} 안이어야 한다. 글자 없는 문단의 줄(13.3px)을 그림 \
         (65.8px) 위에 더 세면 82.9px 로 커진다 (#6761)"
    );
}

/// 마지막 행이 그 쪽에서 온전히 끝난다 — 그림만 이어받는 여분 쪽이 없다.
#[test]
fn last_row_is_not_split_into_an_extra_page() {
    let core = open();
    let page = mark_table_page(&core);
    let height = row_height(&core, page, LAST_ROW_MARK);
    assert!(
        LAST_ROW_HEIGHT.contains(&height),
        "마지막 행이 {height:.1}px 만 담았다 — 정본 실측 69px 기준 {LAST_ROW_HEIGHT:?} \
         안이어야 한다. 앞 행들이 행마다 부풀면 이 행이 쪽 바닥을 넘어 그림만 다음 쪽으로 \
         넘어간다 (#6761)"
    );

    let next = core
        .extract_page_text_native(page + 1)
        .unwrap_or_default()
        .replace(char::is_whitespace, "");
    let head_at = next.find(NEXT_BODY_HEAD).map(|byte_offset| {
        next[..byte_offset].chars().count() // 글머리표 등 앞 글자 수
    });
    assert!(
        head_at.is_some_and(|chars| chars <= 3),
        "표 다음 쪽은 본문 `{NEXT_BODY_HEAD}` 로 시작해야 한다(글머리표 제외). 수정 전에는 \
         반복 제목행과 그림 한 장만 든 여분 쪽이 먼저 왔다. 실제 머리: {}",
        next.chars().take(40).collect::<String>()
    );
}
