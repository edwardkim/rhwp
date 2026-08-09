//! `CommonObjAttr` → CTRL_HEADER `raw_ctrl_data` 바이트 직렬화기.
//!
//! HWP 직렬화기 (`serializer/control.rs:349`) 는 `table.raw_ctrl_data` 를 그대로 기록한다.
//! HWPX 출처 표는 이 필드가 비어있으므로, 어댑터가 `CommonObjAttr` 으로부터 합성해야 한다.
//!
//! 본 모듈은 [`crate::parser::control::shape::parse_common_obj_attr`] 의 역방향이며,
//! 라운드트립 테스트로 검증한다.

use crate::model::shape::CommonObjAttr;
use crate::serializer::byte_writer::ByteWriter;
// [#4400] 비트 팩킹은 본질적으로 직렬화 로직이라 `src/serializer/control.rs` 로 옮겼다 —
// 직렬화기가 `document_core::converters` 를 참조하는 역방향 의존을 없앤다. 이 어댑터와
// `sync_anchor_bits` 는 그 pub(crate) 함수를 그대로 가져다 쓴다.
use crate::serializer::control::{horz_rel_to_to_bits, pack_common_attr_bits, vert_rel_to_to_bits};

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

/// tac/rel_to 마이그레이션 후 stale packed `attr` 동기화.
///
/// 배경 (Issue #3781 실측): `insert_picture_native` 가 `attr` 비트를 seed 하고
/// `migrate_picture_floating_to_inline` 은 **enum 필드만** 갱신한다. 직렬화는
/// `attr != 0` 이면 packed 값을 우선하므로(라운드트립 보존 계약), 마이그레이션된
/// 그림이 HWP 바이너리 왕복에서 floating(Paper) 앵커로 되살아나
/// `treatAsChar=1 + vertRelTo=PAPER` 모순 산출물이 된다 (한글 렌더 깨짐).
/// 앵커 관련 비트(tac bit0 · vert_rel 3-4 · horz_rel 8-9)만 enum 에서 재기입하고,
/// criterion/wrap/flow 등 나머지 비트는 보존한다 (전체 재패킹은 enum 미동기
/// 필드의 정보 손실 위험).
pub(crate) fn sync_anchor_bits(common: &mut CommonObjAttr) {
    if common.attr == 0 {
        return; // 직렬화가 pack_common_attr_bits 로 전량 재패킹 — 손댈 것 없음.
    }
    let mut a = common.attr;
    a &= !(0x01 | (0x03 << 3) | (0x03 << 8));
    if common.treat_as_char {
        a |= 0x01;
    }
    a |= (vert_rel_to_to_bits(common.vert_rel_to) & 0x03) << 3;
    a |= (horz_rel_to_to_bits(common.horz_rel_to) & 0x03) << 8;
    common.attr = a;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::shape::{
        HorzAlign, HorzRelTo, SizeCriterion, TextFlow, TextWrap, VertAlign, VertRelTo,
    };
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
            affect_line_spacing: false,
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
            locked: false,
            numbering_type: crate::model::shape::ObjectNumberingType::None,
            drop_cap_style: crate::model::shape::DropCapStyle::None,
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
    fn roundtrip_affect_line_spacing_bit2() {
        // [#2784] affectLSpacing 은 개체 공통 속성 attr bit 2 (스펙 표 70) 에 위치.
        // 한컴 원본 issue1949.hwp 의 bit 2 set 개체(표 1 + 수식 5)가 짝 HWPX 의
        // affectLSpacing="1" 개체와 1:1 일치함으로 실파일 검증됨.
        let mut original = make_sample();
        original.affect_line_spacing = true;
        let bytes = serialize_common_obj_attr(&original);
        let attr = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        assert_ne!(
            attr & (1 << 2),
            0,
            "affect_line_spacing 은 bit 2 에 팩돼야 함"
        );
        assert_eq!(attr & (1 << 1), 0, "bit 1 은 예약이라 세팅되면 안 됨");
        let parsed = parse_common_obj_attr(&bytes);
        assert!(parsed.affect_line_spacing);
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

#[cfg(test)]
mod sync_anchor_bits_tests {
    use super::*;
    use crate::model::shape::{HorzRelTo, VertRelTo};

    /// Issue #3781 실측 회귀: insert 가 seed 한 floating attr
    /// ((4<<15)|(2<<18) — tac=0·rel=Paper) 위에서 inline 마이그레이션(enum 갱신) 후
    /// sync 하면 tac/rel 비트만 Para 로 바뀌고 criterion 비트는 보존된다.
    #[test]
    fn sync_anchor_bits_updates_stale_floating_seed() {
        let mut common = CommonObjAttr {
            attr: (4 << 15) | (2 << 18),
            treat_as_char: false,
            vert_rel_to: VertRelTo::Paper,
            horz_rel_to: HorzRelTo::Paper,
            ..Default::default()
        };
        // 마이그레이션이 하는 일 (enum 갱신).
        common.treat_as_char = true;
        common.vert_rel_to = VertRelTo::Para;
        common.horz_rel_to = HorzRelTo::Para;
        sync_anchor_bits(&mut common);
        assert_eq!(common.attr & 0x01, 0x01, "tac bit");
        assert_eq!((common.attr >> 3) & 0x03, 2, "vert_rel_to = Para");
        assert_eq!(
            (common.attr >> 8) & 0x03,
            3,
            "horz_rel_to = Para (Paper0/Page1/Column2/Para3)"
        );
        assert_eq!((common.attr >> 15) & 0x07, 4, "width criterion 보존");
        assert_eq!((common.attr >> 18) & 0x03, 2, "height criterion 보존");
    }

    /// attr=0(합성 경로) 은 무접촉 — 직렬화가 전량 재패킹한다.
    #[test]
    fn sync_anchor_bits_leaves_zero_attr_untouched() {
        let mut common = CommonObjAttr {
            attr: 0,
            treat_as_char: true,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Para,
            ..Default::default()
        };
        sync_anchor_bits(&mut common);
        assert_eq!(common.attr, 0);
    }
}
