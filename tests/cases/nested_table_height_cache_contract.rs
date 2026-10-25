#![cfg(not(target_arch = "wasm32"))]

//! 중첩 표 본체 높이 캐시가 편집 뒤 낡은 높이를 내지 않는다. 먼저 그린 문서의 중첩 표
//! 셀을 편집한 출력은, 같은 편집만 적용한 새 문서를 처음 그린 출력과 같아야 한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

type CellPath = Vec<(usize, usize, usize)>;

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn all_pages_svg(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| core.render_page_svg_native(page).expect("render page"))
        .collect()
}

/// 본문 표 셀 안에 있는 첫 중첩 표의 첫 셀 첫 문단 경로. (구역, 본문 문단, 경로)
fn first_nested_cell(core: &DocumentCore) -> (usize, usize, CellPath) {
    for (section, value) in core.document().sections.iter().enumerate() {
        for (para, paragraph) in value.paragraphs.iter().enumerate() {
            for (control, item) in paragraph.controls.iter().enumerate() {
                let Control::Table(table) = item else {
                    continue;
                };
                for (cell_index, cell) in table.cells.iter().enumerate() {
                    for (cell_para, inner) in cell.paragraphs.iter().enumerate() {
                        for (nested_control, nested) in inner.controls.iter().enumerate() {
                            let Control::Table(nested) = nested else {
                                continue;
                            };
                            if nested
                                .cells
                                .first()
                                .is_some_and(|c| !c.paragraphs.is_empty())
                            {
                                let path =
                                    vec![(control, cell_index, cell_para), (nested_control, 0, 0)];
                                return (section, para, path);
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("fixture has no nested table");
}

#[test]
fn nested_cell_edit_after_render_matches_fresh_render() {
    let bytes = read_fixture("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    // 중첩 표가 여러 줄 자라도록 충분히 긴 문단을 넣는다.
    let text = "중첩 표 높이 캐시 검사 문장입니다. ".repeat(40);

    let mut rendered = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    let (section, para, path) = first_nested_cell(&rendered);
    let before = all_pages_svg(&rendered);
    rendered
        .insert_text_in_cell_by_path(section, para, &path, 0, &text)
        .expect("edit rendered nested cell");
    let edited = all_pages_svg(&rendered);

    let mut fresh = DocumentCore::from_bytes(&bytes).expect("parse fixture again");
    fresh
        .insert_text_in_cell_by_path(section, para, &path, 0, &text)
        .expect("edit fresh nested cell");
    let fresh = all_pages_svg(&fresh);

    assert_ne!(edited, before, "중첩 표 셀 편집이 출력에 반영되지 않았다");
    assert_eq!(
        edited, fresh,
        "먼저 그린 문서의 중첩 표 편집 출력이 새 문서와 다르다"
    );
}
