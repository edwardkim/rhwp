//! `CommonObjAttr` → CTRL_HEADER `raw_ctrl_data` 바이트 직렬화기.
//!
//! HWP 직렬화기 (`serializer/control.rs:349`) 는 `table.raw_ctrl_data` 를 그대로 기록한다.
//! HWPX 출처 표는 이 필드가 비어있으므로, 어댑터가 `CommonObjAttr` 으로부터 합성해야 한다.
//!
//! 본 모듈은 [`crate::parser::control::shape::parse_common_obj_attr`] 의 역방향이며,
//! 라운드트립 테스트로 검증한다.

use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, SizeCriterion, TextFlow, TextWrap, VertAlign, VertRelTo,
};
use crate::serializer::byte_writer::ByteWriter;

/// `CommonObjAttr` 을 CTRL_HEADER ctrl_data 영역 바이트로 직렬화.
///
/// 레이아웃 (parser/control/shape.rs:247 `parse_common_obj_attr` 의 역방향):
/// - attr (u32, 비트 필드)
/// - vertical_offset (u32)
/// - horizontal_offset (u32)
/// - width (u32)
/// - height (u32)
/// - z_order (i32)
/// - margin.left/right/top/bottom (i16 * 4)
/// - instance_id (u32)
/// - prevent_page_break (i32)
/// - description (HWP string: u16 length + UTF-16LE)
/// - raw_extra (그대로 이어붙임 — 라운드트립 보존)
pub fn serialize_common_obj_attr(common: &CommonObjAttr) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // attr 비트 필드 재구성: HWPX 출처는 attr=0 이므로 enum 으로부터 비트 합성.
    // HWP 출처는 attr 가 이미 채워져 있으므로 그대로 사용.
    let attr = if common.attr != 0 {
        common.attr
    } else {
        pack_common_attr_bits(common)
    };
    w.write_u32(attr).unwrap();

    w.write_u32(common.vertical_offset).unwrap();
    w.write_u32(common.horizontal_offset).unwrap();
    w.write_u32(common.width).unwrap();
    w.write_u32(common.height).unwrap();
    w.write_i32(common.z_order).unwrap();

    w.write_i16(common.margin.left).unwrap();
    w.write_i16(common.margin.right).unwrap();
    w.write_i16(common.margin.top).unwrap();
    w.write_i16(common.margin.bottom).unwrap();

    w.write_u32(common.instance_id).unwrap();
    w.write_i32(common.prevent_page_break).unwrap();

    // description (HWP string)
    w.write_hwp_string(&common.description).unwrap();

    // 라운드트립 보존: raw_extra 가 있으면 이어붙임
    if !common.raw_extra.is_empty() {
        w.write_bytes(&common.raw_extra).unwrap();
    }

    w.into_bytes()
}

/// `CommonObjAttr` 의 enum 필드들로부터 attr u32 비트를 합성한다.
///
/// 비트 레이아웃 (parser/control/shape.rs 의 역방향):
/// - bit 0: treat_as_char
/// - bit 3-4: vert_rel_to (Paper=0, Page=1, Para=2)
/// - bit 5-7: vert_align
/// - bit 8-9: horz_rel_to
/// - bit 10-12: horz_align
/// - bit 13: flow_with_text (HWPX object contract)
/// - bit 14: allow_overlap (HWPX object contract)
/// - bit 15-17: width_criterion
/// - bit 18-19: height_criterion
/// - bit 21-23: text_wrap
/// - bit 24-25: text_flow
/// - bit 20: size protect when VertRelTo is Para
/// - bit 26: HWPX GenShape storage high bit 후보
/// - bit 28: HWPX GenShape numbering category high bit 후보
pub(crate) fn pack_common_attr_bits(common: &CommonObjAttr) -> u32 {
    let mut a: u32 = 0;
    if common.treat_as_char {
        a |= 0x01;
    }
    a |= (vert_rel_to_to_bits(common.vert_rel_to) & 0x03) << 3;
    a |= (vert_align_to_bits(common.vert_align) & 0x07) << 5;
    a |= (horz_rel_to_to_bits(common.horz_rel_to) & 0x03) << 8;
    a |= (horz_align_to_bits(common.horz_align) & 0x07) << 10;
    if common.flow_with_text {
        a |= 1 << 13;
    }
    if common.allow_overlap {
        a |= 1 << 14;
    }
    if common.size_protect {
        a |= 1 << 20;
    }
    a |= (width_criterion_to_bits(common.width_criterion) & 0x07) << 15;
    a |= (height_criterion_to_bits(common.height_criterion) & 0x03) << 18;
    a |= (text_wrap_to_bits(common.text_wrap) & 0x07) << 21;
    a |= (text_flow_to_bits(common.text_flow) & 0x03) << 24;
    if common.hwp5_gen_shape_attr_bit26 {
        a |= 1 << 26;
    }
    if common.hwp5_gen_shape_attr_bit28 {
        a |= 1 << 28;
    }
    a
}

fn vert_rel_to_to_bits(v: VertRelTo) -> u32 {
    match v {
        VertRelTo::Paper => 0,
        VertRelTo::Page => 1,
        VertRelTo::Para => 2,
    }
}

fn vert_align_to_bits(v: VertAlign) -> u32 {
    match v {
        VertAlign::Top => 0,
        VertAlign::Center => 1,
        VertAlign::Bottom => 2,
        VertAlign::Inside => 3,
        VertAlign::Outside => 4,
    }
}

fn horz_rel_to_to_bits(v: HorzRelTo) -> u32 {
    match v {
        HorzRelTo::Paper => 0,
        HorzRelTo::Page => 1,
        HorzRelTo::Column => 2,
        HorzRelTo::Para => 3,
    }
}

fn horz_align_to_bits(v: HorzAlign) -> u32 {
    match v {
        HorzAlign::Left => 0,
        HorzAlign::Center => 1,
        HorzAlign::Right => 2,
        HorzAlign::Inside => 3,
        HorzAlign::Outside => 4,
    }
}

fn width_criterion_to_bits(v: SizeCriterion) -> u32 {
    match v {
        SizeCriterion::Paper => 0,
        SizeCriterion::Page => 1,
        SizeCriterion::Column => 2,
        SizeCriterion::Para => 3,
        SizeCriterion::Absolute => 4,
    }
}

fn height_criterion_to_bits(v: SizeCriterion) -> u32 {
    match v {
        SizeCriterion::Paper => 0,
        SizeCriterion::Page => 1,
        // height 는 Absolute 만 의미 있음 (parser bit 18-19, 2비트만 사용)
        _ => 2,
    }
}

fn text_wrap_to_bits(v: TextWrap) -> u32 {
    // hwplib 기준: 0=어울림(Square), 1=자리차지(TopAndBottom), 2=글뒤로(BehindText), 3=글앞으로(InFrontOfText)
    // Tight/Through 는 HWP 5.0 에 직접 매핑이 없어 Square 로 폴백.
    match v {
        TextWrap::Square => 0,
        TextWrap::Tight => 0,
        TextWrap::Through => 0,
        TextWrap::TopAndBottom => 1,
        TextWrap::BehindText => 2,
        TextWrap::InFrontOfText => 3,
    }
}

fn text_flow_to_bits(v: TextFlow) -> u32 {
    match v {
        TextFlow::BothSides => 0,
        TextFlow::LeftOnly => 1,
        TextFlow::RightOnly => 2,
        TextFlow::LargestOnly => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Padding;
    use crate::parser::control::parse_common_obj_attr;

    fn make_sample() -> CommonObjAttr {
        CommonObjAttr {
            ctrl_id: 0,
            attr: 0,
            vertical_offset: 1234,
            horizontal_offset: 5678,
            width: 12000,
            height: 8000,
            z_order: 7,
            margin: Padding {
                left: 100,
                right: 200,
                top: 300,
                bottom: 400,
            },
            instance_id: 0xCAFEBABE,
            prevent_page_break: 0,
            treat_as_char: false,
            flow_with_text: false,
            allow_overlap: false,
            hwp5_gen_shape_attr_bit26: false,
            size_protect: false,
            hwp5_gen_shape_attr_bit28: false,
            vert_rel_to: VertRelTo::Para,
            vert_align: VertAlign::Top,
            horz_rel_to: HorzRelTo::Para,
            horz_align: HorzAlign::Left,
            text_wrap: TextWrap::TopAndBottom,
            text_flow: TextFlow::BothSides,
            width_criterion: SizeCriterion::Absolute,
            height_criterion: SizeCriterion::Absolute,
            description: String::new(),
            raw_extra: Vec::new(),
        }
    }

    #[test]
    fn roundtrip_default() {
        let original = make_sample();
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);

        assert_eq!(parsed.vertical_offset, original.vertical_offset);
        assert_eq!(parsed.horizontal_offset, original.horizontal_offset);
        assert_eq!(parsed.width, original.width);
        assert_eq!(parsed.height, original.height);
        assert_eq!(parsed.z_order, original.z_order);
        assert_eq!(parsed.margin.left, original.margin.left);
        assert_eq!(parsed.margin.right, original.margin.right);
        assert_eq!(parsed.margin.top, original.margin.top);
        assert_eq!(parsed.margin.bottom, original.margin.bottom);
        assert_eq!(parsed.instance_id, original.instance_id);
        assert_eq!(parsed.prevent_page_break, original.prevent_page_break);
        assert_eq!(parsed.treat_as_char, original.treat_as_char);
        assert_eq!(parsed.flow_with_text, original.flow_with_text);
        assert_eq!(parsed.allow_overlap, original.allow_overlap);
        assert_eq!(
            parsed.hwp5_gen_shape_attr_bit26,
            original.hwp5_gen_shape_attr_bit26
        );
        assert_eq!(parsed.size_protect, original.size_protect);
        assert_eq!(
            parsed.hwp5_gen_shape_attr_bit28,
            original.hwp5_gen_shape_attr_bit28
        );
        assert_eq!(parsed.vert_rel_to, original.vert_rel_to);
        assert_eq!(parsed.horz_rel_to, original.horz_rel_to);
        assert_eq!(parsed.text_wrap, original.text_wrap);
    }

    #[test]
    fn roundtrip_treat_as_char() {
        let mut original = make_sample();
        original.treat_as_char = true;
        original.text_wrap = TextWrap::BehindText;
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert!(parsed.treat_as_char);
        assert_eq!(parsed.text_wrap, TextWrap::BehindText);
    }

    #[test]
    fn roundtrip_hwpx_object_contract_bits() {
        let mut original = make_sample();
        original.flow_with_text = true;
        original.allow_overlap = true;
        original.hwp5_gen_shape_attr_bit26 = true;
        original.size_protect = true;
        original.hwp5_gen_shape_attr_bit28 = true;

        let bytes = serialize_common_obj_attr(&original);
        let attr = u32::from_le_bytes(bytes[0..4].try_into().unwrap());

        assert_ne!(attr & (1 << 13), 0);
        assert_ne!(attr & (1 << 14), 0);
        assert_ne!(attr & (1 << 20), 0);
        assert_ne!(attr & (1 << 26), 0);
        assert_ne!(attr & (1 << 28), 0);

        let parsed = parse_common_obj_attr(&bytes);
        assert!(parsed.flow_with_text);
        assert!(parsed.allow_overlap);
        assert!(parsed.size_protect);
        assert!(parsed.hwp5_gen_shape_attr_bit26);
        assert!(parsed.hwp5_gen_shape_attr_bit28);
    }

    #[test]
    fn roundtrip_with_description() {
        let mut original = make_sample();
        original.description = "표 설명".to_string();
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert_eq!(parsed.description, "표 설명");
    }

    #[test]
    fn preserves_existing_attr_when_nonzero() {
        let mut original = make_sample();
        original.attr = 0xDEADBEEF;
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert_eq!(parsed.attr, 0xDEADBEEF);
    }

    #[test]
    fn synthesizes_attr_when_zero() {
        // HWPX 출처 시뮬레이션: attr=0, enum 필드만 채워짐
        let original = make_sample();
        assert_eq!(original.attr, 0);
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert_ne!(parsed.attr, 0, "attr 비트가 enum 으로부터 합성돼야 함");
        assert_eq!(parsed.text_wrap, TextWrap::TopAndBottom);
        assert_eq!(parsed.vert_rel_to, VertRelTo::Para);
    }

    #[test]
    fn produces_min_43_bytes() {
        // 한컴 스펙: ctrl_data 최소 ~43바이트 (CommonObjAttr 헤더)
        let bytes = serialize_common_obj_attr(&make_sample());
        // attr(4) + vert_off(4) + horz_off(4) + w(4) + h(4) + z(4)
        //  + margin(8) + inst(4) + prev(4) + desc_len(2) = 42
        assert!(
            bytes.len() >= 42,
            "예상 42바이트 이상, 실제={}",
            bytes.len()
        );
    }

    #[test]
    fn roundtrip_text_flow_left_only() {
        // HWPX 출처: attr=0, text_flow=LeftOnly → bits 24-25 = 0b01
        let mut original = make_sample();
        original.text_flow = TextFlow::LeftOnly;
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert_eq!(parsed.text_flow, TextFlow::LeftOnly);
    }

    #[test]
    fn roundtrip_text_flow_right_only() {
        let mut original = make_sample();
        original.text_flow = TextFlow::RightOnly;
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert_eq!(parsed.text_flow, TextFlow::RightOnly);
    }

    #[test]
    fn roundtrip_text_flow_largest_only() {
        let mut original = make_sample();
        original.text_flow = TextFlow::LargestOnly;
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert_eq!(parsed.text_flow, TextFlow::LargestOnly);
    }

    #[test]
    fn text_flow_bits_position() {
        // bit 24-25 에 정확히 위치하는지 검증
        let mut original = make_sample();
        original.text_flow = TextFlow::LeftOnly; // 0b01
        let bytes = serialize_common_obj_attr(&original);
        let attr = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        assert_eq!((attr >> 24) & 0x03, 1, "LeftOnly 는 bit24-25 = 0b01");

        original.text_flow = TextFlow::RightOnly; // 0b10
        let bytes = serialize_common_obj_attr(&original);
        let attr = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        assert_eq!((attr >> 24) & 0x03, 2, "RightOnly 는 bit24-25 = 0b10");
    }

    #[test]
    fn text_flow_default_is_both_sides() {
        // 기본값이 BOTH_SIDES(0) 인지 확인
        let original = make_sample();
        assert_eq!(original.text_flow, TextFlow::BothSides);
        let bytes = serialize_common_obj_attr(&original);
        let parsed = parse_common_obj_attr(&bytes);
        assert_eq!(parsed.text_flow, TextFlow::BothSides);
    }
}
