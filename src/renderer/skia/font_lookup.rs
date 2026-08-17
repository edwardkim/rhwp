use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use skia_safe::{FontMgr, FontStyle, Typeface};

use crate::renderer::base_family_without_weight_suffix;

pub(super) type SystemFontFamilies = HashSet<String>;

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

pub(super) fn text_family_candidates(requested: &str) -> Vec<String> {
    let mut families = Vec::new();
    if !requested.trim().is_empty() {
        families.push(requested.to_string());
    }
    if let Some(base) = base_family_without_weight_suffix(requested) {
        if !families.iter().any(|family| family == &base) {
            families.push(base);
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
    custom_typefaces: &HashMap<String, Typeface>,
    bundled_typefaces: &HashMap<String, Typeface>,
    requested: &str,
    style: FontStyle,
) -> (Vec<String>, Vec<TypefaceCandidate>) {
    let families = text_family_candidates(requested);
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut push = |typeface: Typeface, source: &'static str| {
        let key = typeface.family_name();
        if seen.insert(key) {
            chain.push(TypefaceCandidate { typeface, source });
        }
    };
    for family in &families {
        if let Some(typeface) = custom_typefaces.get(family).cloned() {
            push(typeface, "custom");
        }
    }
    for family in &families {
        if let Some(typeface) = match_system_family_style(font_mgr, system_families, family, style)
        {
            push(typeface, "system");
        }
    }
    for family in &families {
        if let Some(typeface) = bundled_typefaces.get(family).cloned() {
            push(typeface, "bundled");
        }
    }
    if let Some(typeface) = legacy_typeface_for_style(font_mgr, style) {
        push(typeface, "legacy");
    }
    (families, chain)
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
