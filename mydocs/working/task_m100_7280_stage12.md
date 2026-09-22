# Task #7280 Stage 12 — R2j 빈 구성 결과·줄 분할 진입 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2i](task_m100_7280_stage11.md), 시작 head `46764c8a1`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2j 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 비범위

일반 fit/넘침 허용 실패 뒤의 잔여 흐름을 `paragraph::place_after_failed_fit`로 옮긴다.
구성된 줄 수 0인 결과는 기존 FullParagraph 항목 추가 후 advance와 trimmed spacing을 계산하고,
state의 `apply_full_paragraph_flow`로 기존 순서대로 반영한다. 이 command는 R2h의
`apply_fitted_paragraph_flow`를 의미에 맞게 이름만 바꾼 것이며 본문은 그대로다.
빈 문자열 문단 전체를 줄 수 0과 같은 것으로 취급하지 않는다. 실제 빈 줄은 기존 구성 결과에 따른다.
일반 전체 배치와 달리 lazy-base 인자를 계속 false로 전달하며 두 경로를 합치지 않는다.

줄이 있으면 `split_entry::inspect_entry`가 첫 줄 요구 높이, 잔여 예산, 저장 reset과 호환성
재수용 근거를 계산한다. 필요 시 state에 빈 문단 spill 소유를 기록한 **뒤** 별도
`should_advance`가 저장 첫 줄 하단과 이월 조건을 검사한다. 그 뒤에만 쪽/단 전환과 기존
`place_split_paragraph`를 호출한다. flag 기록을 나중으로 옮기거나 쪽 전환 후 진입 예산을
다시 계산하지 않는다.

두 Query는 원본/서식/구성 결과와 제한된 읽기 전용 페이지 관측만 받는다. 관측은 항목 slice를
빌리며 결과에는 상태 참조를 남기지 않는다. 각 호출 전에 다시 관측하므로 상태 변경을 가로질러
borrow를 유지하지 않는다. 관측의 base 높이는 부수효과 없는 layout 조회다.

저장 reset 문턱, drift 마진, TAC 이미지 스택 판정, 기존 호환성 정책과 flag 수명을 유지한다.
fit과 실제 advance, 컷과 요구 높이를 합치지 않는다. 입력 포맷·IR·PageItem 의미와 최종
layout/paint, 표/셀 continuation, 테스트 assertion/ID, baseline·golden·ignore는 변경하지 않는다.
기존 heuristic의 타당성 승인이나 조판 버그 수정이 아니다.

## 2. 검증 계획

`output/7280/stage12/verify-entry.mjs`로 Query와 command를 원래 block으로 복원 비교한다.
Query 입출력, 빈 구성 결과의 메트릭/상태 쓰기, spill → 첫 줄 하단 판단 → 쪽 전환 →
split 호출 순서와 기존 나머지 조정자/parent의 불변을 확인한다.

review worktree에서 파생 suite 준비 → fmt·고정 baseline 정책 → native Clippy → 집중 nextest를
순차 실행한다. typeset/composer와 #6568 첫 줄 하단/다음 쪽 첫 줄, #5755 되감김 문단 이월,
#5921 수용 가능한 near-top reset 및 #6753 trim 복원 계약을 포함한다.
기존 계약의 보호 범위와 조건 조합의 직접 커버리지를 구분한다. 줄 수 0의 특수 경로와
Hangul 2024 spill 모든 조합을 새 직접 테스트로 검증했다고 주장하지 않는다.

전체 회귀·최종 WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM/직접 시각
대조는 통합 게이트에 남는다. 원격 push·PR·댓글은 수행하지 않는다.

## 3. 결과와 후속

제품 SHA: `d4e145811eb39bb40e60512d84bac31252d283bc`.
review worktree: `/home/edward/mygithub/rhwp-review-7280-r2j`.
고정 target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.

- 정적 복원 비교 통과: `output/7280/stage12/extraction-proof.json`.
- manifest: 1,382 sources / 5,965 static test attrs / 48 integration targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support items 28 통과. 두 정책 검사는 고정 baseline 대비 실행했다.
- `cargo fmt --all -- --check` 통과.
- native Clippy: `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review
  -- -D warnings`, exit 0, 55.31초.
- 로그: `output/7280/stage12/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

집중 실행 명령(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_011 --test regression_suite_012 --test regression_suite_013 \
  --test regression_suite_022 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6568_para_start_first_line_fit::) | test(issue_5755_rewind_overflow_page_break::) | test(issue_5921_neartop_reset_fits::) | test(issue_6753_lazy_base_keeps_trimmed_spacing_before::)' \
  --no-fail-fast
```

집중 결과: **154건 통과 / 실패 0건**, 필터 비선택 4,765건, 8 binaries, exit 0.
빌드 5분 6초, 테스트 0.220초. 실행 ID: `ce3d83ba-3646-4cba-aceb-2629fa4152a8`.
로그: `output/7280/stage12/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행의 같은
필터에서 선택한 154개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과
별개이며 이번에 전체 회귀를 재실행한 것은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile의 미사용 설정 경고는 기준 실행과 같다.
CI 도구 버전까지 동일하다고 주장하지 않는다. review worktree의 tracked 변경은 없고,
제품 검증 이후에는 계획/완료 기록만 갱신했다. 파생 suite·manifest는 커밋하지 않았다.

일반 문단의 앞단 fit/예산·저장 경계 조정, 표 문단/컨트롤 흐름, 나머지 state 직접 쓰기와
최종 통합 검증은 후속에 남는다.
