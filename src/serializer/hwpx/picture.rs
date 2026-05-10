//! `<hp:pic>` 그림 직렬화 + `<hc:img binaryItemIDRef>` 참조.
//!
//! Stage 4 (#182): Picture IR → `<hp:pic>` + `<hc:img>`. BinData 참조는
//! `SerializeContext::bin_data_map` 을 통해 manifest id 로 변환된다.
//!
//! 속성·자식 순서는 한컴 OWPML 공식 (hancom-io/hwpx-owpml-model, Apache 2.0)
//! `Class/Para/PictureType.cpp` 의 `WriteElement()`, `InitMap()` 기준.
//!
//! ## 자식 순서 (PictureType.cpp:79-102)
//!
//! 부모(AbstractShapeObjectType): sz, pos, outMargin, caption, shapeComment,
//! parameterset, metaTag
//! 부모(AbstractShapeComponentType): offset, orgSz, curSz, flip, rotationInfo,
//! renderingInfo, lineShape, imgRect
//! 자신: imgClip, effects, inMargin, imgDim, img
//!
//! 한컴 관찰 샘플에서 실제 출력은: offset → orgSz → curSz → flip → rotationInfo →
//! renderingInfo → imgRect → imgClip → inMargin → imgDim → img → effects → sz → pos → outMargin
//! (부모 요소들이 자신보다 뒤에 출력됨 — XMLSerializer 구현 특성)
//!
//! ## 3-way 단언
//!
//! `<hc:img binaryItemIDRef>` 에 쓸 manifest id 는 반드시 `ctx.bin_data_map` 에 등록돼
//! 있어야 한다. 등록되지 않은 bin_data_id 참조 시 `SerializeError::XmlError` 반환.

use std::io::Write;

use quick_xml::Writer;

use crate::model::image::{ImageEffect, Picture};
use crate::model::shape::{CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo};

use super::context::SerializeContext;
use super::utils::{empty_tag, end_tag, start_tag, start_tag_attrs};
use super::SerializeError;

/// `<hp:pic>` 직렬화 진입점.
pub fn write_picture<W: Write>(
    w: &mut Writer<W>,
    pic: &Picture,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    // --- <hp:pic> 속성 ---
    // 속성 순서 (PictureType + 부모 AbstractShapeObjectType):
    // id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle,
    // href, groupLevel, instid, reverse
    let id_str = pic.common.instance_id.to_string();
    let z_order = pic.common.z_order.to_string();
    let tw = text_wrap_str(pic.common.text_wrap);
    let tf = text_flow_str(pic.common.text_wrap);
    let instid = pic.instance_id.to_string();

    start_tag_attrs(
        w,
        "hp:pic",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", "PICTURE"),
            ("textWrap", tw),
            ("textFlow", tf),
            ("lock", "0"),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", "0"),
            ("instid", &instid),
            ("reverse", "0"),
        ],
    )?;

    // --- 자식 순서 (한컴 관찰 샘플 기준) ---
    // offset, orgSz, curSz, flip, rotationInfo, renderingInfo, imgRect, imgClip,
    // inMargin, imgDim, img, effects, sz, pos, outMargin
    write_offset(w, &pic.common)?;
    write_org_sz(w)?; // ShapeComponentAttr 매핑 (IR 접근 제한으로 간이)
    write_cur_sz(w, &pic.common)?;
    write_flip(w)?;
    write_rotation_info(w)?;
    write_rendering_info(w)?;
    write_img_rect(w, &pic.common)?;
    write_img_clip(w, pic)?;
    write_in_margin(w, pic)?;
    write_img_dim(w, pic)?;
    write_img(w, pic, ctx)?; // 3-way 단언 지점
    write_effects(w)?;
    write_sz(w, &pic.common)?;
    write_pos(w, &pic.common)?;
    write_out_margin(w, &pic.common)?;

    end_tag(w, "hp:pic")?;
    Ok(())
}

// ---------- 자식 요소 ----------

fn write_offset<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let x = c.horizontal_offset.to_string();
    let y = c.vertical_offset.to_string();
    empty_tag(w, "hp:offset", &[("x", &x), ("y", &y)])
}

fn write_org_sz<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    // IR에서 원본 크기는 shape_attr.original_width/height 이나 접근이 제한적.
    // Stage 4 에선 common.width/height 를 그대로 원본 크기로 출력 (간이).
    // Picture 라운드트립 실제 정확도는 shape_attr 직접 매핑 후 향상됨.
    empty_tag(w, "hp:orgSz", &[("width", "0"), ("height", "0")])
}

fn write_cur_sz<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let width = c.width.to_string();
    let height = c.height.to_string();
    empty_tag(w, "hp:curSz", &[("width", &width), ("height", &height)])
}

fn write_flip<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    empty_tag(w, "hp:flip", &[("horizontal", "0"), ("vertical", "0")])
}

fn write_rotation_info<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    empty_tag(
        w,
        "hp:rotationInfo",
        &[("angle", "0"), ("centerX", "0"), ("centerY", "0"), ("rotateimage", "0")],
    )
}

fn write_rendering_info<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    // 3개 행렬 (transMatrix / scaMatrix / rotMatrix) 을 identity 로 출력.
    start_tag(w, "hp:renderingInfo")?;
    write_matrix(w, "hc:transMatrix")?;
    write_matrix(w, "hc:scaMatrix")?;
    write_matrix(w, "hc:rotMatrix")?;
    end_tag(w, "hp:renderingInfo")?;
    Ok(())
}

fn write_matrix<W: Write>(w: &mut Writer<W>, name: &str) -> Result<(), SerializeError> {
    empty_tag(
        w,
        name,
        &[
            ("e1", "1"),
            ("e2", "0"),
            ("e3", "0"),
            ("e4", "0"),
            ("e5", "1"),
            ("e6", "0"),
        ],
    )
}

fn write_img_rect<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    // 사각형 4개 꼭짓점 — 원본 크기 기준 직사각형
    let w_str = c.width.to_string();
    let h_str = c.height.to_string();
    start_tag(w, "hp:imgRect")?;
    empty_tag(w, "hc:pt0", &[("x", "0"), ("y", "0")])?;
    empty_tag(w, "hc:pt1", &[("x", &w_str), ("y", "0")])?;
    empty_tag(w, "hc:pt2", &[("x", &w_str), ("y", &h_str)])?;
    empty_tag(w, "hc:pt3", &[("x", "0"), ("y", &h_str)])?;
    end_tag(w, "hp:imgRect")?;
    Ok(())
}

fn write_img_clip<W: Write>(w: &mut Writer<W>, p: &Picture) -> Result<(), SerializeError> {
    let l = p.crop.left.to_string();
    let r = p.crop.right.to_string();
    let t = p.crop.top.to_string();
    let b = p.crop.bottom.to_string();
    empty_tag(
        w,
        "hp:imgClip",
        &[("left", &l), ("right", &r), ("top", &t), ("bottom", &b)],
    )
}

fn write_in_margin<W: Write>(w: &mut Writer<W>, p: &Picture) -> Result<(), SerializeError> {
    let l = p.padding.left.to_string();
    let r = p.padding.right.to_string();
    let t = p.padding.top.to_string();
    let b = p.padding.bottom.to_string();
    empty_tag(
        w,
        "hp:inMargin",
        &[("left", &l), ("right", &r), ("top", &t), ("bottom", &b)],
    )
}

fn write_img_dim<W: Write>(w: &mut Writer<W>, p: &Picture) -> Result<(), SerializeError> {
    // imgDim은 원본 크기의 clip 적용 결과. 간이 구현.
    let dw = (p.common.width as i32 - p.crop.left - p.crop.right).max(0).to_string();
    let dh = (p.common.height as i32 - p.crop.top - p.crop.bottom).max(0).to_string();
    empty_tag(w, "hp:imgDim", &[("dimwidth", &dw), ("dimheight", &dh)])
}

/// `<hc:img binaryItemIDRef>` 출력. 3-way 단언의 1차 지점.
fn write_img<W: Write>(
    w: &mut Writer<W>,
    p: &Picture,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    let bin_id = p.image_attr.bin_data_id;
    let manifest_id = ctx.resolve_bin_id(bin_id).ok_or_else(|| {
        SerializeError::XmlError(format!(
            "<hp:pic> binaryItemIDRef 미등록 bin_data_id={} (BinDataContent 누락)",
            bin_id
        ))
    })?;

    let bright = p.image_attr.brightness.to_string();
    let contrast = p.image_attr.contrast.to_string();
    let effect = image_effect_str(p.image_attr.effect);
    empty_tag(
        w,
        "hc:img",
        &[
            ("binaryItemIDRef", manifest_id),
            ("bright", &bright),
            ("contrast", &contrast),
            ("effect", effect),
            ("alpha", "0"),
        ],
    )
}

fn write_effects<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    // Stage 4 에선 빈 effects 출력 (필요시 확장).
    start_tag(w, "hp:effects")?;
    end_tag(w, "hp:effects")?;
    Ok(())
}

fn write_sz<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let width = c.width.to_string();
    let height = c.height.to_string();
    empty_tag(
        w,
        "hp:sz",
        &[
            ("width", &width),
            ("widthRelTo", "ABSOLUTE"),
            ("height", &height),
            ("heightRelTo", "ABSOLUTE"),
            ("protect", "0"),
        ],
    )
}

fn write_pos<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let treat = bool01(c.treat_as_char);
    let vert_offset = c.vertical_offset.to_string();
    let horz_offset = c.horizontal_offset.to_string();
    empty_tag(
        w,
        "hp:pos",
        &[
            ("treatAsChar", treat),
            ("affectLSpacing", "0"),
            ("flowWithText", "1"),
            ("allowOverlap", "0"),
            ("holdAnchorAndSO", "0"),
            ("vertRelTo", vert_rel_to_str(c.vert_rel_to)),
            ("horzRelTo", horz_rel_to_str(c.horz_rel_to)),
            ("vertAlign", vert_align_str(c.vert_align)),
            ("horzAlign", horz_align_str(c.horz_align)),
            ("vertOffset", &vert_offset),
            ("horzOffset", &horz_offset),
        ],
    )
}

fn write_out_margin<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let l = c.margin.left.to_string();
    let r = c.margin.right.to_string();
    let t = c.margin.top.to_string();
    let b = c.margin.bottom.to_string();
    empty_tag(
        w,
        "hp:outMargin",
        &[("left", &l), ("right", &r), ("top", &t), ("bottom", &b)],
    )
}

// ---------- 변환 헬퍼 ----------

fn bool01(b: bool) -> &'static str {
    if b { "1" } else { "0" }
}

fn text_wrap_str(w: TextWrap) -> &'static str {
    use TextWrap::*;
    match w {
        Square => "SQUARE",
        Tight => "TIGHT",
        Through => "THROUGH",
        TopAndBottom => "TOP_AND_BOTTOM",
        BehindText => "BEHIND_TEXT",
        InFrontOfText => "IN_FRONT_OF_TEXT",
    }
}

fn text_flow_str(_: TextWrap) -> &'static str {
    "BOTH_SIDES"
}

fn vert_rel_to_str(v: VertRelTo) -> &'static str {
    use VertRelTo::*;
    match v {
        Paper => "PAPER",
        Page => "PAGE",
        Para => "PARA",
    }
}

fn horz_rel_to_str(h: HorzRelTo) -> &'static str {
    use HorzRelTo::*;
    match h {
        Paper => "PAPER",
        Page => "PAGE",
        Column => "COLUMN",
        Para => "PARA",
    }
}

fn vert_align_str(v: VertAlign) -> &'static str {
    use VertAlign::*;
    match v {
        Top => "TOP",
        Center => "CENTER",
        Bottom => "BOTTOM",
        Inside => "INSIDE",
        Outside => "OUTSIDE",
    }
}

fn horz_align_str(h: HorzAlign) -> &'static str {
    use HorzAlign::*;
    match h {
        Left => "LEFT",
        Center => "CENTER",
        Right => "RIGHT",
        Inside => "INSIDE",
        Outside => "OUTSIDE",
    }
}

fn image_effect_str(e: ImageEffect) -> &'static str {
    use ImageEffect::*;
    match e {
        RealPic => "REAL_PIC",
        GrayScale => "GRAY_SCALE",
        BlackWhite => "BLACK_WHITE",
        Pattern8x8 => "PATTERN_8_8",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bin_data::BinDataContent;
    use crate::model::document::Document;
    use crate::model::image::{ImageAttr, Picture};
    use crate::serializer::hwpx::context::SerializeContext;

    fn make_picture(bin_data_id: u16) -> Picture {
        let mut pic = Picture::default();
        pic.image_attr = ImageAttr {
            bin_data_id,
            brightness: 0,
            contrast: 0,
            effect: ImageEffect::RealPic,
            external_path: None,
        };
        pic.common.width = 1000;
        pic.common.height = 500;
        pic
    }

    fn make_doc_with_bin(bin_data_id: u16, ext: &str) -> Document {
        let mut doc = Document::default();
        doc.bin_data_content.push(BinDataContent {
            id: bin_data_id,
            data: vec![0u8; 4],
            extension: ext.to_string(),
        });
        doc
    }

    fn serialize(pic: &Picture, ctx: &SerializeContext) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_picture(&mut w, pic, ctx).expect("write_picture");
        String::from_utf8(w.into_inner()).unwrap()
    }

    #[test]
    fn pic_root_attrs_in_canonical_order() {
        let doc = make_doc_with_bin(1, "png");
        let ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1);
        let xml = serialize(&pic, &ctx);
        assert!(xml.contains("<hp:pic "));
        let ip = xml.find("id=").unwrap();
        let zp = xml.find("zOrder=").unwrap();
        let nt = xml.find("numberingType=").unwrap();
        let tw = xml.find("textWrap=").unwrap();
        let href = xml.find("href=").unwrap();
        let rev = xml.find("reverse=").unwrap();
        assert!(ip < zp && zp < nt && nt < tw && tw < href && href < rev);
    }

    #[test]
    fn img_uses_manifest_id() {
        let doc = make_doc_with_bin(5, "jpg");
        let ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(5);
        let xml = serialize(&pic, &ctx);
        assert!(
            xml.contains(r#"binaryItemIDRef="image1""#),
            "binaryItemIDRef must resolve to manifest id image1: {}",
            xml
        );
    }

    #[test]
    fn unresolved_bin_data_id_errors() {
        let doc = Document::default(); // bin_data 없음
        let ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(99); // 미등록 id
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        let err = write_picture(&mut w, &pic, &ctx).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("binaryItemIDRef"), "error msg: {}", msg);
        assert!(msg.contains("99"), "error should include bin_data_id: {}", msg);
    }

    #[test]
    fn rendering_info_has_three_matrices() {
        let doc = make_doc_with_bin(1, "png");
        let ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1);
        let xml = serialize(&pic, &ctx);
        assert!(xml.contains("<hc:transMatrix "));
        assert!(xml.contains("<hc:scaMatrix "));
        assert!(xml.contains("<hc:rotMatrix "));
    }

    #[test]
    fn img_rect_has_four_points() {
        let doc = make_doc_with_bin(1, "png");
        let ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1);
        let xml = serialize(&pic, &ctx);
        assert!(xml.contains("<hc:pt0 "));
        assert!(xml.contains("<hc:pt1 "));
        assert!(xml.contains("<hc:pt2 "));
        assert!(xml.contains("<hc:pt3 "));
    }

    #[test]
    fn image_effect_maps_to_string() {
        assert_eq!(image_effect_str(ImageEffect::RealPic), "REAL_PIC");
        assert_eq!(image_effect_str(ImageEffect::GrayScale), "GRAY_SCALE");
        assert_eq!(image_effect_str(ImageEffect::BlackWhite), "BLACK_WHITE");
    }
}
