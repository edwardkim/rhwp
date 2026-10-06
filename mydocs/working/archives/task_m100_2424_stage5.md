# Task M100 #2424 Stage E 완료보고서 — 최신 기준 전체 회귀와 브라우저 수용 검증

## 1. 결론

`upstream/devel@cbddc1cd8` 기준으로 #2424의 resumable pagination 구현과 수용 검증을 완료했다.
거대 표 셀의 56번째 입력 경계에서 기존 약 1.06~1.11초 동기 full pagination은 더 이상 호출되지
않는다. Studio는 약 32~34ms의 begin 뒤 115개 fragment를 macrotask 사이에서 budget 1로 처리하고,
마지막 step에서만 새 115쪽 pagination을 게시한다.

최종 rebase에 포함된 upstream 3커밋은 #2428 종료 검증 문서뿐이며 제품 코드 변경은 없었다.
#2428은 각주 hit-test fast-reject 작업이므로 #2424의 continuation 수명·정확성 계약과도 독립적이다.

## 2. 브라우저 E2E 계약

기존 #2214 E2E는 flow 경계에서 동기 `flushDeferredPagination()` 1회를 요구했다. #2424 이후의 올바른
계약은 입력 직후 pending 상태를 유지하고, 비동기 begin/step이 완료된 뒤 정확한 pagination을
원자 commit하는 것이다. 하네스를 다음과 같이 갱신했다.

- begin/step/cancel/flush 호출과 status, revision, fragment 수를 trace한다.
- 입력 직후와 2-rAF 시점에는 pending을 확인하고, 완료 판정은 고정 지연이 아니라 상태 polling으로 한다.
- 경계 입력에서 동기 flush 0회, begin 1회, budget-1 step 115회, 최종 `complete`를 단언한다.
- page 0 model/tree/layout/cursor/caret와 PNG crop이 pending 전후 동일한 편집 결과를 유지하는지 확인한다.
- IME/iOS raw input도 stable 입력과 flow-boundary 입력을 각각 검증한다.

HWP/HWPX 각 3회 결과는 다음과 같다.

| 형식 | run | 경계 operation | begin | stable operation p95 | step | 동기 flush |
|---|---:|---:|---:|---:|---:|---:|
| HWP | 1 | 77.0ms | 32.0ms | 46.4ms | 115 | 0 |
| HWP | 2 | 78.9ms | 33.6ms | 46.1ms | 115 | 0 |
| HWP | 3 | 81.3ms | 34.0ms | 46.8ms | 115 | 0 |
| HWPX | 1 | 76.3ms | 32.2ms | 46.3ms | 115 | 0 |
| HWPX | 2 | 75.9ms | 32.1ms | 45.7ms | 115 | 0 |
| HWPX | 3 | 76.8ms | 32.4ms | 46.0ms | 115 | 0 |

IME/iOS stable smoke는 begin/step/flush가 모두 0회였고 pending descriptor만 유지했다. 같은 두 raw
경로의 56번째 flow-boundary smoke는 begin 1회, step 115회, 동기 flush 0회로 완료됐다.

## 3. 전체 검증

| 명령 또는 matrix | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --lib --target wasm32-unknown-unknown` | 통과 |
| `cargo test --profile release-test --lib` | 2533 passed, 0 failed, 7 ignored |
| #2424 focused library tests | 5 passed |
| #2214 library oracle | 9 passed |
| `tests/issue_2214_page_local_repaint.rs` | 3 passed |
| `tests/issue_2724_passthrough_invalidation_guard.rs` | 5 passed |
| `wasm-pack build --target web --out-dir pkg --no-opt` | 통과 |
| Studio `npm test` | 509 passed, 0 failed |
| Studio `npm run build` | 통과 |
| `npm run e2e:issue-2214` | HWP/HWPX 3회와 IME/iOS 8개 smoke 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |

production build의 CanvasKit `fs`/`path` externalize와 500kB chunk 경고는 기존 경고이며 실패가 아니다.
E2E PNG와 JSON은 ignored `output/poc/task2214/stage4`에 생성했고 소스 변경에는 포함하지 않았다.

PR #3125의 첫 CI에서는 새 `pub fn(&mut self)` 네 개가 #2724 패스스루 무효화 가드에 미분류되어
default-feature shard 8이 실패했다. 네 API는 편집 IR이 아니라 shadow pagination job, 측정·pagination
cache와 직렬화 비대상 `Table::dirty`만 변경하므로 `Exempt::SessionState`에 근거와 함께 등재했다.
실제 셀 편집의 `section.raw_stream` 무효화는 선행 text-editing 뮤테이터가 계속 담당한다.

## 4. 수용 판정

- correctness: HWP/HWPX 모두 115쪽, 115 fragments, 113 changed cuts와 인접 cut 연속성을 유지한다.
- atomicity: pending 114 steps 동안 공개 pagination은 바뀌지 않고 마지막 step에서만 교체된다.
- responsiveness: 약 1.1초 동기 typeset을 p95 약 11ms인 fragment step으로 분리해 task 사이에 yield한다.
- input contract: 실제 브라우저 경계 입력은 동기 flush 0회이며 약 76~81ms에 반환한다.
- lifetime: 새 입력 stale 교체, cancel, save/save-as/print/manual 동기 barrier가 모두 고정됐다.

첫 fast path는 한 구역·한 문단·단일 열·마지막 non-TAC RowBreak 표로 보수적으로 제한한다. 지원하지
않는 문서는 기존 full pagination으로 fallback한다. begin의 약 32ms 정규화·선택 측정 prelude와 fast-path
범위 확대는 별도 후속 최적화 후보이며 #2424 완료를 막지 않는다.
