use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use skia_safe::{FontMgr, FontStyle, Typeface};

pub(super) type SystemFontFamilies = HashSet<String>;

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
