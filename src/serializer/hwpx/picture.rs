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

use crate::model::image::{
    EffectColor, EffectPoint, EffectRgb, ImageEffect, Picture, PictureShadow,
};
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, ShapeComponentAttr, TextFlow, TextWrap, VertAlign,
    VertRelTo,
};

use super::context::SerializeContext;
use super::table::write_caption;
use super::utils::{empty_tag, end_tag, start_tag, start_tag_attrs};
use super::SerializeError;

/// `<hp:pic>` 직렬화 진입점.
///
/// ctx 가 `&mut` 인 이유: 캡션 subList 문단 직렬화(#1403)가 para id 발급과
/// para_shape/style 참조 수집을 수행한다 (표 캡션 #1387 과 동일 경로).
pub fn write_picture<W: Write>(
    w: &mut Writer<W>,
    pic: &Picture,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    // --- <hp:pic> 속성 ---
    // 속성 순서 (PictureType + 부모 AbstractShapeObjectType):
    // id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle,
    // href, groupLevel, instid, reverse
    let id_str = pic.common.instance_id.to_string();
    let z_order = pic.common.z_order.to_string();
    let tw = text_wrap_str(pic.common.text_wrap);
    let tf = text_flow_str(pic.common.text_flow);
    let instid = pic.instance_id.to_string();
    let href = pic.href.as_deref().unwrap_or("");

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
            ("href", href),
            ("groupLevel", "0"),
            ("instid", &instid),
            ("reverse", "0"),
        ],
    )?;

    // --- 자식 순서 (한컴 관찰 샘플 기준) ---
    // offset, orgSz, curSz, flip, rotationInfo, renderingInfo, imgRect, imgClip,
    // inMargin, imgDim, img, effects, sz, pos, outMargin
    write_offset(w, &pic.common)?;
    write_org_sz(w, &pic.shape_attr)?;
    write_cur_sz(w, pic)?;
    write_flip(w, &pic.shape_attr)?;
    write_rotation_info(w, &pic.shape_attr)?;
    // [#1501] 그룹 자식 pic 의 transMatrix(render_tx/sx) 보존 — 종전 identity 고정 출력은
    // 그룹 내 자식을 원점·고유크기로 붕괴시켰다. shape.rs 의 raw_rendering 디코더 공유.
    super::shape::write_rendering_info(w, &pic.shape_attr)?;
    write_img_rect(w, pic)?;
    write_img_clip(w, pic)?;
    write_in_margin(w, pic)?;
    write_img_dim(w, pic)?;
    write_img(w, pic, ctx)?; // 3-way 단언 지점
    write_effects(w, pic)?;
    write_sz(w, &pic.common)?;
    write_pos(w, &pic.common)?;
    write_out_margin(w, &pic.common)?;
    // 캡션 (#1403) — 한컴 실물(aift.hwpx) 자식 순서: outMargin 뒤
    if let Some(cap) = &pic.caption {
        write_caption(w, cap, ctx)?;
    }
    // 설명 (#1392) — caption 직후 (aift 실물 공존 9건 전수 caption→shapeComment 순서)
    super::shape::write_shape_comment(w, &pic.common)?;

    end_tag(w, "hp:pic")?;
    Ok(())
}

// ---------- 자식 요소 ----------

fn write_offset<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let x = c.horizontal_offset.to_string();
    let y = c.vertical_offset.to_string();
    empty_tag(w, "hp:offset", &[("x", &x), ("y", &y)])
}

fn write_org_sz<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    let ow = sa.original_width.to_string();
    let oh = sa.original_height.to_string();
    empty_tag(w, "hp:orgSz", &[("width", &ow), ("height", &oh)])
}

fn write_cur_sz<W: Write>(w: &mut Writer<W>, p: &Picture) -> Result<(), SerializeError> {
    // [#1389] 현재 크기는 shape_attr.current_width/height (IR 보존). 0 이면 common(sz)
    // 폴백 — 원본도 그 경우 sz=curSz. 종전 common 직출이라 current≠sz 인 pic 변형.
    // [#2017] 파싱 시 orgSz로 materialize된 dimension 은 원본 `0` sentinel 로 복원.
    let cw = if p.shape_attr.current_width_was_zero {
        0
    } else if p.shape_attr.current_width > 0 {
        p.shape_attr.current_width
    } else {
        p.common.width
    };
    let ch = if p.shape_attr.current_height_was_zero {
        0
    } else if p.shape_attr.current_height > 0 {
        p.shape_attr.current_height
    } else {
        p.common.height
    };
    empty_tag(
        w,
        "hp:curSz",
        &[("width", &cw.to_string()), ("height", &ch.to_string())],
    )
}

fn write_flip<W: Write>(w: &mut Writer<W>, sa: &ShapeComponentAttr) -> Result<(), SerializeError> {
    let h = bool01(sa.horz_flip);
    let v = bool01(sa.vert_flip);
    empty_tag(w, "hp:flip", &[("horizontal", h), ("vertical", v)])
}

fn write_rotation_info<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    let angle = sa.rotation_angle.to_string();
    let cx = sa.rotation_center.x.to_string();
    let cy = sa.rotation_center.y.to_string();
    let ri = bool01(sa.rotate_image);
    empty_tag(
        w,
        "hp:rotationInfo",
        &[
            ("angle", &angle),
            ("centerX", &cx),
            ("centerY", &cy),
            ("rotateimage", ri),
        ],
    )
}

fn write_img_rect<W: Write>(w: &mut Writer<W>, p: &Picture) -> Result<(), SerializeError> {
    // [#1389] 꼭짓점은 border_x/border_y (IR 보존). 파서(parse_picture_img_rect)는
    // HWP5 SHAPE_PICTURE 스칼라 레이아웃으로 저장한다:
    //   border_x = [pt0.x, pt0.y, pt1.x, pt1.y], border_y = [pt2.x, pt2.y, pt3.x, pt3.y]
    // 따라서 역매핑하여 pt0~pt3 을 복원한다. 모두 0(미적재)이면 common 합성 폴백.
    let bx = &p.border_x;
    let by = &p.border_y;
    if bx.iter().all(|&v| v == 0) && by.iter().all(|&v| v == 0) {
        let w_str = p.common.width.to_string();
        let h_str = p.common.height.to_string();
        start_tag(w, "hp:imgRect")?;
        empty_tag(w, "hc:pt0", &[("x", "0"), ("y", "0")])?;
        empty_tag(w, "hc:pt1", &[("x", &w_str), ("y", "0")])?;
        empty_tag(w, "hc:pt2", &[("x", &w_str), ("y", &h_str)])?;
        empty_tag(w, "hc:pt3", &[("x", "0"), ("y", &h_str)])?;
        end_tag(w, "hp:imgRect")?;
        return Ok(());
    }
    let pts = [
        (bx[0], bx[1]), // pt0
        (bx[2], bx[3]), // pt1
        (by[0], by[1]), // pt2
        (by[2], by[3]), // pt3
    ];
    start_tag(w, "hp:imgRect")?;
    for (i, (x, y)) in pts.iter().enumerate() {
        empty_tag(
            w,
            &format!("hc:pt{i}"),
            &[("x", &x.to_string()), ("y", &y.to_string())],
        )?;
    }
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
    // [#1389] 원본 이미지 픽셀 크기 verbatim (IR img_dim). 종전 간이 계산
    // (common - crop)은 imgClip extent 의미 오해로 음수→0 변형이었다.
    empty_tag(
        w,
        "hp:imgDim",
        &[
            ("dimwidth", &p.img_dim.0.to_string()),
            ("dimheight", &p.img_dim.1.to_string()),
        ],
    )
}

/// `<hc:img binaryItemIDRef>` 출력. 3-way 단언의 1차 지점.
fn write_img<W: Write>(
    w: &mut Writer<W>,
    p: &Picture,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    let bin_id = p.image_attr.bin_data_id;
    // #1567: bin_id==0 은 원본 `binaryItemIDRef=""`(이미지 참조 없는 placeholder pic, 표 셀
    // 등)에 대응한다(파서 `unwrap_or(0)`). resolve 실패해도 빈 ref 를 verbatim 방출해
    // pic 컨트롤을 보존한다(종전: Err → 호출자 section.rs:701 이 조용히 드롭 → IR_DIFF).
    // 비-0 미해결은 진짜 BinDataContent 누락이므로 진단(Err)을 유지해 손실 은폐를 막는다.
    let manifest_id = match ctx.resolve_bin_id(bin_id) {
        Some(id) => id,
        None if bin_id == 0 => "",
        None => {
            return Err(SerializeError::XmlError(format!(
                "<hp:pic> binaryItemIDRef 미등록 bin_data_id={} (BinDataContent 누락)",
                bin_id
            )))
        }
    };

    let bright = p.image_attr.brightness.to_string();
    let contrast = p.image_attr.contrast.to_string();
    let effect = image_effect_str(p.image_attr.effect);
    let alpha = picture_alpha_str(p.image_attr.clamped_transparency());
    empty_tag(
        w,
        "hc:img",
        &[
            ("binaryItemIDRef", manifest_id),
            ("bright", &bright),
            ("contrast", &contrast),
            ("effect", effect),
            ("alpha", &alpha),
        ],
    )
}

fn picture_alpha_str(transparency: u8) -> String {
    crate::model::image::transparency_percent_to_alpha_byte(transparency).to_string()
}

fn write_effects<W: Write>(w: &mut Writer<W>, pic: &Picture) -> Result<(), SerializeError> {
    start_tag(w, "hp:effects")?;
    if let Some(shadow) = &pic.effects.shadow {
        write_shadow(w, shadow)?;
    }
    end_tag(w, "hp:effects")?;
    Ok(())
}

fn write_shadow<W: Write>(w: &mut Writer<W>, shadow: &PictureShadow) -> Result<(), SerializeError> {
    let mut attrs = Vec::new();
    push_opt_attr(&mut attrs, "style", shadow.style.as_deref());
    push_opt_attr(&mut attrs, "alpha", shadow.alpha.as_deref());
    push_opt_attr(&mut attrs, "radius", shadow.radius.as_deref());
    push_opt_attr(&mut attrs, "direction", shadow.direction.as_deref());
    push_opt_attr(&mut attrs, "distance", shadow.distance.as_deref());
    push_opt_attr(&mut attrs, "alignStyle", shadow.align_style.as_deref());
    push_opt_attr(
        &mut attrs,
        "rotationStyle",
        shadow.rotation_style.as_deref(),
    );

    start_tag_attrs(w, "hp:shadow", &attrs)?;
    if let Some(skew) = &shadow.skew {
        write_effect_point(w, "hp:skew", skew)?;
    }
    if let Some(scale) = &shadow.scale {
        write_effect_point(w, "hp:scale", scale)?;
    }
    if let Some(color) = &shadow.color {
        write_effect_color(w, color)?;
    }
    end_tag(w, "hp:shadow")?;
    Ok(())
}

fn write_effect_point<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    point: &EffectPoint,
) -> Result<(), SerializeError> {
    let mut attrs = Vec::new();
    push_opt_attr(&mut attrs, "x", point.x.as_deref());
    push_opt_attr(&mut attrs, "y", point.y.as_deref());
    empty_tag(w, name, &attrs)
}

fn write_effect_color<W: Write>(
    w: &mut Writer<W>,
    color: &EffectColor,
) -> Result<(), SerializeError> {
    let mut attrs = Vec::new();
    push_opt_attr(&mut attrs, "type", color.color_type.as_deref());
    push_opt_attr(&mut attrs, "schemeIdx", color.scheme_idx.as_deref());
    push_opt_attr(&mut attrs, "systemIdx", color.system_idx.as_deref());
    push_opt_attr(&mut attrs, "presetIdx", color.preset_idx.as_deref());

    if let Some(rgb) = &color.rgb {
        start_tag_attrs(w, "hp:effectsColor", &attrs)?;
        write_effect_rgb(w, rgb)?;
        end_tag(w, "hp:effectsColor")?;
    } else {
        empty_tag(w, "hp:effectsColor", &attrs)?;
    }
    Ok(())
}

fn write_effect_rgb<W: Write>(w: &mut Writer<W>, rgb: &EffectRgb) -> Result<(), SerializeError> {
    let mut attrs = Vec::new();
    push_opt_attr(&mut attrs, "r", rgb.r.as_deref());
    push_opt_attr(&mut attrs, "g", rgb.g.as_deref());
    push_opt_attr(&mut attrs, "b", rgb.b.as_deref());
    empty_tag(w, "hp:rgb", &attrs)
}

fn push_opt_attr<'a>(
    attrs: &mut Vec<(&'static str, &'a str)>,
    key: &'static str,
    value: Option<&'a str>,
) {
    if let Some(value) = value {
        attrs.push((key, value));
    }
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
    let flow_with_text = bool01(c.flow_with_text);
    let allow_overlap = bool01(c.allow_overlap);
    let vert_offset = c.vertical_offset.to_string();
    let horz_offset = c.horizontal_offset.to_string();
    let hold = bool01(c.prevent_page_break != 0); // [#1594] IR 보존
    empty_tag(
        w,
        "hp:pos",
        &[
            ("treatAsChar", treat),
            ("affectLSpacing", "0"),
            ("flowWithText", flow_with_text),
            ("allowOverlap", allow_overlap),
            ("holdAnchorAndSO", hold),
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
    if b {
        "1"
    } else {
        "0"
    }
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

fn text_flow_str(f: TextFlow) -> &'static str {
    match f {
        TextFlow::BothSides => "BOTH_SIDES",
        TextFlow::LeftOnly => "LEFT_ONLY",
        TextFlow::RightOnly => "RIGHT_ONLY",
        TextFlow::LargestOnly => "LARGEST_ONLY",
    }
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
    use crate::model::image::{
        EffectColor, EffectPoint, EffectRgb, ImageAttr, Picture, PictureEffects, PictureShadow,
    };
    use crate::serializer::hwpx::context::SerializeContext;

    fn make_picture(bin_data_id: u16) -> Picture {
        let mut pic = Picture::default();
        pic.image_attr = ImageAttr {
            bin_data_id,
            brightness: 0,
            contrast: 0,
            effect: ImageEffect::RealPic,
            transparency: 0,
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

    fn serialize(pic: &Picture, ctx: &mut SerializeContext) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_picture(&mut w, pic, ctx).expect("write_picture");
        String::from_utf8(w.into_inner()).unwrap()
    }

    #[test]
    fn task1389_cur_sz_uses_shape_attr() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1); // common 1000x500
        pic.shape_attr.current_width = 1366;
        pic.shape_attr.current_height = 1268;
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(r#"<hp:curSz width="1366" height="1268"/>"#),
            "curSz 는 shape_attr.current 사용(sz 아님): {xml}"
        );
    }

    #[test]
    fn task1389_cur_sz_falls_back_to_common_when_zero() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1); // current_width=0 → common 폴백
        let xml = serialize(&pic, &mut ctx);
        assert!(xml.contains(r#"<hp:curSz width="1000" height="500"/>"#));
    }

    #[test]
    fn task1389_img_rect_uses_border_scalar_layout() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1);
        // ta-pic pic0 실측: pt0(0,0) pt1(49380,0) pt2(49380,45840) pt3(0,45840)
        // 스칼라 레이아웃: border_x=[pt0.x,pt0.y,pt1.x,pt1.y], border_y=[pt2.x,pt2.y,pt3.x,pt3.y]
        pic.border_x = [0, 0, 49380, 0];
        pic.border_y = [49380, 45840, 0, 45840];
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(
                r#"<hp:imgRect><hc:pt0 x="0" y="0"/><hc:pt1 x="49380" y="0"/><hc:pt2 x="49380" y="45840"/><hc:pt3 x="0" y="45840"/></hp:imgRect>"#
            ),
            "imgRect 는 border 스칼라 레이아웃 역매핑: {xml}"
        );
    }

    #[test]
    fn issue1452_img_alpha_uses_hwp_alpha_byte() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1);

        pic.image_attr.transparency = 50;
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(r#"alpha="127""#),
            "그림 투명도 50%는 한컴 HWPX alpha byte 127로 저장되어야 한다: {xml}"
        );

        pic.image_attr.transparency = 100;
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(r#"alpha="255""#),
            "그림 투명도 100%는 한컴 HWPX alpha byte 255로 저장되어야 한다: {xml}"
        );
    }

    #[test]
    fn task1389_img_rect_synthesizes_when_border_zero() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1); // border 전부 0 → common 합성
        let xml = serialize(&pic, &mut ctx);
        assert!(xml.contains(r#"<hc:pt2 x="1000" y="500"/>"#), "{xml}");
    }

    #[test]
    fn task1389_img_dim_verbatim() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1);
        pic.img_dim = (49380, 45840);
        // crop 이 달라도 img_dim verbatim (clip 파생 아님)
        pic.crop.right = 99999;
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(r#"<hp:imgDim dimwidth="49380" dimheight="45840"/>"#),
            "imgDim 은 IR verbatim (clip 파생 금지): {xml}"
        );
    }

    #[test]
    fn task1392_pic_shape_comment_emitted_after_caption() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1);
        pic.common.description = "그림입니다.".to_string();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "캡션".to_string();
        let mut cap = crate::model::shape::Caption::default();
        cap.paragraphs.push(para);
        pic.caption = Some(cap);
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains("<hp:shapeComment>그림입니다.</hp:shapeComment>"),
            "shapeComment 방출: {xml}"
        );
        // 순서: caption → shapeComment (aift 실물)
        let cp = xml.find("<hp:caption").unwrap();
        let sc = xml.find("<hp:shapeComment").unwrap();
        assert!(cp < sc, "caption 이 shapeComment 보다 먼저");
    }

    #[test]
    fn task1392_pic_no_description_omits_comment() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1); // description 빈 문자열
        let xml = serialize(&pic, &mut ctx);
        assert!(!xml.contains("<hp:shapeComment"), "빈 설명은 미방출: {xml}");
    }

    #[test]
    fn pic_root_attrs_in_canonical_order() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1);
        let xml = serialize(&pic, &mut ctx);
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
        // [#1891] manifest id 는 `image{bin_data_id}` 숫자 불변식을 따라야 한다.
        // 파서(section.rs)가 binaryItemIDRef 의 숫자를 그대로 bin_data_id 로 읽으므로,
        // 종전 순번 명명(id=5 → "image1")은 재파스에서 참조가 1 로 어긋났다.
        let doc = make_doc_with_bin(5, "jpg");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(5);
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(r#"binaryItemIDRef="image5""#),
            "binaryItemIDRef must resolve to manifest id image5 (숫자 불변식): {}",
            xml
        );
    }

    #[test]
    fn task1567_empty_binary_ref_pic_preserved() {
        // #1567: bin_data_id==0(원본 binaryItemIDRef="" placeholder)은 BinDataContent 가
        // 없어 resolve 실패해도 binaryItemIDRef="" 로 보존되어야 한다(드롭 금지).
        let doc = Document::default(); // bin 미등록
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(0);
        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(r#"binaryItemIDRef="""#),
            "빈 ref pic 은 binaryItemIDRef=\"\" 로 보존(드롭 금지): {xml}"
        );
        assert!(xml.contains("<hp:pic "), "pic 컨트롤 자체 보존: {xml}");
    }

    #[test]
    fn shape_component_attrs_are_serialized() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1);
        pic.shape_attr.original_width = 23456;
        pic.shape_attr.original_height = 12345;
        pic.shape_attr.horz_flip = true;
        pic.shape_attr.vert_flip = true;
        pic.shape_attr.rotation_angle = 34;
        pic.shape_attr.rotation_center.x = 11700;
        pic.shape_attr.rotation_center.y = 14794;
        pic.shape_attr.rotate_image = true;

        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(r#"<hp:orgSz width="23456" height="12345"/>"#),
            "orgSz must use ShapeComponentAttr values: {}",
            xml
        );
        assert!(
            xml.contains(r#"<hp:flip horizontal="1" vertical="1"/>"#),
            "flip must use ShapeComponentAttr values: {}",
            xml
        );
        assert!(
            xml.contains(
                r#"<hp:rotationInfo angle="34" centerX="11700" centerY="14794" rotateimage="1"/>"#
            ),
            "rotationInfo must use ShapeComponentAttr values: {}",
            xml
        );
    }

    #[test]
    fn unresolved_bin_data_id_errors() {
        let doc = Document::default(); // bin_data 없음
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(99); // 미등록 id
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        let err = write_picture(&mut w, &pic, &mut ctx).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("binaryItemIDRef"), "error msg: {}", msg);
        assert!(
            msg.contains("99"),
            "error should include bin_data_id: {}",
            msg
        );
    }

    #[test]
    fn rendering_info_has_three_matrices() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1);
        let xml = serialize(&pic, &mut ctx);
        assert!(xml.contains("<hc:transMatrix "));
        assert!(xml.contains("<hc:scaMatrix "));
        assert!(xml.contains("<hc:rotMatrix "));
    }

    #[test]
    fn img_rect_has_four_points() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1);
        let xml = serialize(&pic, &mut ctx);
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

    #[test]
    fn picture_effects_shadow_are_serialized() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1);
        pic.effects = PictureEffects {
            shadow: Some(PictureShadow {
                style: Some("OUTSIDE".to_string()),
                alpha: Some("0.8".to_string()),
                radius: Some("400".to_string()),
                direction: Some("45".to_string()),
                distance: Some("1000".to_string()),
                align_style: Some("TOP_LEFT".to_string()),
                rotation_style: Some("0".to_string()),
                skew: None,
                scale: Some(EffectPoint {
                    x: Some("1".to_string()),
                    y: Some("1".to_string()),
                }),
                color: Some(EffectColor {
                    color_type: Some("RGB".to_string()),
                    scheme_idx: Some("-1".to_string()),
                    system_idx: Some("-1".to_string()),
                    preset_idx: Some("-1".to_string()),
                    rgb: Some(EffectRgb {
                        r: Some("0".to_string()),
                        g: Some("0".to_string()),
                        b: Some("0".to_string()),
                    }),
                }),
            }),
        };

        let xml = serialize(&pic, &mut ctx);
        assert!(xml.contains(r#"<hp:shadow style="OUTSIDE" alpha="0.8" radius="400" direction="45" distance="1000" alignStyle="TOP_LEFT" rotationStyle="0">"#));
        assert!(xml.contains(r#"<hp:scale x="1" y="1"/>"#));
        assert!(xml.contains(
            r#"<hp:effectsColor type="RGB" schemeIdx="-1" systemIdx="-1" presetIdx="-1">"#
        ));
        assert!(xml.contains(r#"<hp:rgb r="0" g="0" b="0"/>"#));
    }

    // ---------- #1403: 그림 캡션 직렬화 ----------

    #[test]
    fn task1403_pic_caption_is_serialized() {
        // 캡션 유실(#1403) 회귀 고정 — 한컴 실물(aift.hwpx) 순서: outMargin 뒤 마지막 자식.
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut pic = make_picture(1);
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "그림 캡션".to_string();
        let mut cap = crate::model::shape::Caption::default();
        cap.width = 8504;
        cap.spacing = 850;
        cap.max_width = 48081;
        cap.paragraphs.push(para);
        pic.caption = Some(cap);

        let xml = serialize(&pic, &mut ctx);
        assert!(
            xml.contains(
                r#"<hp:caption side="BOTTOM" fullSz="0" width="8504" gap="850" lastWidth="48081">"#
            ),
            "caption 속성이 IR 값으로 방출되어야 함: {}",
            xml
        );
        assert!(
            xml.contains("<hp:t>그림 캡션</hp:t>"),
            "캡션 subList 문단 텍스트가 방출되어야 함: {}",
            xml
        );
        let om = xml.find("<hp:outMargin").unwrap();
        let cp = xml.find("<hp:caption").unwrap();
        assert!(om < cp, "caption 은 outMargin 뒤 (한컴 실물 순서)");
    }

    #[test]
    fn task1403_pic_without_caption_emits_none() {
        let doc = make_doc_with_bin(1, "png");
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let pic = make_picture(1);
        let xml = serialize(&pic, &mut ctx);
        assert!(!xml.contains("<hp:caption"), "캡션 부재 시 미방출: {}", xml);
    }
}
