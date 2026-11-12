//! 사진 칸 + 캡션 칸 행이 쪽 끝에 놓인 쪽 나눔 표 재현 fixture 생성기.
//!
//! 형상(놀이 계획표의 사진 줄과 같은 꼴):
//! - 왼쪽 열 칸 하나가 사진 행부터 표 끝까지 세로로 걸쳐 쪽 경계를 지난다.
//! - 사진 칸은 저장 줄 없는 **글자 없는 문단 하나**뿐이고 그 문단에 문단 기준 자리차지
//!   (TopAndBottom) 그림이 있다. 사진 설명은 다음 행의 별도 칸(캡션 칸)에 있다.
//! - 오른쪽 열 칸은 사진 행 + 캡션 행 두 행에 걸친 빈 칸이다.
//! - 사진 칸 선언 높이는 그림보다 낮아 사진 행 높이는 내용(안 여백 + 그림)이 정한다.
//! - 사진·캡션 행 묶음 6개 뒤에 한 줄짜리 본문 행이 이어진다. 첫 쪽 끝에 다섯째 묶음의
//!   캡션 행이 놓이고, 여섯째 사진 행은 남은 공간에 근소하게 못 들어가 다음 쪽으로 간다.
//!
//! 입력은 공개 합성본 `samples/title_flow_photo/title-flow-photo-row.hwp` 하나다(그 문서의
//! 그림 문단·글자 문단·칸 서식을 재료로 쓴다). 첫 행(여백 행)의 선언 높이로 쪽 끝 위치를
//! 맞춘다.
//!
//! Usage:
//!   cargo run --release --example build_photo_caption_rows_fixture

use rhwp::model::control::Control;
use rhwp::model::table::{Cell, Table};
use std::fs;

const INPUT: &str = "samples/title_flow_photo/title-flow-photo-row.hwp";
const OUT: &str = "samples/photo_caption_rows/photo-caption-rows-page-end.hwp";

/// 열 폭(HU): 왼쪽 걸침 열, 사진·캡션 열, 오른쪽 빈 칸 열.
const COL_W: [u32; 3] = [3000, 9000, 6000];
/// 사진·캡션 행 묶음 수.
const PHOTO_PAIRS: u16 = 6;
/// 사진 칸 선언 높이(HU): 안 여백 282 + 그림 6000 보다 낮다 — 행 높이는 내용(그림)이 정한다.
const PHOTO_ROW_HU: u32 = 6000;
/// 캡션 행 선언 높이(HU).
const CAPTION_ROW_HU: u32 = 1989;
/// 본문 행 선언 높이(HU)와 개수.
const BODY_ROW_HU: u32 = 1183;
const BODY_ROWS: u16 = 6;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spacer_hu = SPACER_ROW_HU;
    let bytes = fs::read(INPUT)?;
    let mut doc = rhwp::parser::parse_hwp(&bytes)?;

    let section = &mut doc.sections[0];
    // 재료: 표 1 의 제목+그림 칸, 결재 칸.
    let (photo_para, title_para, body_para, template_cell, template_table) = {
        let Control::Table(table) = &section.paragraphs[0].controls[2] else {
            return Err("입력 문서 0.0 의 셋째 개체가 표가 아니다".into());
        };
        let photo_cell = &table.cells[0];
        let body_cell = &table.cells[1];
        (
            photo_cell.paragraphs[1].clone(),
            photo_cell.paragraphs[0].clone(),
            body_cell.paragraphs[0].clone(),
            photo_cell.clone(),
            table.as_ref().clone(),
        )
    };

    let cell = |row: u16, col: u16, row_span: u16, height: u32, paragraphs| -> Cell {
        let mut c = template_cell.clone();
        c.row = row;
        c.col = col;
        c.row_span = row_span;
        c.col_span = 1;
        c.width = COL_W[col as usize];
        c.height = height;
        c.paragraphs = paragraphs;
        c
    };

    let mut cells = Vec::new();
    // 0 행: 여백 행(쪽 끝 위치 조절).
    for col in 0..3 {
        cells.push(cell(0, col, 1, spacer_hu, vec![body_para.clone()]));
    }
    let first_pair_row = 1u16;
    let body_first_row = first_pair_row + PHOTO_PAIRS * 2;
    let row_count = body_first_row + BODY_ROWS;
    // 왼쪽 열: 사진 행부터 표 끝까지 한 칸.
    let left_span = row_count - first_pair_row;
    let left_h =
        PHOTO_PAIRS as u32 * (PHOTO_ROW_HU + CAPTION_ROW_HU) + BODY_ROWS as u32 * BODY_ROW_HU;
    cells.push(cell(
        first_pair_row,
        0,
        left_span,
        left_h,
        vec![title_para.clone()],
    ));
    for pair in 0..PHOTO_PAIRS {
        let photo_row = first_pair_row + pair * 2;
        cells.push(cell(
            photo_row,
            1,
            1,
            PHOTO_ROW_HU,
            vec![photo_para.clone()],
        ));
        cells.push(cell(
            photo_row,
            2,
            2,
            PHOTO_ROW_HU + CAPTION_ROW_HU,
            vec![body_para.clone()],
        ));
        cells.push(cell(
            photo_row + 1,
            1,
            1,
            CAPTION_ROW_HU,
            vec![title_para.clone()],
        ));
    }
    for i in 0..BODY_ROWS {
        let row = body_first_row + i;
        cells.push(cell(row, 1, 1, BODY_ROW_HU, vec![body_para.clone()]));
        cells.push(cell(row, 2, 1, BODY_ROW_HU, vec![body_para.clone()]));
    }
    // 저장 순서는 행·열 순서다.
    cells.sort_by_key(|c| (c.row, c.col));

    let mut table: Table = template_table;
    table.row_count = row_count;
    table.col_count = 3;
    table.row_sizes = (0..row_count)
        .map(|r| cells.iter().filter(|c| c.row == r).count() as i16)
        .collect();
    table.cells = cells;
    table.rebuild_grid();
    table.update_ctrl_dimensions();

    // 문서: 0.0 문단의 표를 새 표로 바꾸고, 뒤 문단(표 2·3)은 뺀다.
    section.paragraphs[0].controls[2] = Control::Table(Box::new(table));
    section.paragraphs.truncate(1);
    // 원본 구역 바이트를 그대로 쓰지 않고 바꾼 모델로 다시 직렬화한다.
    section.raw_stream = None;
    section.raw_provenance = None;

    let out = rhwp::serializer::serialize_hwp(&doc)?;
    if let Some(dir) = std::path::Path::new(OUT).parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(OUT, &out)?;
    // 되읽어 표 구조가 그대로 실렸는지 확인한다.
    let reread = rhwp::parser::parse_hwp(&out)?;
    let Control::Table(t) = &reread.sections[0].paragraphs[0].controls[2] else {
        return Err("되읽은 문서에 표가 없다".into());
    };
    assert_eq!((t.row_count, t.col_count), (row_count, 3));
    println!("wrote {OUT} ({} bytes, spacer {spacer_hu}HU)", out.len());
    Ok(())
}

/// 여백 행 선언 높이(HU). 다섯째 사진·캡션 행 묶음이 첫 쪽 끝에 들고, 여섯째 사진 행은
/// 남은 공간에 근소하게 못 들어가 다음 쪽으로 가는 자리다.
///
/// 웹한글기안기 인쇄본과 맞춰 고른 값이다. 다섯째 캡션 행 아래 남는 띠(약 26px)에서
/// 웹한글기안기는 캡션 행을 선언 높이로 두고 여섯째 사진 행을 쪽 끝에서 갈라 오른쪽 걸침 칸의
/// 글만 1쪽에 둔다. rhwp 는 수정 전후 모두 1쪽 마지막 캡션 행 바닥을 쪽 끝에 맞춘다(쪽 경계
/// 병합 칸 기전, 이 수정과 무관). 여백 행이 이보다 낮으면 캡션 행이 선언보다 커지고(실루엣·
/// 테스트 모두 어긋남), 22200HU 이상이면 rhwp 가 캡션 행까지 다음 쪽으로 넘긴다.
const SPACER_ROW_HU: u32 = 21800;
