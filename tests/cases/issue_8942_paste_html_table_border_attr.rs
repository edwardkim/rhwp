//! `<table border="1">` 처럼 `border` 속성으로 테두리를 준 HTML 표를 붙이면 칸에 실선이 남는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::BorderLineType;

/// 붙인 표의 칸마다 네 변(left, right, top, bottom)의 선 종류
fn pasted_cell_borders(core: &DocumentCore) -> Vec<[BorderLineType; 4]> {
    let document = core.document();
    let table = document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("pasted table");
    table
        .cells
        .iter()
        .map(|cell| {
            let fill = &document.doc_info.border_fills[cell.border_fill_id as usize - 1];
            fill.borders.map(|line| line.line_type)
        })
        .collect()
}

fn paste(html: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank document");
    core.paste_html_native(0, 0, 0, html).expect("HTML paste");
    core
}

const SOLID: [BorderLineType; 4] = [BorderLineType::Solid; 4];
const NONE: [BorderLineType; 4] = [BorderLineType::None; 4];

#[test]
fn border_attribute_draws_solid_cell_borders() {
    for html in [
        r#"<table border="1"><tr><td>가</td><td>나</td></tr></table>"#,
        r#"<table border='2'><tr><th>가</th><td>나</td></tr></table>"#,
        r#"<TABLE BORDER="1"><TR><TD>가</TD><TD>나</TD></TR></TABLE>"#,
    ] {
        assert_eq!(pasted_cell_borders(&paste(html)), [SOLID, SOLID], "{html}");
    }
}

#[test]
fn zero_or_missing_border_attribute_keeps_cells_borderless() {
    for html in [
        r#"<table border="0"><tr><td>가</td><td>나</td></tr></table>"#,
        r#"<table><tr><td>가</td><td>나</td></tr></table>"#,
    ] {
        assert_eq!(pasted_cell_borders(&paste(html)), [NONE, NONE], "{html}");
    }
}

#[test]
fn cell_css_overrides_the_border_attribute() {
    let html = r#"<table border="1"><tr><td style="border:none">가</td><td style="border:2px dashed #000">나</td></tr></table>"#;
    assert_eq!(
        pasted_cell_borders(&paste(html)),
        [NONE, [BorderLineType::Dash; 4]]
    );
}

#[test]
fn border_attribute_survives_hwpx_save_and_reopen() {
    let core = paste(r#"<table border="1"><tr><td>가</td><td>나</td></tr></table>"#);
    let hwpx = core.export_hwpx_native().expect("HWPX export");
    let reopened = DocumentCore::from_bytes(&hwpx).expect("reopen HWPX");
    assert_eq!(pasted_cell_borders(&reopened), [SOLID, SOLID]);
}
