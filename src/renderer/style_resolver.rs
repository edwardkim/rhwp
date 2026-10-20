//! 스타일 해소 (Style Resolution)
//!
//! DocInfo 참조 테이블을 렌더링에서 바로 사용할 수 있는
//! 해소된 스타일 목록(ResolvedStyleSet)으로 변환한다.

use super::{hwpunit_to_px, GradientFillInfo, PatternFillInfo, TabStop};
use crate::model::document::{DocInfo, Document};
use crate::model::image::ImageEffect;
use crate::model::style::{
    Alignment, BorderFill, BorderLine, Bullet, CenterLine, CharShape, DiagonalLine, FillType,
    HeadType, ImageFillMode, LineSpacingType, Numbering, ParaShape, TabDef, UnderlineType,
};
use crate::model::ColorRef;
use crate::renderer::font_rule_layout_name_projection::{
    find_font_rule_layout_name, GeneratedFontRuleProjection,
};

/// HWP 언어 카테고리 수 (한국어, 영어, 한자, 일본어, 기타, 기호, 사용자)
pub const LANG_COUNT: usize = 7;

mod ascii_punctuation;
mod page_number;
mod resolved;
pub use resolved::{
    ResolvedBorderStyle, ResolvedCharStyle, ResolvedImageFill, ResolvedParaStyle, ResolvedStyleSet,
};

/// DocInfo 참조 테이블을 해소된 스타일 목록으로 변환한다.
pub fn resolve_styles(doc_info: &DocInfo, dpi: f64) -> ResolvedStyleSet {
    resolve_styles_with_variant(doc_info, dpi, false)
}

/// Resolve styles with the document's format-specific style normalization.
/// Layout provenance itself remains owned by `LayoutCompatibilityProfile` and
/// is passed separately to consumers that need it.
pub(crate) fn resolve_styles_for_document(document: &Document, dpi: f64) -> ResolvedStyleSet {
    let profile = document.layout_profile();
    let mut styles = resolve_styles_with_variant(&document.doc_info, dpi, profile.hwp3_layout());
    super::legacy_font_advances::select_native_programs(document, &mut styles);
    // [#7051] 계보 신호가 없는 저장본의 HFT ASCII 반각 판정은 로드 시 한 번 내려 출처에 둔다.
    styles.hft_ascii_halfwidth |= document.provenance.hft_ascii_halfwidth_witnessed;
    if profile.hwpx_stored_layout() {
        // 같은 14pt 한양신명조라도 일반 본문과 표 안의 공백 조판은 다르다.
        // 검증 HWPX의 일반 본문은 반각, 표 안은 기존 저장 메트릭을 쓴다.
        // 같은 글자 모양이 양쪽에 쓰이면 전역 스타일 보정으로 구분할 수
        // 없으므로 원래 측정을 유지한다.
        let mut body_space_styles = std::collections::HashSet::new();
        let mut table_space_styles = std::collections::HashSet::new();
        for section in &document.sections {
            for para in &section.paragraphs {
                if !para
                    .controls
                    .iter()
                    .any(|control| matches!(control, crate::model::control::Control::Table(_)))
                {
                    collect_paragraph_space_styles(para, &mut body_space_styles);
                }
                for control in &para.controls {
                    if let crate::model::control::Control::Table(table) = control {
                        collect_table_space_styles(table, &mut table_space_styles);
                    }
                }
            }
        }
        for (id, style) in styles.char_styles.iter_mut().enumerate() {
            if body_space_styles.contains(&(id as u32))
                && !table_space_styles.contains(&(id as u32))
                && style.font_size >= 56.0 / 3.0 - 0.01
                && style.font_family.split(',').next() == Some("한양신명조")
                && style.font_space_em.is_none()
            {
                style.font_space_em = Some(0.5);
            }
        }
    }
    styles
}

fn collect_paragraph_space_styles(
    para: &crate::model::paragraph::Paragraph,
    ids: &mut std::collections::HashSet<u32>,
) {
    for (index, ch) in para.text.chars().enumerate() {
        if ch == ' ' {
            if let Some(id) = para.char_shape_id_at(index) {
                ids.insert(id);
            }
        }
    }
}

fn collect_table_space_styles(
    table: &crate::model::table::Table,
    ids: &mut std::collections::HashSet<u32>,
) {
    for cell in &table.cells {
        for para in &cell.paragraphs {
            collect_paragraph_space_styles(para, ids);
            for control in &para.controls {
                if let crate::model::control::Control::Table(nested) = control {
                    collect_table_space_styles(nested, ids);
                }
            }
        }
    }
}

/// The environment selects the same final face for measurement and every painter.
pub(crate) fn resolve_styles_with_environment(
    document: &Document,
    dpi: f64,
    environment: Option<&super::font_environment::FontEnvironment>,
    profile: crate::model::provenance::LayoutCompatibilityProfile,
) -> ResolvedStyleSet {
    let mut styles = resolve_styles_for_document(document, dpi);
    (
        styles.page_number_char_style_id,
        styles.page_number_font_is_hft,
    ) = page_number::resolve(&document.doc_info, profile.hangul2024_layout());
    if environment.is_some() {
        for (style, shape) in styles
            .char_styles
            .iter_mut()
            .zip(&document.doc_info.char_shapes)
        {
            for lang in 0..LANG_COUNT {
                let decision = lookup_font_name_in_environment(
                    &document.doc_info,
                    lang,
                    shape.font_ids[lang],
                    environment,
                );
                if decision.environment_profile_id.is_some() {
                    style.font_families[lang] = decision.css_family_chain.join(",");
                    // 명시적 프로그램 선택과 독립적으로 검증한 글리프 표가 모두 있어야
                    // TrueType 폭을 사용한다. 선언만으로 임의 face의 표를 신뢰하지 않는다.
                    let target = primary_font_name(&style.font_families[lang]);
                    let explicit_true_type =
                        environment.is_some_and(|env| env.selects_true_type(target));
                    let explicit_replacement = environment.is_some_and(|env| {
                        decision
                            .requested_face
                            .as_deref()
                            .is_some_and(|face| env.replacement(face).is_some())
                    });
                    // 결측 글리프 체인만 추가한 세션은 원 face의 TTF/HFT 신뢰를 바꾸지 않는다.
                    if explicit_true_type || explicit_replacement {
                        style.font_families_metric_trusted[lang] = explicit_true_type
                            && (metric_widths_verified_face(target)
                                || matches!(target, "휴먼명조" | "HumanMyeongJo"));
                    }
                    if explicit_true_type {
                        style.font_families_hft_hangul[lang] = false;
                        style.font_families_hft_fullwidth_dot[lang] = false;
                        style.font_families_metric_face[lang] = None;
                    }
                }
            }
            style.font_family = style.font_families[0].clone();
        }
    }
    styles
}

/// [Task #1001] HWP3 → HWP5 변환본 인지하여 ParaShape spacing/margin 추가 보정.
/// 변환본의 ParaShape 단위는 일반 HWP5 의 2배 (HwpUnitChar / HWPUNIT 의 2배 스케일)
/// 이므로 추가 1/2 보정 적용. 호출자가 Document::is_hwp3_variant 를 전달.
pub fn resolve_styles_with_variant(
    doc_info: &DocInfo,
    dpi: f64,
    is_hwp3_variant: bool,
) -> ResolvedStyleSet {
    let char_styles = resolve_char_styles(doc_info, dpi);
    let para_styles = resolve_para_styles_with_variant(doc_info, dpi, is_hwp3_variant);
    let border_styles = resolve_border_styles(doc_info, dpi);
    let numberings = doc_info.numberings.clone();
    let bullets = doc_info.bullets.clone();
    let (page_number_char_style_id, page_number_font_is_hft) =
        page_number::resolve(doc_info, false);

    ResolvedStyleSet {
        resolved_dpi: Some(dpi),
        page_number_char_style_id,
        page_number_font_is_hft,
        char_styles,
        para_styles,
        border_styles,
        numberings,
        bullets,
        hwp3_variant: is_hwp3_variant,
        hft_ascii_halfwidth: is_hwp3_variant,
        kerning_measurement_context: None,
        horizontal_shaping_context: None,
        supplemental_metrics: None,
    }
}

/// [#7391] 이 face 이름으로 **자기 자신의** 메트릭 표를 찾을 수 있는지.
///
/// 별칭(`layout-metric` 평면)을 타고 남의 표를 빌려 오는 경우는 거짓이다 — 그건
/// 치환 대상의 표와 다를 바 없어서 되돌릴 근거가 못 된다.
/// [#7702] rhwp 의 폭 표가 한/글 **HFT 글리프**를 잰 값인 face.
///
/// 한/글은 이 face 를 HFT 로 선언한 문서(`alt_type == 2`)에서만 자기 HFT 글리프로 그린다.
/// TTF 로 선언하면(`alt_type == 1`) 그런 TrueType 글꼴이 없어 문서가 적은 대체 글꼴로 그린다.
///
/// ```text
///   HCI Poppy  alt_type=2  k-water-rfp 등 50여 문서   정본 `·` = Type3 0.38 em
///              alt_type=1  76076·80168 (대체 Palatino Linotype)   정본 `·` = Palatino 0.25 em
/// ```
fn hft_program_metric_face(face: &&str) -> bool {
    *face == "HCI Poppy"
}

fn has_own_metric_table(face: &&str, bold: bool, italic: bool) -> bool {
    if *face == "HCI Tulip" {
        return true;
    }
    crate::renderer::font_metrics_data::find_metric_decision(face, bold, italic).is_some_and(
        |decision| {
            decision.alias_rule_id.is_none()
                && decision.metric.bold == bold
                && decision.metric.italic == italic
        },
    )
}

/// [#7092] 한/글이 선언 face 대신 **문서가 적은 대체 TTF** 로 그리는 경우, 그 대체 글꼴.
///
/// 한양 계열 이름(`한양중고딕`·`한양신명조`)을 TTF 로 선언한 문서는 같은 이름의 TTF 가 없어
/// 한/글이 대체 글꼴 자리를 본다. 대체가 **TTF `HY중고딕`·`HY신명조`** 면 그 TrueType 으로
/// 그리고(정본 PDF 의 `H2gtrM`·`H2mjsM` 자원, `·` 1.0em), 대체가 HFT(같은 이름) 이거나
/// 없으면 자기 HFT 글리프로 그린다(Type3 자원, `·` 0.38em). 저장소 정본 전수(한양중고딕
/// 선언 TTF 문서)에서 대체 `HY중고딕` 5문서는 전부 `H2gtrM`, 대체 HFT `한양중고딕`
/// 9개 출력은 전부 Type3 이고 반례가 없다 — 같은 문서를 2020·2024 엔진으로 뽑아도 같다.
///
/// ```text
///   78494 · issue1891_external_bindata_link · 75544 · 80550 · regulatory_impact_nested_table_escape
///       한양중고딕 TTF, 대체 TTF HY중고딕      → 정본 H2gtrM  · 1.000~1.010em (n=61·14·26·5·5)
///   80168(2020·2022·2024 출력) · 86712
///       한양중고딕 TTF, 대체 HFT 한양중고딕     → 정본 Type3   · 0.381~0.386em
/// ```
///
/// 두 형식 모두 `subst_font`(HWP5 `FACE_NAME` 의 대체 글꼴·유형, HWPX `<hh:substFont>`)를
/// 보고, 대체가 HFT·임베드가 아니면서 **대체 face 이름이 글꼴 파일로 검증한 TTF 표**
/// ([`metric_widths_verified_face`])인지로 가른다. 같은 이름의 HFT 대체(`한양중고딕`→
/// `한양중고딕`)와 검증되지 않은 대체(`한컴바탕`)는 여기 걸리지 않아 종전 HFT 실측 표
/// (`font_metrics_overlays`)를 그대로 쓴다. 결과는 측정 face(`font_families_metric_face`)와
/// 신뢰 여부에만 쓰고 표시 체인은 바꾸지 않는다.
fn verified_ttf_substitute(font: &crate::model::style::Font) -> Option<&str> {
    if font.alt_type != 1 {
        return None;
    }
    let substitute = match (&font.subst_font, &font.alt_name) {
        (Some(subst), _) => (subst.font_type != 2 && !subst.is_embedded).then(|| subst.face.trim()),
        (None, Some(alt_name)) => Some(alt_name.trim()),
        (None, None) => None,
    }?;
    (!substitute.is_empty()
        && substitute != font.name.trim()
        && metric_widths_verified_face(substitute))
    .then_some(substitute)
}

/// [#7387] CSS 체인의 첫 face 가 **선언한** 공백 글리프 전진폭(em).
///
/// 공백을 반각으로 눌러 두는 [`measure_char_width_embedded_decision_for_font`] 의
/// `c == ' '` 갈래를 우회해, 글꼴 표에 적힌 U+0020 의 값을 그대로 읽는다.
/// `use_font_space` 가 켜진 run 에서만 쓴다.
///
/// [`measure_char_width_embedded_decision_for_font`]: crate::renderer::layout
pub(crate) fn declared_space_advance_em(
    css_family_chain: &str,
    bold: bool,
    italic: bool,
) -> Option<f64> {
    let primary = css_family_chain
        .split(',')
        .next()?
        .trim()
        .trim_matches('\'')
        .trim_matches('"');
    // KoPub 은 메트릭 DB 가 아니라 전용 표가 폭을 소유한다 — 같은 출처에서 공백을 읽는다.
    if let Some(em) = crate::renderer::layout::kopub_declared_space_em(primary) {
        return Some(em);
    }
    let decision = crate::renderer::font_metrics_data::find_metric_decision(primary, bold, italic)?;
    let em = decision.metric.em_size;
    if em == 0 {
        return None;
    }
    let width = decision.metric.get_width(' ')?;
    if width == 0 {
        return None;
    }
    Some(f64::from(width) / f64::from(em))
}

/// CharShape + FontFace → ResolvedCharStyle 목록
fn resolve_char_styles(doc_info: &DocInfo, dpi: f64) -> Vec<ResolvedCharStyle> {
    doc_info
        .char_shapes
        .iter()
        .map(|cs| resolve_single_char_style(cs, doc_info, dpi))
        .collect()
}

/// 개별 CharShape 해소
fn resolve_single_char_style(cs: &CharShape, doc_info: &DocInfo, dpi: f64) -> ResolvedCharStyle {
    // base_size는 HWPUNIT 단위
    let font_size = hwpunit_to_px(cs.base_size, dpi);

    // 7개 언어 카테고리별 폰트 이름, 자간, 장평 해소
    let mut font_families = Vec::with_capacity(LANG_COUNT);
    let mut font_families_metric_trusted = Vec::with_capacity(LANG_COUNT);
    let mut font_families_hft_hangul = Vec::with_capacity(LANG_COUNT);
    let mut font_families_source_hft = Vec::with_capacity(LANG_COUNT);
    let mut font_families_hft_fullwidth_dot = Vec::with_capacity(LANG_COUNT);
    let mut font_families_metric_face: Vec<Option<String>> = Vec::with_capacity(LANG_COUNT);
    let mut letter_spacings = Vec::with_capacity(LANG_COUNT);
    let mut ratios = Vec::with_capacity(LANG_COUNT);
    let mut font_sizes = Vec::with_capacity(LANG_COUNT);
    let mut latin_slot_legacy = false;
    let mut latin_declared_hft = false;

    for lang in 0..LANG_COUNT {
        let font_id = cs.font_ids[lang];
        font_families_source_hft.push(
            doc_info
                .font_faces
                .get(lang)
                .and_then(|faces| faces.get(font_id as usize))
                .is_some_and(|face| face.alt_type == 2),
        );
        let decision = lookup_font_name_decision(doc_info, lang, font_id);
        if lang == 1 {
            latin_slot_legacy =
                decision.substitution_boundary == Some(FontSubstitutionBoundary::LegacyLatin);
        }
        let substituted = decision.substitution_boundary.is_some()
            && decision.normalized_face != decision.requested_face;
        if lang == 1 {
            latin_declared_hft = decision.alt_type == Some(2) && !substituted;
        }
        // [#7092] 문서가 적은 대체 TTF 로 한/글이 실제로 그리는 face — 폭 표도 그 글꼴을 쓴다.
        let substitute_ttf = doc_info
            .font_faces
            .get(lang)
            .and_then(|faces| faces.get(font_id as usize))
            .and_then(verified_ttf_substitute);
        font_families_metric_trusted.push(
            substitute_ttf.is_some()
                || (decision.alt_type == Some(1)
                    && !substituted
                    && decision
                        .requested_face
                        .as_deref()
                        .is_some_and(metric_widths_verified_face)),
        );
        font_families_hft_hangul
            .push(decision.substitution_boundary == Some(FontSubstitutionBoundary::Hft));
        font_families_hft_fullwidth_dot.push(decision.alt_type == Some(2) && !substituted);
        // [#7391] legacy-latin은 선언 face의 기존 표를 사용한다. HFT는
        // 프로그램 자체를 확인한 표만 사용하며 표시용 TTF와 구분한다.
        // HWP3 ASCII 반각 여부는 기존 문서의 저장 줄 증인이 계속 결정한다.
        font_families_metric_face.push(if let Some(face) = substitute_ttf {
            Some(face.to_string())
        } else if decision.alt_type == Some(2)
            && decision.requested_face.as_deref().is_some_and(|face| {
                crate::renderer::font_metrics_data::legacy_hft_metric_face(face).is_some()
            })
        {
            // 표시 이름의 치환 여부와 원본 HFT 프로그램의 폭은 별개다.
            decision
                .requested_face
                .as_deref()
                .and_then(crate::renderer::font_metrics_data::legacy_hft_metric_face)
                .map(str::to_string)
        } else {
            (decision.substitution_boundary == Some(FontSubstitutionBoundary::LegacyLatin))
                .then(|| {
                    decision
                        .requested_face
                        .as_deref()
                        .filter(|face| has_own_metric_table(face, cs.bold, cs.italic))
                        .filter(|face| {
                            decision.alt_type == Some(2) || !hft_program_metric_face(face)
                        })
                })
                .flatten()
                .map(str::to_string)
        });
        font_families.push(decision.css_family_chain.join(","));

        // 굵기의 추가 전진은 측정에서 별도로 소비한다. 음수 자간과 상쇄되는
        // 사례도 저장 자간을 무시하는 근거가 아니므로 원본 값을 유지한다.
        let spacing_percent = cs.spacings[lang] as f64;

        ratios.push(cs.ratios[lang] as f64 / 100.0);
        // 100%는 저장 줄과 같은 원래 크기를 보존한다. 먼저 곱하고 나누면
        // 반올림 잔차로 글꼴이 저장 줄보다 커져 유효한 0 간격이 재계산된다.
        let relative_size = f64::from(cs.relative_sizes[lang]) / 100.0;
        let lang_font_size = font_size * relative_size;
        // 자간은 그 언어 슬롯이 실제로 그리는 크기(상대 크기 적용 후)의 백분율이다.
        letter_spacings.push(lang_font_size * spacing_percent / 100.0);
        font_sizes.push(lang_font_size);
    }

    // [#7387] 공백은 영문 슬롯(1) 글꼴이 정한다. 속성이 꺼졌거나 그 글꼴의 공백폭을
    // 모르면 `None` 으로 두어 종전 반각 측정을 그대로 쓴다.
    let font_space_em = cs
        .use_font_space
        .then(|| {
            font_families
                .get(1)
                .and_then(|chain| declared_space_advance_em(chain, cs.bold, cs.italic))
        })
        .flatten();

    let ascii_punct_latin_slot = ascii_punctuation::uses_latin_slot(
        &font_families,
        font_families_metric_face[1].as_deref(),
        latin_slot_legacy,
        font_families_hft_hangul[0],
        latin_declared_hft,
        cs.bold,
        cs.italic,
    );

    // 한국어(0번) 값을 기본값으로 사용
    let font_family = font_families[0].clone();
    let letter_spacing = letter_spacings[0];
    let ratio = ratios[0];

    ResolvedCharStyle {
        font_family,
        font_families,
        font_families_metric_trusted,
        font_families_hft_hangul,
        font_families_source_hft,
        font_families_hft_fullwidth_dot,
        ascii_punct_latin_slot,
        font_families_metric_face,
        font_space_em,
        font_size,
        font_sizes,
        font_positions: cs.char_offsets.to_vec(),
        bold: cs.bold,
        italic: cs.italic,
        text_color: cs.text_color,
        underline: cs.underline_type,
        underline_color: cs.underline_color,
        strike_color: cs.strike_color,
        strikethrough: cs.strikethrough,
        letter_spacing,
        letter_spacings,
        ratio,
        ratios,
        border_fill_id: cs.border_fill_id,
        outline_type: cs.outline_type,
        shadow_type: cs.shadow_type,
        shadow_color: cs.shadow_color,
        shadow_offset_x: cs.shadow_offset_x,
        shadow_offset_y: cs.shadow_offset_y,
        emboss: cs.emboss,
        engrave: cs.engrave,
        superscript: cs.superscript,
        subscript: cs.subscript,
        emphasis_dot: cs.emphasis_dot,
        underline_shape: cs.underline_shape,
        strike_shape: cs.strike_shape,
        kerning: cs.kerning,
        shade_color: cs.shade_color,
    }
}

/// [#7702] 한/글이 앞 글자 언어와 무관하게 **영문 슬롯** 글꼴로 재고 그리는 기호.
///
/// 글꼴 슬롯만 영문일 뿐 영문 낱말을 이루는 글자는 아니다 — 줄 나눔은 이 글자를
/// 종전처럼 구두점으로 다룬다(`composer::line_breaking`).
pub(crate) fn is_latin_slot_symbol(ch: char) -> bool {
    matches!(
        ch,
        '\u{00B0}' | '\u{00B7}' | '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}'
    )
}

/// Unicode 코드포인트로 HWP 언어 카테고리를 판별한다.
///
/// 반환값: 0=한국어, 1=영어(라틴), 2=한자, 3=일본어, 4=기타, 5=기호, 6=사용자
///
/// 공백/일반 구두점은 언어 중립으로 간주하여 기본값(한국어)을 반환한다.
/// 호출부에서 "이전 문자의 언어를 따르는" 로직을 별도 처리해야 한다.
pub fn detect_lang_category(ch: char) -> usize {
    let cp = ch as u32;
    match cp {
        // [#2070] ㆍ(아래아, U+318D)는 한컴이 USER 스크립트 폰트로 렌더한다.
        // 80168 실문서(user=9='명조', 반각 오라클)와 사다리 v3(user=한양신명조,
        // 전각 실측)를 동시에 만족하는 유일 분류. 호환 자모 블록이지만
        // 한글(0)이 아니라 사용자(6)로 분류해 user 폰트를 태운다.
        0x318D => 6,

        // [#7092] 굽은 따옴표 ‘ ’ “ ” 는 한컴이 **영문(1) 슬롯** 글꼴로 그린다.
        // 저장소 `samples/`↔`pdf/` 정본 233문서에서 한글·영문·기호 슬롯 face 가 서로
        // 다른 출현만 골라 정본 글자의 PDF face 와 대조하면, 영문 슬롯만 일치하는 문서가
        // ‘ 46 · ’ 39 · “ 29 · ” 27 개이고 한글 슬롯만 일치하는 문서는 0 개다(기호 슬롯만
        // 일치 1 개). 같은 General Punctuation 블록이라도 `․`(U+2024)는 기호 슬롯 쪽으로
        // 갈리고 `…`·`–`·`—`·`※`·`•` 는 판별 가능한 표본이 없어, 블록 전체가 아니라
        // 근거가 있는 네 글자만 옮긴다.
        0x2018 | 0x2019 | 0x201C | 0x201D => 1,

        // 한점 리더는 앞 한글의 face가 아닌 원본 기호 슬롯을 소비한다.
        // 독립 한컴 Print의 HFT 전각 전진과 같은 슬롯의 측정·표시를 유지한다.
        0x2024 => 5,

        // 한국어: Hangul Jamo, Compatibility Jamo, Syllables
        0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7AF |
        // Hangul Jamo Extended-A/B
        0xA960..=0xA97F | 0xD7B0..=0xD7FF => 0,

        // 영어/라틴: Basic Latin letters+digits, Latin Extended
        0x0041..=0x005A | 0x0061..=0x007A | 0x0030..=0x0039 |
        0x00C0..=0x024F |
        // Latin Extended Additional, Extended-B (subset)
        0x1E00..=0x1EFF => 1,

        // [#7702] 가운뎃점 `·` 과 곡선 따옴표 ‘’“” 는 앞 글자가 한글이어도 한/글이 **영문
        // 슬롯** 글꼴로 재고 그린다. 한글 휴먼명조 + 영문 Times New Roman 합성 문단(한/글
        // 2020)에서 `“`·`”` 는 0.444em, `‘`·`’`·`·` 는 0.336em 이고 PDF 글꼴도 Times 다. 영문
        // 슬롯을 휴먼명조로 바꾸면 따옴표가 1.0em 이 된다. 같은 문단의 `…`·`※`·`「` 등은
        // 휴먼명조로 남는다. 도 기호 `°` 도 Times 로 그려진다. 뒤따르는 공백도 Times 로
        // 그려진다(폭은 어느 쪽이든 0.5em).
        _ if is_latin_slot_symbol(ch) => 1,

        // 한자: CJK Unified Ideographs, Extension A
        0x4E00..=0x9FFF | 0x3400..=0x4DBF |
        // CJK Compatibility Ideographs
        0xF900..=0xFAFF |
        // CJK Unified Extension B (서로게이트 쌍이 아닌 범위)
        0x20000..=0x2A6DF => 2,

        // 일본어: Hiragana, Katakana
        0x3040..=0x309F | 0x30A0..=0x30FF |
        // Katakana Phonetic Extensions
        0x31F0..=0x31FF => 3,

        // 기호: 수학 기호, 화살표, 기술 기호, 도형, Dingbats 등
        0x2190..=0x21FF | 0x2200..=0x22FF | 0x2300..=0x23FF |
        0x2500..=0x257F | 0x2580..=0x259F | 0x25A0..=0x25FF |
        0x2600..=0x26FF | 0x2700..=0x27BF |
        // 원 숫자, 괄호 숫자 등
        0x2460..=0x24FF |
        // CJK 기호/구두점 (한자 구두점이 아닌 기호 영역)
        0x3000..=0x303F => 5,

        // 공백/ASCII 구두점/제어문자 → 한국어(기본값)로 반환
        // 호출부에서 "이전 문자의 언어를 따르는" 로직으로 처리
        _ => 0,
    }
}

/// FontFace 테이블에서 폰트 이름 조회 + 폰트 치환 적용
///
/// HWP 문서의 폰트 이름을 웹/SVG에서 렌더링 가능한 폰트로 치환한다.
/// webhwp의 g_SubstFonts 치환 체인을 평탄화(flatten)한 테이블을 사용한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FontSubstitutionBoundary {
    LegacyLatin,
    Hft,
    Ttf,
}

impl FontSubstitutionBoundary {
    pub(crate) const fn source_boundary_id(self) -> &'static str {
        match self {
            Self::LegacyLatin => "rust-style-resolution.legacy-latin",
            Self::Hft => "rust-style-resolution.hft",
            Self::Ttf => "rust-style-resolution.ttf",
        }
    }

    pub(crate) fn language_condition(self, source_face: &str) -> &'static str {
        match self {
            Self::LegacyLatin => "1",
            Self::Ttf => "all",
            Self::Hft
                if find_font_rule_layout_name(self.source_boundary_id(), source_face, 0)
                    .is_some() =>
            {
                "all"
            }
            Self::Hft => "1",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FontNameDecision {
    pub(crate) environment_profile_id: Option<String>,
    pub(crate) language_slot: usize,
    pub(crate) font_id: u16,
    pub(crate) requested_face: Option<String>,
    pub(crate) alt_type: Option<u8>,
    pub(crate) embedded: Option<bool>,
    pub(crate) normalized_face: Option<String>,
    pub(crate) subst_font: Option<String>,
    pub(crate) css_family_chain: Vec<String>,
    pub(crate) substitution_boundary: Option<FontSubstitutionBoundary>,
    pub(crate) substitution_rule_id: Option<&'static str>,
}

/// [#7092] 메트릭 표에 적힌 폭이 **그 글꼴 자신의 전진폭**임을 글꼴 파일로 확인한 face 인지.
///
/// 문서가 적은 `alt_type`(TTF/HFT)은 *문서의 주장*일 뿐 표의 출처를 말해 주지 않는다.
/// `76076_regulatory_analysis.hwp` 는 `함초롬바탕` 을 `alt_type=1`(TTF)로 선언하지만 메트릭
/// 조회는 별칭 규칙(`rule.rust-metric.1de1fcb9b17d66d599b5`)으로 **다른 글꼴인 `HCR Batang`
/// 표**를 쓴다. 한/글 2024 정본은 그 문서의 `·` 를 0.345em 으로 그린다 — `덮개·울`(한글3+·)
/// 37.8pt 와 `덮개·울을`(한글4+·) 49.1pt 에서 한글 전진폭 11.3pt 를 빼면 3.9pt 다. 선언을
/// 근거로 표를 믿으면 이 글자가 전각이 되어 조각 경계가 한 행 밀린다.
///
/// 그래서 근거는 선언이 아니라 **글꼴 파일 실측**으로 둔다. 별칭이 같은 글꼴의 다른 이름인지
/// (`HY신명조` → `HYSinMyeongJo-Medium`) 다른 글꼴인지(`함초롬바탕` → `HCR Batang`)는 규칙
/// 표만으로 구분되지 않으므로, 확인한 face 만 여기에 적는다.
///
/// - `HY신명조` — `H2MJSM.TTF` 가 `periodcentered`(gid 20313 · 윤곽선 1개)를 1024/1024 로
///   갖고, `#7092` 재현체의 한/글 정본이 0.999em 이다(`·` 31회 전부 이 face).
/// - `HY헤드라인M` — `ttfs/hwp/H2HDRM.TTF` 가 `periodcentered` 를 1024/1024 로 갖고
///   윤곽선 bbox 가 `(439, 297, 586, 435)` 다(결측 글리프가 아니다). 저장소 한컴 정본
///   **25개 문서**에서 이 face 의 `·` 전진폭이 0.89~1.06em 이고 중앙값이 1.000 이다
///   (`k-water-rfp` 5종 · `mel-001` 3종 · `aift-2022` · `pr_6528_issue6181_p5` …).
///   좁은 갈래(0.2~0.4em)는 한 건도 없다.
/// - `HY울릉도M` — `ttfs/hwp/HYWULM.TTF` 가 같은 글리프를 1024/1024 로 갖고, 정본
///   `press_release_split_cell_nested_table-hwpx-2020`(n=8, 0.944~1.000) 과
///   `pr_6528_issue6181_p5_2020`(n=1, 1.000) 이 전각을 말한다.
///
/// 새 face 를 넣으려면 그 글꼴 파일의 글리프와 한/글 출력 실측을 함께 남긴다.
///
/// - `휴먼명조` — `HMKMM.TTF` 가 `‘’“”` 를 모두 512/512 로 갖는다. 저장소 한컴 정본
///   23문서에서 이 face 의 `‘`·`’` 전진폭 중앙값이 0.988em 이고(n=2,042), 한/글 2020
///   합성 문단(영문 슬롯 휴먼명조 TTF)도 `‘’“”` 를 1.0em 으로 놓는다(#7702). HFT 로
///   선언된 휴먼명조는 `alt_type == 1` 을 만족하지 않으므로 여기서 걸러진다.
/// - `Times New Roman` — `curly_quote_latin_slot` 독립 Print의 CID TrueType 프로그램과
///   공급한 원 프로그램의 `‘’“”` 전진폭·분해 윤곽선이 모두 동일하다. 작은따옴표는
///   682/2048em, 큰따옴표는 909/2048em으로 원래 좁은 글리프다. 이 TTF 선택을 신뢰하지
///   않으면 측정한 원 전진폭은 유지하면서 Canvas/Skia만 다시 절반으로 축소한다.
///
/// - `HY중고딕` — 공식 설치 H2GTRM.TTF의 `·‘’“”`는 1024/1024em이다.
///   법령의 독립 한컴 Print와 내어쓰기 통제 복사본도 가운뎃점의 전각을 입증한다.
///   글꼴 파일 자체를 저장소에 넣지 않고 프로그램 SHA와 추출 근거를 기록한다.
///   [#7092] 저장소 한컴 정본 **8개 문서**(78494 n=61 · 76076 2020 출력 n=32 · 75544 n=26 ·
///   issue1891_external_bindata_link n=14 · 80550 · regulatory_impact_nested_table_escape ·
///   113424 · pr6275)에서 `H2gtrM` 의 `·` 전진폭 중앙값이 1.000em(0.88~1.02, n=145)이다.
///   이 face 는 직접 선언보다 `한양중고딕` 의 **대체 TTF** 로 더 자주 쓰인다
///   ([`verified_ttf_substitute`]).
/// - `HY견고딕` — `H2GTRE.TTF`(bbox `(432, 286, 593, 446)`) 1024/1024, 정본
///   `trade_report_forced_break_object-2024`(n=1, 1.000) · 같은 문서 2020 출력(0.989).
/// - `HY그래픽` — `H2GPRM.TTF`(family `HY그래픽M`, bbox `(460, 316, 564, 421)`) 1024/1024,
///   정본 `press_release_split_cell_nested_table-hwpx-2020`(n=1, 1.000).
///
/// 아직 넣지 않은 것 — `휴먼고딕`(글꼴 파일은 512/512 이고 정본 2문서 n=9 가 1.0em 이지만
/// 저장소에 TTF 로 선언한 짝 표본이 없다).
fn metric_widths_verified_face(face: &str) -> bool {
    matches!(
        face.trim(),
        "HY신명조"
            | "HYSinMyeongJo-Medium"
            | "HY중고딕"
            | "HYGothic-Medium"
            | "HY견고딕"
            | "HYGothic-Extra"
            | "HY그래픽"
            | "HYGraphic-Medium"
            | "HY헤드라인M"
            | "HYHeadLine-Medium"
            | "HY울릉도M"
            | "HYwulM"
            | "휴먼명조"
            | "HumanMyeongJo"
            | "Times New Roman"
    )
}

pub(crate) fn lookup_font_name_decision(
    doc_info: &DocInfo,
    lang_index: usize,
    font_id: u16,
) -> FontNameDecision {
    let mut decision = FontNameDecision {
        environment_profile_id: None,
        language_slot: lang_index,
        font_id,
        requested_face: None,
        alt_type: None,
        embedded: None,
        normalized_face: None,
        subst_font: None,
        css_family_chain: Vec::new(),
        substitution_boundary: None,
        substitution_rule_id: None,
    };
    if lang_index < doc_info.font_faces.len() {
        let lang_fonts = &doc_info.font_faces[lang_index];
        if (font_id as usize) < lang_fonts.len() {
            let font = &lang_fonts[font_id as usize];
            let name = &font.name;
            decision.requested_face = Some(name.clone());
            decision.alt_type = Some(font.alt_type);
            decision.embedded = Some(font.is_embedded);
            // 폰트 치환: HFT 등 웹 미지원 폰트를 렌더링 가능한 폰트로 완전 대체
            let substitution = resolve_font_substitution_decision(name, font.alt_type, lang_index);
            let resolved = substitution
                .map(|(face, _, _)| face)
                .unwrap_or(name)
                .to_string();
            decision.normalized_face = Some(resolved.clone());
            decision.substitution_boundary = substitution.map(|(_, boundary, _)| boundary);
            decision.substitution_rule_id = substitution.map(|(_, _, rule_id)| rule_id);
            decision.css_family_chain.push(resolved.clone());
            if let Some(substitute) = font
                .subst_font
                .as_ref()
                .filter(|substitute| !substitute.is_embedded)
                .filter(|substitute| !substitute.face.trim().is_empty())
                .filter(|substitute| substitute.face.trim() != resolved)
            {
                let face = substitute.face.trim().to_string();
                decision.subst_font = Some(face.clone());
                decision.css_family_chain.push(face);
            }
        }
    }
    decision
}

pub(crate) fn lookup_font_name_in_environment(
    doc_info: &DocInfo,
    lang_index: usize,
    font_id: u16,
    environment: Option<&super::font_environment::FontEnvironment>,
) -> FontNameDecision {
    let mut decision = lookup_font_name_decision(doc_info, lang_index, font_id);
    if decision.embedded != Some(true) {
        if let Some((environment, target)) = environment.and_then(|env| {
            decision
                .requested_face
                .as_deref()
                .and_then(|face| env.replacement(face))
                .map(|target| (env, target))
        }) {
            decision.normalized_face = Some(target.to_string());
            decision.css_family_chain = vec![target.to_string()];
            decision.substitution_boundary = None;
            decision.substitution_rule_id = None;
            decision.environment_profile_id = Some(environment.id().to_string());
        }
    }
    if decision.embedded != Some(true)
        && environment.is_some_and(|env| {
            decision
                .normalized_face
                .as_deref()
                .is_some_and(|face| env.selects_true_type(face))
        })
    {
        decision.environment_profile_id = environment.map(|env| env.id().to_string());
    }
    if decision.embedded != Some(true) {
        if let Some((environment, fallbacks)) = environment.and_then(|env| {
            decision
                .normalized_face
                .as_deref()
                .and_then(|face| env.glyph_fallbacks(face))
                .map(|fallbacks| (env, fallbacks))
        }) {
            // 호출자가 검증한 체인은 일반 OS 폴백보다 먼저 사용한다.
            // 기본 face·원본 substFont를 보존하고 중복만 제거한다.
            let source_fallbacks = decision.css_family_chain.split_off(1);
            for face in fallbacks.iter().chain(&source_fallbacks) {
                if !decision.css_family_chain.contains(face) {
                    decision.css_family_chain.push(face.clone());
                }
            }
            decision.environment_profile_id = Some(environment.id().to_string());
        }
    }
    decision
}

fn lookup_font_name(doc_info: &DocInfo, lang_index: usize, font_id: u16) -> String {
    lookup_font_name_decision(doc_info, lang_index, font_id)
        .css_family_chain
        .join(",")
}

/// 폰트명에서 원본(첫 번째) 폰트명만 추출 (폴백 제거)
pub fn primary_font_name(font_family: &str) -> &str {
    font_family.split(',').next().unwrap_or(font_family).trim()
}

/// webhwp g_SubstFonts 기반 폰트 치환
///
/// HWP 문서의 원본 폰트 이름 + 타입(TTF/HFT) + 언어 카테고리를 기반으로
/// @font-face에 등록된 최종 폰트로 치환한다.
/// 체인이 이미 평탄화되어 1회 조회로 최종 결과를 반환한다.
pub(crate) fn resolve_font_substitution(
    name: &str,
    alt_type: u8,
    lang_index: usize,
) -> Option<&'static str> {
    resolve_font_substitution_decision(name, alt_type, lang_index).map(|(face, _, _)| face)
}

pub(crate) fn resolve_font_substitution_decision(
    name: &str,
    alt_type: u8,
    lang_index: usize,
) -> Option<(&'static str, FontSubstitutionBoundary, &'static str)> {
    // HWP3 원본/일부 한컴 재저장본은 HCI 영문 폰트를 TTF(type=1) 또는
    // unknown(type=0)으로 싣기도 한다. 한컴은 같은 face를 보여주므로
    // alt_type 차이와 무관하게 legacy 영문 HFT 치환을 우선 적용한다.
    if let Some(rule) =
        resolve_projected_font_rule(FontSubstitutionBoundary::LegacyLatin, name, lang_index)
    {
        return Some((
            rule.target_face_or_policy,
            FontSubstitutionBoundary::LegacyLatin,
            rule.rule_id,
        ));
    }

    // HFT(type=2) 폰트 치환
    if alt_type == 2 {
        if let Some(rule) =
            resolve_projected_font_rule(FontSubstitutionBoundary::Hft, name, lang_index)
        {
            return Some((
                rule.target_face_or_policy,
                FontSubstitutionBoundary::Hft,
                rule.rule_id,
            ));
        }
    }

    // TTF(type=1) 또는 알수없음(type=0) 치환
    resolve_projected_font_rule(FontSubstitutionBoundary::Ttf, name, lang_index).map(|rule| {
        (
            rule.target_face_or_policy,
            FontSubstitutionBoundary::Ttf,
            rule.rule_id,
        )
    })
}

fn resolve_projected_font_rule(
    boundary: FontSubstitutionBoundary,
    name: &str,
    lang_index: usize,
) -> Option<&'static GeneratedFontRuleProjection> {
    find_font_rule_layout_name(boundary.source_boundary_id(), name, lang_index)
}

/// Heavy display 계열 face 여부 판정.
///
/// HY헤드라인M, HY견고딕 등 face 이름 자체가 굵은 display 폰트들은
/// HWP CharShape.bold=false 로 저장되어도 실제로는 시각적 bold 로
/// 렌더된다. 해당 face 가 설치되지 않은 환경에서 Malgun Gothic 등
/// regular weight fallback 으로 떨어지면 PDF(한컴) 출력과 시각 괴리가
/// 발생하므로, 이 리스트에 포함된 face 는 SVG 에서 font-weight="bold"
/// 를 강제해 fallback bold variant 로 근사 렌더한다.
///
/// Task #574: HY견명조 는 한컴 일반 두께 명조 — heavy 가 아님. 제거.
/// HY견명조B (명시 Bold variant) 는 보존.
pub(crate) fn is_heavy_display_face(font_family: &str) -> bool {
    // font_family 는 "HY헤드라인M,'Malgun Gothic',..." 처럼 CSS 체인 형태.
    // 첫 face 만 검사 (HWP 가 지정한 primary face).
    let primary = font_family
        .split(',')
        .next()
        .unwrap_or(font_family)
        .trim()
        .trim_matches('\'')
        .trim_matches('"');
    matches!(
        primary,
        "HY헤드라인M"
            | "HYHeadLine M"
            | "HYHeadLine Medium"
            | "HY견고딕"
            | "HY견명조B"
            | "HY그래픽"
            | "HY그래픽M"
    )
}

fn primary_font_face(font_family: &str) -> &str {
    font_family
        .split(',')
        .next()
        .unwrap_or(font_family)
        .trim()
        .trim_matches('\'')
        .trim_matches('"')
}

/// Face name explicitly carries a bold weight.
pub(crate) fn is_bold_weight_face(font_family: &str) -> bool {
    let primary = primary_font_face(font_family);
    let lower = primary.to_lowercase();
    lower.contains("bold") || lower.contains("볼드")
}

/// Face name explicitly carries a light weight.
pub(crate) fn is_light_weight_face(font_family: &str) -> bool {
    let primary = primary_font_face(font_family);
    let lower = primary.to_lowercase();
    !is_bold_weight_face(font_family)
        && (lower.contains("light")
            || lower.contains("extralight")
            || lower.contains("thin")
            || lower.contains("ultralight"))
}

/// 중고딕/태고딕 계열 (CSS font-weight 500) 폰트 판별.
///
/// HWP 에서 중고딕 계열은 Regular(400)과 Bold(700) 사이의 Medium(500) weight.
/// Fallback 폰트 매칭 시 weight 500 힌트를 주어 선명도를 유지한다.
pub(crate) fn is_medium_weight_face(font_family: &str) -> bool {
    let primary = primary_font_face(font_family);
    let lower = primary.to_lowercase();
    lower.contains("중고딕")
        || lower.contains("태고딕")
        || lower.contains("mediumgothic")
        || lower.contains("hymedium")
}

/// ParaShape → ResolvedParaStyle 목록
fn resolve_para_styles(doc_info: &DocInfo, dpi: f64) -> Vec<ResolvedParaStyle> {
    resolve_para_styles_with_variant(doc_info, dpi, false)
}

/// [Task #1001] 변환본 인지 ParaShape 해소
fn resolve_para_styles_with_variant(
    doc_info: &DocInfo,
    dpi: f64,
    is_hwp3_variant: bool,
) -> Vec<ResolvedParaStyle> {
    let char_unit_hu = para_margin_char_unit_hu(doc_info);
    doc_info
        .para_shapes
        .iter()
        .map(|ps| {
            resolve_single_para_style(ps, &doc_info.tab_defs, dpi, is_hwp3_variant, char_unit_hu)
        })
        .collect()
}

/// [#7816] 문단 여백의 «글자» 단위 하나가 몇 HWPUNIT 인가 — 바탕글 스타일 글자 크기의 1/200.
///
/// 한/글 2020 통제 실험(저장 줄 없는 합성 HWPX·그 HWP 저장본)에서 홀수 저장값 `s` 의 양은
/// `(s − 1) / 2` 단위였고, 단위 하나는 바탕글 10pt 에서 5 HU · 20pt 에서 10 HU 였다.
/// 바탕글이 가리키는 글자 모양만 바꾸면 따라 바뀌고, 글자 모양 0 이나 문단 자신의 글자
/// 크기를 바꾸면 그대로였다. 바탕글을 찾지 못하면 10pt(1000) 로 둔다.
fn para_margin_char_unit_hu(doc_info: &DocInfo) -> f64 {
    let base_size = doc_info
        .styles
        .first()
        .and_then(|style| doc_info.char_shapes.get(style.char_shape_id as usize))
        .map(|shape| shape.base_size)
        .filter(|&size| size > 0)
        .unwrap_or(1000);
    base_size as f64 / 200.0
}

/// [#7816] ParaShape 여백·문단 간격 저장값 → HWPUNIT.
///
/// 저장값은 HWPUNIT 의 2배다. 최하위 비트는 단위 표지라, **홀수면 «글자» 단위**다 —
/// 한/글 HWPX 는 그 자리를 HwpUnitChar `case` 에 `unit="CHAR"` · 값 `(s − 1) / 2` 로 적는다
/// (`serializer/hwpx/header.rs` `write_margin_child`, #6875). 한/글은 이 값을 단위 수로 읽어
/// 단위 하나에 [`para_margin_char_unit_hu`] HU 를 준다. 들여쓰기·왼쪽·오른쪽·문단 위·아래
/// 다섯 항목 모두 같다(저장값 1001·2001·4001 → 33.3·66.7·133.3px, 바탕글 10pt).
fn para_margin_hwpunit(stored: i32, char_unit_hu: f64) -> f64 {
    if stored % 2 != 0 {
        ((stored - 1) / 2) as f64 * char_unit_hu
    } else {
        (stored / 2) as f64
    }
}

/// 개별 ParaShape 해소
fn resolve_single_para_style(
    ps: &ParaShape,
    tab_defs: &[TabDef],
    dpi: f64,
    is_hwp3_variant: bool,
    char_unit_hu: f64,
) -> ResolvedParaStyle {
    // [#2070] 비-Percent 줄간격(Fixed/SpaceOnly/Minimum)도 여백·문단간격·탭과
    // 동일하게 저장값이 유효 HWPUNIT 의 2배다 — 통제 사다리(#2197)로 확정한
    // 계약과 실문서 이중 실증: 시장구조조사 본문 ps Fixed 3320HU, 한글 PDF
    // 줄 pitch 22.1px = 3320/2 (rhwp 종전 44.3px = 2배 팽창); 편람 한컴 HWPX
    // case FIXED=1560/default=3120 (case=유효, default=저장). HWPX 파서도
    // case×2 적재라 양 포맷 IR 동일 스케일 — 일괄 /2 가 정합.
    let line_spacing = match ps.line_spacing_type {
        LineSpacingType::Percent => ps.line_spacing as f64,
        _ => hwpunit_to_px(ps.line_spacing, dpi) / 2.0,
    };

    // 기본 탭 간격: HWP 기본값 80pt (8000 HWPUNIT)
    let default_tab_width = hwpunit_to_px(4000, dpi);

    // 커스텀 탭 정지 해소: TabDef.tabs[] → px 변환
    // TabItem.position은 ParaShape 여백과 동일하게 2배 스케일로 저장되므로
    // 렌더링 시 2로 나누어야 한다 (hwp2hwpx 변환 코드 및 HWP 대화상자 확인).
    let tab_def = tab_defs.get(ps.tab_def_id as usize);
    let tab_stops: Vec<TabStop> = tab_def
        .map(|td| {
            td.tabs
                .iter()
                .map(|t| TabStop {
                    position: hwpunit_to_px(t.position as i32, dpi) / 2.0, // HWP 탭 position은 실제 좌표의 2배로 저장됨 (한컴 격자 비교로 확인)
                    tab_type: t.tab_type,
                    fill_type: t.fill_type,
                })
                .collect()
        })
        .unwrap_or_default();
    let auto_tab_right = tab_def.map(|td| td.auto_tab_right).unwrap_or(false);

    // ParaShape의 여백 및 문단 간격은 HWPUNIT의 2배 값으로 저장된다.
    // margin_left/right/indent: LineSeg.column_start와 비교하면 column_start = margin_left / 2
    // spacing_before/after: pyhwpx 확인 결과 동일하게 2배 스케일 저장
    // 실제 렌더링 시 2로 나누어야 올바른 값이 된다.
    //
    // [Task #1037] HWP5 변환본 의 추가 2배 스케일 (총 4배) 은 parser 단계 (parser/mod.rs)
    // 에서 normalize (halve) 되어 본 단계에서는 normal HWP5 동등 (2배 스케일) — uniform
    // variant_div=2 적용. 종전 variant_div=4 는 raw 값 normalize 전 보정 패턴이었음.
    //
    // [#7816] 홀수 저장값은 «글자» 단위다 — [`para_margin_hwpunit`].
    let _ = is_hwp3_variant;
    let margin_px = |stored: i32| {
        para_margin_hwpunit(stored, char_unit_hu) * dpi / crate::renderer::HWPUNIT_PER_INCH
    };
    ResolvedParaStyle {
        alignment: ps.alignment,
        vertical_alignment: ((ps.attr1 >> 20) & 0x03) as u8,
        line_spacing,
        line_spacing_type: ps.line_spacing_type,
        margin_left: margin_px(ps.margin_left),
        margin_right: margin_px(ps.margin_right),
        indent: margin_px(ps.indent),
        spacing_before: margin_px(ps.spacing_before),
        spacing_after: margin_px(ps.spacing_after),
        head_type: ps.head_type,
        para_level: ps.para_level,
        numbering_id: ps.numbering_id,
        border_fill_id: ps.border_fill_id,
        border_spacing: [
            hwpunit_to_px(ps.border_spacing[0] as i32, dpi),
            hwpunit_to_px(ps.border_spacing[1] as i32, dpi),
            hwpunit_to_px(ps.border_spacing[2] as i32, dpi),
            hwpunit_to_px(ps.border_spacing[3] as i32, dpi),
        ],
        border_connect: ps.attr1 & (1 << 28) != 0,
        border_ignore_margin: ps.attr1 & (1 << 29) != 0,
        default_tab_width,
        tab_stops,
        auto_tab_right,
        condense_min_space: ((ps.attr1 >> 9) & 0x7f).min(75) as u8,
        english_break_unit: ((ps.attr1 >> 5) & 0x03) as u8,
        korean_break_unit: ((ps.attr1 >> 7) & 0x01) as u8,
        widow_orphan: (ps.attr1 >> 16) & 1 != 0,
        keep_with_next: (ps.attr1 >> 17) & 1 != 0,
        keep_lines: (ps.attr1 >> 18) & 1 != 0,
        page_break_before: (ps.attr1 >> 19) & 1 != 0,
    }
}

/// BorderFill → ResolvedBorderStyle 목록
fn resolve_border_styles(doc_info: &DocInfo, dpi: f64) -> Vec<ResolvedBorderStyle> {
    doc_info
        .border_fills
        .iter()
        .map(|bf| resolve_single_border_style(bf, dpi))
        .collect()
}

/// 개별 BorderFill 해소
fn resolve_single_border_style(bf: &BorderFill, dpi: f64) -> ResolvedBorderStyle {
    let fill_color = match bf.fill.fill_type {
        FillType::Solid => bf.fill.solid.as_ref().and_then(|s| {
            // pattern_type > 0: 패턴 채우기 → 단색 fill 아님 (background_color는 패턴 배경)
            // ColorRef 상위 바이트가 0이 아니면 "채우기 없음" (투명)
            // 0xFFFFFFFF = CLR_INVALID/CLR_DEFAULT (Windows COLORREF)
            if s.pattern_type > 0 || (s.background_color >> 24) != 0 {
                None
            } else {
                Some(s.background_color)
            }
        }),
        _ => None,
    };

    let pattern = match bf.fill.fill_type {
        FillType::Solid => bf.fill.solid.as_ref().and_then(|s| {
            if s.pattern_type > 0 {
                Some(PatternFillInfo {
                    pattern_type: s.pattern_type,
                    tile_size: hwpunit_to_px(576, dpi),
                    pattern_color: s.pattern_color,
                    background_color: s.background_color,
                })
            } else {
                None
            }
        }),
        _ => None,
    };

    let gradient = match bf.fill.fill_type {
        FillType::Gradient => bf.fill.gradient.as_ref().and_then(|g| {
            // 유효성 검사: 색상 2개 미만이거나 비정상적으로 많으면 무효
            if g.colors.len() < 2 || g.colors.len() > 64 {
                return None;
            }
            // 중심좌표가 비정상 범위이면 파싱 오류로 판단
            if g.center_x.abs() > 200 || g.center_y.abs() > 200 {
                return None;
            }
            let positions: Vec<f64> = if g.positions.is_empty() {
                let n = g.colors.len();
                (0..n)
                    .map(|i| i as f64 / (n.max(2) - 1).max(1) as f64)
                    .collect()
            } else {
                g.positions.iter().map(|&p| p as f64 / 100.0).collect()
            };
            // [#6822] `step`(띠 개수)·`step_center`(전이 위치)를 stop 으로 편다.
            let (colors, positions) =
                super::expand_gradient_steps(&g.colors, &positions, g.blur, g.step_center);
            Some(Box::new(GradientFillInfo {
                gradient_type: g.gradient_type,
                angle: g.angle,
                center_x: g.center_x,
                center_y: g.center_y,
                colors,
                positions,
            }))
        }),
        _ => None,
    };

    let image_fill = match bf.fill.fill_type {
        FillType::Image => bf.fill.image.as_ref().map(|img| ResolvedImageFill {
            bin_data_id: img.bin_data_id,
            fill_mode: img.fill_mode,
            brightness: img.brightness,
            contrast: img.contrast,
            effect: image_fill_effect(img.effect),
        }),
        _ => None,
    };

    ResolvedBorderStyle {
        borders: bf.borders,
        fill_color,
        pattern,
        gradient,
        image_fill,
        diagonal_attr: bf.attr,
        diagonal: bf.diagonal,
        center_line: if bf.center_line != CenterLine::None {
            bf.center_line
        } else {
            CenterLine::from_hwp_attr(bf.attr)
        },
    }
}

fn image_fill_effect(effect: u8) -> ImageEffect {
    match effect {
        1 => ImageEffect::GrayScale,
        2 => ImageEffect::BlackWhite,
        3 => ImageEffect::Pattern8x8,
        _ => ImageEffect::RealPic,
    }
}

#[cfg(test)]
mod tests;
