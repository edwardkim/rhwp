use rhwp::renderer::equation::tokenizer::{tokenize, Token, TokenType};

fn values(tokens: &[Token]) -> Vec<&str> {
    tokens
        .iter()
        .filter(|t| t.ty != TokenType::Eof)
        .map(|t| t.value.as_str())
        .collect()
}

#[test]
fn long_identifier_checks_only_keyword_length_prefixes() {
    // 긴 식별자는 키워드 접두사가 아니므로 한 토큰이다. 모든 접두사를 해시하면 길이의
    // 제곱만큼 느려져 이 길이에서는 수십 초가 걸린다.
    let script = "x".repeat(40_000);
    let started = std::time::Instant::now();
    let tokens = tokenize(&script);
    assert_eq!(values(&tokens), vec![script.as_str()]);
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    // 키워드 길이 안의 접두사 분리는 그대로다.
    assert_eq!(values(&tokenize("alphabet")), vec!["alpha", "bet"]);
}
