//! 한컴 rm/it 전환을 현재 글줄·그룹의 범위로 묶는다.
//!
//! 공백은 그대로 보존하고, 그룹 내부 전환은 바깥 형제 요소에 전파하지 않는다.
//! LaTeX의 명시적 인자형 FontStyle은 기존의 인자 범위를 유지한다.

use super::ast::EqNode;
use super::symbols::FontStyleKind;

pub(super) fn resolve(children: Vec<EqNode>) -> Vec<EqNode> {
    let mut result = Vec::new();
    let mut style = None;
    let mut run = Vec::new();
    for mut child in children {
        let switch = match &child {
            EqNode::Subscript { base, .. }
            | EqNode::Superscript { base, .. }
            | EqNode::SubSup { base, .. } => font_switch(base),
            _ => font_switch(&child),
        };
        if let Some(next) = switch {
            flush(&mut result, &mut run, style);
            style = Some(next);
            match &mut child {
                EqNode::Subscript { base, .. }
                | EqNode::Superscript { base, .. }
                | EqNode::SubSup { base, .. } => {
                    // 전환 marker는 가시 본체가 아니다. 첨자와 뒤 형제 항이
                    // 같은 전환 범위를 사용하고 빈 본체의 기준선은 보존한다.
                    **base = EqNode::Empty;
                    run.push(child);
                }
                _ => {}
            }
            continue;
        }
        run.push(child);
    }
    flush(&mut result, &mut run, style);
    result
}

fn font_switch(node: &EqNode) -> Option<FontStyleKind> {
    // 구형의 명시적 Group 안에 든 전환은 바깥 글줄의 전환 marker가 아니다.
    // Version 60은 Group을 만들지 않으므로 기존 전역 전환을 유지한다.
    if let EqNode::FontStyle { style, body } = node {
        if matches!(style, FontStyleKind::Roman | FontStyleKind::Italic)
            && matches!(body.as_ref(), EqNode::Empty)
        {
            return Some(*style);
        }
    }
    None
}

fn flush(result: &mut Vec<EqNode>, run: &mut Vec<EqNode>, style: Option<FontStyleKind>) {
    if run.is_empty() {
        return;
    }
    let Some(style) = style else {
        result.append(run);
        return;
    };
    let body = if run.len() == 1 {
        run.pop().unwrap()
    } else {
        EqNode::Row(std::mem::take(run))
    };
    result.push(EqNode::FontStyle {
        style,
        body: Box::new(body),
    });
}
