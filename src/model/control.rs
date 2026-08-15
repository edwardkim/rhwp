//! 인라인 컨트롤 (Ruby, Hyperlink, Field, Bookmark 등)

use std::collections::HashMap;

use super::document::SectionDef;
use super::footnote::{Endnote, Footnote};
use super::header_footer::{Footer, Header};
use super::image::Picture;
use super::page::ColumnDef;
use super::paragraph::Paragraph;
use super::shape::{CommonObjAttr, ShapeObject};
use super::table::Table;

/// 문단 내 컨트롤 (확장 컨트롤)
#[derive(Debug, Clone)]
pub enum Control {
    /// 구역 정의 ('secd')
    SectionDef(Box<SectionDef>),
    /// 단 정의 ('cold')
    ColumnDef(ColumnDef),
    /// 표 ('tbl ')
    Table(Box<Table>),
    /// 그리기 개체 ('$lin', '$rec', '$ell', '$arc', '$pol', '$cur')
    Shape(Box<ShapeObject>),
    /// 그림 ('$pic')
    Picture(Box<Picture>),
    /// 머리말 ('head')
    Header(Box<Header>),
    /// 꼬리말 ('foot')
    Footer(Box<Footer>),
    /// 각주 ('fn  ')
    Footnote(Box<Footnote>),
    /// 미주 ('en  ')
    Endnote(Box<Endnote>),
    /// 자동번호 ('atno')
    AutoNumber(AutoNumber),
    /// 새 번호 지정 ('nwno')
    NewNumber(NewNumber),
    /// 쪽 번호 위치 ('pgnp')
    PageNumberPos(PageNumberPos),
    /// 책갈피 ('bokm')
    Bookmark(Bookmark),
    /// 찾아보기 표식 ('idxm')
    IndexMark(IndexMark),
    /// 쪽 번호 시작 쪽 ('pgct')
    PageNumCtrl(PageNumCtrl),
    /// 하이퍼링크 ('%hlk')
    Hyperlink(Hyperlink),
    /// 덧말 ('tdut')
    Ruby(Ruby),
    /// 글자겹침 ('tcps')
    CharOverlap(CharOverlap),
    /// 감추기 ('pghd')
    PageHide(PageHide),
    /// 숨은 설명 ('tcmt')
    HiddenComment(Box<HiddenComment>),
    /// 수식 ('eqed')
    Equation(Box<Equation>),
    /// 필드 컨트롤 (다양한 필드 타입)
    Field(Field),
    /// 양식 개체 ('form' 컨트롤)
    Form(Box<FormObject>),
    /// 알 수 없는 컨트롤
    Unknown(UnknownControl),
}

impl Control {
    /// 개체가 글자처럼 취급(treat_as_char)되는가.
    ///
    /// 텍스트 흐름 안에서 한 글자 폭을 차지하는 개체 판정의 기반이다.
    pub fn is_treat_as_char_object(&self) -> bool {
        match self {
            Control::Shape(shape) => shape.common().treat_as_char,
            Control::Table(table) => table.common.treat_as_char,
            Control::Picture(picture) => picture.common.treat_as_char,
            Control::Equation(equation) => equation.common.treat_as_char,
            _ => false,
        }
    }

    /// 편집/커서 이동에서 논리적으로 한 글자 폭을 차지하는 컨트롤인가.
    ///
    /// `treat_as_char` 개체에 각주·미주를 더한 것. `SectionDef`/`ColumnDef` 같은
    /// 구조 컨트롤은 흐름에서 자리를 차지하지 않으므로 제외된다.
    pub fn is_logical_inline(&self) -> bool {
        self.is_treat_as_char_object() || matches!(self, Control::Footnote(_) | Control::Endnote(_))
    }

    /// HWP5 `PARA_TEXT` 에서 확장 제어문자 자리(`CTRL_CHAR_CODE_UNITS`)를 차지하는가.
    ///
    /// 직렬화기가 제어문자와 `CTRL_HEADER` 를 방출할지 가리는 규칙과 같은 원본이다
    /// (`serializer/body_text.rs::emits_ctrl_header`). 문단의 UTF-16 오프셋을 **계산하는**
    /// 쪽과 실제로 **쓰는** 쪽이 어긋나면 lineseg `text_start` 가 제어문자 블록 한가운데를
    /// 가리키고, 한글은 그런 문서의 본문을 통째로 버린다 (#4677).
    pub fn occupies_ctrl_char_slot(&self) -> bool {
        match self {
            Control::Hyperlink(_) => false,
            Control::Unknown(u) => u.ctrl_id != 0,
            _ => true,
        }
    }
}

/// HWP5 `PARA_TEXT` 에서 확장 제어문자 하나가 차지하는 UTF-16 코드유닛 수.
///
/// `[코드, ctrl_id 2유닛, 예약 4유닛, 코드]` = 8 유닛 (HWP5 스펙 표 58).
pub const CTRL_CHAR_CODE_UNITS: u32 = 8;

/// [#2727] `Equation::attr` 의 bit 0 — 수식이 차지하는 범위.
///
/// set = 줄 단위 (HWPX `lineMode="LINE"`), clear = 글자 단위 (`lineMode="CHAR"`).
pub const EQUATION_LINE_MODE_BIT: u32 = 0x0000_0001;

/// 수식 ('eqed' 컨트롤, HWP 스펙 표 105)
#[derive(Debug, Clone, Default)]
pub struct Equation {
    /// 개체 공통 속성 (위치, 크기, 배치)
    pub common: CommonObjAttr,
    /// [#2727] HWPTAG_EQEDIT 속성 (HWP5 spec 표 105 attribute, UINT32).
    ///
    /// bit 0 = 수식이 차지하는 범위 (0 = 글자 단위, 1 = 줄 단위) — HWPX
    /// `hp:equation@lineMode` (`CHAR` / `LINE`) 와 대응한다. 나머지 비트는 의미
    /// 미상이므로 UINT32 전체를 원본 그대로 보존해 왕복시킨다. 기본값 0 은
    /// OWPML `lineMode` 기본값 `CHAR` 와 일치한다.
    pub attr: u32,
    /// 수식 스크립트 ("1 over 2" 등)
    pub script: String,
    /// 글자 크기 (HWPUNIT)
    pub font_size: u32,
    /// 글자 색 (0x00BBGGRR)
    pub color: u32,
    /// 기준선 오프셋
    pub baseline: i16,
    /// 미지의 UINT16 필드 (HWP5 spec errata) — hwplib `ForEQEdit.readUInt2()` 정합.
    /// HWP5 spec 표 105 에 누락되어 있으나 한컴 실제 저장본에 baseline 과 version_info
    /// 사이에 UINT16 zero 가 위치. Task #1061 발견.
    pub unknown: u16,
    /// EQEDIT 속성 (UINT32, HWPTAG_EQEDIT 첫 필드).
    /// bit 0: lineMode (0=글자 단위/CHAR, 1=줄 단위/LINE).
    /// 종전엔 파싱 후 버려지고 저장 시 0으로 고정되어 lineMode 유실. Issue #2727.
    pub eqedit: u32,
    /// 버전 정보
    pub version_info: String,
    /// 수식 글꼴명
    pub font_name: String,
    /// 라운드트립용 원본 ctrl_data
    pub raw_ctrl_data: Vec<u8>,
}

/// 자동 번호 ('atno' 컨트롤, HWP 스펙 표 144)
#[derive(Debug, Clone, Default)]
pub struct AutoNumber {
    /// 번호 종류 (각주, 미주, 그림, 표, 수식)
    pub number_type: AutoNumberType,
    /// 번호 형식 (표 145 bit 4~11, 표 134 참조)
    pub format: u8,
    /// 위 첨자 여부 (표 145 bit 12)
    pub superscript: bool,
    /// 할당된 번호 (파싱 시점에 결정됨)
    pub assigned_number: u16,
    /// 스펙상 번호 (UINT16)
    pub number: u16,
    /// 사용자 기호 (WCHAR)
    pub user_symbol: char,
    /// 앞 장식 문자 (WCHAR)
    pub prefix_char: char,
    /// 뒤 장식 문자 (WCHAR)
    pub suffix_char: char,
}

/// 자동 번호 종류
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum AutoNumberType {
    #[default]
    Page,
    Footnote,
    Endnote,
    Picture,
    Table,
    Equation,
    /// 총 쪽수 (표 144 bits 0~3 값 6) — 현재 쪽번호가 아니라 문서 전체 쪽수를 표시.
    TotalPage,
}

/// 새 번호 지정 ('nwno' 컨트롤)
#[derive(Debug, Clone, Default)]
pub struct NewNumber {
    /// 번호 종류
    pub number_type: AutoNumberType,
    /// 새 번호
    pub number: u16,
}

/// 쪽 번호 위치 ('pgnp' 컨트롤, HWP 스펙 표 149)
#[derive(Debug, Clone, Default)]
pub struct PageNumberPos {
    /// 번호 형식 (표 150 bit 0~7, 표 134 참조)
    pub format: u8,
    /// 위치 (표 150 bit 8~11)
    pub position: u8,
    /// 사용자 기호 (WCHAR)
    pub user_symbol: char,
    /// 앞 장식 문자 (WCHAR)
    pub prefix_char: char,
    /// 뒤 장식 문자 (WCHAR)
    pub suffix_char: char,
    /// 대시 문자 (WCHAR, 항상 '-')
    pub dash_char: char,
}

/// 책갈피 ('bokm' 컨트롤)
#[derive(Debug, Clone, Default)]
pub struct Bookmark {
    /// 책갈피 이름
    pub name: String,
}

/// 쪽 번호가 시작되는 쪽 ('pgct') — `<hp:pageNumCtrl pageStartsOn>` 에 대응한다.
///
/// HWP5 CTRL_HEADER payload 는 ctrl_id 뒤 `u32` 하나뿐이다. 한글 2022 양방향 실측
/// (06731 을 HWPX 로 저장 → 속성만 바꿔 다시 HWP5 로 저장, 각 17/17):
///
/// | u32 | `pageStartsOn` |
/// |---|---|
/// | 0 | `BOTH` |
/// | 1 | `EVEN` |
/// | 2 | `ODD` |
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PageStartsOn {
    /// 양쪽 (기본값)
    #[default]
    Both,
    /// 짝수 쪽
    Even,
    /// 홀수 쪽
    Odd,
}

impl PageStartsOn {
    /// HWP5 payload u32 → 열거. 규정 밖 값은 기본값으로 떨어뜨린다.
    pub fn from_hwp5(v: u32) -> Self {
        match v {
            1 => Self::Even,
            2 => Self::Odd,
            _ => Self::Both,
        }
    }

    /// 열거 → HWP5 payload u32.
    pub fn to_hwp5(self) -> u32 {
        match self {
            Self::Both => 0,
            Self::Even => 1,
            Self::Odd => 2,
        }
    }

    /// OWPML `pageStartsOn` 속성값.
    pub fn as_hwpx(self) -> &'static str {
        match self {
            Self::Both => "BOTH",
            Self::Even => "EVEN",
            Self::Odd => "ODD",
        }
    }

    /// OWPML `pageStartsOn` 속성값 → 열거.
    pub fn from_hwpx(v: &str) -> Self {
        match v {
            "EVEN" => Self::Even,
            "ODD" => Self::Odd,
            _ => Self::Both,
        }
    }
}

/// 쪽 번호 시작 쪽 컨트롤 ('pgct')
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PageNumCtrl {
    /// 쪽 번호가 시작되는 쪽
    pub page_starts_on: PageStartsOn,
}

/// 찾아보기 표식 ('idxm') — 색인에 실릴 키워드를 본문 위치에 붙여 둔 표식.
///
/// HWP5 CTRL_DATA 레이아웃(ctrl_id 4바이트 제거 후, 실측 06926):
///   WORD(2) + WCHAR[n]  첫째 키
///   WORD(2) + WCHAR[m]  둘째 키
///   4바이트 예약(실측 전부 0)
///
/// HWPX 는 `<hp:ctrl><hp:indexmark><hp:firstKey/><hp:secondKey/></hp:indexmark></hp:ctrl>`
/// 로 적는다(ParaList XML schema.xml:209).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexMark {
    /// 첫째 키
    pub first_key: String,
    /// 둘째 키 (없으면 빈 문자열)
    pub second_key: String,
}

/// 하이퍼링크 ('%hlk' 필드)
#[derive(Debug, Clone, Default)]
pub struct Hyperlink {
    /// URL
    pub url: String,
    /// 표시 텍스트
    pub text: String,
}

/// 덧말 ('tdut' 컨트롤)
#[derive(Debug, Clone, Default)]
pub struct Ruby {
    /// 기준 텍스트 (`<hp:mainText>`) — 덧말이 달리는 본문 글자. (#1587)
    /// 파서가 para.text 에 넣지 않고 여기 보존한다(시각 충실도 핵심).
    pub main_text: String,
    /// 덧말 텍스트 (`<hp:subText>`)
    pub ruby_text: String,
    /// 위치 (`posType`): 0=TOP, 1=BOTTOM. (#1587)
    pub pos_type: u8,
    /// 정렬 (`align`): 0=LEFT, 1=RIGHT, 2=CENTER. (#1587)
    pub align: u8,
    /// 덧말 크기 비율 (`szRatio`, %). (#1587)
    pub sz_ratio: u8,
    /// 옵션 비트 (`option`). (#1587)
    pub option: u32,
    /// 글자 스타일 참조 (`styleIDRef`). (#1587)
    pub style_id_ref: u16,
}

/// 글자 겹침 ('tcps' 컨트롤, HWP 스펙 표 152)
#[derive(Debug, Clone, Default)]
pub struct CharOverlap {
    /// 겹칠 글자 목록 (최대 9글자)
    pub chars: Vec<char>,
    /// 테두리 타입 (0=없음/글자끼리, 1=원, 2=반전원, 3=사각형, 4=반전사각형)
    pub border_type: u8,
    /// 내부 글자 크기 (%, 양수=축소/확대, 기본 100)
    pub inner_char_size: i8,
    /// 펼침
    pub expansion: u8,
    /// 글자 속성(charshape) ID 배열
    pub char_shape_ids: Vec<u32>,
}

/// 감추기 ('pghd' 컨트롤)
#[derive(Debug, Clone, Default)]
pub struct PageHide {
    /// 머리말 감추기
    pub hide_header: bool,
    /// 꼬리말 감추기
    pub hide_footer: bool,
    /// 바탕쪽 감추기
    pub hide_master_page: bool,
    /// 테두리 감추기
    pub hide_border: bool,
    /// 배경 감추기
    pub hide_fill: bool,
    /// 쪽 번호 감추기
    pub hide_page_num: bool,
}

/// 숨은 설명 ('tcmt' 컨트롤)
#[derive(Debug, Default, Clone)]
pub struct HiddenComment {
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
}

/// 초기 상태 누름틀에서 적재 정규화가 제거한 안내문 본문 run 의 원본 형상 (#3545).
///
/// 편집 IR 은 빈 필드로 정규화된 상태가 authoritative 이고, 이 구조체는 **저장 시
/// 원본 파일 형상을 되돌리기 위한 파생 상태**다 (값 API·렌더는 이 값을 보지 않는다).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuideResidue {
    /// 제거된 본문 텍스트 원문 (한컴이 안내문 뒤에 붙이는 trailing 공백까지 그대로).
    pub text: String,
    /// 그 텍스트를 담던 run 의 `charPrIDRef` — 복원 시 원본 서식을 함께 되살린다.
    pub char_shape_id: u32,
}

/// HWPX `<hp:parameters>`/`<hp:listParam>` 트리의 원소 하나 (#4396).
///
/// OWPML `hp:ParameterList`(`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:2764`)가
/// 이름·타입·구조를 규정한다 — booleanParam/integerParam/floatParam/stringParam 4종의
/// 스칼라와 재귀하는 listParam 1종, 도합 5종. 각 파라미터의 **의미는 해석하지 않고**
/// 이름과 타입 그대로 보존한다(HWP5 왕복 시 `Prop`/`Direction`/`Path`/`Category` 등이
/// `Command` 하나로 축소되던 손실의 근본 수정).
#[derive(Debug, Clone, PartialEq)]
pub enum Parameter {
    Boolean {
        name: Option<String>,
        value: bool,
    },
    Integer {
        name: Option<String>,
        value: i64,
    },
    Float {
        name: Option<String>,
        value: f32,
    },
    String {
        name: Option<String>,
        value: String,
        /// 원본 `xml:space="preserve"` 표시 여부. stringParam 만 갖는 속성(스키마).
        preserve_space: bool,
    },
    /// 재귀 `hp:listParam` — 이름은 중첩된 `ParameterList::name` 이 갖는다(이중 보관 방지).
    List(ParameterList),
}

/// `hp:ParameterList` 자체 — `<hp:parameters>`(필드 루트) 또는 `<hp:listParam>`(중첩) 둘 다
/// 이 표현을 쓴다. `cnt` 속성은 `items.len()` 에서 유도하므로 별도 저장하지 않는다.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParameterList {
    pub name: Option<String>,
    pub items: Vec<Parameter>,
}

impl ParameterList {
    /// 값을 하나도 담지 않은 트리인지 — `Field::parameters` 미보존/미확보 상태의 판별에 쓴다.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 캐노니컬 텍스트 형태로 직렬화한다(`<hp:{tag} cnt="N" name="...">...</hp:{tag}>`).
    ///
    /// 이 텍스트 형태는 HWPX 직렬화기와 HWP5 CTRL_DATA 보존(#4396) 양쪽이 공유하는
    /// 포맷 중립 인코딩이다 — OWPML 문법을 그대로 따르므로 HWPX 파서(`parser::hwpx::section`)
    /// 의 트리 파서로 되읽을 수 있고, HWP5 CTRL_DATA 아이템 문자열로도 그대로 저장할 수 있다.
    /// `tag` 는 루트 호출 시 `"parameters"`, 중첩 리스트는 `"listParam"`.
    pub fn render_xml(&self, tag: &str) -> String {
        let mut out = String::new();
        out.push_str("<hp:");
        out.push_str(tag);
        out.push_str(&format!(" cnt=\"{}\"", self.items.len().max(1)));
        out.push_str(" name=\"");
        out.push_str(&escape_parameter_xml(self.name.as_deref().unwrap_or("")));
        out.push_str("\">");
        for item in &self.items {
            item.render_xml_into(&mut out);
        }
        out.push_str("</hp:");
        out.push_str(tag);
        out.push('>');
        out
    }
}

impl Parameter {
    fn render_xml_into(&self, out: &mut String) {
        match self {
            Parameter::Boolean { name, value } => {
                render_scalar_param(out, "booleanParam", name, if *value { "1" } else { "0" });
            }
            Parameter::Integer { name, value } => {
                render_scalar_param(out, "integerParam", name, &value.to_string());
            }
            Parameter::Float { name, value } => {
                render_scalar_param(out, "floatParam", name, &value.to_string());
            }
            Parameter::String {
                name,
                value,
                preserve_space,
            } => {
                out.push_str("<hp:stringParam name=\"");
                out.push_str(&escape_parameter_xml(name.as_deref().unwrap_or("")));
                out.push('"');
                if *preserve_space {
                    out.push_str(" xml:space=\"preserve\"");
                }
                out.push('>');
                out.push_str(&escape_parameter_xml(value));
                out.push_str("</hp:stringParam>");
            }
            Parameter::List(list) => out.push_str(&list.render_xml("listParam")),
        }
    }
}

fn render_scalar_param(out: &mut String, tag: &str, name: &Option<String>, text: &str) {
    out.push_str("<hp:");
    out.push_str(tag);
    out.push_str(" name=\"");
    out.push_str(&escape_parameter_xml(name.as_deref().unwrap_or("")));
    out.push_str("\">");
    out.push_str(text);
    out.push_str("</hp:");
    out.push_str(tag);
    out.push('>');
}

/// `ParameterList::render_xml` 전용 최소 XML 텍스트/속성값 이스케이프.
/// `parser::hwpx::section::escape_xml_text` 와 동일한 계약(4종 예약 문자 + 제어문자 제거).
fn escape_parameter_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\u{09}' | '\u{0A}' | '\u{0D}' => out.push(c),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                out.push(c)
            }
            _ => {}
        }
    }
    out
}

impl ParameterList {
    /// `render_xml` 이 만든 캐노니컬 텍스트를 다시 트리로 되읽는다 (HWP5 CTRL_DATA
    /// 확장 아이템 복원용, #4396).
    ///
    /// **우리 자신이 `render_xml` 로 만든 문자열만** 상대하는 최소 파서다 — 임의 XML
    /// (주석·CDATA·일반 엔터티 참조·속성 순서 변형 등)은 다루지 않는다. 이 코덱은
    /// rhwp 엔진 내부 왕복 전용이며 실물 HWPX 파일을 파싱하지 않는다(그건
    /// `parser::hwpx::section::parse_field_parameters` 의 몫 — quick_xml 기반으로
    /// 별도 구현돼 있다).
    pub fn parse_xml(xml: &str) -> Option<ParameterList> {
        let mut pos = 0usize;
        let (_, list) = parse_list_tag(xml, &mut pos)?;
        Some(list)
    }
}

/// `<hp:TAG cnt="N" name="...">...items...</hp:TAG>` 하나를 파싱한다.
/// 진입 시 `s[*pos..]` 는 반드시 `<hp:` 로 시작해야 한다. 반환: (태그 이름, 트리).
fn parse_list_tag(s: &str, pos: &mut usize) -> Option<(String, ParameterList)> {
    expect_lit(s, pos, "<hp:")?;
    let tag = read_ident(s, pos)?;
    skip_attr(s, pos, "cnt")?;
    let name = read_attr(s, pos, "name")?;
    expect_lit(s, pos, ">")?;
    let close = format!("</hp:{tag}>");
    let mut items = Vec::new();
    while !s[*pos..].starts_with(&close) {
        items.push(parse_one_param(s, pos)?);
    }
    *pos += close.len();
    Some((
        tag,
        ParameterList {
            name: Some(name),
            items,
        },
    ))
}

fn parse_one_param(s: &str, pos: &mut usize) -> Option<Parameter> {
    if !s[*pos..].starts_with("<hp:") {
        return None;
    }
    let after = &s[*pos + 4..];
    if after.starts_with("listParam") {
        let (_, list) = parse_list_tag(s, pos)?;
        return Some(Parameter::List(list));
    }
    for tag in ["booleanParam", "integerParam", "floatParam", "stringParam"] {
        if !after.starts_with(tag) {
            continue;
        }
        *pos += 4 + tag.len();
        let name = read_attr(s, pos, "name")?;
        let preserve_space =
            tag == "stringParam" && try_consume_lit(s, pos, " xml:space=\"preserve\"");
        expect_lit(s, pos, ">")?;
        let close = format!("</hp:{tag}>");
        let raw_text = read_until(s, pos, &close)?;
        expect_lit(s, pos, &close)?;
        let text = unescape_parameter_xml(&raw_text);
        return Some(match tag {
            "booleanParam" => Parameter::Boolean {
                name: Some(name),
                value: matches!(text.trim(), "1" | "true"),
            },
            "integerParam" => Parameter::Integer {
                name: Some(name),
                value: text.trim().parse::<i64>().unwrap_or(0),
            },
            "floatParam" => Parameter::Float {
                name: Some(name),
                value: text.trim().parse::<f32>().unwrap_or(0.0),
            },
            _ => Parameter::String {
                name: Some(name),
                value: text,
                preserve_space,
            },
        });
    }
    None
}

fn read_ident(s: &str, pos: &mut usize) -> Option<String> {
    let rest = &s[*pos..];
    let end = rest.find(|c: char| c == ' ' || c == '>')?;
    let ident = rest[..end].to_string();
    *pos += end;
    Some(ident)
}

/// ` key="value"` 속성을 값은 버리고 건너뛴다(`cnt` — `items.len()` 에서 유도되므로).
fn skip_attr(s: &str, pos: &mut usize, key: &str) -> Option<()> {
    let needle = format!(" {key}=\"");
    expect_lit(s, pos, &needle)?;
    let end = s[*pos..].find('"')?;
    *pos += end + 1;
    Some(())
}

/// ` key="value"` 속성을 읽어 이스케이프를 되돌린 값을 반환한다.
fn read_attr(s: &str, pos: &mut usize, key: &str) -> Option<String> {
    let needle = format!(" {key}=\"");
    expect_lit(s, pos, &needle)?;
    let end = s[*pos..].find('"')?;
    let value = unescape_parameter_xml(&s[*pos..*pos + end]);
    *pos += end + 1;
    Some(value)
}

fn expect_lit(s: &str, pos: &mut usize, lit: &str) -> Option<()> {
    if !s[*pos..].starts_with(lit) {
        return None;
    }
    *pos += lit.len();
    Some(())
}

fn try_consume_lit(s: &str, pos: &mut usize, lit: &str) -> bool {
    if s[*pos..].starts_with(lit) {
        *pos += lit.len();
        true
    } else {
        false
    }
}

/// `*pos` 부터 `marker` 직전까지를 반환한다. `*pos` 는 `marker` 시작 위치로 이동한다
/// (소비는 호출부가 `expect_lit` 로 한다).
fn read_until(s: &str, pos: &mut usize, marker: &str) -> Option<String> {
    let idx = s[*pos..].find(marker)?;
    let text = s[*pos..*pos + idx].to_string();
    *pos += idx;
    Some(text)
}

/// `escape_parameter_xml` 의 역변환 — 우리 자신이 방출하는 4종 예약 문자 엔터티만 되돌린다.
fn unescape_parameter_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s.as_bytes()[i] == b'&' {
            if s[i..].starts_with("&amp;") {
                out.push('&');
                i += 5;
                continue;
            } else if s[i..].starts_with("&lt;") {
                out.push('<');
                i += 4;
                continue;
            } else if s[i..].starts_with("&gt;") {
                out.push('>');
                i += 4;
                continue;
            } else if s[i..].starts_with("&quot;") {
                out.push('"');
                i += 6;
                continue;
            }
        }
        let ch_len = s[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        out.push_str(&s[i..i + ch_len]);
        i += ch_len;
    }
    out
}

#[cfg(test)]
mod parameter_list_codec_tests {
    use super::*;

    fn sample_tree() -> ParameterList {
        ParameterList {
            name: Some(String::new()),
            items: vec![
                Parameter::Integer {
                    name: Some("Prop".to_string()),
                    value: 9,
                },
                Parameter::String {
                    name: Some("Command".to_string()),
                    value: "Clickhere:set:66:Direction:wstring:23:이곳을 마우스로 누르고 <입력>하세요 & \"확인\""
                        .to_string(),
                    preserve_space: true,
                },
                Parameter::String {
                    name: Some("Direction".to_string()),
                    value: "이곳을 마우스로 누르고 내용을 입력하세요.".to_string(),
                    preserve_space: false,
                },
                Parameter::Boolean {
                    name: Some("Fiexde".to_string()),
                    value: true,
                },
                Parameter::Boolean {
                    name: Some("RefHyperLink".to_string()),
                    value: false,
                },
                Parameter::Float {
                    name: Some("Ratio".to_string()),
                    value: 1.5,
                },
                Parameter::List(ParameterList {
                    name: Some("623".to_string()),
                    items: vec![Parameter::Integer {
                        name: Some("16384".to_string()),
                        value: 42,
                    }],
                }),
            ],
        }
    }

    #[test]
    fn render_then_parse_round_trips_the_tree() {
        let original = sample_tree();
        let xml = original.render_xml("parameters");
        assert!(xml.starts_with("<hp:parameters cnt=\"7\" name=\"\">"));
        assert!(xml.ends_with("</hp:parameters>"));
        let parsed = ParameterList::parse_xml(&xml).expect("parse_xml 실패");
        assert_eq!(parsed, original);
    }

    #[test]
    fn empty_list_round_trips() {
        let original = ParameterList {
            name: Some(String::new()),
            items: vec![],
        };
        let xml = original.render_xml("parameters");
        let parsed = ParameterList::parse_xml(&xml).expect("parse_xml 실패");
        assert_eq!(parsed, original);
    }

    #[test]
    fn is_empty_reflects_items() {
        assert!(ParameterList::default().is_empty());
        assert!(!sample_tree().is_empty());
    }
}

/// 필드 컨트롤
#[derive(Debug, Clone, Default)]
pub struct Field {
    /// 필드 타입
    pub field_type: FieldType,
    /// 필드 이름/명령 (누름틀: 안내문, 하이퍼링크: URL 등)
    pub command: String,
    /// 속성 비트필드 (표 155)
    pub properties: u32,
    /// 기타 속성
    pub extra_properties: u8,
    /// 문서 내 고유 ID
    pub field_id: u32,
    /// 원본 ctrl_id (직렬화용)
    pub ctrl_id: u32,
    /// HWPX `<hp:fieldBegin fieldid="..">` 원본 값 (동종 필드 간 공유되는 instance id).
    /// `id`(=field_id, 문서 내 고유)와 별개 값이며 실물 파일에서 서로 다를 수 있다(#1512).
    /// `None` 이면 fieldid 속성 자체가 없었거나 파서가 값을 못 읽은 경우 — 방출 생략.
    pub instance_id: Option<u32>,
    /// CTRL_DATA에서 읽은 필드 이름 (누름틀 고치기에서 설정)
    pub ctrl_data_name: Option<String>,
    /// 메모 인덱스 (hwplib: memoIndex)
    pub memo_index: u32,
    /// 메모 본문 문단 리스트 (`fieldBegin type="MEMO"` 내부 subList)
    pub memo_paragraphs: Vec<Paragraph>,
    /// 메모 본문 subList 의 `textDirection` 속성 (예: "VERTICAL"). 세로쓰기 메모가
    /// 왕복 시 가로쓰기로 뒤집히지 않도록 원본 값을 보존한다.
    /// `None` 이면 기본값 "HORIZONTAL" 방출.
    pub memo_text_direction: Option<String>,
    /// HWPX `<hp:parameters>` 요소 원문 verbatim (#1391).
    ///
    /// HWPX→HWPX 왕복 전용 바이트 정확 캐시다 — 이미 코퍼스 전수(3,418건)로 검증된
    /// `diff_documents`/`strip_cross_format_noise` 원문 비교 계약(`serializer/hwpx/roundtrip.rs`)
    /// 이 이 필드의 문자열을 그대로 비교하므로, 값이 있으면 직렬화기가 그 바이트를 그대로
    /// 재사용한다 (HWP5 경로엔 안 쓴다 — HWPX 파서만 적재).
    ///
    /// `parameters`(아래, #4396)가 같은 `<hp:parameters>` 를 **의미 있는 트리**로 담는
    /// 별도 표현이다 — 중복이 아니라 용도가 다르다: 이 필드는 "포맷을 벗어나지 않을 때
    /// 원문을 한 글자도 안 바꾼다"는 이미 검증된 계약을 지키는 캐시이고, `parameters`
    /// 는 그 계약이 성립하지 않는 경로(HWPX 파싱 직후, 또는 API 로 새로 만든 필드)에서도
    /// `Command` 외 나머지 named param 을 의미 있게 들고 있기 위한 근거 데이터다. 파서는
    /// 두 필드를 같은 파스 1회에서 함께 채운다.
    pub raw_parameters_xml: Option<String>,
    /// `<hp:parameters>` 트리의 파싱된(named) 표현 (#4396). 스펙(`ParameterList`)이 규정한
    /// 5종 그대로 보존 — 의미 해석 없이 이름·타입·값만 옮긴다.
    ///
    /// HWPX 파서는 항상 이 필드를 채운다(전 fieldBegin 타입). **HWP5 파서는 채우지
    /// 않는다** — `pdf/hwpspec-2024.pdf` §4.2.8(HWPTAG_CTRL_DATA, 표 61)·
    /// §4.2.10.11(책갈피)·§4.2.10.15(필드)를 확인한 결과, HWP5 CTRL_DATA 의
    /// ParameterSet 은 "필드 이름"(item_id=0x4000, hwplib 실측) 외에는 필드별 item ID
    /// 스키마가 스펙에 규정돼 있지 않다. 스펙에 없는 값을 임의로 정해 써넣는 방법도
    /// 검토했으나(`item_id=0x4010`) `document_core::converters::hwpx_to_hwp` 의
    /// `0x4000+idx` 순차 할당과 충돌 가능성이 있어 되돌렸다(#4396 리뷰) — 검증되지 않은
    /// 바이트를 실물 HWP5 파일에 심지 않는다는 판단이다.
    ///
    /// 따라서 HWPX→HWP5→HWPX 왕복에서는 `command`/`memo_index`(HWP5 CTRL_HEADER 에
    /// 실제 슬롯이 있는 항목)를 뺀 나머지가 손실된다 — `serializer::control` 의
    /// `field_parameter_loss_warning` 이 그 손실을 조용히 넘기지 않고 경고로 낸다.
    /// 직렬화기는 `raw_parameters_xml` 이 있으면 그것을 우선 쓰고, 없고 이 트리가
    /// 비어있지 않으면(예: HWPX→HWPX 재직렬화, API 로 트리를 직접 채운 필드) `render_xml`
    /// 로 재조립하며, 둘 다 없으면 기존처럼 `command` 하나만 담은 최소
    /// `<hp:parameters cnt="1">` 를 합성한다.
    pub parameters: ParameterList,
    /// [#3545] 적재 정규화(`clear_initial_field_texts`)가 지운 안내문 본문 run 의 잔재.
    ///
    /// 한컴은 미기입 누름틀(dirty="0")의 안내문을 **파일에는 본문 run 으로 유지**하고
    /// 렌더·인쇄에서만 구분 취급한다 (`samples/hwpx/form-01.hwpx` 의
    /// `<hp:run charPrIDRef="6"><hp:t>여기에 입력</hp:t></hp:run>`). rhwp 는 편집 IR 을
    /// 빈 필드로 정규화하므로, 저장에서 이 잔재를 되살리지 않으면 파일 차원에서
    /// 텍스트가 영구 소실된다. 정규화 계약(값 API 는 빈 값)은 그대로 두고 HWPX 저장
    /// 시에만 원본 run 을 복원한다.
    pub guide_residue: Option<GuideResidue>,
}

impl Field {
    /// 누름틀(ClickHere) command에서 안내문(Direction) 텍스트를 추출한다.
    ///
    /// command 형식: "Clickhere:set:{len}:Direction:wstring:{n}:{text} HelpState:..."
    pub fn guide_text(&self) -> Option<&str> {
        if self.field_type != FieldType::ClickHere {
            return None;
        }
        self.extract_wstring_value("Direction:")
    }

    /// 누름틀(ClickHere) 필드 이름을 반환한다.
    ///
    /// 우선순위: CTRL_DATA name → command Name: 키 → 안내문(Direction) 폴백
    pub fn field_name(&self) -> Option<&str> {
        if self.field_type != FieldType::ClickHere {
            return None;
        }
        // 1. CTRL_DATA에서 읽은 필드 이름
        if let Some(ref name) = self.ctrl_data_name {
            if !name.is_empty() {
                return Some(name.as_str());
            }
        }
        // 2. command 내 Name: 키
        if let Some(name) = self.extract_wstring_value("Name:") {
            return Some(name);
        }
        // 3. 안내문을 대체 이름으로 사용
        self.extract_wstring_value("Direction:")
    }

    /// command 문자열에서 "{key}wstring:{n}:{value}" 패턴의 값을 추출한다.
    pub fn extract_wstring_value(&self, key: &str) -> Option<&str> {
        let key_start = self.command.find(key)?;
        let after_key = &self.command[key_start + key.len()..];
        // "wstring:{n}:" 패턴에서 값 시작 위치 찾기
        let wstring_marker = "wstring:";
        let ws_start = after_key.find(wstring_marker)? + wstring_marker.len();
        let rest = &after_key[ws_start..];
        let colon_pos = rest.find(':')?;
        let value_start = key_start + key.len() + ws_start + colon_pos + 1;
        let value_part = &self.command[value_start..];
        // 다음 키워드(" HelpState:", " Direction:", " Name:" 등)까지
        let end = value_part
            .find(" HelpState:")
            .or_else(|| value_part.find(" Direction:"))
            .or_else(|| value_part.find(" Name:"))
            .unwrap_or(value_part.len());
        let value = value_part[..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    /// 누름틀(ClickHere) command에서 메모(HelpState) 텍스트를 추출한다.
    pub fn memo_text(&self) -> Option<&str> {
        if self.field_type != FieldType::ClickHere {
            return None;
        }
        self.extract_wstring_value("HelpState:")
    }

    /// 양식 모드에서 편집 가능 여부 (properties bit 0)
    pub fn is_editable_in_form(&self) -> bool {
        self.properties & 1 != 0
    }

    /// 수정됨(dirty) 표식 (properties bit 15) — `clear_initial_field_texts` 의 보존 게이트.
    /// HWPX `<hp:fieldBegin dirty="..">` 가 이 비트의 대응물이다 (#3545).
    pub fn is_dirty(&self) -> bool {
        self.properties & (1 << 15) != 0
    }

    /// 누름틀(ClickHere) command 문자열을 한컴 포맷으로 재구축한다.
    ///
    /// 한컴 정답지(`samples/field-01.hwp`, `form-01.hwp`) 동형 (#1434):
    /// ```text
    /// Clickhere:set:{N}:Direction:wstring:{gl}:{guide} HelpState:wstring:{ml}:{memo}␣␣
    /// ```
    /// - HelpState 값 뒤 공백 2개 (구분 1 + trailing 1).
    /// - `set` 길이 N = inner 글자수 − 1 (마지막 trailing 공백 제외).
    /// - 필드 이름(Name)은 command 에 넣지 않는다 — CTRL_DATA 레코드(0x57) 전담.
    ///   (이전엔 Name 키를 넣어 한컴이 Direction 범위를 잘못 잘라 안내문 바인딩 실패.)
    pub fn build_clickhere_command(guide: &str, memo: &str) -> String {
        // §4.5: wstring {n} 은 UTF-16 code unit(WCHAR) 수다. chars().count()(스칼라 수)는
        // BMP 밖 문자(이모지·CJK 확장한자 등)를 문자당 1개 적게 세어, 한컴이 안내문/메모
        // 범위를 잘못 해석한다.
        let guide_len = guide.encode_utf16().count();
        let memo_len = memo.encode_utf16().count();

        // HelpState 값 뒤 공백 2개 (한컴 정답지 동형).
        let inner = format!(
            "Direction:wstring:{}:{} HelpState:wstring:{}:{}  ",
            guide_len, guide, memo_len, memo
        );
        // set 길이도 WCHAR 수 기준. 마지막 trailing 공백 1개 제외(-1) — 공백은 BMP 라 유지.
        let set_len = inner.encode_utf16().count() - 1;
        format!("Clickhere:set:{}:{}", set_len, inner)
    }

    /// FieldType을 문자열로 변환한다.
    pub fn field_type_str(&self) -> &'static str {
        match self.field_type {
            FieldType::Unknown => "unknown",
            FieldType::Date => "date",
            FieldType::DocDate => "docdate",
            FieldType::Path => "path",
            FieldType::Bookmark => "bookmark",
            FieldType::MailMerge => "mailmerge",
            FieldType::CrossRef => "crossref",
            FieldType::Formula => "formula",
            FieldType::ClickHere => "clickhere",
            FieldType::Summary => "summary",
            FieldType::UserInfo => "userinfo",
            FieldType::Hyperlink => "hyperlink",
            FieldType::Memo => "memo",
            FieldType::PrivateInfoSecurity => "privateinfo",
            FieldType::TableOfContents => "toc",
        }
    }
}

/// 필드 타입
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FieldType {
    #[default]
    Unknown,
    Date,
    DocDate,
    Path,
    Bookmark,
    MailMerge,
    CrossRef,
    Formula,
    ClickHere,
    Summary,
    UserInfo,
    Hyperlink,
    Memo,
    PrivateInfoSecurity,
    TableOfContents,
}

/// 양식 개체 타입
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub enum FormType {
    /// 명령 단추
    #[default]
    PushButton,
    /// 선택 상자
    CheckBox,
    /// 목록 상자
    ComboBox,
    /// 라디오 단추
    RadioButton,
    /// 입력 상자
    Edit,
}

/// 양식 개체 ('form' 컨트롤, ctrl_id=0x666f726d)
#[derive(Debug, Clone, Default)]
pub struct FormObject {
    /// 양식 개체 타입
    pub form_type: FormType,
    /// 개체 이름
    pub name: String,
    /// 캡션 (PushButton, CheckBox, RadioButton)
    pub caption: String,
    /// 텍스트 내용 (ComboBox, Edit)
    pub text: String,
    /// 너비 (HWPUNIT)
    pub width: u32,
    /// 높이 (HWPUNIT)
    pub height: u32,
    /// 글자 색 (0x00BBGGRR)
    pub fore_color: u32,
    /// 배경 색 (0x00BBGGRR)
    pub back_color: u32,
    /// 선택 상태 (CheckBox/RadioButton: 0=해제, 1=선택)
    pub value: i32,
    /// 활성화 여부
    pub enabled: bool,
    /// 기타 속성 (원본 키-값 보존)
    pub properties: HashMap<String, String>,
}

/// 알 수 없는 컨트롤
#[derive(Debug, Clone, Default)]
pub struct UnknownControl {
    /// 컨트롤 ID
    pub ctrl_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_variants() {
        let ctrl = Control::Bookmark(Bookmark {
            name: "test".to_string(),
        });
        match ctrl {
            Control::Bookmark(bm) => assert_eq!(bm.name, "test"),
            _ => panic!("Expected Bookmark"),
        }
    }

    #[test]
    fn test_field_type_default() {
        assert_eq!(FieldType::default(), FieldType::Unknown);
    }

    // ---------- #1434: 누름틀 command 한컴 포맷 정합 ----------

    /// 한컴 정답지(field-01/form-01)의 command 문자열과 바이트 동형이어야 한다.
    #[test]
    fn task1434_clickhere_command_hancom_format() {
        // 한컴 원본: `Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  `
        assert_eq!(
            Field::build_clickhere_command("여기에 입력", ""),
            "Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  "
        );
        // 한컴 원본: `Clickhere:set:47:Direction:wstring:5:제목 입력 HelpState:wstring:0:  `
        assert_eq!(
            Field::build_clickhere_command("제목 입력", ""),
            "Clickhere:set:47:Direction:wstring:5:제목 입력 HelpState:wstring:0:  "
        );
    }

    /// command 에 Name 키가 들어가면 안 된다 (이름은 CTRL_DATA 전담). 한컴이 Name 키를
    /// 만나면 Direction 범위를 잘못 잘라 안내문 바인딩 실패 (#1434 회귀 가드).
    #[test]
    fn task1434_command_has_no_name_key() {
        let cmd = Field::build_clickhere_command("여기에 입력", "");
        assert!(
            !cmd.contains("Name:"),
            "command 에 Name 키가 있으면 한컴 안내문 바인딩 실패: {cmd}"
        );
    }

    /// 생성한 command 에서 guide_text/memo_text 가 정확히 재추출되어야 한다 (왕복 정합).
    #[test]
    fn task1434_command_guide_memo_roundtrip() {
        let field = Field {
            field_type: FieldType::ClickHere,
            command: Field::build_clickhere_command("여기에 입력", "도움말"),
            ..Default::default()
        };
        assert_eq!(field.guide_text(), Some("여기에 입력"));
        assert_eq!(field.memo_text(), Some("도움말"));
    }

    /// set 길이는 inner 글자수 − 1 (마지막 trailing 공백 제외) 규칙을 따른다.
    #[test]
    fn task1434_set_length_excludes_trailing_space() {
        let cmd = Field::build_clickhere_command("a", "");
        // inner = "Direction:wstring:1:a HelpState:wstring:0:  " (trailing 2개)
        let inner = cmd.strip_prefix("Clickhere:set:").unwrap();
        let (set_str, body) = inner.split_once(':').unwrap();
        let set_len: usize = set_str.parse().unwrap();
        assert_eq!(
            set_len,
            body.chars().count() - 1,
            "set 길이 = inner 글자수 − 1 (trailing 공백 제외): {cmd}"
        );
    }

    /// §4.5: wstring {n}·set {N} 은 UTF-16 code unit(WCHAR) 수여야 한다.
    /// BMP 밖 문자(U+20000)는 2 WCHAR 이므로 chars().count() 로는 1개 적게 세어진다.
    #[test]
    fn clickhere_wstring_len_counts_utf16_wchars() {
        // guide "a𠀀b": a(1) + U+20000(2 WCHAR) + b(1) = 4 WCHAR (chars().count()=3)
        let cmd = Field::build_clickhere_command("a\u{20000}b", "");
        assert!(
            cmd.contains("Direction:wstring:4:a\u{20000}b "),
            "guide {{n}} 은 UTF-16 code unit 수(4)여야 한다(chars().count()=3 이면 RED): {cmd}"
        );
        // set 길이도 WCHAR 기준 (inner 의 UTF-16 수 − 1).
        let body = cmd.strip_prefix("Clickhere:set:").unwrap();
        let (set_str, inner) = body.split_once(':').unwrap();
        let set_len: usize = set_str.parse().unwrap();
        assert_eq!(
            set_len,
            inner.encode_utf16().count() - 1,
            "set {{N}} 은 WCHAR 수: {cmd}"
        );
    }

    #[test]
    fn test_hyperlink() {
        let link = Hyperlink {
            url: "https://example.com".to_string(),
            text: "Example".to_string(),
        };
        assert!(!link.url.is_empty());
    }
}
