//! `calc()` / `min()` / `max()` / `clamp()` evaluator (CSS-6).
//!
//! Turns the body of a `calc(...)` (or sibling math function) — a
//! `Vec<ComponentValue>` produced by the parser (CSS-2) — into an
//! [`Expr`] tree, and evaluates it to a single resolved px length
//! given a [`Context`] that supplies em / rem / vw / vh / % bases.
//!
//! Property parsing (CSS-8) consumes this when a length-bearing
//! property has a `Function { name: "calc", body }` value:
//!
//! ```ignore
//! use pdf_oxide::html_css::css::calc::{evaluate_function, Context};
//! let ctx = Context { parent_px: 600.0, font_size_px: 16.0,
//!                     root_font_size_px: 16.0, viewport_w_px: 1024.0,
//!                     viewport_h_px: 768.0 };
//! let px = evaluate_function("calc", &body, &ctx)?;  // → 580.0
//! ```
//!
//! v0.3.35 supports the units listed in [`Unit`], the four basic
//! operators, parentheses, `min(a, b, ...)`, `max(a, b, ...)`, and
//! `clamp(min, val, max)`. Constants (`pi`, `e`, `infinity`) and
//! advanced math functions (`sin`, `mod`, `round`, …) are deferred —
//! they parse cleanly via the existing component-value tree but
//! evaluate to a `Err(EvalError::UnsupportedFunction)`.

use thiserror::Error;

use super::parser::ComponentValue;
use super::tokenizer::Token;

// ─────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────

/// Layout context required to resolve relative CSS lengths to px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Context {
    /// Containing block's resolved width (or height — caller picks
    /// based on which axis the property lives on) in px. Drives `%`
    /// resolution.
    pub parent_px: f32,
    /// Element's own resolved `font-size` in px. Drives `em`.
    pub font_size_px: f32,
    /// Root element's resolved `font-size` in px. Drives `rem`.
    pub root_font_size_px: f32,
    /// Viewport width in px — drives `vw`/`vmin`/`vmax`.
    pub viewport_w_px: f32,
    /// Viewport height in px — drives `vh`/`vmin`/`vmax`.
    pub viewport_h_px: f32,
}

impl Default for Context {
    fn default() -> Self {
        // Sensible defaults for unit tests + cases where no DOM
        // context exists yet.
        Self {
            parent_px: 0.0,
            font_size_px: 16.0,
            root_font_size_px: 16.0,
            viewport_w_px: 1024.0,
            viewport_h_px: 768.0,
        }
    }
}

/// CSS length unit understood by the evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Already in px — no conversion needed.
    Px,
    /// `pt` = 4/3 px (CSS reference pixel @ 96 dpi).
    Pt,
    /// `pc` = 16 px.
    Pc,
    /// `in` = 96 px.
    In,
    /// `cm` = 96/2.54 px.
    Cm,
    /// `mm` = `cm`/10.
    Mm,
    /// `Q` = `mm`/4 (quarter-millimetre).
    Q,
    /// `em` = element's font-size.
    Em,
    /// `rem` = root font-size.
    Rem,
    /// `ex` ≈ 0.5em (DejaVu-style approximation; no per-font ex height
    /// here, refine in CSS-8 once the typed length supports per-font
    /// metrics).
    Ex,
    /// `ch` ≈ 0.5em (rough; same caveat as `ex`).
    Ch,
    /// `vw` = viewport width %.
    Vw,
    /// `vh` = viewport height %.
    Vh,
    /// `vmin` = min(vw, vh).
    Vmin,
    /// `vmax` = max(vw, vh).
    Vmax,
    /// Percentage of `parent_px`.
    Percent,
    /// Unitless number — only valid in multipliers/divisors and as
    /// `line-height`. The evaluator returns its raw value when used
    /// as a length, callers decide if that's an error.
    None,
}

impl Unit {
    /// Convert this unit to px given a context.
    pub fn to_px(self, value: f32, ctx: &Context) -> f32 {
        match self {
            Unit::Px => value,
            Unit::Pt => value * (4.0 / 3.0),
            Unit::Pc => value * 16.0,
            Unit::In => value * 96.0,
            Unit::Cm => value * (96.0 / 2.54),
            Unit::Mm => value * (96.0 / 25.4),
            Unit::Q => value * (96.0 / 25.4 / 4.0),
            Unit::Em => value * ctx.font_size_px,
            Unit::Rem => value * ctx.root_font_size_px,
            // Ex / Ch are font-relative; without font metrics, the spec
            // permits a 0.5em fallback for Ex and 0.5em for Ch.
            Unit::Ex | Unit::Ch => value * ctx.font_size_px * 0.5,
            Unit::Vw => value * ctx.viewport_w_px / 100.0,
            Unit::Vh => value * ctx.viewport_h_px / 100.0,
            Unit::Vmin => value * ctx.viewport_w_px.min(ctx.viewport_h_px) / 100.0,
            Unit::Vmax => value * ctx.viewport_w_px.max(ctx.viewport_h_px) / 100.0,
            Unit::Percent => value * ctx.parent_px / 100.0,
            Unit::None => value,
        }
    }

    /// Look up by lowercase unit string. Returns `None` for an
    /// unknown unit.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "px" => Self::Px,
            "pt" => Self::Pt,
            "pc" => Self::Pc,
            "in" => Self::In,
            "cm" => Self::Cm,
            "mm" => Self::Mm,
            "q" => Self::Q,
            "em" => Self::Em,
            "rem" => Self::Rem,
            "ex" => Self::Ex,
            "ch" => Self::Ch,
            "vw" => Self::Vw,
            "vh" => Self::Vh,
            "vmin" => Self::Vmin,
            "vmax" => Self::Vmax,
            _ => return None,
        })
    }
}

/// Calc expression tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal with a unit.
    Number(f32, Unit),
    /// `a + b`.
    Add(Box<Expr>, Box<Expr>),
    /// `a - b`.
    Sub(Box<Expr>, Box<Expr>),
    /// `a * b`.
    Mul(Box<Expr>, Box<Expr>),
    /// `a / b`.
    Div(Box<Expr>, Box<Expr>),
    /// `min(a, b, ...)`.
    Min(Vec<Expr>),
    /// `max(a, b, ...)`.
    Max(Vec<Expr>),
    /// `clamp(low, val, high)`.
    Clamp(Box<Expr>, Box<Expr>, Box<Expr>),
    /// `calc(...)` nested.
    Calc(Box<Expr>),
}

/// Errors from parse + evaluate phases.
#[derive(Debug, Error, PartialEq)]
pub enum EvalError {
    /// Empty expression body.
    #[error("empty calc() body")]
    Empty,
    /// Unrecognised token in a place we expected an operand or
    /// operator.
    #[error("unexpected token in calc()")]
    UnexpectedToken,
    /// Unknown unit (e.g. `12foo`).
    #[error("unknown unit: {0}")]
    UnknownUnit(String),
    /// Division by zero.
    #[error("division by zero in calc()")]
    DivByZero,
    /// `clamp(a, b, c)` requires exactly three arguments.
    #[error("clamp() requires three arguments, got {0}")]
    ClampArity(usize),
    /// Function name we know about but don't yet evaluate (e.g.
    /// `sin`, `mod`).
    #[error("unsupported math function: {0}")]
    UnsupportedFunction(String),
    /// Parser couldn't balance brackets / saw EOF mid-expression.
    #[error("malformed calc() expression")]
    Malformed,
}

/// Top-level: resolve a math-function call (`calc`, `min`, `max`,
/// `clamp`) given its body component-values and a context, returning
/// a final px value.
pub fn evaluate_function(
    name: &str,
    body: &[ComponentValue<'_>],
    ctx: &Context,
) -> Result<f32, EvalError> {
    let expr = match name.to_ascii_lowercase().as_str() {
        "calc" => parse_expr(body)?,
        "min" => Expr::Min(parse_arg_list(body)?),
        "max" => Expr::Max(parse_arg_list(body)?),
        "clamp" => {
            let mut args = parse_arg_list(body)?;
            if args.len() != 3 {
                return Err(EvalError::ClampArity(args.len()));
            }
            let high = args.pop().unwrap();
            let val = args.pop().unwrap();
            let low = args.pop().unwrap();
            Expr::Clamp(Box::new(low), Box::new(val), Box::new(high))
        },
        other => return Err(EvalError::UnsupportedFunction(other.to_string())),
    };
    evaluate(&expr, ctx)
}

/// Evaluate an [`Expr`] tree.
pub fn evaluate(expr: &Expr, ctx: &Context) -> Result<f32, EvalError> {
    match expr {
        Expr::Number(v, u) => Ok(u.to_px(*v, ctx)),
        Expr::Add(a, b) => Ok(evaluate(a, ctx)? + evaluate(b, ctx)?),
        Expr::Sub(a, b) => Ok(evaluate(a, ctx)? - evaluate(b, ctx)?),
        Expr::Mul(a, b) => Ok(evaluate(a, ctx)? * evaluate(b, ctx)?),
        Expr::Div(a, b) => {
            let denom = evaluate(b, ctx)?;
            if denom == 0.0 {
                Err(EvalError::DivByZero)
            } else {
                Ok(evaluate(a, ctx)? / denom)
            }
        },
        Expr::Min(items) => {
            let mut acc = f32::INFINITY;
            for it in items {
                let v = evaluate(it, ctx)?;
                if v < acc {
                    acc = v;
                }
            }
            Ok(acc)
        },
        Expr::Max(items) => {
            let mut acc = f32::NEG_INFINITY;
            for it in items {
                let v = evaluate(it, ctx)?;
                if v > acc {
                    acc = v;
                }
            }
            Ok(acc)
        },
        Expr::Clamp(low, val, high) => {
            let l = evaluate(low, ctx)?;
            let v = evaluate(val, ctx)?;
            let h = evaluate(high, ctx)?;
            Ok(v.clamp(l, h))
        },
        Expr::Calc(inner) => evaluate(inner, ctx),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Parser — Pratt-style precedence climbing over component values
// ─────────────────────────────────────────────────────────────────────

/// Parse the body of a `calc(...)` into an expression tree.
pub fn parse_expr(body: &[ComponentValue<'_>]) -> Result<Expr, EvalError> {
    let mut p = ExprParser::new(body);
    let expr = p.parse_add_sub()?;
    p.expect_end()?;
    Ok(expr)
}

/// Parse a comma-separated list of expressions (for `min`/`max`/`clamp`).
fn parse_arg_list(body: &[ComponentValue<'_>]) -> Result<Vec<Expr>, EvalError> {
    if body.is_empty() {
        return Err(EvalError::Empty);
    }
    let mut args = Vec::new();
    for chunk in split_top_level_commas(body) {
        args.push(parse_expr(chunk)?);
    }
    Ok(args)
}

struct ExprParser<'a, 'i> {
    cvs: &'a [ComponentValue<'i>],
    pos: usize,
}

impl<'a, 'i> ExprParser<'a, 'i> {
    fn new(cvs: &'a [ComponentValue<'i>]) -> Self {
        Self { cvs, pos: 0 }
    }

    fn peek(&self) -> Option<&ComponentValue<'i>> {
        self.cvs.get(self.pos)
    }

    fn bump(&mut self) -> Option<&ComponentValue<'i>> {
        let r = self.cvs.get(self.pos);
        self.pos += 1;
        r
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(ComponentValue::Token(Token::Whitespace))) {
            self.pos += 1;
        }
    }

    fn expect_end(&mut self) -> Result<(), EvalError> {
        self.skip_ws();
        if self.pos < self.cvs.len() {
            Err(EvalError::Malformed)
        } else {
            Ok(())
        }
    }

    /// + / - precedence (lowest).
    fn parse_add_sub(&mut self) -> Result<Expr, EvalError> {
        self.skip_ws();
        let mut left = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            let op = match self.peek() {
                Some(ComponentValue::Token(Token::Delim('+'))) => {
                    self.bump();
                    self.skip_ws();
                    Some(true)
                },
                Some(ComponentValue::Token(Token::Delim('-'))) => {
                    self.bump();
                    self.skip_ws();
                    Some(false)
                },
                _ => None,
            };
            let Some(is_add) = op else { break };
            let right = self.parse_mul_div()?;
            left = if is_add {
                Expr::Add(Box::new(left), Box::new(right))
            } else {
                Expr::Sub(Box::new(left), Box::new(right))
            };
        }
        Ok(left)
    }

    /// * / / precedence (higher).
    fn parse_mul_div(&mut self) -> Result<Expr, EvalError> {
        self.skip_ws();
        let mut left = self.parse_atom()?;
        loop {
            self.skip_ws();
            let op = match self.peek() {
                Some(ComponentValue::Token(Token::Delim('*'))) => {
                    self.bump();
                    self.skip_ws();
                    Some(true)
                },
                Some(ComponentValue::Token(Token::Delim('/'))) => {
                    self.bump();
                    self.skip_ws();
                    Some(false)
                },
                _ => None,
            };
            let Some(is_mul) = op else { break };
            let right = self.parse_atom()?;
            left = if is_mul {
                Expr::Mul(Box::new(left), Box::new(right))
            } else {
                Expr::Div(Box::new(left), Box::new(right))
            };
        }
        Ok(left)
    }

    /// Atom: number, dimension, percentage, parens, or nested
    /// calc/min/max/clamp.
    fn parse_atom(&mut self) -> Result<Expr, EvalError> {
        self.skip_ws();
        let cv = self.bump().ok_or(EvalError::Empty)?;
        match cv {
            ComponentValue::Token(Token::Number(n)) => Ok(Expr::Number(n.value as f32, Unit::None)),
            ComponentValue::Token(Token::Dimension { value, unit }) => {
                let u =
                    Unit::parse(unit).ok_or_else(|| EvalError::UnknownUnit(unit.to_string()))?;
                Ok(Expr::Number(value.value as f32, u))
            },
            ComponentValue::Token(Token::Percentage(n)) => {
                Ok(Expr::Number(n.value as f32, Unit::Percent))
            },
            ComponentValue::Parens(body) => parse_expr(body),
            ComponentValue::Function { name, body } => {
                let lower = name.to_ascii_lowercase();
                match lower.as_str() {
                    "calc" => {
                        let inner = parse_expr(body)?;
                        Ok(Expr::Calc(Box::new(inner)))
                    },
                    "min" => Ok(Expr::Min(parse_arg_list(body)?)),
                    "max" => Ok(Expr::Max(parse_arg_list(body)?)),
                    "clamp" => {
                        let mut args = parse_arg_list(body)?;
                        if args.len() != 3 {
                            return Err(EvalError::ClampArity(args.len()));
                        }
                        let high = args.pop().unwrap();
                        let val = args.pop().unwrap();
                        let low = args.pop().unwrap();
                        Ok(Expr::Clamp(Box::new(low), Box::new(val), Box::new(high)))
                    },
                    _ => Err(EvalError::UnsupportedFunction(lower)),
                }
            },
            // Unary +/- on the next atom.
            ComponentValue::Token(Token::Delim('+')) => self.parse_atom(),
            ComponentValue::Token(Token::Delim('-')) => {
                let inner = self.parse_atom()?;
                Ok(Expr::Sub(Box::new(Expr::Number(0.0, Unit::None)), Box::new(inner)))
            },
            _ => Err(EvalError::UnexpectedToken),
        }
    }
}

fn split_top_level_commas<'a, 'i>(cvs: &'a [ComponentValue<'i>]) -> Vec<&'a [ComponentValue<'i>]> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, cv) in cvs.iter().enumerate() {
        if matches!(cv, ComponentValue::Token(Token::Comma)) {
            out.push(&cvs[start..i]);
            start = i + 1;
        }
    }
    out.push(&cvs[start..]);
    out
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::parser::{parse_stylesheet, Rule};

    /// Helper: pull the body of `calc(...)` (or any function) out of
    /// the first declaration of the first rule in `css`.
    fn func_body<'a>(css: &'a str, name: &str) -> Vec<ComponentValue<'a>> {
        let ss = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let r = match &ss.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        let val = &r.declarations[0].value;
        for cv in val {
            if let ComponentValue::Function { name: n, body } = cv {
                if n.eq_ignore_ascii_case(name) {
                    return body.clone();
                }
            }
        }
        panic!("no {name}() found in: {val:?}");
    }

    fn ctx() -> Context {
        Context {
            parent_px: 600.0,
            font_size_px: 16.0,
            root_font_size_px: 16.0,
            viewport_w_px: 1024.0,
            viewport_h_px: 768.0,
        }
    }

    #[test]
    fn unit_conversions() {
        let c = ctx();
        assert!((Unit::Pt.to_px(12.0, &c) - 16.0).abs() < 1e-3); // 12pt = 16px
        assert!((Unit::In.to_px(1.0, &c) - 96.0).abs() < 1e-3);
        assert!((Unit::Cm.to_px(2.54, &c) - 96.0).abs() < 1e-3);
        assert_eq!(Unit::Em.to_px(2.0, &c), 32.0);
        assert_eq!(Unit::Rem.to_px(1.5, &c), 24.0);
        assert_eq!(Unit::Vw.to_px(50.0, &c), 512.0);
        assert_eq!(Unit::Percent.to_px(25.0, &c), 150.0);
    }

    #[test]
    fn calc_simple_subtract() {
        let body = func_body("p { width: calc(100% - 20px); }", "calc");
        let v = evaluate_function("calc", &body, &ctx()).unwrap();
        // 100% of 600 = 600, - 20 = 580
        assert!((v - 580.0).abs() < 1e-3);
    }

    #[test]
    fn calc_nested_parens_with_precedence() {
        // calc(2 * (10px + 5px)) = 30px
        let body = func_body("p { width: calc(2 * (10px + 5px)); }", "calc");
        let v = evaluate_function("calc", &body, &ctx()).unwrap();
        assert!((v - 30.0).abs() < 1e-3);
    }

    #[test]
    fn calc_mul_div_precedence_over_add() {
        // calc(10 + 2 * 3px) = 10 + 6 = 16
        let body = func_body("p { width: calc(10 + 2 * 3px); }", "calc");
        let v = evaluate_function("calc", &body, &ctx()).unwrap();
        assert!((v - 16.0).abs() < 1e-3);
    }

    #[test]
    fn calc_em_resolves_against_font_size() {
        // calc(2em + 4px) = 32 + 4 = 36
        let body = func_body("p { width: calc(2em + 4px); }", "calc");
        let v = evaluate_function("calc", &body, &ctx()).unwrap();
        assert!((v - 36.0).abs() < 1e-3);
    }

    #[test]
    fn min_picks_smallest() {
        let body = func_body("p { width: min(50px, 100px, 25px); }", "min");
        let v = evaluate_function("min", &body, &ctx()).unwrap();
        assert!((v - 25.0).abs() < 1e-3);
    }

    #[test]
    fn max_picks_largest() {
        let body = func_body("p { width: max(50px, 100px, 25px); }", "max");
        let v = evaluate_function("max", &body, &ctx()).unwrap();
        assert!((v - 100.0).abs() < 1e-3);
    }

    #[test]
    fn clamp_three_args() {
        // clamp(50px, 100%, 800px) where parent is 600px: 50 < 600 < 800 → 600
        let body = func_body("p { width: clamp(50px, 100%, 800px); }", "clamp");
        let v = evaluate_function("clamp", &body, &ctx()).unwrap();
        assert!((v - 600.0).abs() < 1e-3);
    }

    #[test]
    fn clamp_lower_bound_engaged() {
        let body = func_body("p { width: clamp(700px, 100%, 800px); }", "clamp");
        let v = evaluate_function("clamp", &body, &ctx()).unwrap();
        assert!((v - 700.0).abs() < 1e-3);
    }

    #[test]
    fn clamp_upper_bound_engaged() {
        let body = func_body("p { width: clamp(50px, 100%, 400px); }", "clamp");
        let v = evaluate_function("clamp", &body, &ctx()).unwrap();
        assert!((v - 400.0).abs() < 1e-3);
    }

    #[test]
    fn nested_calc_in_calc() {
        let body = func_body("p { width: calc(10px + calc(5px + 5px)); }", "calc");
        let v = evaluate_function("calc", &body, &ctx()).unwrap();
        assert!((v - 20.0).abs() < 1e-3);
    }

    #[test]
    fn min_in_calc() {
        let body = func_body("p { width: calc(10px + min(20px, 30px)); }", "calc");
        let v = evaluate_function("calc", &body, &ctx()).unwrap();
        assert!((v - 30.0).abs() < 1e-3);
    }

    #[test]
    fn unary_minus() {
        let body = func_body("p { width: calc(-10px + 50px); }", "calc");
        let v = evaluate_function("calc", &body, &ctx()).unwrap();
        assert!((v - 40.0).abs() < 1e-3);
    }

    #[test]
    fn division_by_zero_errors() {
        let body = func_body("p { width: calc(10px / 0); }", "calc");
        let res = evaluate_function("calc", &body, &ctx());
        assert!(matches!(res, Err(EvalError::DivByZero)));
    }

    #[test]
    fn unknown_unit_errors() {
        let body = func_body("p { width: calc(12foo + 1px); }", "calc");
        let res = evaluate_function("calc", &body, &ctx());
        assert!(matches!(res, Err(EvalError::UnknownUnit(_))));
    }

    #[test]
    fn unsupported_math_function() {
        let body = func_body("p { width: calc(sin(1deg)); }", "calc");
        let res = evaluate_function("calc", &body, &ctx());
        assert!(matches!(res, Err(EvalError::UnsupportedFunction(_))));
    }
}
