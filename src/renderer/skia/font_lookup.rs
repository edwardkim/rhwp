use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use skia_safe::{FontMgr, FontStyle, Typeface};

use super::font_catalog::TypefaceCatalog;
use crate::renderer::base_family_without_weight_suffix;

pub(super) type SystemFontFamilies = HashSet<String>;

pub(super) fn font_style_for_text(style: &crate::renderer::TextStyle) -> FontStyle {
    let weight = match style.css_font_weight() {
        Some("bold") => 700,
        Some("300") => 300,
        Some("500") => 500,
        _ => 400,
    };
    FontStyle::new(
        skia_safe::font_style::Weight::from(weight),
        skia_safe::font_style::Width::NORMAL,
        if style.italic {
            skia_safe::font_style::Slant::Italic
        } else {
            skia_safe::font_style::Slant::Upright
        },
    )
}

pub(super) fn match_supplied_family(
    families: &TypefaceCatalog,
    family: &str,
    requested: FontStyle,
) -> Option<Typeface> {
    families.candidates(family, requested).into_iter().next()
}

const TEXT_FALLBACK_FAMILIES: &[&str] = &[
    "Noto Sans KR",
    "Noto Serif KR",
    "Noto Sans CJK KR",
    "Noto Serif CJK KR",
    "Nanum Gothic",
    "Nanum Myeongjo",
    "Malgun Gothic",
    "맑은 고딕",
    "Batang",
    "바탕",
    "Apple SD Gothic Neo",
    "AppleMyungjo",
    "DejaVu Sans",
    "Arial",
    "sans-serif",
];

#[derive(Clone)]
pub(super) struct TypefaceCandidate {
    pub(super) typeface: Typeface,
    pub(super) source: &'static str,
}

/// A supplied face has one Skia primary name, but its SFNT naming table may
/// declare localized or typographic family names for that same program.
/// Match family records only: a full face or PostScript name is not a family.
pub(super) fn supplied_family_names(data: &[u8], face_index: u32, primary: &str) -> Vec<String> {
    let mut families = vec![primary.to_string()];
    // Match the same collection face that the bounded catalog constructed.
    if let Ok(face) = ttf_parser::Face::parse(data, face_index) {
        for name in face.names() {
            if !matches!(
                name.name_id,
                ttf_parser::name_id::FAMILY
                    | ttf_parser::name_id::TYPOGRAPHIC_FAMILY
                    | ttf_parser::name_id::WWS_FAMILY
            ) {
                continue;
            }
            if let Some(family) = name.to_string() {
                let family = family.trim();
                if !family.is_empty() && !families.iter().any(|name| name == family) {
                    families.push(family.to_string());
                }
            }
        }
    }
    families
}

pub(super) fn text_family_candidates(requested: &str) -> Vec<String> {
    let mut families = Vec::new();
    for requested in requested
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !families.iter().any(|family| family == requested) {
            families.push(requested.to_string());
        }
        if let Some(base) = base_family_without_weight_suffix(requested) {
            if !families.iter().any(|family| family == &base) {
                families.push(base);
            }
        }
    }
    for family in TEXT_FALLBACK_FAMILIES {
        if !families.iter().any(|candidate| candidate == family) {
            families.push((*family).to_string());
        }
    }
    families
}

pub(super) fn text_typeface_candidates(
    font_mgr: &FontMgr,
    system_families: &SystemFontFamilies,
    custom_typefaces: &TypefaceCatalog,
    bundled_typefaces: &TypefaceCatalog,
    requested: &str,
    style: FontStyle,
) -> (Vec<String>, Vec<TypefaceCandidate>) {
    let families = text_family_candidates(requested);
    let chain = typeface_candidates_for_families(
        font_mgr,
        system_families,
        custom_typefaces,
        bundled_typefaces,
        &families,
        style,
    );
    (families, chain)
}

/// Resolve an ordered family list through the same sources used by body text.
pub(super) fn typeface_candidates_for_families(
    font_mgr: &FontMgr,
    system_families: &SystemFontFamilies,
    custom_typefaces: &TypefaceCatalog,
    bundled_typefaces: &TypefaceCatalog,
    families: &[String],
    style: FontStyle,
) -> Vec<TypefaceCandidate> {
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut push = |typeface: Typeface, source: &'static str| {
        let key = typeface.unique_id();
        if seen.insert(key) {
            chain.push(TypefaceCandidate { typeface, source });
        }
    };
    for family in families {
        for typeface in custom_typefaces.candidates(family, style) {
            push(typeface, "custom");
        }
        // 원 face의 system 프로그램보다 뒤 face의 custom 프로그램을 먼저
        // 고르면 선언 순서를 뒤집는다. 공급 출처는 각 face 내부에서 우선한다.
        if let Some(typeface) = match_system_family_style(font_mgr, system_families, family, style)
        {
            push(typeface, "system");
        }
        for typeface in bundled_typefaces.candidates(family, style) {
            push(typeface, "bundled");
        }
    }
    if let Some(typeface) = legacy_typeface_for_style(font_mgr, style) {
        push(typeface, "legacy");
    }
    chain
}

pub(super) fn select_typeface_for_character<'a>(
    chain: &'a [TypefaceCandidate],
    character: char,
) -> Option<&'a TypefaceCandidate> {
    if character.is_whitespace() {
        return chain.first();
    }
    let codepoint = character as i32;
    chain
        .iter()
        .find(|candidate| candidate.typeface.unichar_to_glyph(codepoint) != 0)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontStyleCacheKey {
    weight: i32,
    width: i32,
    slant: i32,
}

impl FontStyleCacheKey {
    fn new(style: FontStyle) -> Self {
        Self {
            weight: *style.weight(),
            width: *style.width(),
            slant: style.slant() as i32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontLookupKey {
    family: String,
    style: FontStyleCacheKey,
}

thread_local! {
    static SYSTEM_TYPEFACE_CACHE: RefCell<HashMap<FontLookupKey, Option<Typeface>>> =
        RefCell::new(HashMap::new());
    static LEGACY_TYPEFACE_CACHE: RefCell<HashMap<FontStyleCacheKey, Option<Typeface>>> =
        RefCell::new(HashMap::new());
}

pub(super) fn collect_system_families(font_mgr: &FontMgr) -> SystemFontFamilies {
    font_mgr.family_names().collect()
}

pub(super) fn has_system_family(system_families: &SystemFontFamilies, family: &str) -> bool {
    system_families.contains(family)
}

pub(super) fn match_system_family_style(
    font_mgr: &FontMgr,
    system_families: &SystemFontFamilies,
    family: &str,
    style: FontStyle,
) -> Option<Typeface> {
    if !has_system_family(system_families, family) {
        return None;
    }

    let key = FontLookupKey {
        family: family.to_string(),
        style: FontStyleCacheKey::new(style),
    };
    SYSTEM_TYPEFACE_CACHE.with(|cache| {
        if let Some(cached) = { cache.borrow().get(&key).cloned() } {
            return cached;
        }

        let matched = font_mgr.match_family_style(family, style);
        cache.borrow_mut().insert(key, matched.clone());
        matched
    })
}

pub(super) fn legacy_typeface_for_style(font_mgr: &FontMgr, style: FontStyle) -> Option<Typeface> {
    let key = FontStyleCacheKey::new(style);
    LEGACY_TYPEFACE_CACHE.with(|cache| {
        if let Some(cached) = { cache.borrow().get(&key).cloned() } {
            return cached;
        }

        let matched = font_mgr.legacy_make_typeface(None::<&str>, style);
        cache.borrow_mut().insert(key, matched.clone());
        matched
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_family_is_filtered_before_system_lookup() {
        let font_mgr = FontMgr::default();
        let system_families = SystemFontFamilies::new();

        assert!(match_system_family_style(
            &font_mgr,
            &system_families,
            "Definitely Missing RHWP Test Font",
            FontStyle::normal(),
        )
        .is_none());
    }

    #[test]
    fn system_family_membership_uses_exact_family_name() {
        let mut system_families = SystemFontFamilies::new();
        system_families.insert("AppleGothic".to_string());

        assert!(has_system_family(&system_families, "AppleGothic"));
        assert!(!has_system_family(&system_families, "applegothic"));
        assert!(!has_system_family(&system_families, "Missing Family"));
    }
}
