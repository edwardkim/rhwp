//! 나란히 놓인 사진 두 장 칸 재현 fixture 생성기.
//!
//! 형상(활동 보고서의 "활동 모습" 사진 칸과 같은 꼴):
//! - 글자처럼 취급한 쪽 나눔(RowBreak) 표, 열 둘: 이름 칸 + 사진 칸.
//! - 사진 칸은 저장 줄(LINE_SEG) 없는 **글자 없는 문단 하나**에 문단 기준 자리차지
//!   (TopAndBottom) 그림 두 장이 가로로 나란히(겹치지 않게) 앵커된다.
//! - 칸 선언 높이는 그림 높이 + 안 여백보다 조금 크다.
//! - 마지막 두 행은 대조군이다: 그림 한 장 행, 같은 문단의 그림 두 장이 가로로 겹치는
//!   (세로로 쌓이는) 행.
//!
//! 입력은 공개 합성본 `samples/title_flow_photo/title-flow-photo-row.hwp`(저장 줄 없는 원본
//! HWP5 경로) 하나다. 그 문서 첫 표의 그림 문단·결재 문단·칸 서식을 재료로 쓴다.
//!
//! Usage:
//!   cargo run --release --example build_side_by_side_photo_cell_fixture

use rhwp::model::control::Control;
use rhwp::model::table::{Cell, Table};
use std::fs;

const INPUT: &str = "samples/title_flow_photo/title-flow-photo-row.hwp";
const OUT: &str = "samples/side_by_side_photo_cell/side-by-side-photo-rows.hwp";

/// 열 폭(HU): 이름 칸, 사진 칸.
const COL_W: [u32; 2] = [4000, 18000];
/// 그림 크기(HU).
const PIC_W: u32 = 8000;
const PIC_H: u32 = 6000;
/// 나란한 둘째 그림의 가로 오프셋(HU): 첫 그림(0..8000)과 겹치지 않는다.
const SIDE_X: u32 = 9000;
/// 대조군 둘째 그림의 가로 오프셋(HU): 첫 그림과 가로로 겹친다.
const OVERLAP_X: u32 = 4000;
/// 칸 선언 높이(HU): 그림 6000 + 안 여백 282 + 334.
const PHOTO_ROW_HU: u32 = 6616;
/// 나란한 사진 행 수. 마지막에 대조군 행 둘(그림 한 장, 겹친 그림 두 장)이 더 붙는다.
const SIDE_ROWS: u16 = 6;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(INPUT)?;
    let mut doc = rhwp::parser::parse_hwp(&bytes)?;
    let section = &mut doc.sections[0];

    let (photo_para, label_para, template_cell, template_table) = {
        let Control::Table(table) = &section.paragraphs[0].controls[2] else {
            return Err("입력 문서 0.0 의 셋째 개체가 표가 아니다".into());
        };
        (
            table.cells[0].paragraphs[1].clone(),
            table.cells[1].paragraphs[0].clone(),
            table.cells[0].clone(),
            table.as_ref().clone(),
        )
    };
    assert!(photo_para.line_segs.is_empty() && label_para.line_segs.is_empty());

    let two_pictures = |second_x: u32| -> Result<_, Box<dyn std::error::Error>> {
        let mut para = photo_para.clone();
        let Control::Picture(pic) = &mut para.controls[0] else {
            return Err("입력 칸 둘째 문단이 그림 문단이 아니다".into());
        };
        pic.common.width = PIC_W;
        pic.common.height = PIC_H;
        pic.common.horizontal_offset = 0;
        pic.shape_attr.current_width = PIC_W;
        pic.shape_attr.current_height = PIC_H;
        let mut second = para.controls[0].clone();
        let Control::Picture(pic2) = &mut second else {
            unreachable!()
        };
        pic2.common.horizontal_offset = second_x;
        para.controls.push(second);
        // 개체 문자 두 개(8 유닛씩) + 문단 끝.
        para.char_count = photo_para.char_count + 8;
        Ok(para)
    };
    let side = two_pictures(SIDE_X)?;
    let overlap = two_pictures(OVERLAP_X)?;
    let mut single = side.clone();
    single.controls.truncate(1);
    single.char_count = photo_para.char_count;

    let cell = |row: u16, col: u16, paragraphs| -> Cell {
        let mut c = template_cell.clone();
        c.row = row;
        c.col = col;
        c.row_span = 1;
        c.col_span = 1;
        c.width = COL_W[col as usize];
        c.height = PHOTO_ROW_HU;
        c.paragraphs = paragraphs;
        c
    };
    let row_count = SIDE_ROWS + 2;
    let mut cells = Vec::new();
    for row in 0..row_count {
        let photo = if row < SIDE_ROWS {
            side.clone()
        } else if row == SIDE_ROWS {
            single.clone()
        } else {
            overlap.clone()
        };
        cells.push(cell(row, 0, vec![label_para.clone()]));
        cells.push(cell(row, 1, vec![photo]));
    }

    let mut table: Table = template_table;
    table.common.treat_as_char = true;
    table.row_count = row_count;
    table.col_count = 2;
    table.row_sizes = vec![2; row_count as usize];
    table.cells = cells;
    table.rebuild_grid();
    table.update_ctrl_dimensions();
    // 표 직렬화는 원본 CTRL_HEADER 바이트를 쓴다: 공통 속성 attr bit 0(글자처럼 취급)을
    // 거기에도 켠다.
    table.common.attr |= 1;
    if let Some(attr) = table.raw_ctrl_data.first_mut() {
        *attr |= 1;
    }

    section.paragraphs[0].controls[2] = Control::Table(Box::new(table));
    section.paragraphs.truncate(1);
    section.raw_stream = None;
    section.raw_provenance = None;

    let out = rhwp::serializer::serialize_hwp(&doc)?;
    if let Some(dir) = std::path::Path::new(OUT).parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(OUT, &out)?;

    // 되읽어 구조가 그대로 실렸는지 확인한다.
    let reread = rhwp::parser::parse_hwp(&out)?;
    let Control::Table(t) = &reread.sections[0].paragraphs[0].controls[2] else {
        return Err("되읽은 문서에 표가 없다".into());
    };
    assert!(t.common.treat_as_char, "표는 글자처럼 취급");
    assert_eq!((t.row_count, t.col_count), (row_count, 2));
    for c in t.cells.iter().filter(|c| c.col == 1) {
        let [p] = c.paragraphs.as_slice() else {
            return Err("사진 칸 문단이 하나가 아니다".into());
        };
        let pictures = if c.row == SIDE_ROWS { 1 } else { 2 };
        assert_eq!((p.controls.len(), p.line_segs.len()), (pictures, 0));
    }
    println!("wrote {OUT} ({} bytes)", out.len());
    Ok(())
}
