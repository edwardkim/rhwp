//! 표 생성 시 적용하는 열 너비·문단 정렬·머리행 계약.

use crate::model::style::Alignment;

#[derive(Debug)]
pub enum TableColumnWidths {
    /// HWPUNIT 절대 너비. 표 너비는 합계가 된다.
    Absolute(Vec<u32>),
    /// 기존 기본 표 너비에 대한 비율. 합계는 100이어야 한다.
    Percent(Vec<f64>),
}

#[derive(Debug, Default)]
pub struct TableCreationOptions {
    pub column_widths: Option<TableColumnWidths>,
    pub column_alignments: Option<Vec<Alignment>>,
    /// None이면 기존 코어 생성 기본값을 보존한다.
    pub repeat_header: Option<bool>,
}

impl TableCreationOptions {
    pub(crate) fn widths(&self, cols: u16, default_width: u32) -> Result<Vec<u32>, String> {
        let widths = match &self.column_widths {
            None => vec![default_width / u32::from(cols); usize::from(cols)],
            Some(TableColumnWidths::Absolute(widths)) => widths.clone(),
            Some(TableColumnWidths::Percent(percentages)) => {
                let total: f64 = percentages.iter().sum();
                if percentages.iter().any(|v| !v.is_finite() || *v <= 0.0)
                    || !total.is_finite()
                    || (total - 100.0).abs() > 1e-8
                {
                    return Err("열 너비 비율은 양수이고 합계가 100%여야 합니다".into());
                }
                // 누적 경계를 반올림해 정수 HWPUNIT으로 바꾼다. 마지막 경계는
                // default_width이므로 반올림 잔여가 표 전체 너비를 바꾸지 않는다.
                let mut accumulated = 0.0;
                let mut previous = 0;
                percentages
                    .iter()
                    .map(|value| {
                        accumulated += value;
                        let boundary =
                            (f64::from(default_width) * accumulated / total).round() as u32;
                        let width = boundary - previous;
                        previous = boundary;
                        width
                    })
                    .collect()
            }
        };
        if widths.len() != usize::from(cols) || widths.contains(&0) {
            return Err("열 너비 개수는 열 수와 같고 각 너비는 1 HWPUNIT 이상이어야 합니다".into());
        }
        let total = widths.iter().try_fold(0u32, |sum, w| sum.checked_add(*w));
        if total.is_none_or(|sum| sum > i32::MAX as u32) {
            return Err("열 너비 합계는 2147483647 HWPUNIT 이하여야 합니다".into());
        }
        if self
            .column_alignments
            .as_ref()
            .is_some_and(|v| v.len() != usize::from(cols))
        {
            return Err("열 정렬 개수는 열 수와 같아야 합니다".into());
        }
        Ok(widths)
    }
}
