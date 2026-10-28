//! [#7105 증상 ②] OLE 개체를 편집하려 하면 "수식이 아닙니다"만 나와 왜 안 되는지 알 수 없다.
//!
//! 한/글 5.x·97 계열이 `hwpeq5X.ocx` 로 저장한 수식은 native `$eqed` 컨트롤이 아니라
//! **OLE 개체**(`Control::Shape(ShapeObject::Ole)`)다. rhwp 는 그 `Contents` 스트림의
//! 스크립트를 읽어 렌더하지만(#7105 증상 ①) 원래 OLE 바이트에 편집분을 되쓰는 경로는 없다.
//! 현행 문법을 그대로 쓰면 한/글이 그 개체를 못 읽는다(한/글은 레거시 `\CMD … \TAB`
//! 방언을 기대한다). UI는 `promote_ole_equation_native`로 native 수식에 안전하게 전환한 뒤
//! 편집하고, 이 시험은 그 전환을 건너뛴 직접 API 호출의 진단 계약을 고정한다.
//!
//! 신고 원본에서 직접 API를 부르면 동작은 이렇게 갈린다 —
//! `실험4 Transistor-MOSFET.hwp` 실측:
//!
//! | 동작 | 코어 실측 |
//! | --- | --- |
//! | `delete-control` · `delete-shape` | **된다** (OLE 22 → 21, 쪽수 11 유지) |
//! | `set-equation-properties` · `delete-equation` | 안 된다 — 대상이 `Control::Equation` 이 아니다 |
//!
//! 삭제가 코어에서 되므로 UI 에서 안 되면 원인은 선택(hit-test) 계층이고, 신고자가 쓴
//! 뷰어 UI 는 이 저장소에 없다. 여기서는 오류가 **왜** 나는지와 **대신 무엇을 쓰면 되는지**를
//! 말하게 한다.
//!
//! 재현체는 이미 있는 `samples/한셀OLE.hwpx` 를 쓴다 — 검사 대상은 "OLE 개체"라는 갈래이지
//! 그 안에 든 것이 수식인지가 아니다. 이 문서의 `(구역0, 문단0)` 은 컨트롤 셋을 갖고
//! `ctrl 2` 만 OLE 라, 같은 문단 안에서 대상과 반례를 함께 잡을 수 있다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/한셀OLE.hwpx";
/// `(구역0, 문단0)` 의 OLE 도형. `rhwp info` 가 `도형 [구역0:문단0]: OLE` 로 보고한다.
const OLE_CTRL: usize = 2;
/// 같은 문단의 비-OLE 컨트롤(구역/단 정의) — 반례.
const NON_OLE_CTRL: usize = 0;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("open OLE fixture")
}

fn equation_edit_error(core: &mut DocumentCore, ctrl: usize) -> String {
    core.set_equation_properties_native(0, 0, ctrl, None, None, "{\"script\":\"a+b\"}")
        .expect_err("수식이 아닌 컨트롤이므로 실패해야 한다")
        .to_string()
}

#[test]
fn issue_7105_ole_object_edit_says_why_it_cannot_be_edited() {
    let mut core = core();
    let msg = equation_edit_error(&mut core, OLE_CTRL);

    assert!(
        msg.contains("OLE"),
        "#7105: 대상이 OLE 개체임을 말해야 한다 — 수정 전 메시지는 \
         `지정된 컨트롤이 수식이 아닙니다` 뿐이라 원인을 알 수 없었다: {msg}"
    );
    assert!(
        msg.contains("delete-control") || msg.contains("delete-shape"),
        "#7105: 대신 쓸 수 있는 경로(도형 삭제)를 알려야 한다: {msg}"
    );
}

#[test]
fn issue_7105_non_ole_controls_keep_the_previous_message() {
    // 반례: 새 안내가 아무 컨트롤에나 붙으면 진단이 오히려 흐려진다.
    let mut core = core();
    let msg = equation_edit_error(&mut core, NON_OLE_CTRL);
    assert!(
        msg.contains("수식이 아닙니다") && !msg.contains("OLE"),
        "#7105: OLE 이 아닌 컨트롤은 종전 메시지를 유지한다: {msg}"
    );
}

#[test]
fn issue_7105_deleting_the_ole_object_still_works() {
    // 신고 증상 ② 중 **삭제**는 코어에서 된다 — 이 계약이 깨지면 위 안내가 거짓말이 된다.
    let mut core = core();
    core.delete_control_native(0, 0, OLE_CTRL)
        .expect("#7105: OLE 개체 삭제는 도형 경로로 되어야 한다");

    // 정확히 그 컨트롤이 사라졌다 — 같은 인덱스가 더는 OLE 로 잡히지 않는다.
    let after = core
        .set_equation_properties_native(0, 0, OLE_CTRL, None, None, "{\"script\":\"a+b\"}")
        .expect_err("삭제 뒤에는 그 자리에 OLE 이 없다")
        .to_string();
    assert!(
        !after.contains("OLE"),
        "#7105: 삭제 후 같은 인덱스는 더 이상 OLE 개체가 아니어야 한다: {after}"
    );
}
