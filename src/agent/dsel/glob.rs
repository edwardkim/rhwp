//! 글롭 일치기 — `~=` 와 `:matches(…)` 의 판정기.
//!
//! ## 왜 정규식이 아닌가
//!
//! 두 가지다.
//!
//! 1. **의존성.** rhwp 는 정규식 크레이트를 쓰지 않는다. 선택자 하나 때문에
//!    의존성을 늘리면 wasm 크기와 감사 표면이 함께 늘어난다.
//! 2. **정지성.** 역추적 정규식은 입력에 따라 지수 시간으로 터진다. 선택자는
//!    **문서에서 온 값**과 맞대어지는데, 문서는 신뢰 경계 바깥이다
//!    (`provenance::MAP` 이 같은 말을 한다). 신뢰할 수 없는 입력에 지수 시간
//!    판정기를 붙이는 것은 DoS 를 스스로 심는 것이다.
//!
//! 그래서 문법을 글롭으로 좁히고, **역추적 지점이 `*` 하나뿐**인 고전 알고리즘을
//! 쓴다. 최악 시간은 `O(패턴 × 입력)` 로 유계이며 지수 경로가 존재하지 않는다.
//!
//! ## 문법
//!
//! | 표기 | 뜻 |
//! | --- | --- |
//! | `*` | 임의 길이(0 포함) |
//! | `?` | 임의의 한 글자 |
//! | `[abc]` | 나열된 글자 중 하나 |
//! | `[a-z]` | 범위 안의 한 글자 |
//! | `[!abc]` / `[^abc]` | 나열되지 **않은** 한 글자 |
//! | `\x` | `x` 를 글자 그대로 |
//!
//! 글자 단위는 `char` 다 — UTF-8 바이트가 아니다. `?` 하나가 한글 한 글자에
//! 대응해야 사람이 쓴 패턴이 예상대로 동작한다.

use super::error::SelectorError;

/// 컴파일된 글롭 패턴.
///
/// 매번 문자열을 다시 훑지 않으려고 조각으로 미리 쪼갠다. 같은 선택자를 여러
/// 노드에 대해 평가하므로 컴파일 한 번 : 판정 N 번의 비율이 된다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glob {
    segs: Vec<Seg>,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Seg {
    /// `*`
    Star,
    /// `?`
    One,
    /// 글자 하나 그대로.
    Lit(char),
    /// 글자 집합.
    Class {
        negated: bool,
        items: Vec<ClassItem>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClassItem {
    Single(char),
    Range(char, char),
}

impl ClassItem {
    fn contains(&self, c: char) -> bool {
        match self {
            ClassItem::Single(x) => *x == c,
            ClassItem::Range(lo, hi) => *lo <= c && c <= *hi,
        }
    }
}

impl Glob {
    /// 패턴을 컴파일한다.
    ///
    /// `offset` 은 원문에서 패턴 문자열이 시작한 위치 — 오류 오프셋을 선택자
    /// 전체 기준으로 돌려주려고 받는다. 패턴 내부 기준으로 돌려주면 캐럿이
    /// 엉뚱한 곳에 선다.
    pub fn compile(pattern: &str, offset: usize) -> Result<Glob, SelectorError> {
        let chars: Vec<char> = pattern.chars().collect();
        let mut segs = Vec::new();
        let mut i = 0usize;

        while i < chars.len() {
            match chars[i] {
                '*' => {
                    // 연속 `*` 는 하나와 같다. 접어 두지 않으면 `***` 가 역추적
                    // 지점을 셋 만들어 최악 시간이 패턴 길이에 곱해진다.
                    if segs.last() != Some(&Seg::Star) {
                        segs.push(Seg::Star);
                    }
                    i += 1;
                }
                '?' => {
                    segs.push(Seg::One);
                    i += 1;
                }
                '\\' => {
                    let c = chars.get(i + 1).copied().ok_or_else(|| {
                        SelectorError::resolve(offset + i, "역슬래시로 끝나는 글롭 패턴")
                            .hinting("역슬래시 자체는 `\\\\` 로 적는다")
                    })?;
                    segs.push(Seg::Lit(c));
                    i += 2;
                }
                '[' => {
                    let (seg, next) = compile_class(&chars, i, offset)?;
                    segs.push(seg);
                    i = next;
                }
                ']' => {
                    return Err(SelectorError::resolve(offset + i, "짝 없는 `]`")
                        .hinting("글자 그대로 쓰려면 `\\]` 로 적는다"));
                }
                c => {
                    segs.push(Seg::Lit(c));
                    i += 1;
                }
            }
        }

        Ok(Glob {
            segs,
            source: pattern.to_string(),
        })
    }

    /// 문자열 전체가 패턴에 맞는가.
    ///
    /// 부분 일치가 아니라 **전체 일치**다. 부분 일치가 필요하면 패턴 양끝에 `*` 를
    /// 붙인다 — 기본을 부분 일치로 두면 `~="합계"` 가 "합계표"에도 맞아서, 정확히
    /// 맞는 것만 고르려면 매번 앵커를 적어야 한다. 정확 일치가 더 흔한 의도다.
    pub fn is_match(&self, text: &str) -> bool {
        let input: Vec<char> = text.chars().collect();
        self.match_chars(&input)
    }

    /// 원문 패턴.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// 이 패턴이 모든 문자열에 맞는가 (`*`, `**` …).
    ///
    /// 평가기가 술어를 통째로 건너뛸 수 있는지 판단하는 데 쓴다.
    pub fn is_universal(&self) -> bool {
        self.segs.iter().all(|s| matches!(s, Seg::Star))
    }

    /// 고전 선형 글롭 일치 — 역추적 지점은 마지막 `*` 하나뿐.
    ///
    /// `star` 는 마지막으로 본 `*` 의 세그먼트 위치, `mark` 는 그때 입력 위치다.
    /// 불일치가 나면 그 `*` 가 한 글자 더 먹은 것으로 치고 재개한다. 재귀가 없으므로
    /// 스택 오버플로 경로도 없다 — 파서와 달리 여기는 깊이 상한이 필요 없다.
    fn match_chars(&self, input: &[char]) -> bool {
        let mut i = 0usize; // 입력 위치
        let mut j = 0usize; // 세그먼트 위치
        let mut star: Option<usize> = None;
        let mut mark = 0usize;

        while i < input.len() {
            match self.segs.get(j) {
                Some(Seg::Star) => {
                    star = Some(j);
                    mark = i;
                    j += 1;
                }
                Some(seg) if seg_matches(seg, input[i]) => {
                    i += 1;
                    j += 1;
                }
                _ => match star {
                    Some(s) => {
                        // `*` 가 한 글자 더 먹는다.
                        j = s + 1;
                        mark += 1;
                        i = mark;
                    }
                    None => return false,
                },
            }
        }

        // 남은 세그먼트는 전부 `*` 여야 한다.
        self.segs[j.min(self.segs.len())..]
            .iter()
            .all(|s| matches!(s, Seg::Star))
    }
}

fn seg_matches(seg: &Seg, c: char) -> bool {
    match seg {
        Seg::One => true,
        Seg::Lit(x) => *x == c,
        Seg::Class { negated, items } => {
            let hit = items.iter().any(|it| it.contains(c));
            hit != *negated
        }
        Seg::Star => unreachable!("호출부가 Star 를 먼저 처리한다"),
    }
}

/// `[` 에서 시작하는 글자 집합을 컴파일한다.
fn compile_class(
    chars: &[char],
    start: usize,
    offset: usize,
) -> Result<(Seg, usize), SelectorError> {
    let mut i = start + 1;
    let negated = matches!(chars.get(i), Some('!') | Some('^'));
    if negated {
        i += 1;
    }

    let mut items = Vec::new();
    // 첫 글자가 `]` 면 글자 그대로 — POSIX 관습이고, 이게 없으면 `]` 를 집합에
    // 넣을 방법이 이스케이프뿐이 된다.
    let mut first = true;

    while i < chars.len() {
        let c = chars[i];
        // `first` 가 거짓이면 앞선 반복이 반드시 항목 하나를 밀어 넣었으므로
        // 여기서 `items` 가 비는 경우는 없다 — 빈 집합 분기를 따로 두지 않는 이유다.
        if c == ']' && !first {
            debug_assert!(!items.is_empty(), "첫 글자가 아닌 `]` 앞에는 항목이 있다");
            return Ok((Seg::Class { negated, items }, i + 1));
        }
        first = false;

        let lo = if c == '\\' {
            i += 1;
            *chars
                .get(i)
                .ok_or_else(|| SelectorError::resolve(offset + i, "역슬래시로 끝나는 글자 집합"))?
        } else {
            c
        };

        // 범위인가 — `a-z`. 뒤가 `]` 면 `-` 는 글자 그대로다.
        if chars.get(i + 1) == Some(&'-') && chars.get(i + 2).is_some_and(|c| *c != ']') {
            let mut k = i + 2;
            let hi = if chars[k] == '\\' {
                k += 1;
                *chars.get(k).ok_or_else(|| {
                    SelectorError::resolve(offset + k, "역슬래시로 끝나는 글자 집합")
                })?
            } else {
                chars[k]
            };
            if hi < lo {
                return Err(SelectorError::resolve(
                    offset + i,
                    format!("뒤집힌 글자 범위 `{lo}-{hi}`"),
                )
                .hinting("작은 글자를 앞에 적는다"));
            }
            items.push(ClassItem::Range(lo, hi));
            i = k + 1;
            continue;
        }

        items.push(ClassItem::Single(lo));
        i += 1;
    }

    Err(
        SelectorError::resolve(offset + start, "닫히지 않은 글자 집합 `[`")
            .hinting("대괄호를 글자로 쓰려면 `\\[` 로 적는다"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(pattern: &str, text: &str) -> bool {
        Glob::compile(pattern, 0).unwrap().is_match(text)
    }

    #[test]
    fn literal_match_is_whole_string() {
        assert!(m("합계", "합계"));
        // 부분 일치가 아니다 — 앵커 없이 접두만 맞는 것은 불일치.
        assert!(!m("합계", "합계표"));
        assert!(m("합계*", "합계표"));
    }

    #[test]
    fn question_mark_counts_chars_not_bytes() {
        // 한글 한 글자는 UTF-8 3바이트. `?` 하나로 맞아야 한다.
        assert!(m("?계", "합계"));
        assert!(!m("??계", "합계"));
    }

    #[test]
    fn star_matches_empty() {
        assert!(m("*", ""));
        assert!(m("a*b", "ab"));
    }

    #[test]
    fn consecutive_stars_collapse() {
        let g = Glob::compile("a***b", 0).unwrap();
        assert_eq!(g.segs.iter().filter(|s| **s == Seg::Star).count(), 1);
        assert!(g.is_match("axxxb"));
    }

    #[test]
    fn classes_ranges_and_negation() {
        assert!(m("[abc]", "b"));
        assert!(!m("[abc]", "d"));
        assert!(m("[a-z]", "q"));
        assert!(!m("[a-z]", "Q"));
        assert!(m("[!abc]", "d"));
        assert!(!m("[!abc]", "a"));
        assert!(m("[^abc]", "d"));
    }

    #[test]
    fn class_can_contain_closing_bracket_first() {
        assert!(m("[]a]", "]"));
        assert!(m("[]a]", "a"));
    }

    #[test]
    fn trailing_hyphen_in_class_is_literal() {
        assert!(m("[a-]", "-"));
        assert!(m("[a-]", "a"));
    }

    #[test]
    fn escapes_disable_metacharacters() {
        assert!(m(r"\*", "*"));
        assert!(!m(r"\*", "x"));
        assert!(m(r"\[a\]", "[a]"));
    }

    #[test]
    fn reversed_range_is_rejected() {
        let err = Glob::compile("[z-a]", 0).unwrap_err();
        assert!(err.message.contains("뒤집힌"));
    }

    #[test]
    fn unclosed_class_is_rejected_with_a_hint() {
        let err = Glob::compile("[abc", 0).unwrap_err();
        assert!(err.hint.unwrap().contains(r"\["));
    }

    #[test]
    fn bare_bracket_pair_is_unclosed_not_empty() {
        // POSIX 관습대로 `[` 바로 뒤의 `]` 는 글자다. 따라서 `[]` 는 "빈 집합"이
        // 아니라 "아직 닫히지 않은 집합"이며, 진단도 그렇게 말해야 한다.
        let err = Glob::compile("[]", 0).unwrap_err();
        assert!(err.message.contains("닫히지 않은"), "{}", err.message);
    }

    #[test]
    fn error_offsets_are_relative_to_the_whole_selector() {
        // 패턴이 선택자의 10번째 문자에서 시작했다면 오류도 그 기준이어야 한다.
        let err = Glob::compile("[abc", 10).unwrap_err();
        assert_eq!(err.offset, 10);
    }

    #[test]
    fn pathological_pattern_terminates_quickly() {
        // 역추적 정규식이라면 지수 시간이 되는 모양. 여기서는 유계다.
        let pattern = "*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*b";
        let text = "a".repeat(2000);
        // 판정 자체가 끝나는 것이 요점이다 — 끝나지 않으면 테스트가 시간 초과로 죽는다.
        assert!(!m(pattern, &text));
    }

    #[test]
    fn universal_pattern_is_detected() {
        assert!(Glob::compile("*", 0).unwrap().is_universal());
        assert!(Glob::compile("**", 0).unwrap().is_universal());
        assert!(!Glob::compile("*a*", 0).unwrap().is_universal());
    }

    #[test]
    fn source_is_preserved_for_diagnostics() {
        assert_eq!(Glob::compile("a*b", 0).unwrap().source(), "a*b");
    }
}
