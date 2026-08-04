//! LaTeX emitter for the EqEdit AST, plus the command-mapping table.
//!
//! [`emit`] walks a [`Node`] tree and produces a LaTeX string with **no** `$`
//! delimiters (DocLang's `<formula>` element takes raw LaTeX).  [`command_for`]
//! is the single source of truth for which bare EqEdit words map to LaTeX
//! commands (Greek letters, operators, relations, arrows, named functions);
//! the parser uses it to decide command-vs-identifier classification.

use crate::doclang::eqedit::parser::Node;

/// Returns the LaTeX command (without the leading backslash) for a known
/// EqEdit word, or `None` if the word is not a recognised command.
///
/// Covers: Greek letters (lower/upper), big operators, binary operators,
/// relations, arrows, set/logic symbols, dots, and named functions.  Named
/// functions (`sin`, `log`, …) map to themselves prefixed with a backslash so
/// they render upright (`\sin`).
///
/// Matching is **case-insensitive**, matching HWP EqEdit semantics where
/// commands may be written in any case (`TIMES`, `Times`, `times`).  An exact
/// match is tried first so capitalised Greek (`Gamma`) keeps its capitalised
/// LaTeX form; otherwise the all-lowercase form of the word is looked up so
/// `TIMES`/`LEFT`-style uppercase commands resolve correctly.  Words that match
/// no command in either form return `None` and pass through as text.
pub fn command_for(word: &str) -> Option<&'static str> {
    command_for_exact(word).or_else(|| {
        // Fall back to an all-lowercase lookup so uppercase/mixed-case command
        // words (e.g. `TIMES`, `Sqrt`) resolve. Only do this when the word is
        // not already lowercase, to avoid a redundant second lookup.
        if word.chars().any(|c| c.is_ascii_uppercase()) {
            command_for_exact(&word.to_ascii_lowercase())
        } else {
            None
        }
    })
}

/// Exact (case-sensitive) command-table lookup. The public [`command_for`]
/// layers case-insensitive fallback on top of this.
fn command_for_exact(word: &str) -> Option<&'static str> {
    let cmd = match word {
        // Lowercase Greek
        "alpha" => "alpha",
        "beta" => "beta",
        "gamma" => "gamma",
        "delta" => "delta",
        "epsilon" => "epsilon",
        "varepsilon" => "varepsilon",
        "zeta" => "zeta",
        "eta" => "eta",
        "theta" => "theta",
        "vartheta" => "vartheta",
        "iota" => "iota",
        "kappa" => "kappa",
        "lambda" => "lambda",
        "mu" => "mu",
        "nu" => "nu",
        "xi" => "xi",
        "omicron" => "omicron",
        "pi" => "pi",
        "varpi" => "varpi",
        "rho" => "rho",
        "varrho" => "varrho",
        "sigma" => "sigma",
        "varsigma" => "varsigma",
        "tau" => "tau",
        "upsilon" => "upsilon",
        "phi" => "phi",
        "varphi" => "varphi",
        "chi" => "chi",
        "psi" => "psi",
        "omega" => "omega",
        // Uppercase Greek (HWP uses Capitalised or ALL-CAPS forms).
        "Gamma" | "GAMMA" => "Gamma",
        "Delta" | "DELTA" => "Delta",
        "Theta" | "THETA" => "Theta",
        "Lambda" | "LAMBDA" => "Lambda",
        "Xi" | "XI" => "Xi",
        "Pi" | "PI" => "Pi",
        "Sigma" | "SIGMA" => "Sigma",
        "Upsilon" | "UPSILON" => "Upsilon",
        "Phi" | "PHI" => "Phi",
        "Psi" | "PSI" => "Psi",
        "Omega" | "OMEGA" => "Omega",
        "Alpha" | "ALPHA" => "Alpha",
        "Beta" | "BETA" => "Beta",
        // Big operators
        "sum" => "sum",
        "prod" => "prod",
        "int" => "int",
        "oint" => "oint",
        "iint" => "iint",
        "iiint" => "iiint",
        "lim" => "lim",
        "coprod" => "coprod",
        "bigcup" => "bigcup",
        "bigcap" => "bigcap",
        // Named functions (upright)
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "cot" => "cot",
        "sec" => "sec",
        "csc" => "csc",
        "sinh" => "sinh",
        "cosh" => "cosh",
        "tanh" => "tanh",
        "arcsin" => "arcsin",
        "arccos" => "arccos",
        "arctan" => "arctan",
        "log" => "log",
        "ln" => "ln",
        "exp" => "exp",
        "max" => "max",
        "min" => "min",
        "gcd" => "gcd",
        "deg" => "deg",
        "det" => "det",
        // Binary operators
        "times" => "times",
        "div" => "div",
        "cdot" => "cdot",
        "pm" => "pm",
        "mp" => "mp",
        "ast" => "ast",
        "star" => "star",
        "circ" => "circ",
        "bullet" => "bullet",
        "oplus" => "oplus",
        "otimes" => "otimes",
        // Relations
        "leq" | "le" => "leq",
        "geq" | "ge" => "geq",
        "neq" | "ne" => "neq",
        "approx" => "approx",
        "equiv" => "equiv",
        "cong" => "cong",
        "sim" => "sim",
        "simeq" => "simeq",
        "propto" => "propto",
        "ll" => "ll",
        "gg" => "gg",
        // Arrows
        "rarrow" | "rightarrow" | "to" => "rightarrow",
        "larrow" | "leftarrow" => "leftarrow",
        "lrarrow" | "leftrightarrow" => "leftrightarrow",
        "Rarrow" | "Rightarrow" => "Rightarrow",
        "Larrow" | "Leftarrow" => "Leftarrow",
        "uparrow" => "uparrow",
        "downarrow" => "downarrow",
        "mapsto" => "mapsto",
        // Sets & logic
        "inf" | "infty" | "infin" => "infty",
        "in" => "in",
        "notin" => "notin",
        "ni" => "ni",
        "subset" => "subset",
        "supset" => "supset",
        "subseteq" => "subseteq",
        "supseteq" => "supseteq",
        "cup" => "cup",
        "cap" => "cap",
        "emptyset" => "emptyset",
        "forall" => "forall",
        "exist" | "exists" => "exists",
        "nabla" => "nabla",
        "partial" => "partial",
        "therefore" => "therefore",
        "because" => "because",
        "neg" | "lnot" => "neg",
        "land" | "wedge" => "wedge",
        "lor" | "vee" => "vee",
        // Dots
        "cdots" => "cdots",
        "ldots" => "ldots",
        "vdots" => "vdots",
        "ddots" => "ddots",
        // Delimiter symbols (used bare, not via left/right)
        "langle" => "langle",
        "rangle" => "rangle",
        "lfloor" => "lfloor",
        "rfloor" => "rfloor",
        "lceil" => "lceil",
        "rceil" => "rceil",
        // More relations / logic
        "perp" => "perp",
        "parallel" => "parallel",
        "mid" => "mid",
        "vdash" => "vdash",
        "models" => "models",
        "asymp" => "asymp",
        "doteq" => "doteq",
        "prec" => "prec",
        "succ" => "succ",
        "preceq" => "preceq",
        "succeq" => "succeq",
        "top" => "top",
        "bot" => "bot",
        // More operators / symbols
        "setminus" => "setminus",
        "triangle" => "triangle",
        "diamond" => "diamond",
        "surd" => "surd",
        "wp" => "wp",
        "sharp" => "sharp",
        "flat" => "flat",
        "natural" => "natural",
        // More arrows
        "longrightarrow" => "longrightarrow",
        "longleftarrow" => "longleftarrow",
        "Longrightarrow" => "Longrightarrow",
        "Longleftarrow" => "Longleftarrow",
        "hookrightarrow" => "hookrightarrow",
        "hookleftarrow" => "hookleftarrow",
        "nearrow" => "nearrow",
        "searrow" => "searrow",
        "swarrow" => "swarrow",
        "nwarrow" => "nwarrow",
        // Misc
        "prime" => "prime",
        "angle" => "angle",
        "deg_sym" => "circ",
        "Re" => "Re",
        "Im" => "Im",
        "aleph" => "aleph",
        "hbar" => "hbar",
        "ell" => "ell",
        _ => return None,
    };
    Some(cmd)
}

/// Emits a LaTeX string for the given AST node (no `$` delimiters).
pub fn emit(node: &Node) -> String {
    emit_with_degraded(node).0
}

/// Emits a LaTeX string and, alongside it, the list of *degraded* tokens:
/// command-like identifiers (alphabetic, length ≥ 2) that were not recognised
/// as commands and fell through to upright `\text{…}`.  These are the tokens a
/// caller should report as a lossy/degraded conversion (e.g. an uppercase
/// `TIMES` that produced `\text{TIMES}` rather than `\times`).
///
/// Single-letter identifiers and identifiers containing digits or non-letters
/// are ordinary variables/labels, not degraded commands, and are excluded.
pub fn emit_with_degraded(node: &Node) -> (String, Vec<String>) {
    let mut out = String::new();
    let mut degraded = Vec::new();
    emit_into(node, &mut out, &mut degraded);
    (out, degraded)
}

/// Internal recursive emitter writing into a shared buffer and collecting
/// degraded (command-like) identifiers into `degraded`.
fn emit_into(node: &Node, out: &mut String, degraded: &mut Vec<String>) {
    match node {
        Node::Number(n) => out.push_str(n),
        Node::Ident(id) => emit_ident(id, out, degraded),
        Node::Symbol(c) => out.push_str(&escape_symbol(*c)),
        Node::ThinSpace => out.push_str("\\,"),
        Node::Space => out.push_str("\\ "),
        Node::Group(nodes) => emit_seq(nodes, out, degraded),
        Node::Frac(a, b) => {
            out.push_str("\\frac{");
            emit_into(a, out, degraded);
            out.push_str("}{");
            emit_into(b, out, degraded);
            out.push('}');
        }
        Node::Atop(a, b) => {
            out.push('{');
            emit_into(a, out, degraded);
            out.push_str(" \\atop ");
            emit_into(b, out, degraded);
            out.push('}');
        }
        Node::Sup(base, exp) => {
            emit_braced(base, out, degraded);
            out.push('^');
            emit_braced(exp, out, degraded);
        }
        Node::Sub(base, sub) => {
            emit_braced(base, out, degraded);
            out.push('_');
            emit_braced(sub, out, degraded);
        }
        Node::Sqrt(arg) => {
            out.push_str("\\sqrt{");
            emit_into(arg, out, degraded);
            out.push('}');
        }
        Node::Root(index, radicand) => {
            out.push_str("\\sqrt[");
            emit_into(index, out, degraded);
            out.push_str("]{");
            emit_into(radicand, out, degraded);
            out.push('}');
        }
        Node::Command(word) => {
            let cmd = command_for(word).unwrap_or(word.as_str());
            out.push('\\');
            out.push_str(cmd);
        }
        Node::Decoration(accent, target) => {
            out.push('\\');
            out.push_str(accent);
            out.push('{');
            emit_into(target, out, degraded);
            out.push('}');
        }
        Node::Bounds { base, lower, upper } => {
            emit_into(base, out, degraded);
            out.push('_');
            emit_braced(lower, out, degraded);
            if let Some(u) = upper {
                out.push('^');
                emit_braced(u, out, degraded);
            }
        }
        Node::Delimited { open, body, close } => {
            out.push_str("\\left");
            out.push_str(open);
            out.push(' ');
            emit_into(body, out, degraded);
            out.push_str(" \\right");
            out.push_str(close);
        }
        Node::Matrix { env, rows } => emit_matrix(env, rows, out, degraded),
        Node::Binom(a, b) => {
            out.push_str("\\binom{");
            emit_into(a, out, degraded);
            out.push_str("}{");
            emit_into(b, out, degraded);
            out.push('}');
        }
        Node::Font(font, target) => {
            out.push('\\');
            out.push_str(font);
            out.push('{');
            emit_into(target, out, degraded);
            out.push('}');
        }
    }
}

/// Emits a node, wrapping it in `{ }` unless it is a single atomic token.
/// This keeps `x^{12}` correct while leaving `x^2` unbraced.
fn emit_braced(node: &Node, out: &mut String, degraded: &mut Vec<String>) {
    if is_single_atom(node) {
        emit_into(node, out, degraded);
    } else {
        out.push('{');
        emit_into(node, out, degraded);
        out.push('}');
    }
}

/// True for nodes that render as exactly one LaTeX character (no braces needed
/// in a sub/superscript position). Commands like `\infty` render as multiple
/// characters and are braced for clarity, matching conventional LaTeX output.
fn is_single_atom(node: &Node) -> bool {
    match node {
        Node::Number(n) => n.chars().count() == 1,
        Node::Symbol(_) => true,
        Node::Ident(id) => id.chars().count() == 1,
        Node::Group(g) => g.len() == 1 && is_single_atom(&g[0]),
        _ => false,
    }
}

/// Emits a sequence of sibling nodes, inserting a single separating space
/// between two *alphabetic* runs while letting punctuation hug its neighbours.
///
/// This yields natural, glue-safe output: command words are separated
/// (`\alpha \beta`, `a \times b`) so they never merge into an undefined control
/// sequence (`\timesb`), while symbols stay tight (`x+1`, `e^{-x}`).
fn emit_seq(nodes: &[Node], out: &mut String, degraded: &mut Vec<String>) {
    let mut prev: Option<&Node> = None;
    for node in nodes {
        if let Some(p) = prev {
            let is_space = matches!(node, Node::ThinSpace | Node::Space);
            // A trailing `\command` always needs a separator so LaTeX does not
            // absorb the next token into the control-sequence name (`\rightarrow0`).
            // A trailing plain letter only needs one before another letter/command
            // (so `x+1` stays tight but `\alpha \beta` and `a \times` separate).
            let need_space = !is_space
                && (ends_command(p) || (ends_letter(p) && starts_letter_or_command(node)));
            if need_space {
                out.push(' ');
            }
        }
        emit_into(node, out, degraded);
        prev = Some(node);
    }
}

/// True if the node's LaTeX output ends with a `\command` word — a following
/// token of any kind must be space-separated to terminate the control sequence.
fn ends_command(node: &Node) -> bool {
    match node {
        Node::Command(_) => true,
        Node::Group(g) => g.last().map(ends_command).unwrap_or(false),
        Node::Sup(_, s) | Node::Sub(_, s) => is_single_atom(s) && ends_command(s),
        Node::Bounds { lower, upper, .. } => match upper {
            Some(u) => is_single_atom(u) && ends_command(u),
            None => is_single_atom(lower) && ends_command(lower),
        },
        _ => false,
    }
}

/// True if the node's LaTeX output ends with a plain alphabetic letter (a
/// variable name or text), which must be separated from a following letter or
/// command but may hug a following symbol (`x+1`).
fn ends_letter(node: &Node) -> bool {
    match node {
        Node::Ident(_) => true,
        Node::Number(n) => n.chars().last().map(|c| c.is_alphabetic()).unwrap_or(false),
        Node::Group(g) => g.last().map(ends_letter).unwrap_or(false),
        Node::Sup(_, s) | Node::Sub(_, s) => is_single_atom(s) && ends_letter(s),
        Node::Bounds { lower, upper, .. } => match upper {
            Some(u) => is_single_atom(u) && ends_letter(u),
            None => is_single_atom(lower) && ends_letter(lower),
        },
        _ => false,
    }
}

/// True if the node's LaTeX output begins with a letter or a `\command`, so it
/// must be separated from a preceding letter.
fn starts_letter_or_command(node: &Node) -> bool {
    match node {
        Node::Command(_)
        | Node::Decoration(_, _)
        | Node::Font(_, _)
        | Node::Frac(_, _)
        | Node::Atop(_, _)
        | Node::Sqrt(_)
        | Node::Root(_, _)
        | Node::Binom(_, _)
        | Node::Matrix { .. }
        | Node::Delimited { .. } => true,
        Node::Ident(_) => true,
        Node::Number(n) => n.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false),
        Node::Group(g) => g.first().map(starts_letter_or_command).unwrap_or(false),
        // A sub/superscript begins with its base operand.
        Node::Sup(base, _) | Node::Sub(base, _) => starts_letter_or_command(base),
        Node::Bounds { base, .. } => starts_letter_or_command(base),
        _ => false,
    }
}

/// Emits a multi-character identifier as upright text so variable-like unknowns
/// (`dx`, foreign words) don't render as italic math runs, while single letters
/// stay as ordinary math italics.
fn emit_ident(id: &str, out: &mut String, degraded: &mut Vec<String>) {
    if id.chars().count() == 1 {
        out.push_str(id);
    } else {
        // Multi-letter unknown identifier: render upright via \text{}.
        // If it *looks* like a command (length ≥ 2, all alphabetic) it is a
        // degraded conversion — a real EqEdit command we failed to recognise
        // (e.g. an uppercase `TIMES`). Record it so the caller can report the
        // loss; identifiers with digits/punctuation are ordinary labels.
        if is_command_like(id) {
            degraded.push(id.to_string());
        }
        out.push_str("\\text{");
        out.push_str(&escape_text(id));
        out.push('}');
    }
}

/// True if an identifier looks like an (unrecognised) command rather than a
/// plain label: at least two characters, all ASCII-alphabetic.
fn is_command_like(id: &str) -> bool {
    id.chars().count() >= 2 && id.chars().all(|c| c.is_ascii_alphabetic())
}

/// Emits a matrix/cases environment.
fn emit_matrix(env: &str, rows: &[Vec<Node>], out: &mut String, degraded: &mut Vec<String>) {
    out.push_str("\\begin{");
    out.push_str(env);
    out.push('}');
    for (r, row) in rows.iter().enumerate() {
        if r > 0 {
            out.push_str(" \\\\ ");
        }
        for (c, cell) in row.iter().enumerate() {
            if c > 0 {
                out.push_str(" & ");
            }
            emit_into(cell, out, degraded);
        }
    }
    out.push_str("\\end{");
    out.push_str(env);
    out.push('}');
}

/// Escapes a verbatim symbol char for LaTeX math mode.
fn escape_symbol(c: char) -> String {
    match c {
        '%' => "\\%".into(),
        '&' => "\\&".into(),
        '$' => "\\$".into(),
        '#' => "\\#".into(),
        '_' => "\\_".into(),
        '{' => "\\{".into(),
        '}' => "\\}".into(),
        other => other.to_string(),
    }
}

/// Escapes text destined for a `\text{ ... }` run.
fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '%' | '&' | '$' | '#' | '_' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            '\\' => out.push_str("\\textbackslash{}"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doclang::eqedit::{lexer::lex, parser::parse};

    fn conv(s: &str) -> String {
        emit(&parse(&lex(s)).expect("parse"))
    }

    #[test]
    fn emit_fraction() {
        assert_eq!(conv("1 over 2"), "\\frac{1}{2}");
    }

    #[test]
    fn emit_nested_fraction() {
        assert_eq!(conv("1 over {2 over 3}"), "\\frac{1}{\\frac{2}{3}}");
    }

    #[test]
    fn emit_sqrt() {
        assert_eq!(conv("sqrt {x+1}"), "\\sqrt{x+1}");
    }

    #[test]
    fn emit_root() {
        assert_eq!(conv("root 3 of {x}"), "\\sqrt[3]{x}");
    }

    #[test]
    fn emit_sup_single_vs_multi() {
        assert_eq!(conv("x^2"), "x^2");
        assert_eq!(conv("x^{12}"), "x^{12}");
    }

    #[test]
    fn emit_greek() {
        assert_eq!(conv("alpha"), "\\alpha");
        assert_eq!(conv("Omega"), "\\Omega");
        assert_eq!(conv("ALPHA"), "\\Alpha");
    }

    #[test]
    fn emit_binom() {
        assert_eq!(conv("binom {n} {k}"), "\\binom{n}{k}");
    }

    #[test]
    fn emit_added_noarg_symbols() {
        // Previously unrecognised symbols that fell through to \text{…}.
        assert_eq!(conv("perp"), "\\perp");
        assert_eq!(conv("parallel"), "\\parallel");
        assert_eq!(conv("langle"), "\\langle");
        assert_eq!(conv("setminus"), "\\setminus");
        assert_eq!(conv("longrightarrow"), "\\longrightarrow");
        // Case-insensitive lookup still applies.
        assert_eq!(conv("PERP"), "\\perp");
    }

    #[test]
    fn emit_added_decorations() {
        assert_eq!(conv("overline x"), "\\overline{x}");
        assert_eq!(conv("underline {x+1}"), "\\underline{x+1}");
        assert_eq!(conv("widehat A"), "\\widehat{A}");
        assert_eq!(conv("ddot x"), "\\ddot{x}");
    }

    #[test]
    fn unknown_word_still_passes_through_as_text() {
        // A genuinely unknown command-like word must still degrade, not resolve.
        assert_eq!(conv("notacommand"), "\\text{notacommand}");
    }
}
