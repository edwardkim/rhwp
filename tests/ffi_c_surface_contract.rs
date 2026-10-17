//! Issue #3891 회귀 가드 — C ABI 표면과 언어 래퍼 선언의 정합.
//!
//! `bindings/Native`(`rhwp-native-ffi`)의 `pub extern "C"` 함수가 권위다. 그 표면을
//! 소비하는 곳이 셋인데 **어느 것도 CI 검사를 받지 않았다**:
//!
//! - C 헤더 `bindings/swift/Sources/CRhwpNative/rhwp_native_ffi.h` (Swift 가 `import`)
//! - C# 래퍼 `bindings/csharp/RhwpNative.cs` (`[DllImport]` + `extern`)
//!
//! 실제로 표류가 있었다 — Rust 가 `rhwp_read_text` 를 export 하는데 C# 래퍼에는
//! 선언이 없었다(C# 은 2026-05-04, 그 함수는 이후 추가, Swift 는 05-13 에 반영).
//! #3664 가 크레이트 자체를 CI 에 올렸지만 **래퍼는 여전히 밖**이라 이 가드를 둔다.
//!
//! # 이 가드가 보장하지 않는 것
//!
//! 선언이 일치해도 **런타임 동작·ABI 호환은 보장하지 않는다.** 실제 링크·호출 검증은
//! Swift·.NET 툴체인이 있는 환경에서 별도로 해야 한다(현 개발 환경은 Linux 로 두
//! 툴체인이 없다). 이 가드는 "이름·인자 수가 어긋나는" 표류만 잡는다.

use std::collections::BTreeMap;
use std::fs;

const RUST_FFI: &str = "bindings/Native/src/lib.rs";
const C_HEADER: &str = "bindings/swift/Sources/CRhwpNative/rhwp_native_ffi.h";
const CSHARP_WRAPPER: &str = "bindings/csharp/RhwpNative.cs";

/// 함수 하나의 정규화된 표면 — 이름 → 인자 개수.
///
/// 타입 문자열을 그대로 비교하지 않는다. `*const c_char` / `const char *` / `byte[]`
/// 는 같은 것을 가리키는 세 표기이고, 공백·별표 위치까지 맞추려 들면 오탐이 난다.
/// 이름과 인자 **개수**가 어긋나는 것이 실제로 터지는 표류다.
type Surface = BTreeMap<String, usize>;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path} 를 읽을 수 없다: {e}"))
}

/// Rust `pub [unsafe] extern "C" fn NAME(args) -> ret` 를 파싱한다.
fn rust_surface() -> Surface {
    let src = [read(RUST_FFI), read("bindings/Native/src/session.rs")].join("\n");
    let mut out = Surface::new();
    let mut rest = src.as_str();

    while let Some(pos) = rest.find(r#"extern "C" fn "#) {
        let after = &rest[pos + r#"extern "C" fn "#.len()..];
        let Some(paren) = after.find('(') else { break };
        let name = after[..paren].trim().to_string();

        // 여는 괄호부터 짝이 맞는 닫는 괄호까지 — 인자 목록.
        let mut depth = 0usize;
        let mut end = None;
        for (i, ch) in after[paren..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(paren + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else { break };
        let args = &after[paren + 1..end];
        out.insert(name, count_args(args));
        rest = &after[end..];
    }

    assert!(
        !out.is_empty(),
        "{RUST_FFI} 에서 extern \"C\" 함수를 하나도 파싱하지 못했다 — \
         선언 형식이 바뀌었다면 이 가드를 함께 고쳐야 한다(조용히 통과시키지 않는다)"
    );
    out
}

/// C 헤더의 `ret name(args);` 선언을 파싱한다.
fn c_header_surface() -> Surface {
    let src = read(C_HEADER);
    let mut out = Surface::new();

    for line in src.lines() {
        let line = line.trim();
        if !line.ends_with(");") || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let Some(paren) = line.find('(') else {
            continue;
        };
        // 여는 괄호 앞 토큰이 함수 이름(`char *rhwp_export_text` → `rhwp_export_text`).
        let head = &line[..paren];
        let Some(name) = head.rsplit(|c: char| c.is_whitespace() || c == '*').next() else {
            continue;
        };
        if !name.starts_with("rhwp_") {
            continue;
        }
        let args = &line[paren + 1..line.len() - 2];
        out.insert(name.to_string(), count_args(args));
    }

    assert!(
        !out.is_empty(),
        "{C_HEADER} 에서 rhwp_ 선언을 하나도 파싱하지 못했다 — 형식 변화 의심"
    );
    out
}

/// C# `private static extern RET name(args);` 를 파싱한다.
fn csharp_surface() -> Surface {
    let src = read(CSHARP_WRAPPER);
    let mut out = Surface::new();

    for line in src.lines() {
        let line = line.trim();
        if !line.contains("extern ") || !line.ends_with(");") {
            continue;
        }
        let Some(paren) = line.find('(') else {
            continue;
        };
        let head = &line[..paren];
        let Some(name) = head.split_whitespace().last() else {
            continue;
        };
        if !name.starts_with("rhwp_") {
            continue;
        }
        let args = &line[paren + 1..line.len() - 2];
        out.insert(name.to_string(), count_args(args));
    }

    assert!(
        !out.is_empty(),
        "{CSHARP_WRAPPER} 에서 extern 선언을 하나도 파싱하지 못했다 — 형식 변화 의심"
    );
    out
}

/// 인자 목록 문자열에서 최상위 콤마로 인자 수를 센다(빈 목록은 0).
fn count_args(args: &str) -> usize {
    // Rust 는 여러 줄 선언에서 후행 콤마를 쓴다(`page: i32,\n)`). 그대로 세면 인자가
    // 하나 더 있는 것으로 읽혀 C 헤더(후행 콤마 없음)와 항상 어긋난다 — 실측으로 잡은
    // 오탐이다. 콤마로 끊은 뒤 빈 조각을 버리는 방식으로 표기 차이를 흡수한다.
    let args = args.trim().trim_end_matches(',').trim();
    if args.is_empty() || args == "void" {
        return 0;
    }
    let mut depth = 0usize;
    let mut n = 1usize;
    for ch in args.chars() {
        match ch {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => n += 1,
            _ => {}
        }
    }
    n
}

fn diff(authority: &Surface, other: &Surface, label: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, argc) in authority {
        match other.get(name) {
            None => problems.push(format!("{label}: `{name}` 선언 없음 (Rust 는 export 함)")),
            Some(other_argc) if other_argc != argc => problems.push(format!(
                "{label}: `{name}` 인자 수 불일치 — Rust {argc} vs {label} {other_argc}"
            )),
            _ => {}
        }
    }
    for name in other.keys() {
        if !authority.contains_key(name) {
            problems.push(format!(
                "{label}: `{name}` 를 선언하는데 Rust 에는 없음 (제거된 함수?)"
            ));
        }
    }
    problems
}

#[test]
fn c_header_matches_rust_ffi_surface() {
    let rust = rust_surface();
    let header = c_header_surface();
    let problems = diff(&rust, &header, "C 헤더");
    assert!(
        problems.is_empty(),
        "C ABI 표면과 헤더가 어긋난다 — Swift 가 이 헤더를 import 한다.\n{}",
        problems.join("\n")
    );
}

#[test]
fn csharp_wrapper_matches_rust_ffi_surface() {
    let rust = rust_surface();
    let csharp = csharp_surface();
    let problems = diff(&rust, &csharp, "C# 래퍼");
    assert!(
        problems.is_empty(),
        "C ABI 표면과 C# 래퍼가 어긋난다.\n{}",
        problems.join("\n")
    );
}
