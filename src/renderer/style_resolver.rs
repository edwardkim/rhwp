//! 스타일 해소 (Style Resolution)
//!
//! DocInfo 참조 테이블을 렌더링에서 바로 사용할 수 있는
//! 해소된 스타일 목록(ResolvedStyleSet)으로 변환한다.

use crate::model::document::DocInfo;
use crate::model::style::{
    Alignment, BorderFill, BorderLine, Bullet, CharShape, DiagonalLine, HeadType,
    ImageFillMode, LineSpacingType, Numbering, ParaShape, TabDef, UnderlineType, FillType,
};
use crate::model::ColorRef;
use super::{hwpunit_to_px, GradientFillInfo, PatternFillInfo, TabStop};

/// HWP 언어 카테고리 수 (한국어, 영어, 한자, 일본어, 기타, 기호, 사용자)
pub const LANG_COUNT: usize = 7;

/// 해소된 글자 스타일 (CharShape + FontFace → 렌더링용)
#[derive(Debug, Clone)]
pub struct ResolvedCharStyle {
    /// 글꼴 이름 (한국어 = 기본값, font_families[0]과 동일)
    pub font_family: String,
    /// 7개 언어 카테고리별 글꼴 이름
    pub font_families: Vec<String>,
    /// 글꼴 크기 (px)
    pub font_size: f64,
    /// 진하게
    pub bold: bool,
    /// 기울임
    pub italic: bool,
    /// 글자 색상
    pub text_color: ColorRef,
    /// 밑줄 종류
    pub underline: UnderlineType,
    /// 밑줄 색상
    pub underline_color: ColorRef,
    /// 취소선 색상
    pub strike_color: ColorRef,
    /// 취소선 여부
    pub strikethrough: bool,
    /// 자간 (px, 한국어 = 기본값, letter_spacings[0]과 동일)
    pub letter_spacing: f64,
    /// 7개 언어 카테고리별 자간 (px)
    pub letter_spacings: Vec<f64>,
    /// 장평 비율 (1.0 = 100%, 한국어 = 기본값, ratios[0]과 동일)
    pub ratio: f64,
    /// 7개 언어 카테고리별 장평 비율
    pub ratios: Vec<f64>,
    /// 글자 테두리/배경 ID (1-based, 0이면 없음)
    pub border_fill_id: u16,
    /// 외곽선 종류 (0=없음, 1~6=종류)
    pub outline_type: u8,
    /// 그림자 종류 (0=없음, 1=비연속, 2=연속)
    pub shadow_type: u8,
    /// 그림자 색
    pub shadow_color: ColorRef,
    /// 그림자 X 오프셋 (-100~100%)
    pub shadow_offset_x: i8,
    /// 그림자 Y 오프셋 (-100~100%)
    pub shadow_offset_y: i8,
    /// 양각
    pub emboss: bool,
    /// 음각
    pub engrave: bool,
    /// 위 첨자
    pub superscript: bool,
    /// 아래 첨자
    pub subscript: bool,
    /// 강조점 종류 (0=없음, 1=● 2=○ 3=ˇ 4=˜ 5=･ 6=:)
    pub emphasis_dot: u8,
    /// 밑줄 모양 (0=실선, 1=긴점선, ..., 10=삼중선, 표 27)
    pub underline_shape: u8,
    /// 취소선 모양 (0=실선, 1=긴점선, ..., 10=삼중선, 표 27)
    pub strike_shape: u8,
    /// 커닝 여부
    pub kerning: bool,
    /// 음영 색 (형광펜, 0xFFFFFF = 없음)
    pub shade_color: ColorRef,
}

impl Default for ResolvedCharStyle {
    fn default() -> Self {
        Self {
            font_family: String::new(),
            font_families: Vec::new(),
            font_size: 12.0,
            bold: false,
            italic: false,
            text_color: 0,
            underline: UnderlineType::None,
            underline_color: 0,
            strike_color: 0,
            strikethrough: false,
            letter_spacing: 0.0,
            letter_spacings: Vec::new(),
            ratio: 1.0,
            ratios: Vec::new(),
            border_fill_id: 0,
            outline_type: 0,
            shadow_type: 0,
            shadow_color: 0x00B2B2B2,
            shadow_offset_x: 0,
            shadow_offset_y: 0,
            emboss: false,
            engrave: false,
            superscript: false,
            subscript: false,
            emphasis_dot: 0,
            underline_shape: 0,
            strike_shape: 0,
            kerning: false,
            shade_color: 0x00FFFFFF,
        }
    }
}

impl ResolvedCharStyle {
    /// 지정 언어 카테고리의 폰트 이름을 반환한다.
    /// 해당 언어에 폰트가 없으면 한국어(0번) 폴백.
    pub fn font_family_for_lang(&self, lang_index: usize) -> &str {
        if lang_index < self.font_families.len() {
            let name = &self.font_families[lang_index];
            if !name.is_empty() {
                return name;
            }
        }
        &self.font_family
    }

    /// 지정 언어 카테고리의 자간(px)을 반환한다.
    pub fn letter_spacing_for_lang(&self, lang_index: usize) -> f64 {
        if lang_index < self.letter_spacings.len() {
            self.letter_spacings[lang_index]
        } else {
            self.letter_spacing
        }
    }

    /// 지정 언어 카테고리의 장평 비율을 반환한다.
    pub fn ratio_for_lang(&self, lang_index: usize) -> f64 {
        if lang_index < self.ratios.len() {
            self.ratios[lang_index]
        } else {
            self.ratio
        }
    }
}

/// 해소된 문단 스타일 (ParaShape → 렌더링용)
#[derive(Debug, Clone)]
pub struct ResolvedParaStyle {
    /// 정렬 방식
    pub alignment: Alignment,
    /// 줄간격 값 (px 또는 비율)
    pub line_spacing: f64,
    /// 줄간격 종류
    pub line_spacing_type: LineSpacingType,
    /// 왼쪽 여백 (px)
    pub margin_left: f64,
    /// 오른쪽 여백 (px)
    pub margin_right: f64,
    /// 들여쓰기 (px)
    pub indent: f64,
    /// 문단 간격 위 (px)
    pub spacing_before: f64,
    /// 문단 간격 아래 (px)
    pub spacing_after: f64,
    /// 문단 머리 모양 종류
    pub head_type: HeadType,
    /// 문단 수준 (0~6)
    pub para_level: u8,
    /// 번호/글머리표 ID 참조
    pub numbering_id: u16,
    /// 테두리/배경 ID 참조 (0이면 없음)
    pub border_fill_id: u16,
    /// 테두리 안쪽 간격 (좌, 우, 상, 하) (px)
    pub border_spacing: [f64; 4],
    /// 기본 탭 간격 (px)
    pub default_tab_width: f64,
    /// 커스텀 탭 정지 목록 (position 오름차순)
    pub tab_stops: Vec<TabStop>,
    /// 문단 오른쪽 끝 자동 탭 여부
    pub auto_tab_right: bool,
    /// 줄 나눔 기준 영어 단위 (0=단어, 1=하이픈, 2=글자) — attr1 bit 5-6
    pub english_break_unit: u8,
    /// 줄 나눔 기준 한글 단위 (0=어절, 1=글자) — attr1 bit 7
    pub korean_break_unit: u8,
    /// 외톨이줄 보호 — attr1 bit 16
    pub widow_orphan: bool,
    /// 다음 문단과 함께 — attr1 bit 17
    pub keep_with_next: bool,
    /// 분단금지 — attr1 bit 18
    pub keep_lines: bool,
    /// 문단 앞에서 항상 쪽 나눔 — attr1 bit 19
    pub page_break_before: bool,
}

impl Default for ResolvedParaStyle {
    fn default() -> Self {
        Self {
            alignment: Alignment::Justify,
            line_spacing: 160.0, // 기본 160%
            line_spacing_type: LineSpacingType::Percent,
            margin_left: 0.0,
            margin_right: 0.0,
            indent: 0.0,
            spacing_before: 0.0,
            spacing_after: 0.0,
            head_type: HeadType::None,
            para_level: 0,
            numbering_id: 0,
            border_fill_id: 0,
            border_spacing: [0.0; 4],
            default_tab_width: 0.0,
            tab_stops: Vec::new(),
            auto_tab_right: false,
            english_break_unit: 0,
            korean_break_unit: 0,
            widow_orphan: false,
            keep_with_next: false,
            keep_lines: false,
            page_break_before: false,
        }
    }
}

/// 해소된 테두리/배경 스타일 (BorderFill → 렌더링용)
#[derive(Debug, Clone)]
pub struct ResolvedBorderStyle {
    /// 4방향 테두리선 (좌, 우, 상, 하)
    pub borders: [BorderLine; 4],
    /// 배경 채우기 색상 (None이면 채우기 없음)
    pub fill_color: Option<ColorRef>,
    /// 패턴 채우기 (pattern_type > 0일 때)
    pub pattern: Option<PatternFillInfo>,
    /// 그라데이션 채우기 (fill_color보다 우선)
    pub gradient: Option<Box<GradientFillInfo>>,
    /// 이미지 채우기 (gradient/fill_color보다 우선)
    pub image_fill: Option<ResolvedImageFill>,
    /// 대각선 속성 비트 (BorderFill.attr)
    pub diagonal_attr: u16,
    /// 대각선 정보
    pub diagonal: DiagonalLine,
}

/// 해소된 이미지 채우기 정보
#[derive(Debug, Clone)]
pub struct ResolvedImageFill {
    /// BinData ID 참조
    pub bin_data_id: u16,
    /// 이미지 채우기 모드
    pub fill_mode: ImageFillMode,
}

impl Default for ResolvedBorderStyle {
    fn default() -> Self {
        Self {
            borders: [BorderLine::default(); 4],
            fill_color: None,
            pattern: None,
            gradient: None,
            image_fill: None,
            diagonal_attr: 0,
            diagonal: DiagonalLine::default(),
        }
    }
}

/// 해소된 스타일 세트 (DocInfo에서 변환)
#[derive(Debug, Default)]
pub struct ResolvedStyleSet {
    /// 글자 스타일 목록 (char_shapes[id]에 대응)
    pub char_styles: Vec<ResolvedCharStyle>,
    /// 문단 스타일 목록 (para_shapes[id]에 대응)
    pub para_styles: Vec<ResolvedParaStyle>,
    /// 테두리/배경 스타일 목록 (border_fills[id]에 대응)
    pub border_styles: Vec<ResolvedBorderStyle>,
    /// 문단 번호 정의 목록 (numberings[id]에 대응)
    pub numberings: Vec<Numbering>,
    /// 글머리표 정의 목록 (bullets[id]에 대응)
    pub bullets: Vec<Bullet>,
}

/// DocInfo 참조 테이블을 해소된 스타일 목록으로 변환한다.
pub fn resolve_styles(doc_info: &DocInfo, dpi: f64) -> ResolvedStyleSet {
    let char_styles = resolve_char_styles(doc_info, dpi);
    let para_styles = resolve_para_styles(doc_info, dpi);
    let border_styles = resolve_border_styles(doc_info);
    let numberings = doc_info.numberings.clone();
    let bullets = doc_info.bullets.clone();

    ResolvedStyleSet {
        char_styles,
        para_styles,
        border_styles,
        numberings,
        bullets,
    }
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
fn resolve_single_char_style(
    cs: &CharShape,
    doc_info: &DocInfo,
    dpi: f64,
) -> ResolvedCharStyle {
    // base_size는 HWPUNIT 단위
    let font_size = hwpunit_to_px(cs.base_size, dpi);

    // 7개 언어 카테고리별 폰트 이름, 자간, 장평 해소
    let mut font_families = Vec::with_capacity(LANG_COUNT);
    let mut letter_spacings = Vec::with_capacity(LANG_COUNT);
    let mut ratios = Vec::with_capacity(LANG_COUNT);

    for lang in 0..LANG_COUNT {
        let font_id = cs.font_ids[lang];
        font_families.push(lookup_font_name(doc_info, lang, font_id));

        let spacing_percent = cs.spacings[lang] as f64;
        letter_spacings.push(font_size * spacing_percent / 100.0);

        ratios.push(cs.ratios[lang] as f64 / 100.0);
    }

    // 한국어(0번) 값을 기본값으로 사용
    let font_family = font_families[0].clone();
    let letter_spacing = letter_spacings[0];
    let ratio = ratios[0];

    ResolvedCharStyle {
        font_family,
        font_families,
        font_size,
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

/// Unicode 코드포인트로 HWP 언어 카테고리를 판별한다.
///
/// 반환값: 0=한국어, 1=영어(라틴), 2=한자, 3=일본어, 4=기타, 5=기호, 6=사용자
///
/// 공백/일반 구두점은 언어 중립으로 간주하여 기본값(한국어)을 반환한다.
/// 호출부에서 "이전 문자의 언어를 따르는" 로직을 별도 처리해야 한다.
pub fn detect_lang_category(ch: char) -> usize {
    let cp = ch as u32;
    match cp {
        // 한국어: Hangul Jamo, Compatibility Jamo, Syllables
        0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7AF |
        // Hangul Jamo Extended-A/B
        0xA960..=0xA97F | 0xD7B0..=0xD7FF => 0,

        // 영어/라틴: Basic Latin letters+digits, Latin Extended
        0x0041..=0x005A | 0x0061..=0x007A | 0x0030..=0x0039 |
        0x00C0..=0x024F |
        // Latin Extended Additional, Extended-B (subset)
        0x1E00..=0x1EFF => 1,

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
fn lookup_font_name(doc_info: &DocInfo, lang_index: usize, font_id: u16) -> String {
    if lang_index < doc_info.font_faces.len() {
        let lang_fonts = &doc_info.font_faces[lang_index];
        if (font_id as usize) < lang_fonts.len() {
            let font = &lang_fonts[font_id as usize];
            let name = &font.name;
            // 폰트 치환: HFT 등 웹 미지원 폰트를 렌더링 가능한 폰트로 완전 대체
            if let Some(resolved) = resolve_font_substitution(name, font.alt_type, lang_index) {
                return resolved.to_string();
            }
            return name.clone();
        }
    }
    String::new()
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
pub(crate) fn resolve_font_substitution(name: &str, alt_type: u8, lang_index: usize) -> Option<&'static str> {
    // HFT(type=2) 폰트 치환
    if alt_type == 2 {
        if let Some(result) = resolve_hft_font(name, lang_index) {
            return Some(result);
        }
    }

    // TTF(type=1) 또는 알수없음(type=0) 치환
    resolve_ttf_font(name)
}

/// HFT 폰트 → @font-face 등록 폰트 치환 (언어별)
///
/// 한국어(0)와 영어(1)가 다른 결과를 가지는 폰트는 언어별 분기 처리.
/// 대부분의 HFT 폰트는 언어에 무관하게 동일한 결과를 갖는다.
fn resolve_hft_font(name: &str, lang_index: usize) -> Option<&'static str> {
    // === 직접 TTF 매핑 (모든 언어 공통) ===
    let common = match name {
        "한양중고딕" => Some("HY중고딕"),
        "한양신명조" => Some("HY신명조"),
        "한양견명조" => Some("HY견명조"),
        "한양견고딕" => Some("HY견고딕"),
        "한양그래픽" => Some("굴림"),
        "한양궁서" => Some("궁서"),
        "신명 태고딕" => Some("HY중고딕"),
        "신명 태명조" => Some("HY신명조"),
        "신명 견고딕" => Some("HY견고딕"),
        "신명 견명조" => Some("HY견명조"),
        "신명 태그래픽" => Some("HY그래픽"),
        "신명 중고딕" => Some("HY중고딕"),
        "태 가는 헤드라인T" => Some("HY헤드라인M"),
        "태 가는 헤드라인D" => Some("HY헤드라인M"),
        "양재 튼튼B" => Some("양재튼튼체B"),
        // 명조 계열 → HY견명조
        "명조" => Some("HY견명조"),
        // 체인 평탄화: 다단계 HFT→HFT→...→TTF 체인의 최종 결과
        "휴먼명조" => Some("HY신명조"),
        "문화바탕" | "문화바탕제목" | "문화쓰기" | "문화쓰기흘림" => Some("HY신명조"),
        "신명 세명조" | "신명 신명조" | "신명 신신명조" | "신명 중명조"
        | "신명 순명조" | "신명 신문명조" => Some("HY신명조"),
        "옛한글" | "양재 다운명조M" => Some("HY신명조"),
        "#세명조" | "#신명조" | "#중명조" | "#신중명조"
        | "#화명조A" | "#화명조B" | "#태명조" | "#신태명조" | "#태신명조"
        | "#견명조" | "#신문명조" | "#신문태명" => Some("HY신명조"),
        // 고딕 계열
        "휴먼고딕" | "문화돋움" | "문화돋움제목" | "태 나무" => Some("돋움"),
        "휴먼옛체" | "딸기" => Some("돋움"),
        "샘물" | "가는한" | "중간한" | "굵은한" => Some("돋움"),
        "휴먼가는샘체" | "휴먼중간샘체" | "휴먼굵은샘체" => Some("돋움"),
        "휴먼가는팸체" | "휴먼중간팸체" | "휴먼굵은팸체" => Some("돋움"),
        "가는안상수체" | "중간안상수체" | "굵은안상수체" => Some("돋움"),
        "양재 매화" | "양재 소슬" | "양재 샤넬" | "옥수수" => Some("돋움"),
        "양재 본목각M" | "복숭아" => Some("돋움"),
        "신명 세고딕" | "신명 디나루" | "신명 세나루" => Some("돋움"),
        "#세고딕" | "#신세고딕" | "#중고딕" | "#태고딕"
        | "#신문고딕" | "#신문태고" | "#세나루" | "#신세나루"
        | "#디나루" | "#신디나루" => Some("돋움"),
        // 그래픽/궁서/기타
        "신명 신그래픽" | "강낭콩" => Some("굴림"),
        "#그래픽" | "#신그래픽" | "#공작" => Some("굴림"),
        "양재 참숯B" | "양재 와당" | "양재 이니셜" => Some("HY견고딕"),
        "#빅" => Some("HY견고딕"),
        "태 헤드라인T" => Some("HY견고딕"),
        "태 헤드라인D" => Some("HY견명조"),
        "가는공한" | "중간공한" | "굵은공한" | "필기" | "타이프" => Some("HY견명조"),
        "가지" | "오이" | "양재 둘기" => Some("HY견명조"),
        "신명 궁서" | "#궁서" => Some("궁서"),
        "#수암A" | "#수암B" => Some("돋움"),
        // 시스템
        "시스템" | "HY둥근고딕" => Some("돋움"),
        "고딕" => Some("돋움"),
        // 영문 HFT
        "산세리프" => Some("Calibri"),
        "HCI Poppy" => Some("Palatino Linotype"),
        "수식" => Some("HY신명조"),
        "한글 풀어쓰기" => Some("HY견명조"),
        _ => None,
    };

    if common.is_some() {
        return common;
    }

    // 영어(1) 전용 HFT 치환
    if lang_index == 1 {
        match name {
            "HCI Tulip" | "HCI Morning Glory" | "HCI Centaurea"
            | "HCI Bellflower" | "AmeriGarmnd BT" | "Bodoni Bd BT"
            | "Bodoni Bk BT" | "Baskerville BT" | "GoudyOlSt BT"
            | "Cooper Blk BT" | "Stencil BT" | "BrushScript BT"
            | "CommercialScript BT" | "Liberty BT" | "MurrayHill Bd BT"
            | "ParkAvenue BT" | "CentSchbook BT" | "펜흘림" => Some("HY견명조"),
            "HCI Hollyhock" | "HCI Hollyhock Narrow" | "HCI Acacia"
            | "Swis721 BT" | "Hobo BT" | "Orbit-B BT"
            | "Blippo Blk BT" | "BroadwayEngraved BT"
            | "FuturaBlack BT" | "Newtext Bk BT" | "DomCasual BT"
            | "가는안상수체영문" | "중간안상수체영문" | "굵은안상수체영문" => Some("HY중고딕"),
            "HCI Columbine" | "Courier10 BT" | "OCR-A BT"
            | "OCR-B-10 BT" | "Orator10 BT" => Some("Calibri"),
            "BernhardFashion BT" | "Freehand591 BT" => Some("HY중고딕"),
            _ => None,
        }
    } else {
        None
    }
}

/// TTF 폰트 → @font-face 등록 폰트 치환 (모든 언어 공통)
fn resolve_ttf_font(name: &str) -> Option<&'static str> {
    match name {
        // 영문 별칭
        "Gulim" => Some("굴림"),
        "HYHeadLine Medium" => Some("HY헤드라인M"),
        "Malgun Gothic" => Some("맑은 고딕"),
        "HY그래픽M" => Some("HY그래픽"),
        "SPOQAHANSANS" => Some("SpoqaHanSans"),
        // 한컴 체인: 한컴바탕→함초롬바탕, 한컴돋움→함초롬돋움
        "한컴바탕" => Some("함초롬바탕"),
        "한컴돋움" => Some("함초롬돋움"),
        // 영어(1) 전용 TTF 치환 (webhwp lang=1)
        "MS Sans Serif" => Some("함초롬돋움"),
        "Tahoma" => Some("함초롬돋움"),
        // "Times New Roman" — 메트릭 DB에 있으므로 치환하지 않음
        // 백묵 계열
        "백묵 굴림" => Some("굴림"),
        "백묵 돋움" => Some("돋움"),
        "백묵 바탕" => Some("바탕"),
        "백묵 헤드라인" => Some("돋움"),
        // Gulimche (lang=6)
        "Gulimche" => Some("돋움"),
        // 새~ 계열 → 함초롬 (TS 체인 최종 결과 평탄화)
        "새바탕" => Some("함초롬바탕"),
        "새돋움" => Some("함초롬돋움"),
        "새굴림" => Some("함초롬돋움"),
        "새궁서" => Some("함초롬바탕"),
        // 맑은 고딕: 웹폰트(@font-face)로 등록되어 있으므로 치환하지 않음
        // 안상수체 TTF 타입
        "가는안상수체" => Some("돋움"),
        "중간안상수체" => Some("돋움"),
        "굵은안상수체" => Some("돋움"),
        _ => None,
    }
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
    let primary = font_family.split(',').next().unwrap_or(font_family)
        .trim()
        .trim_matches('\'')
        .trim_matches('"');
    matches!(primary,
        "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
        | "HY견고딕" | "HY견명조B"
        | "HY그래픽" | "HY그래픽M"
    )
}

/// ParaShape → ResolvedParaStyle 목록
fn resolve_para_styles(doc_info: &DocInfo, dpi: f64) -> Vec<ResolvedParaStyle> {
    doc_info
        .para_shapes
        .iter()
        .map(|ps| resolve_single_para_style(ps, &doc_info.tab_defs, dpi))
        .collect()
}

/// 개별 ParaShape 해소
fn resolve_single_para_style(ps: &ParaShape, tab_defs: &[TabDef], dpi: f64) -> ResolvedParaStyle {
    let line_spacing = match ps.line_spacing_type {
        LineSpacingType::Percent => ps.line_spacing as f64,
        _ => hwpunit_to_px(ps.line_spacing, dpi),
    };

    // 기본 탭 간격: HWP 기본값 80pt (8000 HWPUNIT)
    let default_tab_width = hwpunit_to_px(4000, dpi);

    // 커스텀 탭 정지 해소: TabDef.tabs[] → px 변환
    // TabItem.position은 ParaShape 여백과 동일하게 2배 스케일로 저장되므로
    // 렌더링 시 2로 나누어야 한다 (hwp2hwpx 변환 코드 및 HWP 대화상자 확인).
    let tab_def = tab_defs.get(ps.tab_def_id as usize);
    let tab_stops: Vec<TabStop> = tab_def
        .map(|td| td.tabs.iter().map(|t| TabStop {
            position: hwpunit_to_px(t.position as i32, dpi) / 2.0, // HWP 탭 position은 실제 좌표의 2배로 저장됨 (한컴 격자 비교로 확인)
            tab_type: t.tab_type,
            fill_type: t.fill_type,
        }).collect())
        .unwrap_or_default();
    let auto_tab_right = tab_def.map(|td| td.auto_tab_right).unwrap_or(false);

    // ParaShape의 여백 및 문단 간격은 HWPUNIT의 2배 값으로 저장된다.
    // margin_left/right/indent: LineSeg.column_start와 비교하면 column_start = margin_left / 2
    // spacing_before/after: pyhwpx 확인 결과 동일하게 2배 스케일 저장
    // 실제 렌더링 시 2로 나누어야 올바른 값이 된다.
    ResolvedParaStyle {
        alignment: ps.alignment,
        line_spacing,
        line_spacing_type: ps.line_spacing_type,
        margin_left: hwpunit_to_px(ps.margin_left, dpi) / 2.0,
        margin_right: hwpunit_to_px(ps.margin_right, dpi) / 2.0,
        indent: hwpunit_to_px(ps.indent, dpi) / 2.0,
        spacing_before: hwpunit_to_px(ps.spacing_before, dpi) / 2.0,
        spacing_after: hwpunit_to_px(ps.spacing_after, dpi) / 2.0,
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
        default_tab_width,
        tab_stops,
        auto_tab_right,
        english_break_unit: ((ps.attr1 >> 5) & 0x03) as u8,
        korean_break_unit: ((ps.attr1 >> 7) & 0x01) as u8,
        widow_orphan: (ps.attr1 >> 16) & 1 != 0 || (ps.attr2 >> 5) & 1 != 0,
        keep_with_next: (ps.attr1 >> 17) & 1 != 0 || (ps.attr2 >> 6) & 1 != 0,
        keep_lines: (ps.attr1 >> 18) & 1 != 0 || (ps.attr2 >> 7) & 1 != 0,
        page_break_before: (ps.attr1 >> 19) & 1 != 0 || (ps.attr2 >> 8) & 1 != 0,
    }
}

/// BorderFill → ResolvedBorderStyle 목록
fn resolve_border_styles(doc_info: &DocInfo) -> Vec<ResolvedBorderStyle> {
    doc_info
        .border_fills
        .iter()
        .map(resolve_single_border_style)
        .collect()
}

/// 개별 BorderFill 해소
fn resolve_single_border_style(bf: &BorderFill) -> ResolvedBorderStyle {
    let fill_color = match bf.fill.fill_type {
        FillType::Solid => bf.fill.solid.as_ref().and_then(|s| {
            // ColorRef 상위 바이트가 0이 아니면 "채우기 없음" (투명)
            // 0xFFFFFFFF = CLR_INVALID/CLR_DEFAULT (Windows COLORREF)
            if (s.background_color >> 24) != 0 {
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
                (0..n).map(|i| i as f64 / (n.max(2) - 1).max(1) as f64).collect()
            } else {
                g.positions.iter().map(|&p| p as f64 / 100.0).collect()
            };
            Some(Box::new(GradientFillInfo {
                gradient_type: g.gradient_type,
                angle: g.angle,
                center_x: g.center_x,
                center_y: g.center_y,
                colors: g.colors.clone(),
                positions,
            }))
        }),
        _ => None,
    };

    let image_fill = match bf.fill.fill_type {
        FillType::Image => bf.fill.image.as_ref().map(|img| {
            ResolvedImageFill {
                bin_data_id: img.bin_data_id,
                fill_mode: img.fill_mode,
            }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::DocInfo;
    use crate::model::style::*;
    use crate::renderer::DEFAULT_DPI;

    fn make_doc_info_with_font() -> DocInfo {
        DocInfo {
            font_faces: vec![
                // 한글(lang=0) 폰트
                vec![
                    Font {
                        name: "함초롬돋움".to_string(),
                        ..Default::default()
                    },
                    Font {
                        name: "함초롬바탕".to_string(),
                        ..Default::default()
                    },
                ],
            ],
            char_shapes: vec![
                CharShape {
                    font_ids: [0, 0, 0, 0, 0, 0, 0], // 함초롬돋움
                    base_size: 2400,                     // 24pt = 2400 HWPUNIT (1pt = 100 HWPUNIT)
                    bold: true,
                    italic: false,
                    text_color: 0x00000000, // 검정
                    ratios: [100, 100, 100, 100, 100, 100, 100],
                    spacings: [0, 0, 0, 0, 0, 0, 0],
                    ..Default::default()
                },
                CharShape {
                    font_ids: [1, 1, 1, 1, 1, 1, 1], // 함초롬바탕
                    base_size: 1000,                     // 10pt
                    bold: false,
                    italic: true,
                    text_color: 0x00FF0000, // 파란색 (BGR)
                    ratios: [80, 80, 80, 80, 80, 80, 80],
                    spacings: [-5, -5, -5, -5, -5, -5, -5],
                    underline_type: UnderlineType::Bottom,
                    underline_color: 0x00000000,
                    ..Default::default()
                },
            ],
            para_shapes: vec![
                ParaShape {
                    alignment: Alignment::Center,
                    line_spacing: 160,
                    line_spacing_type: LineSpacingType::Percent,
                    margin_left: 0,
                    margin_right: 0,
                    indent: 0,
                    spacing_before: 0,
                    spacing_after: 400, // 400 HWPUNIT
                    ..Default::default()
                },
                ParaShape {
                    alignment: Alignment::Justify,
                    line_spacing: 1200, // 1200 HWPUNIT (고정)
                    line_spacing_type: LineSpacingType::Fixed,
                    margin_left: 1000,
                    margin_right: 500,
                    indent: 800,
                    spacing_before: 200,
                    spacing_after: 200,
                    ..Default::default()
                },
            ],
            border_fills: vec![
                BorderFill {
                    borders: [
                        BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 },
                        BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 },
                        BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 },
                        BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 },
                    ],
                    fill: Fill {
                        fill_type: FillType::Solid,
                        solid: Some(SolidFill {
                            background_color: 0x00FFFFFF,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_resolve_char_style_font_name() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert_eq!(styles.char_styles.len(), 2);
        assert_eq!(styles.char_styles[0].font_family, "함초롬돋움");
        assert_eq!(styles.char_styles[1].font_family, "함초롬바탕");
    }

    #[test]
    fn test_resolve_char_style_size() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        // 2400 HWPUNIT * 96 / 7200 = 32.0 px
        let expected_24pt = 2400.0 * DEFAULT_DPI / 7200.0;
        assert!((styles.char_styles[0].font_size - expected_24pt).abs() < 0.01);

        // 1000 HWPUNIT * 96 / 7200 ≈ 13.33 px
        let expected_10pt = 1000.0 * DEFAULT_DPI / 7200.0;
        assert!((styles.char_styles[1].font_size - expected_10pt).abs() < 0.01);
    }

    #[test]
    fn test_resolve_char_style_bold_italic() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert!(styles.char_styles[0].bold);
        assert!(!styles.char_styles[0].italic);
        assert!(!styles.char_styles[1].bold);
        assert!(styles.char_styles[1].italic);
    }

    #[test]
    fn test_resolve_char_style_color() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert_eq!(styles.char_styles[0].text_color, 0x00000000);
        assert_eq!(styles.char_styles[1].text_color, 0x00FF0000);
    }

    #[test]
    fn test_resolve_char_style_underline() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert_eq!(styles.char_styles[0].underline, UnderlineType::None);
        assert_eq!(styles.char_styles[1].underline, UnderlineType::Bottom);
    }

    #[test]
    fn test_resolve_char_style_ratio() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert!((styles.char_styles[0].ratio - 1.0).abs() < 0.01);
        assert!((styles.char_styles[1].ratio - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_resolve_char_style_letter_spacing() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        // 첫 번째: spacing=0 → 0.0 px
        assert!((styles.char_styles[0].letter_spacing - 0.0).abs() < 0.01);

        // 두 번째: spacing=-5, font_size ≈ 13.33 → -5% * 13.33 ≈ -0.67
        let expected = styles.char_styles[1].font_size * -5.0 / 100.0;
        assert!((styles.char_styles[1].letter_spacing - expected).abs() < 0.01);
    }

    #[test]
    fn test_resolve_para_style_alignment() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert_eq!(styles.para_styles.len(), 2);
        assert_eq!(styles.para_styles[0].alignment, Alignment::Center);
        assert_eq!(styles.para_styles[1].alignment, Alignment::Justify);
    }

    #[test]
    fn test_resolve_para_style_line_spacing() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        // 퍼센트 타입: 그대로 160.0
        assert!((styles.para_styles[0].line_spacing - 160.0).abs() < 0.01);
        assert_eq!(styles.para_styles[0].line_spacing_type, LineSpacingType::Percent);

        // 고정 타입: 1200 HWPUNIT → px 변환
        let expected = hwpunit_to_px(1200, DEFAULT_DPI);
        assert!((styles.para_styles[1].line_spacing - expected).abs() < 0.01);
        assert_eq!(styles.para_styles[1].line_spacing_type, LineSpacingType::Fixed);
    }

    #[test]
    fn test_resolve_para_style_margins() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        // ParaShape의 여백은 2배 값으로 저장되므로 resolve 시 2로 나눈다
        let margin_left = hwpunit_to_px(1000, DEFAULT_DPI) / 2.0;
        let margin_right = hwpunit_to_px(500, DEFAULT_DPI) / 2.0;
        let indent = hwpunit_to_px(800, DEFAULT_DPI) / 2.0;

        assert!((styles.para_styles[1].margin_left - margin_left).abs() < 0.01);
        assert!((styles.para_styles[1].margin_right - margin_right).abs() < 0.01);
        assert!((styles.para_styles[1].indent - indent).abs() < 0.01);
    }

    #[test]
    fn test_resolve_border_style() {
        let doc_info = make_doc_info_with_font();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert_eq!(styles.border_styles.len(), 1);
        assert_eq!(styles.border_styles[0].fill_color, Some(0x00FFFFFF));
        assert_eq!(styles.border_styles[0].borders[0].line_type, BorderLineType::Solid);
    }

    #[test]
    fn test_resolve_empty_doc_info() {
        let doc_info = DocInfo::default();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        assert!(styles.char_styles.is_empty());
        assert!(styles.para_styles.is_empty());
        assert!(styles.border_styles.is_empty());
    }

    #[test]
    fn test_lookup_font_missing() {
        let doc_info = DocInfo::default();
        let name = lookup_font_name(&doc_info, 0, 0);
        assert!(name.is_empty());
    }

    #[test]
    fn test_resolve_border_no_fill() {
        let doc_info = DocInfo {
            border_fills: vec![BorderFill::default()],
            ..Default::default()
        };
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);
        assert_eq!(styles.border_styles[0].fill_color, None);
    }

    // === 언어 판별 테스트 ===

    #[test]
    fn test_detect_lang_category_korean() {
        assert_eq!(detect_lang_category('가'), 0);
        assert_eq!(detect_lang_category('힣'), 0);
        assert_eq!(detect_lang_category('ㄱ'), 0); // Compatibility Jamo
        assert_eq!(detect_lang_category('ㅎ'), 0);
    }

    #[test]
    fn test_detect_lang_category_english() {
        assert_eq!(detect_lang_category('A'), 1);
        assert_eq!(detect_lang_category('z'), 1);
        assert_eq!(detect_lang_category('0'), 1);
        assert_eq!(detect_lang_category('9'), 1);
        assert_eq!(detect_lang_category('é'), 1); // Latin Extended
    }

    #[test]
    fn test_detect_lang_category_cjk() {
        assert_eq!(detect_lang_category('中'), 2);
        assert_eq!(detect_lang_category('漢'), 2);
    }

    #[test]
    fn test_detect_lang_category_japanese() {
        assert_eq!(detect_lang_category('あ'), 3); // Hiragana
        assert_eq!(detect_lang_category('ア'), 3); // Katakana
    }

    #[test]
    fn test_detect_lang_category_symbol() {
        assert_eq!(detect_lang_category('→'), 5); // 화살표
        assert_eq!(detect_lang_category('★'), 5); // 도형
        assert_eq!(detect_lang_category('①'), 5); // 원숫자
    }

    #[test]
    fn test_detect_lang_category_default() {
        // 공백, 구두점 등은 기본값(한국어=0)
        assert_eq!(detect_lang_category(' '), 0);
        assert_eq!(detect_lang_category('.'), 0);
        assert_eq!(detect_lang_category(','), 0);
    }

    // === 언어별 폰트 해소 테스트 ===

    fn make_doc_info_with_multilang_fonts() -> DocInfo {
        DocInfo {
            font_faces: vec![
                // lang=0 (한국어)
                vec![
                    Font { name: "함초롬돋움".to_string(), ..Default::default() },
                ],
                // lang=1 (영어)
                vec![
                    Font { name: "Arial".to_string(), ..Default::default() },
                ],
                // lang=2 (한자)
                vec![
                    Font { name: "SimSun".to_string(), ..Default::default() },
                ],
                // lang=3~6 (나머지) - 비어있을 수 있음
            ],
            char_shapes: vec![
                CharShape {
                    font_ids: [0, 0, 0, 0, 0, 0, 0], // 모든 언어에서 0번 폰트
                    base_size: 1000,
                    ratios: [100, 80, 90, 100, 100, 100, 100],
                    spacings: [0, -5, 0, 0, 0, 0, 0],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_resolve_char_style_font_families() {
        let doc_info = make_doc_info_with_multilang_fonts();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        let cs = &styles.char_styles[0];
        assert_eq!(cs.font_families.len(), 7);
        assert_eq!(cs.font_families[0], "함초롬돋움"); // 한국어
        assert_eq!(cs.font_families[1], "Arial");       // 영어
        assert_eq!(cs.font_families[2], "SimSun");       // 한자
        assert_eq!(cs.font_families[3], "");             // 일본어 (없음)
        assert_eq!(cs.font_family, "함초롬돋움");        // 기본값 = 한국어
    }

    #[test]
    fn test_resolve_char_style_lang_ratios() {
        let doc_info = make_doc_info_with_multilang_fonts();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        let cs = &styles.char_styles[0];
        assert!((cs.ratios[0] - 1.0).abs() < 0.01);   // 한국어 100%
        assert!((cs.ratios[1] - 0.8).abs() < 0.01);   // 영어 80%
        assert!((cs.ratios[2] - 0.9).abs() < 0.01);   // 한자 90%
        assert!((cs.ratio - 1.0).abs() < 0.01);        // 기본값 = 한국어
    }

    #[test]
    fn test_resolve_char_style_lang_spacings() {
        let doc_info = make_doc_info_with_multilang_fonts();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        let cs = &styles.char_styles[0];
        assert!((cs.letter_spacings[0] - 0.0).abs() < 0.01); // 한국어 spacing=0
        let expected_en = cs.font_size * -5.0 / 100.0;
        assert!((cs.letter_spacings[1] - expected_en).abs() < 0.01); // 영어 spacing=-5
    }

    // === TTF 폰트 치환 보완 테스트 ===

    #[test]
    fn test_resolve_ttf_new_fonts() {
        assert_eq!(resolve_ttf_font("새바탕"), Some("함초롬바탕"));
        assert_eq!(resolve_ttf_font("새돋움"), Some("함초롬돋움"));
        assert_eq!(resolve_ttf_font("새굴림"), Some("함초롬돋움"));
        assert_eq!(resolve_ttf_font("새궁서"), Some("함초롬바탕"));
    }

    #[test]
    fn test_resolve_ttf_malgun_gothic() {
        // 맑은 고딕은 웹폰트로 등록되어 있으므로 치환하지 않음
        assert_eq!(resolve_ttf_font("맑은 고딕"), None);
    }

    #[test]
    fn test_resolve_ttf_ansangsu() {
        assert_eq!(resolve_ttf_font("가는안상수체"), Some("돋움"));
        assert_eq!(resolve_ttf_font("중간안상수체"), Some("돋움"));
        assert_eq!(resolve_ttf_font("굵은안상수체"), Some("돋움"));
    }

    #[test]
    fn test_font_family_for_lang_fallback() {
        let doc_info = make_doc_info_with_multilang_fonts();
        let styles = resolve_styles(&doc_info, DEFAULT_DPI);

        let cs = &styles.char_styles[0];
        assert_eq!(cs.font_family_for_lang(0), "함초롬돋움");
        assert_eq!(cs.font_family_for_lang(1), "Arial");
        assert_eq!(cs.font_family_for_lang(3), "함초롬돋움"); // 빈 문자열 → 한국어 폴백
        assert_eq!(cs.font_family_for_lang(99), "함초롬돋움"); // 범위 초과 → 한국어 폴백
    }
}
