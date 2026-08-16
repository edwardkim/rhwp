//! Issue #2087: `DocumentCore` 는 native 소비자가 `Mutex<DocumentCore>` 로 보관해
//! 스레드 경계 너머에서 소유할 수 있어야 한다. 내부 캐시에 `Rc` 같은 `!Send`
//! 타입이 다시 들어오면 이 테스트가 컴파일 단계에서 실패한다.

use rhwp::DocumentCore;

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

#[test]
fn document_core_remains_send_for_native_consumers() {
    assert_send::<DocumentCore>();
    assert_sync::<std::sync::Mutex<DocumentCore>>();
}
