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

/// 수식 ('eqed' 컨트롤, HWP 스펙 표 105)
#[derive(Debug, Clone, Default)]
pub struct Equation {
    /// 개체 공통 속성 (위치, 크기, 배치)
    pub common: CommonObjAttr,
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
    /// 덧말 텍스트
    pub ruby_text: String,
    /// 정렬 방식
    pub alignment: u8,
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
    /// CTRL_DATA에서 읽은 필드 이름 (누름틀 고치기에서 설정)
    pub ctrl_data_name: Option<String>,
    /// 메모 인덱스 (hwplib: memoIndex)
    pub memo_index: u32,
    /// 메모 본문 문단 리스트 (`fieldBegin type="MEMO"` 내부 subList)
    pub memo_paragraphs: Vec<Paragraph>,
    /// HWPX `<hp:parameters>` 요소 원문 verbatim (#1391).
    ///
    /// 전 fieldBegin 타입(MEMO/HYPERLINK/FORMULA/BOOKMARK 등)이 parameters 를
    /// 가지나 IR 은 Command/Number 만 추출하므로, 무손실 roundtrip 을 위해 원문을
    /// 그대로 보존한다 (HWP5 경로엔 무관 — HWPX 파서만 적재).
    pub raw_parameters_xml: Option<String>,
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
        let guide_len = guide.chars().count();
        let memo_len = memo.chars().count();

        // HelpState 값 뒤 공백 2개 (한컴 정답지 동형).
        let inner = format!(
            "Direction:wstring:{}:{} HelpState:wstring:{}:{}  ",
            guide_len, guide, memo_len, memo
        );
        // set 길이는 마지막 trailing 공백 1개를 제외한 inner 글자수.
        let set_len = inner.chars().count() - 1;
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

    #[test]
    fn test_hyperlink() {
        let link = Hyperlink {
            url: "https://example.com".to_string(),
            text: "Example".to_string(),
        };
        assert!(!link.url.is_empty());
    }
}
