#![cfg(not(target_arch = "wasm32"))]
//! HTML 붙이기: `<p style>`·`<li style>` 의 글자 속성은 그 안 글에 이어지고,
//! 폭을 적지 않은 표는 본문(바깥 셀이면 셀 안쪽) 폭 안에 들어간다.
//! HWP·HWPX 로 저장해 다시 열어도 같아야 한다.
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const STYLED: &str = concat!(
    r#"<p style="font-weight:bold;font-size:20pt;color:#ff0000">굵은 빨강</p>"#,
    r#"<p style="font-style:italic">기울임<span style="font-style:normal">보통</span></p>"#,
    r#"<p>기본</p>"#,
    r#"<ul><li style="font-weight:bold">항목</li></ul>"#,
    r#"<table><tr><td>가</td><td>나</td></tr></table>"#,
);

fn pasted(page_def: Option<&str>, html: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    if let Some(json) = page_def {
        doc.set_page_def(0, json).unwrap();
    }
    doc.paste_html(0, 0, 0, html).unwrap();
    doc
}

fn reopened(doc: &HwpDocument) -> [HwpDocument; 2] {
    [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()]
        .map(|bytes| HwpDocument::new(&bytes).unwrap())
}

/// 글자 하나의 모양: 굵게(B)·기울임(I)·크기·#RRGGBB
fn shape_at(doc: &HwpDocument, para: usize, index: usize) -> String {
    let document = doc.document();
    let paragraph = &document.sections[0].paragraphs[para];
    let id = paragraph.char_shape_id_at(index).unwrap_or(0);
    let shape = &document.doc_info.char_shapes[id as usize];
    let bgr = shape.text_color;
    format!(
        "{}{}{}pt #{:06x}",
        if shape.bold { "B" } else { "" },
        if shape.italic { "I" } else { "" },
        shape.base_size / 100,
        ((bgr & 0xff) << 16) | (bgr & 0xff00) | ((bgr >> 16) & 0xff),
    )
}

/// 문단 글을 같은 글자 모양끼리 묶어 `(모양 글)` 로 잇는다.
fn runs(doc: &HwpDocument, para: usize) -> String {
    let text = &doc.document().sections[0].paragraphs[para].text;
    let mut out = String::new();
    let mut last: Option<String> = None;
    for (index, ch) in text.chars().enumerate() {
        let key = shape_at(doc, para, index);
        if last.as_ref() != Some(&key) {
            if last.is_some() {
                out.push(')');
            }
            out.push_str(&format!("({key} "));
            last = Some(key);
        }
        out.push(ch);
    }
    if last.is_some() {
        out.push(')');
    }
    out
}

fn assert_block_styles(doc: &HwpDocument) {
    let paragraphs: Vec<String> = (0..3).map(|para| runs(doc, para)).collect();
    assert_eq!(
        paragraphs,
        [
            "(B20pt #ff0000 굵은 빨강)",
            "(I10pt #000000 기울임)(10pt #000000 보통)",
            "(10pt #000000 기본)",
        ]
    );
    // <li> 문단은 글머리 기호 "• " 뒤에 글이 온다.
    assert_eq!(doc.document().sections[0].paragraphs[3].text, "• 항목");
    assert_eq!(
        [shape_at(doc, 3, 2), shape_at(doc, 3, 3)],
        ["B10pt #000000", "B10pt #000000"]
    );
}

#[test]
fn p_and_li_style_character_css_reaches_their_text() {
    let doc = pasted(None, STYLED);
    assert_block_styles(&doc);
    for doc in reopened(&doc) {
        assert_block_styles(&doc);
    }
}

/// 본문에서 처음 만나는 표: (문단, 컨트롤 순번, 표)
fn first_table(doc: &HwpDocument) -> (u32, u32, &Table) {
    doc.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(para, paragraph)| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .find_map(|(control, item)| match item {
                    Control::Table(table) => Some((para as u32, control as u32, table.as_ref())),
                    _ => None,
                })
        })
        .expect("붙인 표")
}

fn assert_table_inside_body(doc: &HwpDocument) {
    let page: Value = serde_json::from_str(&doc.get_page_info(0).unwrap()).unwrap();
    let (body_left, body_right) = (
        page["bodyLeft"].as_f64().unwrap(),
        page["bodyRight"].as_f64().unwrap(),
    );
    let (para, control, _) = first_table(doc);
    let bbox: Value = serde_json::from_str(&doc.get_table_bbox(0, para, control).unwrap()).unwrap();
    let left = bbox["x"].as_f64().unwrap();
    let right = left + bbox["width"].as_f64().unwrap();
    assert!(
        body_left <= left && right <= body_right,
        "표 {left}..{right}px 가 본문 {body_left}..{body_right}px 밖으로 나간다"
    );
}

#[test]
fn pasted_table_without_widths_fits_the_body() {
    // 기본 A4(좌우 30mm)와, 그보다 본문이 좁은 쪽(좌우 50mm)
    for page_def in [None, Some(r#"{"marginLeft":14173,"marginRight":14173}"#)] {
        let doc = pasted(page_def, STYLED);
        assert_table_inside_body(&doc);
        for doc in reopened(&doc) {
            assert_table_inside_body(&doc);
        }
    }
}

#[test]
fn nested_table_without_widths_fits_its_cell() {
    let doc = pasted(
        None,
        "<table><tr><td>바깥<table><tr><td>안</td></tr></table></td><td>옆</td></tr></table>",
    );
    let (_, _, outer) = first_table(&doc);
    let cell = &outer.cells[0];
    let inner = cell
        .paragraphs
        .iter()
        .find_map(|paragraph| match paragraph.controls.first() {
            Some(Control::Table(table)) => Some(table.as_ref()),
            _ => None,
        })
        .expect("셀 안 표");
    let inner_width = inner.get_column_widths().iter().sum::<u32>() as i64
        + inner.outer_margin_left as i64
        + inner.outer_margin_right as i64;
    let cell_inner = cell.width as i64 - cell.padding.left as i64 - cell.padding.right as i64;
    assert!(
        inner_width <= cell_inner,
        "셀 안 표 {inner_width}HU 가 셀 안쪽 {cell_inner}HU 보다 넓다"
    );
}
