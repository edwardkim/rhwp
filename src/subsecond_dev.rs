use serde_json::Value;
use subsecond::{HotFn, HotFunction, JumpTable};
use wasm_bindgen::prelude::*;

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
    dispatch_devtools_message(message).code().to_owned()
}

pub fn link_wasm_exports() {
    let _ = crate::version();
    let _ = subsecond_probe();
}

pub(crate) fn hot_fn_ptr<Args, Marker, Function>(function: Function) -> u64
where
    Function: HotFunction<Args, Marker>,
{
    HotFn::current(function).ptr_address().0
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
    fn probe_calls_the_current_function_body() {
        assert_eq!(subsecond_probe(), 41);
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
}
