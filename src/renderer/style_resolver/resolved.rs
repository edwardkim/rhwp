//! 해소된 스타일의 타입과 언어 슬롯별 조회.

use super::*;

/// 해소된 글자 스타일 (CharShape + FontFace → 렌더링용)
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCharStyle {
    /// 글꼴 이름 (한국어 = 기본값, font_families[0]과 동일)
    pub font_family: String,
    /// 7개 언어 카테고리별 글꼴 이름
    pub font_families: Vec<String>,
    /// [#7092] 언어별로 메트릭 표를 **그 글꼴 자신의 폭**으로 믿을 수 있는지
    /// (`font_families` 와 같은 순서).
    ///
    /// 참은 [`metric_widths_verified_face`] 가 인정한 face 가 TTF 로 선언되고 대체 규칙이
    /// 이름을 바꾸지 않았거나 확인된 TrueType 프로그램을 명시적 환경으로 선택한 때다.
    /// HFT 는 한/글이 자기 글리프로 그리고, 대체된 이름은
    /// 다른 글꼴의 표를 빌려 오므로 표에 적힌 폭이 그 글꼴의 폭이라는 보장이 없다.
    pub font_families_metric_trusted: Vec<bool>,
    /// 명시한 glyphFallbacks의 전진 선택. 원 face 전체의 신뢰와 구분한다.
    pub font_families_glyph_fallback_advance_explicit: Vec<bool>,
    /// [#7051] 언어 슬롯별로 선언 글꼴이 **HFT 한글 전용 face** 여서 치환됐는지.
    /// 그런 글꼴의 ASCII 는 한컴이 반각으로 전진시킨다(측정 전용).
    pub font_families_hft_hangul: Vec<bool>,
    /// 대체 여부와 무관하게 보존하는 원본 프로그램 종류다.
    pub font_families_source_hft: Vec<bool>,
    pub font_families_hft_print_frame: Vec<bool>,
    /// 세션이 명시적으로 선택한 프로그램의 자연 윤곽 폭을 보존한다.
    pub font_families_preserve_glyph_ink: Vec<bool>,
    /// 언어 슬롯이 대체되지 않은 원본 HFT 기호 전각 폭을 갖는지.
    pub font_families_hft_fullwidth_dot: Vec<bool>,
    /// [#7418] 한/글이 한글 뒤 ASCII 구두점도 **영문 슬롯** 글꼴로 재고 그리는가.
    ///
    /// 영문 슬롯이 한컴 옛 영문 글꼴(`LegacyLatin` 치환)이고 그 치환이 **한글 글리프가 없는
    /// 라틴 전용 글꼴**일 때 참이다 — 옛 서식의 휴먼명조 + `HCI Poppy`(→ Palatino Linotype).
    /// 정본 PDF 에서 그 줄의 한글 뒤 `-`·`,`·`(` 은 Palatino 로 그려진다(휴먼명조 줄 99%).
    ///
    /// 치환이 한글 글꼴이면(`HCI Hollyhock` → HY중고딕) 그 글꼴이 ASCII 를 가지므로 한/글도
    /// 한글 글꼴 그대로 둔다(`hwp3-sample16-hwp5` 정본: 한글 뒤 구두점 41건 모두 같은 글꼴).
    /// 한글 슬롯이 바탕·맑은 고딕처럼 ASCII 를 가진 TTF 인 문서도 대부분 한글 글꼴 그대로다.
    pub ascii_punct_latin_slot: bool,
    /// 명시 환경이 선택한 원본 영문 슬롯의 구두점 소유.
    pub explicit_latin_slot: bool,
    /// [#7391] 언어 슬롯별로, 폭을 **선언 face 자신의 표**로 재야 하는 경우의 그 이름.
    ///
    /// legacy-latin 치환은 표시할 글꼴이 없는 환경의 폴백이라 라틴 face 를 한글 face 로
    /// 보낸다(`AmeriGarmnd BT` → `HY견명조`). 표시로는 뜻이 있지만 **폭은 범주가 다르다** —
    /// 한글 명조의 라틴 글리프 폭이 BT 계열 라틴 글꼴의 폭일 리 없다.
    ///
    /// rhwp 가 선언 face 자신의 메트릭 표를 이미 갖고 있으면 그 표를 버릴 이유가 없다.
    /// `1341000_research_report_footnotes` 정본은 `AmeriGarmnd BT` 를 **그 글꼴 자신으로**
    /// 그렸고(ASCII 13,919자), 그 전진폭을 후보 표와 대조하면 값이 갈린다:
    ///
    /// ```text
    ///   AmeriGarmnd BT 자기 표                     중앙 오차 0.0398 em
    ///   HY견명조 → HYMyeongJo-Extra (현행 치환 대상)   중앙 오차 0.2134 em   (5.4배)
    ///   HYGothic-Medium                           중앙 오차 0.1324 em
    /// ```
    ///
    /// 선언 face 의 표가 없으면 `None` 이라 종전 치환 대상의 표를 그대로 쓴다.
    /// `HCI Poppy` 가 그 경우이며, 그 치환(`Palatino Linotype`)은 정본 ASCII 24,662자
    /// 대조에서 중앙 오차 0.0040 em 으로 이미 맞다 — 건드리지 않는다.
    pub font_families_metric_face: Vec<Option<String>>,
    pub font_families_original_glyph_metric_face: Vec<Option<String>>,
    /// [#7387] 처음에는 `CharShape.use_font_space`가 켜진 run의 **영문 슬롯**(1)
    /// 공백 글리프 폭이다. 저장 문서 프로필에서 독립 출력에 맞게 보정할 수 있다.
    /// `None`이면 일반 글꼴 측정 규칙을 따른다.
    ///
    /// 한/글은 이 속성이 켜지면 공백을 영문 슬롯 글꼴의 제 공백폭으로 전진시킨다.
    /// 근거와 문서 내 대조군은 [`crate::renderer::TextStyle::font_space_em`] 에 있다.
    pub font_space_em: Option<f64>,
    /// 글꼴 크기 (px)
    pub font_size: f64,
    /// 언어별 상대 크기를 적용한 글리프 크기(px). 기본 줄 크기와 구분한다.
    pub font_sizes: Vec<f64>,
    /// 언어별 글자 위치. 글꼴 크기에 대한 부호 있는 백분율이며 줄 원점과 구분한다.
    pub font_positions: Vec<i8>,
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
            font_families_metric_trusted: Vec::new(),
            font_families_glyph_fallback_advance_explicit: Vec::new(),
            font_families_hft_hangul: Vec::new(),
            font_families_source_hft: Vec::new(),
            font_families_hft_print_frame: Vec::new(),
            font_families_preserve_glyph_ink: Vec::new(),
            font_families_hft_fullwidth_dot: Vec::new(),
            ascii_punct_latin_slot: false,
            explicit_latin_slot: false,
            font_families_metric_face: Vec::new(),
            font_families_original_glyph_metric_face: Vec::new(),
            font_space_em: None,
            font_size: 12.0,
            font_sizes: Vec::new(),
            font_positions: Vec::new(),
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

    /// [#7092] 지정 언어 카테고리의 메트릭 표를 그 글꼴 자신의 폭으로 믿을 수 있는지.
    /// `font_family_for_lang` 과 같은 폴백(이름이 비면 한국어 0번)을 따른다.
    pub fn font_metric_trusted_for_lang(&self, lang_index: usize) -> bool {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_metric_trusted
            .get(slot)
            .copied()
            .unwrap_or(false)
    }

    pub fn glyph_fallback_advance_explicit_for_lang(&self, lang_index: usize) -> bool {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_glyph_fallback_advance_explicit
            .get(slot)
            .copied()
            .unwrap_or(false)
    }

    /// [#7051] 지정 언어 카테고리의 글꼴이 HFT 한글 전용 face 라서 치환됐는지.
    /// `font_family_for_lang` 과 같은 폴백(이름이 비면 한국어 0번)을 따른다.
    pub fn hft_hangul_face_for_lang(&self, lang_index: usize) -> bool {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_hft_hangul
            .get(slot)
            .copied()
            .unwrap_or(false)
    }

    /// 표시용 대체 글꼴과 구분해 원본 언어 슬롯의 HFT 선언을 조회한다.
    pub fn source_hft_for_lang(&self, lang_index: usize) -> bool {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_source_hft
            .get(slot)
            .copied()
            .unwrap_or(false)
    }

    pub fn hft_print_frame_for_lang(&self, lang_index: usize) -> bool {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_hft_print_frame
            .get(slot)
            .copied()
            .unwrap_or(false)
    }

    /// 대체되지 않은 HFT 기호 슬롯의 원본 전각 전진을 보존한다.
    pub fn hft_fullwidth_dot_for_lang(&self, lang_index: usize) -> bool {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_hft_fullwidth_dot
            .get(slot)
            .copied()
            .unwrap_or(false)
    }

    /// [#7391] 이 언어 슬롯의 폭을 잴 때 쓸 face. 되돌릴 게 없으면 `None`.
    pub fn metric_face_for_lang(&self, lang_index: usize) -> Option<&str> {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_metric_face
            .get(slot)
            .and_then(Option::as_deref)
    }

    pub fn original_glyph_metric_face_for_lang(&self, lang_index: usize) -> Option<&str> {
        let slot = if lang_index < self.font_families.len()
            && !self.font_families[lang_index].is_empty()
        {
            lang_index
        } else {
            0
        };
        self.font_families_original_glyph_metric_face
            .get(slot)
            .and_then(Option::as_deref)
    }

    /// 문서의 언어별 상대 크기를 측정과 실제 출력에 함께 적용한다.
    pub fn font_size_for_lang(&self, lang_index: usize) -> f64 {
        self.font_sizes
            .get(lang_index)
            .copied()
            .unwrap_or(self.font_size)
    }

    pub fn font_position_for_lang(&self, lang_index: usize) -> i8 {
        self.font_positions
            .get(lang_index)
            .or_else(|| self.font_positions.first())
            .copied()
            .unwrap_or_default()
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
    /// 줄 안 세로 정렬 (ParaShape attr1 bits 20..21): 글꼴 기준/위/가운데/아래.
    pub vertical_alignment: u8,
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
    /// 같은 테두리를 가진 이웃 문단과 연결 (ParaShape attr1 bit 28).
    pub border_connect: bool,
    /// 문단 테두리/배경의 범위 선택 (ParaShape attr1 bit 29, HWPX `ignoreMargin`).
    ///
    /// 한/글 2020 출력 실측: 꺼져 있으면 테두리/배경이 단 너비 전체와 문단 전체 흐름
    /// (앞 간격·줄간격·뒤 간격 포함)을 덮는다. 켜져 있으면 문단 여백 안쪽과 줄 상자
    /// (첫 줄 위 ~ 마지막 줄 상자 아래)만 덮는다.
    pub border_ignore_margin: bool,
    /// 기본 탭 간격 (px)
    pub default_tab_width: f64,
    /// 커스텀 탭 정지 목록 (position 오름차순)
    pub tab_stops: Vec<TabStop>,
    /// 문단 오른쪽 끝 자동 탭 여부
    pub auto_tab_left: bool,
    pub auto_tab_right: bool,
    /// HWPX paraPr condense / HWP ParaShape attr1 bits 9..15.
    /// Spec name: minimum spacing value, 0..75%.
    pub condense_min_space: u8,
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
            vertical_alignment: 0,
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
            border_connect: false,
            border_ignore_margin: false,
            default_tab_width: 0.0,
            tab_stops: Vec::new(),
            auto_tab_left: false,
            auto_tab_right: false,
            condense_min_space: 0,
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
    /// 중심선 방향
    pub center_line: CenterLine,
}

/// 해소된 이미지 채우기 정보
#[derive(Debug, Clone)]
pub struct ResolvedImageFill {
    /// BinData ID 참조
    pub bin_data_id: u16,
    /// 이미지 채우기 모드
    pub fill_mode: ImageFillMode,
    /// [`crate::model::style::ImageFill::brightness`] 그대로 — 이진 순서다(화면 `contrast`).
    pub brightness: i8,
    /// [`crate::model::style::ImageFill::contrast`] 그대로 — 이진 순서다(화면 `bright`).
    pub contrast: i8,
    /// 그림 효과
    pub effect: ImageEffect,
}

impl ResolvedImageFill {
    /// 화면 순서의 `(bright, contrast)`.
    ///
    /// [`crate::model::style::ImageFill::display_brightness_contrast`] 와 같은 계약이다 —
    /// 이 구조체는 `ImageFill` 의 두 필드를 순서 그대로 옮겨 담는다(#6895).
    pub const fn display_brightness_contrast(&self) -> (i8, i8) {
        (self.contrast, self.brightness)
    }
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
            center_line: CenterLine::None,
        }
    }
}

/// 해소된 스타일 세트 (DocInfo에서 변환)
#[derive(Debug, Default, Clone)]
pub struct ResolvedStyleSet {
    /// 문서 지정 대체가 있어도 현재 선택은 자기 메트릭을 가진 원본 TTF인 슬롯.
    /// 값은 문서가 선언한 대체 프로그램의 해소 결과다. 저장 줄 전체가 이 대체
    /// 프로그램의 채움과 일치하고 현재 원본 프로그램과 다른 경우에만 사용한다.
    pub(crate) source_program_projection_slots:
        std::collections::HashMap<(u32, usize), ResolvedCharStyle>,
    /// 측정과 paint에 함께 전파하는 세션의 첨자 계약이다.
    pub script_metrics: crate::renderer::script_metrics::ScriptMetricsContract,
    pub(crate) native_pua_fonts: std::sync::Arc<crate::renderer::native_pua_fonts::NativePuaFonts>,
    /// HWP5 저장 겹침은 다시 열 때 -80%~+20% 범위로 해소된다.
    /// XML 입력의 원래 값과 구분하며 모델의 저장 바이트는 보존한다.
    pub(crate) char_overlap_size_bounds: Option<(i8, i8)>,
    /// 저장 겹침의 기준 글자모양이 현재 문단과 같은지 원본 속성으로 판정한다.
    /// 대체된 표시 글꼴의 유사성으로 저장 서식의 유효성을 추정하지 않는다.
    pub(crate) source_char_shapes: std::sync::Arc<[crate::model::style::CharShape]>,
    /// px 형태의 해소된 스타일을 원본 HWPUNIT으로 환산할 때의 배율.
    pub(crate) resolved_dpi: Option<f64>,
    /// 한컴 2020의 독립 자모 프로그램. 문서의 글꼴 선언과 구분한다.
    pub(crate) isolated_jamo_font: Option<crate::renderer::IsolatedJamoFont>,
    /// 문서의 `쪽 번호` 스타일이 참조하는 글자 모양. 자동 쪽번호는 본문
    /// 기본 글꼴이 아닌 이 스타일로 출력된다.
    pub page_number_char_style_id: Option<usize>,
    /// 자동 쪽번호 영문 슬롯의 선언 프로그램이 HFT인지 보존한다.
    /// 표시용 TTF 대체 이름으로 원본 글상자의 기준선 메트릭을 추측하지 않는다.
    pub page_number_font_is_hft: bool,
    /// Shared session measurements for DB-missing glyphs, not document styles.
    pub supplemental_metrics:
        Option<std::sync::Arc<crate::renderer::supplemental_metrics::SupplementalMetricSnapshot>>,
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
    /// [#2070] HWP3 → HWP5 변환본 여부 (Document::is_hwp3_variant 전파).
    /// 변환본 한정 레거시 폭 규칙(전체 폭) 게이트에 사용.
    pub hwp3_variant: bool,
    /// [#7051] HFT 한글 전용 face 의 ASCII 를 반각으로 잰다. HWP3 변환본(`hwp3_variant`)이거나,
    /// 그 신호가 없는 저장본에서 문서 자신의 저장 줄이 반각 조판을 증언할 때 켠다
    /// (`hft_ascii_evidence`). `hwp3_variant` 의 다른 보정(문단 간격 등)과는 독립이다.
    pub hft_ascii_halfwidth: bool,
    /// 한 pagination/edit transaction의 모든 fresh-layout 소비자가 함께 읽는
    /// exact-font source snapshot. Font payload는 registry의 Arc에 한 번만 있고,
    /// 스타일 복제는 snapshot owner만 공유한다.
    pub(crate) kerning_measurement_context:
        Option<std::sync::Arc<crate::renderer::kerning::KerningMeasurementContext>>,
    /// Q2-B cluster-aware shadow measurement context.  It is created from the
    /// same immutable registry snapshot as `kerning_measurement_context` and
    /// remains dormant until the composition-owner handoff qualifies.
    pub(crate) horizontal_shaping_context:
        Option<std::sync::Arc<crate::renderer::shaping_context::HorizontalShapingContext>>,
}

impl ResolvedStyleSet {
    pub(crate) fn literal_figure_space_em(
        &self,
        char_style_id: u32,
        lang_index: usize,
    ) -> Option<f64> {
        self.kerning_measurement_context
            .as_ref()
            .and_then(|context| {
                context.literal_figure_space_em(crate::renderer::kerning::ExactFontSlot::new(
                    char_style_id,
                    lang_index,
                ))
            })
            .or_else(|| {
                let style = self.char_styles.get(char_style_id as usize)?;
                self.native_pua_fonts.literal_space_em(
                    style.font_family_for_lang(lang_index),
                    style.bold,
                    style.italic,
                )
            })
    }

    /// 명시/내장 슬롯의 실제 프로그램을 외부 family 캐시로 대체하지 않는다.
    pub(crate) fn has_exact_font_source(&self, char_style_id: u32, lang_index: usize) -> bool {
        self.kerning_measurement_context
            .as_ref()
            .is_some_and(|context| {
                context.has_font_source(crate::renderer::kerning::ExactFontSlot::new(
                    char_style_id,
                    lang_index,
                ))
            })
    }

    /// 저장 값은 건드리지 않고 해당 입력 계약의 표시 범위만 해소한다.
    pub(crate) fn char_overlap_inner_size(&self, raw: i8) -> i8 {
        self.char_overlap_size_bounds
            .map_or(raw, |(min, max)| raw.clamp(min, max))
    }
}

crate::storage::clone_fields!(ResolvedCharStyle, {font_family, font_families, font_families_metric_trusted, font_families_glyph_fallback_advance_explicit, font_families_hft_hangul, font_families_source_hft, font_families_hft_print_frame, font_families_preserve_glyph_ink, font_families_hft_fullwidth_dot, ascii_punct_latin_slot, explicit_latin_slot, font_families_metric_face, font_families_original_glyph_metric_face, font_space_em, font_size, font_sizes, font_positions, bold, italic, text_color, underline, underline_color, strike_color, strikethrough, letter_spacing, letter_spacings, ratio, ratios, border_fill_id, outline_type, shadow_type, shadow_color, shadow_offset_x, shadow_offset_y, emboss, engrave, superscript, subscript, emphasis_dot, underline_shape, strike_shape, kerning, shade_color});

crate::storage::clone_fields!(ResolvedParaStyle, {alignment, vertical_alignment, line_spacing, line_spacing_type, margin_left, margin_right, indent, spacing_before, spacing_after, head_type, para_level, numbering_id, border_fill_id, border_spacing, border_connect, border_ignore_margin, default_tab_width, tab_stops, auto_tab_left, auto_tab_right, condense_min_space, english_break_unit, korean_break_unit, widow_orphan, keep_with_next, keep_lines, page_break_before});

crate::storage::clone_fields!(ResolvedBorderStyle, {borders, fill_color, pattern, gradient, image_fill, diagonal_attr, diagonal, center_line});

crate::storage::clone_fields!(ResolvedStyleSet, {source_program_projection_slots, native_pua_fonts, char_overlap_size_bounds, source_char_shapes, resolved_dpi, isolated_jamo_font, page_number_char_style_id, page_number_font_is_hft, supplemental_metrics, char_styles, para_styles, border_styles, numberings, bullets, hwp3_variant, hft_ascii_halfwidth, kerning_measurement_context, horizontal_shaping_context});

crate::storage::clone_fields!(ResolvedImageFill, {bin_data_id, fill_mode, brightness, contrast, effect});
