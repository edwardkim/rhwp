# Task M100 #2424 Stage B 완료보고서 — descriptor 수명과 stale guard

## 1. 목적

browser task 사이에 pagination continuation을 보존하기 전에, 어떤 deferred edit의 작업인지 식별하고
도중에 새 입력이나 구조 변경이 발생했을 때 오래된 shadow result를 폐기할 계약을 고정한다.

## 2. 구현

`DeferredPaginationDescriptor`는 다음 정보를 보존한다.

- 단조 증가 edit revision
- section / host paragraph / table control / cell / cell paragraph 좌표
- 기존 pagination에서 target table의 첫 global page
- 같은 target에서 이미 관측한 `cellFlowChanged` 누적값
- table structure fingerprint

structure fingerprint는 고정 FNV-1a 조합으로 row/column/span, 셀과 셀 문단 수, 각 문단의 control 종류,
중첩 table 구조를 묶는다. text 자체는 제외해 정상적인 text-only deferred input이 같은 구조로 유지된다.

`deferred_pagination_target_status`는 continuation step 전에 현재 model을 다시 조회해 네 상태를 구분한다.

- `Current`: 최신 revision과 target/구조가 일치한다.
- `Superseded`: 더 최신 deferred edit가 descriptor를 교체했다.
- `TargetMissing`: section부터 cell paragraph까지 target 좌표가 사라졌다.
- `StructureChanged`: target은 존재하지만 table 또는 cell paragraph control 구조가 달라졌다.

`Current` 외 상태는 이후 pending job에서 shadow state를 버리고 기존 full pagination으로 fallback하는 신호다.
현재 단계에서는 기존 동기 flush 동작을 유지하며, 성공한 full pagination이 descriptor를 소비한다.

## 3. 검증

focused test에서 최초 `Current`, 새 edit 뒤 이전 revision `Superseded`, cell paragraph 제거
`TargetMissing`, row count와 paragraph control 변경 `StructureChanged`, 복구 뒤 `Current`, 다른 cell target
교체, full flush 뒤 descriptor 소비를 확인했다.

| 명령 | 결과 |
|---|---|
| `cargo test --profile release-test --lib issue2424_deferred_pagination_descriptor_tracks_latest_edit_until_flush -- --nocapture` | 1 passed |
| `cargo test --profile release-test --lib issue2214 -- --nocapture` | 9 passed; HWP/HWPX 115 fragments·113 changed cuts 유지 |
| `cargo test --profile release-test --test issue_2214_page_local_repaint -- --nocapture` | 3 passed |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

## 4. 다음 단계

1. 최신 `typeset_block_table` continuation loop의 row/cut 진행 상태를 owned cursor로 분리한다.
2. 한 fragment iteration을 `Skipped`/`Emitted`/`Complete` caller-controlled step으로 만든다.
3. budget 1/8/무제한 native driver와 one-shot oracle의 모든 row/cut/continuation/block flag를 exact 비교한다.
