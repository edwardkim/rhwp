# Task M100 #2424 Stage F 완료보고서 — 삭제·한글 IME 보정과 출력 barrier 검증

## 1. 리뷰 판정과 범위

PR #3125의 첫 resumable 경로는 `insertTextInCellDeferredPagination`에만 연결돼 있었다.
일반 문자는 빨라졌지만 Backspace/Delete는 `deleteTextInCell()`의 동기 full pagination을 호출했고,
한글 IME는 조합 갱신마다 이전 조합문자를 삭제하므로 두 번째 자모부터 같은 동기 경로를 반복했다.
전달받은 리뷰의 원인 분석과 약 917ms 프리즈 관찰은 코드와 올바른 PR WASM 실측 모두에서 사실이었다.

이번 보정은 text-only인 flat/depth-1 셀 삭제와 실제 다단계 한글 IME를 기존
deferred/resumable 파이프라인에 포함한다. Enter는 문단 분할이라는 구조 변경이므로 이번 fast path에
섞지 않고 후속 범위로 남긴다. 중첩 표, 본문, 구조 명령과 undo는 기존 동기 pagination을 유지한다.

## 2. 구현

- Rust에 `delete_text_in_cell_native_deferred_pagination`과 WASM
  `deleteTextInCellDeferredPagination`을 추가했다.
- 삭제 전후 셀 문단의 상대 flow advance를 비교하고, render-normalized 복사본과 최신
  `DeferredPaginationDescriptor`를 insert와 같은 방식으로 갱신한다.
- Studio의 `DeleteTextCommand`와 raw IME 삭제가 공통 typed mutation helper를 사용한다.
  IME의 `delete + insert` effect는 한 accumulator에서 OR 누적한 뒤 cursor 조회 전에 한 번 소비한다.
- 삭제로 table nested-text flag가 `true→false`가 될 수 있으므로 owner table의 직접 cell cache와
  flag만 폐기해 다음 접근에서 한 번 재계산한다. unrelated table cache는 보존한다.
- 저장·인쇄 barrier가 flush 실패 뒤 pending 상태를 확인하고, 미완료면 직렬화·페이지 렌더를
  시작하지 않도록 fail-closed했다.

## 3. B — 최종 페이지 수와 레이아웃 원자성

실제 RowBreak 표가 편집 전 1쪽, 마지막 셀 줄 증가 뒤 2쪽이 되는 합성 문서를 추가했다.
`fragment_budget=1`로 shadow job을 진행하는 동안 모든 pending step의 공개 page count는 1로
유지됐고, 마지막 complete에서만 2로 교체됐다.

실문서 HWP/HWPX 115쪽 fixture의 기존 exact oracle도 유지한다. pending 중 공개 cut chain은
바뀌지 않고 마지막 commit 뒤 113개 downstream cut이 full-pagination oracle과 일치하며,
모든 continuation의 `end_cut == next.start_cut` 연속성을 확인한다. 브라우저 smoke는 입력 뒤
caret과 대상 cell path/offset을 보존하는지도 확인한다.

반대 방향도 별도 검증했다. 56자로 5줄 경계를 만든 뒤 마지막 문자를 deferred 삭제해 4줄로
되돌리는 HWP/HWPX 테스트에서, 115개 pending step 동안 공개 cut chain은 그대로였고 마지막
commit 결과만 55자 full-pagination oracle과 정확히 일치했다.

## 4. C — 실제 저장·인쇄 barrier

HWP/HWPX에서 56번째 flow-boundary 입력 직후 pending job이 도는 상태로 실제 파일 메뉴의
`file:save`를 실행했다.

- trace 순서가 `flushDeferredPagination → exportHwp/exportHwpx`임을 확인했다.
- 저장 blob을 다시 열어 최신 56자 입력과 최종 115쪽을 확인했다.
- HWP 저장본은 229,376 bytes, HWPX 저장본은 225,694 bytes였다.

같은 pending 상태에서 실제 `file:print`를 실행했다.

- `flushDeferredPagination → 첫 renderPageSvg` 순서를 확인했다.
- flush 뒤 115개 SVG와 인쇄용 `.page` 115개가 생성됐다.
- barrier 이후 pending 상태는 false였다.

## 5. 실브라우저 삭제·IME 결과

새 PR WASM을 `wasm-pack build --target web --out-dir pkg --no-opt`로 만들고 macOS headless
Chrome에서 HWP/HWPX를 각각 검증했다.

| 형식 | Backspace WASM | Delete WASM | IME `ㅎ→하→한` input handler | 동기 delete/flush |
|---|---:|---:|---:|---:|
| HWP | 2.5ms | 1.5ms | 75.3 / 71.1 / 71.6ms | 0 / 0 |
| HWPX | 1.5ms | 1.5ms | 74.0 / 71.3 / 72.8ms | 0 / 0 |

IME trace는 deferred insert 3회, deferred delete 2회였고 flat immediate delete와
`deleteTextInCellByPath`는 0회였다. 최종 model text는 `한`, cursor offset은 최신 입력 위치,
공개 page count는 115로 유지됐다.

## 6. 검증 자산

- Rust page-count atomicity, deferred delete/IME revision, cache `true→false` 단위 테스트
- Studio command/effect, runner atomic publish, output barrier, mutation registry 회귀
- HWP/HWPX 실브라우저 Backspace/Delete/다단계 IME/save, HWP print
- 브라우저 JSON 결과: ignored `output/poc/task2214/stage4/focused-summary.json`

## 7. 전체 게이트

| 명령 또는 matrix | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --lib --target wasm32-unknown-unknown` | 통과 |
| `cargo test --release --lib` | 2537 passed, 0 failed, 7 ignored |
| `tests/issue_2214_page_local_repaint.rs` | 3 passed |
| `tests/issue_2724_passthrough_invalidation_guard.rs` | 5 passed |
| `wasm-pack build --target web --out-dir pkg --no-opt` | 통과 |
| Studio `npm test` | 511 passed, 0 failed |
| Studio `npm run build` | 통과 |
| `issue-2214-page-local-repaint --review-only` | HWP/HWPX deletion·IME·save, HWP print 통과 |
| `git diff --check` | 통과 |

`npm run e2e:manifest-check`는 이번 변경 이전 HEAD부터 존재한
`embed-save-ack.test.mjs`, `issue-2809-split-alignment.test.mjs` 2개가 manifest에 등재되지 않아
실패한다. 이번 작업은 새 E2E 파일을 만들지 않고 기존 #2214 파일을 확장했으므로 이 선행 누락은
제품·브라우저 수용 결과와 분리해 기록한다.
