# Task #7280 Stage 6 — R2d 문단 fit 보정의 조회·소비 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2c](task_m100_7280_stage5.md), 시작 head `c3fb3c41f`.
- 상태: R2d 구현·집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.
- 고정 제품 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.

## 1. 범위와 기존 실행 순서

문단의 전체 fit/줄 분할에 앞서 수행하는 예산 보정만 분리한다. 조판 규칙·상수·조건·연산 순서와
기존 예외는 그대로 보존한다. 본문/표 문단 분할 알고리즘, IR, public API, paint는 변경하지 않는다.
기존 heuristic의 근거를 이번 리팩토링으로 새로 승인하는 것이 아니다.

`paragraph/fit.rs`는 문서의 drift 안전여백, float 배제 영역 probe 높이, 저장 꼬리 fit 여유를
읽기 전용으로 계산한다. TypesetState/TypesetEngine이나 가변 flag를 받지 않는다.
`state.rs`의 의미 있는 소비 메서드는 안전여백 면제, 각주 안전여백 반환, 저장 꼬리 증거를 각각
한 번 소비한다. 읽기 전용 계산을 하는 것처럼 보이던 가변 flag 접근을 호출 이름에 드러낸다.
범용 setter나 트랜잭션, 새로운 거대 context/result는 만들지 않는다.

상위 조정자 `typeset_paragraph`의 순서는 다음과 같이 유지한다.

1. 기존 strict plain-text flag 소비 및 drift 안전여백 조회
2. 직전 PartialTable 확인 → 안전여백 면제 소비
3. float probe 계산 → `apply_visible_float_exclusions`
4. 각주 안전여백 반환 소비 → 저장 꼬리 증거 소비/fit 여유 계산
5. `(available_height - safety + footnote_margin_addback + tail_overflow).max(0)`
6. 기존 빈 문단·강제 경계·전체 fit·줄 분할 경로로 진행

float 배제 영역 적용은 높이를 바꿀 수 있으므로 2와 4를 합치거나 앞당기지 않는다.
strict flag가 참일 때 면제/반환/꼬리 증거를 철회하는 순서도 그대로다.
저장 꼬리는 기존대로 `.take()` 이후 `base_available_height`와 현재 각주 높이를 읽는다.

## 2. 소비자와 남은 책임

`available`은 전체 fit와 초기 잔여 예산에 사용한다. 이후 줄 분할 루프는 기존대로
`base_available_height`, 각주/zone과 `layout_drift_safety_px`를 다시 반영한다.
이를 하나의 영구 예산으로 치환하지 않는다. 이번 절편에서 값과 생산 시점이 같으므로
PageItem/줄 컷·후속 높이 변경·배치 소비 경로는 모두 기존 구현에 둔다.

`fit.rs`가 공유하는 parent helper/상수는 명시적 import로 남겼다:
`section_has_zero_high_attr_rowbreak_table`, `saved_bounds_overlap_current_flow`,
`BODY_BOTTOM_SEAT_PX`, `SAVED_FRAME_FLOW_DRIFT_TOLERANCE_PX`.
다른 소비자가 있는 helper의 최종 소유권과 state의 다른 직접 쓰기는 후속 정리 대상이다.
strict flag의 기존 helper/테스트 위치, flag를 설정하는 구역 순회는 변경하지 않았다.
전체 상태 캡슐화나 paragraph/table paragraph 분해가 끝난 것으로 보고하지 않는다.

## 3. 검증

`output/7280/stage6/verify-fit.mjs`는 분리된 5개 block을 원래 호출 위치로 복원하고,
입력 인자 매핑·조건·상수·주석·실행 순서 및 그 외 parent 코드를 비교한다.
저장 꼬리 계산 helper 본문도 동일성을 확인한다. formatter 공백/후행 쉼표만 정규화한다.
정적 보존 검사는 직접 시각 판정이나 한컴 피델리티 증거를 대체하지 않는다.

기존 typeset/composer/float 계약과 저장 꼬리·문단 간격 관련 `issue_5941_tail_overflow_drift_gate`,
`issue_6031`, `issue_6753`의 기존 계약을 집중 실행한다. source 테스트·integration source·
baseline/golden/ignore·CI 정책은 수정하지 않는다. 고정 SHA review worktree에서 파생 suite를
준비하고 fmt·정책·native Clippy 후 집중 테스트를 순차 실행한다.

이번 결과를 R2 전체 완료 또는 제출 준비 완료로 보고하지 않는다. 전체 회귀, 최종
native/WASM/workspace lint 묶음, Native Skia, fresh Docker WASM/직접 시각 검증은
별도 통합 게이트에 남는다. 이전 절편의 실행 결과를 이번 제품 SHA의 PASS로 재사용하지 않는다.
원격 push·PR·댓글은 수행하지 않는다.

### 고정 SHA와 실행 기록

- 제품 SHA: `5cb89418ccf76ed617fee368596fec7d9e7ec332`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2d`.
- 정적 보존 비교: `output/7280/stage6/extraction-proof.json` 통과. 이전 inline state 메서드도 동일하다.
- manifest 고정 baseline 검사 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- unit-tier 고정 baseline 검사 통과: 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support items 28.
- `cargo fmt --all -- --check` 통과.
- `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings`
  통과(exit 0, 56.00초).
- 로그: `output/7280/stage6/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

집중 검사 명령:

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --lib --test regression_suite_011 --test regression_suite_012 --test regression_suite_016 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(renderer::float_placement::) | test(issue_5941_tail_overflow_drift_gate::) | test(issue_6031_ladder_sb_omitted_tail_overrun::) | test(issue_6753_lazy_base_keeps_trimmed_spacing_before::)' \
  --no-fail-fast
```

suite 번호는 review worktree에서 `--prepare`한 실제 원본 매핑을 조회해 선택했다.
`issue_5941` 계약의 페이지 수 assertion은 보존 여부의 한 축이며 시각 정확성을 뜻하지 않는다.
`issue_6031`은 SVG 본문 하단 baseline 및 다음 페이지 첫 글줄,
`issue_6753`은 render tree 하단과 다음 페이지의 `비용` 문구를 확인한다.
기존 typeset의 `issue2439_strict_following_plain_text_fit_is_consumed_once`도 실행 대상이다.
3개 상태 보정의 모든 flag 조합을 새로 실행 검증했다고 주장하지 않는다.

집중 테스트 결과: **165건 통과 / 실패 0건**, 필터 비선택 4,533건, 7 binaries, exit 0.
빌드 4분 53초, 테스트 0.583초. 실행 ID: `ed5c3266-b192-4a1e-8f53-89ff00c14458`.
로그: `output/7280/stage6/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline의 전수 결과에서
동일 필터로 선택한 165개 PASS 이름이 일치함을 확인했다. 이번 필터 비선택 4,533건을
기존 ignore 50건과 혼동하지 않는다. 전체 회귀 재실행 결과는 아니다.

nextest 0.9.137(권장 0.9.140), 미사용 observation profile의 `junit.report-skipped` 경고는
기준 실행과 동일하다. 현재 CI 도구 버전까지 동일한 검증으로 주장하지 않는다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 stage하지 않았다.
검증 후 제품 코드 변경 없이 완료 기록만 갱신했다.

## 4. 다음 절편

문단의 전체 fit 판정/줄 분할과 표 문단 흐름 조정의 책임 분리를 이어간다.
기존 state helper를 다시 광범위한 가변 context로 감싸지 않고, 실제 호출 순서와
확정 항목·커서 소유권을 확인한 뒤 분리한다. 최종 전체 검증과 시각 증거는 통합 게이트다.
