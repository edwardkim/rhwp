//! [#8362] PDF 편집 우선 가져오기는 본문 글자 그림 사이에 다른 그림이 끼고 겹치면 문단 전체를
//! 위치 고정 낱말 글상자로 남겼다. 그 줄들은 흐름에서 자리를 차지하지 않아 한/글이 다음 글을 그
//! 자리로 끌어올려 겹쳐 그렸다(시험지 9쪽 «마흐디» 상자 문항 위에 앞 문단이 겹침). 글자를 가리지
//! 않는 그림(테두리 선·채움 없는 상자)과, 겹친 글자가 모두 나중에 그려진 그림(낱말을 두른 상자)은
//! 순서를 바꿔도 화면이 같으므로 본문은 흐름 문단이 되고, 가리는 그림은 본문 아래(글 뒤)로 간다.
#![cfg(feature = "pdf-import")]

use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use rhwp::model::style::FillType;
use rhwp::parser::pdf::{PdfImportLayout, PdfImportOptions};
use rhwp::DocumentCore;

fn import_editable() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21_언어_기출_편집가능본-2010.pdf");
    let options = PdfImportOptions {
        layout: PdfImportLayout::Editable,
        ..PdfImportOptions::default()
    };
    DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &options).unwrap()
}

fn flow_text(core: &DocumentCore) -> String {
    core.document()
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn boxed_word_paragraphs_join_the_flow() {
    // 낱말을 두른 상자가 낀 문단들은 흐름 본문 문단이어야 한다 — 위치 고정 글상자로 남으면
    // 한/글이 그 자리에 다음 글을 겹쳐 그린다.
    let core = import_editable();
    let text = flow_text(&core);
    for probe in [
        "에 대한 이해로 가장 적절한 것은",
        "는 ‘정치적인 것’의 활성화를 제기하였다",
    ] {
        assert!(
            text.contains(probe),
            "흐름 본문 문단에 «{probe}» 가 있어야 한다"
        );
    }
}

#[test]
fn only_occluding_paints_move_behind_the_body() {
    // 글 뒤로 보낸 그림은 본문 글자를 가릴 수 있는 그림(채움 있음)뿐이고, 낱말 상자 채움이
    // 흐름 본문 위(글 앞)에 남아 낱말을 덮지 않는다.
    let core = import_editable();
    let mut behind = 0;
    for paragraph in core.document().sections.iter().flat_map(|s| &s.paragraphs) {
        for control in &paragraph.controls {
            let Control::Shape(shape) = control else {
                continue;
            };
            if shape.common().text_wrap != TextWrap::BehindText {
                continue;
            }
            let drawing = shape.drawing().expect("글 뒤 도형의 그리기 속성");
            assert!(
                drawing.fill.fill_type != FillType::None || drawing.shadow_type != 0,
                "글 뒤로 보낸 도형은 가리는 그림이어야 한다"
            );
            behind += 1;
        }
    }
    assert!(behind > 0, "본문 아래로 보낸 낱말 상자 {behind}개");
}
