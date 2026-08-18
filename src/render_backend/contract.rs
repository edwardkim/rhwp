//! 생명주기·오류·결정론 계약 검사기.
//!
//! 백엔드마다 같은 스크립트를 돌려 같은 오류를 같은 자리에서 내는지 본다.
//! 구현은 `PageState` 와 같은 판정을 재사용하고, 어댑터는 이 스크립트를
//! 통과해야 계약 준수로 본다.

use super::traits::{PageSize, RenderBackend, RenderBackendError};

/// 생명주기 스크립트 한 걸음.
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleStep {
    /// `begin_page(width, height)`.
    Begin { width: f64, height: f64 },
    /// `draw` — 호출만 하고 op 내용은 호출자가 넣는다.
    Draw,
    /// `end_page`.
    End,
    /// `finish`. 스크립트 마지막에만 쓴다.
    Finish,
}

/// 한 걸음의 기대 결과.
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleExpect {
    /// 성공.
    Ok,
    /// 이 오류여야 한다.
    Err(RenderBackendError),
}

/// 스크립트 한 행.
#[derive(Debug, Clone, PartialEq)]
pub struct LifecycleRow {
    /// 걸음.
    pub step: LifecycleStep,
    /// 기대.
    pub expect: LifecycleExpect,
}

/// 이름 있는 스크립트.
#[derive(Debug, Clone, PartialEq)]
pub struct LifecycleScript {
    /// 안정 식별자.
    pub id: &'static str,
    /// 한 줄 설명.
    pub summary: &'static str,
    /// 걸음들.
    pub rows: &'static [LifecycleRow],
}

/// 페이지 치수 유효성 사례.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSizeCase {
    /// 폭.
    pub width: f64,
    /// 높이.
    pub height: f64,
    /// 유효한가.
    pub valid: bool,
    /// 설명.
    pub reason: &'static str,
}

/// 계약이 거부해야 하는 치수와 받아들여야 하는 치수.
pub const PAGE_SIZE_CASES: &[PageSizeCase] = &[
    PageSizeCase {
        width: 1.0,
        height: 1.0,
        valid: true,
        reason: "최소 양수",
    },
    PageSizeCase {
        width: 400.0,
        height: 300.0,
        valid: true,
        reason: "일반 화면",
    },
    PageSizeCase {
        width: 595.28,
        height: 841.89,
        valid: true,
        reason: "A4 px 근사",
    },
    PageSizeCase {
        width: 0.0,
        height: 300.0,
        valid: false,
        reason: "폭 0",
    },
    PageSizeCase {
        width: 300.0,
        height: 0.0,
        valid: false,
        reason: "높이 0",
    },
    PageSizeCase {
        width: -1.0,
        height: 10.0,
        valid: false,
        reason: "음수 폭",
    },
    PageSizeCase {
        width: 10.0,
        height: -1.0,
        valid: false,
        reason: "음수 높이",
    },
    PageSizeCase {
        width: f64::NAN,
        height: 10.0,
        valid: false,
        reason: "NaN 폭",
    },
    PageSizeCase {
        width: 10.0,
        height: f64::NAN,
        valid: false,
        reason: "NaN 높이",
    },
    PageSizeCase {
        width: f64::INFINITY,
        height: 10.0,
        valid: false,
        reason: "무한 폭",
    },
    PageSizeCase {
        width: 10.0,
        height: f64::NEG_INFINITY,
        valid: false,
        reason: "음의 무한 높이",
    },
];

/// `PageSize::is_valid` 가 사례표와 같은지 본다.
pub fn page_size_cases_hold() -> Result<(), String> {
    for case in PAGE_SIZE_CASES {
        let size = PageSize::new(case.width, case.height);
        if size.is_valid() != case.valid {
            return Err(format!(
                "{}: is_valid={} 기대={}",
                case.reason,
                size.is_valid(),
                case.valid
            ));
        }
    }
    Ok(())
}

/// 오류 Display 가 비어 있지 않고 핵심 토큰을 담는가.
pub fn error_display_tokens(err: &RenderBackendError) -> Vec<&'static str> {
    match err {
        RenderBackendError::NoOpenPage { .. } => vec!["begin_page", "열린 페이지"],
        RenderBackendError::PageAlreadyOpen => vec!["end_page", "이미 열린"],
        RenderBackendError::UnclosedPage { .. } => vec!["end_page", "닫지 않은"],
        RenderBackendError::InvalidPageSize { .. } => vec!["치수", "px"],
        RenderBackendError::UnsupportedOp { .. } => vec!["지원하지 않는"],
        RenderBackendError::MultiplePagesUnsupported { .. } => vec!["여러 페이지"],
        RenderBackendError::Backend(_) => vec!["백엔드 오류"],
    }
}

/// Display 가 토큰을 모두 담는가.
pub fn error_display_holds(err: &RenderBackendError) -> Result<(), String> {
    let text = err.to_string();
    if text.is_empty() {
        return Err("오류 Display 가 비었다".into());
    }
    for token in error_display_tokens(err) {
        if !text.contains(token) {
            return Err(format!("`{text}` 에 `{token}` 없음"));
        }
    }
    Ok(())
}

/// 표준 생명주기 스크립트.
pub fn standard_lifecycle_scripts() -> &'static [LifecycleScript] {
    const DRAW_WITHOUT_BEGIN: &[LifecycleRow] = &[LifecycleRow {
        step: LifecycleStep::Draw,
        expect: LifecycleExpect::Err(RenderBackendError::NoOpenPage { call: "draw" }),
    }];
    const END_WITHOUT_BEGIN: &[LifecycleRow] = &[LifecycleRow {
        step: LifecycleStep::End,
        expect: LifecycleExpect::Err(RenderBackendError::NoOpenPage { call: "end_page" }),
    }];
    const DOUBLE_BEGIN: &[LifecycleRow] = &[
        LifecycleRow {
            step: LifecycleStep::Begin {
                width: 100.0,
                height: 100.0,
            },
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::Begin {
                width: 100.0,
                height: 100.0,
            },
            expect: LifecycleExpect::Err(RenderBackendError::PageAlreadyOpen),
        },
    ];
    const FINISH_OPEN: &[LifecycleRow] = &[
        LifecycleRow {
            step: LifecycleStep::Begin {
                width: 40.0,
                height: 30.0,
            },
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::Finish,
            expect: LifecycleExpect::Err(RenderBackendError::UnclosedPage { pages_completed: 0 }),
        },
    ];
    const EMPTY_PAGE: &[LifecycleRow] = &[
        LifecycleRow {
            step: LifecycleStep::Begin {
                width: 40.0,
                height: 30.0,
            },
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::End,
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::Finish,
            expect: LifecycleExpect::Ok,
        },
    ];
    const ONE_DRAW: &[LifecycleRow] = &[
        LifecycleRow {
            step: LifecycleStep::Begin {
                width: 40.0,
                height: 30.0,
            },
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::Draw,
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::End,
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::Finish,
            expect: LifecycleExpect::Ok,
        },
    ];
    const INVALID_THEN_VALID: &[LifecycleRow] = &[
        LifecycleRow {
            step: LifecycleStep::Begin {
                width: 0.0,
                height: 30.0,
            },
            expect: LifecycleExpect::Err(RenderBackendError::InvalidPageSize {
                width: 0.0,
                height: 30.0,
            }),
        },
        LifecycleRow {
            step: LifecycleStep::Begin {
                width: 40.0,
                height: 30.0,
            },
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::End,
            expect: LifecycleExpect::Ok,
        },
        LifecycleRow {
            step: LifecycleStep::Finish,
            expect: LifecycleExpect::Ok,
        },
    ];

    &[
        LifecycleScript {
            id: "draw-without-begin",
            summary: "begin_page 없이 draw 하면 NoOpenPage",
            rows: DRAW_WITHOUT_BEGIN,
        },
        LifecycleScript {
            id: "end-without-begin",
            summary: "begin_page 없이 end_page 하면 NoOpenPage",
            rows: END_WITHOUT_BEGIN,
        },
        LifecycleScript {
            id: "double-begin",
            summary: "열린 페이지에 begin_page 는 PageAlreadyOpen",
            rows: DOUBLE_BEGIN,
        },
        LifecycleScript {
            id: "finish-while-open",
            summary: "안 닫고 finish 하면 UnclosedPage",
            rows: FINISH_OPEN,
        },
        LifecycleScript {
            id: "empty-page",
            summary: "빈 페이지도 정상 생명주기",
            rows: EMPTY_PAGE,
        },
        LifecycleScript {
            id: "one-draw",
            summary: "op 하나 그리고 닫기",
            rows: ONE_DRAW,
        },
        LifecycleScript {
            id: "invalid-then-valid",
            summary: "실패한 begin_page 는 페이지를 열지 않는다",
            rows: INVALID_THEN_VALID,
        },
    ]
}

/// 스크립트를 백엔드에 적용한다. `draw` 는 `draw_op` 를 쓴다.
pub fn run_lifecycle<B, F>(
    backend: &mut B,
    script: &LifecycleScript,
    mut draw_op: F,
) -> Result<(), String>
where
    B: RenderBackend<Error = RenderBackendError>,
    F: FnMut(&mut B) -> Result<(), RenderBackendError>,
{
    for (index, row) in script.rows.iter().enumerate() {
        let result = match &row.step {
            LifecycleStep::Begin { width, height } => {
                backend.begin_page(PageSize::new(*width, *height))
            }
            LifecycleStep::Draw => draw_op(backend),
            LifecycleStep::End => backend.end_page(),
            LifecycleStep::Finish => {
                // finish 는 self 소비라 여기서는 호출하지 않는다.
                // 호출자가 마지막에 따로 검사한다.
                Ok(())
            }
        };
        match (&row.expect, result) {
            (LifecycleExpect::Ok, Ok(())) => {}
            (LifecycleExpect::Err(want), Err(got)) if want == &got => {}
            (expect, got) => {
                return Err(format!(
                    "{} 행 {index}: 기대 {expect:?} 실제 {got:?}",
                    script.id
                ));
            }
        }
    }
    Ok(())
}

/// `finish` 가 스크립트 기대와 같은지 본다.
pub fn finish_matches<B>(backend: B, script: &LifecycleScript) -> Result<B::Output, String>
where
    B: RenderBackend<Error = RenderBackendError>,
{
    let last = script.rows.last();
    match last {
        Some(row) if matches!(row.step, LifecycleStep::Finish) => match &row.expect {
            LifecycleExpect::Ok => backend
                .finish()
                .map_err(|err| format!("{} finish 실패: {err}", script.id)),
            LifecycleExpect::Err(want) => match backend.finish() {
                Err(got) if &got == want => Err("__expected_finish_error__".into()),
                Err(got) => Err(format!(
                    "{} finish 오류 불일치: {got:?} != {want:?}",
                    script.id
                )),
                Ok(_) => Err(format!("{} finish 가 성공하면 안 된다", script.id)),
            },
        },
        _ => backend
            .finish()
            .map_err(|err| format!("{} 끝 finish 실패: {err}", script.id)),
    }
}

/// 결정론 — 같은 입력을 두 번 돌리면 같은 산출물.
pub fn outputs_equal<T: PartialEq>(left: &T, right: &T) -> bool {
    left == right
}
