//! [Issue #6972] 전면 크기 TAC 그림이 **자기 저장 줄이 있는 쪽**으로 라우팅되는지 잠근다.
//!
//! ## 무엇이 어긋났나
//!
//! `56288_규제영향분석서.hwp` 문단 0 은 표지 스캔 그림(글자처럼 취급, `72347HU = 964.6px`)과
//! 글자 `< 규제 개요 >` 를 각각 자기 저장 줄에 담는다.
//!
//! ```text
//!   ls[0] text_start=0   line_height=72347   ← 표지 그림이 소유한 줄
//!   ls[1] text_start=24  line_height=1500    ← '< 규제 개요 >'
//! ```
//!
//! 이 문단은 쪽을 넘어 갈라진다(그림 줄이 본문 971.3px 를 거의 채운다). 그런데
//! `find_inline_control_target_page` 가 개체의 소유 줄을 `control_line_seg_index` 의
//! **글자 위치 투영**으로 찾는 탓에, 개체가 문단의 모든 글자 **앞**에 있으면 첫 글자의
//! 줄(= 개체 줄의 **다음** 줄)을 돌려준다. 컨트롤 문자가 0 이고 첫 글자 offset 이 0 이면
//! `p >= start_txt` 가 `0 >= 0` 으로 참이 되기 때문이다.
//!
//! 그 결과 표지 그림이 **자기 줄이 없는 뒤 조각**으로 라우팅돼 1쪽과 2쪽에 두 번 그려졌다.
//!
//! ## 판정은 기하다
//!
//! 같은 quirk 를 `#6078` 이 이미 만나 기하로 피해 갔다 — "표를 담을 수 있는 줄 높이를 가진
//! seg 가 표 줄이다". 여기서도 저장 `line_height` 가 개체의 흐름 높이와 같은 줄을 소유 줄로
//! 짚는다.
//!
//! ## 이 시험이 잠그는 것
//!
//! 글자 위치 투영으로 되돌아가면 소유 줄이 `1` 이 되고, 그 줄은 **현재 조각**(`current_items`)
//! 에 있으므로 `find_inline_control_target_page` 는 `None`(= 현재 쪽에 둔다)을 돌려준다.
//! 곧 `Some((0, 0))` 이라는 단언 하나가 기하 판정이 살아 있음을 증명한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::image::Picture;
use rhwp::model::page::{ColumnDef, PageDef};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::renderer::page_layout::PageLayoutInfo;
use rhwp::renderer::pagination::{
    find_inline_control_target_page, ColumnContent, PageContent, PageItem,
};

/// 표지 그림의 흐름 높이. 원본 `hp:sz` 와 `ls[0].line_height` 가 같은 값이다.
const PICTURE_HU: i32 = 72347;

fn a4_page_def() -> PageDef {
    PageDef {
        width: 59528,
        height: 84188,
        margin_left: 8504,
        margin_right: 8504,
        margin_top: 5669,
        margin_bottom: 4252,
        margin_header: 4252,
        margin_footer: 4252,
        margin_gutter: 0,
        ..Default::default()
    }
}

/// 56288 문단 0 의 형상 — 전면 TAC 그림이 첫 줄을 통째로 소유하고, 유일한 글자는 둘째 줄에 있다.
fn cover_picture_paragraph() -> Paragraph {
    let mut picture = Picture::default();
    picture.common.treat_as_char = true;
    picture.common.height = PICTURE_HU as _;
    Paragraph {
        text: "< 규제 개요 >".to_string(),
        char_offsets: (8..17).collect(),
        controls: vec![Control::Picture(Box::new(picture))],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: PICTURE_HU,
                ..Default::default()
            },
            LineSeg {
                text_start: 8,
                line_height: 1500,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn page_with_paragraph_fragment(start_line: usize, end_line: usize) -> PageContent {
    PageContent {
        page_index: 0,
        page_number: 1,
        page_number_restarted: false,
        section_index: 0,
        layout: PageLayoutInfo::from_page_def(&a4_page_def(), &ColumnDef::default(), 96.0),
        column_contents: vec![ColumnContent {
            column_index: 0,
            start_height: 0.0,
            endnote_flow: false,
            items: vec![PageItem::PartialParagraph {
                para_index: 0,
                start_line,
                end_line,
            }],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
            wrap_anchors: Default::default(),
            overlay_continuations: Vec::new(),
            overlay_cuts: Vec::new(),
            inline_placements: Default::default(),
            inline_flow_plans: Default::default(),
            paragraph_float_placements: Default::default(),
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
        ladder_band_tables: Vec::new(),
    }
}

#[test]
fn tac_full_page_picture_routes_to_the_page_holding_its_own_stored_line() {
    let para = cover_picture_paragraph();
    let first_page = page_with_paragraph_fragment(0, 1);
    // 조판이 둘째 줄까지 온 시점 — 그림을 여기 두면 표지가 두 쪽에 그려진다.
    let current_items = vec![PageItem::PartialParagraph {
        para_index: 0,
        start_line: 1,
        end_line: 2,
    }];

    assert_eq!(
        find_inline_control_target_page(&[first_page], &current_items, 0, 0, &para),
        Some((0, 0)),
        "전면 TAC 그림은 자기 저장 줄이 있는 첫 쪽으로 라우팅해야 한다 \
         (글자 위치 투영으로 되돌아가면 None 이 되어 현재 쪽에 남는다)"
    );
}

#[test]
fn ambiguous_same_height_lines_keep_the_character_position_projection() {
    // 같은 높이의 줄이 둘이면 어느 줄이 개체 줄인지 기하로 가릴 수 없다(#2004 이미지 스택).
    // 그때는 종전 글자 위치 투영으로 되돌아가야 하고, 그 투영은 둘째 줄을 가리키므로
    // 현재 조각에 그 줄이 있는 이 배치에서는 `None`(= 현재 쪽 유지)이 나온다.
    let mut para = cover_picture_paragraph();
    para.line_segs[1].line_height = PICTURE_HU;
    let first_page = page_with_paragraph_fragment(0, 1);
    let current_items = vec![PageItem::PartialParagraph {
        para_index: 0,
        start_line: 1,
        end_line: 2,
    }];

    assert_eq!(
        find_inline_control_target_page(&[first_page], &current_items, 0, 0, &para),
        None,
        "높이가 같은 줄이 둘이면 기하 판정을 쓰지 않고 종전 경로를 유지해야 한다"
    );
}
