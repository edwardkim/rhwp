//! Tokenizer for the HWP EqEdit equation script language.
//!
//! The lexer splits a raw script into a flat token stream that the parser
//! consumes.  It is intentionally permissive: anything it does not recognise
//! as structural punctuation becomes a [`Token::Word`] (an identifier-or-symbol
//! run) or [`Token::Number`], and the parser/emitter decide later whether a
//! word is a known command, a Greek letter, or unknown text to pass through.

/// A single lexical token from an EqEdit script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// An identifier-like run of letters/digits, e.g. `over`, `alpha`, `dx`, `x1`.
    Word(String),
    /// A numeric literal, e.g. `12` or `3.14`.
    Number(String),
    /// `{` — group open.
    LBrace,
    /// `}` — group close.
    RBrace,
    /// `^` — superscript marker (alias for the `sup` keyword).
    Caret,
    /// `_` — subscript marker (alias for the `sub` keyword).
    Underscore,
    /// `#` — matrix row separator.
    Hash,
    /// `&` — matrix column separator.
    Ampersand,
    /// A backtick `` ` `` — thin space.
    Backtick,
    /// A tilde `~` — normal space.
    Tilde,
    /// Any other single punctuation character that is meaningful verbatim
    /// in LaTeX, e.g. `+`, `-`, `=`, `(`, `)`, `,`, `<`, `>`, `[`, `]`, `|`.
    Symbol(char),
}

/// Returns `true` for characters that may appear inside a [`Token::Word`].
///
/// EqEdit identifiers are ASCII letters plus digits (so `x1`, `log2` stay
/// together) — but a word may not *start* with a digit (that path is handled
/// by number scanning).  Non-ASCII letters (e.g. already-Unicode Greek) are
/// also accepted so callers can feed lightly pre-processed scripts.
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || (c.is_alphabetic() && !c.is_ascii())
}

/// Tokenizes `script` into a flat [`Vec<Token>`].
///
/// This never fails: unrecognised characters become [`Token::Symbol`].
/// Whitespace separates tokens but is otherwise discarded (explicit spacing
/// uses the backtick / tilde tokens).
pub fn lex(script: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = script.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Whitespace: token separator, otherwise ignored.
            c if c.is_whitespace() => {
                chars.next();
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            '^' => {
                chars.next();
                tokens.push(Token::Caret);
            }
            '_' => {
                chars.next();
                tokens.push(Token::Underscore);
            }
            '#' => {
                chars.next();
                tokens.push(Token::Hash);
            }
            '&' => {
                chars.next();
                tokens.push(Token::Ampersand);
            }
            '`' => {
                chars.next();
                tokens.push(Token::Backtick);
            }
            '~' => {
                chars.next();
                tokens.push(Token::Tilde);
            }
            // Numbers: a run of digits with at most one interior decimal point.
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                let mut seen_dot = false;
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num.push(d);
                        chars.next();
                    } else if d == '.' && !seen_dot {
                        // Only consume the dot if a digit follows it.
                        let mut lookahead = chars.clone();
                        lookahead.next();
                        if matches!(lookahead.peek(), Some(d2) if d2.is_ascii_digit()) {
                            seen_dot = true;
                            num.push('.');
                            chars.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num));
            }
            // Words: a run of word characters.
            c if is_word_char(c) => {
                let mut word = String::new();
                while let Some(&w) = chars.peek() {
                    if is_word_char(w) {
                        word.push(w);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Word(word));
            }
            // Everything else is a verbatim symbol.
            other => {
                chars.next();
                tokens.push(Token::Symbol(other));
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_fraction() {
        assert_eq!(
            lex("1 over 2"),
            vec![
                Token::Number("1".into()),
                Token::Word("over".into()),
                Token::Number("2".into()),
            ]
        );
    }

    #[test]
    fn lex_braces_and_caret() {
        assert_eq!(
            lex("x^{2}"),
            vec![
                Token::Word("x".into()),
                Token::Caret,
                Token::LBrace,
                Token::Number("2".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn lex_matrix_separators() {
        assert_eq!(
            lex("a & b # c & d"),
            vec![
                Token::Word("a".into()),
                Token::Ampersand,
                Token::Word("b".into()),
                Token::Hash,
                Token::Word("c".into()),
                Token::Ampersand,
                Token::Word("d".into()),
            ]
        );
    }

    #[test]
    fn lex_decimal_number() {
        assert_eq!(lex("3.14"), vec![Token::Number("3.14".into())]);
    }

    #[test]
    fn lex_trailing_dot_not_part_of_number() {
        // "1." -> number "1" then symbol "."
        assert_eq!(
            lex("1."),
            vec![Token::Number("1".into()), Token::Symbol('.')]
        );
    }

    #[test]
    fn lex_spaces_thin_and_normal() {
        assert_eq!(
            lex("a ` b ~ c")
                .into_iter()
                .filter(|t| matches!(t, Token::Backtick | Token::Tilde))
                .count(),
            2
        );
    }

    #[test]
    fn lex_empty() {
        assert_eq!(lex(""), Vec::<Token>::new());
        assert_eq!(lex("   \n\t "), Vec::<Token>::new());
    }

    #[test]
    fn lex_alnum_word() {
        assert_eq!(lex("x1"), vec![Token::Word("x1".into())]);
        assert_eq!(lex("log2"), vec![Token::Word("log2".into())]);
    }
}
