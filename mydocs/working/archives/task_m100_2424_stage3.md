# Task M100 #2424 Stage C 완료보고서 — resumable continuation cursor/context

## 1. 목적

거대 표의 115-fragment continuation loop를 호출자가 제한된 fragment 수만큼 실행하고 중간 상태를
보존할 수 있는 synchronous resumable 형태로 분리한다. 이 단계에서는 Native caller가 끝까지 동기 drain해
공개 API 동작을 바꾸지 않는다.

## 2. 구현

`TableContinuationCursor`가 다음 진행 상태를 소유한다.

- 현재 row와 cell-unit `start_cut`
- rowspan block cut 여부
- continuation 여부
- 방출한 fragment 수

`BlockTableContinuationPreparedState`는 row/cell spacing, 가용 높이, `LayoutEngine`, rowspan touched,
cut row heights, caption/footnote/host spacing 입력을 소유한다. `BlockTableContinuationContext`는 prepared
state와 cursor, fragment budget, step 수, shadow `TypesetState`를 함께 보존한다.

각 iteration은 `Skipped`, `Emitted`, `Complete` 중 하나를 반환한다. `step`은 `Emitted` 수가 budget에
도달하면 caller에게 반환하고, 다음 호출은 같은 cursor와 page-flow state에서 재개한다. paragraph/table/
measured-table/styles borrow는 context 밖의 `BlockTableContinuationSource`에서 매 step 전달하므로 다음
단계에서 descriptor 좌표 기반 재조회로 교체할 수 있다.

## 3. exact-cut 검증

`RHWP_2424_FRAGMENT_BUDGET`을 1, 8, 미설정으로 바꿔 같은 #2214 HWP/HWPX oracle을 실행했다.

| budget | fragments | steps | 결과 |
|---:|---:|---:|---|
| 1 | 115 | 115 | HWP/HWPX 115쪽, 113 changed cuts, continuity/tree/cursor exact |
| 8 | 115 | 15 | 동일 |
| 무제한 | 115 | 1 | 동일 |

각 설정에서 모든 adjacent fragment는 `end_cut == next.start_cut` 또는 row boundary 연속성을 유지하고,
첫 fragment/마지막 fragment와 `is_continuation`/`is_block_split` 계약도 기존 oracle을 통과했다.

## 4. 회귀 검증

| 명령 | 결과 |
|---|---|
| `cargo test --profile release-test --lib issue2424_ -- --nocapture` | 3 passed |
| budget 1/8/무제한 #2214 exact-cut focused test | 각 1 passed |
| `cargo test --profile release-test --lib issue2214 -- --nocapture` | 9 passed |
| `cargo test --profile release-test --test issue_2214_page_local_repaint -- --nocapture` | 3 passed |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| `cargo fmt --all -- --check` / `git diff --check` | 통과 |

## 5. 다음 단계

1. descriptor 좌표로 source와 measured table을 매 step 재조회하는 pending job을 `DocumentCore`에 둔다.
2. begin/step/finish와 stale/unsupported/error full fallback을 구현한다.
3. 공개 pagination은 완료 전까지 교체하지 않고 shadow result를 revision 재검증 뒤 원자 commit한다.
