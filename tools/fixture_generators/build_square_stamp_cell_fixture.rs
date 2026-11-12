//! 결재 칸(도장 그림만 든 칸) 대조군 fixture 생성기.
//!
//! 형상(상담 일지 오른쪽 위 결재 표와 같은 꼴): 3행×2열 표
//!   행 0 = 여백 행(쪽 끝 위치 조절)
//!   행 1 = 머리 칸 두 개(글줄)
//!   행 2 = 이름 칸(글줄) + 도장 칸(저장 줄 없는 글자 없는 문단 하나 + 칸 글줄 띠 안의 어울림
//!          (Square) 그림 한 장)
//! 뒤에 본문 표가 이어지고, 그 첫 칸은 칸 글줄 띠 안의 어울림 사진 문단 + 저장 줄 없는 빈 문단
//! 둘 + 설명 글줄이다. 두 칸 모두 그림 문단의 빈 줄이 뒤 글줄 자리를 정하지 않는 대조군이다
//! (도장 칸은 내릴 글줄이 없고, 사진 칸은 그림이 글줄 띠 안이라 글줄을 그림 바닥으로 내린다).
//! 빈 문단 둘은 저장 줄 없이 둔다. 저장 줄(vpos 0)을 주면 웹한글기안기는 빈 줄 둘을 사진 아래로
//! 내려 설명 글줄이 그만큼 내려가는데, rhwp 는 수정 전후 모두 저장 줄 빈 문단을 사진 옆에 두어
//! 실루엣이 어긋난다(이 대조군과 무관한 기전).
//!
//! 입력은 공개 합성본 `samples/title_flow_photo/title-flow-photo-row.hwp`(저장 줄 없는 원본
//! HWP5 경로) 하나다. 그 문서 첫 표의 제목 문단·그림 문단·결재 문단을 재료로 쓴다.
//!
//! Usage:
//!   cargo run --release --example build_square_stamp_cell_fixture

use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use std::fs;

const INPUT: &str = "samples/title_flow_photo/title-flow-photo-row.hwp";
const OUT: &str = "samples/square_floor_empty_lead/approval-stamp-cell.hwp";

/// 도장 그림 크기(HU).
const STAMP: u32 = 3131;
/// 머리 행·도장 행 선언 높이(HU).
const HEAD_H: u32 = 1282;
const STAMP_ROW_H: u32 = 2414;
/// 본문 표 칸 선언 높이(HU) — 내용(사진 + 빈 줄 둘 + 설명)보다 높다.
const BODY_CELL_H: u32 = 14000;
/// 여백 행 선언 높이(HU) — 머리 행·도장 행이 첫 쪽에 들지 못해 둘째 쪽 머리로 함께 넘어가는 자리.
const SPACER_H: u32 = 64000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(INPUT)?;
    let mut doc = rhwp::parser::parse_hwp(&bytes)?;
    let section = &mut doc.sections[0];
    {
        let Control::Table(table) = &mut section.paragraphs[0].controls[2] else {
            return Err("입력 문서 0.0 의 셋째 개체가 표가 아니다".into());
        };
        let template = table.cells[0].clone();
        let title = template.paragraphs[0].clone();
        let mut stamp = template.paragraphs[1].clone();
        let approval = table.cells[1].paragraphs[0].clone();
        assert!(title.line_segs.is_empty() && stamp.line_segs.is_empty());
        let Control::Picture(pic) = &mut stamp.controls[0] else {
            return Err("입력 칸 둘째 문단이 그림 문단이 아니다".into());
        };
        pic.common.text_wrap = TextWrap::Square;
        pic.common.width = STAMP;
        pic.common.height = STAMP;
        pic.shape_attr.current_width = STAMP;
        pic.shape_attr.current_height = STAMP;

        let cell = |row: u16, col: u16, height: u32, paragraphs| {
            let mut c = template.clone();
            c.row = row;
            c.col = col;
            c.height = height;
            c.paragraphs = paragraphs;
            c
        };
        let spacer_h = SPACER_H;
        table.cells = vec![
            cell(0, 0, spacer_h, vec![title.clone()]),
            cell(0, 1, spacer_h, vec![approval.clone()]),
            cell(1, 0, HEAD_H, vec![title.clone()]),
            cell(1, 1, HEAD_H, vec![approval.clone()]),
            cell(2, 0, STAMP_ROW_H, vec![title]),
            cell(2, 1, STAMP_ROW_H, vec![stamp]),
        ];
        table.row_count = 3;
        table.row_sizes = vec![2, 2, 2];
        table.rebuild_grid();
        table.update_ctrl_dimensions();
    }
    // 뒤 문단 0.1 의 본문 표 첫 칸: 칸 글줄 띠 안의 어울림 사진 문단 + 저장 줄 없는 빈 문단
    // 두 개 + 설명 글줄. 칸 선언 높이가 내용보다 높아 세로 가운데 정렬이 내용 높이를 쓴다.
    {
        let Control::Table(body) = &mut section.paragraphs[1].controls[0] else {
            return Err("입력 문서 0.1 의 개체가 표가 아니다".into());
        };
        let cell = &mut body.cells[0];
        let label = cell.paragraphs[0].clone();
        let mut photo = cell.paragraphs[1].clone();
        let Control::Picture(pic) = &mut photo.controls[0] else {
            return Err("본문 표 첫 칸 둘째 문단이 그림 문단이 아니다".into());
        };
        pic.common.text_wrap = TextWrap::Square;
        let mut empty = label.clone();
        empty.text.clear();
        empty.char_count = 1;
        empty.char_offsets.clear();
        empty.char_shapes.truncate(1);
        empty.has_para_text = false;
        empty.line_segs.clear();
        cell.paragraphs = vec![photo, empty.clone(), empty, label];
        cell.height = BODY_CELL_H;
        body.cells[1].height = BODY_CELL_H;
        body.update_ctrl_dimensions();
    }
    // 뒤 문단 0.1(본문 표)은 남기고 그 뒤(쪽 나눔 표)는 뺀다.
    section.paragraphs.truncate(2);
    section.raw_stream = None;
    section.raw_provenance = None;

    let out = rhwp::serializer::serialize_hwp(&doc)?;
    if let Some(dir) = std::path::Path::new(OUT).parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(OUT, &out)?;

    let reread = rhwp::parser::parse_hwp(&out)?;
    let Control::Table(table) = &reread.sections[0].paragraphs[0].controls[2] else {
        return Err("되읽은 문서에 표가 없다".into());
    };
    assert_eq!((table.row_count, table.col_count), (3, 2));
    let stamp_cell = &table.cells[5];
    assert_eq!(
        stamp_cell
            .paragraphs
            .iter()
            .map(|p| (p.controls.len(), p.line_segs.len()))
            .collect::<Vec<_>>(),
        vec![(1, 0)],
        "도장 칸: 저장 줄 없는 그림 문단 하나"
    );
    println!("wrote {OUT} ({} bytes)", out.len());
    Ok(())
}
