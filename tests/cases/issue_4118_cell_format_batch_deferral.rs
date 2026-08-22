//! #4118 — 셀 서식 뮤테이터의 배치 지연 재페이지네이션 동등성 계약.
//!
//! `begin_batch`~`end_batch` 사이에서 셀 서식 뮤테이터(`apply_para_format_in_cell`,
//! `apply_char_format_in_cell`, `set_cell_properties`)는 재구성·재페이지네이션을
//! `end_batch` 의 paginate() 1회로 미룬다. 이 테스트는 같은 시작 문서에 대해
//! (a) 호출마다 전체 rebuild 한 결과와 (b) 배치로 묶은 결과가 쪽 수와 저장
//! 바이트까지 동일함을 확인한다 — 지연이 관측 가능한 결과를 바꾸지 않는다는
//! 계약이다. 배치는 호출마다 O(문서) 재조판을 없애 #4118 의 O(n²) 를 O(n) 으로
//! 낮춘다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx")
        .to_string_lossy()
        .into_owned()
}

/// 본문 최상위 표 하나의 좌표와 셀 수.
struct BodyTable {
    section: usize,
    paragraph: usize,
    control: usize,
    cell_count: usize,
}

fn first_body_table(path: &str) -> BodyTable {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for g in extract_tables(doc.document()) {
        if !g.container_path.is_empty() {
            continue;
        }
        let Some(Control::Table(table)) = doc.document().sections[g.section].paragraphs
            [g.paragraph]
            .controls
            .get(g.control)
        else {
            continue;
        };
        if table.cells.len() >= 4 {
            return BodyTable {
                section: g.section,
                paragraph: g.paragraph,
                control: g.control,
                cell_count: table.cells.len(),
            };
        }
    }
    panic!("본문 최상위 표가 없다");
}

fn run(batched: bool, sample_path: &str) -> (u32, String, String, Vec<u8>) {
    let bytes = std::fs::read(sample_path).expect("sample");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let grid = first_body_table(sample_path);

    if batched {
        doc.begin_batch().expect("배치 시작이 성공해야 함");
    }
    for cell_idx in 0..grid.cell_count {
        doc.apply_para_format_in_cell(
            grid.section,
            grid.paragraph,
            grid.control,
            cell_idx,
            0,
            r#"{"alignment":"center","lineSpacing":130}"#,
        )
        .expect("셀 문단 서식 적용이 성공해야 함");
        doc.apply_char_format_in_cell(
            grid.section,
            grid.paragraph,
            grid.control,
            cell_idx,
            0,
            0,
            2,
            r#"{"bold":true}"#,
        )
        .expect("셀 글자 서식 적용이 성공해야 함");
        doc.set_cell_properties(
            grid.section as u32,
            grid.paragraph as u32,
            grid.control as u32,
            cell_idx as u32,
            r#"{"verticalAlign":1}"#,
        )
        .expect("셀 속성 적용이 성공해야 함");
    }
    if batched {
        doc.end_batch().expect("배치 종료가 성공해야 함");
    }

    let pages = doc.page_count();
    let first_page = doc.get_page_info(1).expect("1쪽 정보 조회가 성공해야 함");
    // 표가 놓이는 1쪽만 렌더 비교한다 — 네이티브 JsValue 변환은 렌더 Err 경로가
    // 미구현이라(panics as non-unwinding) 실패 가능성이 있는 전 쪽 순회는 피한다.
    let first_page_svg = doc.render_page_svg(1).expect("1쪽 SVG 렌더가 성공해야 함");
    let exported = doc.export_hwp().expect("내보내기가 성공해야 함");
    (pages, first_page, first_page_svg, exported)
}

#[test]
fn batched_cell_formats_match_eager_rebuild() {
    let sample_path = sample();
    let _ = PathBuf::from(&sample_path);

    let eager = run(false, &sample_path);
    let batched = run(true, &sample_path);

    assert_eq!(
        eager.0, batched.0,
        "배치 여부와 무관하게 쪽 수가 같아야 한다"
    );
    assert_eq!(
        eager.1, batched.1,
        "배치 여부와 무관하게 1쪽 지오메트리가 같아야 한다"
    );
    assert_eq!(
        eager.2, batched.2,
        "배치 여부와 무관하게 1쪽 SVG 렌더가 같아야 한다"
    );
    assert_eq!(
        eager.3, batched.3,
        "배치 여부와 무관하게 저장 바이트가 같아야 한다"
    );
}
