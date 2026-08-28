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
//! 안전 관계: studio 의 피크 동시 참조는 예산 `SNAPSHOT_ID_BUDGET`(= W−2)에
//! 순간 저장 1 을 더한 **W−1** 이다 — #5769 이후 execute 의 before 와 undo
//! 시점의 after 는 서로 다른 시점에 저장되고 각 저장 사이에 예산 강제가 돈다
//! (saveSnapshot 호출부는 command.ts 두 곳뿐). store 상한이 피크 밑이면 참조 중
//! 스냅샷이 무통보 축출된다(#2328). 가드는 미러 관습(양쪽 동치 유지)대로
//! `MAX_SNAPSHOTS >= WASM_MAX_SNAPSHOTS` 를 요구한다 — 최소 충분(W−1)보다
//! 한 슬롯 엄격하며, 여유 1 을 계약으로 고정하는 선택이다.
//! (부기: 축출 루프의 length>1 바닥 때문에 W<6 미소값에서는 예산 자체가 안
//! 지켜질 수 있으나 현행 100 에서는 무관하다.)
//!
//! 파싱은 줄 단위 구조 매칭 — 주석 줄(`//`·`*` 시작)은 trim 후 선언 접두사로
//! 시작할 수 없어 구조적으로 탈락한다(주석 인용의 첫-매치 오염 방지, 리뷰 P1-1).

use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} 읽기 실패: {e}"))
}

/// 선언 줄(들여쓰기 허용)에서만 값을 읽는다 — 주석 속 인용은 매치되지 않는다.
fn parse_decl(source: &str, rel: &str, prefix: &str) -> usize {
    let rest = source
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(prefix))
        .unwrap_or_else(|| {
            panic!("{rel}: 선언 줄(`{prefix}…`)이 없다 — 선언 형태가 바뀌었으면 이 가드를 함께 갱신하라")
        });
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("{rel}: `{prefix}` 값 파싱 실패"))
}

#[test]
fn rust_store_cap_covers_studio_peak_references() {
    let rust = read("src/document_core/commands/document.rs");
    let studio = read("rhwp-studio/src/engine/history.ts");

    let max_snapshots = parse_decl(
        &rust,
        "src/document_core/commands/document.rs",
        "const MAX_SNAPSHOTS: usize = ",
    );
    let wasm_max = parse_decl(
        &studio,
        "rhwp-studio/src/engine/history.ts",
        "const WASM_MAX_SNAPSHOTS = ",
    );

    assert!(
        max_snapshots >= wasm_max,
        "document.rs MAX_SNAPSHOTS({max_snapshots}) < studio WASM_MAX_SNAPSHOTS({wasm_max}) — \
         studio 피크 동시 참조는 {wasm_max}-1(예산 + 순간 저장 1)이고 미러 계약은 동치 이상을 \
         요구한다. 상한이 피크 밑이면 참조 중 스냅샷이 무통보 축출된다. \
         양쪽 상수를 함께 갱신하라 (#2328, #6332)"
    );
}
