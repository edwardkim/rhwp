//! 날짜·금액·수량을 **주소와 함께** 뽑는다 — 행정문서 구조화의 공통 프리미티브.
//!
//! `grep`(#3283)이 검색어에 대해 한 일을 데이터 값에 대해 한다. 평문을 뽑아 밖에서
//! 정규식을 돌리면 값은 얻어도 "어느 구역 몇 번째 문단, 몇 쪽"이 소멸해 근거 제시가
//! 불가능하다. rhwp 는 조판 엔진을 갖고 있어 쪽까지 답할 수 있으므로, 값마다
//! (구역·문단·쪽·문자 오프셋)을 붙여 돌려준다. 페이지 인덱스는 `grep` 과 같은
//! `build_paragraph_page_index`·`build_table_row_page_index` 를 재사용한다.
//!
//! ## 인식 규칙은 실물 표기에서 왔다
//!
//! `samples/` 의 실제 정부 문서를 `export-text --json` 으로 뽑아 세어 본 표기다.
//! `2025 행정업무운영 편람(최종).hwp` 는 `1949. 7. 15.`(점 구분) 675건과
//! `금113,560원`(접두 `금` + 공백 없음), `3,180백만원`·`21,345천원`(단위 배수),
//! `62.9%` 를 담고 있고, `2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 의 날짜는
//! `2026. 1.`(연·월)뿐이다. 그래서 연·월만 있는 표기도 인식 대상이며 ISO-8601 부분
//! 날짜(`2026-01`)로 정규화한다 — 없는 날짜를 1일로 채우지 않는다.
//!
//! ## 모르는 것은 모른다고 한다
//!
//! 두 자리 연도(`'26.8.2`)는 세기를 알 수 없으므로 `normalized: null` 이고 `raw` 만
//! 남는다. 한글·한자 수사 금액(`일금 백이십삼만원` · `金壹百貳拾參萬圓`)도 v1 범위 밖이라
//! 같은 규약이다. 틀린 추정보다 모름이 낫다 — 소비자는 `raw` 를 보고 스스로 판단할 수 있다.
//!
//! 값을 지어내느니 **인식 자체를 포기**하는 형태도 있다. 각각의 이유가 있다.
//!
//! | 형태 | 왜 뽑지 않는가 |
//! |---|---|
//! | `20260802` | 구분자가 없어 일반 정수와 구별되지 않는다 |
//! | `8/2` · `8. 2.` | 연도가 없어 어느 해인지 모르고 분수·번호와 겹친다 |
//! | `26. 8. 2.` | 어깨점 없는 두 자리 연도는 번호 매김과 구별되지 않는다 |
//! | `(1,234,567)원` | 회계의 괄호 음수는 `(3)원칙` 같은 번호 매김과 구별되지 않는다. 한국 행정문서의 음수 표기는 `△` 다 |
//! | `▲12.3%` · `▼` | 화살표의 증가/감소 의미가 문서마다 뒤집힌다 — 부호로 읽으면 절반이 틀린다 |
//! | `$1,234` · `€` · `¥` | 외화는 소수 단위·환율 맥락이 필요하다. KRW 로 표시하면 통화가 틀린 값이 된다 |
//! | `2천억원` | 곱셈형 복합 수사(2천 × 억)는 만 단위 곱셈 규칙이 필요하다 |
//! | `3억` · `5만` | `원` 이 없으면 금액인지 수량인지 문맥이 있어야 안다 |
//! | 단위 없는 맨 숫자 | 표 하나가 수백 건의 잡음이 된다 |
//! | `제3조` · `제137호` · `3차` | 개수가 아니라 번호(서수)다 |
//! | `표 3 개요` 의 `개` | 띄어 쓴 한글 단위는 낱말 첫 글자를 단위로 오인한다 |
//! | `14:30` · 전화번호 · 사업자번호 | 값의 종류가 다르다 — 별도 축이 필요하다 |
//!
//! ## ReDoS 가 원천적으로 없다
//!
//! 정규식을 쓰지 않는다. 문자 슬라이스를 왼쪽에서 오른쪽으로 **한 번** 훑으며 각
//! 위치에서 고정 길이 후보만 시도하고, 인식한 구간은 통째로 건너뛴다. 되추적이
//! 없으므로 중첩 수량자(그룹 안팎에 반복이 겹치는 형태)로 인한 파국적 backtracking
//! 자체가 존재할 수 없고, 숫자 열 한가운데에서 다시 시작하지 않으므로 `1111…1` 같은
//! 입력에서도 입력 길이에 선형이다.
//!
//! 파서/렌더 무변경의 읽기 전용 질의(추가 기능).

use serde::Serialize;

use crate::document_core::DocumentCore;
use crate::model::control::Control;

use super::grep::{CellRef, TextBoxRef};

/// 뽑아내는 값의 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataKind {
    /// 날짜 — `2026년 8월 2일(월)` · `2026. 8. 2.` · `2026-08-02` · `2026/8/2` ·
    /// `2026. 1.`(연·월) · `'26.8.2`(정규화 불가).
    Date,
    /// 금액 — `1,234,567원` · `금113,560원` · `₩1,234,567` · `1,234천원` ·
    /// `3억5천만원`(복합) · `△1,234원`(음수) · `일금 백이십삼만원`(정규화 불가).
    Amount,
    /// 수량 — `12개` · `3.5%` · `1,000명` · `△12.3%`. 단위가 붙은 수만 수량이다.
    Number,
}

impl DataKind {
    /// 전 종류 — `--kind all` 과 기본값이 쓰는 목록.
    pub const ALL: [DataKind; 3] = [DataKind::Date, DataKind::Amount, DataKind::Number];

    /// CLI `--kind` 값 → 종류. `all` 은 호출자가 처리한다.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "date" => Some(Self::Date),
            "amount" => Some(Self::Amount),
            "number" => Some(Self::Number),
            _ => None,
        }
    }

    /// JSON 봉투에 쓰는 이름 — `Serialize` 구현과 같은 문자열이다.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::Amount => "amount",
            Self::Number => "number",
        }
    }
}

/// 정규화 값. JSON 에는 날짜면 문자열, 금액·수량이면 숫자로 그대로 나간다.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Normalized {
    /// ISO-8601 날짜. 일이 없는 표기는 `YYYY-MM` 부분 날짜다.
    Date(String),
    /// 정수 값 (금액은 항상 이쪽이다).
    Int(i64),
    /// 소수 값 (`3.5%` 같은 수량에만 나온다).
    Float(f64),
}

/// 추출된 값 하나 — 값과 주소가 한 몸이다.
#[derive(Debug, Clone, Serialize)]
pub struct DataItem {
    /// 값의 종류.
    pub kind: DataKind,
    /// 문서에 적힌 그대로의 표기.
    pub raw: String,
    /// 정규화 값. **정규화할 수 없으면 `null`** 이고 `raw` 만 믿을 수 있다.
    pub normalized: Option<Normalized>,
    /// 통화 코드(ISO 4217). 금액 항목에만 붙는다. v1 은 원화(`KRW`)만 인식한다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<&'static str>,
    /// 수량의 단위(`개`·`%`·`kg`). 수량 항목에만 붙는다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// 구역 인덱스.
    pub section: usize,
    /// 본문 문단 인덱스 (표 셀·글상자 값은 그 컨트롤을 담은 본문 문단).
    pub paragraph: usize,
    /// 0부터 시작하는 글로벌 페이지 번호. 조판에 배치되지 않은 문단이면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// 문단 텍스트 내 시작 위치 (문자 단위).
    #[serde(rename = "charOffset")]
    pub char_offset: usize,
    /// 표기 길이 (문자 단위).
    pub length: usize,
    /// 표 셀 안의 값이면 셀 좌표. 본문 값이면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<CellRef>,
    /// 글상자 안의 값이면 글상자 좌표. 본문·표 셀 값이면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub textbox: Option<TextBoxRef>,
}

/// 값 하나에 붙일 주소 — 인자 개수를 늘리지 않고 순회 경로마다 갈아 끼운다.
#[derive(Debug, Clone)]
struct Address {
    section: usize,
    paragraph: usize,
    page: Option<u32>,
    cell: Option<CellRef>,
    textbox: Option<TextBoxRef>,
}

/// 주소가 붙기 전의 인식 결과.
#[derive(Debug, Clone)]
struct Extracted {
    kind: DataKind,
    raw: String,
    normalized: Option<Normalized>,
    currency: Option<&'static str>,
    unit: Option<String>,
    char_offset: usize,
    length: usize,
}

// ── 원화 단위 배수 ──────────────────────────────────────────────────────────
//
// `(표기, 10의 지수)`. `3,180백만원` = 3_180_000_000 처럼 단위가 곧 배수다.
// 가장 **긴** 표기를 고르므로 목록 순서에 의존하지 않는다.
const KRW_UNITS: &[(&str, u32)] = &[
    ("조원", 12),
    ("억원", 8),
    ("천만원", 7),
    ("백만원", 6),
    ("십만원", 5),
    ("만원", 4),
    ("천원", 3),
    ("백원", 2),
    ("원", 0),
];

/// `원` 으로 닫히지 않은 자릿수 단위 — 복합 금액(`3억5천만원`)의 중간 마디.
const KRW_SCALES: &[(&str, u32)] = &[
    ("조", 12),
    ("억", 8),
    ("만", 4),
    ("천", 3),
    ("백", 2),
    ("십", 1),
];

/// 수사(數詞) 글자 — 한글·한자·갖은자를 함께 본다.
///
/// `일금 백이십삼만원`(한글) · `金壹百貳拾參萬圓`(갖은자) 는 계약서·영수증의 실물
/// 표기다. **인식은 하되 값은 정규화하지 않는다**(v1) — 자리 올림 규칙(만 단위 곱셈)을
/// 반만 구현하면 조용히 틀린 금액이 나온다. raw 를 남겨 사람이 판단하게 한다.
const NUMERAL_CHARS: &str = "영일이삼사오육칠팔구십백천만억조\
                             零一二三四五六七八九十百千万萬億兆\
                             壹貳貮參参肆伍陸柒捌玖拾佰仟阡";

/// 수사 금액을 닫는 글자 — `원`·`圓`(정자)·`円`·`元`.
const NUMERAL_CURRENCY_END: &str = "원圓円元";

/// 통화 기호 — `₩`(U+20A9)와 전각 `￦`(U+FFE6) 둘 다 실물에 나온다.
const KRW_SIGNS: [char; 2] = ['\u{20a9}', '\u{ffe6}'];

/// 음수 표기 글자.
///
/// `△`(U+25B3)·`▽`(U+25BD)는 한국 행정·통계 문서의 마이너스 기호다(증감란의 `△12.3%`).
/// `▲`·`▼` 는 **문서에 따라 증가/감소가 뒤집히므로 부호로 읽지 않는다** — 뒤집힌 부호는
/// 조용히 틀린 집계가 된다.
const MINUS_MARKS: [char; 5] = ['-', '\u{2212}', '\u{ff0d}', '\u{25b3}', '\u{25bd}'];

/// 두 자리 연도 앞의 어깨점 — 곧은 따옴표와 둥근 따옴표(실물 편람은 `’26. 1.`)를 함께 본다.
const YEAR_APOSTROPHES: [char; 3] = ['\'', '\u{2018}', '\u{2019}'];

/// 숫자에 **붙여 써야** 인정하는 한글 단위.
///
/// 공백을 허용하면 `표 3 개요` 의 `개` 를 수량으로 삼킨다. 실물 표기는 붙여 쓰므로
/// (`2,106개`·`300대`) 붙여 쓴 것만 받는다. 서수 어휘(`호`·`차`·`조`·`항`)는
/// 수량이 아니라 번호라서 넣지 않는다 — `제137호` 를 137개로 세면 안 된다.
const HANGUL_UNITS: &[&str] = &[
    "개월",
    "개소",
    "개",
    "명",
    "건",
    "매",
    "부",
    "회",
    "권",
    "점",
    "대",
    "인",
    "곳",
    "종",
    "세",
    "쪽",
    "장",
    "상자",
    "시간",
    "분",
    "초",
    "주",
    "가구",
    "세대",
    "마리",
    "그루",
    "평",
    "배",
    "팀",
    "톤",
    "퍼센트",
    "층",
    "칸",
    "석",
    "가지",
    "벌",
    "쌍",
    "분기",
    "교시",
    "학점",
];

/// 공백 하나를 허용하는 기호·라틴 단위 (`3.5 %`).
const SYMBOL_UNITS: &[&str] = &[
    "%", "％", "‰", "㎏", "kg", "㎡", "㎢", "㎥", "㎞", "km", "㎝", "cm", "㎜", "mm", "㎖", "mL",
    "GB", "MB", "KB", "TB", "g", "t", "m", "L", "ℓ", "㏊", "ha", "㎾", "kW", "㎿", "MW", "dB", "℃",
    "°C", "㎉", "kcal",
];

/// 백분율 표기는 하나로 모은다 — 나머지 단위는 문서 표기 그대로 둔다.
fn canonical_unit(unit: &str) -> String {
    match unit {
        "％" | "퍼센트" => "%".to_string(),
        other => other.to_string(),
    }
}

// ── 문자 단위 스캐너 원시 함수 ─────────────────────────────────────────────

/// `chars[at..]` 가 `needle` 로 시작하는가.
fn starts_with(chars: &[char], at: usize, needle: &str) -> bool {
    let mut i = at;
    for expected in needle.chars() {
        if chars.get(i) != Some(&expected) {
            return false;
        }
        i += 1;
    }
    true
}

/// 목록에서 `at` 위치에 걸리는 **가장 긴** 단위를 고른다. 목록 순서에 의존하지 않으므로
/// 항목을 덧붙일 때 `백만원` 을 `만원` 앞에 두는 식의 순서 규칙을 지킬 필요가 없다.
fn longest_unit<'a>(units: &[&'a str], chars: &[char], at: usize) -> Option<&'a str> {
    units
        .iter()
        .filter(|u| starts_with(chars, at, u))
        .copied()
        .max_by_key(|u| u.chars().count())
}

/// 원화 단위와 그 10의 지수 — 같은 "가장 긴 것" 규칙을 쓴다.
fn longest_krw_unit(chars: &[char], at: usize) -> Option<(&'static str, u32)> {
    KRW_UNITS
        .iter()
        .filter(|(unit, _)| starts_with(chars, at, unit))
        .max_by_key(|(unit, _)| unit.chars().count())
        .copied()
}

/// 라틴 문자 단위는 뒤에 영숫자가 붙으면 단위가 아니다 (`3gb` 를 3g 로 읽지 않는다).
fn unit_boundary_ok(unit: &str, chars: &[char], after: usize) -> bool {
    if !unit.chars().all(|c| c.is_ascii_alphabetic()) {
        return true;
    }
    !chars.get(after).is_some_and(|c| c.is_ascii_alphanumeric())
}

/// 공백류를 건너뛴다.
fn skip_spaces(chars: &[char], mut i: usize) -> usize {
    while matches!(chars.get(i), Some(' ') | Some('\t') | Some('\u{a0}')) {
        i += 1;
    }
    i
}

/// 낱말 첫 글자인가 — `지금`·`요금` 의 `금` 을 금액 접두로 오인하지 않기 위해서다.
fn at_word_start(chars: &[char], at: usize) -> bool {
    at == 0 || !chars[at - 1].is_alphanumeric()
}

/// 음수 접두를 읽는다 → (숫자 시작 위치, 음수 여부).
///
/// 두 가지를 반드시 걸러야 부호가 거짓말하지 않는다.
/// - 앞 글자가 숫자면 **범위 표기**다 (`3-5개` 를 -5개로 읽으면 안 된다).
/// - 숫자에 **붙여 쓴 것만** 부호다. 한국 문서의 글머리표는 `- 항목`·`△ 항목` 처럼
///   뒤에 공백이 오므로, 붙여쓰기 규칙 하나로 글머리표와 마이너스가 갈린다.
fn read_sign(chars: &[char], at: usize) -> (usize, bool) {
    if !chars.get(at).is_some_and(|c| MINUS_MARKS.contains(c)) {
        return (at, false);
    }
    if at > 0 && (chars[at - 1].is_ascii_digit() || chars[at - 1] == ',') {
        return (at, false);
    }
    match chars.get(at + 1) {
        Some(c) if c.is_ascii_digit() => (at + 1, true),
        _ => (at, false),
    }
}

/// `1,234,567` · `12345` 형태의 정수부 → (끝 위치, 숫자만 이어붙인 문자열).
///
/// 세 자리 묶음은 `,` + **정확히 3자리** 의 고정 길이로만 이어 붙인다. 선택지가 없으므로
/// 되추적이 없고, 소비한 만큼만 전진한다.
fn read_int(chars: &[char], start: usize) -> Option<(usize, String)> {
    let mut i = start;
    let mut digits = String::new();
    while chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
        digits.push(chars[i]);
        i += 1;
    }
    if digits.is_empty() {
        return None;
    }
    // 첫 묶음이 1~3자리일 때만 천 단위 구분이 성립한다 (`1234,567` 은 아니다).
    if digits.len() <= 3 {
        while i + 3 < chars.len()
            && chars[i] == ','
            && chars[i + 1].is_ascii_digit()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit()
            && !chars.get(i + 4).is_some_and(|c| c.is_ascii_digit())
        {
            digits.push(chars[i + 1]);
            digits.push(chars[i + 2]);
            digits.push(chars[i + 3]);
            i += 4;
        }
    }
    Some((i, digits))
}

/// 정수부 뒤의 소수부 `.\d+` → (끝 위치, 소수 자릿수 문자열).
fn read_frac(chars: &[char], start: usize) -> Option<(usize, String)> {
    if chars.get(start) != Some(&'.') {
        return None;
    }
    let mut i = start + 1;
    let mut digits = String::new();
    while chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
        digits.push(chars[i]);
        i += 1;
    }
    if digits.is_empty() {
        None
    } else {
        Some((i, digits))
    }
}

/// 자릿수를 못 박고 읽는다 — 날짜의 연·월·일 전용.
///
/// 최대 자릿수를 넘겨 숫자가 더 이어지면 **날짜가 아니다**(`20261년` 을 2026년으로
/// 읽지 않는다).
fn read_fixed_digits(chars: &[char], start: usize, min: usize, max: usize) -> Option<(usize, u32)> {
    let mut i = start;
    let mut value: u32 = 0;
    let mut count = 0;
    while count < max && chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
        value = value * 10 + chars[i].to_digit(10).unwrap_or(0);
        i += 1;
        count += 1;
    }
    if count < min || chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((i, value))
}

// ── 날짜 ───────────────────────────────────────────────────────────────────

/// 날짜 후보. `iso` 가 `None` 이면 정규화 불가(두 자리 연도)다.
struct DateHit {
    end: usize,
    iso: Option<String>,
}

/// 구분자별 본문 — (끝 위치, 월, 일). 일이 없으면 연·월 표기다.
type MonthDay = (usize, u32, Option<u32>);

fn valid_day(day: u32) -> bool {
    (1..=31).contains(&day)
}

/// `YYYY년 M월 [D일]` 본문.
fn date_body_korean(chars: &[char], at: usize) -> Option<MonthDay> {
    let mut i = skip_spaces(chars, at + 1);
    let (after, month) = read_fixed_digits(chars, i, 1, 2)?;
    i = skip_spaces(chars, after);
    if chars.get(i) != Some(&'월') {
        return None;
    }
    i += 1;
    let year_month_end = i;
    let day_start = skip_spaces(chars, i);
    if let Some((after_day, day)) = read_fixed_digits(chars, day_start, 1, 2) {
        let mark = skip_spaces(chars, after_day);
        if chars.get(mark) == Some(&'일') && valid_day(day) {
            return Some((mark + 1, month, Some(day)));
        }
    }
    Some((year_month_end, month, None))
}

/// `YYYY. M. [D.]` 본문 — 연·월만 있는 표기도 마침표로 닫혀야 한다.
fn date_body_dotted(chars: &[char], at: usize) -> Option<MonthDay> {
    let mut i = skip_spaces(chars, at + 1);
    let (after, month) = read_fixed_digits(chars, i, 1, 2)?;
    i = skip_spaces(chars, after);
    if chars.get(i) != Some(&'.') {
        return None;
    }
    i += 1;
    let year_month_end = i;
    let day_start = skip_spaces(chars, i);
    if let Some((after_day, day)) = read_fixed_digits(chars, day_start, 1, 2) {
        if valid_day(day) {
            // 일 뒤 마침표는 선택이다 — `2026. 8. 2` 도 실물 표기다.
            let mark = skip_spaces(chars, after_day);
            let end = if chars.get(mark) == Some(&'.') {
                mark + 1
            } else {
                after_day
            };
            return Some((end, month, Some(day)));
        }
    }
    Some((year_month_end, month, None))
}

/// `YYYY-MM-DD` · `YYYY/M/D` 본문 — 세 성분이 모두 있어야 성립한다.
fn date_body_separator(chars: &[char], at: usize, sep: char, width: usize) -> Option<MonthDay> {
    let (after_month, month) = read_fixed_digits(chars, at + 1, width, 2)?;
    if chars.get(after_month) != Some(&sep) {
        return None;
    }
    let (after_day, day) = read_fixed_digits(chars, after_month + 1, width, 2)?;
    if !valid_day(day) {
        return None;
    }
    Some((after_day, month, Some(day)))
}

/// `start` 위치에서 시작하는 날짜 표기를 인식한다.
fn try_date(chars: &[char], start: usize) -> Option<DateHit> {
    // 두 자리 연도는 어깨점이 있을 때만 받는다. 맨 `26. 8. 2.` 까지 날짜로 보면
    // 번호 매김·버전 표기가 통째로 날짜가 된다.
    let (after_year, year, four_digit) = if chars
        .get(start)
        .is_some_and(|c| YEAR_APOSTROPHES.contains(c))
    {
        let (end, value) = read_fixed_digits(chars, start + 1, 2, 2)?;
        (end, value, false)
    } else {
        let (end, value) = read_fixed_digits(chars, start, 4, 4)?;
        (end, value, true)
    };
    let sep_at = skip_spaces(chars, after_year);
    let (mut end, month, day) = match chars.get(sep_at) {
        Some('년') => date_body_korean(chars, sep_at)?,
        Some('.') => date_body_dotted(chars, sep_at)?,
        Some('-') => date_body_separator(chars, sep_at, '-', 2)?,
        Some('/') => date_body_separator(chars, sep_at, '/', 1)?,
        _ => return None,
    };
    if !(1..=12).contains(&month) {
        return None;
    }

    // 요일 괄호 — `2026년 8월 2일(월)` · `2026. 8. 2.(월요일)`.
    let paren = skip_spaces(chars, end);
    if chars.get(paren) == Some(&'(')
        && chars
            .get(paren + 1)
            .is_some_and(|c| "월화수목금토일".contains(*c))
    {
        let mut close = paren + 2;
        if starts_with(chars, close, "요일") {
            close += 2;
        }
        if chars.get(close) == Some(&')') {
            end = close + 1;
        }
    }

    // 두 자리 연도는 세기를 추정하지 않는다 — 모름을 그대로 낸다.
    let iso = four_digit.then(|| match day {
        Some(d) => format!("{year:04}-{month:02}-{d:02}"),
        None => format!("{year:04}-{month:02}"),
    });
    Some(DateHit { end, iso })
}

// ── 금액 ───────────────────────────────────────────────────────────────────

/// 금액 후보. `value` 가 `None` 이면 정규화 불가(한글 수사·나누어떨어지지 않는 배수)다.
struct AmountHit {
    end: usize,
    value: Option<i64>,
}

/// 정수·소수 자릿수와 10의 지수로 정확한 정수 값을 만든다.
///
/// 부동소수 곱셈을 쓰지 않는다 — `1.5억원` 은 150_000_000 으로 딱 떨어져야 하고,
/// `1,234.56원` 처럼 정수로 떨어지지 않으면 **추정하지 않고** `None` 이다.
fn scaled_value(int_digits: &str, frac_digits: &str, pow10: u32) -> Option<i64> {
    let mantissa: i128 = format!("{int_digits}{frac_digits}").parse().ok()?;
    let exponent = pow10 as i32 - frac_digits.len() as i32;
    let value = if exponent >= 0 {
        mantissa.checked_mul(10i128.checked_pow(exponent.unsigned_abs())?)?
    } else {
        let divisor = 10i128.checked_pow(exponent.unsigned_abs())?;
        if mantissa % divisor != 0 {
            return None;
        }
        mantissa / divisor
    };
    i64::try_from(value).ok()
}

/// `3억5천만원` 처럼 자릿수 단위가 이어지는 금액을 읽는다 → (끝 위치, 합계).
///
/// 각 마디는 `숫자 + 자릿수 단위` 이고 마지막 마디가 `…원` 으로 닫힌다. 자릿수 단위는
/// **엄격히 작아져야** 한다 — `3만 5억원` 처럼 뒤집힌 표기는 수사가 아니므로 받지 않는다.
/// 반복 횟수는 자릿수 단위 개수로 상한이 있어 되추적도 폭주도 없다.
///
/// 곱셈형 복합 수사(`2천억원` = 2천 × 억)는 v1 미지원이다. 만(萬) 단위 곱셈 규칙을
/// 반만 구현하면 조용히 틀린 값이 나오므로, **인식 자체를 포기**해 아무 값도 내지 않는다.
fn read_krw_chain(chars: &[char], start: usize) -> Option<(usize, Option<i64>)> {
    let mut i = start;
    let mut total: Option<i128> = Some(0);
    let mut previous_scale: Option<u32> = None;

    for _ in 0..=KRW_SCALES.len() {
        let (after_int, int_digits) = read_int(chars, i)?;
        let (after_number, frac_digits) =
            read_frac(chars, after_int).unwrap_or_else(|| (after_int, String::new()));

        let add = |total: Option<i128>, pow10: u32| -> Option<i128> {
            match (total, scaled_value(&int_digits, &frac_digits, pow10)) {
                (Some(sum), Some(term)) => sum.checked_add(i128::from(term)),
                _ => None,
            }
        };

        // 종결 단위(`…원`)를 만나면 거기서 끝난다.
        if let Some((unit, pow10)) = longest_krw_unit(chars, after_number) {
            if previous_scale.is_some_and(|prev| prev <= pow10) {
                return None;
            }
            let total = add(total, pow10);
            let end = after_number + unit.chars().count();
            return Some((end, total.and_then(|v| i64::try_from(v).ok())));
        }

        // 중간 마디 — `원` 없는 자릿수 단위. 없으면 금액 표기가 아니다.
        let (scale, pow10) = KRW_SCALES
            .iter()
            .filter(|(scale, _)| starts_with(chars, after_number, scale))
            .max_by_key(|(scale, _)| scale.chars().count())
            .copied()?;
        if previous_scale.is_some_and(|prev| prev <= pow10) {
            return None;
        }
        total = add(total, pow10);
        previous_scale = Some(pow10);
        i = skip_spaces(chars, after_number + scale.chars().count());
    }
    None
}

/// `start` 위치에서 시작하는 금액 표기를 인식한다.
fn try_amount(chars: &[char], start: usize) -> Option<AmountHit> {
    let (mut i, mut negative) = read_sign(chars, start);
    // 접두 `금`·`일금`·`金`·`一金`. 낱말 중간이면(`지금`) 접두가 아니다.
    let prefixed = if !at_word_start(chars, i) {
        false
    } else if let Some(prefix) = ["일금", "一金", "壹金", "금", "金"]
        .into_iter()
        .find(|prefix| starts_with(chars, i, prefix))
    {
        i += prefix.chars().count();
        true
    } else {
        false
    };
    if prefixed {
        i = skip_spaces(chars, i);
    }
    let signed = chars.get(i).is_some_and(|c| KRW_SIGNS.contains(c));
    if signed {
        i = skip_spaces(chars, i + 1);
        if !negative {
            let (after_sign, is_negative) = read_sign(chars, i);
            i = after_sign;
            negative = is_negative;
        }
    }
    let apply = |value: Option<i64>| value.map(|v| if negative { -v } else { v });

    if let Some((mut end, value)) = read_krw_chain(chars, i) {
        // 갖은자 영수증의 `…원정`.
        if chars.get(end) == Some(&'정') {
            end += 1;
        }
        return Some(AmountHit {
            end,
            value: apply(value),
        });
    }

    // 통화 기호만 있고 단위가 없는 표기 — `₩1,234,567`.
    if signed {
        if let Some((after_int, int_digits)) = read_int(chars, i) {
            let (end, frac_digits) =
                read_frac(chars, after_int).unwrap_or_else(|| (after_int, String::new()));
            return Some(AmountHit {
                end,
                value: apply(scaled_value(&int_digits, &frac_digits, 0)),
            });
        }
    }

    // 한글·한자 수사 — `일금 백이십삼만원` · `金壹百貳拾參萬圓`.
    // 접두나 통화 기호가 있을 때만 본다(맨 `만원` 같은 낱말을 금액으로 삼키지 않는다).
    if prefixed || signed {
        let mut end = i;
        while chars.get(end).is_some_and(|c| NUMERAL_CHARS.contains(*c)) {
            end += 1;
        }
        if end > i
            && chars
                .get(end)
                .is_some_and(|c| NUMERAL_CURRENCY_END.contains(*c))
        {
            end += 1;
            if chars.get(end) == Some(&'정') {
                end += 1;
            }
            // 값은 정규화하지 않는다 — 모름을 그대로 낸다.
            return Some(AmountHit { end, value: None });
        }
    }
    None
}

// ── 수량 ───────────────────────────────────────────────────────────────────

/// 수량 후보.
struct NumberHit {
    end: usize,
    value: Option<Normalized>,
    unit: String,
}

/// `start` 위치에서 시작하는 수량 표기를 인식한다. **단위가 없으면 수량이 아니다** —
/// 맨 숫자까지 뽑으면 표 하나가 수백 건의 잡음이 된다.
fn try_number(chars: &[char], start: usize) -> Option<NumberHit> {
    // `제3조`·`제1항`·`제2장` 의 숫자는 수량이 아니라 서수다.
    if start > 0 && chars[start - 1] == '제' {
        return None;
    }
    // `△12.3%`(감소)처럼 통계표의 음수 표기가 붙는다.
    let (num_start, negative) = read_sign(chars, start);
    let (after_int, int_digits) = read_int(chars, num_start)?;
    let (after_number, frac_digits) =
        read_frac(chars, after_int).unwrap_or_else(|| (after_int, String::new()));

    let spaced = skip_spaces(chars, after_number);
    let (unit, end) = match longest_unit(HANGUL_UNITS, chars, after_number) {
        Some(unit) => (unit, after_number + unit.chars().count()),
        None => {
            let unit = longest_unit(SYMBOL_UNITS, chars, spaced)?;
            let end = spaced + unit.chars().count();
            if !unit_boundary_ok(unit, chars, end) {
                return None;
            }
            (unit, end)
        }
    };

    let sign = if negative { "-" } else { "" };
    let value = if frac_digits.is_empty() {
        format!("{sign}{int_digits}")
            .parse::<i64>()
            .ok()
            .map(Normalized::Int)
    } else {
        format!("{sign}{int_digits}.{frac_digits}")
            .parse::<f64>()
            .ok()
            .map(Normalized::Float)
    };
    Some(NumberHit {
        end,
        value,
        unit: canonical_unit(unit),
    })
}

// ── 텍스트 한 덩이 스캔 ────────────────────────────────────────────────────

/// 문단 텍스트 하나에서 값들을 뽑는다.
///
/// 왼쪽에서 오른쪽으로 한 번 훑으며 **날짜 → 금액 → 수량** 순으로 시도하고, 인식한
/// 구간은 통째로 건너뛴다. 그래서 `2026년 8월 2일` 의 `8` 이 수량으로 다시 잡히지 않고
/// 항목끼리 겹치지 않는다. 종류 필터는 이 판정 뒤에 적용해야 필터에 따라 경계가
/// 달라지지 않는다.
fn scan_text(text: &str) -> Vec<Extracted> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Extracted> = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        // 숫자 열 한가운데에서 다시 시도하지 않는다 — 어차피 성립할 수 없고,
        // 이 한 줄이 최악 입력에서도 선형 시간을 보장한다.
        if chars[i].is_ascii_digit()
            && i > 0
            && (chars[i - 1].is_ascii_digit() || chars[i - 1] == ',' || chars[i - 1] == '.')
        {
            i += 1;
            continue;
        }

        if let Some(hit) = try_date(&chars, i) {
            out.push(Extracted {
                kind: DataKind::Date,
                raw: chars[i..hit.end].iter().collect(),
                normalized: hit.iso.map(Normalized::Date),
                currency: None,
                unit: None,
                char_offset: i,
                length: hit.end - i,
            });
            i = hit.end;
            continue;
        }
        if let Some(hit) = try_amount(&chars, i) {
            out.push(Extracted {
                kind: DataKind::Amount,
                raw: chars[i..hit.end].iter().collect(),
                normalized: hit.value.map(Normalized::Int),
                currency: Some("KRW"),
                unit: None,
                char_offset: i,
                length: hit.end - i,
            });
            i = hit.end;
            continue;
        }
        if let Some(hit) = try_number(&chars, i) {
            out.push(Extracted {
                kind: DataKind::Number,
                raw: chars[i..hit.end].iter().collect(),
                normalized: hit.value,
                currency: None,
                unit: Some(hit.unit),
                char_offset: i,
                length: hit.end - i,
            });
            i = hit.end;
            continue;
        }
        i += 1;
    }
    out
}

/// 인식 결과에 주소를 붙여 담는다.
fn collect_into(out: &mut Vec<DataItem>, text: &str, at: &Address) {
    if text.is_empty() {
        return;
    }
    for found in scan_text(text) {
        out.push(DataItem {
            kind: found.kind,
            raw: found.raw,
            normalized: found.normalized,
            currency: found.currency,
            unit: found.unit,
            section: at.section,
            paragraph: at.paragraph,
            page: at.page,
            char_offset: found.char_offset,
            length: found.length,
            cell: at.cell.clone(),
            textbox: at.textbox.clone(),
        });
    }
}

impl DocumentCore {
    /// 문서 전체에서 날짜·금액·수량을 주소와 함께 뽑는다.
    ///
    /// 본문·표 셀·글상자를 순회한다(`grep` 과 같은 범위에서 수식만 뺀다 — 수식 스크립트의
    /// 숫자는 값이 아니라 식이다). `kinds` 가 비어 있으면 전 종류를 돌려준다.
    ///
    /// 인식 자체는 항상 전 종류로 하고 필터는 마지막에 적용한다 — `--kind amount` 가
    /// 날짜 안의 숫자를 금액으로 다시 읽는 일이 없어야 하기 때문이다.
    pub fn extract_data(&self, kinds: &[DataKind]) -> Vec<DataItem> {
        let page_index = self.build_paragraph_page_index();
        let table_row_pages = self.build_table_row_page_index();
        let mut out: Vec<DataItem> = Vec::new();

        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                let page = page_index.get(&(sec_idx, para_idx)).copied();
                let body = Address {
                    section: sec_idx,
                    paragraph: para_idx,
                    page,
                    cell: None,
                    textbox: None,
                };
                collect_into(&mut out, &para.text, &body);

                for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Table(table) => {
                            for (cell_idx, cell) in table.cells.iter().enumerate() {
                                // [#3403] 분할 표는 그 행이 실제로 렌더되는 쪽을 쓴다.
                                let cell_page = table_row_pages
                                    .get(&(sec_idx, para_idx, ctrl_idx))
                                    .and_then(|ranges| {
                                        ranges
                                            .iter()
                                            .find(|(start, end, _)| {
                                                (cell.row as usize) >= *start
                                                    && (cell.row as usize) < *end
                                            })
                                            .map(|(_, _, page)| *page)
                                    })
                                    .or(page);
                                for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                                    let at = Address {
                                        section: sec_idx,
                                        paragraph: para_idx,
                                        page: cell_page,
                                        cell: Some(CellRef {
                                            control: ctrl_idx,
                                            cell: cell_idx,
                                            paragraph: cp_idx,
                                        }),
                                        textbox: None,
                                    };
                                    collect_into(&mut out, &cp.text, &at);
                                }
                            }
                        }
                        Control::Shape(shape) => {
                            if let Some(tb) =
                                crate::document_core::helpers::get_textbox_from_shape(shape)
                            {
                                for (tp_idx, tp) in tb.paragraphs.iter().enumerate() {
                                    let at = Address {
                                        section: sec_idx,
                                        paragraph: para_idx,
                                        page,
                                        cell: None,
                                        textbox: Some(TextBoxRef {
                                            control: ctrl_idx,
                                            paragraph: tp_idx,
                                        }),
                                    };
                                    collect_into(&mut out, &tp.text, &at);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if kinds.is_empty() {
            out
        } else {
            out.retain(|item| kinds.contains(&item.kind));
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str) -> Extracted {
        let found = scan_text(text);
        assert_eq!(found.len(), 1, "{text:?} → {found:?}");
        found.into_iter().next().expect("항목 1건")
    }

    fn iso(text: &str) -> Option<String> {
        match one(text).normalized {
            Some(Normalized::Date(s)) => Some(s),
            None => None,
            other => panic!("{text:?} 는 날짜가 아님: {other:?}"),
        }
    }

    fn amount(text: &str) -> Option<i64> {
        let item = one(text);
        assert_eq!(item.kind, DataKind::Amount, "{text:?}");
        assert_eq!(item.currency, Some("KRW"), "{text:?}");
        match item.normalized {
            Some(Normalized::Int(v)) => Some(v),
            None => None,
            other => panic!("{text:?} 금액 정규화 형태 오류: {other:?}"),
        }
    }

    #[test]
    fn dates_in_real_notations() {
        assert_eq!(iso("2026년 8월 2일").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026년 8월 2일(월)").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026. 8. 2.").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026.8.2").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026-08-02").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026/8/2").as_deref(), Some("2026-08-02"));
        // 실물 편람의 지배적 표기.
        assert_eq!(iso("1949. 7. 15.").as_deref(), Some("1949-07-15"));
        // 연·월만 있는 표기(기부 보고서 양식의 유일한 날짜)는 부분 날짜로 둔다.
        assert_eq!(iso("2026. 1.").as_deref(), Some("2026-01"));
        assert_eq!(iso("2025년 12월").as_deref(), Some("2025-12"));
    }

    #[test]
    fn two_digit_year_is_not_guessed() {
        // 세기를 추정하면 100년이 틀린다. 모름을 그대로 낸다.
        let item = one("'26.8.2");
        assert_eq!(item.kind, DataKind::Date);
        assert_eq!(item.raw, "'26.8.2");
        assert!(item.normalized.is_none(), "{item:?}");
    }

    #[test]
    fn invalid_dates_are_rejected() {
        assert!(
            scan_text("2026년 13월 2일").is_empty(),
            "13월은 날짜가 아님"
        );
        assert!(scan_text("2026-02-32").is_empty(), "32일은 날짜가 아님");
        // 번호 매김을 날짜로 읽으면 안 된다.
        assert!(scan_text("1. 개요").is_empty());
        assert!(scan_text("제3조 제1항").is_empty());
    }

    #[test]
    fn amounts_in_real_notations() {
        assert_eq!(amount("1,234,567원"), Some(1_234_567));
        // 실물: 접두 `금` 에 공백이 없다.
        assert_eq!(amount("금113,560원"), Some(113_560));
        assert_eq!(amount("금 1,234,567원"), Some(1_234_567));
        assert_eq!(amount("\u{20a9}1,234,567"), Some(1_234_567));
        // 단위 배수 — 실물 편람의 `3,180백만원`·`21,345천원`.
        assert_eq!(amount("3,180백만원"), Some(3_180_000_000));
        assert_eq!(amount("21,345천원"), Some(21_345_000));
        assert_eq!(amount("57억원"), Some(5_700_000_000));
        assert_eq!(amount("1.5억원"), Some(150_000_000));
        assert_eq!(amount("금 1,234,567원정"), Some(1_234_567));
    }

    #[test]
    fn more_date_notations() {
        // 마침표 사이 공백이 없거나 두 자리로 채운 표기.
        assert_eq!(iso("2026.08.02").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026년8월2일").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026 . 8 . 2 .").as_deref(), Some("2026-08-02"));
        // 요일은 한 글자·세 글자 둘 다 실물이다.
        assert_eq!(iso("2026. 8. 2.(월요일)").as_deref(), Some("2026-08-02"));
        assert_eq!(iso("2026년 8월 2일 (일)").as_deref(), Some("2026-08-02"));
        // 어깨점은 곧은 따옴표·둥근 따옴표 둘 다 (실물 편람은 `’26. 1.`).
        for text in ["'26. 8. 2.", "\u{2018}26. 8. 2.", "\u{2019}26. 8. 2."] {
            let item = one(text);
            assert_eq!(item.kind, DataKind::Date, "{text}");
            assert!(item.normalized.is_none(), "{text} → {item:?}");
        }
    }

    #[test]
    fn ambiguous_date_shapes_are_not_extracted() {
        // 구분자 없는 8자리는 일반 정수와 구별할 수 없다.
        assert!(scan_text("20260802").is_empty());
        // 연도 없는 날짜는 어느 해인지 알 수 없고 분수·번호와 구별되지 않는다.
        assert!(scan_text("8/2").is_empty());
        assert!(scan_text("8. 2.").is_empty());
        // 어깨점 없는 두 자리 연도는 번호 매김과 구별할 수 없다.
        assert!(scan_text("26. 8. 2.").is_empty());
    }

    #[test]
    fn korean_and_hanja_numeral_amounts_are_raw_only() {
        // 자리 올림 규칙을 반만 구현하면 조용히 틀린 금액이 된다 — 값은 내지 않는다.
        for text in [
            "일금 백이십삼만원",
            "금일십일만삼천오백육십원",
            "金壹百貳拾參萬圓",
            "一金 百二十三萬圓",
            "일금 백이십삼만원정",
        ] {
            let item = one(text);
            assert_eq!(item.kind, DataKind::Amount, "{text}");
            assert_eq!(item.raw, text, "{text}");
            assert_eq!(item.currency, Some("KRW"), "{text}");
            assert!(item.normalized.is_none(), "{text} → {item:?}");
        }
        // 접두(`금`·`金`)나 통화 기호가 없으면 수사만으로는 금액으로 보지 않는다 —
        // `만원`·`백만` 같은 낱말이 문장 안에 흔하다.
        assert!(scan_text("백이십삼만원").is_empty());
    }

    #[test]
    fn compound_amounts_sum_by_descending_scale() {
        assert_eq!(amount("3억5천만원"), Some(350_000_000));
        assert_eq!(amount("1억 2천만원"), Some(120_000_000));
        assert_eq!(amount("3만5천원"), Some(35_000));
        assert_eq!(amount("1,234만원"), Some(12_340_000));
        assert_eq!(amount("5백원"), Some(500));
    }

    #[test]
    fn malformed_compound_amounts_are_not_guessed() {
        // 자릿수가 뒤집힌 표기는 수사가 아니다 — 합계를 지어내지 않는다.
        let found = scan_text("3만 5억원");
        assert!(
            !found
                .iter()
                .any(|f| f.normalized == Some(Normalized::Int(350_000_000))),
            "뒤집힌 자릿수를 합산했습니다: {found:?}"
        );
        // 곱셈형 복합 수사(`2천억원` = 2천 × 억)는 v1 미지원 — 인식 자체를 포기한다.
        assert!(
            scan_text("2천억원").is_empty(),
            "{:?}",
            scan_text("2천억원")
        );
    }

    #[test]
    fn negative_markers_flip_the_sign() {
        // `△` 는 한국 행정·통계 문서의 마이너스다.
        assert_eq!(amount("-1,234,567원"), Some(-1_234_567));
        assert_eq!(amount("\u{25b3}1,234,567원"), Some(-1_234_567));
        assert_eq!(amount("\u{2212}1,234,567원"), Some(-1_234_567));
        let item = one("\u{25b3}12.3%");
        assert_eq!(item.kind, DataKind::Number);
        assert_eq!(item.normalized, Some(Normalized::Float(-12.3)));
        assert_eq!(item.raw, "\u{25b3}12.3%");
    }

    #[test]
    fn direction_arrows_are_not_signs() {
        // `▲`·`▼` 는 문서에 따라 증가/감소가 뒤집힌다 — 부호로 읽으면 절반이 틀린다.
        let item = one("\u{25b2}12.3%");
        assert_eq!(item.normalized, Some(Normalized::Float(12.3)));
        assert_eq!(item.raw, "12.3%", "화살표는 표기에 포함하지 않는다");
    }

    #[test]
    fn bullets_and_ranges_are_not_signs() {
        // 글머리표는 뒤에 공백이 온다 — 붙여쓰기 규칙 하나로 마이너스와 갈린다.
        let item = one("- 1,234,567원");
        assert_eq!(item.normalized, Some(Normalized::Int(1_234_567)));
        // 범위 표기의 하이픈은 부호가 아니다.
        let found = scan_text("3-5개");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].normalized, Some(Normalized::Int(5)), "{found:?}");
    }

    #[test]
    fn parentheses_are_not_read_as_a_minus() {
        // 회계의 괄호 음수는 한국 행정문서의 번호 매김(`(3)원칙`)과 구별할 수 없다.
        // 부호를 지어내는 대신 그 형태를 아예 뽑지 않는다.
        assert!(scan_text("(1,234,567)원").is_empty());
        assert!(scan_text("(3)원칙").is_empty());
        // 단위가 괄호 안에 있으면 평범한 부연 표기다 — 양수 그대로.
        let item = one("(1,234,567원)");
        assert_eq!(item.normalized, Some(Normalized::Int(1_234_567)));
        assert_eq!(item.raw, "1,234,567원");
    }

    #[test]
    fn currency_symbols_and_foreign_currency() {
        assert_eq!(amount("\u{20a9}1,234,567"), Some(1_234_567));
        assert_eq!(amount("\u{ffe6}1,234,567"), Some(1_234_567));
        assert_eq!(amount("\u{20a9} 1,234,567"), Some(1_234_567));
        assert_eq!(amount("\u{20a9}1,234,567원"), Some(1_234_567));
        // 외화는 v1 범위 밖이다 — 소수 단위(센트)와 환율 맥락이 필요하고, KRW 로
        // 표시하면 통화가 틀린 값이 된다. 지어내지 않고 아예 뽑지 않는다.
        assert!(scan_text("$1,234").is_empty());
        assert!(scan_text("USD 1,234").is_empty());
        assert!(scan_text("\u{20ac}1,234").is_empty());
        assert!(scan_text("\u{00a5}1,234").is_empty());
    }

    #[test]
    fn measurement_units_are_recognized() {
        for (text, unit) in [
            ("35\u{2103}", "\u{2103}"),
            ("30kW", "kW"),
            ("120\u{33a5}", "\u{33a5}"),
            ("3\u{33ca}", "\u{33ca}"),
            ("12개월", "개월"),
            ("3분기", "분기"),
            ("5층", "층"),
        ] {
            assert_eq!(one(text).unit.as_deref(), Some(unit), "{text}");
        }
    }

    #[test]
    fn amount_prefix_is_not_a_syllable_of_another_word() {
        // `지금 5명` 의 `금` 은 금액 접두가 아니다.
        let found = scan_text("지금 5명");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].kind, DataKind::Number);
        assert_eq!(found[0].raw, "5명");
    }

    #[test]
    fn quantities_carry_a_unit() {
        let item = one("12개");
        assert_eq!(item.kind, DataKind::Number);
        assert_eq!(item.unit.as_deref(), Some("개"));
        assert_eq!(item.normalized, Some(Normalized::Int(12)));

        let item = one("3.5%");
        assert_eq!(item.unit.as_deref(), Some("%"));
        assert_eq!(item.normalized, Some(Normalized::Float(3.5)));

        let item = one("1,000명");
        assert_eq!(item.unit.as_deref(), Some("명"));
        assert_eq!(item.normalized, Some(Normalized::Int(1000)));

        // 단위가 없으면 수량이 아니다.
        assert!(scan_text("표에 1234 가 있다").is_empty());
    }

    #[test]
    fn hangul_unit_must_be_adjacent() {
        // `3 개요` 의 `개` 를 단위로 삼키면 안 된다.
        assert!(
            scan_text("표 3 개요").is_empty(),
            "{:?}",
            scan_text("표 3 개요")
        );
        // 기호 단위는 공백 하나를 허용한다.
        assert_eq!(one("62.9 %").unit.as_deref(), Some("%"));
    }

    #[test]
    fn latin_unit_needs_a_boundary() {
        assert_eq!(one("210mm").unit.as_deref(), Some("mm"));
        assert!(
            scan_text("3gb").is_empty(),
            "라틴 단위 뒤 영숫자는 단위가 아님"
        );
    }

    #[test]
    fn items_do_not_overlap() {
        // 날짜가 먼저 구간을 가져가므로 `8` 이 수량으로 다시 잡히지 않는다.
        let found = scan_text("2026년 8월 2일 예산 1,234,567원 집행률 62.9%");
        let kinds: Vec<DataKind> = found.iter().map(|f| f.kind).collect();
        assert_eq!(
            kinds,
            vec![DataKind::Date, DataKind::Amount, DataKind::Number],
            "{found:?}"
        );
        for pair in found.windows(2) {
            assert!(
                pair[0].char_offset + pair[0].length <= pair[1].char_offset,
                "구간이 겹침: {pair:?}"
            );
        }
    }

    #[test]
    fn char_offsets_are_character_based() {
        // 한글이 앞에 있어도 오프셋은 바이트가 아니라 문자 단위다.
        let found = scan_text("지급액은 1,000원");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].char_offset, 5, "{found:?}");
        assert_eq!(found[0].length, "1,000원".chars().count(), "{found:?}");
    }

    #[test]
    fn long_digit_runs_do_not_blow_up() {
        // ReDoS 대응의 실측 — 되추적이 없으므로 대형 입력에서도 즉시 끝난다.
        let text = "1".repeat(200_000);
        let started = std::time::Instant::now();
        let found = scan_text(&text);
        assert!(found.is_empty(), "단위 없는 숫자는 항목이 아니다");
        assert!(
            started.elapsed().as_secs() < 2,
            "20만 자 숫자 열 스캔이 {:?} 걸렸습니다 — 선형이어야 합니다",
            started.elapsed()
        );
    }
}
