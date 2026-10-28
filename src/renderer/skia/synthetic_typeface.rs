use skia_safe::{
    font_arguments::VariationPosition, Canvas, Data, Font, FontArguments, FontMgr, Paint, Point,
    Typeface,
};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static UNEMBOLDENED_TYPEFACES: RefCell<HashMap<u32, Option<Typeface>>> = RefCell::new(HashMap::new());
    static UNOBLIQUED_TYPEFACES: RefCell<HashMap<u32, Option<Typeface>>> = RefCell::new(HashMap::new());
}

/// 합성 굵기는 장치 격자에 맞춘 글자 mask 대신 원 프로그램의 윤곽선을 채우고 그린다.
/// 모든 path를 먼저 확인하므로 fallback 전에 일부 글자만 그리는 경우는 없다.
pub(super) fn draw_synthetic_bold_paths(
    canvas: &Canvas,
    text: &str,
    font: &Font,
    origin: (f32, f32),
    paint: &Paint,
    raster_target: bool,
) -> bool {
    let mut path_font = font.clone();
    path_font.set_hinting(skia_safe::FontHinting::None);
    path_font.set_baseline_snap(false);
    let glyphs = path_font.str_to_glyphs_vec(text);
    if glyphs.is_empty() {
        return false;
    }
    let Some(paths) = glyphs
        .iter()
        .map(|glyph| path_font.get_path(*glyph))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let mut positions = vec![Point::default(); glyphs.len()];
    path_font.get_pos(&glyphs, &mut positions, Some(origin.into()));
    let raster_paints = raster_target.then(|| {
        let mut fill = paint.clone();
        fill.set_style(skia_safe::paint::Style::Fill);
        let mut stroke = paint.clone();
        stroke.set_style(skia_safe::paint::Style::Stroke);
        // PDF의 얇은 획은 raster 장치의 한 단위보다 작게 소비하지 않는다.
        // 조판 좌표나 원래 em 획은 바꾸지 않고 현재 장치 변환만 사용한다.
        if let Some(inverse) = canvas.local_to_device_as_3x3().invert() {
            let x_unit = inverse.map_vector((1.0, 0.0)).length();
            let y_unit = inverse.map_vector((0.0, 1.0)).length();
            let device_unit = x_unit.min(y_unit);
            if device_unit.is_finite() && device_unit > 0.0 {
                stroke.set_stroke_width(stroke.stroke_width().max(device_unit));
            }
        }
        (fill, stroke)
    });
    for (path, position) in paths.iter().zip(positions) {
        canvas.save();
        canvas.translate((position.x, position.y));
        if let Some((fill, stroke)) = &raster_paints {
            canvas.draw_path(path, fill);
            canvas.draw_path(path, stroke);
        } else {
            // Vector 출력의 획은 출력 장치의 해상도가 정해질 때까지 em 단위로 보존한다.
            canvas.draw_path(path, paint);
        }
        canvas.restore();
    }
    true
}

/// 시스템의 합성 weight를 제거하되 같은 font program·collection·가변 축을 보존한다.
/// 실제 Bold 또는 판정 근거가 없는 프로그램은 변경하지 않는다.
pub(super) fn unemboldened_typeface(font_mgr: &FontMgr, typeface: &Typeface) -> Option<Typeface> {
    UNEMBOLDENED_TYPEFACES.with(|cache| {
        let id = typeface.unique_id();
        if let Some(cached) = cache.borrow().get(&id) {
            return cached.clone();
        }
        let result = regular_program_typeface(font_mgr, typeface);
        cache.borrow_mut().insert(id, result.clone());
        result
    })
}

fn regular_program_typeface(font_mgr: &FontMgr, typeface: &Typeface) -> Option<Typeface> {
    let coordinates = typeface.variation_design_position().unwrap_or_default();
    let weight = coordinates
        .iter()
        .find(|coordinate| coordinate.axis == u32::from_be_bytes(*b"wght").into());
    if weight.is_some_and(|coordinate| coordinate.value >= 600.0) {
        return None;
    }
    if *typeface.font_style().weight() < 600 {
        return Some(typeface.clone());
    }
    let mut os2 = [0; 6];
    if typeface.get_table_data(u32::from_be_bytes(*b"OS/2"), &mut os2) != os2.len() {
        return None;
    }
    let program_weight = weight.map_or(
        f32::from(u16::from_be_bytes([os2[4], os2[5]])),
        |coordinate| coordinate.value,
    );
    if !(1.0..600.0).contains(&program_weight) {
        return None;
    }
    // fontconfig의 embolden은 Font::set_embolden(false)로 해제되지 않는다.
    // SFNT 자체를 다시 열어 OS가 덧붙인 가상 weight만 제외한다.
    let (bytes, index) = typeface.to_font_bytes()?;
    let mut regular = font_mgr.new_from_data(Data::new_copy(&bytes), index)?;
    if !coordinates.is_empty() {
        let mut arguments = FontArguments::new().set_variation_design_position(VariationPosition {
            coordinates: &coordinates,
        });
        arguments.set_collection_index(index as usize);
        regular = regular.clone_with_arguments(&arguments)?;
    }
    // 시스템이 붙인 기울임은 SFNT에 남지 않는다. 독립적인 각도 근거 없이
    // 다른 기울임으로 바꾸지 않고 기존 paint를 유지한다.
    if regular.font_style().slant() != typeface.font_style().slant() {
        return None;
    }
    Some(regular)
}

/// fontconfig가 일반 프로그램에 붙인 가상 slant를 paint의 합성 shear와 구분한다.
/// 실제 italic/oblique 프로그램과 활성 slnt/ital 축은 그대로 사용한다.
pub(super) fn unobliqued_typeface(font_mgr: &FontMgr, typeface: &Typeface) -> Option<Typeface> {
    if typeface.font_style().slant() == skia_safe::font_style::Slant::Upright {
        return None;
    }
    UNOBLIQUED_TYPEFACES.with(|cache| {
        let id = typeface.unique_id();
        if let Some(cached) = cache.borrow().get(&id) {
            return cached.clone();
        }
        let result = physical_upright_typeface(font_mgr, typeface);
        cache.borrow_mut().insert(id, result.clone());
        result
    })
}

fn physical_upright_typeface(font_mgr: &FontMgr, typeface: &Typeface) -> Option<Typeface> {
    let coordinates = typeface.variation_design_position().unwrap_or_default();
    if coordinates.iter().any(|coordinate| {
        (coordinate.axis == u32::from_be_bytes(*b"ital").into()
            || coordinate.axis == u32::from_be_bytes(*b"slnt").into())
            && coordinate.value != 0.0
    }) {
        return None;
    }
    let (bytes, index) = typeface.to_font_bytes()?;
    let parsed = ttf_parser::Face::parse(&bytes, u32::try_from(index).ok()?).ok()?;
    if parsed.is_italic() || parsed.is_oblique() || parsed.italic_angle() != 0.0 {
        return None;
    }
    let mut physical = font_mgr.new_from_data(Data::new_copy(&bytes), index)?;
    if !coordinates.is_empty() {
        let mut arguments = FontArguments::new().set_variation_design_position(VariationPosition {
            coordinates: &coordinates,
        });
        arguments.set_collection_index(index as usize);
        physical = physical.clone_with_arguments(&arguments)?;
    }
    (physical.font_style().slant() == skia_safe::font_style::Slant::Upright).then_some(physical)
}
