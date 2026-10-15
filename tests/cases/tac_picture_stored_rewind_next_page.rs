//! 저장 vpos 가 되감긴 atomic TAC 그림은 하단 여백 top-fit 스필 대상이 아니다.
//!
//! `atomic TAC top-fit`(#409)은 분할할 수 없는 인라인 그림이 본문 하단을 60px 안에서
//! 넘쳐도 현재 쪽에 남긴다. 그런데 그 그림 문단의 저장 줄이 앞 문단보다 위로 되감기면
//! 한/글은 그 문단을 다음 쪽 첫머리에 둔 것이다(#5755 의 되감김 + 전체 fit 실패).
//! 1480000-201900698 `pi=88`(그림 6, 370px)은 남은 공간보다 27px 커서 19쪽에 남았고
//! 쪽이 984.7px 로 본문 948.7px 를 넘겼다. 한/글 2020 PDF 는 그 그림을 20쪽 첫머리에
//! 두고, 이후 쪽 경계가 하나씩 앞당겨지던 어긋남(21쪽 그림 7 수용)도 여기서 시작했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::image::Picture;
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, HorzRelTo, TextWrap, VertRelTo};
use rhwp::model::style::ParaShape;

const PICTURE_HEIGHT: i32 = 26_796;

fn line(vertical_pos: i32, line_height: i32, line_spacing: i32) -> LineSeg {
    LineSeg {
        text_start: 0,
        vertical_pos,
        line_height,
        text_height: line_height,
        baseline_distance: line_height * 85 / 100,
        line_spacing,
        segment_width: 48_188,
        ..Default::default()
    }
}

fn text_paragraph(text: &str, seg: LineSeg) -> Paragraph {
    Paragraph {
        text: text.to_string(),
        char_count: text.encode_utf16().count() as u32,
        char_offsets: (0..text.encode_utf16().count() as u32).collect(),
        line_segs: vec![seg],
        ..Default::default()
    }
}

/// 본문 70,018 HU 중 앞 두 문단이 45,620 HU 를 쓴 뒤 357px 그림 문단이 온다.
/// 그림은 남은 24,398 HU 보다 2,398 HU(약 32px) 커서 #409 의 60px 허용 안에 든다.
fn document(picture_vpos: i32) -> DocumentCore {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];

    let picture = Picture {
        common: CommonObjAttr {
            width: 40_000,
            height: PICTURE_HEIGHT as u32,
            treat_as_char: true,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Para,
            text_wrap: TextWrap::TopAndBottom,
            ..Default::default()
        },
        ..Default::default()
    };
    let picture_paragraph = Paragraph {
        char_count: 9,
        line_segs: vec![line(picture_vpos, PICTURE_HEIGHT, 660)],
        controls: vec![Control::Picture(Box::new(picture))],
        ..Default::default()
    };

    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59_528,
        height: 84_188,
        margin_left: 5_669,
        margin_right: 5_669,
        margin_top: 4_535,
        margin_bottom: 3_685,
        margin_header: 3_118,
        margin_footer: 2_834,
        ..Default::default()
    };
    section.paragraphs = vec![
        text_paragraph("앞 문단", line(300, 20_000, 660)),
        text_paragraph("둘째 문단", line(20_960, 24_000, 660)),
        picture_paragraph,
        text_paragraph("뒤 문단", line(picture_vpos + PICTURE_HEIGHT + 660, 1_100, 660)),
    ];
    doc.sections.push(section);

    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    core
}

fn page_has_paragraph(core: &DocumentCore, page: u32, para_index: u64) -> bool {
    core.dump_page_items_json(Some(page))
        .as_array()
        .and_then(|pages| pages.first())
        .and_then(|page| page.get("columns"))
        .and_then(|columns| columns.as_array())
        .into_iter()
        .flatten()
        .filter_map(|column| column.get("items").and_then(|items| items.as_array()))
        .flatten()
        .any(|item| item.get("paraIndex").and_then(|value| value.as_u64()) == Some(para_index))
}

#[test]
fn rewound_atomic_picture_starts_the_next_page() {
    // 저장 줄이 쪽 위쪽 띠(300 HU)로 되감긴다 — 한/글이 그림 문단에서 쪽을 끊었다.
    let core = document(300);

    assert_eq!(core.page_count(), 2);
    assert!(
        page_has_paragraph(&core, 0, 1),
        "되감김 직전 문단은 첫 쪽에 남아야 한다"
    );
    assert!(
        !page_has_paragraph(&core, 0, 2),
        "저장 vpos 가 되감긴 그림 문단은 첫 쪽 하단 여백으로 스필하지 않아야 한다"
    );
    assert!(
        page_has_paragraph(&core, 1, 2),
        "그림 문단은 다음 쪽 첫머리에 있어야 한다"
    );
    assert!(
        page_has_paragraph(&core, 1, 3),
        "뒤 문단은 그림을 따라 다음 쪽에 있어야 한다"
    );
}

#[test]
fn continuing_atomic_picture_keeps_the_existing_top_fit_spill() {
    // 저장 줄이 앞 문단 바로 뒤(45,620 HU)에서 이어진다 — 한/글이 같은 쪽에 둔 배치.
    let core = document(45_620);

    assert!(
        page_has_paragraph(&core, 0, 2),
        "되감김이 없는 atomic 그림은 #409 top-fit 대로 첫 쪽에 남아야 한다"
    );
}

#[test]
fn sample_1480000_figure_6_starts_page_20() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5941/1480000-201900698-native-neartop-reset.hwp");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("parse 1480000-201900698");

    // 19쪽(0-기준 18)은 pi=87 빈 문단으로 끝나고, 그림 6(pi=88)과 캡션(pi=89)은
    // 한/글 2020 PDF 처럼 20쪽(0-기준 19)에서 시작한다.
    assert!(page_has_paragraph(&core, 18, 87));
    assert!(
        !page_has_paragraph(&core, 18, 88),
        "그림 6 은 19쪽 하단 여백으로 스필하지 않아야 한다"
    );
    assert!(page_has_paragraph(&core, 19, 88));
    assert!(page_has_paragraph(&core, 19, 89));
}
