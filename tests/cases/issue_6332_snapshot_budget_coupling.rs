#![cfg(not(target_arch = "wasm32"))]

//! [#6332] 스냅샷 예산 상수의 Rust–studio 결합 가드 (rust 레인 절반).
//!
//! document_core 의 store 축출 상한 `MAX_SNAPSHOTS`(document.rs)와 studio 의
//! `WASM_MAX_SNAPSHOTS`(history.ts)는 주석으로만 결합돼 있었다 — 순 Rust 변경은
//! frontend 두 레인이 모두 skip 되므로(undo depth 게이트 #5769 포함) 상수를
//! 낮추고 studio 갱신을 잊어도 CI 가 그린이었다. 이 테스트가 rust 레인에서
//! 결합을 기계 검증한다. studio 방향 절반은
//! rhwp-studio/tests/command-history-snapshot.test.ts 의 소스 가드가 담당한다.
//!
//! 안전 관계: studio 는 예산 `SNAPSHOT_ID_BUDGET = WASM_MAX_SNAPSHOTS - 2` 개의
//! live id 에 순간 저장 +2(execute 의 before, undo 시점의 after)를 더해 최대
//! `WASM_MAX_SNAPSHOTS` 개를 동시 참조한다. store 상한이 그보다 작으면 참조 중
//! 스냅샷이 무통보 축출돼 undo 예외가 재발한다(#2328).
//! 따라서 `MAX_SNAPSHOTS >= WASM_MAX_SNAPSHOTS`.

use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} 읽기 실패: {e}"))
}

fn parse_const(source: &str, pattern: &str, label: &str) -> usize {
    let start = source.find(pattern).unwrap_or_else(|| {
        panic!("{label} 선언(`{pattern}`)이 없다 — 선언 형태가 바뀌었으면 이 가드를 함께 갱신하라")
    });
    let digits: String = source[start + pattern.len()..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("{label} 값 파싱 실패"))
}

#[test]
fn rust_store_cap_covers_studio_peak_references() {
    let rust = read("src/document_core/commands/document.rs");
    let studio = read("rhwp-studio/src/engine/history.ts");

    let max_snapshots = parse_const(&rust, "const MAX_SNAPSHOTS: usize = ", "MAX_SNAPSHOTS");
    let wasm_max = parse_const(&studio, "const WASM_MAX_SNAPSHOTS = ", "WASM_MAX_SNAPSHOTS");

    assert!(
        max_snapshots >= wasm_max,
        "document.rs MAX_SNAPSHOTS({max_snapshots}) < studio 피크 참조 수({wasm_max} = \
         SNAPSHOT_ID_BUDGET + 순간 2) — studio 가 참조 중인 스냅샷이 무통보 축출된다. \
         양쪽 상수를 함께 갱신하라 (#2328, #6332)"
    );
}
