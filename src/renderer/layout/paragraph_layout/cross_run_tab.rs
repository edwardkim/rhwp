//! 교차 런 탭의 저장 전진과 현재 프레임의 정렬 정지를 구분한다.

use super::{
    estimate_text_width, estimate_text_width_exact, find_next_tab_stop, TabStop, TextStyle,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PendingTab {
    /// 유효한 저장 줄에서 이미 정렬된 다음 블록의 시작점이다.
    StoredStart {
        position: f64,
        fill_type: u8,
    },
    Right {
        position: f64,
        fill_type: u8,
    },
    Center {
        position: f64,
        fill_type: u8,
    },
}

impl PendingTab {
    /// 측정과 배치는 같은 단 기준 시작을 소비한다. 저장 전진에는 폭을 다시 빼지 않는다.
    pub(super) fn block_start(self, block_width: f64) -> f64 {
        match self {
            Self::StoredStart { position, .. } => position,
            Self::Right { position, .. } => position - block_width,
            Self::Center { position, .. } => position - block_width / 2.0,
        }
    }

    pub(super) fn is_right_alignment(self) -> bool {
        matches!(self, Self::Right { .. })
    }
}

/// `extensions`는 composition과 출력 프로필이 수용한 현재 런의 기하만 전달한다.
/// 저장 줄 부재·재조판·HWP3 재정렬은 생산자에서 저장 전진을 제거한다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_last_tab_pending(
    run_text: &str,
    last_inline_idx: usize,
    extensions: &[[u16; 7]],
    style: &TextStyle,
    stops: &[TabStop],
    tab_width: f64,
    auto_right: bool,
    available_width: f64,
) -> Option<PendingTab> {
    let last_tab = run_text.rfind('\t')?;
    let before = &run_text[..last_tab];
    if let Some(ext) = extensions
        .get(last_inline_idx)
        .filter(|ext| !crate::model::paragraph::tab_ext_is_placeholder(ext))
    {
        match ext[2] >> 8 {
            0 | 1 => return None,
            2 => {
                // HWP5와 HWPX의 width는 하위·상위 16비트를 합친 저장 전진이다.
                // 자동 우측 탭은 현재 프레임·블록 폭으로 다시 정렬하는 계약을 유지한다.
                let width_hu = u32::from(ext[0]) | (u32::from(ext[1]) << 16);
                if !auto_right && width_hu > 0 {
                    return Some(PendingTab::StoredStart {
                        position: style.line_x_offset
                            + estimate_text_width_exact(before, style)
                            + f64::from(width_hu) * 96.0 / 7200.0,
                        fill_type: (ext[2] & 0xff) as u8,
                    });
                }
            }
            3 => {}
            _ => return None,
        }
    }
    let (position, kind, fill_type) = find_next_tab_stop(
        style.line_x_offset + estimate_text_width(before, style),
        stops,
        if tab_width > 0.0 { tab_width } else { 48.0 },
        auto_right,
        available_width,
        style.text_start_offset,
    );
    match kind {
        1 => Some(PendingTab::Right {
            position,
            fill_type,
        }),
        2 => Some(PendingTab::Center {
            position,
            fill_type,
        }),
        _ => None,
    }
}
