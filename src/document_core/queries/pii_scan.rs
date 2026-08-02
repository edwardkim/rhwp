//! 개인정보 탐지 — 주소(구역·문단·페이지·문자 오프셋)를 가진 **읽기 전용** 판정.
//!
//! 공개 전 문서에서 지워야 할 값(주민등록번호·전화번호·이메일·카드번호)을 찾아
//! "무엇을, 어디서" 를 먼저 보여주는 것이 이 모듈의 전부다. 편집은 하지 않는다 —
//! 실제 마스킹은 CLI 가 기존 치환 경로(`replace_all_native`)로 수행한다.
//!
//! # 설계 원칙: 오탐 0 우선
//!
//! 마스킹은 **되돌릴 수 없다**. 오탐 하나가 본문 숫자를 영구히 훼손하므로, 형태가
//! 맞아도 **검증을 통과하지 못하면 탐지하지 않는다**:
//!
//! - 주민등록번호: 생년월일 유효성 + 성별/세기 코드 + 검증 숫자(mod 11) 전부 통과해야 한다.
//! - 카드번호: Luhn 검증을 통과해야 한다.
//! - 전화번호: 하이픈이 있는 이동전화(01X)·서울(02) 형태만 본다. 하이픈 없는 긴 숫자열,
//!   그 밖의 지역번호는 회계 코드·문서번호와 구별할 근거가 없어 **의도적으로 제외**한다.
//! - 이메일: 지역/도메인 문자 집합과 최상위 도메인(영문 2자 이상)을 모두 만족해야 한다.
//!
//! 어느 규칙이든 앞뒤가 같은 부류의 문자(숫자 옆의 숫자 등)면 더 긴 토큰의 일부로 보고
//! 버린다 — 22자리 계좌번호 안에서 16자리 부분열을 카드로 오인하지 않기 위해서다.
//!
//! # 주소
//!
//! 매치 주소는 [`DocumentCore::grep`] 을 재사용해 얻는다. 같은 값이 표 셀·글상자에 다시
//! 나오면 그 자리도 함께 보고되므로, 사용자는 마스킹 전에 전 발생 지점을 볼 수 있다.

use serde::Serialize;

use crate::document_core::DocumentCore;
use crate::model::control::Control;

/// 탐지 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PiiKind {
    /// 주민등록번호 (`######-#######`, 검증 숫자 통과).
    Ssn,
    /// 카드번호 (4-4-4-4 등, Luhn 통과).
    Card,
    /// 전화번호 (이동전화 01X, 서울 02 — 하이픈 필수).
    Phone,
    /// 이메일 주소.
    Email,
}

impl PiiKind {
    /// CLI `--kind` 토큰.
    pub fn as_str(self) -> &'static str {
        match self {
            PiiKind::Ssn => "ssn",
            PiiKind::Card => "card",
            PiiKind::Phone => "phone",
            PiiKind::Email => "email",
        }
    }

    /// CLI 토큰을 종류로 해석한다. 알 수 없는 토큰은 `None`.
    pub fn parse(token: &str) -> Option<PiiKind> {
        match token {
            "ssn" => Some(PiiKind::Ssn),
            "card" => Some(PiiKind::Card),
            "phone" => Some(PiiKind::Phone),
            "email" => Some(PiiKind::Email),
            _ => None,
        }
    }

    /// `--kind all` 이 뜻하는 전 종류. 탐지 우선순위 순서다(겹치면 앞이 이긴다).
    pub const fn all() -> [PiiKind; 4] {
        [PiiKind::Ssn, PiiKind::Card, PiiKind::Phone, PiiKind::Email]
    }
}

/// 탐지 결과 하나 — 원문·마스킹 결과·문서 주소.
#[derive(Debug, Clone, Serialize)]
pub struct PiiFinding {
    /// 탐지 종류 (`ssn`·`card`·`phone`·`email`).
    pub kind: &'static str,
    /// 탐지된 원문. **개인정보 그 자체**이므로 로그·이슈에 그대로 붙이지 않는다.
    pub raw: String,
    /// 마스킹 결과 — 원문과 **문자 수가 같다**.
    pub masked: String,
    /// 구역 인덱스.
    pub section: usize,
    /// 본문 문단 인덱스 (표 셀·글상자 매치는 그 컨트롤을 담은 본문 문단).
    pub paragraph: usize,
    /// 0부터 시작하는 글로벌 페이지 번호. 조판에 배치되지 않은 문단이면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// 문단 텍스트 내 시작 위치 (문자 단위).
    #[serde(rename = "charOffset")]
    pub char_offset: usize,
}

/// 텍스트 한 조각에서 찾은 후보 — `(종류, 시작 문자 위치, 문자 길이)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Candidate {
    kind: PiiKind,
    start: usize,
    len: usize,
}

/// 자릿수를 유지한 마스킹 결과를 만든다.
///
/// 영숫자만 `mask` 로 바꾸고 구분자(`-`·`@`·`.`·공백)는 남긴다. 길이가 변하면 조판이
/// 흔들리므로 **문자 수는 반드시 보존한다**.
pub fn mask_value(raw: &str, mask: char) -> String {
    raw.chars()
        .map(|c| if c.is_alphanumeric() { mask } else { c })
        .collect()
}

/// 주민등록번호 검증 숫자 (mod 11).
///
/// 앞 12자리에 가중치 `[2,3,4,5,6,7,8,9,2,3,4,5]` 를 곱해 더하고, `(11 - 합 % 11) % 10`
/// 이 13번째 자리와 같아야 한다. 이 검증이 없으면 `123456-1234567` 같은 예시 문자열까지
/// 지워 버린다.
fn ssn_checksum_ok(digits: &[u8; 13]) -> bool {
    const WEIGHTS: [u32; 12] = [2, 3, 4, 5, 6, 7, 8, 9, 2, 3, 4, 5];
    let sum: u32 = digits
        .iter()
        .zip(WEIGHTS)
        .map(|(d, w)| u32::from(*d) * w)
        .sum();
    let check = (11 - (sum % 11)) % 10;
    check == u32::from(digits[12])
}

/// 그레고리력 월별 일수 (윤년 반영).
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// 성별/세기 코드로 출생 세기를 정한다. 1~8 이외는 탐지하지 않는다.
///
/// 9·0(1800년대 출생)은 현존 인구가 사실상 없어 오탐 쪽 비용이 크다 — 제외한다.
fn ssn_century(code: u8) -> Option<u32> {
    match code {
        1 | 2 | 5 | 6 => Some(1900),
        3 | 4 | 7 | 8 => Some(2000),
        _ => None,
    }
}

/// 주민등록번호 유효성 — 날짜·성별코드·검증숫자를 모두 본다.
fn ssn_is_valid(digits: &[u8; 13]) -> bool {
    let century = match ssn_century(digits[6]) {
        Some(c) => c,
        None => return false,
    };
    let year = century + digits[0] as u32 * 10 + digits[1] as u32;
    let month = digits[2] as u32 * 10 + digits[3] as u32;
    let day = digits[4] as u32 * 10 + digits[5] as u32;
    if !(1..=12).contains(&month) {
        return false;
    }
    if day == 0 || day > days_in_month(year, month) {
        return false;
    }
    ssn_checksum_ok(digits)
}

/// Luhn 검증 — 카드번호의 표준 체크섬.
fn luhn_ok(digits: &[u8]) -> bool {
    if digits.len() < 13 {
        return false;
    }
    let mut sum = 0u32;
    for (i, d) in digits.iter().rev().enumerate() {
        let mut v = *d as u32;
        if i % 2 == 1 {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
    }
    sum.is_multiple_of(10)
}

/// 이메일 지역부(local-part)에 허용하는 문자.
fn is_email_local(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
}

/// 이메일 도메인 라벨에 허용하는 문자.
fn is_email_domain(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}

/// `chars[at]` 이 숫자열의 한복판인지 — 앞뒤가 숫자면 더 긴 토큰의 일부다.
fn digit_boundary_ok(chars: &[char], start: usize, end: usize) -> bool {
    let left_ok = start == 0 || !chars[start - 1].is_ascii_digit();
    let right_ok = end >= chars.len() || !chars[end].is_ascii_digit();
    left_ok && right_ok
}

/// `chars[start..]` 에서 `pattern` 이 뜻하는 숫자 그룹 배열을 읽는다.
///
/// `pattern` 은 각 그룹의 자릿수이고, 그룹 사이에는 **같은** 구분자 한 글자(`-` 또는
/// 공백)가 와야 한다. 성공하면 `(끝 위치, 숫자열)`.
fn read_digit_groups(chars: &[char], start: usize, pattern: &[usize]) -> Option<(usize, Vec<u8>)> {
    let mut i = start;
    let mut digits: Vec<u8> = Vec::new();
    let mut separator: Option<char> = None;
    for (g, want) in pattern.iter().enumerate() {
        if g > 0 {
            let sep = *chars.get(i)?;
            if sep != '-' && sep != ' ' {
                return None;
            }
            match separator {
                // 구분자가 섞이면(`1234-5678 9012`) 사람이 쓴 카드번호로 보지 않는다.
                Some(prev) if prev != sep => return None,
                Some(_) => {}
                None => separator = Some(sep),
            }
            i += 1;
        }
        for _ in 0..*want {
            let c = *chars.get(i)?;
            if !c.is_ascii_digit() {
                return None;
            }
            digits.push(c as u8 - b'0');
            i += 1;
        }
    }
    Some((i, digits))
}

/// 위치 `start` 에서 시작하는 주민등록번호 후보를 읽는다.
fn scan_ssn(chars: &[char], start: usize) -> Option<usize> {
    let (end, digits) = read_digit_groups(chars, start, &[6, 7])?;
    if !digit_boundary_ok(chars, start, end) {
        return None;
    }
    let mut fixed = [0u8; 13];
    fixed.copy_from_slice(&digits);
    if !ssn_is_valid(&fixed) {
        return None;
    }
    Some(end)
}

/// 위치 `start` 에서 시작하는 카드번호 후보를 읽는다 (Luhn 필수).
///
/// 받는 형태: `4-4-4-4`(16자리, `-`/공백), Amex `4-6-5`(15자리), 그리고 구분자 없는
/// 15·16자리 연속 숫자. 13·14·19자리 등 그 밖의 길이는 회계 숫자와 구별할 근거가 없어
/// 제외한다.
fn scan_card(chars: &[char], start: usize) -> Option<usize> {
    const PATTERNS: [&[usize]; 4] = [&[4, 4, 4, 4], &[4, 6, 5], &[16], &[15]];
    for pattern in PATTERNS {
        let Some((end, digits)) = read_digit_groups(chars, start, pattern) else {
            continue;
        };
        if !digit_boundary_ok(chars, start, end) {
            continue;
        }
        if luhn_ok(&digits) {
            return Some(end);
        }
    }
    None
}

/// 위치 `start` 에서 시작하는 전화번호 후보를 읽는다.
///
/// 이동전화 `01[016789]-3~4자리-4자리` 와 서울 `02-3~4자리-4자리` 만 본다. 하이픈이
/// 없으면 보지 않는다 — `01012345678` 은 문서번호와 형태가 같다.
fn scan_phone(chars: &[char], start: usize) -> Option<usize> {
    const MOBILE_SECOND: [char; 6] = ['0', '1', '6', '7', '8', '9'];
    if *chars.get(start)? != '0' {
        return None;
    }
    let second = *chars.get(start + 1)?;
    let prefix_len = if second == '1' {
        // 01X — 세 번째 자리가 이동전화 식별번호여야 한다.
        let third = *chars.get(start + 2)?;
        if !MOBILE_SECOND.contains(&third) {
            return None;
        }
        3
    } else if second == '2' {
        2
    } else {
        return None;
    };
    // 국번은 3자리 또는 4자리다. 긴 쪽을 먼저 시도한다.
    for middle in [4usize, 3] {
        let pattern = [prefix_len, middle, 4];
        let Some((end, _)) = read_digit_groups(chars, start, &pattern) else {
            continue;
        };
        if digit_boundary_ok(chars, start, end) {
            return Some(end);
        }
    }
    None
}

/// 위치 `at` 의 `@` 를 중심으로 이메일 주소 범위를 넓힌다.
///
/// 성공하면 `(시작, 끝)` 문자 위치.
fn scan_email(chars: &[char], at: usize) -> Option<(usize, usize)> {
    // 지역부 — 왼쪽으로 확장. 점으로 시작/끝나는 지역부는 받지 않는다.
    let mut start = at;
    while start > 0 && is_email_local(chars[start - 1]) {
        start -= 1;
    }
    if start == at {
        return None;
    }
    while start < at && chars[start] == '.' {
        start += 1;
    }
    if start == at || chars[at - 1] == '.' {
        return None;
    }

    // 도메인 — 라벨(영숫자·하이픈) 을 점으로 이어 최소 2개, 마지막은 영문 2자 이상.
    let mut i = at + 1;
    let mut labels: Vec<(usize, usize)> = Vec::new();
    loop {
        let label_start = i;
        while i < chars.len() && is_email_domain(chars[i]) {
            i += 1;
        }
        if i == label_start {
            return None;
        }
        labels.push((label_start, i));
        if i < chars.len() && chars[i] == '.' {
            // 점 뒤에 라벨이 없으면(문장 끝) 여기서 끊는다.
            if chars.get(i + 1).is_some_and(|c| is_email_domain(*c)) {
                i += 1;
                continue;
            }
        }
        break;
    }
    if labels.len() < 2 {
        return None;
    }
    let (tld_start, tld_end) = *labels.last().expect("라벨 2개 이상");
    if tld_end - tld_start < 2
        || !chars[tld_start..tld_end]
            .iter()
            .all(|c| c.is_ascii_alphabetic())
    {
        return None;
    }
    // 라벨은 하이픈으로 시작하거나 끝날 수 없다.
    for &(label_start, label_end) in &labels {
        if chars[label_start] == '-' || chars[label_end - 1] == '-' {
            return None;
        }
    }
    Some((start, tld_end))
}

/// 텍스트 한 조각에서 개인정보 후보를 찾는다 (순수 함수 — 테스트 가능).
///
/// 겹치는 후보는 [`PiiKind::all`] 순서(주민번호 → 카드 → 전화 → 이메일)로 앞이 이긴다.
fn detect(text: &str, kinds: &[PiiKind]) -> Vec<Candidate> {
    let chars: Vec<char> = text.chars().collect();
    let mut found: Vec<Candidate> = Vec::new();
    let wants = |k: PiiKind| kinds.contains(&k);

    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '@' && wants(PiiKind::Email) {
            if let Some((s, e)) = scan_email(&chars, i) {
                found.push(Candidate {
                    kind: PiiKind::Email,
                    start: s,
                    len: e - s,
                });
                i = e;
                continue;
            }
        }
        if chars[i].is_ascii_digit() {
            let mut hit: Option<Candidate> = None;
            if wants(PiiKind::Ssn) {
                if let Some(end) = scan_ssn(&chars, i) {
                    hit = Some(Candidate {
                        kind: PiiKind::Ssn,
                        start: i,
                        len: end - i,
                    });
                }
            }
            if hit.is_none() && wants(PiiKind::Card) {
                if let Some(end) = scan_card(&chars, i) {
                    hit = Some(Candidate {
                        kind: PiiKind::Card,
                        start: i,
                        len: end - i,
                    });
                }
            }
            if hit.is_none() && wants(PiiKind::Phone) {
                if let Some(end) = scan_phone(&chars, i) {
                    hit = Some(Candidate {
                        kind: PiiKind::Phone,
                        start: i,
                        len: end - i,
                    });
                }
            }
            if let Some(c) = hit {
                i = c.start + c.len;
                found.push(c);
                continue;
            }
        }
        i += 1;
    }
    found
}

/// 텍스트에서 탐지된 원문 값들을 문서 순서로 돌려준다 (중복 제거하지 않음).
///
/// 테스트가 규칙 자체를 직접 겨눌 수 있도록 공개한다 — 문서를 만들지 않고도
/// "체크섬이 틀린 문자열은 탐지되지 않는다"를 고정할 수 있다.
pub fn detect_values(text: &str, kinds: &[PiiKind]) -> Vec<(PiiKind, String)> {
    let chars: Vec<char> = text.chars().collect();
    detect(text, kinds)
        .into_iter()
        .map(|c| {
            (
                c.kind,
                chars[c.start..c.start + c.len].iter().collect::<String>(),
            )
        })
        .collect()
}

/// 문단·표 셀·글상자 텍스트를 문서 순서로 모은다 (읽기 전용).
fn collect_texts(doc: &DocumentCore) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for section in &doc.document.sections {
        for para in &section.paragraphs {
            texts.push(para.text.clone());
            for ctrl in &para.controls {
                match ctrl {
                    Control::Table(table) => {
                        for cell in &table.cells {
                            for cp in &cell.paragraphs {
                                texts.push(cp.text.clone());
                            }
                        }
                    }
                    Control::Shape(shape) => {
                        if let Some(tb) =
                            crate::document_core::helpers::get_textbox_from_shape(shape)
                        {
                            for tp in &tb.paragraphs {
                                texts.push(tp.text.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    texts
}

impl DocumentCore {
    /// 문서에서 지워야 할 개인정보를 찾아 **주소가 붙은** 목록을 돌려준다.
    ///
    /// 문서를 바꾸지 않는다. 같은 값이 여러 곳에 있으면 전 발생 지점이 나온다
    /// (마스킹은 전량 치환이므로 사용자가 미리 전부 볼 수 있어야 한다).
    pub fn scan_pii(&self, kinds: &[PiiKind], mask: char) -> Vec<PiiFinding> {
        // ① 값 수집 — 문서 순서, 중복 제거(같은 값을 두 번 치환하지 않는다).
        let mut values: Vec<(PiiKind, String)> = Vec::new();
        for text in collect_texts(self) {
            for (kind, raw) in detect_values(&text, kinds) {
                if !values.iter().any(|(_, v)| *v == raw) {
                    values.push((kind, raw));
                }
            }
        }

        // ② 주소 붙이기 — grep 재사용(구역·문단·페이지·오프셋을 이미 계산한다).
        let mut out: Vec<PiiFinding> = Vec::new();
        for (kind, raw) in values {
            let masked = mask_value(&raw, mask);
            for m in self.grep(&raw, true, None) {
                out.push(PiiFinding {
                    kind: kind.as_str(),
                    raw: raw.clone(),
                    masked: masked.clone(),
                    section: m.section,
                    paragraph: m.paragraph,
                    page: m.page,
                    char_offset: m.char_offset,
                });
            }
        }
        out.sort_by_key(|f| (f.section, f.paragraph, f.char_offset));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [PiiKind; 4] = PiiKind::all();

    /// 오탐 0 — 형태만 같고 검증 숫자가 틀린 주민등록번호는 탐지되지 않는다.
    #[test]
    fn invalid_ssn_checksum_is_not_detected() {
        // 900101-1234567 은 형태는 맞지만 mod 11 검증 숫자가 틀리다.
        let found = detect_values("계약자 900101-1234567 참조", &ALL);
        assert!(
            found.iter().all(|(k, _)| *k != PiiKind::Ssn),
            "검증 숫자가 틀린 문자열을 주민등록번호로 탐지했다: {found:?}"
        );
    }

    /// 검증 숫자가 맞으면 탐지된다 — 규칙이 통째로 죽어 있으면 위 테스트가 공허해진다.
    #[test]
    fn valid_ssn_is_detected() {
        let found = detect_values("계약자 900101-1234568 참조", &ALL);
        assert_eq!(
            found,
            vec![(PiiKind::Ssn, "900101-1234568".to_string())],
            "검증 숫자가 맞는 주민등록번호를 놓쳤다"
        );
    }

    /// 성별/세기 코드가 1~8 밖이면(9·0 = 1800년대 출생) 탐지하지 않는다.
    ///
    /// 검증 숫자만으로는 `######-#######` 형태의 임의 숫자쌍 11개 중 1개가 우연히
    /// 통과한다. 세기 코드까지 걸어야 오탐이 실질적으로 사라진다.
    #[test]
    fn out_of_range_gender_code_is_not_detected() {
        for raw in ["900101-9234568", "900101-0234568"] {
            let found = detect_values(raw, &ALL);
            assert!(
                found.iter().all(|(k, _)| *k != PiiKind::Ssn),
                "세기 코드가 범위 밖인데 탐지했다: {raw} → {found:?}"
            );
        }
    }

    /// 윤년 2월 29일은 유효하다 — 날짜 검사가 과하게 좁으면 실제 개인정보를 놓친다.
    #[test]
    fn leap_day_birth_date_is_accepted() {
        // 000229-3……: 세기 코드 3 → 2000년생, 2000년은 윤년(400으로 나뉨).
        let found = detect_values("000229-3123454", &ALL);
        assert_eq!(
            found,
            vec![(PiiKind::Ssn, "000229-3123454".to_string())],
            "윤년 2월 29일 생년월일을 버렸다"
        );
    }

    /// 존재하지 않는 날짜(13월·2월 30일)는 검증 숫자와 무관하게 버린다.
    #[test]
    fn impossible_birth_date_is_not_detected() {
        for raw in ["901301-1234567", "900230-1234567"] {
            let found = detect_values(raw, &ALL);
            assert!(
                found.iter().all(|(k, _)| *k != PiiKind::Ssn),
                "불가능한 생년월일을 주민등록번호로 탐지했다: {raw} → {found:?}"
            );
        }
    }

    /// Luhn 을 통과하지 못하는 16자리는 카드번호가 아니다.
    #[test]
    fn non_luhn_card_is_not_detected() {
        let found = detect_values("계좌 1234-5678-9012-3456 입금", &ALL);
        assert!(
            found.iter().all(|(k, _)| *k != PiiKind::Card),
            "Luhn 실패 숫자열을 카드번호로 탐지했다: {found:?}"
        );
        let ok = detect_values("카드 4111-1111-1111-1111 승인", &ALL);
        assert_eq!(
            ok,
            vec![(PiiKind::Card, "4111-1111-1111-1111".to_string())],
            "Luhn 을 통과하는 카드번호를 놓쳤다"
        );
    }

    /// 구분자를 섞어 쓴 숫자열은 사람이 쓴 카드번호로 보지 않는다.
    ///
    /// `1234-5678 9012-3456` 같은 형태는 표 안의 코드가 잘못 붙은 것일 가능성이 크다.
    /// 하이픈/공백 중 하나로 일관돼야 카드번호로 인정한다.
    #[test]
    fn mixed_separators_are_not_a_card() {
        let found = detect_values("4111-1111 1111-1111", &[PiiKind::Card]);
        assert!(
            found.is_empty(),
            "구분자가 섞인 숫자열을 카드번호로 탐지했다: {found:?}"
        );
    }

    /// 공백 구분·연속 16자리·Amex 4-6-5 도 Luhn 을 통과하면 카드번호다.
    #[test]
    fn card_accepted_shapes() {
        for raw in [
            "4111 1111 1111 1111",
            "4111111111111111",
            "3782-822463-10005",
        ] {
            let found = detect_values(raw, &[PiiKind::Card]);
            assert_eq!(
                found,
                vec![(PiiKind::Card, raw.to_string())],
                "카드번호 형태를 놓쳤다: {raw}"
            );
        }
    }

    /// 13자리 카드(구형 Visa)는 회계 숫자와 구별할 근거가 없어 **의도적으로 제외**한다.
    ///
    /// 놓치는 쪽을 택한 결정이다 — 이 케이스는 규칙이 우연히 넓어지면 즉시 깨진다.
    #[test]
    fn thirteen_digit_card_is_out_of_scope_by_design() {
        let found = detect_values("4222222222222", &[PiiKind::Card]);
        assert!(
            found.is_empty(),
            "범위 밖 자릿수를 카드번호로 탐지했다: {found:?}"
        );
    }

    /// 더 긴 숫자열의 부분열을 잘라 오탐하지 않는다.
    #[test]
    fn longer_digit_run_is_not_sliced() {
        let found = detect_values("문서번호 41111111111111119999", &ALL);
        assert!(
            found.is_empty(),
            "긴 숫자열에서 부분 매치가 났다: {found:?}"
        );
    }

    /// 전화번호도 앞뒤 숫자 경계를 본다 — 긴 코드 안의 부분열을 잡지 않는다.
    #[test]
    fn phone_respects_digit_boundaries() {
        let found = detect_values("1010-1234-5678", &[PiiKind::Phone]);
        assert!(
            found.is_empty(),
            "앞에 숫자가 붙은 문자열을 전화번호로 탐지했다: {found:?}"
        );
    }

    /// 전화번호는 하이픈이 있는 이동전화·서울 국번만 본다.
    #[test]
    fn phone_rules() {
        for raw in [
            "010-1234-5678",
            "010-123-4567",
            "011-234-5678",
            "016-234-5678",
            "017-234-5678",
            "018-234-5678",
            "019-234-5678",
            "02-123-4567",
            "02-1234-5678",
        ] {
            assert_eq!(
                detect_values(raw, &[PiiKind::Phone]),
                vec![(PiiKind::Phone, raw.to_string())],
                "전화번호를 놓쳤다: {raw}"
            );
        }
        // 하이픈 없는 숫자열·이동전화가 아닌 01X·02 밖의 지역번호는 의도적으로 제외한다.
        for raw in [
            "01012345678",
            "031-123-4567",
            "051-123-4567",
            "012-345-6789",
        ] {
            assert!(
                detect_values(raw, &[PiiKind::Phone]).is_empty(),
                "범위 밖 전화 형태를 탐지했다: {raw}"
            );
        }
    }

    /// 이메일은 최상위 도메인까지 확인한다.
    #[test]
    fn email_rules() {
        assert_eq!(
            detect_values("문의 hong.gil-dong@example.co.kr 로", &[PiiKind::Email]),
            vec![(PiiKind::Email, "hong.gil-dong@example.co.kr".to_string())]
        );
        // 문장 끝의 마침표는 주소에 포함하지 않는다 — 포함하면 치환이 문장을 깬다.
        assert_eq!(
            detect_values("회신은 hong@example.com.", &[PiiKind::Email]),
            vec![(PiiKind::Email, "hong@example.com".to_string())]
        );
        // 대문자 도메인도 주소다.
        assert_eq!(
            detect_values("USER@EXAMPLE.COM", &[PiiKind::Email]),
            vec![(PiiKind::Email, "USER@EXAMPLE.COM".to_string())]
        );
        // 도메인 라벨이 하나뿐이거나 지역부가 없으면 주소가 아니다.
        for raw in [
            "@example",
            "a@localhost",
            "user@example.12",
            "a@-example.com",
            "a@example-.com",
        ] {
            assert!(
                detect_values(raw, &[PiiKind::Email]).is_empty(),
                "주소가 아닌 문자열을 이메일로 탐지했다: {raw}"
            );
        }
    }

    /// `--kind` 로 좁히면 그 종류만 본다 — 다른 종류는 아예 판정하지 않는다.
    #[test]
    fn kind_filter_is_respected() {
        let text = "홍길동 900101-1234568 / 010-1234-5678 / hong@example.com / 4111-1111-1111-1111";
        for kind in ALL {
            let found = detect_values(text, &[kind]);
            assert!(
                !found.is_empty() && found.iter().all(|(k, _)| *k == kind),
                "{kind:?} 로 좁혔는데 결과가 이상하다: {found:?}"
            );
        }
        assert_eq!(
            detect_values(text, &ALL).len(),
            4,
            "전 종류 탐지 수가 다르다"
        );
    }

    /// 마스킹은 문자 수를 보존하고 구분자를 남긴다.
    #[test]
    fn mask_preserves_length() {
        for raw in [
            "900101-1234568",
            "010-1234-5678",
            "4111-1111-1111-1111",
            "hong@example.com",
        ] {
            for mask in ['*', '#', '●'] {
                let masked = mask_value(raw, mask);
                assert_eq!(
                    masked.chars().count(),
                    raw.chars().count(),
                    "마스킹으로 길이가 바뀌었다: {raw} → {masked}"
                );
                assert!(
                    !masked.chars().any(|c| c.is_alphanumeric()),
                    "마스킹 후에도 영숫자가 남았다: {masked}"
                );
            }
        }
        assert_eq!(mask_value("900101-1234568", '*'), "******-*******");
        assert_eq!(mask_value("hong@example.com", '*'), "****@*******.***");
    }
}
