//! DSEL 파서 — 재귀 하강, 축 사전 대조, 상한 강제.
//!
//! ## 파싱 중에 의미까지 검사하는 이유
//!
//! 축 이름·속성 이름·연산자 적합성을 **파싱 단계에서** 확인한다. 문법만 보고
//! 통과시킨 뒤 평가에서 거절하면, 오류 위치가 "이 선택자 어딘가"로 뭉개진다.
//! 파싱 중에는 지금 어느 축의 어느 속성을 읽는 중인지 정확히 알고 있으므로
//! `축 para 에 없는 속성 rows — 기대: text|len|styleId…` 처럼 후보까지 낼 수 있다.
//! 진단 품질은 에이전트 왕복 횟수로 바로 환산된다.
//!
//! ## 상한을 파서가 강제하는 이유
//!
//! 선택자는 **모델이 만들고 문서에 맞대는 값**이다. 즉 길이도 중첩도 신뢰
//! 경계 밖에서 온다. 중첩 `:has(:has(:has(…)))` 는 재귀 하강 파서에서 그대로
//! 스택 깊이가 되고, 그건 손상 입력 스택 오버플로(#4830 과 같은 부류)다.
//! 상한은 평가기가 아니라 여기 있어야 한다 — 평가까지 가기 전에 막아야
//! 스택이 이미 깊어진 상태를 피한다.

use super::ast::{
    unknown_attr, unknown_axis, AttrPred, AttrType, Axis, AxisKind, CmpOp, Combinator, Literal,
    Path, Pred, Pseudo, PseudoArity, PseudoDef, Selector, Step, AXIS_NAMES, PSEUDO_DEFS,
};
use super::error::SelectorError;
use super::glob::Glob;
use super::lex::{lex, Lexed};
use super::token::{Tok, Token};

/// 선택자 원문 최대 길이(문자).
///
/// 4096 은 넉넉하다 — 실무에서 가장 긴 선택자도 200자를 넘지 않는다. 상한의
/// 목적은 표현력 제한이 아니라 "무한히 긴 입력이 파서에 들어오지 않는다"는
/// 보장이다.
pub const MAX_SOURCE_CHARS: usize = 4096;

/// `:has`/`:not` 중첩 최대 깊이.
pub const MAX_NESTING: usize = 8;

/// 합집합 가지 최대 개수.
pub const MAX_PATHS: usize = 64;

/// 한 경로의 최대 스텝 수.
pub const MAX_STEPS: usize = 32;

/// 한 스텝의 최대 술어 수.
pub const MAX_PREDS: usize = 16;

/// 선택자 문자열을 파싱한다.
pub fn parse(source: &str) -> Result<Selector, SelectorError> {
    let char_len = source.chars().count();
    if char_len > MAX_SOURCE_CHARS {
        return Err(SelectorError::limit(
            MAX_SOURCE_CHARS,
            format!("선택자가 너무 길다 ({char_len}자, 상한 {MAX_SOURCE_CHARS}자)"),
        ));
    }

    let lexed = lex(source)?;
    let mut parser = Parser {
        toks: &lexed.tokens,
        char_len: lexed.char_len,
        pos: 0,
        depth: 0,
    };

    let paths = parser.parse_paths(source)?;
    parser.skip_ws();
    if let Some(tok) = parser.peek() {
        let offset = parser.offset();
        return Err(
            SelectorError::parse(offset, format!("선택자 끝에 남은 {}", tok.describe()))
                .expecting([","]),
        );
    }

    Ok(Selector {
        paths,
        source: source.to_string(),
    })
}

struct Parser<'a> {
    toks: &'a [Token],
    char_len: usize,
    pos: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }

    /// 현재 위치의 문자 오프셋. 입력 끝이면 전체 길이.
    fn offset(&self) -> usize {
        self.toks
            .get(self.pos)
            .map(|t| t.offset)
            .unwrap_or(self.char_len)
    }

    fn bump(&mut self) -> Option<&'a Tok> {
        let t = self.toks.get(self.pos).map(|t| &t.tok);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    /// 공백 토큰을 흘린다. 실제로 흘렸는지 돌려준다 — 자손 결합자 판정에 쓴다.
    fn skip_ws(&mut self) -> bool {
        let mut seen = false;
        while matches!(self.peek(), Some(Tok::Ws)) {
            self.pos += 1;
            seen = true;
        }
        seen
    }

    fn expect(&mut self, want: &Tok, what: &str) -> Result<(), SelectorError> {
        let offset = self.offset();
        match self.peek() {
            Some(t) if t == want => {
                self.pos += 1;
                Ok(())
            }
            Some(t) => Err(
                SelectorError::parse(offset, format!("{what} 자리에 {}", t.describe()))
                    .expecting([want.symbol()]),
            ),
            None => Err(
                SelectorError::parse(offset, format!("{what} 없이 선택자가 끝났다"))
                    .expecting([want.symbol()]),
            ),
        }
    }

    /// 쉼표로 이어진 경로들.
    fn parse_paths(&mut self, source: &str) -> Result<Vec<Path>, SelectorError> {
        let mut paths = Vec::new();
        loop {
            self.skip_ws();
            paths.push(self.parse_path(source)?);
            if paths.len() > MAX_PATHS {
                return Err(SelectorError::limit(
                    self.offset(),
                    format!("합집합 가지가 너무 많다 (상한 {MAX_PATHS})"),
                ));
            }
            self.skip_ws();
            if matches!(self.peek(), Some(Tok::Comma)) {
                self.pos += 1;
                continue;
            }
            break;
        }
        Ok(paths)
    }

    /// 결합자로 이어진 스텝들.
    fn parse_path(&mut self, source: &str) -> Result<Path, SelectorError> {
        let mut steps = Vec::new();
        let mut combinator = Combinator::Root;

        loop {
            steps.push(self.parse_step(combinator, source)?);
            if steps.len() > MAX_STEPS {
                return Err(SelectorError::limit(
                    self.offset(),
                    format!("경로 스텝이 너무 많다 (상한 {MAX_STEPS})"),
                ));
            }

            // 다음 결합자를 정한다. 공백을 흘리기 **전** 위치를 기억해 두는 이유:
            // 공백 뒤가 `,` 나 `)` 면 그 공백은 결합자가 아니라 여백이므로
            // 되감아야 상위 규칙이 같은 공백을 다시 처리할 수 있다.
            let save = self.pos;
            let had_ws = self.skip_ws();
            combinator = match self.peek() {
                None => break,
                Some(Tok::Comma) | Some(Tok::RParen) => {
                    self.pos = save;
                    break;
                }
                Some(Tok::Gt) => {
                    self.pos += 1;
                    self.skip_ws();
                    Combinator::Child
                }
                Some(Tok::Plus) => {
                    self.pos += 1;
                    self.skip_ws();
                    Combinator::NextSibling
                }
                Some(Tok::Tilde) => {
                    self.pos += 1;
                    self.skip_ws();
                    Combinator::FollowingSibling
                }
                Some(_) if had_ws => Combinator::Descendant,
                Some(tok) => {
                    let offset = self.offset();
                    return Err(SelectorError::parse(
                        offset,
                        format!("스텝 뒤에 예상 밖 {}", tok.describe()),
                    )
                    .expecting([">", "+", "~", ",", "공백"]));
                }
            };
        }

        Ok(Path { steps })
    }

    /// 축 + 술어들.
    fn parse_step(&mut self, combinator: Combinator, source: &str) -> Result<Step, SelectorError> {
        let offset = self.offset();
        let axis = match self.peek() {
            Some(Tok::Star) => {
                self.pos += 1;
                Axis::Any
            }
            Some(Tok::Ident(name)) => {
                let name = name.clone();
                self.pos += 1;
                match AxisKind::from_name(&name) {
                    Some(k) => Axis::Kind(k),
                    None => return Err(unknown_axis(&name, offset)),
                }
            }
            // 축을 생략한 스텝은 `*` 이다 — CSS 의 `:not(:empty)`·`[k=v]` 와 같은
            // 규칙. 토큰을 소비하지 않으므로 뒤이어 술어가 반드시 하나 이상 붙고,
            // 따라서 폭 0 스텝이 만들어지는 경로는 없다(무한 루프 불가).
            Some(Tok::LBracket) | Some(Tok::Colon) => Axis::Any,
            Some(tok) => {
                return Err(
                    SelectorError::parse(offset, format!("축 자리에 {}", tok.describe()))
                        .expecting(AXIS_NAMES.iter().map(|(n, _)| *n).chain(["*"])),
                );
            }
            None => {
                return Err(SelectorError::parse(offset, "축 없이 선택자가 끝났다")
                    .expecting(AXIS_NAMES.iter().map(|(n, _)| *n).chain(["*"])));
            }
        };

        let mut preds = Vec::new();
        loop {
            match self.peek() {
                Some(Tok::LBracket) => preds.push(Pred::Attr(self.parse_attr(axis)?)),
                Some(Tok::Colon) => preds.push(Pred::Pseudo(self.parse_pseudo(source)?)),
                _ => break,
            }
            if preds.len() > MAX_PREDS {
                return Err(SelectorError::limit(
                    self.offset(),
                    format!("한 스텝의 술어가 너무 많다 (상한 {MAX_PREDS})"),
                ));
            }
        }

        Ok(Step {
            combinator,
            axis,
            preds,
            offset,
        })
    }

    /// `[name]` 또는 `[name op value]`.
    fn parse_attr(&mut self, axis: Axis) -> Result<AttrPred, SelectorError> {
        self.expect(&Tok::LBracket, "속성 술어 시작")?;
        self.skip_ws();

        let name_offset = self.offset();
        let name = match self.bump() {
            Some(Tok::Ident(n)) => n.clone(),
            Some(tok) => {
                return Err(SelectorError::parse(
                    name_offset,
                    format!("속성 이름 자리에 {}", tok.describe()),
                )
                .expecting(axis.attributes().iter().map(|a| a.name)));
            }
            None => {
                return Err(
                    SelectorError::parse(name_offset, "속성 이름 없이 선택자가 끝났다")
                        .expecting(axis.attributes().iter().map(|a| a.name)),
                );
            }
        };

        let def = match axis.attributes().iter().find(|a| a.name == name) {
            Some(d) => *d,
            None => return Err(unknown_attr(axis, &name, name_offset)),
        };

        self.skip_ws();
        let compare = if self.peek().is_some_and(Tok::is_compare_op) {
            let op_offset = self.offset();
            let op = cmp_op(self.bump().expect("직전에 확인했다"));
            if !def.ty.accepts(op) {
                return Err(SelectorError::resolve(
                    op_offset,
                    format!(
                        "{} 타입 속성 `{name}` 에는 `{}` 를 쓸 수 없다",
                        def.ty.as_str(),
                        op.symbol()
                    ),
                )
                .expecting(
                    ALL_OPS
                        .iter()
                        .filter(|o| def.ty.accepts(**o))
                        .map(|o| o.symbol()),
                )
                .hinting(match def.ty {
                    AttrType::Str => {
                        "문자열은 대소 비교를 하지 않는다 — 로캘에 따라 답이 달라지기 때문"
                    }
                    AttrType::Int => "정수에는 부분 일치를 쓸 수 없다",
                    AttrType::Bool => "불리언은 `=` 와 `!=` 만 받는다",
                }));
            }

            self.skip_ws();
            let lit_offset = self.offset();
            let lit = self.parse_literal(def.ty, &name, lit_offset)?;

            // 글롭은 여기서 컴파일해 본다. 평가까지 미루면 패턴 오류가 "결과 0건"
            // 으로 둔갑해 원인을 알 수 없게 된다.
            if op == CmpOp::Glob {
                if let Literal::Str(pattern) = &lit {
                    Glob::compile(pattern, lit_offset + 1)?;
                }
            }

            Some((op, lit))
        } else {
            None
        };

        self.skip_ws();
        self.expect(&Tok::RBracket, "속성 술어 끝")?;

        Ok(AttrPred {
            name,
            compare,
            offset: name_offset,
        })
    }

    /// 속성 타입에 맞는 리터럴 하나.
    fn parse_literal(
        &mut self,
        ty: AttrType,
        attr: &str,
        offset: usize,
    ) -> Result<Literal, SelectorError> {
        let tok = self.bump().cloned();
        let lit =
            match tok {
                Some(Tok::Str(s)) => Literal::Str(s),
                Some(Tok::Int(n)) => Literal::Int(n),
                // 따옴표 없는 식별자는 문자열 값으로 본다 — `[kind=table]` 이 가장 흔한
                // 모양이라 매번 따옴표를 요구하면 실수 유발이 더 크다. `true`/`false`
                // 만 불리언으로 승격한다.
                Some(Tok::Ident(s)) => match s.as_str() {
                    "true" => Literal::Bool(true),
                    "false" => Literal::Bool(false),
                    _ => Literal::Str(s),
                },
                Some(tok) => {
                    return Err(SelectorError::parse(
                        offset,
                        format!("값 자리에 {}", tok.describe()),
                    )
                    .expecting(["문자열", "정수", "식별자"]));
                }
                None => {
                    return Err(SelectorError::parse(offset, "값 없이 선택자가 끝났다")
                        .expecting(["문자열", "정수", "식별자"]));
                }
            };

        let ok = matches!(
            (ty, &lit),
            (AttrType::Str, Literal::Str(_))
                | (AttrType::Int, Literal::Int(_))
                | (AttrType::Bool, Literal::Bool(_))
        );
        if !ok {
            let hint = match ty {
                AttrType::Str => "문자열 값은 따옴표로 감싸거나 따옴표 없는 이름으로 적는다",
                AttrType::Int => "정수 값에 따옴표를 쓰지 않는다",
                AttrType::Bool => "`true` 또는 `false` 만 온다",
            };
            return Err(SelectorError::resolve(
                offset,
                format!(
                    "속성 `{attr}` 은 {} 인데 {} 값이 왔다",
                    ty.as_str(),
                    lit.type_name()
                ),
            )
            .expecting([ty.as_str()])
            .hinting(hint));
        }

        Ok(lit)
    }

    /// `:name` 또는 `:name(args)`.
    fn parse_pseudo(&mut self, source: &str) -> Result<Pseudo, SelectorError> {
        self.expect(&Tok::Colon, "의사 선택자 시작")?;
        let name_offset = self.offset();
        let name = match self.bump() {
            Some(Tok::Ident(n)) => n.clone(),
            Some(tok) => {
                return Err(SelectorError::parse(
                    name_offset,
                    format!("의사 선택자 이름 자리에 {}", tok.describe()),
                )
                .expecting(PSEUDO_DEFS.iter().map(|d| d.name)));
            }
            None => {
                return Err(SelectorError::parse(
                    name_offset,
                    "의사 선택자 이름 없이 선택자가 끝났다",
                )
                .expecting(PSEUDO_DEFS.iter().map(|d| d.name)));
            }
        };

        let def = match PseudoDef::from_name(&name) {
            Some(d) => d,
            None => {
                let err =
                    SelectorError::resolve(name_offset, format!("알 수 없는 의사 선택자 `{name}`"))
                        .expecting(PSEUDO_DEFS.iter().map(|d| d.name));
                return Err(
                    match super::ast::nearest(&name, PSEUDO_DEFS.iter().map(|d| d.name)) {
                        Some(c) => err.hinting(format!("`:{c}` 를 뜻했나")),
                        None => err,
                    },
                );
            }
        };

        if def.arity == PseudoArity::None {
            if matches!(self.peek(), Some(Tok::LParen)) {
                return Err(SelectorError::resolve(
                    self.offset(),
                    format!("`:{name}` 은 인자를 받지 않는다"),
                )
                .hinting("괄호를 지운다"));
            }
            return Ok(match name.as_str() {
                "first" => Pseudo::First,
                "last" => Pseudo::Last,
                "empty" => Pseudo::Empty,
                other => unreachable!("무인자 의사 선택자 `{other}` 가 사전에만 있다"),
            });
        }

        self.expect(&Tok::LParen, format!("`:{name}` 의 인자 목록").as_str())?;
        self.skip_ws();

        let pseudo = match def.arity {
            PseudoArity::None => unreachable!("위에서 돌아갔다"),
            PseudoArity::Int => Pseudo::Nth(self.parse_int_arg(&name)?),
            PseudoArity::Str => {
                let offset = self.offset();
                let text = self.parse_str_arg(&name)?;
                if name == "matches" {
                    // 인자 문자열의 첫 글자는 여는 따옴표 다음이다.
                    Glob::compile(&text, offset + 1)?;
                    Pseudo::Matches(text)
                } else {
                    Pseudo::Contains(text)
                }
            }
            PseudoArity::Range => {
                let from = self.parse_int_arg(&name)?;
                self.skip_ws();
                self.expect(&Tok::DotDot, "범위 구분자")?;
                self.skip_ws();
                let to = self.parse_int_arg(&name)?;
                // 같은 부호끼리만 대소를 판정할 수 있다. `0..-1` 은 "처음부터
                // 마지막 직전까지"라는 뜻이 성립하므로 오류가 아니다.
                if from.signum() == to.signum() && from > to {
                    return Err(SelectorError::resolve(
                        self.offset(),
                        format!("뒤집힌 범위 `{from}..{to}`"),
                    )
                    .hinting("반열림 구간이라 시작이 끝보다 작아야 한다"));
                }
                Pseudo::Range { from, to }
            }
            PseudoArity::Selector => {
                if self.depth + 1 > MAX_NESTING {
                    return Err(SelectorError::limit(
                        self.offset(),
                        format!("중첩 선택자가 너무 깊다 (상한 {MAX_NESTING})"),
                    ));
                }
                self.depth += 1;
                let paths = self.parse_paths(source)?;
                self.depth -= 1;
                let inner = Selector {
                    paths,
                    source: source.to_string(),
                };
                match name.as_str() {
                    "not" => Pseudo::Not(Box::new(inner)),
                    "has" => Pseudo::Has(Box::new(inner)),
                    other => unreachable!("선택자 인자 의사 선택자 `{other}` 가 사전에만 있다"),
                }
            }
        };

        self.skip_ws();
        self.expect(&Tok::RParen, format!("`:{name}` 의 인자 목록 끝").as_str())?;
        Ok(pseudo)
    }

    fn parse_int_arg(&mut self, pseudo: &str) -> Result<i64, SelectorError> {
        let offset = self.offset();
        match self.bump() {
            Some(Tok::Int(n)) => Ok(*n),
            Some(tok) => Err(SelectorError::parse(
                offset,
                format!("`:{pseudo}` 인자 자리에 {}", tok.describe()),
            )
            .expecting(["정수"])),
            None => Err(SelectorError::parse(
                offset,
                format!("`:{pseudo}` 인자 없이 선택자가 끝났다"),
            )
            .expecting(["정수"])),
        }
    }

    fn parse_str_arg(&mut self, pseudo: &str) -> Result<String, SelectorError> {
        let offset = self.offset();
        match self.bump() {
            Some(Tok::Str(s)) => Ok(s.clone()),
            Some(Tok::Ident(s)) => Ok(s.clone()),
            Some(tok) => Err(SelectorError::parse(
                offset,
                format!("`:{pseudo}` 인자 자리에 {}", tok.describe()),
            )
            .expecting(["문자열"])
            .hinting("공백이나 기호가 든 값은 따옴표로 감싼다")),
            None => Err(SelectorError::parse(
                offset,
                format!("`:{pseudo}` 인자 없이 선택자가 끝났다"),
            )
            .expecting(["문자열"])),
        }
    }
}

/// 비교 연산자 전체 — 오류 메시지의 후보 목록에 쓴다.
const ALL_OPS: &[CmpOp] = &[
    CmpOp::Eq,
    CmpOp::Ne,
    CmpOp::Gt,
    CmpOp::Lt,
    CmpOp::Ge,
    CmpOp::Le,
    CmpOp::Prefix,
    CmpOp::Suffix,
    CmpOp::Substr,
    CmpOp::Glob,
];

fn cmp_op(tok: &Tok) -> CmpOp {
    match tok {
        Tok::Eq => CmpOp::Eq,
        Tok::Ne => CmpOp::Ne,
        Tok::Gt => CmpOp::Gt,
        Tok::Lt => CmpOp::Lt,
        Tok::Ge => CmpOp::Ge,
        Tok::Le => CmpOp::Le,
        Tok::Prefix => CmpOp::Prefix,
        Tok::Suffix => CmpOp::Suffix,
        Tok::Substr => CmpOp::Substr,
        Tok::Glob => CmpOp::Glob,
        other => unreachable!("비교 연산자가 아닌 {other:?} 로 호출됐다"),
    }
}

/// 렉싱 결과를 재사용하는 내부 진입점 — 스키마 산출기가 문법 예시를 검증할 때 쓴다.
#[allow(dead_code)]
pub(crate) fn parse_lexed(lexed: &Lexed, source: &str) -> Result<Selector, SelectorError> {
    let mut parser = Parser {
        toks: &lexed.tokens,
        char_len: lexed.char_len,
        pos: 0,
        depth: 0,
    };
    let paths = parser.parse_paths(source)?;
    Ok(Selector {
        paths,
        source: source.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(src: &str) -> Selector {
        parse(src).unwrap_or_else(|e| panic!("파싱 실패: {}", e.render(src)))
    }

    fn err(src: &str) -> SelectorError {
        parse(src).expect_err("파싱이 성공하면 안 된다")
    }

    #[test]
    fn single_axis() {
        let sel = ok("para");
        assert_eq!(sel.paths.len(), 1);
        assert_eq!(sel.paths[0].steps.len(), 1);
        assert_eq!(sel.paths[0].steps[0].axis, Axis::Kind(AxisKind::Para));
        assert_eq!(sel.paths[0].steps[0].combinator, Combinator::Root);
    }

    #[test]
    fn combinators_are_distinguished() {
        let sel = ok("section > table cell + para ~ run");
        let combs: Vec<Combinator> = sel.paths[0].steps.iter().map(|s| s.combinator).collect();
        assert_eq!(
            combs,
            vec![
                Combinator::Root,
                Combinator::Child,
                Combinator::Descendant,
                Combinator::NextSibling,
                Combinator::FollowingSibling,
            ]
        );
    }

    /// 오프셋·원문을 뺀 구조만 뽑는다 — 공백 차이는 오프셋을 바꾸므로
    /// `Selector` 통째 비교로는 "구조가 같다"를 확인할 수 없다.
    fn shape(sel: &Selector) -> Vec<Vec<(Combinator, Axis)>> {
        sel.paths
            .iter()
            .map(|p| p.steps.iter().map(|s| (s.combinator, s.axis)).collect())
            .collect()
    }

    #[test]
    fn whitespace_around_explicit_combinators_is_optional() {
        assert_eq!(shape(&ok("section>table")), shape(&ok("section > table")));
        assert_eq!(shape(&ok("para+para")), shape(&ok("para  +  para")));
        assert_eq!(shape(&ok("para~para")), shape(&ok("para\t~\tpara")));
    }

    #[test]
    fn union_paths_split_on_comma() {
        let sel = ok("para, table, cell");
        assert_eq!(sel.paths.len(), 3);
    }

    #[test]
    fn trailing_whitespace_is_not_a_descendant_combinator() {
        // 끝의 공백이 결합자로 읽히면 "축 없이 끝났다"로 죽는다.
        ok("para ");
        ok("  para  ,  table  ");
    }

    #[test]
    fn attribute_predicates_bind_to_the_axis() {
        let sel = ok(r#"para[text*="합계"]"#);
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Attr(a) => {
                assert_eq!(a.name, "text");
                assert_eq!(
                    a.compare,
                    Some((CmpOp::Substr, Literal::Str("합계".into())))
                );
            }
            other => panic!("속성 술어가 아니다: {other:?}"),
        }
    }

    #[test]
    fn unknown_axis_names_the_candidates() {
        let e = err("tabel");
        assert!(e.hint.unwrap().contains("table"));
    }

    #[test]
    fn attribute_unknown_on_this_axis_is_rejected_even_if_valid_elsewhere() {
        // `rows` 는 table 의 속성이지 para 의 속성이 아니다. 문법만 보는 파서라면
        // 통과시켰을 자리.
        let e = err("para[rows>1]");
        assert!(e.message.contains("축 `para` 에 없는 속성"));
    }

    #[test]
    fn string_attributes_reject_ordering_and_explain_why() {
        let e = err(r#"para[text>="가"]"#);
        assert!(e.hint.unwrap().contains("로캘"));
    }

    #[test]
    fn int_attribute_rejects_quoted_value() {
        let e = err(r#"para[len="3"]"#);
        assert!(e.message.contains("integer"));
        assert!(e.hint.unwrap().contains("따옴표"));
    }

    #[test]
    fn bare_identifier_is_a_string_value() {
        let sel = ok("control[kind=table]");
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Attr(a) => assert_eq!(a.compare, Some((CmpOp::Eq, Literal::Str("table".into())))),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn true_false_become_booleans() {
        let sel = ok("para[empty=true]");
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Attr(a) => assert_eq!(a.compare, Some((CmpOp::Eq, Literal::Bool(true)))),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn bare_attribute_is_an_existence_test() {
        let sel = ok("field[name]");
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Attr(a) => assert!(a.compare.is_none()),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn pseudo_arity_is_enforced() {
        assert!(err("para:first(1)").message.contains("인자를 받지 않는다"));
        assert!(err("para:nth").message.contains("인자 목록"));
        assert!(err("para:nth(x)").message.contains("인자 자리에"));
    }

    #[test]
    fn range_pseudo_parses_and_rejects_inversion() {
        let sel = ok("para:range(1..4)");
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Pseudo(Pseudo::Range { from, to }) => {
                assert_eq!((*from, *to), (1, 4));
            }
            other => panic!("{other:?}"),
        }
        assert!(err("para:range(4..1)").message.contains("뒤집힌 범위"));
        // 부호가 다르면 뒤집힌 것이 아니다 — `0..-1` 은 유효한 뜻을 갖는다.
        ok("para:range(0..-1)");
    }

    #[test]
    fn glob_patterns_are_validated_at_parse_time() {
        let e = err(r#"para[text~="[abc"]"#);
        assert!(e.message.contains("닫히지 않은 글자 집합"));
        let e2 = err(r#"para:matches("[z-a]")"#);
        assert!(e2.message.contains("뒤집힌"));
    }

    #[test]
    fn glob_error_offset_points_inside_the_pattern() {
        // 패턴 안의 오류인데 오프셋이 선택자 시작을 가리키면 캐럿이 쓸모없다.
        let e = err(r#"para[text~="[abc"]"#);
        assert!(e.offset > 10, "오프셋이 패턴 안이어야 한다: {}", e.offset);
    }

    #[test]
    fn nested_selectors_parse() {
        let sel = ok("para:has(table)");
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Pseudo(Pseudo::Has(inner)) => {
                assert_eq!(inner.paths[0].steps[0].axis, Axis::Kind(AxisKind::Table));
            }
            other => panic!("{other:?}"),
        }
        ok("para:not(:empty)");
        ok("table:has(cell[text*=\"합계\"])");
    }

    #[test]
    fn nesting_depth_is_capped() {
        let mut src = String::from("para");
        for _ in 0..(MAX_NESTING + 2) {
            src = format!("para:has({src})");
        }
        let e = parse(&src).expect_err("상한을 넘겨야 한다");
        assert!(e.message.contains("너무 깊다"));
    }

    #[test]
    fn source_length_is_capped_before_lexing() {
        let src = "a".repeat(MAX_SOURCE_CHARS + 1);
        let e = parse(&src).expect_err("상한을 넘겨야 한다");
        assert!(e.message.contains("너무 길다"));
    }

    #[test]
    fn step_count_is_capped() {
        let src = vec!["para"; MAX_STEPS + 2].join(" > ");
        let e = parse(&src).expect_err("상한을 넘겨야 한다");
        assert!(e.message.contains("스텝이 너무 많다"));
    }

    #[test]
    fn omitted_axis_is_the_wildcard_like_css() {
        // `:not(:empty)` 의 안쪽 스텝에는 축이 없다. CSS 와 같은 규칙으로 `*` 이다.
        let sel = ok("[index=0]");
        assert_eq!(sel.paths[0].steps[0].axis, Axis::Any);
        assert_eq!(sel.paths[0].steps[0].preds.len(), 1);
        ok("para:not(:empty)");
        ok(":first");
    }

    #[test]
    fn space_before_a_predicate_reports_the_wildcard_cause() {
        // `para [len>0]` 은 문법 오류가 아니라 `para` 의 자손 `*[len>0]` 이다.
        // 그리고 `*` 에는 `len` 이 없다 — 진단이 그 인과를 그대로 말해야 한다.
        let e = err("para [len>0]");
        assert!(e.message.contains("축 `*` 에 없는 속성"));
        assert!(e.hint.unwrap().contains("자손 결합자"));
    }

    #[test]
    fn dangling_combinator_is_rejected() {
        assert!(err("para >").message.contains("축 없이"));
        assert!(err("> para").message.contains("축 자리에"));
        assert!(err("para,").message.contains("축 없이"));
    }

    #[test]
    fn result_axes_uses_only_the_last_step() {
        let sel = ok("table cell, section para");
        let axes = sel.result_axes();
        assert_eq!(axes.len(), 2);
        assert!(axes.contains(&Axis::Kind(AxisKind::Cell)));
        assert!(axes.contains(&Axis::Kind(AxisKind::Para)));
        // 경유 축은 들어가지 않는다.
        assert!(!axes.contains(&Axis::Kind(AxisKind::Table)));
    }

    #[test]
    fn max_nesting_is_reported_from_the_ast() {
        assert_eq!(ok("para").max_nesting(), 0);
        assert_eq!(ok("para:has(table)").max_nesting(), 1);
        assert_eq!(ok("para:has(table:has(cell))").max_nesting(), 2);
    }

    #[test]
    fn wildcard_axis_only_takes_common_attributes() {
        ok("*[index=0]");
        assert!(err("*[text=\"x\"]").message.contains("축 `*` 에 없는 속성"));
    }

    #[test]
    fn korean_values_survive_parsing() {
        let sel = ok(r#"field[name="수급자성명"]"#);
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Attr(a) => assert_eq!(
                a.compare,
                Some((CmpOp::Eq, Literal::Str("수급자성명".into())))
            ),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn trailing_garbage_is_reported_at_the_right_place() {
        let e = err("para]");
        assert_eq!(e.offset, 4);
    }
}
