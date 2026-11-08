//! HWP3 수식 상자의 원본 배치 정보와 공통 Equation IR의 단위를 연결한다.

use crate::model::{control::Equation, table::Table};

pub(super) fn from_box(table: Table, baseline_1800dpi: u16) -> Equation {
    let mut script = String::new();
    if let Some(cell) = table.cells.first() {
        for paragraph in &cell.paragraphs {
            script.push_str(&paragraph.text);
            script.push('\n');
        }
    }
    // HWP3 1800dpi 길이 → 7200dpi HU → 상자 높이의 백분율.
    let baseline_hu = u64::from(baseline_1800dpi) * 4;
    let height = u64::from(table.common.height);
    let baseline = if height > 0 && baseline_hu <= height {
        ((baseline_hu * 100 + height / 2) / height) as i16
    } else {
        0
    };
    Equation {
        common: table.common,
        script: script.trim().to_owned(),
        // HWP3의 별도 수식 기본 크기. 본문 글꼴 크기와 독립이며 한컴 Print/저장본과 대조했다.
        font_size: 1200,
        font_name: "HYhwpEQ".to_owned(),
        baseline,
        ..Default::default()
    }
}
