//! [#5791] 도움말 텍스트의 출력 경계 — 통짜 출력과 명령별 절 출력을 **한 텍스트**에서 만든다.
//!
//! `rhwp --help` 안에는 이미 명령별 절이 있다(`  <명령> <인자> [옵션]` 머리줄 +
//! 6칸 들여쓴 본문). 그러므로 명령별 `--help` 는 새 텍스트를 쓰는 일이 아니라
//! **이미 있는 절을 고르는 일**이다.
//!
//! 고르는 자리를 `println!` **호출 지점 1,163개**가 아니라 출력 경계 한 곳에 둔다.
//! 그래서 이 모듈은 형제 모듈이 쓰는 `println!` 을 같은 이름의 매크로로 가려
//! 여기 `emit` 으로 모은다 — 호출 지점은 한 줄도 건드리지 않고, 그 파일들이 계속
//! "그냥 도움말을 찍는 코드"로 읽힌다(명령이 추가되는 자리라 병렬 PR 충돌면도 그대로).
//!
//! 스코프가 없으면 `emit` 은 곧장 stdout 으로 찍는다 — 통짜 출력은 바이트가 같다.

use std::cell::RefCell;

thread_local! {
    static SCOPE: RefCell<Option<Scope>> = const { RefCell::new(None) };
}

/// 고를 절과, 고르는 동안의 위치.
struct Scope {
    head: String,
    sub: Option<String>,
    /// 지금 지나가는 줄이 고를 절 안인가.
    inside: bool,
    lines: Vec<String>,
}

/// 절 머리줄인가 — 정확히 2칸 들여쓰고 그다음이 공백이 아니면 새 절이 시작한다.
fn block_head(line: &str) -> Option<(&str, Option<&str>)> {
    let rest = line.strip_prefix("  ")?;
    if rest.starts_with(' ') || rest.is_empty() {
        return None;
    }
    let mut it = rest.split_whitespace();
    let name = it.next()?;
    Some((name, it.next()))
}

impl Scope {
    fn feed(&mut self, line: String) {
        if let Some((name, next)) = block_head(&line) {
            self.inside = name == self.head
                && match self.sub.as_deref() {
                    // 하위 지목이면 머리줄의 두 번째 토큰까지 같아야 한다.
                    Some(want) => next == Some(want),
                    None => true,
                };
        } else if !line.is_empty() && !line.starts_with("  ") {
            // 들여쓰지 않은 비어 있지 않은 줄 = 절 밖(구역 제목). 절이 여기서 끝난다.
            self.inside = false;
        }
        if self.inside {
            self.lines.push(line);
        }
    }
}

/// 도움말 한 줄. 스코프가 걸려 있으면 고르고, 아니면 그대로 찍는다.
pub(super) fn emit(line: String) {
    SCOPE.with(|cell| match cell.borrow_mut().as_mut() {
        None => std::println!("{line}"),
        Some(scope) => scope.feed(line),
    });
}

/// `printer` 를 스코프 안에서 돌려 해당 절의 줄만 모은다. 한 줄도 없으면 빈 벡터.
pub(super) fn collect(head: &str, sub: Option<&str>, printer: impl Fn()) -> Vec<String> {
    SCOPE.with(|cell| {
        *cell.borrow_mut() = Some(Scope {
            head: head.to_string(),
            sub: sub.map(str::to_string),
            inside: false,
            lines: Vec::new(),
        });
    });
    printer();
    let mut lines = SCOPE
        .with(|cell| cell.borrow_mut().take())
        .map(|s| s.lines)
        .unwrap_or_default();
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    lines
}

/// 형제 모듈(`public`·`edit`·`protocol`)의 `println!` 을 가려 `emit` 으로 보낸다.
macro_rules! println {
    () => { $crate::cli::metadata::help::sink::emit(String::new()) };
    ($($arg:tt)*) => { $crate::cli::metadata::help::sink::emit(format!($($arg)*)) };
}
pub(super) use println;
