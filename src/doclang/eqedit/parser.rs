//! Recursive-descent parser turning an EqEdit [`Token`] stream into a small AST.
//!
//! Grammar (informal, lowest-to-highest precedence):
//!
//! ```text
//! expr      := frac
//! frac      := script ( ("over" | "atop") script )*        // left-assoc fraction
//! script    := postfix ( ("^"|"sup"|"_"|"sub") postfix )*  // sup/sub
//! postfix   := atom ( "from" group "to" group )?           // bounds on sums/ints/lim
//! atom      := group | command | greek | word | number | symbol | space
//! group     := "{" expr* "}"
//! ```
//!
//! The parser is **permissive**: unknown words become [`Node::Ident`] and are
//! emitted verbatim/escaped by the LaTeX backend.  The only hard error is an
//! unbalanced brace that cannot be recovered.

use crate::doclang::eqedit::error::EqError;
use crate::doclang::eqedit::lexer::Token;

/// One AST node of a parsed EqEdit expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A literal number, emitted verbatim.
    Number(String),
    /// An identifier / unknown word (variable name or unrecognised command).
    Ident(String),
    /// A verbatim symbol such as `+`, `=`, `(`.
    Symbol(char),
    /// A thin space (`` ` ``) — emitted as `\,`.
    ThinSpace,
    /// A normal space (`~`) — emitted as `\ `.
    Space,
    /// A sequence of nodes (a group `{ ... }` or the top-level expression).
    Group(Vec<Node>),
    /// `a over b` → `\frac{a}{b}`.
    Frac(Box<Node>, Box<Node>),
    /// `a atop b` → `{a \atop b}`.
    Atop(Box<Node>, Box<Node>),
    /// `base ^ exp`.
    Sup(Box<Node>, Box<Node>),
    /// `base _ sub`.
    Sub(Box<Node>, Box<Node>),
    /// `sqrt {x}` → `\sqrt{x}`.
    Sqrt(Box<Node>),
    /// `root n of {x}` → `\sqrt[n]{x}`.
    Root(Box<Node>, Box<Node>),
    /// A named LaTeX command with no argument, e.g. `\alpha`, `\sum`, `\times`.
    /// The string is the LaTeX command name **without** the leading backslash.
    Command(String),
    /// A decoration accent, e.g. `bar x` → `\bar{x}`. First field is the LaTeX
    /// accent command name (without backslash); second is the decorated node.
    Decoration(String, Box<Node>),
    /// `from a to b` bounds attached to a preceding operator (sum/int/lim/…).
    /// `lower` is always present; `upper` is optional (`from` without `to`).
    Bounds {
        /// The operator the bounds attach to (e.g. a `Command("sum")`).
        base: Box<Node>,
        /// The lower bound (`from` argument).
        lower: Box<Node>,
        /// The upper bound (`to` argument), if any.
        upper: Option<Box<Node>>,
    },
    /// `left ( ... right )` delimited group. `open`/`close` are the delimiter
    /// strings already translated to LaTeX (e.g. `(`, `\{`, `.`).
    Delimited {
        /// LaTeX form of the opening delimiter (or `.` for none).
        open: String,
        /// The enclosed body.
        body: Box<Node>,
        /// LaTeX form of the closing delimiter (or `.` for none).
        close: String,
    },
    /// A matrix-like environment: `matrix`, `pmatrix`, `bmatrix`, `dmatrix`,
    /// `cases`. `env` is the LaTeX environment name; `rows`/cols hold the cells.
    Matrix {
        /// LaTeX environment name (e.g. `matrix`, `pmatrix`, `cases`).
        env: String,
        /// Rows of cells; each cell is itself a node.
        rows: Vec<Vec<Node>>,
    },
    /// `binom {a} {b}` → `\binom{a}{b}`.
    Binom(Box<Node>, Box<Node>),
    /// A font switch applied to the following node, e.g. `it x` → `\mathit{x}`.
    Font(String, Box<Node>),
}

/// Maximum equation nesting depth accepted by the recursive-descent parser.
///
/// Adversarial input (`{{{…}}}`, `sqrt sqrt …`, `bar bar …`, deep `matrix`/`left`
/// nesting, or long `over`/`sup`/`sub` chains) would otherwise drive unbounded
/// recursion — and, even when built iteratively, an unbounded-depth [`Node`] tree
/// whose recursive `Drop`/LaTeX emit overflows the stack. This cap mirrors the
/// sibling parser guards (`MAX_HWPX_SECTION_DEPTH = 64`, `MAX_HWP5_SHAPE_DEPTH`,
/// `MAX_DRAWING_OBJECT_DEPTH`) and sits far below the measured debug-build stack
/// limit (~150 nested groups) while dwarfing any real equation's nesting.
pub const MAX_EQ_DEPTH: u32 = 64;

/// Parses a full token stream into a top-level [`Node::Group`].
///
/// Returns [`EqError::UnbalancedBrace`] if a `}` appears with no matching `{`,
/// or a `{` is never closed, and [`EqError::TooDeep`] if the script nests beyond
/// [`MAX_EQ_DEPTH`].
pub fn parse(tokens: &[Token]) -> Result<Node, EqError> {
    let mut p = Parser {
        tokens,
        pos: 0,
        depth: 0,
    };
    let nodes = p.parse_seq(/* in_group = */ false)?;
    Ok(Node::Group(nodes))
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    /// Current recursion depth, tracked at the [`Parser::parse_atom`] funnel so
    /// unbounded nesting is rejected before it can overflow the stack.
    depth: u32,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    /// Parses a sequence of expressions until end-of-input (top level) or a
    /// closing brace (inside a group). When `in_group` is true, a matching `}`
    /// is required and consumed; its absence is an unbalanced-brace error.
    fn parse_seq(&mut self, in_group: bool) -> Result<Vec<Node>, EqError> {
        let mut nodes = Vec::new();
        loop {
            match self.peek() {
                None => {
                    if in_group {
                        // Opened a group that was never closed.
                        return Err(EqError::UnbalancedBrace);
                    }
                    break;
                }
                Some(Token::RBrace) => {
                    if in_group {
                        self.next(); // consume the matching '}'
                        break;
                    }
                    // Stray '}' at top level with no opener.
                    return Err(EqError::UnbalancedBrace);
                }
                // Separators terminate a sequence parsed by a matrix collector;
                // at the ordinary sequence level they should not appear, but if
                // they do (malformed input), stop so the caller can handle them.
                Some(Token::Hash) | Some(Token::Ampersand) => break,
                _ => nodes.push(self.parse_frac()?),
            }
        }
        Ok(nodes)
    }

    /// Lowest precedence: fraction operators `over` / `atop` (left-associative).
    ///
    /// A long `a over b over c …` chain builds a left-nested tree iteratively, so
    /// the [`parse_atom`](Self::parse_atom) depth guard never fires; the resulting
    /// deep tree would overflow the stack on recursive `Drop`/emit. Cap the chain
    /// length at [`MAX_EQ_DEPTH`] so the produced tree depth stays bounded.
    fn parse_frac(&mut self) -> Result<Node, EqError> {
        let mut left = self.parse_script()?;
        let mut chain = 0u32;
        while let Some(Token::Word(w)) = self.peek() {
            if w.eq_ignore_ascii_case("over") {
                chain += 1;
                if chain > MAX_EQ_DEPTH {
                    return Err(EqError::TooDeep);
                }
                self.next();
                let right = self.parse_script()?;
                left = Node::Frac(Box::new(left), Box::new(right));
            } else if w.eq_ignore_ascii_case("atop") {
                chain += 1;
                if chain > MAX_EQ_DEPTH {
                    return Err(EqError::TooDeep);
                }
                self.next();
                let right = self.parse_script()?;
                left = Node::Atop(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Superscript / subscript via `^`, `_`, or the `sup` / `sub` keywords.
    ///
    /// As with [`parse_frac`](Self::parse_frac), a long `x^a^b…` / `x_a_b…` chain
    /// builds a deep left-nested tree iteratively, so the chain length is capped
    /// at [`MAX_EQ_DEPTH`] to keep the tree shallow enough for recursive
    /// `Drop`/emit.
    fn parse_script(&mut self) -> Result<Node, EqError> {
        let mut base = self.parse_postfix()?;
        let mut chain = 0u32;
        loop {
            let is_script = matches!(self.peek(), Some(Token::Caret) | Some(Token::Underscore))
                || matches!(self.peek(), Some(Token::Word(w))
                if w.eq_ignore_ascii_case("sup") || w.eq_ignore_ascii_case("sub"));
            if is_script {
                chain += 1;
                if chain > MAX_EQ_DEPTH {
                    return Err(EqError::TooDeep);
                }
            }
            match self.peek() {
                Some(Token::Caret) => {
                    self.next();
                    let exp = self.parse_postfix()?;
                    base = Node::Sup(Box::new(base), Box::new(exp));
                }
                Some(Token::Underscore) => {
                    self.next();
                    let sub = self.parse_postfix()?;
                    base = Node::Sub(Box::new(base), Box::new(sub));
                }
                Some(Token::Word(w)) if w.eq_ignore_ascii_case("sup") => {
                    self.next();
                    let exp = self.parse_postfix()?;
                    base = Node::Sup(Box::new(base), Box::new(exp));
                }
                Some(Token::Word(w)) if w.eq_ignore_ascii_case("sub") => {
                    self.next();
                    let sub = self.parse_postfix()?;
                    base = Node::Sub(Box::new(base), Box::new(sub));
                }
                _ => break,
            }
        }
        Ok(base)
    }

    /// Optional `from … to …` bounds following an atom (e.g. `sum from a to b`).
    fn parse_postfix(&mut self) -> Result<Node, EqError> {
        let base = self.parse_atom()?;
        // Bound arguments are parsed as single atoms so that `from … to …`
        // keywords are not swallowed by a lower-precedence parse.
        if let Some(Token::Word(w)) = self.peek() {
            if w.eq_ignore_ascii_case("from") {
                self.next();
                let lower = self.parse_atom()?;
                let upper = if matches!(self.peek(), Some(Token::Word(w)) if w.eq_ignore_ascii_case("to"))
                {
                    self.next();
                    Some(Box::new(self.parse_atom()?))
                } else {
                    None
                };
                return Ok(Node::Bounds {
                    base: Box::new(base),
                    lower: Box::new(lower),
                    upper,
                });
            }
            if w.eq_ignore_ascii_case("to") {
                // `to` without a preceding `from` — treat as bounds with only upper.
                self.next();
                let upper = self.parse_atom()?;
                return Ok(Node::Bounds {
                    base: Box::new(base),
                    lower: Box::new(Node::Group(vec![])),
                    upper: Some(Box::new(upper)),
                });
            }
        }
        Ok(base)
    }

    /// A single atomic unit, including prefix commands that consume arguments.
    ///
    /// This is the universal recursion funnel: nested groups (`{`), command
    /// arguments (`sqrt`/`root`/`binom`/decorations/fonts), matrix cells, and
    /// `left … right` bodies all re-enter here one level deeper. Guarding it with
    /// a depth counter therefore bounds *every* nesting path; exceeding
    /// [`MAX_EQ_DEPTH`] returns [`EqError::TooDeep`] instead of overflowing.
    fn parse_atom(&mut self) -> Result<Node, EqError> {
        self.depth += 1;
        if self.depth > MAX_EQ_DEPTH {
            self.depth -= 1;
            return Err(EqError::TooDeep);
        }
        let r = self.parse_atom_inner();
        self.depth -= 1;
        r
    }

    fn parse_atom_inner(&mut self) -> Result<Node, EqError> {
        match self.peek() {
            None => Ok(Node::Group(vec![])),
            Some(Token::LBrace) => {
                self.next();
                let inner = self.parse_seq(true)?;
                Ok(Node::Group(inner))
            }
            Some(Token::Number(n)) => {
                let n = n.clone();
                self.next();
                Ok(Node::Number(n))
            }
            Some(Token::Backtick) => {
                self.next();
                Ok(Node::ThinSpace)
            }
            Some(Token::Tilde) => {
                self.next();
                Ok(Node::Space)
            }
            Some(Token::Symbol(c)) => {
                let c = *c;
                self.next();
                Ok(Node::Symbol(c))
            }
            // Stray separators handled by the sequence layer; if reached here,
            // surface them as symbols so nothing is silently dropped.
            Some(Token::Hash) => {
                self.next();
                Ok(Node::Symbol('#'))
            }
            Some(Token::Ampersand) => {
                self.next();
                Ok(Node::Symbol('&'))
            }
            Some(Token::RBrace) => {
                // Reached by parse_atom only via malformed nesting.
                Err(EqError::UnbalancedBrace)
            }
            Some(Token::Caret) | Some(Token::Underscore) => {
                // A script operator with no base: treat the base as empty group.
                Ok(Node::Group(vec![]))
            }
            Some(Token::Word(_)) => self.parse_word(),
        }
    }

    /// Dispatches a word token: prefix commands consume arguments; decorations,
    /// roots, matrices, delimiters, fonts, and bare identifiers are handled here.
    fn parse_word(&mut self) -> Result<Node, EqError> {
        let word = match self.next() {
            Some(Token::Word(w)) => w.clone(),
            _ => unreachable!("parse_word called without a leading word"),
        };

        // Structural keywords are matched case-insensitively (HWP EqEdit treats
        // commands case-insensitively). We match on a lowercased copy; the
        // original `word` is preserved for `classify_word`/`decoration_command`
        // so non-command identifiers pass through with their original casing.
        let kw = word.to_ascii_lowercase();
        match kw.as_str() {
            "sqrt" => {
                let arg = self.parse_atom()?;
                Ok(Node::Sqrt(Box::new(arg)))
            }
            "root" => {
                // `root n of {x}` ; tolerate missing `of`.
                let index = self.parse_atom()?;
                if matches!(self.peek(), Some(Token::Word(w)) if w.eq_ignore_ascii_case("of")) {
                    self.next();
                }
                let radicand = self.parse_atom()?;
                Ok(Node::Root(Box::new(index), Box::new(radicand)))
            }
            "binom" => {
                let a = self.parse_atom()?;
                let b = self.parse_atom()?;
                Ok(Node::Binom(Box::new(a), Box::new(b)))
            }
            "left" => self.parse_delimited(),
            "matrix" => self.parse_matrix("matrix"),
            "pmatrix" => self.parse_matrix("pmatrix"),
            "bmatrix" => self.parse_matrix("bmatrix"),
            "dmatrix" => self.parse_matrix("vmatrix"),
            "cases" => self.parse_matrix("cases"),
            // Decorations: accent / over-under command applies to the following atom.
            "bar" | "hat" | "vec" | "dot" | "ddot" | "tilde" | "acute" | "grave" | "check"
            | "breve" | "mathring" | "widehat" | "widetilde" | "overline" | "underline"
            | "overbrace" | "underbrace" | "overrightarrow" => {
                let accent = decoration_command(&kw);
                let target = self.parse_atom()?;
                Ok(Node::Decoration(accent.to_string(), Box::new(target)))
            }
            // Font switches apply to the following atom.
            "it" => {
                let target = self.parse_atom()?;
                Ok(Node::Font("mathit".into(), Box::new(target)))
            }
            "rm" => {
                let target = self.parse_atom()?;
                Ok(Node::Font("mathrm".into(), Box::new(target)))
            }
            "bold" => {
                let target = self.parse_atom()?;
                Ok(Node::Font("mathbf".into(), Box::new(target)))
            }
            _ => {
                // Either a known no-argument command (greek, operators, …) or an
                // unknown identifier; the LaTeX backend resolves which. The
                // original casing is preserved so uppercase Greek (`GAMMA`) and
                // mixed-case passthrough words are handled correctly.
                Ok(classify_word(&word))
            }
        }
    }

    /// Parses `left <delim> … right <delim>`. The opening `left` keyword has
    /// already been consumed.
    fn parse_delimited(&mut self) -> Result<Node, EqError> {
        let open = self.parse_delimiter();
        // Body: parse fractions until we hit a `right` keyword or run out.
        let mut body = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Word(w)) if w.eq_ignore_ascii_case("right") => {
                    self.next();
                    break;
                }
                None => break, // tolerate missing `right`
                Some(Token::RBrace) => {
                    return Err(EqError::UnbalancedBrace);
                }
                _ => body.push(self.parse_frac()?),
            }
        }
        let close = self.parse_delimiter();
        Ok(Node::Delimited {
            open,
            body: Box::new(Node::Group(body)),
            close,
        })
    }

    /// Reads the delimiter token after a `left` / `right` keyword and returns
    /// its LaTeX form. A missing/`.`/`none` delimiter yields `.`.
    fn parse_delimiter(&mut self) -> String {
        match self.peek() {
            Some(Token::Symbol(c)) => {
                let c = *c;
                self.next();
                match c {
                    '(' => "(".into(),
                    ')' => ")".into(),
                    '[' => "[".into(),
                    ']' => "]".into(),
                    '|' => "|".into(),
                    '.' => ".".into(),
                    other => other.to_string(),
                }
            }
            Some(Token::LBrace) => {
                self.next();
                "\\{".into()
            }
            Some(Token::RBrace) => {
                self.next();
                "\\}".into()
            }
            Some(Token::Word(w)) if w.eq_ignore_ascii_case("none") => {
                self.next();
                ".".into()
            }
            _ => ".".into(),
        }
    }

    /// Parses the body of a matrix/cases environment: `{ cell & cell # cell & cell }`.
    /// The environment keyword has already been consumed.
    fn parse_matrix(&mut self, env: &str) -> Result<Node, EqError> {
        // Expect an opening brace; if absent, produce an empty matrix.
        if !matches!(self.peek(), Some(Token::LBrace)) {
            return Ok(Node::Matrix {
                env: env.to_string(),
                rows: vec![],
            });
        }
        self.next(); // consume '{'

        let mut rows: Vec<Vec<Node>> = Vec::new();
        let mut current_row: Vec<Node> = Vec::new();
        let mut current_cell: Vec<Node> = Vec::new();

        loop {
            match self.peek() {
                None => return Err(EqError::UnbalancedBrace),
                Some(Token::RBrace) => {
                    self.next();
                    current_row.push(wrap_cell(current_cell));
                    rows.push(current_row);
                    break;
                }
                Some(Token::Ampersand) => {
                    self.next();
                    current_row.push(wrap_cell(std::mem::take(&mut current_cell)));
                }
                Some(Token::Hash) => {
                    self.next();
                    current_row.push(wrap_cell(std::mem::take(&mut current_cell)));
                    rows.push(std::mem::take(&mut current_row));
                }
                _ => current_cell.push(self.parse_frac()?),
            }
        }

        Ok(Node::Matrix {
            env: env.to_string(),
            rows,
        })
    }
}

/// Wraps a cell's node list into a single node.
fn wrap_cell(nodes: Vec<Node>) -> Node {
    Node::Group(nodes)
}

/// Maps an EqEdit decoration keyword to its LaTeX accent command (no backslash).
fn decoration_command(word: &str) -> &'static str {
    match word {
        "bar" => "bar",
        "hat" => "hat",
        "vec" => "vec",
        "dot" => "dot",
        "ddot" => "ddot",
        "tilde" => "tilde",
        "acute" => "acute",
        "grave" => "grave",
        "check" => "check",
        "breve" => "breve",
        "mathring" => "mathring",
        "widehat" => "widehat",
        "widetilde" => "widetilde",
        "overline" => "overline",
        "underline" => "underline",
        "overbrace" => "overbrace",
        "underbrace" => "underbrace",
        "overrightarrow" => "overrightarrow",
        _ => "bar",
    }
}

/// Classifies a bare word as a known no-argument LaTeX command or a plain
/// identifier. The mapping table lives in the LaTeX backend; here we only need
/// to know whether it resolves to a command.
fn classify_word(word: &str) -> Node {
    if crate::doclang::eqedit::latex::command_for(word).is_some() {
        Node::Command(word.to_string())
    } else {
        Node::Ident(word.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doclang::eqedit::lexer::lex;

    fn ast(s: &str) -> Node {
        parse(&lex(s)).expect("parse should succeed")
    }

    #[test]
    fn parse_fraction() {
        match ast("1 over 2") {
            Node::Group(g) => match &g[0] {
                Node::Frac(a, b) => {
                    assert_eq!(**a, Node::Number("1".into()));
                    assert_eq!(**b, Node::Number("2".into()));
                }
                other => panic!("expected Frac, got {other:?}"),
            },
            other => panic!("expected Group, got {other:?}"),
        }
    }

    #[test]
    fn parse_unbalanced_open_brace_errors() {
        assert_eq!(parse(&lex("{ x")), Err(EqError::UnbalancedBrace));
    }

    #[test]
    fn parse_unbalanced_close_brace_errors() {
        assert_eq!(parse(&lex("x }")), Err(EqError::UnbalancedBrace));
    }

    #[test]
    fn parse_sup_sub() {
        // x^2 -> Sup(x, 2)
        match &ast("x^2") {
            Node::Group(g) => assert!(matches!(g[0], Node::Sup(_, _))),
            _ => panic!(),
        }
        match &ast("x_i") {
            Node::Group(g) => assert!(matches!(g[0], Node::Sub(_, _))),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_bounds() {
        match &ast("sum from {i=1} to {n}") {
            Node::Group(g) => assert!(matches!(g[0], Node::Bounds { .. })),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_matrix_rows() {
        match &ast("matrix{a & b # c & d}") {
            Node::Group(g) => match &g[0] {
                Node::Matrix { rows, .. } => {
                    assert_eq!(rows.len(), 2);
                    assert_eq!(rows[0].len(), 2);
                    assert_eq!(rows[1].len(), 2);
                }
                other => panic!("expected Matrix, got {other:?}"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn parse_empty_is_empty_group() {
        assert_eq!(parse(&lex("")), Ok(Node::Group(vec![])));
    }

    // --- DoS 하드닝 회귀 (적대적 깊은 중첩) ---
    // 가드가 없으면 아래 입력들은 재귀 하강 파서(깊은 그룹/명령)나, 반복으로 쌓인
    // 깊은 AST 의 재귀적 Drop/LaTeX emit 에서 스택을 오버플로한다. 상한 초과 시
    // `EqError::TooDeep` 로 우아하게 실패해야 한다.

    #[test]
    fn dos_deeply_nested_groups_return_toodeep_not_overflow() {
        let s = "{".repeat(5000) + "x" + &"}".repeat(5000);
        assert_eq!(parse(&lex(&s)), Err(EqError::TooDeep));
    }

    #[test]
    fn dos_deeply_nested_commands_return_toodeep() {
        let sqrt = "sqrt ".repeat(5000) + "x";
        assert_eq!(parse(&lex(&sqrt)), Err(EqError::TooDeep));
        let bar = "bar ".repeat(5000) + "x";
        assert_eq!(parse(&lex(&bar)), Err(EqError::TooDeep));
        let root = "root ".repeat(5000) + "x";
        assert_eq!(parse(&lex(&root)), Err(EqError::TooDeep));
    }

    #[test]
    fn dos_long_over_and_script_chains_return_toodeep() {
        // 반복으로 쌓이는 좌편향 깊은 트리 — parse_atom 재귀 가드로는 못 막고
        // 연쇄 길이 캡으로 막는다.
        let over = "1 ".to_string() + &"over 1 ".repeat(5000);
        assert_eq!(parse(&lex(&over)), Err(EqError::TooDeep));
        let atop = "1 ".to_string() + &"atop 1 ".repeat(5000);
        assert_eq!(parse(&lex(&atop)), Err(EqError::TooDeep));
        let sup = "x".to_string() + &"^x".repeat(5000);
        assert_eq!(parse(&lex(&sup)), Err(EqError::TooDeep));
        let sub = "x".to_string() + &"_x".repeat(5000);
        assert_eq!(parse(&lex(&sub)), Err(EqError::TooDeep));
    }

    #[test]
    fn dos_nested_matrix_returns_toodeep() {
        let s = "matrix{".repeat(5000) + "a" + &"}".repeat(5000);
        assert_eq!(parse(&lex(&s)), Err(EqError::TooDeep));
    }

    #[test]
    fn valid_moderate_nesting_below_cap_still_parses() {
        // 상한 한참 아래(30단계) 균형 중괄호는 정상 파싱되어 `x` 로 수렴한다.
        let n = 30;
        assert!(n < MAX_EQ_DEPTH as usize);
        let s = "{".repeat(n) + "x" + &"}".repeat(n);
        assert_eq!(super::super::convert(&s).unwrap(), "x");
    }
}
