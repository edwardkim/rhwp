//! 어울림 그림 칸 앞의 빈 그림 문단 재현 fixture 생성기.
//!
//! 형상(활동 사진 칸과 같은 꼴): 저장 줄(LINE_SEG) 없는 표 칸 하나에
//!   p0 = 글자 없는 문단 + 칸 글줄 띠 밖에 앵커한 작은 어울림(Square) 그림 두 장
//!   p1 = 글자 없는 문단 + 사진 어울림(Square) 그림
//!   p2 = 이름표 글줄
//! 이 차례로 놓인다. 이름표는 사진 바닥 아래에 놓이고, 사진 문단의 원점은 p0 의 빈 줄
//! (글자 모양의 em 줄박스 + 줄 간격) 아래다. 칸 선언 높이는 내용보다 낮아 행 높이는
//! 측정이 정한다.
//!
//! 입력은 공개 합성본 `samples/title_flow_photo/title-flow-photo-row.hwp`(저장 줄 없는 원본
//! HWP5 경로) 하나다. 그 문서 첫 표의 제목 문단·그림 문단을 재료로 쓴다.
//!
//! Usage:
//!   cargo run --release --example build_square_floor_empty_lead_fixture

use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use std::fs;

const INPUT: &str = "samples/title_flow_photo/title-flow-photo-row.hwp";
const OUT: &str = "samples/square_floor_empty_lead/cell-label-below-picture-after-empty-anchor.hwp";

/// 앞 빈 문단의 그림 크기(HU).
const LEAD_W: u32 = 2000;
const LEAD_H: u32 = 1000;
/// 앞 그림의 가로 오프셋(HU) — 칸 폭(9000) 밖이라 글줄을 막지 않는다.
const LEAD_X: u32 = 30825;
/// 칸 선언 높이(HU) — 내용보다 낮다.
const CELL_H: u32 = 3000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(INPUT)?;
    let mut doc = rhwp::parser::parse_hwp(&bytes)?;
    let section = &mut doc.sections[0];
    {
        let Control::Table(table) = &mut section.paragraphs[0].controls[2] else {
            return Err("입력 문서 0.0 의 셋째 개체가 표가 아니다".into());
        };
        let cell = &mut table.cells[0];
        let label = cell.paragraphs[0].clone();
        let mut photo = cell.paragraphs[1].clone();
        assert!(label.line_segs.is_empty() && photo.line_segs.is_empty());
        let Control::Picture(pic) = &mut photo.controls[0] else {
            return Err("입력 칸 둘째 문단이 그림 문단이 아니다".into());
        };
        pic.common.text_wrap = TextWrap::Square;

        let mut lead = photo.clone();
        let Control::Picture(small) = &mut lead.controls[0] else {
            unreachable!()
        };
        small.common.width = LEAD_W;
        small.common.height = LEAD_H;
        small.common.horizontal_offset = LEAD_X;
        small.shape_attr.current_width = LEAD_W;
        small.shape_attr.current_height = LEAD_H;
        let second = lead.controls[0].clone();
        lead.controls.push(second);
        // 개체 문자 두 개(8 유닛씩) + 문단 끝.
        lead.char_count = photo.char_count + 8;

        cell.paragraphs = vec![lead, photo, label];
        cell.height = CELL_H;
    }
    // 뒤 문단(표 2·3)은 뺀다.
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
    let Control::Table(table) = &reread.sections[0].paragraphs[0].controls[2] else {
        return Err("되읽은 문서에 표가 없다".into());
    };
    let cell = &table.cells[0];
    let shape: Vec<(usize, usize)> = cell
        .paragraphs
        .iter()
        .map(|p| (p.controls.len(), p.line_segs.len()))
        .collect();
    assert_eq!(
        shape,
        vec![(2, 0), (1, 0), (0, 0)],
        "칸 문단의 (개체 수, 저장 줄 수)"
    );
    println!("wrote {OUT} ({} bytes)", out.len());
    Ok(())
}
