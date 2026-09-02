//! 의존이 없는 계측용 백엔드 둘 — `NullBackend`, `TraceBackend`.
//!
//! 두 백엔드는 그림을 그리지 않는다. 대신 **계약이 지켜졌는지**를 볼 수 있게
//! 만든다. `TraceBackend` 의 출력은 백엔드 간 정합 시험의 기준선이 된다 —
//! "SVG 와 PDF 가 같은 페이지에서 같은 op 시퀀스를 받았는가"는 그림을 비교하지
//! 않고도 이 문자열로 판정할 수 있다.

use std::collections::BTreeMap;

use crate::paint::PaintOp;

use super::caps::BackendCapabilities;
use super::traits::{PageSize, RenderBackend, RenderBackendError};
use super::util::{paint_op_kind, PageState};

/// `NullBackend` 가 세어 모은 계측값.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DrawStats {
    /// 정상으로 닫힌 페이지 수.
    pub pages: usize,
    /// 그린 op 총수.
    pub ops: usize,
    /// op 종류별 개수. 키는 [`paint_op_kind`] 의 이름이며,
    /// `BTreeMap` 이라 순회 순서가 결정적이다.
    pub per_kind: BTreeMap<&'static str, usize>,
}

impl DrawStats {
    /// 특정 종류의 op 를 몇 번 그렸는가.
    pub fn count_of(&self, kind: &str) -> usize {
        self.per_kind.get(kind).copied().unwrap_or(0)
    }
}

/// 그린 op 를 세기만 하는 계측용 백엔드.
///
/// 쓰임새는 둘이다.
/// 1. 조판 파이프라인이 **무엇을 얼마나** 내보내는지 그리기 비용 없이 재는 것.
/// 2. 새 백엔드를 붙일 때 생명주기 계약이 지켜지는지 먼저 확인하는 것.
#[derive(Debug, Clone, Default)]
pub struct NullBackend {
    state: PageState,
    stats: DrawStats,
}

impl NullBackend {
    /// 빈 계측기를 만든다.
    pub fn new() -> Self {
        Self::default()
    }

    /// 지금까지 모은 계측값을 `finish` 없이 들여다본다.
    pub fn stats(&self) -> &DrawStats {
        &self.stats
    }
}

impl RenderBackend for NullBackend {
    type Output = DrawStats;
    type Error = RenderBackendError;

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            multi_page: true,
            deterministic: true,
            ..BackendCapabilities::none("null")
        }
    }

    fn begin_page(&mut self, size: PageSize) -> Result<(), Self::Error> {
        self.state.begin(size)
    }

    fn draw(&mut self, op: &PaintOp) -> Result<(), Self::Error> {
        self.state.record_draw()?;
        self.stats.ops += 1;
        *self.stats.per_kind.entry(paint_op_kind(op)).or_insert(0) += 1;
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), Self::Error> {
        self.state.end()?;
        self.stats.pages += 1;
        Ok(())
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        self.state.assert_finished()?;
        Ok(self.stats)
    }

    fn finish_boxed(self: Box<Self>) -> Result<Self::Output, Self::Error> {
        (*self).finish()
    }
}

/// op 시퀀스를 결정적 문자열로 기록하는 백엔드.
///
/// # 왜 이게 그림보다 쓸모 있나
///
/// 백엔드 정합을 픽셀로 비교하려면 두 백엔드가 다 완성돼 있어야 하고, 차이가
/// 나도 **어느 단계에서** 갈렸는지 알 수 없다. `TraceBackend` 는 조판이 내보낸
/// op 시퀀스 자체를 고정하므로, 두 백엔드가 다른 그림을 냈을 때
/// "같은 입력을 받았는데 다르게 그린 것"인지 "애초에 다른 입력을 받은 것"인지를
/// 갈라준다.
///
/// # 출력 형식
///
/// ```text
/// begin_page 400.00x300.00
///   textRun bbox=20.00,20.00,120.00,20.00
///   rectangle bbox=0.00,0.00,10.00,10.00
/// end_page ops=2
/// ```
///
/// 좌표는 항상 `{:.2}` 로 찍는다. `f64` 기본 출력의 자릿수 흔들림을 없애
/// 같은 입력이 언제나 같은 바이트열을 내게 하려는 것이다.
#[derive(Debug, Clone, Default)]
pub struct TraceBackend {
    state: PageState,
    lines: Vec<String>,
}

impl TraceBackend {
    /// 빈 기록기를 만든다.
    pub fn new() -> Self {
        Self::default()
    }

    /// 지금까지 기록된 줄들.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// 지금까지 기록된 내용을 `finish` 없이 문자열로 본다.
    pub fn trace(&self) -> String {
        self.lines.join("\n")
    }
}

impl RenderBackend for TraceBackend {
    type Output = String;
    type Error = RenderBackendError;

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            multi_page: true,
            deterministic: true,
            ..BackendCapabilities::none("trace")
        }
    }

    fn begin_page(&mut self, size: PageSize) -> Result<(), Self::Error> {
        self.state.begin(size)?;
        self.lines
            .push(format!("begin_page {:.2}x{:.2}", size.width, size.height));
        Ok(())
    }

    fn draw(&mut self, op: &PaintOp) -> Result<(), Self::Error> {
        self.state.record_draw()?;
        let b = op.bounds();
        self.lines.push(format!(
            "  {} bbox={:.2},{:.2},{:.2},{:.2}",
            paint_op_kind(op),
            b.x,
            b.y,
            b.width,
            b.height
        ));
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), Self::Error> {
        let (_, ops) = self.state.end()?;
        self.lines.push(format!("end_page ops={ops}"));
        Ok(())
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        self.state.assert_finished()?;
        Ok(self.lines.join("\n"))
    }

    fn finish_boxed(self: Box<Self>) -> Result<Self::Output, Self::Error> {
        (*self).finish()
    }
}
