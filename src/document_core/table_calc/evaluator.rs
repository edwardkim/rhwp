//! 계산식 평가기: AST → 숫자 결과

use super::parser::{parse_formula, BinOpKind, FormulaNode};
use super::tokenizer::{DirectionKind, WILDCARD_ROW};

/// 셀 값 조회 함수 타입
/// (col_index: 0-based, row_index: 0-based) → Option<f64>
pub type CellValueFn<'a> = &'a dyn Fn(usize, usize) -> Option<f64>;

/// 셀의 계산식이 주어진 방향 지정자를 쓰는지 묻는 함수 타입
/// (col_index: 0-based, row_index: 0-based, 방향) → bool
pub type SameDirectionFn<'a> = &'a dyn Fn(usize, usize, DirectionKind) -> bool;

/// 0으로 나눈 계산식의 오류. 한/글 12.30은 이런 셀에 "잘못된 계산식"을 쓴다.
pub(crate) const DIVIDE_BY_ZERO: &str = "0으로 나눌 수 없음";
/// 결과가 유한한 숫자가 아닌 계산식(SQRT(-1) 등)의 오류.
pub(crate) const NOT_FINITE: &str = "계산 결과가 유한한 숫자가 아닙니다";

fn finite_result(result: f64) -> Result<f64, String> {
    if result.is_finite() {
        Ok(result)
    } else {
        Err(NOT_FINITE.into())
    }
}

/// 구문·참조는 맞지만 값을 낼 수 없는 계산식인가. 한/글 12.30은 1/0·MOD(5,0)·SQRT(-1)·LN(0)·
/// ASIN(2)를 모두 셀에 "잘못된 계산식"으로 쓴다.
pub(crate) fn is_invalid_result_error(error: &str) -> bool {
    error == DIVIDE_BY_ZERO || error == NOT_FINITE
}

/// 평가 중 셀을 읽는 두 함수와 함수 결과 반올림 여부.
struct Cells<'a> {
    value: CellValueFn<'a>,
    same_direction: SameDirectionFn<'a>,
    round: bool,
}

/// 표 정보
pub struct TableContext {
    /// 행 수
    pub row_count: usize,
    /// 열 수
    pub col_count: usize,
    /// 현재 셀 위치 (계산식이 입력된 셀)
    pub current_row: usize,
    pub current_col: usize,
}

/// 계산식 문자열을 평가하여 결과를 반환한다.
///
/// # Arguments
/// * `formula` - 계산식 문자열 (예: "=SUM(A1:A5)+B3*2")
/// * `ctx` - 표 정보 (행/열 수, 현재 셀)
/// * `get_cell` - 셀 값 조회 함수 (col, row) → Option<f64>
pub fn evaluate_formula(
    formula: &str,
    ctx: &TableContext,
    get_cell: CellValueFn,
) -> Result<f64, String> {
    let ast = parse_formula(formula).ok_or_else(|| "수식 파싱 실패".to_string())?;
    finite_result(eval_node(
        &ast,
        ctx,
        &Cells {
            value: get_cell,
            same_direction: &|_, _, _| false,
            round: false,
        },
    )?)
}

/// 한/글 계산식 필드의 평가. 함수 결과를 소수 여섯째 자리로 맞춘 뒤 다음 연산에 쓰고(한/글 12.30:
/// SQRT(2)*1000000000=1,414,214,000, SUM(0.1234567,0)*10=1.23457, ABS(-1/3)*3=0.999999), 방향
/// 평균(AVG(LEFT) 등)은 같은 방향 계산식이 든 셀을 뺀다. 한/글 12.30: 가로 합계 옆 가로 평균은 합계
/// 칸을, 세로 곱 아래 세로 평균은 곱 칸을 빼고, 다른 방향(SUM(ABOVE))·방향 없는 식(=A5+B5,
/// SUM(A13:B13))·범위 평균(AVG(A6:C6))은 넣는다. 결과와 참조하는 0-based (열, 행)을 돌려주고,
/// 결과가 유한한 숫자가 아니면 [`NOT_FINITE`] 오류다.
pub(crate) fn evaluate_formula_in_table(
    formula: &str,
    ctx: &TableContext,
    get_cell: CellValueFn,
    same_direction: SameDirectionFn,
) -> Result<(f64, Vec<(usize, usize)>), String> {
    let refs = formula_references(formula, ctx)?;
    let ast = parse_formula(formula).ok_or_else(|| "수식 파싱 실패".to_string())?;
    let result = finite_result(eval_node(
        &ast,
        ctx,
        &Cells {
            value: get_cell,
            same_direction,
            round: true,
        },
    )?)?;
    Ok((result, refs))
}

/// 계산식이 참조하는 0-based (열, 행) 목록. 값은 계산하지 않는다.
pub(crate) fn formula_references(
    formula: &str,
    ctx: &TableContext,
) -> Result<Vec<(usize, usize)>, String> {
    if ctx.current_row >= ctx.row_count || ctx.current_col >= ctx.col_count {
        return Err("결과 셀이 표 범위 밖에 있습니다".into());
    }
    let ast = parse_formula(formula).ok_or_else(|| "수식 파싱 실패".to_string())?;
    let mut cells = Vec::new();
    collect_references(&ast, ctx, &mut cells)?;
    cells.sort_unstable_by_key(|&(col, row)| (row, col));
    cells.dedup();
    Ok(cells)
}

fn collect_references(
    node: &FormulaNode,
    ctx: &TableContext,
    cells: &mut Vec<(usize, usize)>,
) -> Result<(), String> {
    match node {
        FormulaNode::CellRef { .. } | FormulaNode::Range { .. } | FormulaNode::Direction(_) => {
            cells.extend(collect_cells(node, ctx)?);
        }
        FormulaNode::BinOp { left, right, .. } => {
            collect_references(left, ctx, cells)?;
            collect_references(right, ctx, cells)?;
        }
        FormulaNode::Negate(inner) => collect_references(inner, ctx, cells)?,
        FormulaNode::FuncCall { args, .. } => {
            for arg in args {
                collect_references(arg, ctx, cells)?;
            }
        }
        FormulaNode::Number(_) => {}
    }
    Ok(())
}

fn eval_node(node: &FormulaNode, ctx: &TableContext, get_cell: &Cells) -> Result<f64, String> {
    match node {
        FormulaNode::Number(n) => Ok(*n),

        FormulaNode::CellRef { col, row } => {
            let (c, r) = resolve_cell_ref(col, *row, ctx)?;
            Ok((get_cell.value)(c, r).unwrap_or(0.0))
        }

        FormulaNode::Negate(inner) => Ok(-eval_node(inner, ctx, get_cell)?),

        FormulaNode::BinOp { op, left, right } => {
            let l = eval_node(left, ctx, get_cell)?;
            let r = eval_node(right, ctx, get_cell)?;
            match op {
                BinOpKind::Add => Ok(l + r),
                BinOpKind::Sub => Ok(l - r),
                BinOpKind::Mul => Ok(l * r),
                BinOpKind::Div => {
                    if r == 0.0 {
                        Err(DIVIDE_BY_ZERO.into())
                    } else {
                        Ok(l / r)
                    }
                }
            }
        }

        FormulaNode::Range { .. } => Err("범위 참조는 함수 인수로만 사용 가능".into()),

        FormulaNode::Direction(_) => Err("방향 지정자는 함수 인수로만 사용 가능".into()),

        FormulaNode::FuncCall { name, args } => {
            let value = eval_function(name, args, ctx, get_cell)?;
            Ok(if get_cell.round {
                round_function_result(value)
            } else {
                value
            })
        }
    }
}

/// 셀 참조를 (col_index, row_index) 0-based로 변환
fn resolve_cell_ref(col: &str, row: u32, ctx: &TableContext) -> Result<(usize, usize), String> {
    let c = if col == "?" {
        ctx.current_col
    } else {
        if col.is_empty() || !col.chars().all(|ch| ch.is_ascii_uppercase()) {
            return Err(format!("잘못된 열: {col}"));
        }
        let one_based = col.chars().try_fold(0usize, |acc, ch| {
            acc.checked_mul(26)
                .and_then(|value| value.checked_add((ch as u8 - b'A' + 1) as usize))
        });
        one_based
            .and_then(|value| value.checked_sub(1))
            .ok_or_else(|| format!("잘못된 열: {col}"))?
    };
    let r = if row == WILDCARD_ROW {
        ctx.current_row // 와일드카드 행
    } else {
        (row as usize)
            .checked_sub(1)
            .ok_or_else(|| "행은 1부터 시작".to_string())?
    };
    Ok((c, r))
}

/// 범위/방향에서 셀 좌표 목록을 수집
fn collect_cells(arg: &FormulaNode, ctx: &TableContext) -> Result<Vec<(usize, usize)>, String> {
    match arg {
        FormulaNode::Range { start, end } => {
            if let (
                FormulaNode::CellRef { col: c1, row: r1 },
                FormulaNode::CellRef { col: c2, row: r2 },
            ) = (start.as_ref(), end.as_ref())
            {
                let (sc, sr) = resolve_cell_ref(c1, *r1, ctx)?;
                let (ec, er) = resolve_cell_ref(c2, *r2, ctx)?;
                let mut cells = Vec::new();
                let (min_r, max_r) = (sr.min(er), sr.max(er));
                let (min_c, max_c) = (sc.min(ec), sc.max(ec));
                for r in min_r..=max_r {
                    for c in min_c..=max_c {
                        cells.push((c, r));
                    }
                }
                Ok(cells)
            } else {
                Err("범위 참조 형식 오류".into())
            }
        }
        FormulaNode::Direction(dir) => {
            let mut cells = Vec::new();
            match dir {
                DirectionKind::Left => {
                    for c in 0..ctx.current_col {
                        cells.push((c, ctx.current_row));
                    }
                }
                DirectionKind::Right => {
                    for c in (ctx.current_col + 1)..ctx.col_count {
                        cells.push((c, ctx.current_row));
                    }
                }
                DirectionKind::Above => {
                    for r in 0..ctx.current_row {
                        cells.push((ctx.current_col, r));
                    }
                }
                DirectionKind::Below => {
                    for r in (ctx.current_row + 1)..ctx.row_count {
                        cells.push((ctx.current_col, r));
                    }
                }
            }
            Ok(cells)
        }
        FormulaNode::CellRef { col, row } => {
            let (c, r) = resolve_cell_ref(col, *row, ctx)?;
            Ok(vec![(c, r)])
        }
        _ => Err("함수 인수가 범위/셀/방향이 아님".into()),
    }
}

/// 소수 여섯째 자리로 맞춘다. 기본 표시 형식(printf "%.6f")과 같은 반올림이다.
fn round_function_result(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    format!("{value:.6}").parse().unwrap_or(value)
}

/// 함수 인수에서 셀 값들을 수집한다. `skip_same_direction`이면 방향 인수에서 같은 방향
/// 계산식이 든 셀을 뺀다(평균).
fn collect_values(
    args: &[FormulaNode],
    ctx: &TableContext,
    get_cell: &Cells,
    skip_same_direction: bool,
) -> Result<Vec<f64>, String> {
    let mut values = Vec::new();
    for arg in args {
        match arg {
            FormulaNode::Range { .. } | FormulaNode::Direction(_) => {
                let direction = match arg {
                    FormulaNode::Direction(dir) if skip_same_direction => Some(*dir),
                    _ => None,
                };
                let cells = collect_cells(arg, ctx)?;
                for (c, r) in cells {
                    if direction.is_some_and(|dir| (get_cell.same_direction)(c, r, dir)) {
                        continue;
                    }
                    if let Some(v) = (get_cell.value)(c, r) {
                        values.push(v);
                    }
                }
            }
            FormulaNode::CellRef { col, row } => {
                let (c, r) = resolve_cell_ref(col, *row, ctx)?;
                if let Some(v) = (get_cell.value)(c, r) {
                    values.push(v);
                }
            }
            _ => {
                values.push(eval_node(arg, ctx, get_cell)?);
            }
        }
    }
    Ok(values)
}

/// 시트 함수 평가
fn eval_function(
    name: &str,
    args: &[FormulaNode],
    ctx: &TableContext,
    get_cell: &Cells,
) -> Result<f64, String> {
    match name {
        "SUM" => {
            let vals = collect_values(args, ctx, get_cell, false)?;
            Ok(vals.iter().sum())
        }
        "AVERAGE" | "AVG" => {
            let vals = collect_values(args, ctx, get_cell, true)?;
            if vals.is_empty() {
                return Ok(0.0);
            }
            Ok(vals.iter().sum::<f64>() / vals.len() as f64)
        }
        "PRODUCT" => {
            let vals = collect_values(args, ctx, get_cell, false)?;
            Ok(vals.iter().product())
        }
        "MIN" => {
            let vals = collect_values(args, ctx, get_cell, false)?;
            Ok(vals.iter().cloned().fold(f64::INFINITY, f64::min))
        }
        "MAX" => {
            let vals = collect_values(args, ctx, get_cell, false)?;
            Ok(vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
        }
        "COUNT" => {
            let vals = collect_values(args, ctx, get_cell, false)?;
            Ok(vals.len() as f64)
        }
        // 단항 수학 함수
        "ABS" => unary_fn(args, ctx, get_cell, f64::abs),
        "SQRT" => unary_fn(args, ctx, get_cell, f64::sqrt),
        "EXP" => unary_fn(args, ctx, get_cell, f64::exp),
        // 한컴 도움말·한/글 12.30: LOG는 상용로그(LOG(100)=2), LN이 자연로그다.
        "LOG" | "LOG10" => unary_fn(args, ctx, get_cell, f64::log10),
        "LN" => unary_fn(args, ctx, get_cell, f64::ln),
        "SIN" => unary_fn(args, ctx, get_cell, f64::sin),
        "COS" => unary_fn(args, ctx, get_cell, f64::cos),
        "TAN" => unary_fn(args, ctx, get_cell, f64::tan),
        "ASIN" => unary_fn(args, ctx, get_cell, f64::asin),
        "ACOS" => unary_fn(args, ctx, get_cell, f64::acos),
        "ATAN" => unary_fn(args, ctx, get_cell, f64::atan),
        "DEGTORAD" | "RADIAN" => {
            unary_fn(args, ctx, get_cell, |d| d * std::f64::consts::PI / 180.0)
        }
        "RADTODEG" => unary_fn(args, ctx, get_cell, |r| r * 180.0 / std::f64::consts::PI),
        "SIGN" => unary_fn(args, ctx, get_cell, |v| {
            if v > 0.0 {
                1.0
            } else if v < 0.0 {
                -1.0
            } else {
                0.0
            }
        }),
        "INT" => unary_fn(args, ctx, get_cell, |v| v.trunc()),
        "CEILING" => unary_fn(args, ctx, get_cell, f64::ceil),
        "FLOOR" => unary_fn(args, ctx, get_cell, f64::floor),
        "ROUND" => {
            // 한/글은 ROUND(x, 자릿수)만 받는다. 자릿수를 뺀 ROUND(x)는 rhwp가 받아 온 형태라 0자리로 둔다.
            if args.is_empty() || args.len() > 2 {
                return Err("ROUND는 1~2개 인수 필요 (값, 자릿수)".into());
            }
            let value = eval_node(&args[0], ctx, get_cell)?;
            let digits = match args.get(1) {
                Some(arg) => eval_node(arg, ctx, get_cell)?,
                None => 0.0,
            };
            Ok(round_to_digits(value, digits))
        }
        "TRUNC" => unary_fn(args, ctx, get_cell, f64::trunc),
        "MOD" => {
            if args.len() < 2 {
                return Err("MOD는 2개 인수 필요".into());
            }
            let a = eval_node(&args[0], ctx, get_cell)?;
            let b = eval_node(&args[1], ctx, get_cell)?;
            if b == 0.0 {
                Err(DIVIDE_BY_ZERO.into())
            } else {
                Ok(a % b)
            }
        }
        "IF" => {
            if args.len() < 3 {
                return Err("IF는 3개 인수 필요 (조건, 참, 거짓)".into());
            }
            let cond = eval_node(&args[0], ctx, get_cell)?;
            if cond != 0.0 {
                eval_node(&args[1], ctx, get_cell)
            } else {
                eval_node(&args[2], ctx, get_cell)
            }
        }
        _ => Err(format!("지원하지 않는 함수: {}", name)),
    }
}

/// 소수점 아래 `digits` 자리에서 반올림한다(음수면 소수점 위, 소수 부분은 버린다). 한/글 12.30처럼
/// 0.5는 0에서 먼 쪽으로 올리고, 이진 표현 오차를 15자리 유효 숫자에서 걷어 ROUND(2.675,2)를
/// 2.68로 낸다.
fn round_to_digits(value: f64, digits: f64) -> f64 {
    let digits = digits.trunc().clamp(-308.0, 308.0) as i32;
    let scale = 10f64.powi(digits.abs());
    let scaled = if digits >= 0 {
        value * scale
    } else {
        value / scale
    };
    if !scaled.is_finite() || !scale.is_finite() {
        return value;
    }
    let rounded = format!("{scaled:.14e}")
        .parse::<f64>()
        .unwrap_or(scaled)
        .round();
    if digits >= 0 {
        rounded / scale
    } else {
        rounded * scale
    }
}

fn unary_fn(
    args: &[FormulaNode],
    ctx: &TableContext,
    get_cell: &Cells,
    f: impl Fn(f64) -> f64,
) -> Result<f64, String> {
    if args.is_empty() {
        return Err("함수 인수 필요".into());
    }
    let v = eval_node(&args[0], ctx, get_cell)?;
    Ok(f(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> TableContext {
        TableContext {
            row_count: 5,
            col_count: 5,
            current_row: 4,
            current_col: 0,
        }
    }

    fn sample_cell(col: usize, row: usize) -> Option<f64> {
        // 5x5 표: 값 = (row+1)*10 + (col+1)
        Some((row as f64 + 1.0) * 10.0 + (col as f64 + 1.0))
    }

    #[test]
    fn test_literal() {
        let ctx = make_ctx();
        let r = evaluate_formula("=42", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 42.0);
    }

    #[test]
    fn test_cell_ref() {
        let ctx = make_ctx();
        // A1 = (0,0) = 11.0
        let r = evaluate_formula("=A1", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 11.0);
    }

    #[test]
    fn test_arithmetic() {
        let ctx = make_ctx();
        // A1(11) + B2(22) * 2 = 11 + 44 = 55
        let r = evaluate_formula("=A1+B2*2", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 55.0);
    }

    #[test]
    fn test_sum_range() {
        let ctx = make_ctx();
        // SUM(A1:A3) = 11 + 21 + 31 = 63
        let r = evaluate_formula("=SUM(A1:A3)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 63.0);
    }

    #[test]
    fn test_sum_direction() {
        // 현재 셀 (0, 4), above = (0,0)~(0,3) = 11+21+31+41 = 104
        let ctx = make_ctx();
        let r = evaluate_formula("=SUM(above)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 104.0);
    }

    #[test]
    fn test_avg() {
        let ctx = make_ctx();
        // AVG(A1:A3) = (11+21+31)/3 = 21
        let r = evaluate_formula("=AVG(A1:A3)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 21.0);
    }

    #[test]
    fn test_product() {
        let ctx = make_ctx();
        // PRODUCT(B1,C3) = 12 * 33 = 396
        let r = evaluate_formula("=PRODUCT(B1,C3)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 396.0);
    }

    #[test]
    fn test_nested_function() {
        let ctx = make_ctx();
        // SUM(A1:A3, AVG(B1,B2)) = 63 + (12+22)/2 = 63 + 17 = 80
        let r = evaluate_formula("=SUM(A1:A3,AVG(B1,B2))", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 80.0);
    }

    #[test]
    fn test_min_max() {
        let ctx = make_ctx();
        let r = evaluate_formula("=MIN(A1:C1)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 11.0);
        let r = evaluate_formula("=MAX(A1:C1)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 13.0);
    }

    #[test]
    fn test_abs_sqrt() {
        let ctx = make_ctx();
        let r = evaluate_formula("=ABS(-25)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 25.0);
        let r = evaluate_formula("=SQRT(16)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 4.0);
    }

    #[test]
    fn test_if_function() {
        let ctx = make_ctx();
        let r = evaluate_formula("=IF(1,10,20)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 10.0);
        let r = evaluate_formula("=IF(0,10,20)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 20.0);
    }

    #[test]
    fn test_mod_function() {
        let ctx = make_ctx();
        let r = evaluate_formula("=MOD(10,3)", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 1.0);
    }

    #[test]
    fn test_complex_formula() {
        let ctx = make_ctx();
        // a1+(b3-3)*2+sum(a1:b5,avg(c3,e5-3))
        // a1=11, b3=32, sum(a1:b5)=11+12+21+22+31+32+41+42+51+52=315
        // avg(c3, e5-3) = avg(33, 55-3) = avg(33, 52) = 42.5
        // = 11 + (32-3)*2 + (315 + 42.5) = 11 + 58 + 357.5 = 426.5
        let r =
            evaluate_formula("=a1+(b3-3)*2+sum(a1:b5,avg(c3,e5-3))", &ctx, &sample_cell).unwrap();
        assert_eq!(r, 426.5);
    }

    #[test]
    fn test_div_by_zero() {
        let ctx = make_ctx();
        let r = evaluate_formula("=1/0", &ctx, &sample_cell);
        assert!(r.is_err());
    }

    #[test]
    fn test_row_zero_is_not_wildcard() {
        // 스펙(mydocs/plans/archives/task_370.md): 행은 1부터 시작하고, 와일드카드는 '?'만 인정한다.
        // "A0"처럼 명시적으로 0행을 참조하는 것은 잘못된 입력이며, 현재 행(current_row)으로
        // 조용히 대체되어서는 안 된다.
        let ctx = make_ctx(); // current_row = 4
        let r = evaluate_formula("=A0", &ctx, &sample_cell);
        assert!(
            r.is_err(),
            "A0은 현재 행(A5=41.0)으로 치환되지 않고 오류가 되어야 하는데 {:?} 를 반환함",
            r
        );
    }
}
