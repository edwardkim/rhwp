//! 관계형: HTML 원 속성·문자 서식과 편집·저장 후 내용/소유 보존을 검사한다.
//! 주석 갱신: 2026-11-12T14:12:12+09:00; 직전 파일 커밋: 2026-11-12T09:20:03+09:00 (db88c6180c9ae66a1270937a181e749ce48a0f7d).
//! 합성 공개 API 계약 검사이며 절대 px·고정 영역·쪽수 golden을 검사하지 않는다.
//! 별도 사전 시각 검증(2026-11-12, PDFium, 96dpi, 색 경계 보정 전 2px raw):
//! samples/issue8942의 border-hwp/hwpx와 속성 경계 4종, samples/issue8943의 leading-hwp/hwpx와
//! styled-caret/styled-cell, samples/issue8942-8943의 combined-hwp/hwpx 각각 전 1쪽에서
//! Native/fresh WASM 모두 100.00%, 전체 12문서·12쪽씩 최저도 양쪽 100.00%.
//! 수치는 위 저장본의 실물 측정이며 각 합성 API 입력의 픽셀 측정값으로 대신하지 않는다.
//! 독립 한컴 2020 Print와 확대 판독 및 입력/출력 SHA 근거: mydocs/pr/assets/bongseop-kim-20261112/README.md.
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

#[test]
fn attribute_names_and_quoted_values_do_not_alias_a_real_border() {
    for html in [
        r#"<table data-border="1"><tr><td>A</td></tr></table>"#,
        r#"<table data-border="1" border="0"><tr><td>A</td></tr></table>"#,
        r#"<table title='border="1"'><tr><td>A</td></tr></table>"#,
        r#"<table title='border="1"' border="0"><tr><td>A</td></tr></table>"#,
        r#"<table x-border="1" border="0"><tr><td>A</td></tr></table>"#,
    ] {
        assert_eq!(pasted_cell_borders(&paste(html)), [NONE], "{html}");
    }
    for html in [
        r#"<TABLE BORDER = '1'><TR><TD>A</TD></TR></TABLE>"#,
        r#"<table class=notice border="1"><tr><td>A</td></tr></table>"#,
        r#"<table title="a > b" border="1"><tr><td>A</td></tr></table>"#,
    ] {
        assert_eq!(pasted_cell_borders(&paste(html)), [SOLID], "{html}");
    }
}

#[test]
fn shared_attribute_parser_preserves_real_merged_cell_spans() {
    let core = paste(
        r#"<table border="1"><tr><td data-colspan="4" colspan = "2" rowspan='2'>Merged</td><td>Right</td></tr><tr><td>Below</td></tr></table>"#,
    );
    let table = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!((table.cells[0].row_span, table.cells[0].col_span), (2, 2));
    assert_eq!(table.cells[0].paragraphs[0].text, "Merged");
    let text: String = table
        .cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .map(|p| p.text.as_str())
        .collect();
    assert_eq!(text, "MergedRightBelow");
}

#[test]
fn shared_attribute_parser_preserves_image_units_and_quoted_boundaries() {
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    for width in ["48px", "36pt", "12.7mm", "1.27cm", "0.5in"] {
        let html = format!(
            r#"<p>앞<img data-width="999" width = '{width}' height="0.25in" src="data:image/png;base64,{png}">뒤</p>"#
        );
        let core = paste(&html);
        let paragraph = &core.document().sections[0].paragraphs[0];
        let picture = paragraph
            .controls
            .iter()
            .find_map(|c| {
                if let Control::Picture(p) = c {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        // HTML 96 px = 1 in, HWP 7200 unit = 1 in. 원 단위의 관계이며 화면 좌표 핀이 아니다.
        assert_eq!(picture.common.width, 3600, "{width}");
        assert_eq!(picture.common.height, 1800);
        assert_eq!(paragraph.text, "앞뒤");
        assert!(paragraph.control_text_positions().contains(&1));
    }
}

#[test]
fn table_border_and_contents_survive_both_formats_twice() {
    for format in ["hwp", "hwpx"] {
        let mut core = paste(
            r#"<p>BEFORE</p><table border="1"><tr><td>가</td><td>나</td></tr></table><p>AFTER</p>"#,
        );
        for _ in 0..2 {
            let bytes = if format == "hwp" {
                core.export_hwp_native().unwrap()
            } else {
                core.export_hwpx_native().unwrap()
            };
            core = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(pasted_cell_borders(&core), [SOLID, SOLID]);
            let body: Vec<_> = core.document().sections[0]
                .paragraphs
                .iter()
                .filter(|p| !p.text.is_empty())
                .map(|p| p.text.as_str())
                .collect();
            assert_eq!(body, ["BEFORE", "AFTER"]);
            let cells: Vec<_> = core.document().sections[0]
                .paragraphs
                .iter()
                .flat_map(|p| &p.controls)
                .filter_map(|c| {
                    if let Control::Table(t) = c {
                        Some(t)
                    } else {
                        None
                    }
                })
                .flat_map(|t| &t.cells)
                .flat_map(|c| &c.paragraphs)
                .filter(|p| !p.text.is_empty())
                .map(|p| p.text.as_str())
                .collect();
            assert_eq!(cells, ["가", "나"]);
        }
    }
}

#[test]
fn standalone_picture_style_uses_the_real_attribute() {
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    let core = paste(&format!(
        r#"<img data-style='text-align:right' style = "text-align:center" width="48" height="24" src="data:image/png;base64,{png}">"#
    ));
    let paragraph = core.document().sections[0]
        .paragraphs
        .iter()
        .find(|p| p.controls.iter().any(|c| matches!(c, Control::Picture(_))))
        .unwrap();
    assert_eq!(
        core.document().doc_info.para_shapes[paragraph.para_shape_id as usize].alignment,
        rhwp::model::style::Alignment::Center
    );
}
