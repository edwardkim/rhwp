//! EqEdit → LaTeX converter.
//!
//! HWP stores equations as a script in Hancom's *EqEdit* language (see
//! [`crate::doclang::ir::formula::Formula::raw_eqedit`]).  This module converts that
//! script to raw LaTeX suitable for a DocLang `<formula>` element (no `$`
//! delimiters).
//!
//! # Design
//!
//! The pipeline is the classic three stages:
//!
//! 1. [`lexer`] — split the script into a flat token stream.
//! 2. [`parser`] — recursive-descent build of a small AST ([`parser::Node`]).
//! 3. [`latex`] — emit LaTeX from the AST.
//!
//! # Error policy
//!
//! Conversion is **permissive by design**.  Unknown commands and identifiers
//! are *not* errors: they pass through as upright `\text{…}` or as a literal
//! command so the caller still gets usable output.  The **only** failure is
//! structurally broken input — unbalanced braces that cannot be recovered —
//! which returns [`EqError`].  Callers (the writer) treat any `Err` as a signal
//! to fall back to a placeholder and record `LossKind::FormulaFallback`.
//!
//! ```
//! use rhwp::doclang::eqedit::convert;
//! assert_eq!(convert("1 over 2").unwrap(), "\\frac{1}{2}");
//! assert!(convert("{ x").is_err()); // unbalanced brace
//! ```

pub mod error;
pub mod latex;
pub mod lexer;
pub mod parser;

pub use error::EqError;

/// The outcome of a successful EqEdit → LaTeX conversion.
///
/// Conversion always produces usable `latex`; `degraded` lists any command-like
/// identifiers (alphabetic, length ≥ 2) that were **not** recognised as commands
/// and therefore fell through to upright `\text{…}`.  A non-empty `degraded`
/// means the LaTeX is structurally valid but *semantically lossy* — e.g. an
/// uppercase `TIMES` that rendered as `\text{TIMES}` instead of `\times`.
/// Callers should keep the `latex` but record the degradation as a loss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EqOutcome {
    /// The emitted LaTeX (no `$` delimiters).
    pub latex: String,
    /// Command-like identifiers that degraded to `\text{…}` (may be empty).
    pub degraded: Vec<String>,
}

/// Converts an EqEdit `script` into a LaTeX string (no `$` delimiters).
///
/// Returns [`EqError`] only for structurally irrecoverable input (unbalanced
/// braces). Unknown commands/identifiers are passed through, not rejected.
///
/// This is a convenience wrapper over [`convert_with_degraded`] that discards
/// the degraded-token list; use [`convert_with_degraded`] when the caller needs
/// to know whether the conversion was semantically lossy.
///
/// # Examples
///
/// ```
/// use rhwp::doclang::eqedit::convert;
/// assert_eq!(convert("sum from {i=1} to {n} i^2").unwrap(),
///            "\\sum_{i=1}^n i^2");
/// ```
pub fn convert(script: &str) -> Result<String, EqError> {
    convert_with_degraded(script).map(|o| o.latex)
}

/// Converts an EqEdit `script` into LaTeX, also reporting any degraded
/// (command-like-but-unrecognised) tokens.
///
/// Returns [`EqError`] only for structurally irrecoverable input (unbalanced
/// braces). A successful conversion always yields usable LaTeX; inspect
/// [`EqOutcome::degraded`] to learn whether it was semantically lossy.
///
/// # Examples
///
/// ```
/// use rhwp::doclang::eqedit::convert_with_degraded;
/// let out = convert_with_degraded("TIMES").unwrap();
/// // Uppercase commands now resolve, so nothing degrades:
/// assert_eq!(out.latex, "\\times");
/// assert!(out.degraded.is_empty());
/// ```
pub fn convert_with_degraded(script: &str) -> Result<EqOutcome, EqError> {
    let tokens = lexer::lex(script);
    let ast = parser::parse(&tokens)?;
    let (latex, degraded) = latex::emit_with_degraded(&ast);
    Ok(EqOutcome { latex, degraded })
}

#[cfg(test)]
mod tests {
    use super::{convert, convert_with_degraded, EqError};

    /// Helper: assert a script converts to the expected LaTeX.
    fn check(script: &str, expected: &str) {
        assert_eq!(convert(script).unwrap(), expected, "script: {script:?}");
    }

    #[test]
    fn fraction() {
        check("1 over 2", "\\frac{1}{2}");
    }

    #[test]
    fn nested_fraction() {
        check("a over {b over c}", "\\frac{a}{\\frac{b}{c}}");
    }

    #[test]
    fn atop() {
        check("a atop b", "{a \\atop b}");
    }

    #[test]
    fn sqrt_group() {
        check("sqrt {x+1}", "\\sqrt{x+1}");
    }

    #[test]
    fn root_n_of() {
        check("root 3 of {x}", "\\sqrt[3]{x}");
    }

    #[test]
    fn superscript_single() {
        check("x^2", "x^2");
    }

    #[test]
    fn superscript_multi_braced() {
        check("x^{n+1}", "x^{n+1}");
    }

    #[test]
    fn subscript_keyword_and_symbol() {
        check("x sub i", "x_i");
        check("x_i", "x_i");
    }

    #[test]
    fn sum_with_bounds() {
        check("sum from {i=1} to {n} i^2", "\\sum_{i=1}^n i^2");
    }

    #[test]
    fn int_with_bounds_and_infty() {
        check(
            "int from 0 to infty e^{-x} dx",
            "\\int_0^{\\infty}e^{-x}\\text{dx}",
        );
    }

    #[test]
    fn oint_command() {
        check("oint", "\\oint");
    }

    #[test]
    fn lim_with_bound() {
        check("lim from {x rarrow 0}", "\\lim_{x \\rightarrow 0}");
    }

    #[test]
    fn greek_lowercase() {
        check("alpha beta gamma", "\\alpha \\beta \\gamma");
    }

    #[test]
    fn greek_uppercase_forms() {
        check("Gamma", "\\Gamma");
        check("GAMMA", "\\Gamma");
    }

    #[test]
    fn matrix_2x2() {
        check(
            "matrix{a & b # c & d}",
            "\\begin{matrix}a & b \\\\ c & d\\end{matrix}",
        );
    }

    #[test]
    fn pmatrix_2x2() {
        check(
            "pmatrix{1 & 0 # 0 & 1}",
            "\\begin{pmatrix}1 & 0 \\\\ 0 & 1\\end{pmatrix}",
        );
    }

    #[test]
    fn cases_environment() {
        check(
            "cases{x & x>0 # 0 & x leq 0}",
            "\\begin{cases}x & x>0 \\\\ 0 & x \\leq 0\\end{cases}",
        );
    }

    #[test]
    fn decoration_bar_hat_vec() {
        check("bar x", "\\bar{x}");
        check("hat y", "\\hat{y}");
        check("vec v", "\\vec{v}");
    }

    #[test]
    fn decoration_dot_tilde() {
        check("dot x", "\\dot{x}");
        check("tilde a", "\\tilde{a}");
    }

    #[test]
    fn left_right_delimiters() {
        check("left ( a over b right )", "\\left( \\frac{a}{b} \\right)");
    }

    #[test]
    fn left_right_brace_delimiters() {
        check("left { x right }", "\\left\\{ x \\right\\}");
    }

    #[test]
    fn binom() {
        check("binom {n} {k}", "\\binom{n}{k}");
    }

    #[test]
    fn font_switches() {
        check("it x", "\\mathit{x}");
        check("rm d", "\\mathrm{d}");
        check("bold v", "\\mathbf{v}");
    }

    #[test]
    fn operators_and_relations() {
        check("a times b", "a \\times b");
        check("a div b", "a \\div b");
        check("a leq b", "a \\leq b");
        check("a geq b", "a \\geq b");
        check("a neq b", "a \\neq b");
        check("a approx b", "a \\approx b");
        check("a pm b", "a \\pm b");
        check("a cdot b", "a \\cdot b");
    }

    #[test]
    fn set_and_logic_symbols() {
        check("x in A", "x \\in A");
        check("x notin A", "x \\notin A");
        check("A subset B", "A \\subset B");
        check("A cup B", "A \\cup B");
        check("A cap B", "A \\cap B");
        check("forall x", "\\forall x");
        check("exist y", "\\exists y");
        check("therefore p", "\\therefore p");
        check("because q", "\\because q");
    }

    #[test]
    fn dots_and_arrows() {
        check("cdots", "\\cdots");
        check("ldots", "\\ldots");
        check("a rarrow b", "a \\rightarrow b");
        check("a larrow b", "a \\leftarrow b");
    }

    #[test]
    fn thin_and_normal_space() {
        check("a ` b", "a\\,b");
        check("a ~ b", "a\\ b");
    }

    #[test]
    fn unknown_command_passthrough() {
        // An unknown multi-letter identifier is emitted as upright text,
        // never an error.
        check("foobar", "\\text{foobar}");
    }

    #[test]
    fn unknown_mixed_with_known() {
        check("alpha widget", "\\alpha \\text{widget}");
    }

    #[test]
    fn single_letter_identifier_stays_italic() {
        check("x", "x");
    }

    #[test]
    fn empty_input_is_empty_output() {
        check("", "");
    }

    #[test]
    fn whitespace_only_is_empty() {
        check("   \t\n ", "");
    }

    #[test]
    fn unbalanced_open_brace_errors() {
        assert_eq!(convert("{ x"), Err(EqError::UnbalancedBrace));
    }

    #[test]
    fn unbalanced_close_brace_errors() {
        assert_eq!(convert("x }"), Err(EqError::UnbalancedBrace));
    }

    #[test]
    fn deeply_nested_balanced_ok() {
        check("{{{x}}}", "x");
    }

    #[test]
    fn plain_arithmetic_passthrough() {
        check("a + b - c = d", "a+b-c=d");
    }

    // --- Case-insensitive keyword matching (criterion 7 fix) ---

    #[test]
    fn uppercase_over_is_fraction() {
        check("1 OVER 2", "\\frac{1}{2}");
    }

    #[test]
    fn uppercase_sqrt_is_radical() {
        check("SQRT {x}", "\\sqrt{x}");
    }

    #[test]
    fn uppercase_times_is_command() {
        // Previously degraded to `\text{TIMES}`; now resolves to `\times`.
        check("TIMES", "\\times");
    }

    #[test]
    fn uppercase_left_right_delimiters() {
        // Previously degraded to `\text{LEFT}(…\text{RIGHT})`.
        check("LEFT ( x RIGHT )", "\\left( x \\right)");
    }

    #[test]
    fn mixed_case_keywords_resolve() {
        check("1 Over 2", "\\frac{1}{2}");
        check("Sqrt {x}", "\\sqrt{x}");
        check("a Times b", "a \\times b");
    }

    /// The real eq-01 corpus formula pattern: lowercase `over` mixed with
    /// uppercase `TIMES`/`LEFT`/`RIGHT`. Before the fix this produced
    /// `…\text{TIMES} \text{LEFT}(\frac{…}{…}\text{RIGHT})`; it must now produce
    /// proper `\times` / `\left` / `\right`.
    #[test]
    fn eq01_corpus_pattern_uses_real_commands() {
        let latex =
            convert("최저입찰가격 TIMES LEFT ( 최저입찰가격 over 해당입찰가격 RIGHT )").unwrap();
        assert!(latex.contains("\\times"), "expected \\times in {latex:?}");
        assert!(latex.contains("\\left("), "expected \\left( in {latex:?}");
        assert!(latex.contains("\\right)"), "expected \\right) in {latex:?}");
        assert!(latex.contains("\\frac{"), "expected \\frac in {latex:?}");
        // The bug signature must be gone.
        assert!(
            !latex.contains("\\text{TIMES}"),
            "TIMES still degraded: {latex:?}"
        );
        assert!(
            !latex.contains("\\text{LEFT}"),
            "LEFT still degraded: {latex:?}"
        );
        assert!(
            !latex.contains("\\text{RIGHT}"),
            "RIGHT still degraded: {latex:?}"
        );
    }

    // --- Mixed-case non-command passthrough ---

    #[test]
    fn mixed_case_non_command_passes_through_as_text() {
        // A mixed-case word that matches no command must still pass through as
        // upright text, unchanged — not be coerced into a command.
        check("Widget", "\\text{Widget}");
        check("fooBar", "\\text{fooBar}");
    }

    // --- Honest degradation reporting ---

    #[test]
    fn known_command_does_not_degrade() {
        let out = convert_with_degraded("TIMES").unwrap();
        assert_eq!(out.latex, "\\times");
        assert!(out.degraded.is_empty(), "degraded should be empty: {out:?}");
    }

    #[test]
    fn unknown_command_like_word_is_reported_degraded() {
        // A multi-letter all-alphabetic identifier that matches no command is a
        // degraded token (rendered as \text{…}) and must be reported.
        let out = convert_with_degraded("FOOBAR").unwrap();
        assert_eq!(out.latex, "\\text{FOOBAR}");
        assert_eq!(out.degraded, vec!["FOOBAR".to_string()]);
    }

    #[test]
    fn single_letter_and_labels_not_degraded() {
        // Single letters are ordinary variables; words with digits are labels.
        let out = convert_with_degraded("x").unwrap();
        assert!(out.degraded.is_empty());
        let out = convert_with_degraded("x1").unwrap();
        assert!(
            out.degraded.is_empty(),
            "x1 is a label, not degraded: {out:?}"
        );
    }

    #[test]
    fn degraded_list_collects_multiple_tokens() {
        // None of these are commands or structural keywords, so all three
        // degrade to \text{…} and are reported.
        let out = convert_with_degraded("FOO QUUX baz").unwrap();
        assert_eq!(
            out.degraded,
            vec!["FOO".to_string(), "QUUX".to_string(), "baz".to_string()]
        );
    }
}
