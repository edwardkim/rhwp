//! 수식 명령 디스패치 분류.
//!
//! `EqParser::parse_command_inner` 의 명령 분기를 한 표로 고정한다.
//! 분류 순서와 가족은 구 if 연쇄와 같고, 파서 동작은 바꾸지 않는다.
//!
//! 모듈 지도는 `src/renderer/equation/README.md`, 매뉴얼은
//! `mydocs/manual/equation_module.md`.

use super::ast::{MatrixStyle, PileAlign};
use super::symbols::is_big_operator;

/// 수식 명령 가족. `parse_command_inner` 가 이 분류만 보고 핸들러를 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EqCommandClass {
    /// 중위 `OVER`/`ATOP`. 단독 출현은 버리고, 결합은 `parse_expression` 이 맡는다.
    InfixDiscard,
    /// LaTeX `\frac`/`\dfrac`/`\tfrac`.
    LatexFraction,
    /// `\text`/`\operatorname` — 로만체 본문.
    RomanText,
    /// `\phantom` 계열. 인자를 소비하고 공백 한 칸만 남긴다.
    Phantom,
    /// LaTeX 간격 명령.
    LatexSpacing,
    /// `\overset`/`\stackrel`.
    Overset,
    /// `\underset`.
    Underset,
    /// `\begin{env}`.
    BeginEnv,
    /// `\end{env}` — 인자만 건너뛴다.
    EndEnv,
    /// `SQRT`/`ROOT`. `ROOT` 도 현행과 같이 `Sqrt` AST 로 간다.
    Sqrt,
    /// 적분 기호. `BigOp` 보다 먼저 매칭해 nolimits(일반 첨자)로 둔다.
    IntegralNolimits,
    /// `SUM`/`PROD` 등 limits 큰 연산자.
    BigOperator,
    /// `lim`/`Lim`. 원문 대소문자를 구분한다.
    Limit,
    /// `MATRIX`/`PMATRIX`/`BMATRIX`/`DMATRIX`. `VMATRIX` 는 여기 넣지 않는다.
    Matrix,
    /// 조건식.
    Cases,
    /// 칸 맞춤 정렬.
    EqAlign,
    /// `PILE`/`LPILE`/`RPILE`.
    Pile,
    /// `LEFT` … `RIGHT` 그룹. 뒤 첨자는 그룹 전체에 붙인다.
    LeftDelim,
    /// 짝 없는 `RIGHT`.
    RightDiscard,
    /// `REL`/`BUILDREL`.
    Rel,
    /// 긴 나눗셈 자리표시.
    LongDiv,
    /// `LADDER`/`SLADDER` → 평문 행렬 fallback.
    Ladder,
    /// 벤젠 자리표시. `try_parse_scripts` 를 타지 않는다.
    Benzene,
    /// `BIGG` — 크기 변경은 무시하고 다음 요소만 돌린다.
    Bigg,
    /// `CHOOSE`.
    Choose,
    /// `BINOM`.
    Binom,
    /// `COLOR`.
    Color,
    /// `LSUB`/`LSUP`.
    LeftScript,
    /// `SUP` 동의어.
    Sup,
    /// `SUB` 동의어.
    Sub,
    /// 장식·글꼴·기호·함수·미지 명령.
    Fallback,
}

/// 명령 토큰을 가족으로 분류한다.
///
/// 적분은 [`is_big_operator`] 보다 앞, `lim`/`Lim` 은 그보다 뒤다.
/// 이 순서를 바꾸면 골든(M09-1, PR #5412)이 깨진다.
pub(crate) fn classify_command(cmd: &str) -> EqCommandClass {
    let upper = cmd.to_ascii_uppercase();
    let cu = upper.as_str();

    match cu {
        "OVER" | "ATOP" => return EqCommandClass::InfixDiscard,
        "FRAC" | "DFRAC" | "TFRAC" => return EqCommandClass::LatexFraction,
        "TEXT" | "OPERATORNAME" => return EqCommandClass::RomanText,
        "PHANTOM" | "VPHANTOM" | "HPHANTOM" => return EqCommandClass::Phantom,
        "QUAD" | "QQUAD" | "THINSPACE" | "MEDSPACE" | "THICKSPACE" | "NEGSPACE" | "ENSPACE" => {
            return EqCommandClass::LatexSpacing;
        }
        "OVERSET" | "STACKREL" => return EqCommandClass::Overset,
        "UNDERSET" => return EqCommandClass::Underset,
        "BEGIN" => return EqCommandClass::BeginEnv,
        "END" => return EqCommandClass::EndEnv,
        "SQRT" | "ROOT" => return EqCommandClass::Sqrt,
        "INT" | "INTEGRAL" | "SMALLINT" | "DINT" | "TINT" | "OINT" | "SMALLOINT" | "ODINT"
        | "OTINT" => return EqCommandClass::IntegralNolimits,
        _ => {}
    }

    if is_big_operator(cu) || is_big_operator(cmd) {
        return EqCommandClass::BigOperator;
    }
    if cmd == "lim" || cmd == "Lim" {
        return EqCommandClass::Limit;
    }

    match cu {
        "MATRIX" | "PMATRIX" | "BMATRIX" | "DMATRIX" => EqCommandClass::Matrix,
        "CASES" => EqCommandClass::Cases,
        "EQALIGN" => EqCommandClass::EqAlign,
        "PILE" | "LPILE" | "RPILE" => EqCommandClass::Pile,
        "LEFT" => EqCommandClass::LeftDelim,
        "RIGHT" => EqCommandClass::RightDiscard,
        "REL" | "BUILDREL" => EqCommandClass::Rel,
        "LONGDIV" => EqCommandClass::LongDiv,
        "LADDER" | "SLADDER" => EqCommandClass::Ladder,
        "BENZENE" => EqCommandClass::Benzene,
        "BIGG" => EqCommandClass::Bigg,
        "CHOOSE" => EqCommandClass::Choose,
        "BINOM" => EqCommandClass::Binom,
        "COLOR" => EqCommandClass::Color,
        "LSUB" | "LSUP" => EqCommandClass::LeftScript,
        "SUP" => EqCommandClass::Sup,
        "SUB" => EqCommandClass::Sub,
        _ => EqCommandClass::Fallback,
    }
}

/// `MATRIX` 계열 대문자 이름 → 괄호 스타일. 미지 이름은 Plain.
pub(crate) fn matrix_style(cmd_upper: &str) -> MatrixStyle {
    match cmd_upper {
        "PMATRIX" => MatrixStyle::Paren,
        "BMATRIX" => MatrixStyle::Bracket,
        "DMATRIX" => MatrixStyle::Vert,
        _ => MatrixStyle::Plain,
    }
}

/// `PILE` 계열 대문자 이름 → 가로 정렬. 미지 이름은 Center.
pub(crate) fn pile_align(cmd_upper: &str) -> PileAlign {
    match cmd_upper {
        "LPILE" => PileAlign::Left,
        "RPILE" => PileAlign::Right,
        _ => PileAlign::Center,
    }
}
