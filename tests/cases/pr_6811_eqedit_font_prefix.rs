use rhwp::doclang::eqedit::{convert, convert_with_degraded};

fn check(script: &str, expected: &str) {
    assert_eq!(convert(script).unwrap(), expected, "script: {script:?}");
}

/// EqEdit lets a font switch run straight into its argument with no
/// space. Every test above uses the spaced form, which is how the
/// unspaced one went unhandled: the whole word missed the command table
/// and fell through to an identifier, emitting a literal `\text{rmC}`.
///
/// That is a wrong symbol rather than a missing one — `\text{rmm}` reads
/// as "rmm" where the source says the unit "m", and `\mu \text{rmC}` as
/// "μrmC" where it says "μC". Both forms must produce the same LaTeX.
#[test]
fn font_switches_without_a_space() {
    check("rmd", "\\mathrm{d}");
    check("itx", "\\mathit{x}");
    check("boldv", "\\mathbf{v}");
    // The literals above pin the value; this pins the contract, which is
    // that the two spellings are interchangeable. Stated as an equivalence
    // because a multi-character argument nests a `\text{…}` inside the
    // font switch, and how the backend renders an identifier is a separate
    // concern from whether the switch was recognised at all. The words are
    // the ones the corpus carries: `rmAgCl(its)` for AgCl(s), `(itaq)` for
    // the aqueous marker.
    for (unspaced, spaced) in [
        ("rmAgCl", "rm AgCl"),
        ("itaq", "it aq"),
        ("rmC", "rm C"),
        ("boldPQ", "bold PQ"),
    ] {
        assert_eq!(
            convert(unspaced).unwrap(),
            convert(spaced).unwrap(),
            "{unspaced:?} must convert as {spaced:?} does"
        );
    }
    // Only a *leading* switch splits. `mu rmC` is two tokens in the source
    // and stays two; the run-together `murmC` is one word the corpus means
    // literally, and must not be mined for an interior `rm`.
    check("mu rmC", "\\mu \\mathrm{C}");
    check("murmC", "\\text{murmC}");
}

/// The split must not fire on ordinary words that merely begin with those
/// letters. The corpus carries `mod`, `out`, `EMP`, `DNO`, `MSE` and
/// `pit` as genuine content, and a blunter rule keyed on any command
/// prefix would turn `pit` into π followed by t. Decorations are excluded
/// for the same reason: `bar` is a pressure unit, and `dot`, `vec` and
/// `check` are ordinary words.
#[test]
fn ordinary_words_are_not_split() {
    for word in [
        "mod", "out", "EMP", "DNO", "MSE", "pit", "ideal", "atm", "kHz",
    ] {
        let latex = convert(word).expect("converts");
        assert!(
            !latex.contains("\\mathrm")
                && !latex.contains("\\mathit")
                && !latex.contains("\\mathbf"),
            "{word:?} must not be split into a font switch, got {latex:?}"
        );
    }
}

#[test]
fn glued_font_prefixes_reuse_structural_and_numeric_parsing() {
    for (glued, spaced) in [
        ("rmboldv", "rm bold v"),
        ("bolditx", "bold it x"),
        ("rmsqrt{x}", "rm sqrt{x}"),
        ("rmbar{x}", "rm bar{x}"),
        ("rm12.5", "rm 12.5"),
        ("bold12x", "bold 12 x"),
        ("{rmC}^2", "{rm C}^2"),
        ("RMboldv", "RM bold v"),
    ] {
        assert_eq!(
            convert_with_degraded(glued).unwrap(),
            convert_with_degraded(spaced).unwrap(),
            "{glued:?} must preserve the spaced form {spaced:?}",
        );
    }
}

#[test]
fn unicode_and_complete_commands_remain_intact() {
    for word in ["alpha", "GAMMA", "infty", "한글", "가나다", "αβγ"] {
        assert_eq!(
            rhwp::doclang::eqedit::lexer::lex(word),
            vec![rhwp::doclang::eqedit::lexer::Token::Word(word.into())],
        );
    }
}
