//! 한컴 수식 명령어 → Unicode 매핑 테이블
//!
//! 수식 스크립트 버전 6.0
//! 참조: openhwp/docs/hwpx/appendix-i-formula.md

use std::collections::HashMap;
use std::sync::LazyLock;

/// 그리스 문자 (소문자, 대소문자 구분)
static GREEK_LOWER: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("alpha", "α"), ("beta", "β"), ("gamma", "γ"), ("delta", "δ"),
        ("epsilon", "ε"), ("varepsilon", "ε"), ("zeta", "ζ"), ("eta", "η"),
        ("theta", "θ"), ("vartheta", "ϑ"), ("iota", "ι"), ("kappa", "κ"),
        ("lambda", "λ"), ("mu", "μ"), ("nu", "ν"), ("xi", "ξ"),
        ("omicron", "ο"), ("pi", "π"), ("varpi", "ϖ"), ("rho", "ρ"),
        ("sigma", "σ"), ("varsigma", "ς"), ("tau", "τ"), ("upsilon", "υ"),
        ("phi", "φ"), ("varphi", "φ"), ("chi", "χ"), ("psi", "ψ"),
        ("omega", "ω"),
    ])
});

/// 그리스 문자 (대문자, 대소문자 구분)
static GREEK_UPPER: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("Alpha", "Α"), ("Beta", "Β"), ("Gamma", "Γ"), ("Delta", "Δ"),
        ("Epsilon", "Ε"), ("Zeta", "Ζ"), ("Eta", "Η"), ("Theta", "Θ"),
        ("Iota", "Ι"), ("Kappa", "Κ"), ("Lambda", "Λ"), ("Mu", "Μ"),
        ("Nu", "Ν"), ("Xi", "Ξ"), ("Omicron", "Ο"), ("Pi", "Π"),
        ("Rho", "Ρ"), ("Sigma", "Σ"), ("Tau", "Τ"), ("Upsilon", "Υ"),
        ("varupsilon", "ϒ"), ("Phi", "Φ"), ("Chi", "Χ"), ("Psi", "Ψ"),
        ("Omega", "Ω"),
    ])
});

/// 특수 문자 및 기호
static SPECIAL_SYMBOLS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("INF", "∞"), ("ALEPH", "ℵ"), ("HBAR", "ℏ"),
        ("IMATH", "ı"), ("JMATH", "ȷ"), ("ELL", "ℓ"), ("LITER", "ℓ"),
        ("WP", "℘"), ("IMAG", "ℑ"), ("image", "ℑ"), ("REIMAGE", "ℜ"),
        ("ANGSTROM", "Å"), ("MHO", "℧"), ("OHM", "Ω"),
        ("CDOTS", "⋯"), ("LDOTS", "…"), ("VDOTS", "⋮"), ("DDOTS", "⋱"),
        ("DOTS", "…"),
        // LaTeX spacing
        ("QUAD", "\u{2003}"), ("QQUAD", "\u{2003}\u{2003}"),
        ("THINSPACE", "\u{2009}"), ("MEDSPACE", "\u{205F}"), ("THICKSPACE", "\u{2004}"),
        ("NEGSPACE", ""), ("ENSPACE", "\u{2002}"),
        ("TRIANGLE", "△"), ("TRIANGLED", "▽"),
        ("ANGLE", "∠"), ("MSANGLE", "∡"), ("SANGLE", "∢"), ("RTANGLE", "⊾"),
        ("BOT", "⊥"), ("TOP", "⊤"),
        ("LAPLACE", "ℒ"), ("CENTIGRADE", "℃"), ("FAHRENHEIT", "℉"),
        ("DEG", "°"), ("prime", "′"),
        ("LSLANT", "/"), ("RSLANT", "\\"),
        ("ATT", "@"), ("HUND", "‰"), ("THOU", "‱"), ("WELL", "♯"),
        ("BASE", "△"), ("BENZENE", "⌬"),
    ])
});

/// 연산자 및 관계 기호
static OPERATORS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // 산술
        ("TIMES", "×"), ("DIV", "÷"), ("DIVDE", "÷"),
        ("PLUSMINUS", "±"), ("PM", "±"), ("MINUSPLUS", "∓"), ("MP", "∓"),
        ("CDOT", "·"), ("CIRC", "∘"), ("BULLET", "•"),
        ("AST", "∗"), ("STAR", "★"), ("DSUM", "⊞"),
        // 비교/관계
        ("NEQ", "≠"), ("!=", "≠"),
        ("LE", "≤"), ("LEQ", "≤"), ("GE", "≥"), ("GEQ", "≥"),
        ("<=", "≤"), (">=", "≥"),
        ("<<", "≪"), (">>", "≫"),
        ("LLL", "⋘"), ("<<<", "⋘"), ("GGG", "⋙"), (">>>", "⋙"),
        ("APPROX", "≈"), ("SIM", "∼"), ("SIMEQ", "≃"),
        ("CONG", "≅"), ("EQUIV", "≡"), ("==", "≡"),
        ("ASYMP", "≍"), ("DOTEQ", "≐"), ("PROPTO", "∝"),
        // 집합/논리
        ("SUBSET", "⊂"), ("SUPERSET", "⊃"),
        ("SUBSETEQ", "⊆"), ("SUPSETEQ", "⊇"),
        ("SQSUBSET", "⊏"), ("SQSUPSET", "⊐"),
        ("SQSUBSETEQ", "⊑"), ("SQSUPSETEQ", "⊒"),
        ("IN", "∈"), ("NOTIN", "∉"), ("OWNS", "∋"), ("NI", "∋"),
        ("PREC", "≺"), ("SUCC", "≻"),
        ("FORALL", "∀"), ("EXIST", "∃"), ("LNOT", "¬"),
        ("WEDGE", "∧"), ("LAND", "∧"), ("VEE", "∨"), ("LOR", "∨"),
        ("XOR", "⊻"),
        // 기타
        ("PARTIAL", "∂"), ("EMPTYSET", "∅"),
        ("THEREFORE", "∴"), ("BECAUSE", "∵"), ("IDENTICAL", "∷"),
        ("VDASH", "⊢"), ("HLEFT", "⊣"), ("MODELS", "⊨"),
        ("DAGGER", "†"), ("DDAGGER", "‡"),
        ("BIGCIRC", "○"), ("DIAMOND", "◇"), ("ISO", "⋄"),
        // LaTeX aliases
        ("ne", "≠"), ("neq", "≠"), ("le", "≤"), ("leq", "≤"),
        ("ge", "≥"), ("geq", "≥"), ("ll", "≪"), ("gg", "≫"),
        ("approx", "≈"), ("sim", "∼"), ("simeq", "≃"),
        ("cong", "≅"), ("equiv", "≡"), ("propto", "∝"),
        ("subset", "⊂"), ("supset", "⊃"),
        ("subseteq", "⊆"), ("supseteq", "⊇"),
        ("in", "∈"), ("notin", "∉"), ("ni", "∋"),
        ("forall", "∀"), ("exists", "∃"), ("nexists", "∄"),
        ("lnot", "¬"), ("neg", "¬"),
        ("wedge", "∧"), ("land", "∧"), ("vee", "∨"), ("lor", "∨"),
        ("nabla", "∇"), ("partial", "∂"), ("emptyset", "∅"),
        ("infty", "∞"), ("aleph", "ℵ"),
        ("therefore", "∴"), ("because", "∵"),
        ("cdot", "·"), ("times", "×"), ("div", "÷"),
        ("pm", "±"), ("mp", "∓"),
        ("cup", "∪"), ("cap", "∩"),
        ("vdash", "⊢"), ("models", "⊨"),
        ("oplus", "⊕"), ("otimes", "⊗"),
        ("dagger", "†"), ("ddagger", "‡"),
        ("star", "★"), ("circ", "∘"),
        ("perp", "⊥"),
    ])
});

/// 큰 연산자 (적분, 합, 곱, 집합)
static BIG_OPERATORS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // 적분
        ("INT", "∫"), ("INTEGRAL", "∫"), ("SMALLINT", "∫"),
        ("DINT", "∬"), ("TINT", "∭"),
        ("OINT", "∮"), ("SMALLOINT", "∮"),
        ("ODINT", "∯"), ("OTINT", "∰"),
        // 합/곱
        ("SUM", "∑"), ("SMALLSUM", "Σ"),
        ("PROD", "∏"), ("SMALLPROD", "∏"),
        ("COPROD", "∐"), ("SMCOPROD", "∐"), ("AMALG", "∐"),
        // 집합
        ("UNION", "∪"), ("BIGCUP", "∪"), ("SMALLUNION", "∪"), ("CUP", "∪"),
        ("INTER", "∩"), ("BIGCAP", "∩"), ("SMALLINTER", "∩"), ("CAP", "∩"),
        ("SQCUP", "⊔"), ("BIGSQCUP", "⊔"),
        ("SQCAP", "⊓"), ("BIGSQCAP", "⊓"),
        ("UPLUS", "⊎"), ("BIGUPLUS", "⊎"),
        ("BIGWEDGE", "⋀"), ("BIGVEE", "⋁"),
        // 원 연산자
        ("OPLUS", "⊕"), ("BIGOPLUS", "⊕"),
        ("OTIMES", "⊗"), ("BIGOTIMES", "⊗"),
        ("ODOT", "⊙"), ("BIGODOT", "⊙"),
        ("OMINUS", "⊖"), ("BIGOMINUS", "⊖"),
        ("ODIV", "⊘"), ("BIGODIV", "⊘"), ("OSLASH", "⊘"),
        // LaTeX lowercase aliases for big operators
        ("sum", "∑"), ("prod", "∏"), ("coprod", "∐"),
        ("bigcup", "∪"), ("bigcap", "∩"),
        ("bigwedge", "⋀"), ("bigvee", "⋁"),
        ("bigoplus", "⊕"), ("bigotimes", "⊗"),
        ("int", "∫"), ("iint", "∬"), ("iiint", "∭"),
        ("oint", "∮"),
    ])
});

/// 화살표 (대소문자 구분)
static ARROWS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // 일반 화살표
        ("larrow", "←"), ("rarrow", "→"),
        ("uparrow", "↑"), ("downarrow", "↓"),
        ("lrarrow", "↔"), ("udarrow", "↕"),
        // 이중선 화살표
        ("LARROW", "⇐"), ("RARROW", "⇒"),
        ("UPARROW", "⇑"), ("DOWNARROW", "⇓"),
        ("LRARROW", "⇔"), ("UDARROW", "⇕"),
        // 대각선
        ("nwarrow", "↖"), ("nearrow", "↗"),
        ("swarrow", "↙"), ("searrow", "↘"),
        // 특수
        ("mapsto", "↦"), ("hookleft", "↩"), ("hookright", "↪"),
        // 특수 (continued)
        ("longrightarrow", "⟶"), ("longleftarrow", "⟵"),
        ("Longrightarrow", "⟹"), ("Longleftarrow", "⟸"),
        ("longmapsto", "⟼"),
        // LaTeX aliases
        ("leftarrow", "←"), ("rightarrow", "→"), ("to", "→"), ("gets", "←"),
        ("Leftarrow", "⇐"), ("Rightarrow", "⇒"), ("implies", "⇒"), ("iff", "⇔"),
        ("leftrightarrow", "↔"), ("Leftrightarrow", "⇔"),
        // 막대
        ("vert", "|"), ("VERT", "‖"),
    ])
});

/// 괄호 명령어
static BRACKETS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("LBRACE", "{"), ("RBRACE", "}"),
        ("LCEIL", "⌈"), ("RCEIL", "⌉"),
        ("LFLOOR", "⌊"), ("RFLOOR", "⌋"),
        // LaTeX angle brackets
        ("langle", "⟨"), ("rangle", "⟩"),
        ("LANGLE", "⟨"), ("RANGLE", "⟩"),
        // LaTeX aliases
        ("lbrace", "{"), ("rbrace", "}"),
        ("lceil", "⌈"), ("rceil", "⌉"),
        ("lfloor", "⌊"), ("rfloor", "⌋"),
        ("lvert", "|"), ("rvert", "|"),
        ("lVert", "‖"), ("rVert", "‖"),
    ])
});

/// 함수 (삼각함수, 로그 등) — 로만체로 렌더링
static FUNCTIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("sin", "sin"), ("cos", "cos"), ("tan", "tan"),
        ("cot", "cot"), ("sec", "sec"), ("csc", "csc"),
        ("arcsin", "arcsin"), ("arccos", "arccos"), ("arctan", "arctan"),
        ("sinh", "sinh"), ("cosh", "cosh"), ("tanh", "tanh"), ("coth", "coth"),
        ("log", "log"), ("ln", "ln"), ("lg", "lg"), ("exp", "exp"),
        ("det", "det"), ("dim", "dim"), ("ker", "ker"), ("hom", "hom"),
        ("arg", "arg"), ("deg", "deg"), ("gcd", "gcd"), ("lcm", "lcm"),
        ("max", "max"), ("min", "min"),
        ("mod", "mod"),
        // LaTeX additional functions
        ("sup", "sup"), ("inf", "inf"), ("lim", "lim"),
        ("limsup", "lim sup"), ("liminf", "lim inf"),
        ("Pr", "Pr"),
    ])
});

/// 글자 장식 명령어
pub static DECORATIONS: LazyLock<HashMap<&'static str, DecoKind>> = LazyLock::new(|| {
    HashMap::from([
        ("hat", DecoKind::Hat), ("check", DecoKind::Check),
        ("tilde", DecoKind::Tilde), ("acute", DecoKind::Acute),
        ("grave", DecoKind::Grave), ("dot", DecoKind::Dot),
        ("ddot", DecoKind::DDot), ("bar", DecoKind::Bar),
        ("vec", DecoKind::Vec), ("dyad", DecoKind::Dyad),
        ("under", DecoKind::Under), ("arch", DecoKind::Arch),
        ("UNDERLINE", DecoKind::Underline), ("OVERLINE", DecoKind::Overline),
        ("NOT", DecoKind::StrikeThrough),
        // LaTeX 소문자 별칭
        ("underline", DecoKind::Underline), ("overline", DecoKind::Overline),
        ("not", DecoKind::StrikeThrough),
        ("widehat", DecoKind::Hat), ("widetilde", DecoKind::Tilde),
        ("overrightarrow", DecoKind::Vec),
        ("overleftarrow", DecoKind::Vec),
        ("overbrace", DecoKind::Arch), ("underbrace", DecoKind::Under),
    ])
});

/// 글꼴 스타일 명령어
pub static FONT_STYLES: LazyLock<HashMap<&'static str, FontStyleKind>> = LazyLock::new(|| {
    HashMap::from([
        ("rm", FontStyleKind::Roman),
        ("it", FontStyleKind::Italic),
        ("bold", FontStyleKind::Bold),
        // LaTeX \math* 계열
        ("mathrm", FontStyleKind::Roman),
        ("mathit", FontStyleKind::Italic),
        ("mathbf", FontStyleKind::Bold),
        ("mathbb", FontStyleKind::Blackboard),
        ("mathcal", FontStyleKind::Calligraphy),
        ("mathfrak", FontStyleKind::Fraktur),
        ("mathsf", FontStyleKind::SansSerif),
        ("mathtt", FontStyleKind::Monospace),
        ("textbf", FontStyleKind::Bold),
        ("textrm", FontStyleKind::Roman),
        ("textit", FontStyleKind::Italic),
    ])
});

/// 구조 명령어 (파서에서 특별 처리)
pub fn is_structure_command(cmd: &str) -> bool {
    matches!(cmd,
        "OVER" | "ATOP" | "SQRT" | "ROOT" | "FRAC" | "DFRAC" | "TFRAC" | "TEXT" | "BEGIN" | "END" |
        "LEFT" | "RIGHT" | "BIGG" |
        "OPERATORNAME" | "PHANTOM" | "VPHANTOM" | "HPHANTOM" |
        "OVERSET" | "UNDERSET" | "STACKREL" |
        "QUAD" | "QQUAD" | "THINSPACE" | "MEDSPACE" | "THICKSPACE" | "NEGSPACE" | "ENSPACE" |
        "MATRIX" | "PMATRIX" | "BMATRIX" | "DMATRIX" | "VMATRIX" | "SMALLMATRIX" |
        "CASES" | "PILE" | "LPILE" | "RPILE" |
        "CHOOSE" | "BINOM" |
        "lim" | "Lim" |
        "REL" | "BUILDREL" |
        "LADDER" | "SLADDER" | "LONGDIV" |
        "COLOR" |
        "SUP" | "SUB" | "LSUB" | "LSUP"
    )
}

/// 큰 연산자인지 확인
pub fn is_big_operator(cmd: &str) -> bool {
    BIG_OPERATORS.contains_key(cmd)
}

/// 함수인지 확인
pub fn is_function(cmd: &str) -> bool {
    FUNCTIONS.contains_key(cmd)
}

/// 명령어에 대한 Unicode 기호 조회
///
/// 한컴 수식은 대소문자를 구분하지 않으므로 (예: `times` = `TIMES`),
/// 원래 대소문자로 먼저 찾고 실패하면 대문자 변환 후 재시도한다.
/// 그리스 문자와 화살표는 대소문자가 의미를 가지므로 (alpha ≠ Alpha) 원래 값만 사용.
pub fn lookup_symbol(cmd: &str) -> Option<&'static str> {
    // 1차: 원래 대소문자로 조회
    if let Some(s) = GREEK_LOWER.get(cmd)
        .or_else(|| GREEK_UPPER.get(cmd))
        .or_else(|| SPECIAL_SYMBOLS.get(cmd))
        .or_else(|| OPERATORS.get(cmd))
        .or_else(|| BIG_OPERATORS.get(cmd))
        .or_else(|| ARROWS.get(cmd))
        .or_else(|| BRACKETS.get(cmd))
    {
        return Some(s);
    }

    // 2차: 대문자 변환 후 재시도 (SPECIAL_SYMBOLS, OPERATORS, BIG_OPERATORS, BRACKETS)
    let upper = cmd.to_ascii_uppercase();
    if upper != cmd {
        if let Some(s) = SPECIAL_SYMBOLS.get(upper.as_str())
            .or_else(|| OPERATORS.get(upper.as_str()))
            .or_else(|| BIG_OPERATORS.get(upper.as_str()))
            .or_else(|| BRACKETS.get(upper.as_str()))
        {
            return Some(s);
        }
    }

    None
}

/// 함수 이름 조회
pub fn lookup_function(cmd: &str) -> Option<&'static str> {
    FUNCTIONS.get(cmd).copied()
}

/// 글자 장식 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum DecoKind {
    Hat,           // ^
    Check,         // ˇ
    Tilde,         // ~
    Acute,         // ´
    Grave,         // `
    Dot,           // ˙
    DDot,          // ¨
    Bar,           // ¯
    Vec,           // →
    Dyad,          // ↔
    Under,         // _
    Arch,          // ⌢
    Underline,     // ___
    Overline,      // ‾‾‾
    StrikeThrough, // /
}

/// 글꼴 스타일 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum FontStyleKind {
    Roman,      // 로만체 (upright) — rm, \mathrm
    Italic,     // 이탤릭체 — it, \mathit
    Bold,       // 볼드체 — bold, \mathbf
    Blackboard, // 흑판 볼드 — \mathbb (ℝ, ℤ, ℕ 등)
    Calligraphy,// 필기체 — \mathcal (ℒ, ℋ 등)
    Fraktur,    // 프락투르 — \mathfrak
    SansSerif,  // 산세리프 — \mathsf
    Monospace,  // 고정폭 — \mathtt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greek_lower() {
        assert_eq!(lookup_symbol("alpha"), Some("α"));
        assert_eq!(lookup_symbol("omega"), Some("ω"));
    }

    #[test]
    fn test_greek_upper() {
        assert_eq!(lookup_symbol("Gamma"), Some("Γ"));
        assert_eq!(lookup_symbol("Omega"), Some("Ω"));
    }

    #[test]
    fn test_operators() {
        assert_eq!(lookup_symbol("TIMES"), Some("×"));
        assert_eq!(lookup_symbol("PLUSMINUS"), Some("±"));
        assert_eq!(lookup_symbol("INF"), Some("∞"));
    }

    #[test]
    fn test_case_insensitive_operators() {
        // 소문자로 입력해도 대문자 연산자/기호 매핑
        assert_eq!(lookup_symbol("times"), Some("×"));
        assert_eq!(lookup_symbol("div"), Some("÷"));
        assert_eq!(lookup_symbol("neq"), Some("≠"));
        assert_eq!(lookup_symbol("leq"), Some("≤"));
        assert_eq!(lookup_symbol("geq"), Some("≥"));
        assert_eq!(lookup_symbol("inf"), Some("∞"));
        assert_eq!(lookup_symbol("pm"), Some("±"));
        // 그리스 문자는 대소문자 구분 유지
        assert_eq!(lookup_symbol("alpha"), Some("α"));
        assert_eq!(lookup_symbol("Alpha"), Some("Α"));
        assert_ne!(lookup_symbol("alpha"), lookup_symbol("Alpha"));
    }

    #[test]
    fn test_big_operators() {
        assert!(is_big_operator("INT"));
        assert!(is_big_operator("SUM"));
        assert!(is_big_operator("PROD"));
        assert!(!is_big_operator("alpha"));
    }

    #[test]
    fn test_arrows() {
        assert_eq!(lookup_symbol("rarrow"), Some("→"));
        assert_eq!(lookup_symbol("RARROW"), Some("⇒"));
    }

    #[test]
    fn test_functions() {
        assert!(is_function("sin"));
        assert!(is_function("log"));
        assert!(!is_function("OVER"));
    }

    #[test]
    fn test_structure_commands() {
        assert!(is_structure_command("OVER"));
        assert!(is_structure_command("SQRT"));
        assert!(is_structure_command("lim"));
        assert!(!is_structure_command("alpha"));
    }
}
