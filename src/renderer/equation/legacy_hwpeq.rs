//! [#7105] 레거시 hwpeq5 OLE 수식 방언(`\CMD 인자 \TAB`)을 현행 문법으로 정규화한다.
//!
//! 한/글 5.x·97 계열이 `hwpeq5X.ocx` 로 저장한 수식은 native `$eqed` 컨트롤이 아니라
//! **OLE 개체**이고, 그 `Contents` 스트림(`Hwp 5.0 Equation Editor` 서명)에 담긴 스크립트는
//! 현행 수식 문법과 다른 방언이다. 두 가지가 다르다.
//!
//! 1. 명령이 역슬래시로 시작한다 — `\SUB` `\SUP` `\OVER` `\BAR` `\CDOT` `\RTN` `\(`.
//! 2. **`\TAB` 이 인자 종결자**다. 중괄호가 없고, 인자는 `\TAB` 이 닫는다.
//!
//! ```text
//! I \SUB E \TAB =I \SUB C \TAB +I \SUB B \TAB     →  I_{E}=I_{C}+I_{B}
//! \OVER 1.7-0.7 \TAB 20K \TAB =0.05mA             →  {1.7-0.7} over {20K}=0.05mA
//! \BAR  \BAR A \TAB  \TAB =A                      →  bar {bar {A}}=A
//! \( V \SUB GS \TAB - … \TAB  \TAB  \SUP 2 \TAB   →  left ( … right )^{2}
//! ```
//!
//! # 근거
//!
//! 신고 원본 `실험4 Transistor-MOSFET.hwp`(#7105) 의 OLE 22개에서 스크립트 42개를 뽑았고
//! 그중 30개가 이 방언을 쓴다(`\TAB` 78 · `\BAR` 43 · `\SUB` 27 · `\CDOT` 26 · `\RTN` 4 ·
//! `\OVER` 3 · `\SUP` 1). 신고자가 첨부한 한/글 22 출력(`Hwp 2022 12.0.0.4204`) 2쪽을
//! `pdftotext -bbox` 로 96dpi 환산해 재면
//!
//! | 대상 | 정본 |
//! | --- | --- |
//! | 본체 글리프 | y=464.0 h=16.8 |
//! | `\SUB` 인자 | y=472.1 **h=10.5** (0.625배·baseline +8.1px) |
//! | `\OVER` | y=518.0 / 528.0 / 536.0 세 단으로 쌓임 |
//! | `TAB` 글리프 | **없음** |
//!
//! 곧 한/글은 `\TAB` 을 소비하고, `\SUB` 를 앞 피연산자에 붙는 진짜 첨자로, `\OVER a \TAB
//! b \TAB` 을 분수로 조판한다. 정규화 전 rhwp 는 `\TAB` 을 식별자로 흘려 `TAB` 세 글자를
//! 찍고, 그 때문에 조판 폭이 저장 extent 의 2.3~2.9배가 되어 렌더러가 `scale(0.42,1.0)` 로
//! 압착했다.
//!
//! # 적용 범위
//!
//! 게이트는 **`\TAB` 이 있는 스크립트**뿐이다. 현행 문법 스크립트는 `\TAB` 을 쓰지 않으며
//! (`fixtures/catalog.tsv` 머리말: *"Scripts must not contain TAB"*) 정규화를 통과하지 않는다.
//! `A \CDOT 1=A` 처럼 역슬래시는 쓰되 `\TAB` 이 없는 스크립트도 종전 경로 그대로다.
//!
//! **미포함**: `&=&` 정렬과 `\RTN` 줄바꿈이 함께 오는 eqalign 형태(원본 4개 스크립트,
//! `\BAR A \CDOT B \TAB &=& … \RTN …`)는 이 변경이 다루지 않는다. `\RTN` 은 행 구분자
//! `#` 로만 옮기고 eqalign 로 감싸지 않는다 — 감싸는 규칙의 독립 근거가 아직 없다.

/// `\TAB` 을 쓰는 레거시 hwpeq5 스크립트면 현행 문법으로 옮긴 문자열을 준다.
///
/// 현행 문법 스크립트에는 `None` 을 주어 호출부가 원문을 그대로 쓰게 한다.
pub(crate) fn normalize(script: &str) -> Option<String> {
    if !has_tab_command(script) {
        return None;
    }
    let chars: Vec<char> = script.chars().collect();
    let mut cursor = Cursor { chars, pos: 0 };
    Some(cursor.parse_sequence(false))
}

/// `\TAB` 명령이 들어 있는지. `\TABLE` 같은 더 긴 이름은 세지 않는다.
fn has_tab_command(script: &str) -> bool {
    let bytes: Vec<char> = script.chars().collect();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if bytes[i] == '\\'
            && bytes[i + 1] == 'T'
            && bytes[i + 2] == 'A'
            && bytes[i + 3] == 'B'
            && bytes.get(i + 4).is_none_or(|c| !c.is_ascii_alphanumeric())
        {
            return true;
        }
        i += 1;
    }
    false
}

struct Cursor {
    chars: Vec<char>,
    pos: usize,
}

impl Cursor {
    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    /// 현재 위치의 `\NAME` 을 읽되 소비하지는 않는다.
    fn peek_command(&self) -> Option<String> {
        if self.chars.get(self.pos) != Some(&'\\') {
            return None;
        }
        let mut end = self.pos + 1;
        while self.chars.get(end).is_some_and(|c| c.is_ascii_alphabetic()) {
            end += 1;
        }
        if end == self.pos + 1 {
            return None;
        }
        Some(self.chars[self.pos + 1..end].iter().collect())
    }

    fn consume_command(&mut self, name: &str) {
        self.pos += 1 + name.chars().count();
    }

    /// `until_tab` 이면 `\TAB` 을 만날 때까지, 아니면 입력 끝까지 읽어 현행 문법으로 옮긴다.
    ///
    /// 인자 하나를 읽는 모든 경로가 이 함수를 `true` 로 부르므로, `\TAB` 의 종결 의미가
    /// 한 곳에서만 구현된다. 최상위(`false`)에서 만나는 `\TAB` 은 짝이 없으므로 버린다 —
    /// 원본 스크립트는 대부분 `\TAB` 으로 끝난다.
    fn parse_sequence(&mut self, until_tab: bool) -> String {
        let mut out = String::new();
        while !self.at_end() {
            let Some(name) = self.peek_command() else {
                // `\(` 는 알파벳이 아니라 위에서 안 걸린다.
                if self.chars[self.pos] == '\\' && self.chars.get(self.pos + 1) == Some(&'(') {
                    self.pos += 2;
                    let inner = self.parse_sequence(true);
                    out.push_str(" left ( ");
                    out.push_str(inner.trim());
                    out.push_str(" right ) ");
                    continue;
                }
                out.push(self.chars[self.pos]);
                self.pos += 1;
                continue;
            };

            match name.as_str() {
                "TAB" => {
                    self.consume_command(&name);
                    if until_tab {
                        return out;
                    }
                    // 짝 없는 종결자 — 버린다.
                }
                "SUB" | "SUP" => {
                    self.consume_command(&name);
                    let arg = self.parse_sequence(true);
                    // 첨자는 **앞** 피연산자에 붙는다. 현행 토크나이저는 `_`/`^` 로만
                    // 결합하므로 사이의 공백을 지워 붙여 준다.
                    while out.ends_with(' ') {
                        out.pop();
                    }
                    out.push(if name == "SUB" { '_' } else { '^' });
                    out.push('{');
                    out.push_str(arg.trim());
                    out.push('}');
                }
                "OVER" => {
                    self.consume_command(&name);
                    let numerator = self.parse_sequence(true);
                    let denominator = self.parse_sequence(true);
                    out.push_str(" {");
                    out.push_str(numerator.trim());
                    out.push_str("} over {");
                    out.push_str(denominator.trim());
                    out.push_str("} ");
                }
                "BAR" => {
                    self.consume_command(&name);
                    let arg = self.parse_sequence(true);
                    out.push_str(" bar {");
                    out.push_str(arg.trim());
                    out.push_str("} ");
                }
                "RTN" => {
                    self.consume_command(&name);
                    out.push_str(" # ");
                }
                // 나머지(`\CDOT` `\TIMES` …)는 이름만 넘긴다 — 현행 기호·장식표가 이미 안다.
                _ => {
                    self.consume_command(&name);
                    out.push(' ');
                    out.push_str(&name);
                    out.push(' ');
                }
            }
        }
        out
    }
}
