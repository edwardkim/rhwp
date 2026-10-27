//! 선택한 실제 face와 본문 스타일로 동일한 글자 폰트를 만든다.
use super::super::font_lookup::{
    select_typeface_for_character, supplied_family_names, TypefaceCandidate,
};
use super::super::synthetic_typeface::{unemboldened_typeface, unobliqued_typeface};
use crate::renderer::{internal_font_family_members, TextStyle};
use skia_safe::{font, Canvas, Font, FontMgr, Paint, Typeface};
use std::{cell::RefCell, collections::HashMap};

pub(in crate::renderer::skia) fn compatibility_glyph(font: &Font, text: &str) -> Option<char> {
    crate::renderer::pua_glyph::candidate(text)
        .into_iter()
        .chain(crate::renderer::pua_glyph::fallback_candidate(text))
        .find(|ch| {
            let glyph = font.typeface().unichar_to_glyph(*ch as i32);
            glyph != 0 && font.get_path(glyph).is_some_and(|path| !path.is_empty())
        })
}

thread_local! {
    static OWN_HEAVY_FACES: RefCell<HashMap<(u32, String), bool>> = RefCell::new(HashMap::new());
}

fn owns_intrinsic_heavy_face(typeface: &Typeface, style: &TextStyle) -> bool {
    if style.bold {
        return false;
    }
    let members = internal_font_family_members(&style.font_family);
    let Some(&requested) = members.first() else {
        return false;
    };
    if !crate::renderer::style_resolver::is_heavy_display_face(requested) {
        return false;
    }
    let key = (typeface.unique_id(), requested.to_owned());
    if let Some(owned) = OWN_HEAVY_FACES.with(|cache| cache.borrow().get(&key).copied()) {
        return owned;
    }
    let primary = typeface.family_name();
    // 대체명 목록에는 fallback도 섞인다. 이름 사다리만으로 원 face라고
    // 판단하지 않고 실제 선택한 프로그램의 family 레코드를 확인한다.
    let owned = primary.eq_ignore_ascii_case(requested)
        || typeface
            .to_font_bytes()
            .and_then(|(bytes, index)| {
                u32::try_from(index).ok().map(|index| {
                    supplied_family_names(&bytes, index, &primary)
                        .iter()
                        .any(|family| family.eq_ignore_ascii_case(requested))
                })
            })
            .unwrap_or(false);
    OWN_HEAVY_FACES.with(|cache| cache.borrow_mut().insert(key, owned));
    owned
}

pub(super) fn for_text(
    chain: &[TypefaceCandidate],
    sample: &str,
    size: f32,
    style: &TextStyle,
    font_mgr: &FontMgr,
    geometric_text: bool,
) -> Option<Font> {
    if let Some(ch) = sample
        .chars()
        .find(|ch| !crate::renderer::text_glyph_visibility::is_nonprinting_character(*ch))
    {
        return select_typeface_for_character(chain, ch).map(|candidate| {
            finish(
                candidate.typeface.clone(),
                size,
                style,
                font_mgr,
                geometric_text,
            )
        });
    }
    if let Some(candidate) = chain.first() {
        Some(finish(
            candidate.typeface.clone(),
            size,
            style,
            font_mgr,
            geometric_text,
        ))
    } else {
        let mut font = Font::default();
        font.set_size(size);
        font.set_edging(font::Edging::AntiAlias);
        font.set_subpixel(true);
        preserve_geometry(&mut font, geometric_text);
        Some(font)
    }
}

pub(super) fn preserve_geometry(font: &mut Font, geometric_text: bool) {
    if geometric_text {
        font.set_subpixel(true);
        font.set_hinting(skia_safe::FontHinting::None);
        font.set_baseline_snap(false);
    }
}

fn finish(
    tf: Typeface,
    size: f32,
    style: &TextStyle,
    font_mgr: &FontMgr,
    geometric_text: bool,
) -> Font {
    let tf = if style.italic {
        unobliqued_typeface(font_mgr, &tf).unwrap_or(tf)
    } else {
        tf
    };
    // nominal과 SkShaper가 같은 일반 텍스트용 기본 윤곽 프로그램을 소비한다.
    let tf = super::monochrome::text_typeface(font_mgr, &tf).unwrap_or(tf);
    let owns_heavy = owns_intrinsic_heavy_face(&tf, style);
    let tf = if owns_heavy {
        // source의 bold가 아닌 fallback 보완 weight다. fontconfig가 붙인
        // 가상 weight도 원 프로그램으로 되돌리며 실제 bold·기울임은 보존한다.
        unemboldened_typeface(font_mgr, &tf).unwrap_or(tf)
    } else {
        tf
    };
    let is_bold_face = *tf.font_style().weight() >= 600;
    let is_slanted_face = tf.font_style().slant() != skia_safe::font_style::Slant::Upright;
    let mut font = Font::new(tf, size);
    font.set_edging(font::Edging::AntiAlias);
    // 글줄이 정한 소수 원점을 glyph raster에서도 보존한다.
    font.set_subpixel(true);
    // 원본 폭과 기준선으로 배치한 생성 번호의 소수 원점을
    // 정수로 반올림하지 않는다. Canvas도 이 원점에서 그린다.
    if style.fixed_glyph_advances {
        font.set_baseline_snap(false);
    }
    preserve_geometry(&mut font, geometric_text);
    if style.is_visually_bold() && !is_bold_face && !owns_heavy {
        font.set_embolden(true);
    }
    // 한컴 Print의 직립 프로그램 합성은 x/y shear 1/3이다.
    // 실제 기울임 face에는 중복 shear를 적용하지 않는다.
    if style.italic && !is_slanted_face {
        font.set_skew_x(crate::renderer::text_transform::SYNTHETIC_ITALIC_SKEW as f32);
    }
    font
}

/// Print의 독립 glyph는 실제 선택한 Font의 윤곽을 기존 paint·원점에 재생한다.
/// 여러 scalar의 shaping과 path가 없는 glyph는 원래 텍스트 재생에 맡긴다.
pub(super) fn draw_positioned_cluster(
    canvas: &Canvas,
    text: &str,
    origin: (f32, f32),
    font: &Font,
    paint: &Paint,
    geometric_text: bool,
) {
    if geometric_text {
        let mut characters = text.chars();
        if let Some(character) = characters.next().filter(|_| characters.next().is_none()) {
            let glyph = font.unichar_to_glyph(character as i32);
            if let Some(path) = font.get_path(glyph) {
                canvas.save();
                canvas.translate(origin);
                canvas.draw_path(&path, paint);
                canvas.restore();
                return;
            }
        }
    }
    canvas.draw_str(text, origin, font, paint);
}
