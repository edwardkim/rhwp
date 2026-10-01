//! 글 사이 `<img>`(data: URI)가 든 HTML 을 붙이면 그 자리에 글자처럼 취급하는 그림이 들어간다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

/// 1×1 PNG
const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";

fn img() -> String {
    format!(r#"<img src="data:image/png;base64,{PNG}" width="20" height="20">"#)
}

fn document_with(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank document");
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).expect("text");
    }
    core
}

fn paragraphs(core: &DocumentCore) -> &[Paragraph] {
    &core.document().sections[0].paragraphs
}

/// 문단 글과, 글자처럼 취급하는 그림마다 그 앞 글자 수
fn text_and_pictures(paragraph: &Paragraph) -> (&str, Vec<usize>) {
    let pictures = paragraph
        .controls
        .iter()
        .zip(paragraph.control_text_positions())
        .filter(|(control, _)| matches!(control, Control::Picture(p) if p.common.treat_as_char))
        .map(|(_, position)| position)
        .collect();
    (paragraph.text.as_str(), pictures)
}

#[test]
fn paragraph_image_between_text_is_pasted_inline_and_saved() {
    let mut core = document_with("");
    core.paste_html_native(0, 0, 0, &format!("<p>앞{}뒤</p>", img()))
        .expect("HTML paste");

    assert_eq!(paragraphs(&core).len(), 1);
    assert_eq!(text_and_pictures(&paragraphs(&core)[0]), ("앞뒤", vec![1]));

    for (format, bytes) in [
        ("HWP", core.export_hwp_native()),
        ("HWPX", core.export_hwpx_native()),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes.expect("export")).expect(format);
        assert_eq!(
            text_and_pictures(&paragraphs(&reopened)[0]),
            ("앞뒤", vec![1]),
            "{format} 저장 뒤에도 그림이 글 사이에 남아야 한다"
        );
    }
}

#[test]
fn paragraph_images_merge_into_the_caret_paragraph_with_their_styles() {
    let mut core = document_with("가나");
    let result = core
        .paste_html_native(
            0,
            0,
            1,
            &format!("<p>{0}앞<b>뒤</b>{0}</p><p>다음</p>", img()),
        )
        .expect("HTML paste");

    assert_eq!(result, r#"{"ok":true,"paraIdx":1,"charOffset":2}"#);
    let paragraphs = paragraphs(&core);
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(text_and_pictures(&paragraphs[0]), ("가앞뒤", vec![1, 3]));
    assert_eq!(text_and_pictures(&paragraphs[1]), ("다음나", vec![]));

    let bold = |char_offset| {
        let id = paragraphs[0]
            .char_shape_id_at(char_offset)
            .expect("글자 모양");
        core.document().doc_info.char_shapes[id as usize].bold
    };
    assert!(!bold(1), "그림 뒤 '앞'은 굵지 않다");
    assert!(bold(2), "<b> 구간은 그림 자리만큼 밀린 '뒤'에서 시작한다");
}

#[test]
fn list_item_image_keeps_its_place_after_the_bullet() {
    let mut core = document_with("");
    core.paste_html_native(0, 0, 0, &format!("<ul><li>앞{}뒤</li></ul>", img()))
        .expect("HTML paste");

    assert_eq!(
        text_and_pictures(&paragraphs(&core)[0]),
        ("• 앞뒤", vec![3])
    );
}

/// 문서 본문의 그림 수. 붙여넣은 그림 데이터는 모두 그림 하나가 써야 한다.
fn picture_count(core: &DocumentCore) -> usize {
    paragraphs(core)
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .filter(|control| matches!(control, Control::Picture(_)))
        .count()
}

#[test]
fn image_only_list_item_and_span_keep_their_image() {
    for (html, expected) in [
        (format!("<ul><li>{}</li></ul>", img()), ("• ", vec![2])),
        (format!("<span>{}</span>", img()), ("", vec![0])),
    ] {
        let mut core = document_with("");
        core.paste_html_native(0, 0, 0, &html).expect("HTML paste");

        assert_eq!(text_and_pictures(&paragraphs(&core)[0]), expected, "{html}");
        assert_eq!(
            core.document().doc_info.bin_data_list.len(),
            picture_count(&core),
            "{html}: 쓰지 않는 그림 데이터가 문서에 남으면 안 된다"
        );
    }
}

#[test]
fn cell_paste_drops_the_image_without_shifting_styles_or_keeping_its_data() {
    for html in [
        format!("<p>앞{}<b>뒤</b></p>", img()),
        format!("<p>앞{}<b>뒤</b></p><p>다음</p>", img()),
    ] {
        let mut core = document_with("");
        core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
        let control_idx = paragraphs(&core)[0]
            .controls
            .iter()
            .position(|control| matches!(control, Control::Table(_)))
            .expect("표");
        core.paste_html_in_cell_native(0, 0, control_idx, 0, 0, 0, &html)
            .expect("셀 HTML 붙여넣기");

        let Control::Table(table) = &paragraphs(&core)[0].controls[control_idx] else {
            unreachable!()
        };
        let cell = &table.cells[0].paragraphs[0];
        assert_eq!(text_and_pictures(cell), ("앞뒤", vec![]), "{html}");
        let bold = |char_offset| {
            let id = cell.char_shape_id_at(char_offset).expect("글자 모양");
            core.document().doc_info.char_shapes[id as usize].bold
        };
        assert!(!bold(0), "{html}: '앞'은 굵지 않다");
        assert!(bold(1), "{html}: '뒤'만 굵다");
        assert!(
            core.document().doc_info.bin_data_list.is_empty(),
            "{html}: 셀에 넣지 않은 그림 데이터가 문서에 남으면 안 된다"
        );
    }
}

#[test]
fn pasting_into_the_first_paragraph_keeps_its_column_definition() {
    for html in [
        format!("<p>{}</p>", img()),
        "<table><tr><td>표</td></tr></table>".to_string(),
    ] {
        let mut core = document_with("");
        core.set_column_def_native(0, 2, 0, true, 0).expect("2단");
        core.paste_html_native(0, 0, 0, &html).expect("HTML paste");

        let reopened =
            DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX")).expect("HWPX");
        let columns: Vec<u16> = paragraphs(&reopened)[0]
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::ColumnDef(column) => Some(column.column_count),
                _ => None,
            })
            .collect();
        assert_eq!(columns, [2], "{html}: 첫 문단의 단 정의가 남아야 한다");
        assert_eq!(picture_count(&reopened), picture_count(&core), "{html}");
        assert_eq!(
            picture_count(&core),
            usize::from(html.starts_with("<p>")),
            "{html}"
        );
    }
}
