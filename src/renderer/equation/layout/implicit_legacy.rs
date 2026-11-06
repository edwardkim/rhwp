//! HWP5의 생략된 선택 문자열이 사용하는 구형 한글·글자 줄 표시 계약.
//! 독립 한컴 2020의 두 크기·한글/아래첨자·빈 행 대조는 통합 분석 문서에 기록한다.

use super::{EqLayout, EqNode, LayoutBox, LayoutKind};

fn atom(node: &EqNode) -> bool {
    match node.ungrouped() {
        EqNode::Text(_) | EqNode::Number(_) | EqNode::Space(_) => true,
        EqNode::Symbol(text) | EqNode::MathSymbol(text) => text.chars().all(|ch| {
            matches!(
                ch,
                '+' | '-' | '−' | '=' | '×' | '%' | ',' | '.' | ':' | '(' | ')'
            )
        }),
        EqNode::Row(items) => items.iter().all(atom),
        EqNode::FontStyle {
            style: super::FontStyleKind::Roman,
            body,
        }
        | EqNode::Color { body, .. } => atom(body),
        _ => false,
    }
}

pub(super) fn supports(node: &EqNode) -> bool {
    match node.ungrouped() {
        EqNode::Newline => true,
        EqNode::Row(items) => !items.is_empty() && items.iter().all(supports),
        EqNode::Subscript { base, sub } => atom(base) && atom(sub),
        EqNode::FontStyle {
            style: super::FontStyleKind::Roman,
            body,
        }
        | EqNode::Color { body, .. } => supports(body),
        _ => atom(node),
    }
}

pub(super) fn line_frame(mut body: LayoutBox, fs: f64) -> LayoutBox {
    let ascent = body.baseline.max(fs * 1.2 * 0.84);
    let descent = (body.height - body.baseline).max(fs * 1.2 * (1.0 - 0.84));
    body.y = ascent - body.baseline;
    LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width: body.width,
        height: ascent + descent,
        baseline: ascent,
        kind: LayoutKind::Row(vec![body]),
    }
}

pub(super) fn cjk_text(text: &str, fs: f64) -> LayoutBox {
    let mut width = 0.0;
    let positions = text
        .chars()
        .map(|ch| {
            let x = width;
            // 원본 HFT 표시의 1024 논리 전진과 1000 UPM 출력 프로그램을 구분한다.
            width += if super::super::text_has_cjk(&ch.to_string()) {
                fs * 1024.0 / 1000.0
            } else {
                super::estimate_text_width(&ch.to_string(), fs, false)
            };
            x
        })
        .collect();
    let run = super::super::hancom_glyphs::GlyphRun {
        font_family: "한양신명조, HY Sinmyeongjo, Haansoft Batang, 한컴바탕, Batang, 바탕, Noto Serif CJK KR, serif",
        glyphs: text.to_owned(),
        positions,
        shears: vec![0.0; text.chars().count()],
        width,
        baseline_shift: 0.0,
    };
    LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width,
        height: fs * 1.2,
        baseline: fs * 1.2 * 0.84,
        kind: LayoutKind::HancomGlyphs(run),
    }
}

/// 항을 모으기 위한 투명 Row는 연산자 앞뒤 문맥의 경계가 아니다.
/// 명시 Group·첨자·글꼴 전환의 소유는 그대로 둔다.
pub(super) fn flatten_terms(nodes: &[EqNode]) -> Vec<EqNode> {
    let mut output = Vec::new();
    for node in nodes {
        if let EqNode::Row(children) = node {
            output.extend(flatten_terms(children));
        } else {
            output.push(node.clone());
        }
    }
    output
}

impl EqLayout {
    pub(super) fn layout_implicit_rows(&self, nodes: &[EqNode], fs: f64) -> LayoutBox {
        let mut height = 0.0_f64;
        let mut width = 0.0_f64;
        let mut rows: Vec<LayoutBox> = nodes
            .split(|node| matches!(node, EqNode::Newline))
            .enumerate()
            .map(|(index, nodes)| {
                let mut row = line_frame(self.layout_row(nodes, fs), fs);
                row.y = index as f64 * fs * 1.2;
                height = height.max(row.y + row.height);
                width = width.max(row.width);
                row
            })
            .collect();
        // 독립 구형 Print의 각 행은 마지막 항의 논리 끝을 공유한다.
        // 선행/후행 명시 공백도 행의 폭에 포함한 뒤 같은 오른쪽 원점을 쓴다.
        for row in &mut rows {
            row.x = width - row.width;
        }
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height,
            baseline: height / 2.0,
            kind: LayoutKind::Row(rows),
        }
    }
}

/// 같은 논리 프레임 안에서도 한글 대체와 수학 프로그램의 기준선·첨자 간격을 구분한다.
pub(super) fn has_cjk(body: &LayoutBox) -> bool {
    match &body.kind {
        LayoutKind::HancomGlyphs(run) => super::super::text_has_cjk(&run.glyphs),
        LayoutKind::Text(text) => super::super::text_has_cjk(text),
        LayoutKind::Row(items) => items.iter().any(has_cjk),
        LayoutKind::Color { body, .. } | LayoutKind::FontStyle { body, .. } => has_cjk(body),
        _ => false,
    }
}
