//! 에이전트 대상 유니코드 기만 탐지 — **보고만 하고 절대 변형하지 않는다**.
//!
//! rhwp 의 `--json` 봉투와 MCP 도구 결과는 LLM 에이전트가 "검증된 도구 출력"으로
//! 읽는다. 그런데 그 안에 담기는 누름틀 이름·값·본문 텍스트는 전부 **공격자가
//! 내용을 정할 수 있는 문서**에서 온다(민원인이 올린 서식, 웹에서 받은 문서).
//! 이 모듈은 그 경계에서 기만 신호를 탐지해 봉투에 표시한다.
//!
//! 탐지 대상 3축:
//!
//! - **혼합 스크립트**(`MixedScript`) — 한 낱말에 라틴·키릴·그리스가 섞였다.
//!   `Тotal`(키릴 Т) 처럼 화면상 라틴과 구별되지 않는 이름을 만든다.
//! - **혼동 충돌**(`ConfusableCollision`) — 같은 문서 안에 골격(skeleton)이 같은
//!   서로 다른 이름이 둘 이상 있다. 이것이 실제 공격 서명이다: 에이전트가
//!   `Total` 을 채우면 사람이 보는 칸은 `Тotal` 인 채로 남는다.
//! - **보이지 않는 문자·방향 제어**(`BidiControl`/`InvisibleChar`/`AnsiEscape`) —
//!   Trojan Source(CVE-2021-42574) 계열 방향 오버라이드, 제로폭 문자, 터미널
//!   이스케이프. 화면 표시와 실제 바이트가 어긋나게 만든다.
//!
//! ## 왜 변형하지 않는가
//!
//! rhwp 는 문서 엔진이다. 사용자 문서의 글자를 조용히 바꾸는 것은 어떤 보안
//! 이득으로도 정당화되지 않는다 — 키릴로 쓰인 정당한 러시아어 인용문을 라틴으로
//! 고쳐 저장하는 순간 그 문서는 손상된 것이다. 그래서 이 모듈의 모든 함수는
//! `&str` 을 받아 **판정만** 돌려준다. 정화(sanitize)는 하지 않는다.
//!
//! ## 왜 의존성을 더하지 않는가
//!
//! 판정 범위가 좁다(혼동 가능한 스크립트는 라틴·키릴·그리스 3종). UTS #39 전체
//! 혼동 표는 수만 항목이고 WASM 산출물 크기에 그대로 얹힌다. 실제 공격에 쓰이는
//! 고빈도 동형자만 담은 아래 표로 같은 방어력을 얻는다.

/// 탐지된 위험 1건.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRisk {
    pub kind: RiskKind,
    /// 문제가 된 코드포인트(중복 제거·오름차순). 봉투에 `U+04XX` 형태로 싣는다.
    pub codepoints: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskKind {
    /// 방향 오버라이드·임베딩·격리 (U+202A~U+202E, U+2066~U+2069).
    BidiControl,
    /// 제로폭·보이지 않는 문자 (U+200B~U+200F, U+2060, U+FEFF, U+00AD 등).
    InvisibleChar,
    /// 터미널 이스케이프 시작 (U+001B) — CLI 출력을 보는 사람을 속인다.
    AnsiEscape,
    /// 한 낱말에 라틴·키릴·그리스가 섞였다.
    MixedScript,
}

impl RiskKind {
    /// 봉투용 안정 식별자 — 소비자가 문자열로 분기한다.
    pub fn label(self) -> &'static str {
        match self {
            RiskKind::BidiControl => "bidiControl",
            RiskKind::InvisibleChar => "invisibleChar",
            RiskKind::AnsiEscape => "ansiEscape",
            RiskKind::MixedScript => "mixedScript",
        }
    }

    /// 사람이 읽는 한 줄 설명 — 에이전트가 그대로 사용자에게 전달할 수 있다.
    pub fn describe(self) -> &'static str {
        match self {
            RiskKind::BidiControl => {
                "방향 오버라이드 문자가 있습니다 — 화면에 보이는 순서와 실제 문자 순서가 다를 수 있습니다"
            }
            RiskKind::InvisibleChar => {
                "보이지 않는 문자가 있습니다 — 화면에 나타나지 않는 내용이 값에 포함돼 있습니다"
            }
            RiskKind::AnsiEscape => {
                "터미널 이스케이프 문자가 있습니다 — 콘솔 출력을 조작할 수 있습니다"
            }
            RiskKind::MixedScript => {
                "한 낱말에 라틴·키릴·그리스 문자가 섞여 있습니다 — 다른 이름과 화면상 구별되지 않을 수 있습니다"
            }
        }
    }
}

/// 혼동 가능한(=동형자를 가진) 스크립트. 한글·한자·숫자·문장부호는 라틴과
/// 헷갈릴 일이 없으므로 판정에서 제외한다 — 한국어 문서가 라틴을 섞는 것은
/// 지극히 정상이라, 이들을 세면 오탐만 쏟아진다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfusableScript {
    Latin,
    Cyrillic,
    Greek,
}

fn script_of(ch: char) -> Option<ConfusableScript> {
    let c = ch as u32;
    match c {
        // 라틴: ASCII 문자 + Latin-1 Supplement/Extended-A·B 의 문자 영역
        0x41..=0x5A | 0x61..=0x7A => Some(ConfusableScript::Latin),
        0xC0..=0xFF if c != 0xD7 && c != 0xF7 => Some(ConfusableScript::Latin),
        0x100..=0x24F => Some(ConfusableScript::Latin),
        // 그리스·콥트 (+ 확장)
        0x370..=0x3FF | 0x1F00..=0x1FFF => Some(ConfusableScript::Greek),
        // 키릴 (+ 보충·확장)
        0x400..=0x52F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F => Some(ConfusableScript::Cyrillic),
        _ => None,
    }
}

/// 고빈도 동형자 → 라틴 정규형. 실제 스푸핑에 쓰이는 글자만 담는다.
///
/// 출처 원칙: 키릴·그리스에서 라틴 글리프와 **사실상 동일하게 렌더되는** 글자.
/// 목록을 넓히는 것보다 정확히 유지하는 편이 오탐을 막는다.
fn confusable_to_latin(ch: char) -> Option<char> {
    Some(match ch {
        // 키릴 소문자
        'а' => 'a',
        'в' => 'b',
        'с' => 'c',
        'е' => 'e',
        'ѕ' => 's',
        'һ' => 'h',
        'і' => 'i',
        'ј' => 'j',
        'к' => 'k',
        'м' => 'm',
        'н' => 'h',
        'о' => 'o',
        'р' => 'p',
        'т' => 't',
        'у' => 'y',
        'х' => 'x',
        'ч' => 'y',
        'ԁ' => 'd',
        'ԛ' => 'q',
        'ԝ' => 'w',
        'ա' => 'w',
        // 키릴 대문자
        'А' => 'A',
        'В' => 'B',
        'Е' => 'E',
        'Ѕ' => 'S',
        'І' => 'I',
        'Ј' => 'J',
        'К' => 'K',
        'М' => 'M',
        'Н' => 'H',
        'О' => 'O',
        'Р' => 'P',
        'С' => 'C',
        'Т' => 'T',
        'У' => 'Y',
        'Х' => 'X',
        'Ԁ' => 'D',
        'Ԛ' => 'Q',
        'Ԝ' => 'W',
        'Ғ' => 'F',
        'Ԍ' => 'G',
        // 그리스 소문자
        'α' => 'a',
        'ο' => 'o',
        'ρ' => 'p',
        'ν' => 'v',
        'υ' => 'u',
        'κ' => 'k',
        'ι' => 'i',
        'τ' => 't',
        // 그리스 대문자
        'Α' => 'A',
        'Β' => 'B',
        'Ε' => 'E',
        'Ζ' => 'Z',
        'Η' => 'H',
        'Ι' => 'I',
        'Κ' => 'K',
        'Μ' => 'M',
        'Ν' => 'N',
        'Ο' => 'O',
        'Ρ' => 'P',
        'Τ' => 'T',
        'Υ' => 'Y',
        'Χ' => 'X',
        _ => return None,
    })
}

fn is_bidi_control(c: u32) -> bool {
    // LRE RLE PDF LRO RLO / LRI RLI FSI PDI — Trojan Source 계열.
    (0x202A..=0x202E).contains(&c) || (0x2066..=0x2069).contains(&c)
}

fn is_invisible(c: u32) -> bool {
    matches!(
        c,
        0x00AD          // SOFT HYPHEN
            | 0x061C     // ARABIC LETTER MARK
            | 0x180E     // MONGOLIAN VOWEL SEPARATOR
            | 0x200B..=0x200F // ZWSP ZWNJ ZWJ LRM RLM
            | 0x2060..=0x2064 // WJ, invisible operators
            | 0xFEFF     // BOM / ZWNBSP
    )
}

/// 문자열 하나를 훑어 보이지 않는 문자·방향 제어·터미널 이스케이프를 찾는다.
/// 본문 텍스트·필드 값처럼 **자유 서술 문자열**에 쓴다(혼합 스크립트는 보지 않는다 —
/// 한국어 문서가 러시아어 인용을 담는 것은 정상이다).
pub fn scan_text(s: &str) -> Vec<TextRisk> {
    let mut bidi: Vec<u32> = Vec::new();
    let mut invis: Vec<u32> = Vec::new();
    let mut ansi: Vec<u32> = Vec::new();
    for ch in s.chars() {
        let c = ch as u32;
        if is_bidi_control(c) {
            push_unique(&mut bidi, c);
        } else if is_invisible(c) {
            push_unique(&mut invis, c);
        } else if c == 0x1B {
            push_unique(&mut ansi, c);
        }
    }
    let mut out = Vec::new();
    for (kind, cps) in [
        (RiskKind::BidiControl, bidi),
        (RiskKind::InvisibleChar, invis),
        (RiskKind::AnsiEscape, ansi),
    ] {
        if !cps.is_empty() {
            out.push(TextRisk {
                kind,
                codepoints: cps,
            });
        }
    }
    out
}

/// 이름처럼 **에이전트가 지목에 쓰는 문자열**을 훑는다 — `scan_text` 에 더해
/// 혼합 스크립트까지 본다. 누름틀 이름·표 머리글처럼 "이걸 채워줘"의 대상이
/// 되는 값이 여기 해당한다.
pub fn scan_identifier(s: &str) -> Vec<TextRisk> {
    let mut out = scan_text(s);
    let mut scripts: Vec<ConfusableScript> = Vec::new();
    let mut offenders: Vec<u32> = Vec::new();
    for ch in s.chars() {
        if let Some(sc) = script_of(ch) {
            if !scripts.contains(&sc) {
                scripts.push(sc);
            }
        }
    }
    if scripts.len() > 1 {
        // 소수파 스크립트의 글자를 지목한다 — 보통 그쪽이 심어진 쪽이다.
        let mut counts = [(ConfusableScript::Latin, 0usize)].to_vec();
        counts.clear();
        for sc in &scripts {
            let n = s.chars().filter(|c| script_of(*c) == Some(*sc)).count();
            counts.push((*sc, n));
        }
        let min = counts.iter().map(|(_, n)| *n).min().unwrap_or(0);
        for ch in s.chars() {
            if let Some(sc) = script_of(ch) {
                if counts.iter().any(|(s2, n)| *s2 == sc && *n == min) {
                    push_unique(&mut offenders, ch as u32);
                }
            }
        }
        out.push(TextRisk {
            kind: RiskKind::MixedScript,
            codepoints: offenders,
        });
    }
    out
}

/// 한글 조합형(NFD) 자모 나열을 완성형(NFC) 음절로 접는다.
///
/// **한국어 문서 엔진에서 가장 현실적인 쌍둥이 벡터가 바로 이것이다.** `총액` 을
/// 완성형(U+CD1D U+C561)으로 쓴 필드와 조합형(ᄎ ᅩ ᆼ ᄋ ᅢ ᆨ)으로 쓴 필드는 화면상
/// 완전히 같지만 바이트가 다르다 — 키릴 동형자처럼 낯선 글자를 심을 필요조차 없고,
/// macOS 파일시스템과 일부 한글 IME 가 자연스럽게 만들어 내는 형태라 "수상한 문서"로
/// 보이지도 않는다.
///
/// 한글 음절 조합은 표가 아니라 **산술**이다(Unicode 3.12 Hangul Syllable
/// Composition) — 그래서 정규화 크레이트 없이 정확히 접을 수 있다:
/// `S = (L-0x1100)*588 + (V-0x1161)*28 + (T-0x11A7) + 0xAC00`
fn compose_hangul(chars: &[char]) -> Vec<char> {
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const S_BASE: u32 = 0xAC00;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;

    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let l = chars[i] as u32;
        let li = l.wrapping_sub(L_BASE);
        if li < L_COUNT && i + 1 < chars.len() {
            let vi = (chars[i + 1] as u32).wrapping_sub(V_BASE);
            if vi < V_COUNT {
                // 종성은 선택적 — 있으면 먹고, 없으면 초·중성만 합친다.
                let mut ti = 0;
                let mut consumed = 2;
                if i + 2 < chars.len() {
                    let t = (chars[i + 2] as u32).wrapping_sub(T_BASE);
                    if t > 0 && t < T_COUNT {
                        ti = t;
                        consumed = 3;
                    }
                }
                let s = S_BASE + (li * V_COUNT + vi) * T_COUNT + ti;
                if let Some(ch) = char::from_u32(s) {
                    out.push(ch);
                    i += consumed;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// 혼동 골격 — 보이지 않는 문자를 걷어내고, 한글 조합형을 완성형으로 합치고,
/// 동형자를 라틴 정규형으로 접고, 대소문자를 없앤 형태.
/// 두 이름의 골격이 같으면 화면상 구별이 사실상 불가능하다.
pub fn confusable_skeleton(s: &str) -> String {
    let stripped: Vec<char> = s
        .chars()
        .filter(|c| {
            let u = *c as u32;
            !is_bidi_control(u) && !is_invisible(u)
        })
        .collect();
    compose_hangul(&stripped)
        .into_iter()
        .map(|c| confusable_to_latin(c).unwrap_or(c))
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// 이름 목록 안에서 **골격이 같은 서로 다른 이름** 무리를 찾는다.
///
/// 이것이 실제 공격 서명이다 — 한 문서에 `Total`(라틴)과 `Тotal`(키릴)이 함께
/// 있는 정상 문서는 사실상 없다. 반환은 `(골격, 그 골격을 공유하는 원본 이름들)`
/// 이고, 원본 이름이 2종 이상인 무리만 담는다(같은 이름의 단순 반복은 제외 —
/// 그건 기존 `ambiguous` 판정이 이미 다룬다).
pub fn confusable_collisions(names: &[String]) -> Vec<(String, Vec<String>)> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for name in names {
        let skel = confusable_skeleton(name);
        if skel.is_empty() {
            continue;
        }
        match groups.iter_mut().find(|(s, _)| *s == skel) {
            Some((_, members)) => {
                if !members.iter().any(|m| m == name) {
                    members.push(name.clone());
                }
            }
            None => groups.push((skel, vec![name.clone()])),
        }
    }
    groups.retain(|(_, members)| members.len() > 1);
    groups
}

fn push_unique(v: &mut Vec<u32>, c: u32) {
    if !v.contains(&c) {
        v.push(c);
    }
}

/// `U+0422` 형태 표기 — 봉투와 오류 메시지가 같은 어휘를 쓰게 한다.
pub fn format_codepoint(c: u32) -> String {
    format!("U+{c:04X}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_scripts_are_not_flagged() {
        // 한국어 문서가 라틴·한자·숫자를 섞는 것은 정상이다.
        assert!(scan_identifier("회사명").is_empty());
        assert!(scan_identifier("Total").is_empty());
        assert!(scan_identifier("2026년 Q3 보고서 v2").is_empty());
        assert!(scan_identifier("株式會社").is_empty());
        // 순수 키릴(정당한 러시아어)도 단일 스크립트라 통과한다.
        assert!(scan_identifier("Москва").is_empty());
        // 순수 그리스(수식 기호 이름)도 마찬가지.
        assert!(scan_identifier("αβγ").is_empty());
    }

    #[test]
    fn mixed_script_name_is_flagged() {
        let risks = scan_identifier("Тotal"); // 키릴 Т + 라틴 otal
        assert_eq!(risks.len(), 1, "{risks:?}");
        assert_eq!(risks[0].kind, RiskKind::MixedScript);
        assert!(
            risks[0].codepoints.contains(&0x0422),
            "심어진 키릴 Т 를 지목해야 한다: {risks:?}"
        );
    }

    #[test]
    fn bidi_and_invisible_in_free_text() {
        let risks = scan_text("Accounting \u{202E}txet\u{202C} \u{200B}hidden");
        let kinds: Vec<RiskKind> = risks.iter().map(|r| r.kind).collect();
        assert!(kinds.contains(&RiskKind::BidiControl), "{risks:?}");
        assert!(kinds.contains(&RiskKind::InvisibleChar), "{risks:?}");
        // 자유 서술 문자열에서는 혼합 스크립트를 보지 않는다.
        assert!(!kinds.contains(&RiskKind::MixedScript), "{risks:?}");
    }

    #[test]
    fn ansi_escape_is_flagged() {
        let risks = scan_text("정상\u{1B}[2J지움");
        assert_eq!(risks[0].kind, RiskKind::AnsiEscape, "{risks:?}");
    }

    #[test]
    fn skeleton_folds_confusables() {
        assert_eq!(confusable_skeleton("Тotal"), confusable_skeleton("Total"));
        assert_eq!(confusable_skeleton("Тоtаl"), confusable_skeleton("Total"));
        // 보이지 않는 문자로 골격을 흐리는 우회도 접는다.
        assert_eq!(
            confusable_skeleton("To\u{200B}tal"),
            confusable_skeleton("Total")
        );
        // 서로 다른 낱말은 접히지 않는다.
        assert_ne!(confusable_skeleton("Total"), confusable_skeleton("Tota"));
        assert_ne!(confusable_skeleton("회사명"), confusable_skeleton("작성자"));
    }

    #[test]
    fn hangul_nfd_and_nfc_share_a_skeleton() {
        // 완성형 '총액' vs 조합형(총 액) — 화면상 동일, 바이트는 다르다.
        let nfc = "총액";
        let nfd = "\u{110E}\u{1169}\u{11BC}\u{110B}\u{1162}\u{11A8}";
        assert_ne!(nfc, nfd, "전제: 두 문자열의 바이트는 달라야 한다");
        assert_eq!(
            confusable_skeleton(nfc),
            confusable_skeleton(nfd),
            "조합형·완성형 한글이 같은 골격으로 접혀야 한다"
        );
        // 종성 없는 음절도 접힌다.
        assert_eq!(
            confusable_skeleton("가"),
            confusable_skeleton("\u{1100}\u{1161}")
        );
        // 서로 다른 한글은 접히지 않는다.
        assert_ne!(confusable_skeleton("총액"), confusable_skeleton("총역"));
        assert_ne!(confusable_skeleton("합계"), confusable_skeleton("합게"));
    }

    #[test]
    fn hangul_collision_is_reported() {
        let names: Vec<String> = [
            "총액",
            "\u{110E}\u{1169}\u{11BC}\u{110B}\u{1162}\u{11A8}",
            "비고",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let cols = confusable_collisions(&names);
        assert_eq!(cols.len(), 1, "한글 NFC/NFD 쌍둥이를 잡아야 한다: {cols:?}");
        assert_eq!(cols[0].1.len(), 2, "{cols:?}");
    }

    #[test]
    fn collisions_report_only_cross_script_twins() {
        let names: Vec<String> = ["Total", "Тotal", "부서명", "목차1", "목차1"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let cols = confusable_collisions(&names);
        assert_eq!(cols.len(), 1, "쌍둥이 무리 1개여야 한다: {cols:?}");
        assert_eq!(cols[0].1.len(), 2, "{cols:?}");
        // 같은 이름의 단순 반복(목차1 ×2)은 기존 ambiguous 판정의 몫이다.
        assert!(
            !cols.iter().any(|(_, m)| m.iter().any(|n| n == "목차1")),
            "{cols:?}"
        );
    }

    #[test]
    fn all_hangul_document_is_quiet() {
        // 실제 한국 서식의 전형적 이름들 — 단 한 건도 경고가 나오면 안 된다.
        let names = [
            "회사명",
            "작성자",
            "부서명",
            "전화번호",
            "이메일",
            "제목",
            "목차1",
            "합계",
            "비고",
            "2026-08-01",
            "E-mail",
            "URL",
        ];
        for n in names {
            assert!(scan_identifier(n).is_empty(), "오탐: {n}");
        }
        let owned: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        assert!(confusable_collisions(&owned).is_empty());
    }
}
