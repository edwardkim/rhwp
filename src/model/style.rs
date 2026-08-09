//! 스타일 정보 (Font, Color, Border, Fill, Gradient)

use super::*;

/// HWP 선 굵기 enum: index(0~15) ↔ mm. 한컴 표준 16단계.
///
/// 파서(mm→index 최근접)와 직렬화기(index→mm 문자열)가 이 단일 테이블을 공유해
/// 라운드트립 무손실을 보장한다. 종전엔 파서가 6단계 coarse bucket
/// (mm≤0.3→1, ≤0.5→2, ≤1.0→3)으로, 직렬화기가 16단계로 달라서 0.4mm→0.15mm,
/// 0.6mm→0.2mm 처럼 테두리 굵기가 변질됐다(IR index 는 안정이라 diff=0 이지만
/// 시각적으로 다른 굵기로 출력).
pub const BORDER_WIDTHS: [(f64, &str); 16] = [
    (0.1, "0.1"),
    (0.12, "0.12"),
    (0.15, "0.15"),
    (0.2, "0.2"),
    (0.25, "0.25"),
    (0.3, "0.3"),
    (0.4, "0.4"),
    (0.5, "0.5"),
    (0.6, "0.6"),
    (0.7, "0.7"),
    (1.0, "1.0"),
    (1.5, "1.5"),
    (2.0, "2.0"),
    (3.0, "3.0"),
    (4.0, "4.0"),
    (5.0, "5.0"),
];

/// mm 값에 가장 가까운 [`BORDER_WIDTHS`] 굵기 index(0~15)를 돌려준다(파서용).
pub fn border_width_index(mm: f64) -> u8 {
    BORDER_WIDTHS
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (mm - a.0).abs().total_cmp(&(mm - b.0).abs()))
        .map(|(i, _)| i as u8)
        .unwrap_or(0)
}

/// 굵기 index(0~15)에 대응하는 mm 문자열을 돌려준다(직렬화기용). 범위를 벗어나면
/// 기본값 "0.1".
pub fn border_width_mm_str(index: u8) -> &'static str {
    BORDER_WIDTHS
        .get(index as usize)
        .map(|(_, s)| *s)
        .unwrap_or("0.1")
}

/// 글꼴 정보 (HWPTAG_FACE_NAME)
#[derive(Debug, Clone, Default)]
pub struct Font {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 글꼴 이름
    pub name: String,
    /// 글꼴 유형 (0: 알 수 없음, 1: TTF, 2: HFT)
    pub alt_type: u8,
    /// HWPX 부모 `<hh:font>`가 embedded font resource를 가리키는지 여부.
    pub is_embedded: bool,
    /// HWPX 부모 `<hh:font>`의 embedded binary item reference.
    pub bin_item_id_ref: String,
    /// HWPX package manifest에서 해소된 BinData storage ID.
    ///
    /// 원본 `binaryItemIDRef`는 round-trip을 위해 그대로 보존하고, renderer는 이
    /// 필드만 사용해 임베디드 font bytes를 찾는다.
    pub resolved_bin_data_id: Option<u16>,
    /// 대체 글꼴 이름
    pub alt_name: Option<String>,
    /// 글꼴 유형 정보 (HWP5 FACE_NAME type info 10바이트)
    pub type_info: Option<[u8; 10]>,
    /// 기본 글꼴 이름
    pub default_name: Option<String>,
    /// 대체 글꼴 (HWPX `<hh:substFont>`) — 원본 글꼴 부재 시 대체될 글꼴 정보.
    /// HWP5 의 `alt_name`/`alt_type` 과 달리 type·임베드 정보를 독립적으로 보존한다.
    pub subst_font: Option<SubstFont>,
}

/// 대체 글꼴 (HWPX `<hh:substFont>`)
///
/// 4개 속성을 모두 보존해 라운드트립 무손실을 보장한다. `font_type`/`is_embedded`/
/// `bin_item_id_ref` 는 부모 `<hh:font>` 의 같은 이름 속성과 독립적이다
/// (예: HFT 글꼴이 TTF 대체 글꼴을 가질 수 있음).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubstFont {
    /// 대체 글꼴 이름
    pub face: String,
    /// 대체 글꼴 유형 (0: 알 수 없음, 1: TTF, 2: HFT)
    pub font_type: u8,
    /// 임베드 여부
    pub is_embedded: bool,
    /// 임베드 바이너리 아이템 ID 참조 (비임베드 시 빈 문자열; 항상 존재)
    pub bin_item_id_ref: String,
    /// HWPX package manifest에서 해소된 BinData storage ID.
    pub resolved_bin_data_id: Option<u16>,
}

/// 글자 모양 (HWPTAG_CHAR_SHAPE)
///
/// `Default` 는 [수동 구현](#impl-Default-for-CharShape)이다 — 파생하면 `relative_sizes` 가
/// 스펙 위반값 0 이 된다(#4141).
#[derive(Debug, Clone)]
pub struct CharShape {
    /// 원본 레코드 바이트 (라운드트립 보존용, 있으면 직렬화 시 우선 사용)
    pub raw_data: Option<Vec<u8>>,
    /// 언어별 글꼴 ID 참조값 (한글, 영어, 한자, 일어, 기타, 기호, 사용자)
    pub font_ids: [u16; 7],
    /// 언어별 장평 (50%~200%)
    pub ratios: [u8; 7],
    /// 언어별 자간 (-50%~50%)
    pub spacings: [i8; 7],
    /// 언어별 상대 크기 (10%~250%)
    pub relative_sizes: [u8; 7],
    /// 언어별 글자 위치 (-100%~100%)
    pub char_offsets: [i8; 7],
    /// 기준 크기 (HWPUNIT, 0pt~4096pt)
    pub base_size: i32,
    /// 속성 비트 플래그
    pub attr: u32,
    /// 기울임 여부
    pub italic: bool,
    /// 진하게 여부
    pub bold: bool,
    /// 밑줄 종류
    pub underline_type: UnderlineType,
    /// 외곽선 종류
    pub outline_type: u8,
    /// 그림자 종류
    pub shadow_type: u8,
    /// 그림자 X 방향 오프셋 (-100~100%)
    pub shadow_offset_x: i8,
    /// 그림자 Y 방향 오프셋 (-100~100%)
    pub shadow_offset_y: i8,
    /// 글자 색
    pub text_color: ColorRef,
    /// 밑줄 색
    pub underline_color: ColorRef,
    /// 음영 색
    pub shade_color: ColorRef,
    /// 그림자 색
    pub shadow_color: ColorRef,
    /// 글자 테두리/배경 ID (5.0.2.1 이상)
    pub border_fill_id: u16,
    /// 취소선 색
    pub strike_color: ColorRef,
    /// 취소선 여부
    pub strikethrough: bool,
    /// 위/아래 첨자
    pub subscript: bool,
    pub superscript: bool,
    /// 양각 여부 (bit 13)
    pub emboss: bool,
    /// 음각 여부 (bit 14)
    pub engrave: bool,
    /// 강조점 종류 (bit 21-24, 0=없음, 1=● 2=○ 3=ˇ 4=˜ 5=･ 6=:)
    pub emphasis_dot: u8,
    /// 밑줄 모양 (bit 4-7, 표 27 선 종류: 0=실선, 1=긴점선, ...)
    pub underline_shape: u8,
    /// 취소선 모양 (bit 26-29, 표 27 선 종류)
    pub strike_shape: u8,
    /// 커닝 여부 (bit 30)
    pub kerning: bool,
    /// 글꼴에 어울리는 빈칸 사용 여부 (bit 25)
    pub use_font_space: bool,
}

/// `relative_sizes` 만 Rust 파생 기본값(0)이 아니라 **스펙 기본값 100** 이다. 나머지 필드는
/// 파생값과 같다.
///
/// # 왜 파생 Default 로는 안 되는가 (#4141)
///
/// OWPML 은 `relSz` 를 `xs:positiveInteger` minInclusive=10 / maxInclusive=250,
/// `default="100"` 으로 정의한다(`mydocs/manual/OWPML SCHEMA/Header XML schema.xml:716-728`).
/// **0 은 타입 수준에서 이미 불법이다.** 한컴은 실효 크기를 `기준 크기 × 상대크기%` 로
/// 해석하므로 0 이 저장되면 10pt 글자가 0.1pt 로 그려져 문서가 사실상 백지가 된다
/// (`samples/SO-SUEOP.hwp` 변환본 한컴 PDF 실측: 46쪽 10,604 span 전부 0.12pt).
///
/// 이 값을 채우지 않는 생성 경로가 여럿이다 — HWP3 변환(`parser/hwp3/mod.rs:526`, HWP3
/// 레코드에 상대크기 개념 자체가 없다), HWPX `charPr` 의 `relSz` 자식 부재와 id 갭
/// 채움(`parser/hwpx/header.rs:588`, `:848-858`), HML `RELSIZE` 부재
/// (`parser/hml/reader.rs:599-605`), HTML import. 세 라이터(HWP5·HWPX·HML)는 모두 가드 없이
/// IR 값을 그대로 방출한다. 기본값을 스펙에 맞추면 그 전부가 한 곳에서 해소된다.
///
/// HWP5 바이너리 파서는 이미 100 을 폴백한다(`parser/doc_info.rs:542-545`) — 파생 Default 의
/// 0 은 그 폴백과도 불일치였다.
///
/// # 왜 `ratios`·`base_size` 는 그대로 두는가
///
/// 렌더러가 소비하는 필드다(`renderer/style_resolver.rs:341`,`:355`). 기본값을 바꾸면 렌더
/// 회귀 검증 lane 이 필요해지므로 별도 이슈로 분리한다. `relative_sizes` 는 렌더 경로에서
/// 참조가 0건이라(`document_core/queries/hidden_text.rs:267-287`) 이 변경의 렌더 영향은 없다.
///
/// # 음영색 sentinel (#4155)
///
/// `shade_color` 의 파생 기본값 0 은 **검정**이고, HWP5 라이터가 그대로 저장하면 한컴이
/// 글자마다 순검정 사각형을 칠해 본문 전체가 검정 막대가 된다(`samples/SO-SUEOP.hwp`
/// 변환본 한컴 PDF 실측: 3쪽에 줄 크기 검정 fill 65개, 원본은 글리프 크기 35개).
/// rhwp 자신은 이 결함을 볼 수 없다 — 렌더러가 검정을 "음영 없음" sentinel 로 읽는다.
///
/// 한컴은 "음영 없음"을 `0xFFFFFFFF` 로 쓴다 — 코퍼스 380건에서 22,189회, 검정은 0회다.
/// HWPX `shadeColor="none"` 과 한/글 HML `4294967295` 도 같은 값으로 수렴한다. 정의는
/// [`crate::model::color::NONE`] 하나이며, 세 라이터(HWP5·HWPX·HML)가 무수정으로 정합한다.
///
/// # 왜 필드를 전부 나열하는가
///
/// `CharShape` 에 필드가 추가되면 이 impl 이 컴파일 에러를 낸다. 새 필드의 기본값을 스펙과
/// 대조하도록 강제하는 장치다 — 조용한 표류를 막는다.
impl Default for CharShape {
    fn default() -> Self {
        Self {
            raw_data: None,
            font_ids: [0; 7],
            ratios: [0; 7],
            spacings: [0; 7],
            // ↓ 이 한 줄만 파생값과 다르다 (파생값 0 은 OWPML 유효범위 10~250 밖)
            relative_sizes: [100; 7],
            char_offsets: [0; 7],
            base_size: 0,
            attr: 0,
            italic: false,
            bold: false,
            underline_type: UnderlineType::None,
            outline_type: 0,
            shadow_type: 0,
            shadow_offset_x: 0,
            shadow_offset_y: 0,
            text_color: 0,
            underline_color: 0,
            // ↓ 파생값 0(검정)은 한컴이 본문을 검정 막대로 덮게 만든다 (#4155)
            shade_color: super::color::NONE,
            shadow_color: 0,
            border_fill_id: 0,
            strike_color: 0,
            strikethrough: false,
            subscript: false,
            superscript: false,
            emboss: false,
            engrave: false,
            emphasis_dot: 0,
            underline_shape: 0,
            strike_shape: 0,
            kerning: false,
            use_font_space: false,
        }
    }
}

/// CharShape 비교: raw_data 필드 제외 (라운드트립용 원본 바이트는 논리적 동일성과 무관)
impl PartialEq for CharShape {
    fn eq(&self, other: &Self) -> bool {
        self.font_ids == other.font_ids
            && self.ratios == other.ratios
            && self.spacings == other.spacings
            && self.relative_sizes == other.relative_sizes
            && self.char_offsets == other.char_offsets
            && self.base_size == other.base_size
            && self.attr == other.attr
            && self.italic == other.italic
            && self.bold == other.bold
            && self.underline_type == other.underline_type
            && self.outline_type == other.outline_type
            && self.shadow_type == other.shadow_type
            && self.shadow_offset_x == other.shadow_offset_x
            && self.shadow_offset_y == other.shadow_offset_y
            && self.text_color == other.text_color
            && self.underline_color == other.underline_color
            && self.shade_color == other.shade_color
            && self.shadow_color == other.shadow_color
            && self.border_fill_id == other.border_fill_id
            && self.strike_color == other.strike_color
            && self.strikethrough == other.strikethrough
            && self.subscript == other.subscript
            && self.superscript == other.superscript
            && self.emboss == other.emboss
            && self.engrave == other.engrave
            && self.emphasis_dot == other.emphasis_dot
            && self.underline_shape == other.underline_shape
            && self.strike_shape == other.strike_shape
            && self.kerning == other.kerning
            && self.use_font_space == other.use_font_space
    }
}

impl Eq for CharShape {}

/// 밑줄 종류
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize)]
pub enum UnderlineType {
    #[default]
    None,
    Bottom,
    Top,
}

/// 문단 머리 모양 종류 (attr1 bit 23~24)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum HeadType {
    /// 없음
    #[default]
    None,
    /// 개요
    Outline,
    /// 번호
    Number,
    /// 글머리표
    Bullet,
}

/// 문단 모양 (HWPTAG_PARA_SHAPE)
#[derive(Debug, Clone, Default)]
pub struct ParaShape {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 속성1 비트 플래그
    pub attr1: u32,
    /// 왼쪽 여백
    pub margin_left: i32,
    /// 오른쪽 여백
    pub margin_right: i32,
    /// 들여쓰기/내어쓰기
    pub indent: i32,
    /// 문단 간격 위
    pub spacing_before: i32,
    /// 문단 간격 아래
    pub spacing_after: i32,
    /// 줄 간격
    pub line_spacing: i32,
    /// 정렬 방식
    pub alignment: Alignment,
    /// 줄 간격 종류
    pub line_spacing_type: LineSpacingType,
    /// 탭 정의 ID 참조
    pub tab_def_id: u16,
    /// 번호/글머리표 ID 참조
    pub numbering_id: u16,
    /// 테두리/배경 ID 참조
    pub border_fill_id: u16,
    /// 문단 테두리 간격 (좌, 우, 상, 하)
    pub border_spacing: [i16; 4],
    /// 속성2 (5.0.1.7 이상)
    pub attr2: u32,
    /// 속성3 - 줄 간격 종류 확장 (5.0.2.5 이상)
    pub attr3: u32,
    /// 줄 간격 (5.0.2.5 이상, 이전 line_spacing 대체)
    pub line_spacing_v2: u32,
    /// 문단 머리 모양 종류 (attr1 bit 23~24)
    pub head_type: HeadType,
    /// 문단 수준 (0~6 → 1~7수준, attr1 bit 25~27)
    pub para_level: u8,
    /// [#1986] HWPX breakSetting@breakLatinWord 원문 보존
    /// (BREAK_WORD/KEEP_WORD/HYPHENATION). 파서 미수집 시 None → 직렬화 기본값
    /// KEEP_WORD. 값이 3가지라 attr1 비트 인코딩 대신 원문 보존으로 무손실 방출.
    /// 꼬리말·표셀 등 재계산 경로에서 줄나눔이 달라져 레이아웃이 갈리는 것을 막는다.
    pub break_latin_word: Option<String>,
}

/// ParaShape 비교: raw_data 필드 제외 (라운드트립용 원본 바이트는 논리적 동일성과 무관)
impl PartialEq for ParaShape {
    fn eq(&self, other: &Self) -> bool {
        self.attr1 == other.attr1
            && self.margin_left == other.margin_left
            && self.margin_right == other.margin_right
            && self.indent == other.indent
            && self.spacing_before == other.spacing_before
            && self.spacing_after == other.spacing_after
            && self.line_spacing == other.line_spacing
            && self.alignment == other.alignment
            && self.line_spacing_type == other.line_spacing_type
            && self.tab_def_id == other.tab_def_id
            && self.numbering_id == other.numbering_id
            && self.border_fill_id == other.border_fill_id
            && self.border_spacing == other.border_spacing
            && self.attr2 == other.attr2
            && self.attr3 == other.attr3
            && self.line_spacing_v2 == other.line_spacing_v2
            && self.head_type == other.head_type
            && self.para_level == other.para_level
            && self.break_latin_word == other.break_latin_word
    }
}

impl Eq for ParaShape {}

/// 문단 번호 정의 (HWPTAG_NUMBERING)
#[derive(Debug, Clone, Default)]
pub struct Numbering {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 수준별(1~7) 문단 머리 정보
    pub heads: [NumberingHead; 7],
    /// 수준별 번호 형식 문자열
    pub level_formats: [String; 7],
    /// 시작 번호
    pub start_number: u16,
    /// 수준별 시작 번호
    pub level_start_numbers: [u32; 7],
    /// HWPX `<hh:numbering>` 의 자식 `<hh:paraHead>` 영역 원본 XML
    /// (여는/닫는 태그 사이 그대로). 모델은 7수준만 표현하지만 HWPX 는
    /// 10수준 + align/useInstWidth/autoIndent/checkable/형식문자열 등을
    /// 가지므로, 무손실 라운드트립을 위해 원본 구간을 그대로 보존해 splice 한다.
    /// HWP5 바이너리 경로 등 원본 XML 이 없으면 `None` → 하드코딩 폴백.
    pub raw_para_heads: Option<String>,
}

/// 문단 머리 정보 (표 41)
#[derive(Debug, Clone, Copy, Default)]
pub struct NumberingHead {
    /// 속성 (정렬, 너비 따름, 자동 내어쓰기 등)
    pub attr: u32,
    /// 너비 보정값
    pub width_adjust: i16,
    /// 본문과의 거리
    pub text_distance: i16,
    /// 글자 모양 아이디 참조
    pub char_shape_id: u32,
    /// 번호 형식 코드 (표 43, attr bit 5~8에서 추출)
    pub number_format: u8,
}

/// 글머리표 정의 (HWPTAG_BULLET, 표 44, 24바이트)
#[derive(Debug, Clone, Default)]
pub struct Bullet {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 속성 (정렬, 너비 따름, 자동 내어쓰기 등)
    pub attr: u32,
    /// 너비 보정값
    pub width_adjust: i16,
    /// 본문과의 거리
    pub text_distance: i16,
    /// 글자 모양 아이디 참조 (문단 머리 정보 12바이트의 마지막 4바이트)
    pub char_shape_id: u32,
    /// 글머리표 문자 (●, ■, ▶ 등)
    pub bullet_char: char,
    /// 이미지 글머리표 여부 (0=문자, ID=이미지)
    pub image_bullet: i32,
    /// 이미지 글머리 데이터 (대비, 밝기, 효과, ID)
    pub image_data: [u8; 4],
    /// 체크 글머리표 문자
    pub check_bullet_char: char,
    /// HWPX `<hh:bullet>` 자식 `<hh:paraHead>`(+`<hh:img>`) 원본 구간 (무손실 splice 용).
    /// align/useInstWidth/autoIndent/textOffsetType/checkable 등 7수준 필드로 표현
    /// 못하는 HWPX 전용 속성 보존. [#2790]
    pub raw_para_head: Option<String>,
}

/// 텍스트 정렬 방식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Alignment {
    #[default]
    Justify,
    Left,
    Right,
    Center,
    Distribute,
    Split,
}

/// 줄 간격 종류
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum LineSpacingType {
    #[default]
    Percent,
    Fixed,
    SpaceOnly,
    Minimum,
}

/// 탭 정의 (HWPTAG_TAB_DEF)
#[derive(Debug, Clone, Default)]
pub struct TabDef {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 속성 비트 플래그
    pub attr: u32,
    /// 탭 항목 리스트
    pub tabs: Vec<TabItem>,
    /// 왼쪽 끝 자동 탭 유무
    pub auto_tab_left: bool,
    /// 오른쪽 끝 자동 탭 유무
    pub auto_tab_right: bool,
}

/// 탭 항목
#[derive(Debug, Clone, Default)]
pub struct TabItem {
    /// 탭 위치
    pub position: HwpUnit,
    /// 탭 종류 (0: 왼쪽, 1: 오른쪽, 2: 가운데, 3: 소수점)
    pub tab_type: u8,
    /// 채움 종류
    pub fill_type: u8,
}

impl PartialEq for TabItem {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.tab_type == other.tab_type
            && self.fill_type == other.fill_type
    }
}
impl Eq for TabItem {}

impl PartialEq for TabDef {
    fn eq(&self, other: &Self) -> bool {
        self.auto_tab_left == other.auto_tab_left
            && self.auto_tab_right == other.auto_tab_right
            && self.tabs == other.tabs
    }
}
impl Eq for TabDef {}

/// 스타일 (HWPTAG_STYLE)
#[derive(Debug, Clone, Default)]
pub struct Style {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 한글 스타일 이름
    pub local_name: String,
    /// 영문 스타일 이름
    pub english_name: String,
    /// 스타일 종류 (0: 문단, 1: 글자) — 표 47/48
    pub style_type: u8,
    /// 다음 스타일 ID
    pub next_style_id: u8,
    /// [Task #1058 후속] 언어 아이디 (INT16, default 1042=한국어).
    /// 한컴 spec 표 47 의 next_style_id 다음 필드. 누락 시 ps_id/cs_id 가
    /// 2 byte 앞당겨져 한컴이 잘못된 ParaShape 적용. footnote-01 의 정답지
    /// 비교로 입증 — rhwp 의 style record size=28 vs 정답지 size=32 (4 byte 누락).
    pub lang_id: i16,
    /// 문단 모양 ID 참조
    pub para_shape_id: u16,
    /// 글자 모양 ID 참조
    pub char_shape_id: u16,
    /// [Task #2839] 양식(폼) 필드 잠금 여부 (HWPX `lockForm`).
    /// 파서가 값을 읽지 않고 시리얼라이저가 "0" 을 하드코딩해 원본이 항상
    /// 잠금 해제 상태로 바뀌던 결함 수정.
    pub lock_form: bool,
}

/// 테두리/배경 (HWPTAG_BORDER_FILL)
#[derive(Debug, Clone, Default)]
pub struct BorderFill {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 속성 비트 플래그
    pub attr: u16,
    /// 4방향 테두리선 (좌, 우, 상, 하)
    pub borders: [BorderLine; 4],
    /// 대각선
    pub diagonal: DiagonalLine,
    /// 중심선 방향
    pub center_line: CenterLine,
    /// 채우기 정보
    pub fill: Fill,
    /// 3차원 효과 (HWPX borderFill@threeD)
    pub three_d: bool,
}

/// 중심선 방향 (HWPX borderFill@centerLine)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CenterLine {
    /// 없음
    #[default]
    None,
    /// HWPX `VERTICAL` 값. 한컴 2024 기준으로는 셀 중앙 가로선으로 표시된다.
    Vertical,
    /// HWPX `HORIZONTAL` 값. 한컴 2024 기준으로는 셀 중앙 세로선으로 표시된다.
    Horizontal,
    /// 가로+세로 중심선
    Cross,
}

impl CenterLine {
    pub fn from_hwp_attr(attr: u16) -> Self {
        if attr & (1 << 13) == 0 {
            return Self::None;
        }
        let slash_crooked = attr & (1 << 8) != 0;
        let backslash_crooked = attr & (1 << 10) != 0;
        match (slash_crooked, backslash_crooked) {
            (true, false) => Self::Vertical,
            (false, true) => Self::Horizontal,
            _ => Self::Cross,
        }
    }

    pub fn from_hwpx(value: &str) -> Self {
        match value {
            "VERTICAL" => Self::Vertical,
            "HORIZONTAL" => Self::Horizontal,
            "CROSS" => Self::Cross,
            _ => Self::None,
        }
    }

    pub fn hwp_attr_bits(self) -> u16 {
        match self {
            Self::None => 0,
            Self::Vertical => (1 << 13) | (0x03 << 8),
            Self::Horizontal => (1 << 13) | (1 << 10),
            Self::Cross => (1 << 13) | (0x03 << 8) | (1 << 10),
        }
    }

    pub fn hwp_binary_attr_bits(self) -> u16 {
        match self {
            Self::None => 0,
            Self::Vertical => (1 << 13) | (0x03 << 8),
            Self::Horizontal => (1 << 13) | (1 << 10),
            Self::Cross => (1 << 13) | (0x03 << 8) | (1 << 10),
        }
    }

    pub fn as_hwpx(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Vertical => "VERTICAL",
            Self::Horizontal => "HORIZONTAL",
            Self::Cross => "CROSS",
        }
    }
}

/// 테두리선 정보
#[derive(Debug, Clone, Copy, Default)]
pub struct BorderLine {
    /// 선 종류
    pub line_type: BorderLineType,
    /// 선 굵기 인덱스
    pub width: u8,
    /// 선 색상
    pub color: ColorRef,
}

/// 테두리선 종류 (HWP 스펙 표 27)
/// 0=선없음, 1=실선, 2=파선, 3=점선, ...
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BorderLineType {
    /// 선 없음 (0)
    None,
    /// 실선 (1)
    #[default]
    Solid,
    /// 파선 (2)
    Dash,
    /// 점선 (3)
    Dot,
    /// 일점 쇄선 (4)
    DashDot,
    /// 이점 쇄선 (5)
    DashDotDot,
    /// 긴 파선 (6)
    LongDash,
    /// 원형 파선 (7)
    Circle,
    /// 이중선 (8)
    Double,
    /// 가는선-굵은선 이중선 (9)
    ThinThickDouble,
    /// 굵은선-가는선 이중선 (10)
    ThickThinDouble,
    /// 가는선-굵은선-가는선 삼중선 (11)
    ThinThickThinTriple,
    /// 물결선 (12)
    Wave,
    /// 이중 물결선 (13)
    DoubleWave,
    /// 3D (14)
    Thick3D,
    /// 3D 반전 (15)
    Thick3DReverse,
    /// 3D 가는선 (16)
    Thin3D,
    /// 3D 가는선 반전 (17)
    Thin3DReverse,
}

/// 대각선 정보
#[derive(Debug, Clone, Copy, Default)]
pub struct DiagonalLine {
    /// 대각선 선 종류 코드. BorderLineType의 HWP/HWPX 코드와 같은 값을 사용한다.
    pub diagonal_type: u8,
    /// 대각선 굵기
    pub width: u8,
    /// 대각선 색상
    pub color: ColorRef,
}

/// 채우기 정보
#[derive(Debug, Clone, Default)]
pub struct Fill {
    /// 채우기 종류
    pub fill_type: FillType,
    /// 단색 채우기
    pub solid: Option<SolidFill>,
    /// 그러데이션 채우기
    pub gradient: Option<GradientFill>,
    /// 이미지 채우기
    pub image: Option<ImageFill>,
    /// 채우기 불투명도 (0=완전투명/미설정, 255=불투명)
    pub alpha: u8,
}

/// 채우기 종류
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FillType {
    #[default]
    None,
    Solid,
    Image,
    Gradient,
}

/// 단색 채우기
#[derive(Debug, Clone, Copy, Default)]
pub struct SolidFill {
    /// 배경색
    pub background_color: ColorRef,
    /// 무늬색
    pub pattern_color: ColorRef,
    /// 무늬 종류
    pub pattern_type: i32,
}

/// 그러데이션 채우기
#[derive(Debug, Clone, Default)]
pub struct GradientFill {
    /// 유형 (1: 줄무늬, 2: 원형, 3: 원뿔형, 4: 사각형)
    pub gradient_type: i16,
    /// 기울임 (시작 각)
    pub angle: i16,
    /// 가로 중심
    pub center_x: i16,
    /// 세로 중심
    pub center_y: i16,
    /// 번짐 정도 (0~100)
    pub blur: i16,
    /// 번짐 중심 (0~100)
    pub step_center: u8,
    /// 색상 목록
    pub colors: Vec<ColorRef>,
    /// 색상 위치 목록
    pub positions: Vec<i32>,
}

/// 이미지 채우기
#[derive(Debug, Clone, Default)]
pub struct ImageFill {
    /// 채우기 유형
    pub fill_mode: ImageFillMode,
    /// 밝기
    pub brightness: i8,
    /// 명암
    pub contrast: i8,
    /// 그림 효과
    pub effect: u8,
    /// BinData ID 참조
    pub bin_data_id: u16,
}

/// 이미지 채우기 유형
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize)]
pub enum ImageFillMode {
    #[default]
    TileAll,
    TileHorzTop,
    TileHorzBottom,
    TileVertLeft,
    TileVertRight,
    FitToSize,
    Total,
    Center,
    CenterTop,
    CenterBottom,
    LeftCenter,
    LeftTop,
    LeftBottom,
    RightCenter,
    RightTop,
    RightBottom,
    None,
}

/// 테두리 선 정보 (그리기 개체용)
#[derive(Debug, Clone, Copy, Default)]
pub struct ShapeBorderLine {
    /// 선 색상
    pub color: ColorRef,
    /// 선 굵기 (HWPUNIT, INT32)
    pub width: i32,
    /// 속성 비트 플래그
    pub attr: u32,
    /// 아웃라인 스타일 (0: normal, 1: outer, 2: inner)
    pub outline_style: u8,
}

/// 글자 모양 수정 사항 (None이면 해당 속성 변경 안 함)
#[derive(Debug, Clone, Default)]
pub struct CharShapeMods {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub font_id: Option<u16>,
    pub base_size: Option<i32>,
    pub text_color: Option<super::ColorRef>,
    pub shade_color: Option<super::ColorRef>,
    /// 밑줄 종류 (underline bool보다 우선)
    pub underline_type: Option<UnderlineType>,
    pub underline_color: Option<super::ColorRef>,
    pub outline_type: Option<u8>,
    pub shadow_type: Option<u8>,
    pub shadow_color: Option<super::ColorRef>,
    pub shadow_offset_x: Option<i8>,
    pub shadow_offset_y: Option<i8>,
    pub strike_color: Option<super::ColorRef>,
    /// 아래첨자 (superscript와 상호 배타)
    pub subscript: Option<bool>,
    /// 위첨자 (subscript와 상호 배타)
    pub superscript: Option<bool>,
    /// 언어별 장평 (50~200%)
    pub ratios: Option<[u8; 7]>,
    /// 언어별 자간 (-50~50%)
    pub spacings: Option<[i8; 7]>,
    /// 언어별 상대 크기 (10~250%)
    pub relative_sizes: Option<[u8; 7]>,
    /// 언어별 글자 위치 (-100~100%)
    pub char_offsets: Option<[i8; 7]>,
    /// 양각
    pub emboss: Option<bool>,
    /// 음각
    pub engrave: Option<bool>,
    /// 언어별 개별 글꼴 ID (7개: 한글/영문/한자/일어/기타/기호/사용자)
    pub font_ids: Option<[u16; 7]>,
    /// 글자 테두리/배경 ID (1-based, 0=없음)
    pub border_fill_id: Option<u16>,
    /// 강조점 종류 (0~6)
    pub emphasis_dot: Option<u8>,
    /// 밑줄 모양 (0~10, 표 27 선 종류)
    pub underline_shape: Option<u8>,
    /// 취소선 모양 (0~10, 표 27 선 종류)
    pub strike_shape: Option<u8>,
    /// 커닝 여부
    pub kerning: Option<bool>,
}

impl CharShapeMods {
    /// 기존 CharShape에 수정사항을 적용한 새 CharShape를 반환한다.
    pub fn apply_to(&self, base: &CharShape) -> CharShape {
        let mut cs = base.clone();
        // 수정된 CharShape는 원본 바이트와 달라지므로 raw_data 무효화
        cs.raw_data = None;
        if let Some(v) = self.bold {
            cs.bold = v;
        }
        if let Some(v) = self.italic {
            cs.italic = v;
        }
        if let Some(v) = self.underline {
            cs.underline_type = if v {
                UnderlineType::Bottom
            } else {
                UnderlineType::None
            };
        }
        if let Some(v) = self.strikethrough {
            cs.strikethrough = v;
        }
        if let Some(id) = self.font_id {
            // 모든 언어에 동일한 글꼴 ID 적용
            for fid in &mut cs.font_ids {
                *fid = id;
            }
        }
        if let Some(v) = self.base_size {
            cs.base_size = v;
        }
        if let Some(v) = self.text_color {
            cs.text_color = v;
        }
        if let Some(v) = self.shade_color {
            cs.shade_color = v;
        }
        // underline_type이 있으면 underline bool보다 우선
        if let Some(v) = self.underline_type {
            cs.underline_type = v;
        }
        if let Some(v) = self.underline_color {
            cs.underline_color = v;
        }
        if let Some(v) = self.outline_type {
            cs.outline_type = v;
        }
        if let Some(v) = self.shadow_type {
            cs.shadow_type = v;
        }
        if let Some(v) = self.shadow_color {
            cs.shadow_color = v;
        }
        if let Some(v) = self.shadow_offset_x {
            cs.shadow_offset_x = v;
        }
        if let Some(v) = self.shadow_offset_y {
            cs.shadow_offset_y = v;
        }
        if let Some(v) = self.strike_color {
            cs.strike_color = v;
        }
        // subscript/superscript: 상호 배타
        if let Some(v) = self.superscript {
            cs.superscript = v;
            if v {
                cs.subscript = false;
            }
        }
        if let Some(v) = self.subscript {
            cs.subscript = v;
            if v {
                cs.superscript = false;
            }
        }
        if let Some(v) = self.ratios {
            cs.ratios = v;
        }
        if let Some(v) = self.spacings {
            cs.spacings = v;
        }
        if let Some(v) = self.relative_sizes {
            cs.relative_sizes = v;
        }
        if let Some(v) = self.char_offsets {
            cs.char_offsets = v;
        }
        // emboss/engrave: 상호 배타
        if let Some(v) = self.emboss {
            cs.emboss = v;
            if v {
                cs.engrave = false;
            }
        }
        if let Some(v) = self.engrave {
            cs.engrave = v;
            if v {
                cs.emboss = false;
            }
        }
        if let Some(ids) = self.font_ids {
            cs.font_ids = ids;
        }
        if let Some(v) = self.border_fill_id {
            cs.border_fill_id = v;
        }
        if let Some(v) = self.emphasis_dot {
            cs.emphasis_dot = v;
        }
        if let Some(v) = self.underline_shape {
            cs.underline_shape = v;
        }
        if let Some(v) = self.strike_shape {
            cs.strike_shape = v;
        }
        if let Some(v) = self.kerning {
            cs.kerning = v;
        }
        cs
    }
}

/// 문단 모양 수정 사항 (None이면 해당 속성 변경 안 함)
#[derive(Debug, Clone, Default)]
pub struct ParaShapeMods {
    pub alignment: Option<Alignment>,
    pub line_spacing: Option<i32>,
    pub line_spacing_type: Option<LineSpacingType>,
    pub indent: Option<i32>,
    pub margin_left: Option<i32>,
    pub margin_right: Option<i32>,
    pub spacing_before: Option<i32>,
    pub spacing_after: Option<i32>,
    // 확장 탭 속성
    pub head_type: Option<HeadType>,
    pub para_level: Option<u8>,
    pub widow_orphan: Option<bool>,
    pub keep_with_next: Option<bool>,
    pub keep_lines: Option<bool>,
    pub page_break_before: Option<bool>,
    pub font_line_height: Option<bool>,
    pub single_line: Option<bool>,
    pub auto_space_kr_en: Option<bool>,
    pub auto_space_kr_num: Option<bool>,
    pub vertical_align: Option<u8>,
    // 줄바꿈 모드
    pub english_break_unit: Option<u8>, // 0=단어, 1=하이픈, 2=글자
    pub korean_break_unit: Option<u8>,  // 0=어절, 1=글자
    // 탭 설정 탭 속성
    pub tab_def_id: Option<u16>,
    // 번호/글머리표 ID
    pub numbering_id: Option<u16>,
    // 테두리/배경 탭 속성
    pub border_fill_id: Option<u16>,
    pub border_spacing: Option<[i16; 4]>,
    pub border_connect: Option<bool>,
    pub border_ignore_margin: Option<bool>,
}

impl ParaShapeMods {
    /// 기존 ParaShape에 수정사항을 적용한 새 ParaShape를 반환한다.
    pub fn apply_to(&self, base: &ParaShape) -> ParaShape {
        let mut ps = base.clone();
        // 수정된 ParaShape는 원본 바이트와 달라지므로 raw_data 무효화
        ps.raw_data = None;
        if let Some(v) = self.alignment {
            ps.alignment = v;
        }
        if let Some(v) = self.line_spacing {
            ps.line_spacing = v;
        }
        if let Some(v) = self.line_spacing_type {
            ps.line_spacing_type = v;
        }
        if let Some(v) = self.indent {
            ps.indent = v;
        }
        if let Some(v) = self.margin_left {
            ps.margin_left = v;
        }
        if let Some(v) = self.margin_right {
            ps.margin_right = v;
        }
        if let Some(v) = self.spacing_before {
            ps.spacing_before = v;
        }
        if let Some(v) = self.spacing_after {
            ps.spacing_after = v;
        }
        // 확장 탭: 구조체 필드 + attr1/attr2 비트 동기화
        fn set_bit(val: &mut u32, bit: u32, on: bool) {
            if on {
                *val |= 1 << bit;
            } else {
                *val &= !(1 << bit);
            }
        }
        if let Some(v) = self.head_type {
            ps.head_type = v;
            ps.attr1 = (ps.attr1 & !(0x03 << 23)) | ((v as u32 & 0x03) << 23);
        }
        if let Some(v) = self.para_level {
            ps.para_level = v;
            ps.attr1 = (ps.attr1 & !(0x07 << 25)) | ((v as u32 & 0x07) << 25);
        }
        if let Some(v) = self.widow_orphan {
            set_bit(&mut ps.attr1, 16, v);
        }
        if let Some(v) = self.keep_with_next {
            set_bit(&mut ps.attr1, 17, v);
        }
        if let Some(v) = self.keep_lines {
            set_bit(&mut ps.attr1, 18, v);
        }
        if let Some(v) = self.page_break_before {
            set_bit(&mut ps.attr1, 19, v);
        }
        if let Some(v) = self.font_line_height {
            set_bit(&mut ps.attr1, 22, v);
        }
        if let Some(v) = self.single_line {
            ps.attr2 = (ps.attr2 & !0x03) | if v { 1 } else { 0 };
        }
        if let Some(v) = self.auto_space_kr_en {
            set_bit(&mut ps.attr2, 4, v);
        }
        if let Some(v) = self.auto_space_kr_num {
            set_bit(&mut ps.attr2, 5, v);
        }
        if let Some(v) = self.vertical_align {
            ps.attr1 = (ps.attr1 & !(0x03 << 20)) | ((v as u32 & 0x03) << 20);
        }
        if let Some(v) = self.english_break_unit {
            ps.attr1 = (ps.attr1 & !(0x03 << 5)) | ((v as u32 & 0x03) << 5);
        }
        if let Some(v) = self.korean_break_unit {
            ps.attr1 = (ps.attr1 & !(0x01 << 7)) | ((v as u32 & 0x01) << 7);
        }
        if let Some(v) = self.tab_def_id {
            ps.tab_def_id = v;
        }
        if let Some(v) = self.numbering_id {
            ps.numbering_id = v;
        }
        if let Some(v) = self.border_fill_id {
            ps.border_fill_id = v;
        }
        if let Some(v) = self.border_spacing {
            ps.border_spacing = v;
        }
        if let Some(v) = self.border_connect {
            set_bit(&mut ps.attr1, 28, v);
        }
        if let Some(v) = self.border_ignore_margin {
            set_bit(&mut ps.attr1, 29, v);
        }
        ps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_width_table_is_a_lossless_bijection() {
        // 모든 enum index 가 mm 문자열로 나갔다가 최근접 매핑으로 같은 index 로 돌아와야 한다.
        for (i, (mm, s)) in BORDER_WIDTHS.iter().enumerate() {
            assert_eq!(border_width_mm_str(i as u8), *s, "index {i} → 문자열");
            let reparsed: f64 = s.parse().unwrap();
            assert_eq!(
                border_width_index(reparsed),
                i as u8,
                "{mm}mm 가 index {i} 로 최근접 복원되어야 함"
            );
        }
    }

    #[test]
    fn border_width_index_fixes_coarse_bucket_regression() {
        // 종전 coarse bucket 이 변질시키던 실제 값들이 정확히 보존되는지 확인.
        assert_eq!(border_width_index(0.4), 6); // 종전 2(→"0.15")로 변질
        assert_eq!(border_width_mm_str(6), "0.4");
        assert_eq!(border_width_index(0.6), 8); // 종전 3(→"0.2")로 변질
        assert_eq!(border_width_mm_str(8), "0.6");
        assert_eq!(border_width_index(0.1), 0);
        assert_eq!(border_width_mm_str(0), "0.1");
        // 범위 밖 index 는 기본값.
        assert_eq!(border_width_mm_str(99), "0.1");
    }

    #[test]
    fn test_char_shape_default() {
        let cs = CharShape::default();
        assert!(!cs.bold);
        assert!(!cs.italic);
        assert_eq!(cs.underline_type, UnderlineType::None);
    }

    /// `Default` 수동 구현이 파생값과 어긋나는 필드는 `relative_sizes`(#4141)와
    /// `shade_color`(#4155) **둘뿐**임을 고정한다.
    ///
    /// 이 비대칭은 의도된 것이다. 두 필드는 파생값이 각각 스펙 위반(relSz 유효범위 밖)과
    /// 실제 색(검정)이라 저장 바이트에서 한컴을 깨뜨렸다. 반면 `ratios`·`base_size` 는
    /// 렌더러가 소비하므로(`renderer/style_resolver.rs:341`,`:355`) 같이 고치면 렌더 회귀
    /// 검증이 필요해진다. 그래서 별도 이슈로 남겼다 — 이 테스트가 그 경계를 읽히게 한다.
    #[test]
    fn char_shape_default_matches_spec_only_for_relative_sizes_and_shade() {
        let cs = CharShape::default();

        // 이번에 고친 것 — OWPML relSz default="100", 유효범위 10~250
        // (mydocs/manual/OWPML SCHEMA/Header XML schema.xml:716-728)
        assert_eq!(
            cs.relative_sizes, [100; 7],
            "상대크기 기본값은 OWPML 기본값 100 이어야 한다. 0 이면 한컴이 \
             `크기 × 상대크기%` 로 해석해 전 본문을 0.1pt 로 그린다 (#4141)"
        );

        // 의도적으로 고치지 않은 것 — 렌더러가 소비하므로 별도 이슈
        assert_eq!(
            cs.ratios, [0; 7],
            "장평 기본값을 바꾸려면 렌더 회귀 검증(Native Skia·시각 증적)이 필요하다. \
             #4141 범위 밖이며 별도 이슈로 다룬다 — 무심코 바꾸지 마라"
        );
        assert_eq!(
            cs.base_size, 0,
            "기준 크기도 렌더러가 소비한다 — 위와 같은 이유"
        );

        // 0 이 스펙상 유효값이라 손대지 않는 것
        // (OWPML offset default=0 범위 [-100,100], spacing default=0 범위 [-50,50])
        assert_eq!(cs.char_offsets, [0; 7]);
        assert_eq!(cs.spacings, [0; 7]);

        // 이번에 고친 것 — "음영 없음"은 색이 아니라 sentinel 이다. 파생값 0(검정)을
        // HWP5 로 저장하면 한컴이 글자마다 순검정 사각형을 칠한다 (#4155).
        assert_eq!(
            cs.shade_color,
            crate::model::color::NONE,
            "음영 없음 sentinel 은 한컴 HWP5·HWPX \"none\"·한/글 HML 4294967295 와 같은 \
             0xFFFFFFFF 다 (#4155). 0 으로 되돌리면 HWP3 변환본이 검정 막대가 된다"
        );

        // 0 이 그대로 유효한 나머지 색상값
        assert_eq!(cs.shadow_color, 0);
        assert_eq!(cs.underline_color, 0);
        assert_eq!(cs.text_color, 0);
    }

    #[test]
    fn test_alignment_variants() {
        assert_eq!(Alignment::default(), Alignment::Justify);
    }

    #[test]
    fn test_fill_type() {
        let fill = Fill::default();
        assert_eq!(fill.fill_type, FillType::None);
    }
}
