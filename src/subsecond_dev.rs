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

#[wasm_bindgen(js_name = applySubsecondDevtoolsMessage)]
pub fn apply_subsecond_devtools_message(message: &str) -> bool {
    let Ok(message) = serde_json::from_str::<Value>(message) else {
        return false;
    };
    let Some(hot_reload) = message.get("HotReload") else {
        return false;
    };
    if hot_reload
        .get("for_build_id")
        .and_then(Value::as_u64)
        .is_some_and(|build_id| build_id != 0)
    {
        return false;
    }
    let Some(jump_table) = hot_reload.get("jump_table") else {
        return false;
    };
    let Ok(jump_table) = serde_json::from_value::<JumpTable>(jump_table.clone()) else {
        return false;
    };
    unsafe { subsecond::apply_patch(jump_table).is_ok() }
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

    #[test]
    fn probe_calls_the_current_function_body() {
        assert_eq!(subsecond_probe(), 41);
    }

    #[test]
    fn ignores_non_patch_devserver_messages() {
        assert!(!apply_subsecond_devtools_message(
            r#"{"HotPatchStart":null}"#
        ));
        assert!(!apply_subsecond_devtools_message("not json"));
    }
}
