//! 계산식 필드의 명령 문자열과 결과 표시 형식 (한/글 12.30 저장본 기준)
//!
//! 한/글은 계산식을 셀의 FORMULA 필드로 저장한다. HWP5 CTRL_HEADER의 명령과 HWPX `Command`는
//! `식??결과 형식;;마지막 결과`(예: `=SUM(LEFT)??%g,;;150`)이고, HWPX는 `Formula`·`ResultFormat`·
//! `LastResult`도 따로 쓴다. 결과 형식은 `%g`(기본 형식)·`%.0f`(정수형)·`%.Nf`(소수 N자리)이고,
//! 끝의 `,`는 세 자리마다 쉼표다.

/// 값을 낼 수 없는 계산식(1/0·SQRT(-1)·LN(0) 등)에 한/글 12.30이 쓰는 셀 글.
pub const INVALID_RESULT_TEXT: &str = "잘못된 계산식";

/// 한/글 HWPX 저장본의 FORMULA 필드 `Prop` 값이자 HWP5 CTRL_HEADER의 기타 속성.
pub const FORMULA_FIELD_PROP: u8 = 8;

/// `식??형식;;결과`를 (식, 형식, 결과)로 나눈다. 모양이 다르면 `None`. 와일드카드로 끝나는 식
/// (`=A?+B?` → `=A?+B???%g,;;3`)이 있어 뒤에서부터 나눈다.
pub fn split_formula_command(command: &str) -> Option<(&str, &str, &str)> {
    let (rest, last) = command.rsplit_once(";;")?;
    let (formula, format) = rest.rsplit_once("??")?;
    Some((formula, format, last))
}

/// (식, 형식, 결과)로 한/글 명령 문자열을 만든다.
pub fn formula_command(formula: &str, result_format: &str, last_result: &str) -> String {
    format!("{formula}??{result_format};;{last_result}")
}

/// 한/글 12.30의 계산 결과 글.
///
/// 기본 형식(`%g`)은 printf("%.6f")처럼 정확한 값을 소수 여섯째 자리에서 짝수 쪽으로 맞추고
/// 끝의 0을 지운다(1/128 → 0.007812). 정수형·소수 N자리는 절반을 0에서 먼 쪽으로 올린다
/// (2.5 → 3, 0.125 → 0.13). 큰 수도 지수 없이 모든 자리를 쓰고, 0으로 맞춘 음수는 부호를
/// 남긴다(-0.0000004 → -0). 모르는 형식은 기본 형식으로 쓴다.
pub fn format_formula_result(value: f64, result_format: &str) -> String {
    let (spec, thousands) = match result_format.strip_suffix(',') {
        Some(spec) => (spec, true),
        None => (result_format, false),
    };
    let digits = spec
        .strip_prefix("%.")
        .and_then(|rest| rest.strip_suffix('f'))
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|n| *n <= 20);
    let abs = value.abs();
    let mut text = match digits {
        Some(digits) => round_half_away(abs, digits),
        None => {
            let fixed = format!("{abs:.6}");
            fixed
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    };
    if thousands {
        let (integer, fraction) = match text.split_once('.') {
            Some((integer, fraction)) => (integer.to_string(), Some(fraction.to_string())),
            None => (text.clone(), None),
        };
        let mut grouped = String::with_capacity(integer.len() + integer.len() / 3 + 8);
        for (i, ch) in integer.chars().enumerate() {
            if i > 0 && (integer.len() - i) % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(ch);
        }
        if let Some(fraction) = fraction {
            grouped.push('.');
            grouped.push_str(&fraction);
        }
        text = grouped;
    }
    if value < 0.0 {
        text.insert(0, '-');
    }
    text
}

/// 음 아닌 값을 소수 `digits`자리에서 절반은 위로 올려 쓴다. 이진 값의 정확한 십진 전개로
/// 판단해야 0.125 같은 딱 절반과 2.675(실제로는 2.67499…) 같은 근사를 가른다.
fn round_half_away(abs: f64, digits: usize) -> String {
    // f64의 십진 전개는 소수 1074자리 안에서 끝난다.
    let exact = format!("{abs:.1074}");
    let (integer, fraction) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut kept: Vec<u8> = integer
        .bytes()
        .chain(fraction.bytes().take(digits))
        .collect();
    if fraction.as_bytes().get(digits).is_some_and(|d| *d >= b'5') {
        let mut i = kept.len();
        loop {
            if i == 0 {
                kept.insert(0, b'1');
                break;
            }
            i -= 1;
            if kept[i] == b'9' {
                kept[i] = b'0';
            } else {
                kept[i] += 1;
                break;
            }
        }
    }
    let split = kept.len() - digits;
    let mut text = String::from_utf8(kept[..split].to_vec()).unwrap_or_default();
    if digits > 0 {
        text.push('.');
        text.push_str(std::str::from_utf8(&kept[split..]).unwrap_or_default());
    }
    text
}
