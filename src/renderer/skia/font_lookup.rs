use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use skia_safe::{FontMgr, FontStyle, Typeface};

use super::font_catalog::TypefaceCatalog;
use crate::renderer::{
    base_family_without_weight_suffix, generic_fallback_for_weight, installed_render_font_aliases,
    internal_font_family_members,
};

#[derive(Clone, Default)]
pub(super) struct SystemFontFamilies {
    names: HashSet<String>,
    postscript_faces: Arc<OnceLock<HashMap<String, Typeface>>>,
}

impl SystemFontFamilies {
    fn postscript_face(&self, font_mgr: &FontMgr, requested: &str) -> Option<Typeface> {
        // OpenType name ID 6은 공백·비ASCII·PostScript 구분 문자를 허용하지 않는다.
        // 일반 family의 이름 수용과 구별하며 없는 한국어 family를 색인하지 않는다.
        if requested.is_empty()
            || !requested
                .bytes()
                .all(|byte| (33..=126).contains(&byte) && !b"[](){}<>/%".contains(&byte))
        {
            return None;
        }
        // family와 다른 식별자다. FULL_NAME이나 추측한 접미사를 색인하지 않는다.
        self.postscript_faces
            .get_or_init(|| {
                let mut faces = HashMap::new();
                for family in 0..font_mgr.count_families() {
                    let mut styles = font_mgr.new_style_set(family);
                    for style in 0..styles.count() {
                        if let Some(face) = styles.new_typeface(style) {
                            if let Some(name) = face.post_script_name() {
                                faces.entry(name).or_insert(face);
                            }
                        }
                    }
                }
                faces
            })
            .get(requested)
            .cloned()
    }
}

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
    // Read the same collection face that the bounded catalog constructed.
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
    text_family_candidates_for_weight(requested, false)
}

fn text_family_candidates_for_weight(requested: &str, bold: bool) -> Vec<String> {
    let mut families = declared_family_candidates(requested);
    let requested_members = internal_font_family_members(requested);
    // SVG/Canvas와 같은 서체 분류·굵기 계약을 소비한다. 원 바탕 face가
    // 없다는 이유로 고딕을 우선하거나 굵은 본문에 ExtraLight를 고르지 않는다.
    // 공통 정책의 정적 CSS 이름만 해제하며 사용자 요청 이름은 아래에서 보존한다.
    let fallback =
        generic_fallback_for_weight(requested_members.first().copied().unwrap_or(""), bold);
    for family in fallback
        .split(',')
        .map(|name| name.trim().trim_matches('\''))
    {
        if !families.iter().any(|candidate| candidate == family) {
            families.push(family.to_string());
        }
    }
    families
}

fn declared_family_candidates(requested: &str) -> Vec<String> {
    let mut families = Vec::new();
    // CSS 체인을 하나의 family 이름으로 조회하면 설치된 첫 face도 놓친다.
    // SVG/Canvas와 같은 원본 순서·설치 별칭·굵기 접미사 해소를 소비한다.
    let requested_members = internal_font_family_members(requested);
    for requested_family in &requested_members {
        let requested_family = *requested_family;
        if !families.iter().any(|family| family == requested_family) {
            families.push(requested_family.to_string());
        }
        for alias in installed_render_font_aliases(requested_family) {
            if !families.iter().any(|family| family == alias) {
                families.push((*alias).to_string());
            }
        }
        if let Some(base) = base_family_without_weight_suffix(requested_family) {
            if !families.iter().any(|family| family == &base) {
                families.push(base);
            }
        }
        // 고정 advance 여부와 무관하게 같은 원 face의 설치 별칭을 먼저 찾는다.
        // paint와 renderer snapshot 진단이 이 후보 순서를 함께 소비한다.
        for alias in crate::renderer::installed_render_font_aliases(requested) {
            if !families.iter().any(|family| family == alias) {
                families.push((*alias).to_string());
            }
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
    let families = text_family_candidates_for_weight(requested, *style.weight() >= 600);
    let chain = typeface_candidates_for_families(
        font_mgr,
        system_families,
        custom_typefaces,
        bundled_typefaces,
        &families,
        &declared_family_candidates(requested),
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
    postscript_requests: &[String],
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
        // 생성 fallback은 family 요청이다. 원본/설치 별칭의 명시 ID 6만
        // 별도 프로그램으로 해석하며 같은 family의 문자 fallback은 유지한다.
        if let Some(typeface) = (has_system_family(system_families, family)
            || postscript_requests.contains(family))
        .then(|| match_system_family_style(font_mgr, system_families, family, style))
        .flatten()
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
    SystemFontFamilies {
        names: font_mgr.family_names().collect(),
        ..Default::default()
    }
}

pub(super) fn has_system_family(system_families: &SystemFontFamilies, family: &str) -> bool {
    // Skia의 실제 family 조회는 ASCII 대소문자를 구분하지 않는다.
    // 설치 이름 HyhwpEQ를 요청 HYhwpEQ의 사전 필터에서 제외하면
    // 측정 face 대신 대체 글꼴로 표시하게 된다. 없는 이름은 계속 제외한다.
    system_families.names.contains(family)
        || system_families
            .names
            .iter()
            .any(|installed| installed.eq_ignore_ascii_case(family))
}

pub(super) fn match_system_family_style(
    font_mgr: &FontMgr,
    system_families: &SystemFontFamilies,
    family: &str,
    style: FontStyle,
) -> Option<Typeface> {
    if system_families.names.is_empty() {
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

        let matched = if has_system_family(system_families, family) {
            font_mgr.match_family_style(family, style)
        } else {
            // 설치된 style set의 name ID 6만 조회한다. 없는 이름을 OS에
            // 질의하면 headless CoreText의 다운로드 IPC를 유발할 수 있다.
            system_families.postscript_face(font_mgr, family)
        };
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
        let mut system_families = SystemFontFamilies::default();

        assert!(match_system_family_style(
            &font_mgr,
            &system_families,
            "Definitely Missing RHWP Test Font",
            FontStyle::normal(),
        )
        .is_none());
        system_families
            .names
            .insert("RHWP membership witness".into());
        for name in [
            "없는 가족",
            "Missing Family",
            "Missing/Program",
            "Font[Face]",
        ] {
            assert!(match_system_family_style(
                &font_mgr,
                &system_families,
                name,
                FontStyle::normal(),
            )
            .is_none());
        }
        // 사양상 ID 6일 수 없는 요청은 실제 시스템 색인을 시작하지 않는다.
        assert!(system_families.postscript_faces.get().is_none());
    }

    #[test]
    fn system_family_membership_matches_skia_family_lookup() {
        let mut system_families = SystemFontFamilies::default();
        system_families.names.insert("AppleGothic".to_string());

        assert!(has_system_family(&system_families, "AppleGothic"));
        assert!(has_system_family(&system_families, "applegothic"));
        assert!(has_system_family(&system_families, "APPLEGOTHIC"));
        assert!(!has_system_family(&system_families, "AppleGothic Extra"));
        assert!(!has_system_family(&system_families, "Missing Family"));
    }
}
