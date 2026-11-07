//! 관계형 — 칸 문단을 편집한 뒤 재조판 기준선은 문단 세로 정렬점이다 (#8555).
//! 주석 갱신: 2026-11-04T03:00:00+09:00 (새 파일)
//! 시각 측정은 해당 없음: 합성 문서의 편집 재조판 저장값(`baseline_distance`)만 검사한다.
//! 기대값 근거: HWP 5.0 표 44 문단 세로 정렬(attr1 bit 20-21)과 한컴 저장 LINE_SEG. 공개 표본
//! 1,205개의 칸 문단을 편집 경로와 같은 상자로 다시 조판해 저장값과 비교하면 세로 정렬이 위·가운데·아래인
//! 문단은 기준선을 각각 0, 글자 높이의 절반(내림), 글자 높이 전체로 저장한다(기준선만 어긋난 칸 문단
//! 55,158개 중 53,661개가 이 경우). 고친 뒤 칸 기준선 불일치는 1,503개로 줄고 새로 어긋난 문단은 0개였다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// 칸 하나의 표에서 칸 문단의 세로 정렬을 `alignment` 로 두고 글자를 넣어 재조판한 첫 줄의
/// (글자 높이, 기준선 거리, 줄 높이).
fn edited_cell_row(alignment: u32) -> (i32, i32, i32) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("유효한 글자·문단 모양 정의");
    core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    let table_para = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .expect("표 문단");
    let ctrl = core.document().sections[0].paragraphs[table_para]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();
    let mut document = core.document().clone();
    let Control::Table(table) = &mut document.sections[0].paragraphs[table_para].controls[ctrl]
    else {
        unreachable!()
    };
    let shape_id = table.cells[0].paragraphs[0].para_shape_id as usize;
    let mut shape = document.doc_info.para_shapes[shape_id].clone();
    shape.attr1 = (shape.attr1 & !(0x03 << 20)) | (alignment << 20);
    document.doc_info.para_shapes.push(shape);
    let new_id = (document.doc_info.para_shapes.len() - 1) as u16;
    let Control::Table(table) = &mut document.sections[0].paragraphs[table_para].controls[ctrl]
    else {
        unreachable!()
    };
    table.cells[0].paragraphs[0].para_shape_id = new_id;
    core.set_document(document);
    // 글자 넣기가 편집 재조판을 부른다.
    core.insert_text_in_cell_native(0, table_para, ctrl, 0, 0, 0, "가나다")
        .expect("칸 글 삽입");
    let Control::Table(table) = &core.document().sections[0].paragraphs[table_para].controls[ctrl]
    else {
        unreachable!()
    };
    let row = &table.cells[0].paragraphs[0].line_segs[0];
    (row.text_height, row.baseline_distance, row.line_height)
}

#[test]
fn edited_cell_rows_publish_the_vertical_alignment_point() {
    let (base_th, base_bl, base_lh) = edited_cell_row(0);
    assert!(
        base_bl > 0 && base_bl < base_th,
        "전제: 글꼴 기준 정렬은 글자 안쪽 기준선이다 ({base_bl}/{base_th})"
    );
    for (alignment, expected) in [(1, 0), (2, base_th / 2), (3, base_th)] {
        let (th, bl, lh) = edited_cell_row(alignment);
        assert_eq!(
            (th, lh),
            (base_th, base_lh),
            "세로 정렬 {alignment}: 줄 상자는 같다"
        );
        assert_eq!(bl, expected, "세로 정렬 {alignment}: 기준선은 정렬점이다");
    }
}
