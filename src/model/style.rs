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
}

/// 글자 모양 (HWPTAG_CHAR_SHAPE)
#[derive(Debug, Clone, Default)]
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
    /// [#1984] HWPX breakSetting@breakLatinWord 원문 보존
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
