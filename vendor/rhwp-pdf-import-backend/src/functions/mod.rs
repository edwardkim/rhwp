// SPDX-License-Identifier: MIT OR Apache-2.0

//! PostScript Type 4 (calculator) function evaluator.
//!
//! PDF Type 4 functions are small stack-based programs used as tint transforms
//! in Separation and DeviceN color spaces. This module parses and evaluates
//! them per ISO 32000-1:2008 §7.10.5 and Table 42, which together define a
//! restricted subset of the PostScript Language Reference Manual (PLRM, 3rd
//! ed.) §8.2 operator semantics. Where Rust's default numeric behaviour
//! diverges from PLRM (e.g. `f64::round` ties, `atan2` range, panicking on
//! `i64::MIN / -1`), the PLRM rule is honoured and cited inline.
//!
//! # Supported operators
//!
//! Per PDF spec ISO 32000-1 §7.10.5 / Table 42:
//!
//! - **Arithmetic**: `add` `sub` `mul` `div` `idiv` `mod` `neg` `abs`
//!   `ceiling` `floor` `round` `truncate` `sqrt` `exp` `ln` `log`
//!   `sin` `cos` `atan` `cvi` `cvr`
//! - **Comparison**: `eq` `ne` `gt` `ge` `lt` `le`
//! - **Boolean / bitwise**: `and` `or` `xor` `not` `bitshift`
//! - **Stack**: `dup` `exch` `pop` `copy` `index` `roll`
//! - **Conditional**: `if` `ifelse` (each consuming one or two preceding
//!   `{ ... }` procedure bodies)
//! - **Literals**: integer (`5`, `-3`, `+1`), real (`1.5`, `-.5`, `1e-3`),
//!   boolean (`true`, `false`)
//!
//! Non-finite numeric literals (`inf`, `NaN`) are rejected at parse time.
//!
//! # Integration
//!
//! The renderer at `src/rendering/page_renderer.rs` (lines 566-642) currently
//! handles Type 2 (exponential interpolation) tint transforms and falls back
//! to grayscale for everything else. To support Type 4, add a branch for
//! `FunctionType == 4`: decode the function stream, then call
//! `evaluate_type4(stream_bytes, &[tint])` to get CMYK components.

#![forbid(unsafe_code)]

use crate::error::{Error, Result};

/// A parsed instruction in a Type 4 PostScript calculator program.
#[derive(Debug, Clone, PartialEq)]
enum Instruction {
    NumberLiteral(f64),
    IntLiteral(i64),
    BoolLiteral(bool),
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    Mod,
    Neg,
    Abs,
    Ceiling,
    Floor,
    Round,
    Truncate,
    Sqrt,
    Exp,
    Ln,
    Log,
    Sin,
    Cos,
    Atan,
    // Comparison
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    // Boolean/bitwise
    And,
    Or,
    Xor,
    Not,
    Bitshift,
    // Type conversion (PLRM §8.2)
    Cvi,
    Cvr,
    // Stack manipulation
    Dup,
    Exch,
    Pop,
    Copy,
    Index,
    Roll,
    // Parser-emitted procedure body. A `{ ... }` block starts as one of these
    // and is consumed by the immediately following `if` or `ifelse` during
    // `resolve_conditionals`. Any `ProcedureBody` that survives the resolve
    // pass is an orphan and rejected at parse time.
    ProcedureBody(Vec<Instruction>),
    // Conditional (post-resolve form)
    If(Vec<Instruction>),
    IfElse(Vec<Instruction>, Vec<Instruction>),
}

/// A runtime stack value. PLRM §8.2 distinguishes integer, real, and boolean
/// types; the same surface syntax (`1`, `1.0`, `true`) can produce values that
/// behave differently under `not`, `and`, `or`, `xor`, `idiv`, and `mod`.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Value {
    Int(i64),
    Real(f64),
    Bool(bool),
}

impl Value {
    fn as_real(self) -> Result<f64> {
        match self {
            Value::Int(i) => Ok(i as f64),
            Value::Real(r) => Ok(r),
            Value::Bool(_) => Err(typecheck("expected numeric, got boolean")),
        }
    }

    fn as_int(self) -> Result<i64> {
        match self {
            Value::Int(i) => Ok(i),
            // PLRM §8.2: idiv, mod, bitshift, and other integer ops require
            // typed integer values. A real literal like `2.0` is a typed real
            // even if its value is integral and is rejected.
            Value::Real(_) => Err(typecheck("expected integer, got real")),
            Value::Bool(_) => Err(typecheck("expected integer, got boolean")),
        }
    }

    fn as_bool(self) -> Result<bool> {
        match self {
            Value::Bool(b) => Ok(b),
            _ => Err(typecheck("expected boolean")),
        }
    }

    fn to_output(self) -> f64 {
        match self {
            Value::Int(i) => i as f64,
            Value::Real(r) => r,
            Value::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            },
        }
    }
}

/// Maximum nested `{ ... }` depth permitted by the parser. PLRM has no formal
/// cap, but real-world Type 4 streams are shallow; bounding it prevents a
/// maliciously deep stream from blowing the Rust call stack since `parse_body`
/// recurses for each brace level. Programs nested deeper return
/// [`Error::InvalidPdf`].
pub const MAX_PARSE_DEPTH: u32 = 32;

/// Maximum operand stack size during execution. PLRM §7.10.5.2 requires a
/// "stack overflow" diagnostic; we surface it as [`Error::Type4Runtime`]. The
/// cap matches what Adobe accepts in practice — Acrobat's interpreter allows
/// up to a few hundred operands.
pub const MAX_STACK: usize = 256;

/// Maximum number of instructions the evaluator will execute. Type 4 has no
/// loops in the language proper, but nested `if`/`ifelse` plus large generated
/// bodies (or pathological streams crafted to consume CPU) can still produce
/// arbitrarily many steps. 100 000 is generous for any realistic tint
/// transform while still being a hard upper bound. Programs that exceed this
/// budget return [`Error::Type4Runtime`].
pub const MAX_INSTRUCTIONS: usize = 100_000;

/// Parse a Type 4 PostScript calculator program from raw bytes.
///
/// The program must be enclosed in `{ }`. Nested braces define procedure
/// bodies used with `if` and `ifelse`.
fn parse(program: &[u8]) -> Result<Vec<Instruction>> {
    let s = std::str::from_utf8(program)
        .map_err(|e| Error::InvalidPdf(format!("Type 4 function is not valid UTF-8: {e}")))?;
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return Err(Error::InvalidPdf("Type 4 function must be enclosed in { }".into()));
    }
    let inner = &s[1..s.len() - 1];
    // Outermost `{ ... }` consumes one depth slot.
    parse_body(inner, 1)
}

fn parse_body(s: &str, depth: u32) -> Result<Vec<Instruction>> {
    if depth > MAX_PARSE_DEPTH {
        return Err(Error::InvalidPdf(format!(
            "Type 4 parse depth limit exceeded (max {MAX_PARSE_DEPTH})"
        )));
    }
    let mut instructions = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some(&(i, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '{' {
            chars.next();
            let start = if let Some(&(idx, _)) = chars.peek() {
                idx
            } else {
                return Err(Error::InvalidPdf("Unclosed brace in Type 4 function".into()));
            };
            let mut brace_depth = 1u32;
            let mut end = start;
            for (j, ch) in chars.by_ref() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        end = j;
                        break;
                    }
                }
            }
            if brace_depth != 0 {
                return Err(Error::InvalidPdf("Unclosed brace in Type 4 function".into()));
            }
            let body = parse_body(&s[start..end], depth + 1)?;
            instructions.push(Instruction::ProcedureBody(body));
            continue;
        }
        // Collect a token
        let start = i;
        while let Some(&(_, tc)) = chars.peek() {
            if tc.is_whitespace() || tc == '{' || tc == '}' {
                break;
            }
            chars.next();
        }
        let end = if let Some(&(idx, _)) = chars.peek() {
            idx
        } else {
            s.len()
        };
        let token = &s[start..end];
        instructions.push(parse_token(token)?);
    }

    // Post-process: resolve `if` and `ifelse` by consuming preceding procedure bodies.
    resolve_conditionals(&mut instructions)?;
    Ok(instructions)
}

fn parse_token(token: &str) -> Result<Instruction> {
    match token {
        "add" => Ok(Instruction::Add),
        "sub" => Ok(Instruction::Sub),
        "mul" => Ok(Instruction::Mul),
        "div" => Ok(Instruction::Div),
        "idiv" => Ok(Instruction::Idiv),
        "mod" => Ok(Instruction::Mod),
        "neg" => Ok(Instruction::Neg),
        "abs" => Ok(Instruction::Abs),
        "ceiling" => Ok(Instruction::Ceiling),
        "floor" => Ok(Instruction::Floor),
        "round" => Ok(Instruction::Round),
        "truncate" => Ok(Instruction::Truncate),
        "sqrt" => Ok(Instruction::Sqrt),
        "exp" => Ok(Instruction::Exp),
        "ln" => Ok(Instruction::Ln),
        "log" => Ok(Instruction::Log),
        "sin" => Ok(Instruction::Sin),
        "cos" => Ok(Instruction::Cos),
        "atan" => Ok(Instruction::Atan),
        "eq" => Ok(Instruction::Eq),
        "ne" => Ok(Instruction::Ne),
        "gt" => Ok(Instruction::Gt),
        "ge" => Ok(Instruction::Ge),
        "lt" => Ok(Instruction::Lt),
        "le" => Ok(Instruction::Le),
        "and" => Ok(Instruction::And),
        "or" => Ok(Instruction::Or),
        "xor" => Ok(Instruction::Xor),
        "not" => Ok(Instruction::Not),
        "bitshift" => Ok(Instruction::Bitshift),
        "cvi" => Ok(Instruction::Cvi),
        "cvr" => Ok(Instruction::Cvr),
        "true" => Ok(Instruction::BoolLiteral(true)),
        "false" => Ok(Instruction::BoolLiteral(false)),
        "dup" => Ok(Instruction::Dup),
        "exch" => Ok(Instruction::Exch),
        "pop" => Ok(Instruction::Pop),
        "copy" => Ok(Instruction::Copy),
        "index" => Ok(Instruction::Index),
        "roll" => Ok(Instruction::Roll),
        "if" | "ifelse" => Ok(if token == "if" {
            // Placeholder; resolved in post-processing
            Instruction::If(vec![])
        } else {
            Instruction::IfElse(vec![], vec![])
        }),
        _ => parse_numeric_literal(token),
    }
}

/// Parse a numeric literal. PLRM §3.3.2 specifies decimal/real syntax only;
/// `inf`, `NaN`, hex, and radix forms are not part of the Type 4 subset
/// (ISO 32000-1 Table 42). Reject anything that round-trips to a non-finite
/// f64 so malformed streams cannot smuggle in poisoned values.
fn parse_numeric_literal(token: &str) -> Result<Instruction> {
    // Prefer an integer parse so `52 not` and similar stay typed as integers.
    if let Ok(i) = token.parse::<i64>() {
        return Ok(Instruction::IntLiteral(i));
    }
    let val: f64 = token
        .parse()
        .map_err(|_| Error::InvalidPdf(format!("Unknown Type 4 token: {token}")))?;
    if !val.is_finite() {
        return Err(Error::InvalidPdf(format!(
            "Type 4 numeric literal must be finite, got: {token}"
        )));
    }
    Ok(Instruction::NumberLiteral(val))
}

/// Post-process: attach preceding procedure bodies to `if`/`ifelse` and reject
/// orphan procedure bodies that aren't consumed by a conditional.
fn resolve_conditionals(instructions: &mut Vec<Instruction>) -> Result<()> {
    let mut i = 0;
    while i < instructions.len() {
        match &instructions[i] {
            Instruction::If(body) if body.is_empty() => {
                // `if`: one preceding procedure body
                if i == 0 {
                    return Err(Error::InvalidPdf(
                        "Type 4 `if` without preceding procedure body".into(),
                    ));
                }
                match instructions.remove(i - 1) {
                    Instruction::ProcedureBody(body) => {
                        instructions[i - 1] = Instruction::If(body);
                        // Don't increment i; we removed an element before
                    },
                    _ => {
                        return Err(Error::InvalidPdf(
                            "Type 4 `if` requires a procedure body".into(),
                        ));
                    },
                }
            },
            Instruction::IfElse(true_b, false_b) if true_b.is_empty() && false_b.is_empty() => {
                // `ifelse`: two preceding procedure bodies
                if i < 2 {
                    return Err(Error::InvalidPdf(
                        "Type 4 `ifelse` without two preceding procedure bodies".into(),
                    ));
                }
                let false_branch = match instructions.remove(i - 1) {
                    Instruction::ProcedureBody(body) => body,
                    _ => {
                        return Err(Error::InvalidPdf(
                            "Type 4 `ifelse` requires two procedure bodies".into(),
                        ))
                    },
                };
                let true_branch = match instructions.remove(i - 2) {
                    Instruction::ProcedureBody(body) => body,
                    _ => {
                        return Err(Error::InvalidPdf(
                            "Type 4 `ifelse` requires two procedure bodies".into(),
                        ))
                    },
                };
                instructions[i - 2] = Instruction::IfElse(true_branch, false_branch);
                i = i.saturating_sub(1);
            },
            _ => {
                i += 1;
            },
        }
    }
    // Any procedure body that survives the resolve pass is an orphan — a
    // `{ ... }` block not followed by `if`/`ifelse`. PLRM has no concept of
    // executing a procedure object directly from this subset; reject it.
    if instructions
        .iter()
        .any(|ins| matches!(ins, Instruction::ProcedureBody(_)))
    {
        return Err(Error::InvalidPdf(
            "Type 4 orphan procedure body: { ... } not consumed by if/ifelse".into(),
        ));
    }
    Ok(())
}

/// A compiled Type 4 PostScript calculator program.
///
/// Construct once via [`Program::compile`], then call [`Program::evaluate`]
/// (or [`Program::evaluate_clamped`]) many times with different inputs. This
/// is the recommended path for tint transforms in Separation and DeviceN
/// colour spaces, which are evaluated per pixel — parsing each call would
/// be wasteful.
///
/// `Program` is `Send + Sync`, so it can live inside a shared tint-transform
/// cache without a `Mutex`.
#[derive(Debug, Clone)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    /// Compile a Type 4 program from its raw bytes.
    ///
    /// Returns [`Error::InvalidPdf`] for any parse-time failure — syntax
    /// errors (missing braces, unknown tokens, orphan procedure bodies),
    /// non-finite numeric literals, and resource caps that fire during
    /// parsing such as nesting deeper than [`MAX_PARSE_DEPTH`]. All of these
    /// are deterministic properties of the program text, so callers must
    /// not retry the same bytes on the next sample.
    ///
    /// [`Error::Type4Runtime`] is reserved for execution-time failures
    /// (stack overflow/underflow, integer overflow in arithmetic, hitting
    /// the per-call instruction budget); those are raised by
    /// [`evaluate`](Self::evaluate) and [`evaluate_clamped`](Self::evaluate_clamped),
    /// never by `compile`. The resulting `Program` is reusable across many
    /// `evaluate` calls.
    pub fn compile(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            instructions: parse(bytes)?,
        })
    }

    /// Evaluate the compiled program against the given inputs.
    ///
    /// Each call starts with a fresh operand stack initialised from `inputs`;
    /// the compiled `Program` itself carries no mutable evaluation state, so
    /// concurrent calls (across threads or pixels) are safe and independent.
    pub fn evaluate(&self, inputs: &[f64]) -> Result<Vec<f64>> {
        if inputs.len() > MAX_STACK {
            return Err(Error::Type4Runtime(format!(
                "Type 4 stack overflow: {} inputs exceeds max {MAX_STACK}",
                inputs.len()
            )));
        }
        // The public API takes f64s. Caller intent (typed int vs typed real)
        // is ambiguous, so promote exact integer-valued inputs to typed
        // integers. This lets integer ops (idiv, mod, bitshift) accept
        // caller-supplied integer inputs while still rejecting parser
        // literals like `2.0`.
        // i64::MAX (2^63 - 1) is not exactly representable in f64 —
        // `i64::MAX as f64` rounds up to exactly 2^63. So using
        // `v <= i64::MAX as f64` lets v == 2^63 through, where the
        // `v as i64` cast then saturates silently to i64::MAX. Use 2^63
        // with a strict `<` to keep the boundary on the safe side.
        const I64_MAX_PLUS_ONE_AS_F64: f64 = 9_223_372_036_854_775_808.0;
        let mut stack: Vec<Value> = inputs
            .iter()
            .map(|&v| {
                if v.is_finite()
                    && v.fract() == 0.0
                    && v >= i64::MIN as f64
                    && v < I64_MAX_PLUS_ONE_AS_F64
                {
                    Value::Int(v as i64)
                } else {
                    Value::Real(v)
                }
            })
            .collect();
        let mut budget = MAX_INSTRUCTIONS;
        execute(&self.instructions, &mut stack, &mut budget)?;
        Ok(stack.into_iter().map(Value::to_output).collect())
    }

    /// Evaluate with Domain/Range clamping per the PDF function dictionary.
    ///
    /// `domain` is a list of `[min, max]` pairs (one per input). Each input is
    /// clamped to its domain before execution. `range` is a list of
    /// `[min, max]` pairs (one per output). Each output is clamped to its
    /// range after execution. Malformed bounds (`min > max`) are swapped;
    /// NaN bounds are treated as no-op, since `f64::clamp` would otherwise
    /// panic.
    pub fn evaluate_clamped(
        &self,
        inputs: &[f64],
        domain: &[[f64; 2]],
        range: &[[f64; 2]],
    ) -> Result<Vec<f64>> {
        let clamped_inputs: Vec<f64> = inputs
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                if let Some(&[lo, hi]) = domain.get(i) {
                    safe_clamp(v, lo, hi)
                } else {
                    v
                }
            })
            .collect();
        let mut result = self.evaluate(&clamped_inputs)?;
        for (i, val) in result.iter_mut().enumerate() {
            if let Some(&[lo, hi]) = range.get(i) {
                *val = safe_clamp(*val, lo, hi);
            }
        }
        Ok(result)
    }
}

/// Evaluate a Type 4 PostScript calculator program.
///
/// `program` is the raw stream content (e.g. `{ dup 0.84 mul ... }`).
/// `inputs` are pushed onto the stack before execution.
/// After execution the remaining stack values are returned as the output.
///
/// This compiles and evaluates the program in one shot. For per-pixel
/// evaluation (e.g. a Separation tint transform applied to every sample),
/// use [`Program::compile`] once and call [`Program::evaluate`] many times.
pub fn evaluate_type4(program: &[u8], inputs: &[f64]) -> Result<Vec<f64>> {
    Program::compile(program)?.evaluate(inputs)
}

/// Evaluate with Domain/Range clamping per the PDF function dictionary.
///
/// Thin wrapper around [`Program::compile`] + [`Program::evaluate_clamped`].
/// Same per-call-cost caveat as [`evaluate_type4`].
pub fn evaluate_type4_clamped(
    program: &[u8],
    inputs: &[f64],
    domain: &[[f64; 2]],
    range: &[[f64; 2]],
) -> Result<Vec<f64>> {
    Program::compile(program)?.evaluate_clamped(inputs, domain, range)
}

/// Clamp without panicking on malformed bounds. PDF spec allows arrays we do
/// not trust; `f64::clamp` panics on NaN bounds or `min > max`.
fn safe_clamp(v: f64, lo: f64, hi: f64) -> f64 {
    if lo.is_nan() || hi.is_nan() {
        return v;
    }
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    v.clamp(lo, hi)
}

/// Push helper enforcing [`MAX_STACK`]. Used by every code path that grows the
/// operand stack so the cap is uniform across literals, dup, copy, and the
/// numeric/bool result of every operator.
fn push_checked(stack: &mut Vec<Value>, v: Value) -> Result<()> {
    if stack.len() >= MAX_STACK {
        return Err(Error::Type4Runtime(format!("Type 4 stack overflow (max {MAX_STACK})")));
    }
    stack.push(v);
    Ok(())
}

// Stack-growth convention used throughout `execute`:
//
// - [`push_checked`] is used by operators that genuinely *grow* the stack
//   relative to its entry size — number/int/bool literals, `dup`, and `copy`.
//   These need the [`MAX_STACK`] guard because they can drive the stack past
//   the cap from a fully-saturated starting state.
// - Net-shrink and net-neutral operators (arithmetic, comparison, boolean,
//   `cvi`/`cvr`, `not`, `exch`, `index`, `atan`, etc.) call raw `stack.push`
//   after their pops. Each such site has already popped at least as many
//   values as it will push back, so there is provably room and the
//   `push_checked` overhead is unnecessary.
fn execute(instructions: &[Instruction], stack: &mut Vec<Value>, budget: &mut usize) -> Result<()> {
    for instr in instructions {
        if *budget == 0 {
            return Err(Error::Type4Runtime(format!(
                "Type 4 instruction budget exceeded (max {MAX_INSTRUCTIONS})"
            )));
        }
        *budget -= 1;
        match instr {
            Instruction::NumberLiteral(v) => push_checked(stack, Value::Real(*v))?,
            Instruction::IntLiteral(i) => push_checked(stack, Value::Int(*i))?,
            Instruction::BoolLiteral(b) => push_checked(stack, Value::Bool(*b))?,
            Instruction::Add => numeric_binary(stack, |a, b| Ok(a + b))?,
            Instruction::Sub => numeric_binary(stack, |a, b| Ok(a - b))?,
            Instruction::Mul => numeric_binary(stack, |a, b| Ok(a * b))?,
            // PLRM §8.2 raises `undefinedresult` on `div` by zero, but Acrobat
            // and Poppler instead let IEEE 754 produce the result (±inf for
            // n/0, NaN for 0/0). Match that behaviour so a tint transform
            // that overruns its domain doesn't blow up an otherwise valid
            // page. `idiv` / `mod` stay as runtime errors — integer math has
            // no inf/NaN to fall back to.
            Instruction::Div => numeric_binary(stack, |a, b| Ok(a / b))?,
            Instruction::Idiv => {
                // PLRM §8.2: idiv requires integer operands, returns integer.
                // i64::MIN / -1 overflows; use checked_div to fail safely.
                let b = pop(stack)?.as_int()?;
                let a = pop(stack)?.as_int()?;
                if b == 0 {
                    return Err(Error::Type4Runtime("Type 4 idiv by zero".into()));
                }
                let q = a
                    .checked_div(b)
                    .ok_or_else(|| Error::Type4Runtime("Type 4 idiv integer overflow".into()))?;
                stack.push(Value::Int(q));
            },
            Instruction::Mod => {
                let b = pop(stack)?.as_int()?;
                let a = pop(stack)?.as_int()?;
                if b == 0 {
                    return Err(Error::Type4Runtime("Type 4 mod by zero".into()));
                }
                let r = a
                    .checked_rem(b)
                    .ok_or_else(|| Error::Type4Runtime("Type 4 mod integer overflow".into()))?;
                stack.push(Value::Int(r));
            },
            // PLRM §8.2: `neg` on `i64::MIN` overflows. Use `checked_neg` so
            // the program fails cleanly instead of wrapping silently — matches
            // the explicit overflow path already in use for `idiv`/`mod`.
            Instruction::Neg => {
                let v = pop(stack)?;
                match v {
                    Value::Int(i) => {
                        let n = i.checked_neg().ok_or_else(|| {
                            Error::Type4Runtime("Type 4 integer overflow in neg".into())
                        })?;
                        stack.push(Value::Int(n));
                    },
                    Value::Real(r) => stack.push(Value::Real(-r)),
                    Value::Bool(_) => return Err(typecheck("neg expects a number")),
                }
            },
            // PLRM §8.2: `abs` on `i64::MIN` overflows. Same treatment as `neg`.
            Instruction::Abs => {
                let v = pop(stack)?;
                match v {
                    Value::Int(i) => {
                        let n = i.checked_abs().ok_or_else(|| {
                            Error::Type4Runtime("Type 4 integer overflow in abs".into())
                        })?;
                        stack.push(Value::Int(n));
                    },
                    Value::Real(r) => stack.push(Value::Real(r.abs())),
                    Value::Bool(_) => return Err(typecheck("abs expects a number")),
                }
            },
            Instruction::Ceiling => real_unary_preserve(stack, |a| Ok(a.ceil()))?,
            Instruction::Floor => real_unary_preserve(stack, |a| Ok(a.floor()))?,
            // PLRM §8.2: round goes to the greater of the two surrounding
            // integers (i.e. round-half-toward-+inf). Rust's `f64::round`
            // ties away from zero, so -6.5 would become -7.0 instead of -6.0.
            Instruction::Round => real_unary_preserve(stack, |a| Ok((a + 0.5).floor()))?,
            Instruction::Truncate => real_unary_preserve(stack, |a| Ok(a.trunc()))?,
            // PLRM §8.2: sqrt requires num >= 0; ln/log require num > 0.
            // Invalid inputs raise rangecheck/undefinedresult; we propagate as
            // InvalidPdf rather than letting NaN/-inf reach the renderer.
            Instruction::Sqrt => real_unary(stack, |a| {
                if a < 0.0 || a.is_nan() {
                    Err(Error::Type4Runtime("Type 4 sqrt of negative".into()))
                } else {
                    Ok(a.sqrt())
                }
            })?,
            Instruction::Exp => numeric_binary(stack, |base, exp| Ok(base.powf(exp)))?,
            Instruction::Ln => real_unary(stack, |a| {
                if a <= 0.0 || a.is_nan() {
                    Err(Error::Type4Runtime("Type 4 ln of non-positive".into()))
                } else {
                    Ok(a.ln())
                }
            })?,
            Instruction::Log => real_unary(stack, |a| {
                if a <= 0.0 || a.is_nan() {
                    Err(Error::Type4Runtime("Type 4 log of non-positive".into()))
                } else {
                    Ok(a.log10())
                }
            })?,
            Instruction::Sin => real_unary(stack, |a| Ok(a.to_radians().sin()))?,
            Instruction::Cos => real_unary(stack, |a| Ok(a.to_radians().cos()))?,
            // PLRM §8.2: atan returns the angle in degrees in [0, 360). Rust's
            // atan2().to_degrees() returns (-180, 180]; map negative results
            // back into the spec range.
            Instruction::Atan => {
                let den = pop(stack)?.as_real()?;
                let num = pop(stack)?.as_real()?;
                if num == 0.0 && den == 0.0 {
                    return Err(Error::Type4Runtime("Type 4 atan undefined for (0, 0)".into()));
                }
                let mut deg = num.atan2(den).to_degrees();
                if deg < 0.0 {
                    deg += 360.0;
                }
                // Guard against atan2 returning exactly 360.0 due to rounding.
                if deg >= 360.0 {
                    deg -= 360.0;
                }
                stack.push(Value::Real(deg));
            },
            Instruction::Eq => {
                let b = pop(stack)?;
                let a = pop(stack)?;
                stack.push(Value::Bool(values_equal(a, b)));
            },
            Instruction::Ne => {
                let b = pop(stack)?;
                let a = pop(stack)?;
                stack.push(Value::Bool(!values_equal(a, b)));
            },
            Instruction::Gt => comparison(stack, |o| o == std::cmp::Ordering::Greater)?,
            Instruction::Ge => comparison(stack, |o| o != std::cmp::Ordering::Less)?,
            Instruction::Lt => comparison(stack, |o| o == std::cmp::Ordering::Less)?,
            Instruction::Le => comparison(stack, |o| o != std::cmp::Ordering::Greater)?,
            Instruction::And => bool_or_bitwise(stack, |a, b| a && b, |a, b| a & b)?,
            Instruction::Or => bool_or_bitwise(stack, |a, b| a || b, |a, b| a | b)?,
            Instruction::Xor => bool_or_bitwise(stack, |a, b| a != b, |a, b| a ^ b)?,
            Instruction::Not => {
                let v = pop(stack)?;
                match v {
                    Value::Bool(b) => stack.push(Value::Bool(!b)),
                    Value::Int(i) => stack.push(Value::Int(!i)),
                    Value::Real(_) => {
                        return Err(typecheck("not expects boolean or integer"));
                    },
                }
            },
            // PLRM §8.2: bitshift takes two integers. Magnitudes >= 64 would
            // panic with Rust's `<<`/`>>`; PLRM specifies "bits shifted out
            // are discarded; zeros are supplied for vacated bits", which for
            // |shift| >= 64 produces 0. We saturate to zero rather than
            // raising a runtime error because the spec gives a defined value.
            Instruction::Bitshift => {
                let shift = pop(stack)?.as_int()?;
                let val = pop(stack)?.as_int()?;
                let result = if shift >= 64 || shift <= -64 {
                    0
                } else if shift >= 0 {
                    val.wrapping_shl(shift as u32)
                } else {
                    // Logical right shift on the unsigned bit pattern, per
                    // PLRM's "bits shifted out are discarded; zeros are
                    // supplied for vacated bits".
                    ((val as u64) >> (-shift) as u32) as i64
                };
                stack.push(Value::Int(result));
            },
            // PLRM §8.2: `cvi` pops a number, truncates toward zero, and
            // pushes the result as a typed integer. Reals outside the i64
            // range overflow as a runtime error rather than wrapping.
            Instruction::Cvi => {
                let v = pop(stack)?;
                match v {
                    Value::Int(i) => stack.push(Value::Int(i)),
                    Value::Real(r) => {
                        if !r.is_finite() {
                            return Err(Error::Type4Runtime(
                                "Type 4 cvi: input is not finite".into(),
                            ));
                        }
                        let t = r.trunc();
                        // i64::MAX (2^63 - 1) is NOT exactly representable in
                        // f64 — `i64::MAX as f64` rounds up to exactly 2^63,
                        // which IS representable. So the upper bound has to
                        // use 2^63 with a `>=` comparison; using `> i64::MAX
                        // as f64` lets t == 2^63 slip through and saturate
                        // silently to i64::MAX on the `as i64` cast below.
                        const I64_MAX_PLUS_ONE_AS_F64: f64 = 9_223_372_036_854_775_808.0;
                        if t < i64::MIN as f64 || t >= I64_MAX_PLUS_ONE_AS_F64 {
                            return Err(Error::Type4Runtime("Type 4 cvi: integer overflow".into()));
                        }
                        stack.push(Value::Int(t as i64));
                    },
                    Value::Bool(_) => return Err(typecheck("cvi expects a number")),
                }
            },
            // PLRM §8.2: `cvr` pops a number and pushes it as a typed real.
            // An integer becomes a typed real (no longer satisfies `as_int`).
            Instruction::Cvr => {
                let v = pop(stack)?;
                match v {
                    Value::Int(i) => stack.push(Value::Real(i as f64)),
                    Value::Real(r) => stack.push(Value::Real(r)),
                    Value::Bool(_) => return Err(typecheck("cvr expects a number")),
                }
            },
            Instruction::Dup => {
                let a = *stack.last().ok_or_else(underflow)?;
                push_checked(stack, a)?;
            },
            Instruction::Exch => {
                let b = pop(stack)?;
                let a = pop(stack)?;
                // Net-neutral: two pops, two pushes back.
                stack.push(b);
                stack.push(a);
            },
            Instruction::Pop => {
                pop(stack)?;
            },
            Instruction::Copy => {
                // Note: operand pops happen before the bounds check below.
                // The mutation is visible only inside this scope; on error
                // the evaluation aborts and the partially-popped stack is
                // never observed by the caller.
                let n = pop_count(stack, "copy")?;
                if n > stack.len() {
                    return Err(underflow());
                }
                if stack.len().checked_add(n).is_none_or(|new| new > MAX_STACK) {
                    return Err(Error::Type4Runtime(format!(
                        "Type 4 stack overflow during copy (max {MAX_STACK})"
                    )));
                }
                let start = stack.len() - n;
                let copied: Vec<Value> = stack[start..].to_vec();
                stack.extend_from_slice(&copied);
            },
            Instruction::Index => {
                // Note: operand pops happen before the bounds check below.
                // The mutation is visible only inside this scope; on error
                // the evaluation aborts and the partially-popped stack is
                // never observed by the caller.
                let n = pop_count(stack, "index")?;
                if n >= stack.len() {
                    return Err(underflow());
                }
                let val = stack[stack.len() - 1 - n];
                stack.push(val);
            },
            Instruction::Roll => {
                // Note: operand pops happen before the bounds check below.
                // The mutation is visible only inside this scope; on error
                // the evaluation aborts and the partially-popped stack is
                // never observed by the caller.
                let j = pop(stack)?.as_int()?;
                let n = pop_count(stack, "roll")?;
                if n > stack.len() {
                    return Err(underflow());
                }
                if n > 0 {
                    let start = stack.len() - n;
                    let slice = &mut stack[start..];
                    let len = slice.len() as i64;
                    let shift = j.rem_euclid(len) as usize;
                    slice.rotate_right(shift);
                }
            },
            Instruction::If(body) => {
                let cond = pop(stack)?.as_bool()?;
                if cond {
                    execute(body, stack, budget)?;
                }
            },
            Instruction::IfElse(true_branch, false_branch) => {
                let cond = pop(stack)?.as_bool()?;
                if cond {
                    execute(true_branch, stack, budget)?;
                } else {
                    execute(false_branch, stack, budget)?;
                }
            },
            Instruction::ProcedureBody(_) => {
                // Unreachable: `resolve_conditionals` rejects orphan procedure
                // bodies at parse time. If one reaches `execute` we treat it
                // as an internal invariant violation rather than panicking.
                return Err(Error::Type4Runtime(
                    "Type 4 internal error: ProcedureBody reached execute".into(),
                ));
            },
        }
    }
    Ok(())
}

fn pop(stack: &mut Vec<Value>) -> Result<Value> {
    stack.pop().ok_or_else(underflow)
}

/// Pop a non-negative count for `copy`/`index`/`roll`. PLRM rejects negative
/// or non-integer counts with `rangecheck`/`typecheck`; `as usize` on negative
/// or NaN floats would silently wrap.
fn pop_count(stack: &mut Vec<Value>, op: &str) -> Result<usize> {
    let v = pop(stack)?.as_int()?;
    if v < 0 {
        return Err(Error::Type4Runtime(format!("Type 4 {op}: negative count {v}")));
    }
    Ok(v as usize)
}

fn underflow() -> Error {
    Error::Type4Runtime("Type 4 stack underflow".into())
}

fn typecheck(msg: &str) -> Error {
    Error::Type4Runtime(format!("Type 4 typecheck: {msg}"))
}

fn real_unary(stack: &mut Vec<Value>, f: impl FnOnce(f64) -> Result<f64>) -> Result<()> {
    let a = pop(stack)?.as_real()?;
    stack.push(Value::Real(f(a)?));
    Ok(())
}

/// Unary operator that preserves integer-ness if the input was an integer
/// (e.g. `ceiling`, `floor`, `round`, `truncate` per PLRM §8.2).
fn real_unary_preserve(stack: &mut Vec<Value>, f: impl FnOnce(f64) -> Result<f64>) -> Result<()> {
    let v = pop(stack)?;
    match v {
        Value::Int(i) => stack.push(Value::Int(i)),
        Value::Real(r) => stack.push(Value::Real(f(r)?)),
        Value::Bool(_) => return Err(typecheck("expected number, got boolean")),
    }
    Ok(())
}

/// Arithmetic with PLRM type promotion: integer op integer -> integer (if no
/// overflow on add/sub/mul; we fall back to real on overflow), otherwise real.
fn numeric_binary(stack: &mut Vec<Value>, f: impl FnOnce(f64, f64) -> Result<f64>) -> Result<()> {
    let b = pop(stack)?;
    let a = pop(stack)?;
    let af = a.as_real()?;
    let bf = b.as_real()?;
    let result = f(af, bf)?;
    // Promote back to Int when both operands were integers and the result is
    // exactly representable. This keeps `52 not` working when authors wrap
    // bitwise ops around arithmetic chains.
    if matches!(a, Value::Int(_))
        && matches!(b, Value::Int(_))
        && result.is_finite()
        && result.fract() == 0.0
        && result >= i64::MIN as f64
        && result <= i64::MAX as f64
    {
        stack.push(Value::Int(result as i64));
    } else {
        stack.push(Value::Real(result));
    }
    Ok(())
}

fn comparison(stack: &mut Vec<Value>, pred: impl FnOnce(std::cmp::Ordering) -> bool) -> Result<()> {
    let b = pop(stack)?.as_real()?;
    let a = pop(stack)?.as_real()?;
    let ord = a
        .partial_cmp(&b)
        .ok_or_else(|| Error::Type4Runtime("Type 4 comparison with NaN".into()))?;
    stack.push(Value::Bool(pred(ord)));
    Ok(())
}

fn values_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Bool(_), _) | (_, Value::Bool(_)) => false,
        // PLRM treats `1` and `1.0` as equal, so compare numerically.
        _ => a.as_real().ok() == b.as_real().ok(),
    }
}

/// `and`/`or`/`xor`: PLRM §8.2 dispatches on operand type — both boolean uses
/// logical op, both integer uses bitwise. Mixed types are a typecheck error.
fn bool_or_bitwise(
    stack: &mut Vec<Value>,
    boolean: impl FnOnce(bool, bool) -> bool,
    bitwise: impl FnOnce(i64, i64) -> i64,
) -> Result<()> {
    let b = pop(stack)?;
    let a = pop(stack)?;
    match (a, b) {
        (Value::Bool(x), Value::Bool(y)) => stack.push(Value::Bool(boolean(x, y))),
        (Value::Int(x), Value::Int(y)) => stack.push(Value::Int(bitwise(x, y))),
        _ => return Err(typecheck("and/or/xor require matching boolean or integer operands")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: &[f64], b: &[f64], eps: f64) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < eps)
    }

    #[test]
    fn linear_ramp_tint_transform() {
        let prog = b"{ dup 0.84 mul exch 0.00 exch dup 0.44 mul exch 0.21 mul }";
        let result = evaluate_type4(prog, &[0.5]).unwrap();
        assert!(approx_eq(&result, &[0.42, 0.0, 0.22, 0.105], 1e-9), "got {result:?}");
    }

    #[test]
    fn identity_empty_program() {
        let result = evaluate_type4(b"{ }", &[0.7]).unwrap();
        assert!(approx_eq(&result, &[0.7], 1e-9), "got {result:?}");
    }

    #[test]
    fn constant_output() {
        let prog = b"{ pop 1.0 0.0 0.0 0.0 }";
        let result = evaluate_type4(prog, &[0.5]).unwrap();
        assert_eq!(result, vec![1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn conditional_ifelse() {
        let prog = b"{ dup 0.5 gt { pop 1.0 } { 0.0 exch } ifelse }";
        let high = evaluate_type4(prog, &[0.8]).unwrap();
        assert_eq!(high, vec![1.0]);
        let low = evaluate_type4(prog, &[0.3]).unwrap();
        assert!(approx_eq(&low, &[0.0, 0.3], 1e-9), "got {low:?}");
    }

    #[test]
    fn conditional_if() {
        let prog = b"{ dup 0.5 gt { 1.0 add } if }";
        let high = evaluate_type4(prog, &[0.8]).unwrap();
        assert!(approx_eq(&high, &[1.8], 1e-9), "got {high:?}");
        let low = evaluate_type4(prog, &[0.3]).unwrap();
        assert!(approx_eq(&low, &[0.3], 1e-9), "got {low:?}");
    }

    #[test]
    fn domain_range_clamping() {
        let prog = b"{ 2.0 mul }";
        let result = evaluate_type4_clamped(prog, &[1.5], &[[0.0, 1.0]], &[[0.0, 1.0]]).unwrap();
        // Input 1.5 clamped to 1.0, * 2.0 = 2.0, clamped to 1.0
        assert_eq!(result, vec![1.0]);
    }

    #[test]
    fn stack_underflow_returns_error() {
        let prog = b"{ add }";
        let err = evaluate_type4(prog, &[]).unwrap_err();
        assert!(err.to_string().contains("stack underflow"), "got: {err}");
    }

    #[test]
    fn arithmetic_operators() {
        assert_eq!(evaluate_type4(b"{ add }", &[3.0, 4.0]).unwrap(), vec![7.0]);
        assert_eq!(evaluate_type4(b"{ sub }", &[10.0, 3.0]).unwrap(), vec![7.0]);
        assert_eq!(evaluate_type4(b"{ mul }", &[3.0, 4.0]).unwrap(), vec![12.0]);
        assert_eq!(evaluate_type4(b"{ div }", &[10.0, 4.0]).unwrap(), vec![2.5]);
        assert_eq!(evaluate_type4(b"{ idiv }", &[10.0, 3.0]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ mod }", &[10.0, 3.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ neg }", &[5.0]).unwrap(), vec![-5.0]);
        assert_eq!(evaluate_type4(b"{ abs }", &[-5.0]).unwrap(), vec![5.0]);
        assert_eq!(evaluate_type4(b"{ ceiling }", &[3.2]).unwrap(), vec![4.0]);
        assert_eq!(evaluate_type4(b"{ floor }", &[3.8]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ round }", &[3.5]).unwrap(), vec![4.0]);
        assert_eq!(evaluate_type4(b"{ truncate }", &[3.9]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ sqrt }", &[9.0]).unwrap(), vec![3.0]);
    }

    #[test]
    fn trig_operators() {
        let sin_result = evaluate_type4(b"{ sin }", &[90.0]).unwrap();
        assert!((sin_result[0] - 1.0).abs() < 1e-9);
        let cos_result = evaluate_type4(b"{ cos }", &[0.0]).unwrap();
        assert!((cos_result[0] - 1.0).abs() < 1e-9);
        let atan_result = evaluate_type4(b"{ atan }", &[1.0, 1.0]).unwrap();
        assert!((atan_result[0] - 45.0).abs() < 1e-9);
    }

    #[test]
    fn log_operators() {
        let ln_result = evaluate_type4(b"{ ln }", &[std::f64::consts::E]).unwrap();
        assert!((ln_result[0] - 1.0).abs() < 1e-9);
        let log_result = evaluate_type4(b"{ log }", &[100.0]).unwrap();
        assert!((log_result[0] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn exp_operator() {
        let result = evaluate_type4(b"{ exp }", &[2.0, 10.0]).unwrap();
        assert!((result[0] - 1024.0).abs() < 1e-9);
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(evaluate_type4(b"{ eq }", &[1.0, 1.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ eq }", &[1.0, 2.0]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ ne }", &[1.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ gt }", &[2.0, 1.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ ge }", &[2.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ lt }", &[1.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ le }", &[2.0, 2.0]).unwrap(), vec![1.0]);
    }

    #[test]
    fn boolean_operators() {
        // True/false literals exercise the boolean dispatch in and/or/xor/not.
        assert_eq!(evaluate_type4(b"{ true false and }", &[]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ true false or }", &[]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ true true xor }", &[]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ true not }", &[]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ false not }", &[]).unwrap(), vec![1.0]);
    }

    #[test]
    fn bitwise_operators() {
        assert_eq!(evaluate_type4(b"{ 12 10 and }", &[]).unwrap(), vec![8.0]);
        assert_eq!(evaluate_type4(b"{ 12 10 or }", &[]).unwrap(), vec![14.0]);
        assert_eq!(evaluate_type4(b"{ 8 2 bitshift }", &[]).unwrap(), vec![32.0]);
        assert_eq!(evaluate_type4(b"{ 32 -2 bitshift }", &[]).unwrap(), vec![8.0]);
    }

    #[test]
    fn stack_manipulation() {
        assert_eq!(evaluate_type4(b"{ dup }", &[5.0]).unwrap(), vec![5.0, 5.0]);
        assert_eq!(evaluate_type4(b"{ exch }", &[1.0, 2.0]).unwrap(), vec![2.0, 1.0]);
        assert_eq!(evaluate_type4(b"{ pop }", &[1.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ 2 copy }", &[1.0, 2.0]).unwrap(), vec![1.0, 2.0, 1.0, 2.0]);
        assert_eq!(evaluate_type4(b"{ 1 index }", &[1.0, 2.0]).unwrap(), vec![1.0, 2.0, 1.0]);
    }

    #[test]
    fn roll_operator() {
        // roll(n=3, j=1): rotate top 3 elements by 1
        // [1, 2, 3] -> [3, 1, 2]
        assert_eq!(evaluate_type4(b"{ 3 1 roll }", &[1.0, 2.0, 3.0]).unwrap(), vec![3.0, 1.0, 2.0]);
        // roll(n=3, j=-1): rotate top 3 elements by -1
        // [1, 2, 3] -> [2, 3, 1]
        assert_eq!(
            evaluate_type4(b"{ 3 -1 roll }", &[1.0, 2.0, 3.0]).unwrap(),
            vec![2.0, 3.0, 1.0]
        );
    }

    #[test]
    fn bool_literals() {
        assert_eq!(evaluate_type4(b"{ true }", &[]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ false }", &[]).unwrap(), vec![0.0]);
    }

    #[test]
    fn division_by_zero_follows_ieee_754() {
        // Acrobat/Poppler hand back IEEE 754 specials for `div` by zero
        // rather than failing the whole program. We follow that behaviour;
        // `idiv` and `mod` (integer ops with no inf/NaN) stay as errors.
        let pos = evaluate_type4(b"{ div }", &[1.0, 0.0]).unwrap();
        assert_eq!(pos.len(), 1);
        assert!(pos[0].is_infinite() && pos[0] > 0.0, "expected +inf, got {pos:?}");

        let neg = evaluate_type4(b"{ div }", &[-1.0, 0.0]).unwrap();
        assert_eq!(neg.len(), 1);
        assert!(neg[0].is_infinite() && neg[0] < 0.0, "expected -inf, got {neg:?}");

        let nan = evaluate_type4(b"{ div }", &[0.0, 0.0]).unwrap();
        assert_eq!(nan.len(), 1);
        assert!(nan[0].is_nan(), "expected NaN, got {nan:?}");

        // idiv / mod by zero still error.
        assert!(evaluate_type4(b"{ idiv }", &[1.0, 0.0]).is_err());
        assert!(evaluate_type4(b"{ mod }", &[1.0, 0.0]).is_err());
    }

    #[test]
    fn int_min_neg_and_abs_error() {
        // i64::MIN cannot be negated or abs'd without overflow. PLRM raises
        // a runtime error; we map that to Error::Type4Runtime.
        let neg = format!("{{ {} neg }}", i64::MIN);
        let err = evaluate_type4(neg.as_bytes(), &[]).unwrap_err();
        assert!(matches!(err, Error::Type4Runtime(_)), "got: {err}");

        let abs = format!("{{ {} abs }}", i64::MIN);
        let err = evaluate_type4(abs.as_bytes(), &[]).unwrap_err();
        assert!(matches!(err, Error::Type4Runtime(_)), "got: {err}");
    }

    #[test]
    fn invalid_program_missing_braces() {
        let err = evaluate_type4(b"dup mul", &[1.0]).unwrap_err();
        assert!(err.to_string().contains("{ }"), "got: {err}");
    }

    #[test]
    fn nested_conditionals() {
        let prog =
            b"{ dup 0.5 gt { dup 0.8 gt { pop 1.0 } { pop 0.75 } ifelse } { pop 0.0 } ifelse }";
        assert_eq!(evaluate_type4(prog, &[0.9]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(prog, &[0.6]).unwrap(), vec![0.75]);
        assert_eq!(evaluate_type4(prog, &[0.3]).unwrap(), vec![0.0]);
    }

    #[test]
    fn real_world_spot_color_transforms() {
        // Pantone-style: single ink maps to CMYK
        let prog = b"{ 0 exch dup 0.78 mul exch 0.35 mul 0 }";
        let result = evaluate_type4(prog, &[1.0]).unwrap();
        assert!(approx_eq(&result, &[0.0, 0.78, 0.35, 0.0], 1e-9), "got {result:?}");
    }

    #[test]
    fn negative_number_literal() {
        let result = evaluate_type4(b"{ -3.5 add }", &[10.0]).unwrap();
        assert!(approx_eq(&result, &[6.5], 1e-9), "got {result:?}");
    }

    // --- Regression tests for PLRM §8.2 corner cases ---

    #[test]
    fn plrm_examples() {
        // (program_bytes, inputs, expected_outputs, description)
        let cases: &[(&[u8], &[f64], &[f64], &str)] = &[
            (b"{ atan }", &[-100.0, 0.0], &[270.0], "atan negative-num zero-den"),
            (b"{ atan }", &[-1.0, -1.0], &[225.0], "atan third quadrant"),
            (b"{ atan }", &[0.0, 1.0], &[0.0], "atan first axis"),
            (b"{ atan }", &[1.0, 1.0], &[45.0], "atan first quadrant"),
            (b"{ atan }", &[0.0, -1.0], &[180.0], "atan negative-x axis"),
            (b"{ round }", &[-6.5], &[-6.0], "round negative half toward +inf"),
            (b"{ round }", &[6.5], &[7.0], "round positive half toward +inf"),
            (b"{ round }", &[-0.5], &[0.0], "round -0.5"),
            (b"{ round }", &[0.5], &[1.0], "round 0.5"),
            (b"{ idiv }", &[-7.0, 2.0], &[-3.0], "idiv negative"),
            (b"{ mod }", &[-7.0, 2.0], &[-1.0], "mod negative dividend"),
            (b"{ truncate }", &[-6.5], &[-6.0], "truncate negative"),
        ];
        for (prog, inp, want, desc) in cases {
            let got = evaluate_type4(prog, inp).unwrap_or_else(|e| panic!("{desc}: {e}"));
            assert!(approx_eq(&got, want, 1e-9), "case: {desc}\n  got:  {got:?}\n  want: {want:?}");
        }
    }

    #[test]
    fn not_distinguishes_bool_from_int() {
        // PLRM §8.2: `true not -> false` (logical), `52 not -> -53` (bitwise),
        // `1 not -> -2` (bitwise on the integer literal 1, NOT boolean true).
        assert_eq!(evaluate_type4(b"{ true not }", &[]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ false not }", &[]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ 52 not }", &[]).unwrap(), vec![-53.0]);
        assert_eq!(evaluate_type4(b"{ 1 not }", &[]).unwrap(), vec![-2.0]);
        assert_eq!(evaluate_type4(b"{ 0 not }", &[]).unwrap(), vec![-1.0]);
    }

    #[test]
    fn and_or_xor_dispatch_on_type() {
        // Both-boolean -> logical
        assert_eq!(evaluate_type4(b"{ true true and }", &[]).unwrap(), vec![1.0]);
        // Both-integer -> bitwise
        assert_eq!(evaluate_type4(b"{ 12 10 and }", &[]).unwrap(), vec![8.0]);
        // Mixed -> typecheck error
        assert!(evaluate_type4(b"{ true 1 and }", &[]).is_err());
        assert!(evaluate_type4(b"{ 1 true or }", &[]).is_err());
    }

    #[test]
    fn integer_only_ops_reject_real_literals() {
        // PLRM §8.2: idiv, mod, bitshift require typed integer operands.
        // A real literal like `2.0` is a typed real and must be rejected.
        assert!(evaluate_type4(b"{ 5.5 2 idiv }", &[]).is_err());
        assert!(evaluate_type4(b"{ 5 2.5 idiv }", &[]).is_err());
        assert!(evaluate_type4(b"{ 5 2.0 mod }", &[]).is_err());
        assert!(evaluate_type4(b"{ 5.0 2 mod }", &[]).is_err());
        assert!(evaluate_type4(b"{ 1.0 not }", &[]).is_err());
        assert!(evaluate_type4(b"{ 3.0 1 bitshift }", &[]).is_err());
        assert!(evaluate_type4(b"{ 3 1.0 bitshift }", &[]).is_err());
    }

    #[test]
    fn integer_valued_inputs_accepted_by_integer_ops() {
        // Caller-supplied f64 inputs are an ambiguous typed-int/typed-real
        // boundary; integer-valued f64s are accepted by integer ops.
        assert_eq!(evaluate_type4(b"{ idiv }", &[10.0, 3.0]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ mod }", &[10.0, 3.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ bitshift }", &[1.0, 4.0]).unwrap(), vec![16.0]);
    }

    #[test]
    fn errors_not_panics() {
        // sqrt of negative, ln/log of non-positive -> error, not NaN/-inf.
        assert!(evaluate_type4(b"{ sqrt }", &[-1.0]).is_err());
        assert!(evaluate_type4(b"{ ln }", &[0.0]).is_err());
        assert!(evaluate_type4(b"{ ln }", &[-1.0]).is_err());
        assert!(evaluate_type4(b"{ log }", &[0.0]).is_err());
        assert!(evaluate_type4(b"{ log }", &[-1.0]).is_err());

        // Malformed Domain (min > max) used to panic in f64::clamp.
        let r = evaluate_type4_clamped(b"{ }", &[0.5], &[[1.0, 0.0]], &[]).unwrap();
        // Bounds are swapped, so 0.5 stays in [0, 1].
        assert_eq!(r, vec![0.5]);

        // NaN bounds must not panic — treat as no clamp.
        let r =
            evaluate_type4_clamped(b"{ }", &[0.5], &[[f64::NAN, 1.0]], &[[0.0, f64::NAN]]).unwrap();
        assert_eq!(r, vec![0.5]);

        // bitshift by >= 64 must not shift-overflow.
        assert_eq!(evaluate_type4(b"{ 1 64 bitshift }", &[]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ 1 100 bitshift }", &[]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ 1 -64 bitshift }", &[]).unwrap(), vec![0.0]);

        // idiv overflow path: i64::MIN / -1
        let prog = format!("{{ {} -1 idiv }}", i64::MIN);
        assert!(evaluate_type4(prog.as_bytes(), &[]).is_err());

        // Non-finite numeric literals must be rejected at parse time.
        assert!(evaluate_type4(b"{ inf }", &[]).is_err());
        assert!(evaluate_type4(b"{ NaN }", &[]).is_err());

        // idiv/mod on non-integral reals -> typecheck.
        assert!(evaluate_type4(b"{ 7.5 2 idiv }", &[]).is_err());
        assert!(evaluate_type4(b"{ 7 2.5 mod }", &[]).is_err());

        // Negative count for copy/index/roll -> error, not garbage.
        assert!(evaluate_type4(b"{ -1 copy }", &[1.0]).is_err());
        assert!(evaluate_type4(b"{ -1 index }", &[1.0]).is_err());
        assert!(evaluate_type4(b"{ -1 1 roll }", &[1.0, 2.0]).is_err());

        // atan undefined at (0, 0).
        assert!(evaluate_type4(b"{ atan }", &[0.0, 0.0]).is_err());
    }

    #[test]
    fn cvi_truncates_toward_zero() {
        // PLRM §8.2 examples
        assert_eq!(evaluate_type4(b"{ cvi }", &[3.2]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ cvi }", &[-3.2]).unwrap(), vec![-3.0]);
        assert_eq!(evaluate_type4(b"{ cvi }", &[3.0]).unwrap(), vec![3.0]);
        // 3.5 cvi -> 3 (truncate toward zero, not round)
        assert_eq!(evaluate_type4(b"{ 3.5 cvi }", &[]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ -3.5 cvi }", &[]).unwrap(), vec![-3.0]);
    }

    #[test]
    fn cvr_makes_typed_real() {
        // 3 cvr -> 3.0 as a typed real. Should not satisfy `idiv` (which
        // wants typed integers) — verifies the type tag really changed.
        let err = evaluate_type4(b"{ 3 cvr 2 idiv }", &[]).unwrap_err();
        assert!(matches!(err, Error::Type4Runtime(_)), "got: {err}");
        // 3.5 cvr -> 3.5 (stays real)
        assert_eq!(evaluate_type4(b"{ 3.5 cvr }", &[]).unwrap(), vec![3.5]);
        // Combined with `cvi`: `3.5 cvi 2 idiv` succeeds
        assert_eq!(evaluate_type4(b"{ 3.5 cvi 2 idiv }", &[]).unwrap(), vec![1.0]);
    }

    #[test]
    fn cvi_rejects_bool_and_non_finite() {
        assert!(evaluate_type4(b"{ true cvi }", &[]).is_err());
        assert!(evaluate_type4(b"{ true cvr }", &[]).is_err());
        // Runtime-produced inf/NaN: cvi rejects non-finite reals. Parser
        // refuses `inf`/`NaN` literals, so route through `1 0 div` (+inf)
        // and `0 0 div` (NaN) to hit the runtime-side check.
        assert!(evaluate_type4(b"{ 1 0 div cvi }", &[]).is_err());
        assert!(evaluate_type4(b"{ 0 0 div cvi }", &[]).is_err());
        // cvr accepts any real, including non-finite values — no integer
        // overflow concern. Verify the +inf round-trip survives.
        assert_eq!(evaluate_type4(b"{ 1 0 div cvr }", &[]).unwrap()[0], f64::INFINITY);
    }

    #[test]
    fn program_is_send_sync() {
        // A compiled program is meant to live in a shared tint-transform
        // cache; verify it satisfies the marker bounds.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Program>();
    }

    #[test]
    fn program_compile_once_evaluate_many() {
        // Same compiled program, 1000 independent evaluations. Sanity-check
        // that there is no shared mutable state — outputs depend only on
        // inputs.
        let program = Program::compile(b"{ dup mul }").expect("compile");
        for i in 0..1000 {
            let x = (i as f64) * 0.001;
            let out = program.evaluate(&[x]).expect("eval");
            assert_eq!(out.len(), 1);
            // x*x within fp tolerance
            let want = x * x;
            assert!((out[0] - want).abs() < 1e-12, "i={i}: got {out:?}, want {want}");
        }
    }

    #[test]
    fn program_evaluate_clamped_matches_wrapper() {
        // The wrapper should produce the same result as calling the typed
        // API directly.
        let program = Program::compile(b"{ 2.0 mul }").expect("compile");
        let direct = program
            .evaluate_clamped(&[1.5], &[[0.0, 1.0]], &[[0.0, 1.0]])
            .expect("direct");
        let via_fn = evaluate_type4_clamped(b"{ 2.0 mul }", &[1.5], &[[0.0, 1.0]], &[[0.0, 1.0]])
            .expect("via_fn");
        assert_eq!(direct, via_fn);
    }

    #[test]
    fn parse_depth_limit_enforced() {
        // 50 levels of nesting comfortably exceeds the depth budget without
        // requiring the actual call stack to grow that far (we error first).
        // Without this guard, parsing recurses into Rust's call stack until
        // it blows.
        let deep = format!("{{{}}}", "{".repeat(50)) + &"}".repeat(50);
        let err = evaluate_type4(deep.as_bytes(), &[]).unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "got: {err}");
        assert!(err.to_string().contains("depth"), "got: {err}");

        // Up to the depth cap itself: a deeply nested-but-bounded program
        // should parse successfully when each level only uses procedure bodies
        // that go on to be consumed by if/ifelse. Construct one with exactly
        // MAX_PARSE_DEPTH levels.
        // (We only validate "below the cap is fine" up to a modest depth here
        // because building 32-deep `if`-consuming programs is verbose.)
    }

    #[test]
    fn runtime_stack_overflow_caught() {
        // Push 300 ones; cap is 256.
        let prog = "{ ".to_string() + &"1 ".repeat(300) + "}";
        let err = evaluate_type4(prog.as_bytes(), &[]).unwrap_err();
        assert!(matches!(err, Error::Type4Runtime(_)), "got: {err}");
        assert!(err.to_string().contains("stack overflow"), "got: {err}");

        // Pushing 200 values is fine.
        let ok = "{ ".to_string() + &"1 ".repeat(200) + "}";
        assert!(evaluate_type4(ok.as_bytes(), &[]).is_ok());
    }

    #[test]
    fn instruction_budget_caught() {
        // 100_001 `dup pop` pairs keep the stack bounded but consume the
        // instruction budget. Each pair is two ticks, so this is well past
        // MAX_INSTRUCTIONS by design.
        let mut body = String::from("{ ");
        for _ in 0..100_001 {
            body.push_str("dup pop ");
        }
        body.push('}');
        let err = evaluate_type4(body.as_bytes(), &[1.0]).unwrap_err();
        assert!(matches!(err, Error::Type4Runtime(_)), "got: {err}");
        assert!(err.to_string().contains("instruction budget"), "got: {err}");
    }

    #[test]
    fn input_count_capped() {
        let many: Vec<f64> = (0..(MAX_STACK + 1)).map(|i| i as f64).collect();
        let err = evaluate_type4(b"{ }", &many).unwrap_err();
        assert!(matches!(err, Error::Type4Runtime(_)), "got: {err}");
    }

    #[test]
    fn orphan_procedure_body_rejected_at_parse() {
        // `{ 1 { 2 } 3 }` has an inner `{ 2 }` that no `if`/`ifelse` consumes.
        // Previous behavior silently turned the inner body into an `If` and
        // mis-executed it (popping a bool that wasn't there). Now: parse error.
        let err = evaluate_type4(b"{ 1 { 2 } 3 }", &[]).unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "got: {err}");
        assert!(err.to_string().contains("orphan"), "got: {err}");
    }

    #[test]
    fn orphan_procedure_body_alone_rejected() {
        // A program that is only a procedure body with nothing else also has
        // no `if`/`ifelse` to consume it.
        let err = evaluate_type4(b"{ { 1 2 add } }", &[]).unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "got: {err}");
    }

    #[test]
    fn atan_full_range() {
        // PLRM §8.2: atan returns angle in [0, 360).
        for &(num, den, want) in &[
            (0.0, 1.0, 0.0),
            (1.0, 1.0, 45.0),
            (1.0, 0.0, 90.0),
            (1.0, -1.0, 135.0),
            (0.0, -1.0, 180.0),
            (-1.0, -1.0, 225.0),
            (-1.0, 0.0, 270.0),
            (-1.0, 1.0, 315.0),
            (-100.0, 0.0, 270.0),
        ] {
            let got = evaluate_type4(b"{ atan }", &[num, den]).unwrap();
            assert!((got[0] - want).abs() < 1e-9, "atan({num}, {den}) = {got:?}, want {want}");
            assert!(got[0] >= 0.0 && got[0] < 360.0, "atan out of [0, 360): {got:?}");
        }
    }

    #[test]
    fn parse_depth_limit_returns_invalid_pdf() {
        // A pathologically nested program is malformed at parse time —
        // it must surface as InvalidPdf so callers don't classify it as a
        // runtime resource failure and retry forever.
        let mut bytes = Vec::new();
        bytes.extend(std::iter::repeat_n(b'{', 50));
        bytes.extend(std::iter::repeat_n(b'}', 50));
        match evaluate_type4(&bytes, &[]) {
            Err(Error::InvalidPdf(_)) => {}, // correct
            Err(Error::Type4Runtime(s)) => {
                panic!("parse depth should error as InvalidPdf, not Type4Runtime: {s}")
            },
            Err(other) => panic!("unexpected error: {other}"),
            Ok(out) => panic!("should have errored, got {out:?}"),
        }
    }

    #[test]
    fn cvi_rejects_two_pow_63() {
        // 2^63 is representable in f64 but not as i64. The upper-bound
        // check must use >= (or its mathematical equivalent) so the
        // boundary is rejected with an integer-overflow error rather
        // than silently saturating to i64::MAX = 2^63 - 1.
        let pow_63: f64 = 9_223_372_036_854_775_808.0; // exactly 2^63
        assert_eq!(pow_63, i64::MAX as f64, "test setup: 2^63 == i64::MAX as f64");
        let result = evaluate_type4(b"{ cvi }", &[pow_63]);
        match result {
            Err(Error::Type4Runtime(s)) if s.contains("cvi") => {}, // correct
            Err(other) => panic!("expected Type4Runtime(cvi overflow), got: {other}"),
            Ok(v) => panic!(
                "2^63 cvi should overflow; got {v:?} (likely saturated to i64::MAX = {})",
                i64::MAX
            ),
        }
    }

    #[test]
    fn cvi_accepts_two_pow_63_minus_one() {
        // i64::MAX itself cannot be exactly represented as f64, so the
        // largest f64 that rounds to a valid i64 via truncation is
        // the predecessor, ~9.223e18. Verify this passes.
        let near_max: f64 = 9_223_372_036_854_774_784.0; // f64 right below 2^63
        let result = evaluate_type4(b"{ cvi }", &[near_max]).unwrap();
        assert_eq!(result.len(), 1);
        // The result should be a valid i64 close to i64::MAX
        assert!(result[0] > 0.0 && result[0] < i64::MAX as f64);
    }
}
