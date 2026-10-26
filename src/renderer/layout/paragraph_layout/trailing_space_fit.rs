//! 보존할 말미 공백의 전진과, 본문을 압축해야 하는 점유를 구분한다.

use super::{
    effective_text_for_metrics, estimate_text_width_exact, ComposedLine, ResolvedStyleSet,
};

/// 원본의 평문 말미 공백은 실제 전진으로 남지만 앞 글리프를 압축하지 않는다.
/// 개체·필드·장식이 소유한 공백에는 같은 제외를 적용하지 않는다.
pub(super) fn plain_trailing_width(
    line: &ComposedLine,
    styles: &ResolvedStyleSet,
    limit: usize,
) -> f64 {
    let mut remaining = limit;
    let mut width = 0.0;
    for run in line.runs.iter().rev() {
        if remaining == 0
            || run.inserted_control_text
            || run.char_overlap.is_some()
            || run.display_text.is_some()
        {
            break;
        }
        let text = effective_text_for_metrics(run);
        let count = text
            .chars()
            .rev()
            .take_while(|&c| c == ' ')
            .count()
            .min(remaining);
        if count == 0 {
            break;
        }
        let style = run.text_style(styles);
        if style.underline != crate::renderer::UnderlineType::None || style.strikethrough {
            break;
        }
        width += estimate_text_width_exact(&" ".repeat(count), &style);
        remaining -= count;
        if count != text.chars().count() {
            break;
        }
    }
    width
}
