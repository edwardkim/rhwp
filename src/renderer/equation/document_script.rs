//! 저장 문서 수식은 한컴 명령과 LaTeX 호환 명령의 생성 문법을 구분한다.

use super::dispatch::{classify_command, EqCommandClass};
use super::tokenizer::{self, Token, TokenType};

/// 파일 속성의 표시 세대를 측정과 모든 실제 배치 경로에 함께 전달한다.
pub(crate) fn parse(equation: &crate::model::control::Equation) -> super::ast::EqNode {
    let tokens = tokenize_with_display(&equation.script, equation.uses_legacy_display());
    super::parser::EqParser::for_equation(tokens, equation).parse()
}

/// bare LaTeX 전용 이름은 문서에서 일반 문자다. 인자와 첨자는 원래 토큰을
/// 유지한다. 백슬래시의 문서 해석도 공개 LaTeX API와 구분한다.
pub(crate) fn tokenize(script: &str) -> Vec<Token> {
    tokenize_with_display(script, false)
}

fn tokenize_with_display(script: &str, legacy_display: bool) -> Vec<Token> {
    let normalized = super::legacy_hwpeq::normalize(script);
    let script = normalized.as_deref().unwrap_or(script);
    let lexer = if legacy_display {
        tokenizer::Tokenizer::for_legacy_document(script)
    } else {
        tokenizer::Tokenizer::for_document(script)
    };
    let mut tokens = lexer.tokenize();
    let chars: Vec<char> = script.chars().collect();
    let mut roman_literal = false;
    for token in &mut tokens {
        // 한컴의 백슬래시 문자 전환은 그룹 닫힘을 넘어 유지되며
        // 명시적 it에서 변수 글리프로 복귀한다. 일반 rm/it의 그룹
        // 스타일과 섞지 않고 각 문자 토큰에 표시 속성으로 전달한다.
        if token.ty == TokenType::Text && chars.get(token.pos) == Some(&'\\') {
            roman_literal = true;
        } else if token.ty == TokenType::Command && token.value.eq_ignore_ascii_case("IT") {
            roman_literal = false;
        }
        token.roman_literal = roman_literal;
        if token.ty != TokenType::Command {
            continue;
        }
        match token.value.as_str() {
            "LEFTARROW" => token.value = "Leftarrow".to_owned(),
            "RIGHTARROW" => token.value = "Rightarrow".to_owned(),
            _ if token.value.eq_ignore_ascii_case("LINE") => {
                token.ty = TokenType::Symbol;
                token.value = "|".to_owned();
                continue;
            }
            _ => {}
        }
        let class = classify_command(&token.value);
        // 독립 한컴 Print에서 혼합 대소문자의 SQRT/ROOT는 일반 문자다.
        // 모든 대문자·소문자 및 공개 LaTeX tokenizer의 계약은 유지한다.
        let mixed_root = class == EqCommandClass::Sqrt
            && token.value != token.value.to_ascii_uppercase()
            && token.value != token.value.to_ascii_lowercase();
        if mixed_root
            || matches!(
                class,
                EqCommandClass::LatexFraction
                    | EqCommandClass::RomanText
                    | EqCommandClass::LatexSpacing
                    | EqCommandClass::BeginEnv
                    | EqCommandClass::EndEnv
            )
            || token.value.eq_ignore_ascii_case("STACKREL")
        {
            token.ty = TokenType::Text;
        }
    }
    retain_escaped_underline_brace(tokens, &chars)
}

/// escaped UNDERLINE이 표시하는 첫 여는 괄호와 실제 인자 닫힘을 함께
/// 처리한다. 내부 그룹 및 바깥 그룹의 닫힘 소유는 그대로 보존한다.
fn retain_escaped_underline_brace(mut tokens: Vec<Token>, chars: &[char]) -> Vec<Token> {
    // 모든 짝을 한 번 계산한다. 중첩 입력마다 뒤 토큰을 다시 훑지 않는다.
    let mut closers = vec![None; tokens.len()];
    let mut opens = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token.ty {
            TokenType::LBrace => opens.push(index),
            TokenType::RBrace => {
                if let Some(open) = opens.pop() {
                    closers[open] = Some(index);
                }
            }
            _ => {}
        }
    }
    let mut removed_closers = vec![false; tokens.len()];
    for index in 0..tokens.len() {
        if tokens[index].ty != TokenType::Text
            || !tokens[index].value.eq_ignore_ascii_case("UNDERLINE")
            || chars.get(tokens[index].pos) != Some(&'\\')
            || tokens.get(index + 1).map(|token| token.ty) != Some(TokenType::LBrace)
        {
            continue;
        }
        if let Some(close) = closers[index + 1] {
            tokens[index + 1].ty = TokenType::Symbol;
            removed_closers[close] = true;
        }
    }
    tokens
        .into_iter()
        .enumerate()
        .filter_map(|(index, token)| (!removed_closers[index]).then_some(token))
        .collect()
}
