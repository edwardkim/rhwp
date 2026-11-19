//! 칸 안 표의 캡션을 셀 경로로 읽고 고친다. 경로 마디의 셀 65534는 그 표의 캡션이다
//! (본문 표 셀 번호 API·가변 경로 해석기와 같은 꼴). 전에는 불변 경로 조회가 표의 칸만 찾아
//! `getCellParagraphCountByPath`·`getCellParagraphLengthByPath`·`insertTextInCellByPath`가
//! "셀 65534 범위 초과"로 실패했고, 칸 안 표 캡션 글은 같은 번호의 본문 표 캡션 주소를 가져
//! 경로 캐럿이 그 캡션을 찾지 못했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::shape::CaptionDirection;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;

// [#5875] 바깥 표 한 칸 안에 `<표 1> 공급전압 차단` 같은 글자 캡션을 단 중첩 표가 있다.
const SAMPLE: &str = "samples/issue5875/nested_table_text_caption.hwp";
const CAPTION: usize = 65534;
type Path = Vec<(usize, usize, usize)>;

fn load() -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

/// 위·아래 캡션을 단 첫 중첩 표: (구역, 본문 문단, 그 표 첫 칸 경로, 캡션 방향).
fn nested_captioned_table(doc: &HwpDocument) -> (usize, usize, Path, CaptionDirection) {
    for (sec, section) in doc.document().sections.iter().enumerate() {
        for (para, paragraph) in section.paragraphs.iter().enumerate() {
            for (ci, control) in paragraph.controls.iter().enumerate() {
                let Control::Table(outer) = control else {
                    continue;
                };
                for (cell, outer_cell) in outer.cells.iter().enumerate() {
                    for (cp, cell_para) in outer_cell.paragraphs.iter().enumerate() {
                        for (ni, nested) in cell_para.controls.iter().enumerate() {
                            let Control::Table(nested) = nested else {
                                continue;
                            };
                            let Some(caption) = nested.caption.as_ref() else {
                                continue;
                            };
                            if matches!(
                                caption.direction,
                                CaptionDirection::Top | CaptionDirection::Bottom
                            ) {
                                return (
                                    sec,
                                    para,
                                    vec![(ci, cell, cp), (ni, 0, 0)],
                                    caption.direction,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("{SAMPLE}: 캡션을 단 중첩 표가 있어야 한다");
}

fn with_cell(path: &Path, cell: usize) -> Path {
    let mut path = path.clone();
    let last = path.last_mut().unwrap();
    *last = (last.0, cell, 0);
    path
}

fn path_json(path: &Path) -> String {
    serde_json::to_string(
        &path
            .iter()
            .map(|&(c, cell, p)| {
                serde_json::json!({"controlIndex": c, "cellIndex": cell, "cellParaIndex": p})
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn nested_table<'a>(doc: &'a HwpDocument, sec: usize, para: usize, path: &Path) -> &'a Table {
    let (ci, cell, cp) = path[0];
    let Control::Table(outer) = &doc.document().sections[sec].paragraphs[para].controls[ci] else {
        unreachable!()
    };
    let Control::Table(nested) = &outer.cells[cell].paragraphs[cp].controls[path[1].0] else {
        unreachable!()
    };
    nested
}

fn caret(doc: &HwpDocument, sec: usize, para: usize, path: &Path, offset: u32) -> (f64, f64) {
    let rect: serde_json::Value = serde_json::from_str(
        &doc.get_cursor_rect_by_path(sec as u32, para as u32, &path_json(path), offset)
            .unwrap(),
    )
    .unwrap();
    let y = rect["y"].as_f64().unwrap();
    (y, y + rect["height"].as_f64().unwrap())
}

#[test]
fn nested_table_caption_is_read_and_edited_by_path() {
    let mut doc = load();
    let (sec, para, cell, direction) = nested_captioned_table(&doc);
    let caption = with_cell(&cell, CAPTION);
    let expected = nested_table(&doc, sec, para, &cell)
        .caption
        .as_ref()
        .unwrap()
        .paragraphs
        .len();

    assert_eq!(
        doc.get_cell_paragraph_count_by_path(sec as u32, para as u32, &path_json(&caption))
            .unwrap() as usize,
        expected
    );
    let len = doc
        .get_cell_paragraph_length_by_path(sec as u32, para as u32, &path_json(&caption))
        .unwrap() as usize;
    doc.insert_text_in_cell_by_path(sec, para, &caption, len, "AB")
        .unwrap();
    let text = doc
        .get_text_in_cell_by_path(sec, para, &caption, 0, usize::MAX / 2)
        .unwrap();
    assert!(text.ends_with("AB"), "중첩 표 캡션에 든다: {text:?}");

    // 캡션 캐럿은 그 표의 캡션 줄에서 찾는다: 위 캡션은 첫 칸 위, 아래 캡션은 마지막 칸 아래.
    let (top, bottom) = caret(&doc, sec, para, &caption, len as u32 + 2);
    let last_cell = nested_table(&doc, sec, para, &cell).cells.len() - 1;
    match direction {
        CaptionDirection::Top => {
            let (first_top, _) = caret(&doc, sec, para, &cell, 0);
            assert!(
                bottom <= first_top + 0.5,
                "캡션 캐럿 {top}..{bottom} 은 첫 칸 {first_top} 위"
            );
        }
        _ => {
            let (_, last_bottom) = caret(&doc, sec, para, &with_cell(&cell, last_cell), 0);
            assert!(
                top >= last_bottom - 0.5,
                "캡션 캐럿 {top}..{bottom} 은 마지막 칸 {last_bottom} 아래"
            );
        }
    }

    let reopened = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    let text = reopened
        .get_text_in_cell_by_path(sec, para, &caption, 0, usize::MAX / 2)
        .unwrap();
    assert!(text.ends_with("AB"), "저장 뒤에도 남는다: {text:?}");
}

#[test]
fn caption_step_without_caption_is_an_error() {
    let doc = load();
    // 바깥 표에는 캡션이 없다.
    let (sec, para, cell, _) = nested_captioned_table(&doc);
    let ci = cell[0].0;
    let Control::Table(outer) = &doc.document().sections[sec].paragraphs[para].controls[ci] else {
        unreachable!()
    };
    assert!(outer.caption.is_none(), "전제: 바깥 표에는 캡션이 없다");
    let error = doc
        .get_text_in_cell_by_path(sec, para, &[(ci, CAPTION, 0)], 0, 1)
        .unwrap_err();
    assert!(error.to_string().contains("캡션이 없습니다"), "{error}");
}
