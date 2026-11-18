//! 첨자의 버전별 크기·기준선 계약. 측정과 모든 paint가 같은 결과를 소비한다.

use std::fmt::Write;

use super::TextStyle;
use crate::model::provenance::LayoutCompatibilityProfile;

/// 문서 데이터에 저장하지 않는 세션 조판 계약이다.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScriptMetricsContract {
    #[default]
    Legacy,
    /// 한컴 2020·2024 Print의 공통 계약. 최초 2024 검증 때의 variant 이름을 유지한다.
    Hangul2024,
    /// Note auto-number codes share the reference marker's measured metrics.
    NoteReference,
}
crate::storage::clone_copy!(ScriptMetricsContract);

impl ScriptMetricsContract {
    pub(crate) fn for_profile(profile: LayoutCompatibilityProfile) -> Self {
        // 2024 전용 조판 옵션으로 첨자 계약까지 제한하면 한컴 2020/2022의
        // HWP5·HWPX도 0.7배로 재어 경계의 본문 글자를 다음 줄로 밀게 된다.
        // 독립 2020 Print 3글꼴×3크기와 2024 18조건은 같은 크기·기준선
        // 계약을 확인한다. HWP3 원본/호환 계보와 다른 입력의 기존 계약은 보존한다.
        if !profile.hwp3_layout()
            && !profile.hwp3_native_layout()
            && (profile.hangul2024_layout() || profile.hwp5_container() || profile.hwpx_container())
        {
            Self::Hangul2024
        } else {
            Self::Legacy
        }
    }

    pub fn font_size(self, base_size: f64, superscript: bool, subscript: bool) -> f64 {
        if !superscript && !subscript {
            return base_size;
        }
        base_size
            * match self {
                Self::Legacy => super::SCRIPT_FONT_SCALE,
                Self::Hangul2024 => 0.64,
                Self::NoteReference => 0.75,
            }
    }

    pub fn baseline_shift(self, base_size: f64, superscript: bool, subscript: bool) -> f64 {
        let ratio = match (self, superscript, subscript) {
            (Self::Legacy, true, _) => -super::SUPERSCRIPT_RISE_EM,
            (Self::Legacy, false, true) => super::SUBSCRIPT_DROP_EM,
            (Self::Hangul2024, true, _) => -0.44,
            (Self::Hangul2024, false, true) => 0.12,
            (Self::NoteReference, true, _) => -0.21,
            (Self::NoteReference, false, true) => 0.0,
            (_, false, false) => return 0.0,
        };
        base_size * ratio
    }

    pub fn draw_metrics(
        self,
        base_size: f64,
        baseline: f64,
        superscript: bool,
        subscript: bool,
        font_position: Option<i8>,
    ) -> (f64, f64) {
        // 원본 글자 위치는 본문 크기의 백분율로 한 번만 적용한다.
        let baseline = baseline + base_size * f64::from(font_position.unwrap_or_default()) / 100.0;
        let size = self.font_size(base_size, superscript, subscript);
        if !superscript && !subscript {
            return (size, baseline);
        }
        (
            size,
            baseline + self.baseline_shift(base_size, superscript, subscript),
        )
    }

    /// CanvasKit도 생산자가 확정한 값만 소비한다. 기존 payload는 그대로 유지한다.
    pub(crate) fn write_json(
        self,
        buf: &mut String,
        base_size: f64,
        superscript: bool,
        subscript: bool,
        font_position: Option<i8>,
    ) {
        if self == Self::Legacy || (!superscript && !subscript) {
            return;
        }
        let (size, shift) =
            self.draw_metrics(base_size, 0.0, superscript, subscript, font_position);
        let _ = write!(
            buf,
            ",\"scriptMetrics\":{{\"fontSize\":{size:.6},\"baselineShift\":{shift:.6}}}"
        );
    }
}

impl TextStyle {
    /// 전진 폭・kerning・shaping도 paint와 같은 첨자 크기를 사용한다.
    pub fn script_font_size(&self, base_size: f64) -> f64 {
        self.script_metrics
            .font_size(base_size, self.superscript, self.subscript)
    }
}
