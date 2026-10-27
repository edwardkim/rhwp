//! 한컴 수식 스크립트 파싱 및 렌더링
//!
//! 수식 스크립트(버전 6.0)를 토큰화하고 AST로 변환한 뒤 SVG로 렌더링한다.
//! 참조: openhwp/docs/hwpx/appendix-i-formula.md
//!
//! 명령 분기는 [`dispatch`] 가 가족으로 분류하고, [`parser`] 가 같은 핸들러로
//! 소비한다. 동작은 바꾸지 않는다. 모듈 지도는 `README.md`, 매뉴얼은
//! `mydocs/manual/equation_module.md`.

mod accent_outline;
mod advance_contract;
pub mod ast;
mod box_types;
mod brace_outline;
mod brace_recovery;
pub(crate) mod brace_stretch;
#[cfg(target_arch = "wasm32")]
pub mod canvas_render;
pub(crate) mod declared_frame;
pub(crate) mod dispatch;
mod dot_geometry;
mod font_frame_22;
mod font_frame_metrics;
mod font_style_runs;
mod fraction_sup_frame;
mod hancom_eqn_metrics;
mod hancom_frame;
pub mod hancom_glyphs;
pub(crate) mod inline_box;
pub(crate) mod integral_outline;
pub(crate) mod multiplication_outline;
pub(crate) mod negation_geometry;
mod not_equal_geometry;
mod operator_outline;
mod script_fraction_frame;
pub use integral_outline::Command as GlyphOutlineCommand;
pub use negation_geometry::NegationGeometry;
pub(crate) mod document_script;
pub mod layout;
pub(crate) mod legacy_hwpeq;
mod logical_font_frame;
pub mod parser;
mod radical_line_frame;
pub(crate) mod radical_outline;
mod relation_outline;
mod round_stretch;
mod stored_alignment_metrics;
pub mod svg_render;
pub mod symbols;
pub mod tokenizer;
mod uncached_flow_metrics;
mod variable_outline;

/// 한글 수식은 수학 글꼴의 누락 글리프 대신 본문 명조 계열을 사용한다.
pub(crate) const CJK_EQUATION_FONT_FAMILY: &str = "Haansoft Batang, 한컴바탕, Batang, 바탕, serif";

pub(crate) fn text_has_cjk(text: &str) -> bool {
    text.chars().any(|ch| {
        matches!(ch,
        '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}')
    })
}

/// 기본 글자보다 낮은 저장 수식 상자는 압축된 표시 프레임이다.
/// 일반 수식 높이에는 줄 여백이 포함될 수 있으므로 기존 글꼴 비율을 유지한다.
/// 압축 프레임에서는 배치 기준선과 모든 출력 backend가 같은 세로 비율을 쓴다.
pub(crate) fn stored_vertical_scale(height: f64, intrinsic_height: f64, font_size: f64) -> f64 {
    if height > 0.0 && height < font_size && intrinsic_height > 0.0 {
        height / intrinsic_height
    } else {
        1.0
    }
}

/// A Hancom linear glyph run paints at natural size while the stored object
/// advance still owns the following text. This is also observed after Hancom Save.
pub(crate) fn stored_horizontal_scale(width: f64, layout: &layout::LayoutBox) -> f64 {
    if hancom_glyphs::is_linear(layout) || layout.width <= 0.0 || width <= 0.0 {
        1.0
    } else {
        width / layout.width
    }
}

/// 수식 스크립트와 BaseUnit에서 레이아웃이 소비할 intrinsic HWPUNIT 크기를 계산한다.
pub fn intrinsic_size_hwp(script: &str, font_size: u32) -> (u32, u32) {
    let font_size_px = super::hwpunit_to_px(font_size.max(1) as i32, super::DEFAULT_DPI);
    let tokens = document_script::tokenize(script);
    let ast = parser::EqParser::for_document(tokens).parse();
    let layout = layout::EqLayout::new(font_size_px).layout(&ast);
    (
        super::px_to_hwpunit(layout.width, super::DEFAULT_DPI).max(1) as u32,
        super::px_to_hwpunit(layout.height, super::DEFAULT_DPI).max(1) as u32,
    )
}

/// Legacy edits must produce the logical HFT frame used by their painter.
/// The modern editor's existing sizing contract remains unchanged.
pub(crate) fn edited_size_hwp(equation: &crate::model::control::Equation) -> (u32, u32) {
    if !uses_legacy_hft_metrics(equation) {
        return intrinsic_size_hwp(&equation.script, equation.font_size);
    }
    let ast = parser::EqParser::new(tokenizer::tokenize(&equation.script)).parse();
    let layout = layout::EqLayout::for_equation(equation, super::DEFAULT_DPI).layout(&ast);
    (
        super::px_to_hwpunit(layout.width, super::DEFAULT_DPI).max(1) as u32,
        super::px_to_hwpunit(layout.height, super::DEFAULT_DPI).max(1) as u32,
    )
}

/// 수학 글꼴 HYhwpEQ에는 한글 cmap이 없으므로 한글은 본문 대체 글꼴로 그린다.
/// 일반 수식의 저장 프레임과 보호된 크기는 유지한다. 독립 한컴 출력에서
/// 확인한 글자 줄·윗줄·단일 위첨자·BOX의 낡은 캐시만 논리 상자로 재발행한다.
/// 보호된 크기와 한글 대체가 없는 수식은 저장 상자 계약을 유지한다.
/// legacy HFT 글리프 메트릭 선택 자체는 이 물리 상자의 재조판 증거가 아니다.
/// 대체 경로는 폭·높이·기준선을 같은 AST에서 얻어 측정과 paint가 함께 소비한다.
pub(crate) fn flow_metrics_hwp(equation: &crate::model::control::Equation) -> (u32, u32, i32) {
    if declared_frame::retains_unpositioned_host_frame(equation) {
        return (
            equation.common.width,
            equation.common.height,
            (equation.common.height as i32).saturating_mul(i32::from(equation.baseline)) / 100,
        );
    }
    cached_flow_metrics(equation)
}

/// 선언 프레임 밖 흐름 메트릭이 읽는 수식 값 전부다. 값이 같으면 결과도 같으므로
/// 편집으로 값이 바뀌면 다른 키가 되어 이전 결과를 재사용하지 않는다.
#[derive(PartialEq, Eq, Hash)]
struct FlowMetricsKey {
    script: String,
    font_name: String,
    version_info: String,
    font_size: u32,
    attr: u32,
    baseline: i16,
    width: u32,
    height: u32,
    size_protect: bool,
}

/// 미주 조판은 같은 수식의 메트릭을 판정마다 다시 묻는다. 토큰화·조판을 반복하지 않는다.
fn cached_flow_metrics(equation: &crate::model::control::Equation) -> (u32, u32, i32) {
    use std::cell::RefCell;
    use std::collections::HashMap;
    // 가득 차면 통째로 비운다. 키가 입력 값 전부라 비워도 결과는 같고 다시 계산할 뿐이다.
    const CAPACITY: usize = 4096;
    thread_local! {
        static CACHE: RefCell<HashMap<FlowMetricsKey, (u32, u32, i32)>> = RefCell::default();
    }
    // 새 필드가 생기면 컴파일이 멈춰 키에 넣을지 판단하게 한다.
    // eqedit·treat_as_char·봉인은 선언 프레임 판정만 읽고 그 판정은 캐시 앞에서 끝난다.
    let crate::model::control::Equation {
        common,
        attr,
        script,
        font_size,
        color: _,
        baseline,
        unknown: _,
        eqedit: _,
        version_info,
        font_name,
        raw_ctrl_data: _,
        raw_ctrl_seal: _,
        unpositioned_host_frame_seal: _,
    } = equation;
    let key = FlowMetricsKey {
        script: script.clone(),
        font_name: font_name.clone(),
        version_info: version_info.clone(),
        font_size: *font_size,
        attr: *attr,
        baseline: *baseline,
        width: common.width,
        height: common.height,
        size_protect: common.size_protect,
    };
    if let Some(metrics) = CACHE.with(|cache| cache.borrow().get(&key).copied()) {
        return metrics;
    }
    let metrics = computed_flow_metrics(equation);
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() >= CAPACITY {
            cache.clear();
        }
        cache.insert(key, metrics);
    });
    metrics
}

fn computed_flow_metrics(equation: &crate::model::control::Equation) -> (u32, u32, i32) {
    if let Some(intrinsic) = layout::EqLayout::legacy_flow_layout(equation, super::DEFAULT_DPI) {
        return (
            super::px_to_hwpunit(intrinsic.width, super::DEFAULT_DPI).max(1) as u32,
            super::px_to_hwpunit(intrinsic.height, super::DEFAULT_DPI).max(1) as u32,
            super::px_to_hwpunit(intrinsic.baseline, super::DEFAULT_DPI),
        );
    }
    // 표시 내용이 없는 구문과 유효한 빈 템플릿은 다르다. 정상 한컴 저장본의
    // 인자 없는 PILE은 범위 밖 기준선을 가진 무효 캐시를 남기지만, 호스트의
    // 빈 글줄은 여전히 점유한다. 캐시를 표시 상자의 크기로 보정하지 않는다.
    if !equation.common.size_protect
        && !(0..=100).contains(&equation.baseline)
        && matches!(
            parser::EqParser::for_document(document_script::tokenize(&equation.script)).parse(),
            ast::EqNode::Empty
        )
    {
        return (0, 0, 0);
    }
    if let Some(metrics) = uncached_flow_metrics::metrics(equation) {
        return metrics;
    }
    // An HFT control's declared logical slot remains authoritative when its
    // CJK paint uses another face. Missing frames still need intrinsic sizing;
    // actual legacy edits refresh the slot in their producer, not during paint.
    let stored_legacy_frame = uses_legacy_hft_metrics(equation)
        && equation.common.width > 0
        && equation.common.height > 0;
    if !equation.common.size_protect
        && !stored_legacy_frame
        && equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
        && equation
            .script
            .chars()
            .any(|ch| matches!(ch, '\u{AC00}'..='\u{D7A3}'))
    {
        let ast =
            parser::EqParser::for_document(document_script::tokenize(&equation.script)).parse();
        let intrinsic = layout::EqLayout::for_equation(equation, super::DEFAULT_DPI).layout(&ast);
        let font_frame = (equation.attr & 1 == 0 && equation.font_size > 0)
            .then(|| {
                logical_font_frame::metrics(
                    &intrinsic,
                    super::hwpunit_to_px(equation.font_size as i32, super::DEFAULT_DPI),
                )
            })
            .flatten();
        let (height, baseline) = font_frame.unwrap_or((intrinsic.height, intrinsic.baseline));
        (
            super::px_to_hwpunit(intrinsic.width, super::DEFAULT_DPI).max(1) as u32,
            super::px_to_hwpunit(height, super::DEFAULT_DPI).max(1) as u32,
            super::px_to_hwpunit(baseline, super::DEFAULT_DPI),
        )
    } else if let Some(metrics) = normalized_simple_math_metrics(equation) {
        metrics
    } else {
        (
            equation.common.width,
            equation.common.height,
            (f64::from(equation.common.height) * f64::from(equation.baseline) / 100.0).round()
                as i32,
        )
    }
}

/// 글자 줄·단일 위첨자·윗줄 및 기존 BOX의 논리 프레임 계약.
/// 복합 수식의 자연 글리프 높이를 유효한 저장 프레임과 동일시하지 않는다.
pub(crate) fn normalized_simple_math_metrics(
    equation: &crate::model::control::Equation,
) -> Option<(u32, u32, i32)> {
    // A valid legacy logical frame is shared by flow and HFT paint. Modern
    // simple-math cache normalization must not replace that stored owner;
    // editing refreshes the legacy frame before this query is called.
    if (uses_legacy_hft_metrics(equation)
        && equation.common.width > 0
        && equation.common.height > 0)
        || equation.common.size_protect
        || equation.attr & 1 != 0
        || equation.font_size == 0
        || !equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
    {
        return None;
    }
    let ast = parser::EqParser::for_document(document_script::tokenize(&equation.script)).parse();
    let intrinsic = layout::EqLayout::for_equation(equation, super::DEFAULT_DPI).layout(&ast);
    let fs = super::hwpunit_to_px(equation.font_size.max(1) as i32, super::DEFAULT_DPI);
    if intrinsic.width <= 0.0
        || math_layout_has_line_break(&intrinsic)
        || logical_font_frame::dyad_font_line(&intrinsic, fs)
    {
        // DYAD 글자 줄의 저장 높이는 표시 글리프보다 큰 예약 영역일 수 있다.
        // paint가 보존하는 그 영역을 일반 글자 캐시 재발행으로 줄이지 않는다.
        return None;
    }
    if let Some(metrics) = stored_alignment_metrics::metrics(equation, &intrinsic) {
        return Some(metrics);
    }
    let single_box_line =
        simple_box_height(&intrinsic, fs).is_some() && is_simple_box_expression(&intrinsic);
    let font_frame = logical_font_frame::metrics(&intrinsic, fs);
    if !single_box_line && font_frame.is_none() {
        return None;
    }
    let (height, baseline) = font_frame.unwrap_or((intrinsic.height, intrinsic.baseline));
    let height = super::px_to_hwpunit(height, super::DEFAULT_DPI).max(1) as u32;
    if equation.common.height <= height {
        return None;
    }
    // 높이가 무효여도 미지원 구조의 폭까지 무효라고 추정하지 않는다.
    // 독립 한컴 전진폭을 확인한 일반 글자 줄만 paint와 같은 논리 폭으로 재발행한다.
    let width = if advance_contract::has_source_advances(&ast) {
        super::px_to_hwpunit(intrinsic.width, super::DEFAULT_DPI).max(1) as u32
    } else {
        equation.common.width
    };
    Some((
        width,
        height,
        super::px_to_hwpunit(baseline, super::DEFAULT_DPI),
    ))
}

fn math_layout_has_line_break(value: &layout::LayoutBox) -> bool {
    match &value.kind {
        layout::LayoutKind::Newline => true,
        layout::LayoutKind::Row(items) => items.iter().any(math_layout_has_line_break),
        layout::LayoutKind::Color { body, .. } | layout::LayoutKind::FontStyle { body, .. } => {
            math_layout_has_line_break(body)
        }
        _ => false,
    }
}

fn simple_box_height(value: &layout::LayoutBox, fs: f64) -> Option<f64> {
    match &value.kind {
        layout::LayoutKind::Decoration {
            kind: symbols::DecoKind::Box,
            body,
            ..
        } if (body.height - fs).abs() <= f64::EPSILON * fs * 4.0 => Some(value.height),
        layout::LayoutKind::Row(items) => items
            .iter()
            .filter_map(|item| simple_box_height(item, fs))
            .max_by(f64::total_cmp),
        layout::LayoutKind::Color { body, .. } | layout::LayoutKind::FontStyle { body, .. } => {
            simple_box_height(body, fs)
        }
        layout::LayoutKind::Decoration { body, .. }
        | layout::LayoutKind::Sqrt { body, .. }
        | layout::LayoutKind::Paren { body, .. } => simple_box_height(body, fs),
        layout::LayoutKind::Superscript { base, sup } => {
            simple_box_height(base, fs).or_else(|| simple_box_height(sup, fs))
        }
        layout::LayoutKind::Subscript { base, sub } => {
            simple_box_height(base, fs).or_else(|| simple_box_height(sub, fs))
        }
        layout::LayoutKind::SubSup { base, sub, sup } => simple_box_height(base, fs)
            .or_else(|| simple_box_height(sub, fs))
            .or_else(|| simple_box_height(sup, fs)),
        _ => None,
    }
}

/// BOX 여백을 소비하는 단일 표현식의 합성. 분수·배열·큰 연산자와
/// 다줄의 저장 논리 프레임을 공개 글꼴 자연 높이로 재정의하지 않는다.
fn is_simple_box_expression(value: &layout::LayoutBox) -> bool {
    match &value.kind {
        layout::LayoutKind::Row(items) => items.iter().all(is_simple_box_expression),
        layout::LayoutKind::Color { body, .. }
        | layout::LayoutKind::FontStyle { body, .. }
        | layout::LayoutKind::Decoration { body, .. }
        | layout::LayoutKind::Paren { body, .. } => is_simple_box_expression(body),
        layout::LayoutKind::Sqrt { body, index } => {
            is_simple_box_expression(body) && index.as_deref().is_none_or(is_simple_box_expression)
        }
        layout::LayoutKind::Superscript { base, sup } => {
            is_simple_box_expression(base) && is_simple_box_expression(sup)
        }
        layout::LayoutKind::Subscript { base, sub } => {
            is_simple_box_expression(base) && is_simple_box_expression(sub)
        }
        layout::LayoutKind::SubSup { base, sub, sup } => {
            is_simple_box_expression(base)
                && is_simple_box_expression(sub)
                && is_simple_box_expression(sup)
        }
        layout::LayoutKind::Text(_)
        | layout::LayoutKind::Number(_)
        | layout::LayoutKind::Symbol(_)
        | layout::LayoutKind::MathSymbol(_)
        | layout::LayoutKind::FontGlyph { .. }
        | layout::LayoutKind::HancomGlyphs(_)
        | layout::LayoutKind::Function(_)
        | layout::LayoutKind::Space(_)
        | layout::LayoutKind::Empty => true,
        _ => false,
    }
}

/// 버전이 없는 기존 수식은 한컴에서 HFT 메트릭으로 출력된다.
/// 명시적인 현대 버전과 다른 글꼴은 기존 경로를 유지한다.
pub(crate) fn uses_legacy_hft_metrics(equation: &crate::model::control::Equation) -> bool {
    equation.version_info.trim().is_empty() && equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
}

pub(crate) fn flow_width_hwp(equation: &crate::model::control::Equation) -> u32 {
    flow_metrics_hwp(equation).0
}

fn inline_horizontal_margins(equation: &crate::model::control::Equation) -> (i32, i32) {
    if equation.common.treat_as_char {
        (
            i32::from(equation.common.margin.left),
            i32::from(equation.common.margin.right),
        )
    } else {
        (0, 0)
    }
}

pub(crate) fn inline_advance_width_hwp(equation: &crate::model::control::Equation) -> i32 {
    let (left, right) = inline_horizontal_margins(equation);
    (flow_width_hwp(equation) as i32)
        .saturating_add(left)
        .saturating_add(right)
}

/// The composer reserves this slot including margins. Each painter consumes
/// its content box from those same margins; subsequent text keeps the slot advance.
pub(crate) fn inline_content_box(
    equation: &crate::model::control::Equation,
    slot: super::render_tree::BoundingBox,
    dpi: f64,
) -> super::render_tree::BoundingBox {
    let (left, right) = inline_horizontal_margins(equation);
    let left_px = super::hwpunit_to_px(left, dpi);
    let right_px = super::hwpunit_to_px(right, dpi);
    super::render_tree::BoundingBox::new(
        slot.x + left_px,
        slot.y,
        slot.width - left_px - right_px,
        slot.height,
    )
}

pub(crate) fn flow_height_hwp(equation: &crate::model::control::Equation) -> u32 {
    flow_metrics_hwp(equation).1
}

/// 줄 측정과 실제 인라인 원점이 같은 수식 기준선 거리를 소비한다.
/// 저장 기준선이 없는 입력만 실제 수식 메트릭으로 되돌아간다.
pub(crate) fn inline_baseline_px(
    equation: &crate::model::control::Equation,
    intrinsic: &layout::LayoutBox,
    dpi: f64,
) -> f64 {
    let (_, height, baseline) = flow_metrics_hwp(equation);
    if height > 0 && baseline > 0 && baseline <= height as i32 {
        super::hwpunit_to_px(baseline, dpi)
    } else {
        intrinsic.baseline
            * stored_vertical_scale(
                super::hwpunit_to_px(height as i32, dpi),
                intrinsic.height,
                super::hwpunit_to_px(equation.font_size as i32, dpi),
            )
    }
}

/// 줄이 소비한 수식 프레임에 공통 paint 원점을 연결한다.
/// DYAD 글자 줄은 이미 배치한 예약 프레임 안의 자연 glyph 원점을 유지한다.
pub(crate) fn layout_for_paint(
    equation: &crate::model::control::Equation,
    ast: &ast::EqNode,
    dpi: f64,
) -> layout::LayoutBox {
    let mut result = layout::EqLayout::for_equation(equation, dpi).layout(ast);
    if declared_frame::retains_unpositioned_host_frame(equation) {
        // 호스트의 기준선은 선언 상자를 배치한다. 상자 안의 자연 표시 원점을
        // 같은 기준선에 다시 맞추면 예약 높이가 큰 수식의 글리프가 내려간다.
        return result;
    }
    let fs = super::hwpunit_to_px(equation.font_size as i32, dpi);
    if uses_legacy_hft_metrics(equation) && layout::legacy_logical_frame(&result, fs).is_some() {
        // HFT 논리 글줄은 layout이 이미 내부 표시 원점을 확정했다.
        // 저장 기준선 비율은 바깥 호스트 상자가 소비하므로 이를 내부 glyph에
        // 다시 적용하면 기준선만 편집해도 같은 수식 프로그램이 이동한다.
        return result;
    }
    let scale = stored_vertical_scale(
        super::hwpunit_to_px(flow_height_hwp(equation) as i32, dpi),
        result.height,
        fs,
    );
    if scale == 1.0
        && equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
        && logical_font_frame::dyad_font_line(&result, fs)
    {
        // bbox는 이미 저장 예약 프레임의 기준선으로 배치됐다. 자연 glyph를
        // 본문 기준선에 다시 맞추면 큰 예약 프레임 안의 위쪽 배치가 사라진다.
        return result;
    }
    result.y = inline_baseline_px(equation, &result, dpi) / scale - result.baseline;
    result
}

/// 측정한 한글 전진폭을 그대로 글자 원점에 적용한다. 글리프 자체 크기는 바꾸지 않는다.
/// CJK가 아닌 문자는 기존 추정 폭을 보존한다.
pub(crate) fn positioned_cjk_text(text: &str, font_size: f64, width: f64) -> Vec<(char, f64)> {
    let count = text
        .chars()
        .filter(|ch| text_has_cjk(&ch.to_string()))
        .count();
    let other_width: f64 = text
        .chars()
        .filter(|ch| !text_has_cjk(&ch.to_string()))
        .map(|ch| layout::estimate_text_width(&ch.to_string(), font_size, false))
        .sum();
    let advance = if count > 0 {
        (width - other_width) / count as f64
    } else {
        0.0
    };
    let mut x = 0.0;
    text.chars()
        .map(|ch| {
            let origin = x;
            x += if text_has_cjk(&ch.to_string()) {
                advance
            } else {
                layout::estimate_text_width(&ch.to_string(), font_size, false)
            };
            (ch, origin)
        })
        .collect()
}
