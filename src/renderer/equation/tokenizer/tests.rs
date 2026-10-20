use super::*;

fn values(tokens: &[Token]) -> Vec<&str> {
    tokens
        .iter()
        .filter(|t| t.ty != TokenType::Eof)
        .map(|t| t.value.as_str())
        .collect()
}

fn types(tokens: &[Token]) -> Vec<TokenType> {
    tokens
        .iter()
        .filter(|t| t.ty != TokenType::Eof)
        .map(|t| t.ty)
        .collect()
}

#[test]
fn test_simple_fraction() {
    let tokens = tokenize("1 over 2");
    assert_eq!(values(&tokens), vec!["1", "over", "2"]);
    assert_eq!(
        types(&tokens),
        vec![TokenType::Number, TokenType::Command, TokenType::Number]
    );
}

#[test]
fn test_task1122_over_atop_followed_by_number_splits() {
    let tokens = tokenize("11 over20");
    assert_eq!(values(&tokens), vec!["11", "over", "20"]);
    assert_eq!(
        types(&tokens),
        vec![TokenType::Number, TokenType::Command, TokenType::Number]
    );

    let tokens = tokenize("7 OVER10");
    assert_eq!(values(&tokens), vec!["7", "OVER", "10"]);
    assert_eq!(
        types(&tokens),
        vec![TokenType::Number, TokenType::Command, TokenType::Number]
    );

    let tokens = tokenize("a atop2");
    assert_eq!(values(&tokens), vec!["a", "atop", "2"]);
}

#[test]
fn test_task1122_over_prefix_non_numeric_identifiers_stay_intact() {
    let tokens = tokenize("overlap overline overset");
    assert_eq!(values(&tokens), vec!["overlap", "overline", "overset"]);

    let tokens = tokenize(r"\overline{AB}");
    assert_eq!(values(&tokens), vec!["overline", "{", "AB", "}"]);
}

#[test]
fn test_superscript() {
    let tokens = tokenize("E=mc^2");
    assert_eq!(values(&tokens), vec!["E", "=", "mc", "^", "2"]);
}

#[test]
fn test_subscript_superscript() {
    let tokens = tokenize("sum_{i=0}^n");
    assert_eq!(
        values(&tokens),
        vec!["sum", "_", "{", "i", "=", "0", "}", "^", "n"]
    );
}

#[test]
fn test_whitespace_chars() {
    let tokens = tokenize("a~b`c#d&e");
    let ws_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.ty == TokenType::Whitespace)
        .map(|t| t.value.as_str())
        .collect();
    assert_eq!(ws_tokens, vec!["~", "`", "#", "&"]);
}

#[test]
fn test_korean_text() {
    let tokens = tokenize("평점=입찰가격");
    assert_eq!(
        types(&tokens),
        vec![TokenType::Text, TokenType::Symbol, TokenType::Text]
    );
    assert_eq!(values(&tokens), vec!["평점", "=", "입찰가격"]);
}

#[test]
fn test_quoted_string() {
    let tokens = tokenize("\"1234567890\" over 5");
    assert_eq!(
        types(&tokens),
        vec![TokenType::Quoted, TokenType::Command, TokenType::Number]
    );
    assert_eq!(values(&tokens), vec!["1234567890", "over", "5"]);
}

#[test]
fn test_multi_char_symbols() {
    let tokens = tokenize("a <= b >= c != d == e");
    let syms: Vec<_> = tokens
        .iter()
        .filter(|t| t.ty == TokenType::Symbol)
        .map(|t| t.value.as_str())
        .collect();
    assert_eq!(syms, vec!["<=", ">=", "!=", "=="]);
}

#[test]
fn test_pua_conditional_bar() {
    // #1343: 한컴 수식 PUA 조건부 막대 U+E04D → 단일 `|` 기호와 동일 토큰
    let tokens = tokenize("rm P LEFT ( it A \u{E04D} B RIGHT )");
    let bar: Vec<_> = tokens
        .iter()
        .filter(|t| t.ty == TokenType::Symbol)
        .map(|t| t.value.as_str())
        .collect();
    assert_eq!(bar, vec!["|"]);
    // PUA 원형 코드포인트가 토큰에 남지 않아야 한다(두부 방지)
    assert!(tokens.iter().all(|t| !t.value.contains('\u{E04D}')));
}

#[test]
fn test_arrow() {
    let tokens = tokenize("x->0");
    assert_eq!(values(&tokens), vec!["x", "->", "0"]);
}

#[test]
fn test_left_right() {
    let tokens = tokenize("LEFT ( a over b RIGHT )");
    assert_eq!(
        values(&tokens),
        vec!["LEFT", "(", "a", "over", "b", "RIGHT", ")"]
    );
}

#[test]
fn test_matrix() {
    let tokens = tokenize("matrix{a & b # c & d}");
    assert_eq!(
        values(&tokens),
        vec!["matrix", "{", "a", "&", "b", "#", "c", "&", "d", "}"]
    );
}

#[test]
fn test_decimal_number() {
    let tokens = tokenize("3.14");
    assert_eq!(types(&tokens), vec![TokenType::Number]);
    assert_eq!(values(&tokens), vec!["3.14"]);
}

#[test]
fn test_sample_eq01() {
    // 실제 eq-01.hwp 수식 스크립트의 일부
    let tokens = tokenize("TIMES  LEFT ( {최저입찰가격} over {해당입찰가격} RIGHT )");
    let cmds: Vec<_> = tokens
        .iter()
        .filter(|t| t.ty == TokenType::Command)
        .map(|t| t.value.as_str())
        .collect();
    assert!(cmds.contains(&"TIMES"));
    assert!(cmds.contains(&"LEFT"));
    assert!(cmds.contains(&"over"));
    assert!(cmds.contains(&"RIGHT"));
}

// Task #488: 폰트 스타일 키워드(rm/it/bold) prefix 분리

#[test]
fn test_font_style_prefix_rm_uppercase() {
    let tokens = tokenize("rmK ^{+}");
    assert_eq!(values(&tokens), vec!["rm", "K", "^", "{", "+", "}"]);
}

#[test]
fn test_font_style_prefix_rm_compound() {
    let tokens = tokenize("rmCa ^{2+}");
    assert_eq!(values(&tokens), vec!["rm", "Ca", "^", "{", "2", "+", "}"]);
}

#[test]
fn test_font_style_prefix_rm_lowercase() {
    let tokens = tokenize("1`rmmol");
    assert_eq!(values(&tokens), vec!["1", "`", "rm", "mol"]);
}

#[test]
fn test_font_style_prefix_it_compound() {
    let tokens = tokenize("LEFT ( itaq RIGHT )");
    assert_eq!(values(&tokens), vec!["LEFT", "(", "it", "aq", "RIGHT", ")"]);
}

#[test]
fn test_font_style_prefix_it_single_letter() {
    let tokens = tokenize("LEFT ( itl RIGHT )");
    assert_eq!(values(&tokens), vec!["LEFT", "(", "it", "l", "RIGHT", ")"]);
}

#[test]
fn test_font_style_prefix_bold() {
    let tokens = tokenize("boldX");
    assert_eq!(values(&tokens), vec!["bold", "X"]);
}

#[test]
fn test_font_style_keyword_alone_unchanged() {
    // 키워드 직후가 공백/기호/EOF: 분리하지 않고 그대로 키워드
    let tokens = tokenize("rm K");
    assert_eq!(values(&tokens), vec!["rm", "K"]);
    let tokens = tokenize("it{x}");
    assert_eq!(values(&tokens), vec!["it", "{", "x", "}"]);
    let tokens = tokenize("rm");
    assert_eq!(values(&tokens), vec!["rm"]);
}

// [#1204-A] root/sqrt 가 숫자에 붙은 경우 분리 (`root3`→√3)
#[test]
fn test_root_sqrt_prefix_split_on_digit() {
    assert_eq!(values(&tokenize("root3 y")), vec!["root", "3", "y"]);
    assert_eq!(values(&tokenize("sqrt5")), vec!["sqrt", "5"]);
    assert_eq!(
        values(&tokenize("2 over {root3 a}")),
        vec!["2", "over", "{", "root", "3", "a", "}"]
    );
    // 관계연산자도 숫자에 붙으면 분리 (`GEQ5`→≥5, `GE0`→≥0)
    assert_eq!(values(&tokenize("GEQ5")), vec!["GEQ", "5"]);
    assert_eq!(values(&tokenize("GE0")), vec!["GE", "0"]);
    assert_eq!(values(&tokenize("LEQ3")), vec!["LEQ", "3"]);
    // 숫자가 아니면 분리하지 않음 (letter 변수 충돌 회피)
    assert_eq!(values(&tokenize("rootn")), vec!["rootn"]);
    // 중괄호/공백 형태는 영향 없음
    assert_eq!(
        values(&tokenize("root {4} of {x}")),
        vec!["root", "{", "4", "}", "of", "{", "x", "}"]
    );
}

// [#1204-C] prime 이 alnum 에 붙은 경우 분리 (`primeF`→′ F)
#[test]
fn test_prime_prefix_split() {
    assert_eq!(values(&tokenize("F primeF")), vec!["F", "prime", "F"]);
    // 공백 형태는 기존대로
    assert_eq!(values(&tokenize("f prime")), vec!["f", "prime"]);
}

// [#1204-E] 함수/장식/관계연산자/도형 키워드가 글자에 붙은 경우 allowlist 분리.
#[test]
fn test_glued_keyword_letter_split() {
    assert_eq!(values(&tokenize("tanx")), vec!["tan", "x"]);
    assert_eq!(values(&tokenize("sinx")), vec!["sin", "x"]);
    assert_eq!(values(&tokenize("barMH")), vec!["bar", "MH"]);
    assert_eq!(values(&tokenize("LEQb")), vec!["LEQ", "b"]);
    assert_eq!(values(&tokenize("trianglePQR")), vec!["triangle", "PQR"]);
    // chain: rm + bar + FF
    assert_eq!(values(&tokenize("rmbarFF")), vec!["rm", "bar", "FF"]);
    // longest-match: cosh 는 cos+h 로 쪼개지지 않음
    assert_eq!(values(&tokenize("coshx")), vec!["cosh", "x"]);
}

// 알려진 전체 명령과 따옴표 문자열은 접두사 분리로 훼손하지 않는다.
#[test]
fn test_glued_keyword_no_oversplit() {
    assert_eq!(values(&tokenize("alphabet")), vec!["alpha", "bet"]); // 한컴 Print: αbet
    assert_eq!(values(&tokenize("argmax")), vec!["argmax"]); // arg/max 제외
    assert_eq!(values(&tokenize("rootn")), vec!["rootn"]); // root letter 제외
}

// [#1204-E] cap/cup (집합연산) 가 글자에 붙은 경우 분리 (`capB`→∩ B)
#[test]
fn test_cap_cup_glued_split() {
    assert_eq!(values(&tokenize("capB")), vec!["cap", "B"]);
    assert_eq!(values(&tokenize("cupB")), vec!["cup", "B"]);
}

// [#1204] 생략기호 연접 분리 (`cdotscdots`→⋯⋯). 단일/공백형은 기존대로.
#[test]
fn test_dots_glued_split() {
    assert_eq!(values(&tokenize("cdotscdots")), vec!["cdots", "cdots"]);
    assert_eq!(values(&tokenize("cdots")), vec!["cdots"]);
    assert_eq!(values(&tokenize("a cdots b")), vec!["a", "cdots", "b"]);
    assert_eq!(values(&tokenize("ldotsldots")), vec!["ldots", "ldots"]);
}

// [#1204-E] over/atop 가 짧은(≤2) 글자 피연산자에 붙으면 분리(분수), 긴 word 는 유지.
#[test]
fn test_over_glued_short_letter_operand() {
    assert_eq!(values(&tokenize("overa")), vec!["over", "a"]);
    assert_eq!(values(&tokenize("overdx")), vec!["over", "dx"]);
    assert_eq!(values(&tokenize("x overy")), vec!["x", "over", "y"]);
    // 가드: 긴 word(≥3) 및 keyword 는 유지 (#1122)
    assert_eq!(values(&tokenize("overlap")), vec!["overlap"]);
    assert_eq!(values(&tokenize("overline")), vec!["overline"]);
    assert_eq!(values(&tokenize("overset")), vec!["overset"]);
}

#[test]
fn test_existing_commands_unchanged() {
    // 기존 명령은 회귀 없음
    let tokens = tokenize("OVER MATRIX SQRT alpha beta");
    assert_eq!(
        values(&tokens),
        vec!["OVER", "MATRIX", "SQRT", "alpha", "beta"]
    );
}

#[test]
fn test_latex_command_prefix() {
    let tokens = tokenize(r"\frac{1}{2}");
    assert_eq!(
        types(&tokens),
        vec![
            TokenType::Command,
            TokenType::LBrace,
            TokenType::Number,
            TokenType::RBrace,
            TokenType::LBrace,
            TokenType::Number,
            TokenType::RBrace,
        ]
    );
    assert_eq!(values(&tokens), vec!["frac", "{", "1", "}", "{", "2", "}"]);
}

// Task #576: times/sim 연산자 키워드 prefix 분리

#[test]
fn test_task576_times_lowercase_prefix_split() {
    let tokens = tokenize("a timesm");
    assert_eq!(values(&tokens), vec!["a", "times", "m"]);
}

#[test]
fn test_task576_sim_lowercase_prefix_split() {
    let tokens = tokenize("rm X simZ");
    assert_eq!(values(&tokens), vec!["rm", "X", "sim", "Z"]);
}

#[test]
fn test_task576_times_uppercase_prefix_split() {
    let tokens = tokenize("1TIMES10");
    assert_eq!(values(&tokens), vec!["1", "TIMES", "10"]);
}

#[test]
fn test_task576_sim_uppercase_prefix_split() {
    let tokens = tokenize("rmA SIMC");
    assert_eq!(values(&tokens), vec!["rm", "A", "SIM", "C"]);
}

#[test]
fn test_task576_alpha_no_split() {
    // 회귀 차단: alpha 는 그 자체 keyword 이므로 분리되면 안 됨
    let tokens = tokenize("alpha");
    assert_eq!(values(&tokens), vec!["alpha"]);
    // 한컴의 그리스 명령은 붙은 피연산자와 분리되며, 인용 문자열은 보존한다.
    let tokens = tokenize("alphabet");
    assert_eq!(values(&tokens), vec!["alpha", "bet"]);
    let literal = tokenize("\"alphabet\"");
    assert_eq!(values(&literal), vec!["alphabet"]);
    assert_eq!(types(&literal), vec![TokenType::Quoted]);
}

#[test]
fn test_task576_times_followed_by_space() {
    // 키워드 다음 공백/기호 오면 분리 불필요 (기존 동작 보존)
    let tokens = tokenize("a times b");
    assert_eq!(values(&tokens), vec!["a", "times", "b"]);
    let tokens = tokenize("rmA SIM C");
    assert_eq!(values(&tokens), vec!["rm", "A", "SIM", "C"]);
}
