//! 숨은 마크(텍스트 스테가노그래피) 탐지 + 방어적 정화 코어 — **읽기 전용 판정**과
//! **순수 문자열 정화**만 담는다. 문서를 심는(embed) 기능은 만들지 않는다.
//!
//! ## 목적 — 방어/탐지 전용
//!
//! (1) 받은 문서에 누군가 심어 둔 **은닉 추적·워터마크**(보이지 않는 문자로 실은 식별자,
//! 동형자 서명, 공백 비트열)를 찾아내고, (2) 내 손을 거치는 문서에서 그것을 **지워**
//! 프라이버시를 지킨다. 은닉 워터마크를 **심는** 도구도, AI 생성 표식을 벗겨 사람 글로
//! 위장하려는 도구도 아니다 — 저장소의 `inspect`/`sanitize` 보안 계열과 같은 결이다.
//!
//! ## `text_security`(= `inspect unicode`)와 겹치지 않고 더하는 것
//!
//! [`crate::document_core::text_security`] 의 `scan_deception` 은 "화면과 바이트가
//! 어긋나는" 유니코드 기만을 축별로 신고한다(제로폭·bidi·태그·동형자). 이 모듈은 그 위에
//! **스테가노그래피(은닉 payload) 관점**을 더한다:
//!
//! - **제로폭 비트 채널 복호** — 제로폭 문자 두 종을 0/1 로 읽어 심어진 **비트열**(과
//!   8비트 배수면 **ASCII**)을 복원해 보여 준다. `scan_deception` 은 열 길이만 보고하지
//!   payload 를 풀지 않는다. 이 복호가 "워터마크를 찾아낸다"의 핵심이다.
//! - **넓은 비가시 집합** — U+180E·U+2061–2064(보이지 않는 수학 연산자)까지 본다.
//! - **공백 인코딩(whitespace stego)** — 뒤따르는 공백/탭 열, 탭·공백이 섞인 내부 열처럼
//!   비트를 실을 수 있는 공백 이상(anomaly)을 신고한다. `scan_deception` 에 없는 축이다.
//!
//! ## 정화는 순수 함수 [`sanitize_stego`] — 문서 쓰기 경로는 분리한다
//!
//! `inspect` 는 저장소 규약상 **문서를 고치지 않는 읽기 전용** 명령군이다. 그래서 이
//! 모듈의 탐지는 `inspect watermark` 로 노출하고, 실제 정화(문서 재저장)는 검증된
//! 본문 치환 경로(`delete/insert_text_native`)에 얹는 `edit` 계열 후속 작업으로 붙인다.
//! 여기서는 그 정화의 **핵심 변환**을 순수 함수로 제공한다 — `&str` 을 받아 정화된
//! `String` 을 돌려주고, 탐지가 신고하는 마크만 지운다(정당한 것은 남긴다). 탐지와 정화가
//! 같은 판정 헬퍼([`hidden_run_is_benign`]·[`homoglyph_offsets`])를 공유하므로 "정화 후
//! 재검사 = 0" 이 구조적으로 성립한다.
//!
//! ## 절대 훼손하지 않는 정당한 쓰임 (보수적 유지)
//!
//! - **맨 앞 BOM**(U+FEFF at 0) — 정상 인코딩 표식.
//! - **옛한글 조판 제로폭** — 국어 고전 자료가 PUA 옛한글 낱자에 잇대는 U+200B(조판 보조).
//! - **이모지 ZWJ** — 👨‍👩‍👧 같은 이모지 결합 열의 U+200D.
//! - **순수 비라틴 낱말** — 러시아어·그리스어 인용문의 동형자는 위장이 아니다(라틴 낱말에
//!   섞였을 때만 신고·정규화한다).
//! - **내부 정렬 공백** — 탭이 섞이지 않은 긴 공백 열은 조판일 수 있어 정화하지 않는다.

use crate::document_core::text_security::{confusable_to_latin, format_codepoint, Severity};
use std::collections::BTreeSet;

/// `inspect watermark` 가 신고하는 숨은 마크 축.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MarkKind {
    /// 제로폭·비가시 문자 열 — U+200B/C/D·FEFF·2060·180E·2061–2064·태그 문자.
    HiddenChar,
    /// 라틴 낱말에 섞인 동형자 — 키릴 а vs 라틴 a (워터마크·스푸핑 채널).
    Homoglyph,
    /// 공백 인코딩 — 뒤따르는 공백/탭 열, 탭·공백이 섞인 내부 열.
    Whitespace,
}

impl MarkKind {
    /// 선언 순서가 곧 보고 순서다 — 소비자가 축 목록을 열거할 때 쓴다.
    pub const ALL: [MarkKind; 3] = [
        MarkKind::HiddenChar,
        MarkKind::Homoglyph,
        MarkKind::Whitespace,
    ];

    /// 봉투 `findings[].kind` 값 — 소비자가 문자열로 분기한다.
    pub fn label(self) -> &'static str {
        match self {
            MarkKind::HiddenChar => "hidden_char",
            MarkKind::Homoglyph => "homoglyph",
            MarkKind::Whitespace => "whitespace",
        }
    }

    /// `--kind` 필터 어휘. CLI 플래그와 MCP `inputSchema` 의 enum 이 이 하나를 공유한다.
    pub fn filter_name(self) -> &'static str {
        match self {
            MarkKind::HiddenChar => "hidden",
            MarkKind::Homoglyph => "homoglyph",
            MarkKind::Whitespace => "whitespace",
        }
    }

    /// `--kind <값>` 파싱. `all`(=필터 없음)은 호출자가 `None` 으로 다룬다.
    pub fn from_filter(s: &str) -> Option<MarkKind> {
        MarkKind::ALL.into_iter().find(|k| k.filter_name() == s)
    }

    /// 봉투 `findings[].why` — 에이전트가 그대로 사용자에게 전달할 수 있는 한 줄.
    pub fn why(self) -> &'static str {
        match self {
            MarkKind::HiddenChar => {
                "보이지 않는 문자 열입니다 — 화면에 없는 식별자·지시가 텍스트에 숨어 있을 수 있습니다(제로폭 비트열이면 복원해 보여 줍니다)"
            }
            MarkKind::Homoglyph => {
                "라틴 낱말에 다른 스크립트의 동형자가 섞였습니다 — 화면상 구별되지 않는 워터마크·위장일 수 있습니다"
            }
            MarkKind::Whitespace => {
                "비정상 공백 열입니다 — 뒤따르는 공백이나 탭·공백 혼합 열에 비트를 실을 수 있습니다"
            }
        }
    }
}

/// 탐지 1건. 위치(문자 오프셋)·열 길이·지목 코드포인트와, 사람이 읽을 발췌·해설을 담는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StegoFinding {
    pub kind: MarkKind,
    pub severity: Severity,
    /// 문단 텍스트 안 위치(문자 단위, 0 기준). 연속 열이면 그 열의 첫 글자다.
    pub char_offset: usize,
    /// 같은 종류가 몇 글자 연속인지. 낱개면 1.
    pub run_length: usize,
    /// 지목 코드포인트(중복 제거·발견 순). HiddenChar 는 열에 쓰인 종류, Homoglyph 는 동형자 1개.
    pub codepoints: Vec<u32>,
    /// 앞뒤 문맥. 비가시·제어 문자는 `<U+XXXX>` 로, 공백/탭은 눈에 보이게 드러낸다 —
    /// 보고 채널이 다시 사람을 속이면 안 된다.
    pub excerpt: String,
    /// 복호·해설. HiddenChar: 비트열(과 ASCII 복원). Homoglyph: `Т(U+0422) → T`.
    /// Whitespace: 공백/탭 개수. 해설이 없으면 `None`.
    pub detail: Option<String>,
}

// ── 코드포인트 판정 ──────────────────────────────────────────────────────────

/// 이 축이 비가시로 보는 코드포인트 — `text_security` 의 zero-width 보다 넓다.
///
/// U+2060–2064 는 WORD JOINER + 보이지 않는 수학 연산자(FUNCTION APPLICATION·INVISIBLE
/// TIMES·INVISIBLE SEPARATOR·INVISIBLE PLUS)다. U+180E(MONGOLIAN VOWEL SEPARATOR)와
/// 태그 문자(U+E0000–E007F)는 정상 한국어 문서 본문에 있을 이유가 없는 은닉 채널이다.
fn is_hidden_char(c: u32) -> bool {
    matches!(c,
        0x200B..=0x200D   // ZWSP ZWNJ ZWJ
        | 0x2060..=0x2064 // WORD JOINER + 보이지 않는 수학 연산자
        | 0xFEFF          // BOM / ZWNBSP
        | 0x180E          // MONGOLIAN VOWEL SEPARATOR
    ) || is_tag_char(c)
}

/// 태그 문자 — 렌더링되지 않는데 텍스트에는 남는다. 알려진 은닉 지시 채널.
fn is_tag_char(c: u32) -> bool {
    (0xE0000..=0xE007F).contains(&c)
}

/// 수학 연산자(U+2061–2064) — 정상 산문에 나올 이유가 없어 "정당한 쓰임" 완화에서 뺀다.
fn is_invisible_math(c: u32) -> bool {
    (0x2061..=0x2064).contains(&c)
}

/// 비트 채널에 흔히 쓰이는 제로폭 종류(태그·수학연산자 제외). PUA 조판 완화가 적용되는 집합.
fn is_zero_width_bit(c: u32) -> bool {
    matches!(c, 0x200B | 0x200C | 0x200D | 0x2060 | 0xFEFF | 0x180E)
}

/// 방향 제어 — 발췌 표기에서 드러내기 위한 판정(이 축이 직접 신고하진 않는다).
fn is_bidi(c: u32) -> bool {
    (0x202A..=0x202E).contains(&c) || (0x2066..=0x2069).contains(&c)
}

/// 사용자 정의 영역(PUA). 한/글은 옛한글 낱자·조판부호를 PUA 로 싣는다 — 곁의 제로폭은
/// 은닉이 아니라 조판 보조일 수 있다.
fn is_private_use(c: u32) -> bool {
    (0xE000..=0xF8FF).contains(&c)
        || (0xF0000..=0xFFFFD).contains(&c)
        || (0x100000..=0x10FFFD).contains(&c)
}

/// 확장 그림문자(이모지) 대략 판정 — ZWJ 이모지 열의 정당한 U+200D 를 지우지 않기 위한
/// 보수적 근사. 넓게 잡아 오히려 "지우지 않는" 쪽으로 안전하게 기운다.
fn is_emoji_like(c: u32) -> bool {
    // 0x1F000..=0x1FAFF 가 이모지 주요 블록(지역 표시자 0x1F1E6..=0x1F1FF 포함)을 덮는다.
    matches!(c,
        0x1F000..=0x1FAFF   // 이모지 주요 블록 + 지역 표시자
        | 0x2600..=0x27BF   // 기타 기호·딩벳
        | 0x2B00..=0x2BFF   // 기타 기호·화살표
        | 0xFE00..=0xFE0F   // variation selectors
    )
}

/// 라틴 글자인가 — ASCII + Latin-1/확장(×·÷ 제외). 동형자 판정의 "정상" 쪽.
fn is_latin_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c as u32, 0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0x24F)
}

/// 낱말을 이루는 글자 — 라틴이거나 라틴 동형자(키릴·그리스). 동형자로 낱말을 갈라
/// 판정을 피하는 우회를 막기 위해, 낱말 경계는 제로폭을 건너뛰며 잡는다(호출부 참조).
fn is_word_char(c: char) -> bool {
    is_latin_letter(c) || confusable_to_latin(c).is_some()
}

// ── 발췌(보고 채널 안전화) ──────────────────────────────────────────────────

const EXCERPT_RADIUS: usize = 32;

/// 앞뒤 문맥을 봉투에 실어도 안전하게 만든다 — 비가시·제어는 `<U+XXXX>` 로, 공백/탭은
/// (`reveal_ws` 면) 눈에 보이는 기호로 드러낸다. `…` 로 절단을 표시한다.
fn context_excerpt(chars: &[char], at: usize, len: usize, reveal_ws: bool) -> String {
    let start = at.saturating_sub(EXCERPT_RADIUS);
    let end = (at + len + EXCERPT_RADIUS).min(chars.len());
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    for &ch in &chars[start..end] {
        let c = ch as u32;
        if ch == '\t' {
            out.push_str(if reveal_ws { "→" } else { "<U+0009>" });
        } else if ch == ' ' && reveal_ws {
            out.push('·');
        } else if is_hidden_char(c) || is_bidi(c) || c == 0x7F || (c < 0x20) {
            out.push('<');
            out.push_str(&format_codepoint(c));
            out.push('>');
        } else {
            out.push(ch);
        }
    }
    if end < chars.len() {
        out.push('…');
    }
    out
}

// ── 제로폭 payload 복호 ──────────────────────────────────────────────────────

/// 제로폭 열을 비트 채널로 읽어 해설을 만든다.
///
/// 서로 다른 코드포인트가 정확히 2종이면 낮은 쪽을 0, 높은 쪽을 1 로(결정론) 읽어 비트열을
/// 만들고, 길이가 8의 배수이며 모든 바이트가 인쇄 가능 ASCII 면 복원 문자열도 덧붙인다.
/// 1종의 반복은 길이(unary) 인코딩 가능성만 알린다. 3종 이상은 단순 비트 채널로 단정하지 않는다.
fn decode_bits(run: &[char]) -> Option<String> {
    if run.len() < 2 {
        return None;
    }
    let mut symbols: Vec<u32> = Vec::new();
    for &ch in run {
        let c = ch as u32;
        if !symbols.contains(&c) {
            symbols.push(c);
        }
    }
    match symbols.len() {
        1 => Some(format!(
            "같은 코드포인트 {}회 연속(길이 인코딩 가능성)",
            run.len()
        )),
        2 => {
            let (mut lo, mut hi) = (symbols[0], symbols[1]);
            if lo > hi {
                std::mem::swap(&mut lo, &mut hi);
            }
            let bits: String = run
                .iter()
                .map(|&ch| if ch as u32 == lo { '0' } else { '1' })
                .collect();
            let mut detail = format!(
                "비트열({}=0, {}=1): {}",
                format_codepoint(lo),
                format_codepoint(hi),
                bits
            );
            if bits.len().is_multiple_of(8) {
                let bytes: Vec<u8> = bits
                    .as_bytes()
                    .chunks(8)
                    .map(|c| c.iter().fold(0u8, |a, &b| (a << 1) | (b - b'0')))
                    .collect();
                if bytes.iter().all(|&b| (0x20..=0x7E).contains(&b)) {
                    if let Ok(s) = String::from_utf8(bytes) {
                        detail.push_str(&format!("; ASCII \"{s}\""));
                    }
                }
            }
            Some(detail)
        }
        _ => None,
    }
}

/// 태그 문자 열이 실어 나른 ASCII 를 복원한다 (U+E0020–E007E → 0x20–0x7E).
fn decode_tags(run: &[char]) -> Option<String> {
    let mut s = String::new();
    for &ch in run {
        let c = ch as u32;
        if (0xE0020..=0xE007E).contains(&c) {
            if let Some(d) = char::from_u32(c - 0xE0000) {
                s.push(d);
            }
        }
    }
    (!s.is_empty()).then(|| format!("태그 문자 복원 ASCII \"{s}\""))
}

// ── 정당한 쓰임 판정 (탐지·정화가 공유) ─────────────────────────────────────

/// 제로폭 열 하나가 **정당한 쓰임**인가 — 탐지는 이걸 skip, 정화는 이걸 남긴다.
/// 두 경로가 같은 함수를 쓰므로 "정화 후 재검사 = 0" 이 어긋나지 않는다.
///
/// 태그 문자·수학 연산자가 하나라도 있으면 정당하지 않다(그쪽은 정상 용도가 없다).
fn hidden_run_is_benign(chars: &[char], start: usize, len: usize) -> bool {
    let slice = &chars[start..start + len];
    if slice
        .iter()
        .any(|&ch| is_tag_char(ch as u32) || is_invisible_math(ch as u32))
    {
        return false;
    }
    // 맨 앞 단일 BOM.
    if len == 1 && start == 0 && chars[start] as u32 == 0xFEFF {
        return true;
    }
    // 이모지 사이 단일 ZWJ.
    if len == 1 && chars[start] as u32 == 0x200D {
        let prev = start
            .checked_sub(1)
            .map(|i| is_emoji_like(chars[i] as u32))
            .unwrap_or(false);
        let next = chars
            .get(start + 1)
            .map(|c| is_emoji_like(*c as u32))
            .unwrap_or(false);
        if prev && next {
            return true;
        }
    }
    // PUA 인접 제로폭 조판(옛한글) — 열의 앞이나 뒤가 PUA 글자면 조판 보조로 본다.
    let before_pua = start
        .checked_sub(1)
        .map(|i| is_private_use(chars[i] as u32))
        .unwrap_or(false);
    let after_pua = chars
        .get(start + len)
        .map(|c| is_private_use(*c as u32))
        .unwrap_or(false);
    before_pua || after_pua
}

/// 라틴 낱말 안에서 **정규화 대상 동형자**의 오프셋 집합 — 탐지와 정화가 공유한다.
///
/// 낱말은 라틴/동형자 글자의 연속(제로폭은 건너뛴다). 라틴 글자가 2자 이상 있고 동형자가
/// 섞인 낱말에서만 그 동형자를 지목한다 — 순수 러시아어·그리스어 인용문은 통과시킨다.
fn homoglyph_offsets(chars: &[char]) -> BTreeSet<usize> {
    let mut set = BTreeSet::new();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        if is_word_char(chars[i]) {
            let start = i;
            while i < n && (is_word_char(chars[i]) || is_hidden_char(chars[i] as u32)) {
                i += 1;
            }
            let end = i;
            let latin = chars[start..end]
                .iter()
                .filter(|c| is_latin_letter(**c))
                .count();
            if latin >= 2 {
                for off in start..end {
                    let ch = chars[off];
                    if !is_latin_letter(ch)
                        && !is_hidden_char(ch as u32)
                        && confusable_to_latin(ch).is_some()
                    {
                        set.insert(off);
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    set
}

// ── 탐지 ────────────────────────────────────────────────────────────────────

/// 공백 인코딩 임계 — 뒤따르는 공백/탭 열의 최소 길이.
///
/// 실측 근거(samples 45건 스윕): 실제 한국어 HWP 문서는 문단 끝에 정렬용 공백/탭을 4–7자
/// 흔히 둔다. 그걸 스테가노로 올리면 경보가 통째로 무시된다 — 8자 이상만 신고해 정당한
/// 정렬 공백을 통과시킨다(실측에서 8·28자 트레일링이 남았고, 28자는 진짜 패딩 채널이다).
/// 공백 채널은 원래 약한 신호라 등급을 낮게 잡는다.
const WS_TRAIL_MIN: usize = 8;
/// 탭·공백이 섞인 내부 열 최소 길이(양쪽 다 있어야 하며, 정렬용 순수 공백 열은 제외한다).
const WS_MIX_MIN: usize = 4;
/// 뒤따르는 공백이 이 길이 이상이면 medium(그 아래는 low) — 길수록 비트 채널 냄새가 짙다.
const WS_TRAIL_MEDIUM: usize = 16;

/// 문자열 하나를 훑어 숨은 마크 신호를 모은다. `only` 가 `Some(k)` 면 그 축만 본다.
///
/// 비용은 문자 수에 선형이다. 발췌는 탐지 1건당 고정 크기 창(±32자)으로 묶여 있다.
pub fn scan_stego(text: &str, only: Option<MarkKind>) -> Vec<StegoFinding> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<StegoFinding> = Vec::new();
    let want = |k: MarkKind| only.is_none() || only == Some(k);

    if want(MarkKind::HiddenChar) {
        scan_hidden(&chars, &mut out);
    }
    if want(MarkKind::Homoglyph) {
        scan_homoglyph(&chars, &mut out);
    }
    if want(MarkKind::Whitespace) {
        scan_whitespace(&chars, &mut out);
    }

    out.sort_by_key(|f| (f.char_offset, f.kind));
    out
}

fn scan_hidden(chars: &[char], out: &mut Vec<StegoFinding>) {
    let n = chars.len();
    let mut i = 0;
    while i < n {
        if is_hidden_char(chars[i] as u32) {
            let mut run = 1;
            while i + run < n && is_hidden_char(chars[i + run] as u32) {
                run += 1;
            }
            if !hidden_run_is_benign(chars, i, run) {
                let slice = &chars[i..i + run];
                let has_tag = slice.iter().any(|&ch| is_tag_char(ch as u32));
                let detail = if has_tag {
                    decode_tags(slice)
                } else {
                    decode_bits(slice)
                };
                let carries_payload = detail
                    .as_deref()
                    .map(|d| d.contains("ASCII"))
                    .unwrap_or(false);
                let severity = if has_tag || carries_payload || run >= 6 {
                    Severity::High
                } else if run >= 2 {
                    Severity::Medium
                } else {
                    Severity::Low
                };
                let mut cps: Vec<u32> = Vec::new();
                for &ch in slice {
                    let c = ch as u32;
                    if !cps.contains(&c) {
                        cps.push(c);
                    }
                }
                out.push(StegoFinding {
                    kind: MarkKind::HiddenChar,
                    severity,
                    char_offset: i,
                    run_length: run,
                    codepoints: cps,
                    excerpt: context_excerpt(chars, i, run, false),
                    detail,
                });
            }
            i += run;
            continue;
        }
        i += 1;
    }
}

fn scan_homoglyph(chars: &[char], out: &mut Vec<StegoFinding>) {
    for at in homoglyph_offsets(chars) {
        let ch = chars[at];
        if let Some(canon) = confusable_to_latin(ch) {
            out.push(StegoFinding {
                kind: MarkKind::Homoglyph,
                severity: Severity::Medium,
                char_offset: at,
                run_length: 1,
                codepoints: vec![ch as u32],
                excerpt: context_excerpt(chars, at, 1, false),
                detail: Some(format!(
                    "{}({}) → {}",
                    ch,
                    format_codepoint(ch as u32),
                    canon
                )),
            });
        }
    }
}

fn scan_whitespace(chars: &[char], out: &mut Vec<StegoFinding>) {
    let n = chars.len();

    // (1) 뒤따르는 공백/탭 — 실제 산문에는 드물고, 비트를 싣는 고전 채널이다.
    let last_visible = chars.iter().rposition(|&c| c != ' ' && c != '\t');
    let trail_start = last_visible.map(|i| i + 1).unwrap_or(0);
    if trail_start < n {
        let run = &chars[trail_start..n];
        let tabs = run.iter().filter(|&&c| c == '\t').count();
        let spaces = run.len() - tabs;
        if run.len() >= WS_TRAIL_MIN {
            let severity = if run.len() >= WS_TRAIL_MEDIUM {
                Severity::Medium
            } else {
                Severity::Low
            };
            out.push(StegoFinding {
                kind: MarkKind::Whitespace,
                severity,
                char_offset: trail_start,
                run_length: run.len(),
                codepoints: whitespace_codepoints(run),
                excerpt: context_excerpt(chars, trail_start, run.len(), true),
                detail: Some(format!(
                    "뒤따르는 공백 {}자 (공백 {spaces} · 탭 {tabs})",
                    run.len()
                )),
            });
        }
    }

    // (2) 내부의 탭·공백 혼합 열 — 순수 공백 열(정렬)은 흔하므로, 탭이 섞인 열만 신고한다.
    let mut i = 0;
    while i < n {
        if (chars[i] == ' ' || chars[i] == '\t') && i >= 1 {
            let start = i;
            while i < n && (chars[i] == ' ' || chars[i] == '\t') {
                i += 1;
            }
            // 열이 문자열 끝까지면 (1)이 이미 다뤘다.
            if i < n {
                let run = &chars[start..i];
                let tabs = run.iter().filter(|&&c| c == '\t').count();
                if run.len() >= WS_MIX_MIN && tabs > 0 && tabs < run.len() {
                    out.push(StegoFinding {
                        kind: MarkKind::Whitespace,
                        severity: Severity::Low,
                        char_offset: start,
                        run_length: run.len(),
                        codepoints: whitespace_codepoints(run),
                        excerpt: context_excerpt(chars, start, run.len(), true),
                        detail: Some(format!("탭·공백이 섞인 열 {}자 (탭 {tabs})", run.len())),
                    });
                }
            }
        } else {
            i += 1;
        }
    }
}

fn whitespace_codepoints(run: &[char]) -> Vec<u32> {
    let mut cps: Vec<u32> = Vec::new();
    for &ch in run {
        let c = ch as u32;
        if !cps.contains(&c) {
            cps.push(c);
        }
    }
    cps.sort_unstable();
    cps
}

// ── 정화(순수 변환 코어) ────────────────────────────────────────────────────

/// [`sanitize_stego`] 결과 — 정화된 텍스트와 무엇을 얼마나 정화했는지.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StegoClean {
    pub text: String,
    /// 지운 비가시 마크 문자 수.
    pub removed_hidden: usize,
    /// 라틴으로 되돌린 동형자 수.
    pub normalized_homoglyphs: usize,
    /// 잘라 낸 뒤따르는 공백/탭 문자 수.
    pub trimmed_whitespace: usize,
}

impl StegoClean {
    /// 무언가 바뀌었는가.
    pub fn changed(&self) -> bool {
        self.removed_hidden + self.normalized_homoglyphs + self.trimmed_whitespace > 0
    }
}

/// 받은 텍스트에서 숨은 마크를 지운다 — **탐지가 신고하는 것만** 지우고 정당한 쓰임은 남긴다.
///
/// - 비가시 마크: 정당한 열(BOM·옛한글 조판·이모지 ZWJ)이 아니면 제거.
/// - 동형자: 라틴 낱말에 섞인 것만 라틴 정규형으로 되돌림(순수 비라틴 인용문은 불변).
/// - 공백: **뒤따르는** 공백/탭만 잘라 냄(내부 정렬 공백은 조판일 수 있어 건드리지 않는다).
///
/// 멱등이다 — `sanitize_stego(sanitize_stego(x).text) == sanitize_stego(x).text`. 탐지와
/// 같은 판정 헬퍼를 쓰므로 정화된 텍스트를 [`scan_stego`] 로 다시 검사하면 hidden·homoglyph 은 0 이다.
pub fn sanitize_stego(text: &str) -> StegoClean {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();

    // 뒤따르는 공백/탭 — 탐지와 같은 조건일 때만 자른다.
    let last_visible = chars.iter().rposition(|&c| c != ' ' && c != '\t');
    let trail_start = last_visible.map(|i| i + 1).unwrap_or(0);
    let trail_run = &chars[trail_start..n];
    let trim_trailing = trail_start < n && trail_run.len() >= WS_TRAIL_MIN;
    let effective_end = if trim_trailing { trail_start } else { n };
    let trimmed_whitespace = n - effective_end;

    let homoglyphs = homoglyph_offsets(&chars);

    let mut out = String::with_capacity(text.len());
    let mut removed_hidden = 0usize;
    let mut normalized_homoglyphs = 0usize;
    let mut idx = 0usize;
    while idx < effective_end {
        let ch = chars[idx];
        let c = ch as u32;
        if is_hidden_char(c) {
            // 탐지와 동일하게 열 단위로 정당성 판정 — 열 전체를 지우거나 열 전체를 남긴다.
            let mut run = 1;
            while idx + run < effective_end && is_hidden_char(chars[idx + run] as u32) {
                run += 1;
            }
            if hidden_run_is_benign(&chars, idx, run) {
                for k in 0..run {
                    out.push(chars[idx + k]);
                }
            } else {
                removed_hidden += run;
            }
            idx += run;
            continue;
        }
        if homoglyphs.contains(&idx) {
            if let Some(canon) = confusable_to_latin(ch) {
                out.push(canon);
                normalized_homoglyphs += 1;
                idx += 1;
                continue;
            }
        }
        out.push(ch);
        idx += 1;
    }

    StegoClean {
        text: out,
        removed_hidden,
        normalized_homoglyphs,
        trimmed_whitespace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(fs: &[StegoFinding]) -> Vec<MarkKind> {
        fs.iter().map(|f| f.kind).collect()
    }

    /// 비트열(0→U+200B, 1→U+200C)로 인코딩한 제로폭 열을 만든다.
    fn zw_bits(bits: &str) -> String {
        bits.chars()
            .map(|b| if b == '0' { '\u{200B}' } else { '\u{200C}' })
            .collect()
    }

    #[test]
    fn zero_width_bit_channel_is_decoded_to_ascii() {
        // "Hi" = 0x48 0x69 = 01001000 01101001
        let payload = zw_bits("0100100001101001");
        let text = format!("문서{payload}끝");
        let fs = scan_stego(&text, None);
        assert_eq!(kinds(&fs), vec![MarkKind::HiddenChar], "{fs:?}");
        let f = &fs[0];
        assert_eq!(f.run_length, 16, "{f:?}");
        assert_eq!(f.severity, Severity::High, "16비트 payload 는 높게: {f:?}");
        assert_eq!(f.char_offset, 2, "'문서' 뒤에서 시작: {f:?}");
        let detail = f.detail.as_deref().unwrap_or("");
        assert!(detail.contains("ASCII \"Hi\""), "복호 실패: {detail}");
        assert!(f.codepoints.contains(&0x200B) && f.codepoints.contains(&0x200C));
    }

    #[test]
    fn tag_characters_are_decoded_and_high() {
        // U+E0041 U+E0042 = 태그 'A' 'B'
        let text = "제목\u{E0041}\u{E0042}본문";
        let fs = scan_stego(text, Some(MarkKind::HiddenChar));
        assert_eq!(fs.len(), 1, "{fs:?}");
        assert_eq!(fs[0].severity, Severity::High);
        assert!(
            fs[0].detail.as_deref().unwrap_or("").contains("AB"),
            "{:?}",
            fs[0].detail
        );
    }

    #[test]
    fn single_stray_zero_width_is_low() {
        let fs = scan_stego("총\u{200B}액", None);
        assert_eq!(kinds(&fs), vec![MarkKind::HiddenChar], "{fs:?}");
        assert_eq!(fs[0].run_length, 1);
        assert_eq!(fs[0].severity, Severity::Low);
    }

    #[test]
    fn homoglyph_in_latin_word_is_flagged_and_normalized() {
        // 키릴 Т(U+0422) + 라틴 otal
        let fs = scan_stego("\u{0422}otal 보고서", None);
        assert_eq!(kinds(&fs), vec![MarkKind::Homoglyph], "{fs:?}");
        assert_eq!(fs[0].char_offset, 0);
        assert_eq!(fs[0].codepoints, vec![0x0422]);
        assert!(fs[0].detail.as_deref().unwrap_or("").contains("→ T"));

        let cleaned = sanitize_stego("\u{0422}otal 보고서");
        assert_eq!(cleaned.text, "Total 보고서");
        assert_eq!(cleaned.normalized_homoglyphs, 1);
    }

    #[test]
    fn pure_cyrillic_word_is_not_a_homoglyph() {
        // 정당한 러시아어 — 라틴이 섞이지 않았으니 위장이 아니다.
        let fs = scan_stego("Москва 회의록", None);
        assert!(fs.is_empty(), "{fs:?}");
        assert!(!sanitize_stego("Москва 회의록").changed());
    }

    #[test]
    fn trailing_whitespace_is_flagged_and_trimmed() {
        let fs = scan_stego("합계        ", None); // 공백 8
        assert_eq!(kinds(&fs), vec![MarkKind::Whitespace], "{fs:?}");
        assert_eq!(fs[0].run_length, 8);
        assert_eq!(fs[0].severity, Severity::Low, "8자는 낮게: {fs:?}");

        let cleaned = sanitize_stego("합계        ");
        assert_eq!(cleaned.text, "합계");
        assert_eq!(cleaned.trimmed_whitespace, 8);

        // 아주 긴 뒤공백은 패딩 채널 냄새가 짙다 — medium.
        let long = scan_stego(&format!("끝{}", " ".repeat(20)), None);
        assert_eq!(long[0].severity, Severity::Medium, "{long:?}");
    }

    #[test]
    fn short_trailing_whitespace_and_tab_are_not_flagged() {
        // 실측(samples 45건 스윕): 실제 HWP 문서는 문단 끝에 정렬용 공백/탭을 4–7자 흔히
        // 둔다 — 8자 미만은 잡지 않아 오탐을 억제한다.
        assert!(scan_stego("문장 ", None).is_empty());
        assert!(
            scan_stego("항목\t", None).is_empty(),
            "트레일링 탭 1자 오탐"
        );
        assert!(
            scan_stego("정렬       ", None).is_empty(),
            "트레일링 공백 7자 오탐"
        );
        assert!(!sanitize_stego("정렬       ").changed());
    }

    #[test]
    fn mixed_tab_space_interior_run_is_flagged() {
        let fs = scan_stego("A\t \t B", Some(MarkKind::Whitespace));
        assert_eq!(fs.len(), 1, "{fs:?}");
        assert_eq!(fs[0].kind, MarkKind::Whitespace);
        assert!(fs[0].run_length >= WS_MIX_MIN);
    }

    #[test]
    fn legitimate_uses_are_never_touched() {
        // 맨 앞 BOM.
        assert!(scan_stego("\u{FEFF}보고서 본문", None).is_empty());
        assert_eq!(
            sanitize_stego("\u{FEFF}보고서 본문").text,
            "\u{FEFF}보고서 본문"
        );
        // 이모지 ZWJ 결합.
        let emoji = "\u{1F468}\u{200D}\u{1F469}";
        assert!(scan_stego(emoji, None).is_empty(), "이모지 ZWJ 오탐");
        assert_eq!(sanitize_stego(emoji).text, emoji);
        // PUA 옛한글 조판 곁의 제로폭.
        assert!(scan_stego("\u{F152}\u{200B}가나", None).is_empty());
        assert_eq!(
            sanitize_stego("\u{F152}\u{200B}가나").text,
            "\u{F152}\u{200B}가나"
        );
        // 평범한 한국어.
        assert!(scan_stego("정상 문서입니다.", None).is_empty());
        assert!(!sanitize_stego("정상 문서입니다.").changed());
    }

    #[test]
    fn sanitize_removes_planted_marks_and_rescan_is_clean() {
        let payload = zw_bits("0100100001101001"); // "Hi"
        let text = format!("\u{0422}otal{payload} 결과        "); // 동형자 + 제로폭 + 뒤 공백 8
        let cleaned = sanitize_stego(&text);
        assert_eq!(cleaned.text, "Total 결과");
        assert_eq!(cleaned.removed_hidden, 16);
        assert_eq!(cleaned.normalized_homoglyphs, 1);
        assert_eq!(cleaned.trimmed_whitespace, 8);
        // 재검사: 숨은 마크 0.
        assert!(
            scan_stego(&cleaned.text, None).is_empty(),
            "정화 후에도 신호가 남음"
        );
    }

    #[test]
    fn sanitize_is_idempotent() {
        let payload = zw_bits("0100100001101001");
        let text = format!("\u{0422}otal{payload} 결과   \t");
        let once = sanitize_stego(&text);
        let twice = sanitize_stego(&once.text);
        assert_eq!(once.text, twice.text);
        assert!(
            !twice.changed(),
            "두 번째 정화는 아무것도 바꾸지 않아야 한다"
        );
    }

    #[test]
    fn kind_filter_isolates_axis() {
        let payload = zw_bits("0100100001101001");
        let text = format!("\u{0422}otal{payload} 결과   ");
        assert!(scan_stego(&text, Some(MarkKind::HiddenChar))
            .iter()
            .all(|f| f.kind == MarkKind::HiddenChar));
        assert!(scan_stego(&text, Some(MarkKind::Homoglyph))
            .iter()
            .all(|f| f.kind == MarkKind::Homoglyph));
        assert!(scan_stego(&text, Some(MarkKind::Whitespace))
            .iter()
            .all(|f| f.kind == MarkKind::Whitespace));
    }

    #[test]
    fn from_filter_roundtrips() {
        for k in MarkKind::ALL {
            assert_eq!(MarkKind::from_filter(k.filter_name()), Some(k));
        }
        assert_eq!(MarkKind::from_filter("all"), None);
        assert_eq!(MarkKind::from_filter("bogus"), None);
    }
}
