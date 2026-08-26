use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Once};
use subsecond::{HotFn, JumpTable};
use tracing::field::{Field, Visit};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};
use wasm_bindgen::prelude::*;

static REGISTER_PATCH_HANDLER: Once = Once::new();
#[cfg(target_arch = "wasm32")]
static REGISTER_TRACING: Once = Once::new();
const PATCH_COMMIT_EVENT: &str = "rhwp-subsecond-commit";
const HOT_TRACE_TARGET: &str = "rhwp::layout";
const HOT_TRACE_LIMIT: usize = 16;
const HOT_TRACE_SESSION_LIMIT: usize = 4;
const HOT_TRACE_DISABLED_TOKEN: u32 = 1 << 31;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct HotTraceEntry {
    function: String,
    args: BTreeMap<String, Value>,
    duration_ms: f64,
    depth: usize,
    #[serde(skip)]
    sequence: u64,
}

struct HotTraceSession {
    token: u32,
    entries: VecDeque<HotTraceEntry>,
    next_sequence: u64,
}

#[derive(Default)]
struct HotTraceState {
    sessions: Vec<HotTraceSession>,
    active: Vec<u32>,
    next_token: u32,
}

struct HotTraceSpan {
    function: &'static str,
    args: BTreeMap<String, Value>,
    started_at: f64,
    depth: usize,
    sequence: u64,
    token: u32,
}

#[derive(Default)]
struct HotTraceFields(BTreeMap<String, Value>);

impl Visit for HotTraceFields {
    fn record_f64(&mut self, field: &Field, value: f64) {
        if let Some(value) = serde_json::Number::from_f64(value) {
            self.0.insert(field.name().to_owned(), Value::Number(value));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_owned(), Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), Value::from(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), Value::from(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(
            field.name().to_owned(),
            Value::from(value.chars().take(256).collect::<String>()),
        );
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_str(field, &format!("{value:?}"));
    }
}

pub(crate) struct HotTraceLayer;

impl<S> Layer<S> for HotTraceLayer
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        if attrs.metadata().target() != HOT_TRACE_TARGET {
            return;
        }
        let Some((token, sequence)) = HOT_TRACE.with(|state| {
            let mut state = state.borrow_mut();
            let token = *state.active.last()?;
            let session = state
                .sessions
                .iter_mut()
                .find(|session| session.token == token)?;
            let sequence = session.next_sequence;
            session.next_sequence = session.next_sequence.wrapping_add(1);
            Some((session.token, sequence))
        }) else {
            return;
        };
        let mut fields = HotTraceFields::default();
        attrs.record(&mut fields);
        let parent = attrs
            .parent()
            .and_then(|parent| ctx.span(parent))
            .or_else(|| {
                attrs
                    .is_contextual()
                    .then(|| ctx.lookup_current())
                    .flatten()
            });
        let depth = parent
            .as_ref()
            .and_then(|parent| {
                let extensions = parent.extensions();
                extensions
                    .get::<HotTraceSpan>()
                    .filter(|trace| trace.token == token)
                    .map(|trace| trace.depth + 1)
            })
            .unwrap_or(0);
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(HotTraceSpan {
                function: attrs.metadata().name(),
                args: fields.0,
                started_at: trace_now_ms(),
                depth,
                sequence,
                token,
            });
        }
    }

    fn on_record(&self, id: &tracing::Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };
        let mut extensions = span.extensions_mut();
        let Some(trace) = extensions.get_mut::<HotTraceSpan>() else {
            return;
        };
        let mut fields = HotTraceFields(std::mem::take(&mut trace.args));
        values.record(&mut fields);
        trace.args = fields.0;
    }

    fn on_close(&self, id: tracing::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else { return };
        let extensions = span.extensions();
        let Some(trace) = extensions.get::<HotTraceSpan>() else {
            return;
        };
        let entry = HotTraceEntry {
            function: trace.function.to_owned(),
            args: trace.args.clone(),
            duration_ms: (trace_now_ms() - trace.started_at).max(0.0),
            depth: trace.depth,
            sequence: trace.sequence,
        };
        retain_hot_trace(trace.token, entry);
    }
}

fn retain_hot_trace(token: u32, entry: HotTraceEntry) {
    HOT_TRACE.with(|state| {
        let mut state = state.borrow_mut();
        let Some(session) = state
            .sessions
            .iter_mut()
            .find(|session| session.token == token)
        else {
            return;
        };
        retain_trace_entry(&mut session.entries, entry);
    });
}

fn retain_trace_entry(entries: &mut VecDeque<HotTraceEntry>, entry: HotTraceEntry) {
    if let Some(index) = entries
        .iter()
        .position(|existing| existing.function == entry.function)
    {
        entries.remove(index);
    } else if entries.len() == HOT_TRACE_LIMIT {
        entries.pop_front();
    }
    entries.push_back(entry);
}

thread_local! {
    static HOT_TRACE: RefCell<HotTraceState> = RefCell::new(HotTraceState::default());
}

#[derive(Clone)]
pub(crate) struct HotTraceCheckpoint {
    token: u32,
    entries: VecDeque<HotTraceEntry>,
}

pub(crate) fn hot_trace_checkpoint() -> Option<HotTraceCheckpoint> {
    HOT_TRACE.with(|state| {
        let state = state.borrow();
        let token = *state.active.last()?;
        let session = state
            .sessions
            .iter()
            .find(|session| session.token == token)?;
        Some(HotTraceCheckpoint {
            token,
            entries: session.entries.clone(),
        })
    })
}

pub(crate) fn restore_hot_trace(checkpoint: Option<HotTraceCheckpoint>) {
    let Some(checkpoint) = checkpoint else { return };
    HOT_TRACE.with(|state| {
        if let Some(session) = state
            .borrow_mut()
            .sessions
            .iter_mut()
            .find(|session| session.token == checkpoint.token)
        {
            session.entries = checkpoint.entries;
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn trace_now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn trace_now_ms() -> f64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1_000.0
}

pub(crate) fn hot_trace_enabled() -> bool {
    HOT_TRACE.with(|state| {
        let state = state.borrow();
        state
            .active
            .last()
            .is_some_and(|token| state.sessions.iter().any(|session| session.token == *token))
    })
}

#[wasm_bindgen(js_name = beginSubsecondTrace)]
pub fn begin_subsecond_trace() -> u32 {
    HOT_TRACE.with(|state| {
        let mut state = state.borrow_mut();
        state.next_token = state.next_token.wrapping_add(1) & !HOT_TRACE_DISABLED_TOKEN;
        state.next_token = state.next_token.max(1);
        let token = state.next_token;
        if state.sessions.len() == HOT_TRACE_SESSION_LIMIT {
            return token | HOT_TRACE_DISABLED_TOKEN;
        }
        state.sessions.push(HotTraceSession {
            token,
            entries: VecDeque::new(),
            next_sequence: 0,
        });
        token
    })
}

#[wasm_bindgen(js_name = activateSubsecondTrace)]
pub fn activate_subsecond_trace(token: u32) {
    HOT_TRACE.with(|state| {
        let mut state = state.borrow_mut();
        if token & HOT_TRACE_DISABLED_TOKEN != 0
            || state.sessions.iter().any(|session| session.token == token)
        {
            state.active.push(token);
        }
    });
}

#[wasm_bindgen(js_name = deactivateSubsecondTrace)]
pub fn deactivate_subsecond_trace(token: u32) {
    HOT_TRACE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(index) = state.active.iter().rposition(|active| *active == token) {
            state.active.remove(index);
        }
    });
}

#[wasm_bindgen(js_name = endSubsecondTrace)]
pub fn end_subsecond_trace(token: u32, retain: bool) -> String {
    HOT_TRACE.with(|state| {
        let mut state = state.borrow_mut();
        state.active.retain(|active| *active != token);
        let Some(index) = state
            .sessions
            .iter()
            .position(|session| session.token == token)
        else {
            return "[]".to_owned();
        };
        let session = state.sessions.remove(index);
        if !retain || session.entries.is_empty() {
            return "[]".to_owned();
        }
        let mut entries = session.entries.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.sequence);
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_owned())
    })
}

fn register_tracing() {
    #[cfg(target_arch = "wasm32")]
    REGISTER_TRACING.call_once(|| {
        use tracing_subscriber::prelude::*;
        let _ = tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(HotTraceLayer),
        );
    });
}

fn register_patch_handler() {
    register_tracing();
    REGISTER_PATCH_HANDLER.call_once(|| {
        subsecond::register_handler(Arc::new(|| {
            #[cfg(target_arch = "wasm32")]
            if let (Some(window), Ok(event)) =
                (web_sys::window(), web_sys::Event::new(PATCH_COMMIT_EVENT))
            {
                let _ = window.dispatch_event(&event);
            }
        }));
    });
}

fn probe_value() -> u32 {
    41
}

#[wasm_bindgen(js_name = subsecondProbe)]
pub fn subsecond_probe() -> u32 {
    let mut hot = HotFn::current(probe_value);
    hot.call(())
}

/// 데브서버 메시지 한 건을 처리하고 **이 함수가 실제로 관찰한 것**을 담는다.
///
/// 종전 반환값은 `bool` 하나였다. 그 값은 다섯 가지 거절 사유를 전부 `false` 로 접었고,
/// 더 나쁘게는 `true` 가 wasm32 에서 보고할 수 없는 것을 광고했다.
///
/// # wasm32 에서 구조적으로 보고할 수 없는 것
///
/// `subsecond::apply_patch` 는 wasm32 에서 patch wasm 의 fetch/compile/instantiate future 를
/// 띄우고 **즉시** `Ok(())` 를 돌려준다(subsecond 0.7.10 `src/lib.rs:551`, `:690`). 그래서
/// [`Self::PatchDispatched`] 는 "점프 테이블을 subsecond 에 넘겼다"까지만 뜻하며 "패치가
/// 화면에 반영됐다"는 뜻이 **아니다**. future 안의 실패는 전부 `.unwrap()`/`panic!`
/// (`lib.rs:578-582`)이라 이 반환값이 될 수 없다. 그 실패는 두 곳으로만 나간다.
///
/// 1. `crate::init_panic_hook` 이 등록한 `console_error_panic_hook` 의 `console.error`
///    (기본 feature이므로 dx 빌드에도 들어간다).
/// 2. panic 이 abort → wasm trap 이 되어 microtask 경계를 넘는 전역 오류 이벤트.
///
/// 브라우저 쪽에서 2번을 잡아 "패치를 넘긴 뒤 무언가 터졌다"로 잇는 곳은
/// `rhwp-studio/src/core/subsecond-runtime.ts` 다. 어느 쪽도 이 함수의 반환값이 될 수 없으므로
/// 여기서는 **없는 정보를 지어내지 않고** 성공값의 이름으로 그 한계를 드러낸다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolsMessageOutcome {
    /// 텍스트 프레임이 JSON 이 아니다.
    NotJson,
    /// JSON 이지만 `HotReload` 가 아니다. 데브서버의 정상 제어 트래픽이 여기 들어온다.
    NotHotReload,
    /// 다른 빌드를 향한 패치다. 스튜디오는 `build_id=0` 으로만 접속한다.
    ForeignBuildId,
    /// `HotReload` 에 `jump_table` 이 없다.
    MissingJumpTable,
    /// `jump_table` 이 `subsecond::JumpTable` 로 역직렬화되지 않는다.
    UndeserializableJumpTable,
    /// `apply_patch` 가 오류를 돌려줬다. **네이티브에서만 나온다** — 위 문단 참고.
    PatchRejected(String),
    /// 점프 테이블을 `apply_patch` 에 넘겼다. 적용 완료가 아니다.
    PatchDispatched,
}

impl DevtoolsMessageOutcome {
    /// JS 로 넘기는 안정된 식별자.
    ///
    /// [`Self::PatchRejected`] 의 상세 문자열은 싣지 않는다. 그 변형은 wasm32 에서 나올 수
    /// 없어 JS 소비자가 볼 일이 없고, 네이티브 소비자는 이 열거형을 직접 읽는다.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotJson => "not-json",
            Self::NotHotReload => "not-hot-reload",
            Self::ForeignBuildId => "foreign-build-id",
            Self::MissingJumpTable => "missing-jump-table",
            Self::UndeserializableJumpTable => "undeserializable-jump-table",
            Self::PatchRejected(_) => "patch-rejected",
            Self::PatchDispatched => "patch-dispatched",
        }
    }
}

/// 메시지를 판정하고, 패치면 `apply_patch` 에 넘긴다.
pub fn dispatch_devtools_message(message: &str) -> DevtoolsMessageOutcome {
    use DevtoolsMessageOutcome as Outcome;

    let Ok(message) = serde_json::from_str::<Value>(message) else {
        return Outcome::NotJson;
    };
    let Some(hot_reload) = message.get("HotReload") else {
        return Outcome::NotHotReload;
    };
    if hot_reload
        .get("for_build_id")
        .and_then(Value::as_u64)
        .is_some_and(|build_id| build_id != 0)
    {
        return Outcome::ForeignBuildId;
    }
    let Some(jump_table) = hot_reload.get("jump_table") else {
        return Outcome::MissingJumpTable;
    };
    let Ok(jump_table) = serde_json::from_value::<JumpTable>(jump_table.clone()) else {
        return Outcome::UndeserializableJumpTable;
    };
    match unsafe { subsecond::apply_patch(jump_table) } {
        Ok(()) => Outcome::PatchDispatched,
        Err(error) => Outcome::PatchRejected(error.to_string()),
    }
}

#[wasm_bindgen(js_name = applySubsecondDevtoolsMessage)]
pub fn apply_subsecond_devtools_message(message: &str) -> String {
    register_patch_handler();
    dispatch_devtools_message(message).code().to_owned()
}

pub fn link_wasm_exports() {
    register_patch_handler();
    let _ = crate::version();
    let _ = subsecond_probe();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 유효한 `JumpTable` 을 담은 `HotReload` 프레임. `lib` 은 존재하지 않는 경로라
    /// 네이티브의 `apply_patch` 는 dlopen 단계에서 오류를 돌려주고 아무것도 detour 하지 않는다.
    const UNLOADABLE_PATCH: &str = r#"{"HotReload":{"for_build_id":0,"jump_table":{
        "lib":"/nonexistent/rhwp-subsecond-patch.dylib","map":{},
        "aslr_reference":0,"new_base_address":0,"ifunc_count":0}}}"#;

    #[test]
    fn probe_and_layout_trace_contracts_hold() {
        assert_eq!(subsecond_probe(), 41);
        layout_trace_is_capture_gated_and_keeps_only_the_latest_frames();
        layout_frame_trace_contains_true_inputs_and_computed_height();
        interleaved_sessions_attribute_each_sync_segment_to_its_token();
        empty_or_discarded_capture_cannot_return_older_evidence();
    }

    #[test]
    fn every_rejected_message_names_its_own_reason() {
        let reasons = [
            ("not json", "not-json"),
            (r#"{"HotPatchStart":null}"#, "not-hot-reload"),
            (
                r#"{"HotReload":{"for_build_id":7,"jump_table":{}}}"#,
                "foreign-build-id",
            ),
            (r#"{"HotReload":{"for_build_id":0}}"#, "missing-jump-table"),
            (
                r#"{"HotReload":{"jump_table":{"lib":42}}}"#,
                "undeserializable-jump-table",
            ),
        ];
        for (message, expected) in reasons {
            assert_eq!(
                apply_subsecond_devtools_message(message),
                expected,
                "{message}"
            );
        }
    }

    /// 다섯 거절 사유는 서로 구별돼야 한다 — 종전 `bool` 은 전부 `false` 로 접었다.
    #[test]
    fn rejection_reasons_are_distinguishable_from_each_other() {
        let codes: Vec<String> = [
            "not json",
            r#"{"HotPatchStart":null}"#,
            r#"{"HotReload":{"for_build_id":7,"jump_table":{}}}"#,
            r#"{"HotReload":{"for_build_id":0}}"#,
            r#"{"HotReload":{"jump_table":{"lib":42}}}"#,
        ]
        .into_iter()
        .map(apply_subsecond_devtools_message)
        .collect();
        let mut unique = codes.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), codes.len(), "{codes:?}");
    }

    /// 네이티브에서만 도달하는 가지. wasm32 에서는 `apply_patch` 가 오류를 돌려줄 수 없다
    /// ([`DevtoolsMessageOutcome`] 문서 참고).
    #[test]
    fn a_patch_that_cannot_be_loaded_reports_the_loader_error() {
        let DevtoolsMessageOutcome::PatchRejected(error) =
            dispatch_devtools_message(UNLOADABLE_PATCH)
        else {
            panic!("네이티브 apply_patch 는 없는 라이브러리를 거절해야 한다");
        };
        assert!(
            error.contains("/nonexistent/rhwp-subsecond-patch.dylib"),
            "{error}"
        );
        assert_eq!(
            apply_subsecond_devtools_message(UNLOADABLE_PATCH),
            "patch-rejected"
        );
    }

    /// 성공값은 "적용됐다" 가 아니라 "넘겼다" 를 뜻해야 한다.
    #[test]
    fn the_success_code_does_not_claim_the_patch_was_applied() {
        assert_eq!(
            DevtoolsMessageOutcome::PatchDispatched.code(),
            "patch-dispatched"
        );
    }

    fn layout_trace_is_capture_gated_and_keeps_only_the_latest_frames() {
        use tracing_subscriber::prelude::*;

        HOT_TRACE.with(|state| *state.borrow_mut() = HotTraceState::default());
        let mut serialized = String::new();
        let subscriber = tracing_subscriber::registry().with(HotTraceLayer);
        tracing::subscriber::with_default(subscriber, || {
            drop(tracing::trace_span!(
                target: HOT_TRACE_TARGET,
                "ordinary_render",
                page = 0_u64,
            ));
            assert!(!hot_trace_enabled());

            let token = begin_subsecond_trace();
            activate_subsecond_trace(token);
            for para_index in 0_u64..24 {
                let span = tracing::trace_span!(
                    target: HOT_TRACE_TARGET,
                    "flow_advance_height",
                    para_index,
                    result_height = tracing::field::Empty,
                );
                span.record("result_height", para_index as f64 + 0.5);
                drop(span);
            }
            deactivate_subsecond_trace(token);
            serialized = end_subsecond_trace(token, true);
        });

        let trace: Vec<Value> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            trace.len(),
            1,
            "repeated rows keep the latest function frame"
        );
        assert_eq!(trace[0]["function"], "flow_advance_height");
        assert_eq!(trace[0]["args"]["para_index"], 23);
        assert_eq!(trace[0]["args"]["result_height"], 23.5);

        let token = begin_subsecond_trace();
        for index in 0_u64..24 {
            retain_hot_trace(
                token,
                HotTraceEntry {
                    function: format!("layout_{index}"),
                    args: BTreeMap::new(),
                    duration_ms: 0.0,
                    depth: 0,
                    sequence: index,
                },
            );
        }
        let bounded: Vec<Value> = serde_json::from_str(&end_subsecond_trace(token, true)).unwrap();
        assert_eq!(bounded.len(), HOT_TRACE_LIMIT);
        assert_eq!(bounded[0]["function"], "layout_8");
        assert_eq!(bounded[15]["function"], "layout_23");
        nested_capture_cannot_clear_or_disable_its_outer_owner();
        saturated_capture_cannot_write_into_an_existing_owner();
    }

    fn layout_frame_trace_contains_true_inputs_and_computed_height() {
        use crate::model::paragraph::LineSeg;
        use crate::renderer::layout_frame::{FrameRowMetrics, LayoutFrame, RowSegment};
        use tracing_subscriber::prelude::*;

        HOT_TRACE.with(|state| *state.borrow_mut() = HotTraceState::default());
        let mut serialized = String::new();
        let subscriber = tracing_subscriber::registry().with(HotTraceLayer);
        tracing::subscriber::with_default(subscriber, || {
            let token = begin_subsecond_trace();
            activate_subsecond_trace(token);
            let mut frame = LayoutFrame::new(100..500, 40, Vec::new());
            let interval = frame.carve(30)[0].clone();
            assert_eq!(
                frame.commit_carved_row(
                    FrameRowMetrics {
                        vertical_pos: 0,
                        line_height: 30,
                        text_height: 20,
                        baseline_distance: 20,
                        line_spacing: 5,
                    },
                    vec![RowSegment::new(
                        0..4,
                        interval,
                        LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    )],
                ),
                Some(0)
            );
            deactivate_subsecond_trace(token);
            serialized = end_subsecond_trace(token, true);
        });

        let trace: Vec<Value> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(trace[0]["function"], "layout_frame_carve");
        assert_eq!(trace[0]["args"]["top"], 40);
        assert_eq!(trace[0]["args"]["band_height"], 30);
        assert_eq!(trace[0]["args"]["result_interval_count"], 1);
        assert_eq!(trace[1]["function"], "layout_frame_commit_row");
        assert_eq!(trace[1]["args"]["line_height"], 30);
        assert_eq!(trace[1]["args"]["line_spacing"], 5);
        assert_eq!(trace[1]["args"]["result_top"], 75);
        assert_eq!(trace[1]["args"]["result_accepted"], true);
    }

    fn nested_capture_cannot_clear_or_disable_its_outer_owner() {
        use tracing_subscriber::prelude::*;

        HOT_TRACE.with(|state| *state.borrow_mut() = HotTraceState::default());
        let mut outer_serialized = String::new();
        let subscriber = tracing_subscriber::registry().with(HotTraceLayer);
        tracing::subscriber::with_default(subscriber, || {
            let outer = begin_subsecond_trace();
            activate_subsecond_trace(outer);
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "outer_before"));

            let rebuild = begin_subsecond_trace();
            activate_subsecond_trace(rebuild);
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "nested_rebuild"));
            deactivate_subsecond_trace(rebuild);
            let rebuild: Vec<Value> =
                serde_json::from_str(&end_subsecond_trace(rebuild, true)).unwrap();
            assert_eq!(rebuild[0]["function"], "nested_rebuild");

            assert!(hot_trace_enabled(), "the outer owner remains active");
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "outer_after"));
            deactivate_subsecond_trace(outer);
            outer_serialized = end_subsecond_trace(outer, true);
        });

        let outer: Vec<Value> = serde_json::from_str(&outer_serialized).unwrap();
        assert_eq!(outer[0]["function"], "outer_before");
        assert_eq!(outer[1]["function"], "outer_after");
    }

    fn saturated_capture_cannot_write_into_an_existing_owner() {
        use tracing_subscriber::prelude::*;

        HOT_TRACE.with(|state| *state.borrow_mut() = HotTraceState::default());
        let mut owner_serialized = String::new();
        let subscriber = tracing_subscriber::registry().with(HotTraceLayer);
        tracing::subscriber::with_default(subscriber, || {
            let owners = (0..HOT_TRACE_SESSION_LIMIT)
                .map(|_| begin_subsecond_trace())
                .collect::<Vec<_>>();
            activate_subsecond_trace(owners[0]);
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "owner_before"));
            let disabled = (0..100)
                .map(|_| begin_subsecond_trace())
                .collect::<Vec<_>>();
            HOT_TRACE
                .with(|state| assert_eq!(state.borrow().sessions.len(), HOT_TRACE_SESSION_LIMIT));
            let saturated = *disabled.last().unwrap();
            assert_ne!(saturated & HOT_TRACE_DISABLED_TOKEN, 0);
            activate_subsecond_trace(saturated);
            assert!(
                !hot_trace_enabled(),
                "the disabled top owner blocks attribution"
            );
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "must_not_leak"));
            deactivate_subsecond_trace(saturated);
            assert!(hot_trace_enabled());
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "owner_after"));
            deactivate_subsecond_trace(owners[0]);
            for token in disabled {
                assert_eq!(end_subsecond_trace(token, true), "[]");
            }
            for (index, token) in owners.into_iter().enumerate() {
                let serialized = end_subsecond_trace(token, true);
                if index == 0 {
                    owner_serialized = serialized;
                } else {
                    assert_eq!(serialized, "[]");
                }
            }
            HOT_TRACE.with(|state| {
                let state = state.borrow();
                assert!(state.sessions.is_empty());
                assert!(state.active.is_empty());
            });
        });

        let owner: Vec<Value> = serde_json::from_str(&owner_serialized).unwrap();
        assert_eq!(owner[0]["function"], "owner_before");
        assert_eq!(owner[1]["function"], "owner_after");
    }

    fn interleaved_sessions_attribute_each_sync_segment_to_its_token() {
        use tracing_subscriber::prelude::*;

        HOT_TRACE.with(|state| *state.borrow_mut() = HotTraceState::default());
        let mut a_serialized = String::new();
        let mut b_serialized = String::new();
        let subscriber = tracing_subscriber::registry().with(HotTraceLayer);
        tracing::subscriber::with_default(subscriber, || {
            let a = begin_subsecond_trace();
            let b = begin_subsecond_trace();
            activate_subsecond_trace(a);
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "a_after_await"));
            deactivate_subsecond_trace(a);
            activate_subsecond_trace(b);
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "b_after_await"));
            deactivate_subsecond_trace(b);
            a_serialized = end_subsecond_trace(a, true);
            b_serialized = end_subsecond_trace(b, true);
        });

        let a: Vec<Value> = serde_json::from_str(&a_serialized).unwrap();
        let b: Vec<Value> = serde_json::from_str(&b_serialized).unwrap();
        assert_eq!(a[0]["function"], "a_after_await");
        assert_eq!(b[0]["function"], "b_after_await");
    }

    fn empty_or_discarded_capture_cannot_return_older_evidence() {
        use tracing_subscriber::prelude::*;

        HOT_TRACE.with(|state| *state.borrow_mut() = HotTraceState::default());
        let subscriber = tracing_subscriber::registry().with(HotTraceLayer);
        tracing::subscriber::with_default(subscriber, || {
            let old = begin_subsecond_trace();
            activate_subsecond_trace(old);
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "old_page"));
            deactivate_subsecond_trace(old);

            let empty = begin_subsecond_trace();
            activate_subsecond_trace(empty);
            deactivate_subsecond_trace(empty);
            assert_eq!(end_subsecond_trace(empty, true), "[]");

            let discarded = begin_subsecond_trace();
            activate_subsecond_trace(discarded);
            drop(tracing::trace_span!(target: HOT_TRACE_TARGET, "aborted_page"));
            deactivate_subsecond_trace(discarded);
            assert_eq!(end_subsecond_trace(discarded, false), "[]");

            let old: Vec<Value> = serde_json::from_str(&end_subsecond_trace(old, true)).unwrap();
            assert_eq!(old[0]["function"], "old_page");
        });
    }
}
