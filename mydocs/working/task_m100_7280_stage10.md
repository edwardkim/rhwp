# Task #7280 Stage 10 — R2h 전체 문단 배치 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2g](task_m100_7280_stage9.md), 시작 head `158574ba0`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2h 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

진입 fit을 통과한 일반 전체 문단 배치만 `paragraph::place_fitted_paragraph`로 옮긴다.
`placement::defer_preceding_float`는 기존 항목 slice와 IR을 읽어 앞 float의 순서 지연을
판정한다. `state::insert_fitted_paragraph`가 기존 pop/push 순서를 적용한다.
이후 조정자가 advance → 기존 진단 → trimmed spacing 계산을 수행하고,
`state::apply_fitted_paragraph_flow`가 trim → 높이 → underrun → 저장 본문 하단을 반영한다.

항목 삽입을 메트릭 계산 뒤로 옮기지 않는다. 두 메트릭 조회의 단락/다단, stored layout,
trim 복원, 다음 경계, lazy base 조건과 단락마다 관측하는 순서를 보존한다.
기존 FormattedParagraph 계산 결과를 사용하며 fit 높이와 실제 전진량을 합치지 않는다.
PageItem의 실제 layout/paint 소비 경로와 IR/public API는 변경하지 않는다.

fit의 조건, atomic 넘침 허용, inkless tail, 빈 구성 결과 및 분할 문단은 이번 이동 대상이
아니다. 이들은 trim/underrun 갱신 차이가 있으므로 새 command로 일괄 치환하지 않는다.
기존 heuristic을 정당한 조판 규칙으로 승인하거나 버그를 수정하는 절편이 아니다.
테스트 assertion/모듈 ID, baseline, golden, ignore를 변경하지 않는다.

## 2. 검증 계획

`output/7280/stage10/verify-full-placement.mjs`로 순서 Query, 항목 삽입, 흐름 메트릭,
진단과 상태 쓰기를 원래 block으로 복원 비교한다. 남은 root와 분할 조정자의 불변도 검사한다.
공백/후행 쉼표를 제외한 조건·연산 차이를 허용하지 않는다.

고정 review worktree에서 파생 suite 준비, fmt, baseline 대비 manifest/unit-tier 정책,
native Clippy를 순차 수행한 뒤 typeset/composer와 float host/spacing 관련 기존 계약을 실행한다.
검증은 native 집중 계약이며 전체 회귀나 모든 배치 분기의 직접 시각 검증을 대신하지 않는다.
최종 전체 회귀, WASM/workspace lint, workspace build, Native Skia 및 fresh Docker WASM/
시각 대조는 통합 게이트에 남는다. 원격 push·PR·댓글은 수행하지 않는다.

## 3. 결과와 다음 절편

제품 SHA: `ce3b30c1247cf814129644961ee4d995ebe060fa`.
review worktree: `/home/edward/mygithub/rhwp-review-7280-r2h`.
고정 target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.

- 정적 복원 비교 통과: `output/7280/stage10/extraction-proof.json`.
- manifest: 1,382 sources / 5,965 static test attrs / 48 integration targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support items 28 통과. 두 정책 검사는 고정 baseline 대비 실행했다.
- `cargo fmt --all -- --check` 통과.
- native Clippy: `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review
  -- -D warnings`, exit 0, 55.52초.
- 로그: `output/7280/stage10/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

집중 실행 명령(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_003 --test regression_suite_011 --test regression_suite_012 \
  --test regression_suite_014 --test regression_suite_015 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_5870_empty_host_float_flow_advance::) | test(issue_6133_host_line_above_offset_float::) | test(issue_6147_empty_anchor_band_host_line::) | test(issue_6753_lazy_base_keeps_trimmed_spacing_before::) | test(issue_6031_ladder_sb_omitted_tail_overrun::)' \
  --no-fail-fast
```

#5870은 본표 상단과 결재란의 간격을 SVG에서, #6133/#6147은 host와 offset float/anchor band의
실제 render tree 위치를 확인한다. #6753은 trimmed spacing 복원과 본문 하단/다음 쪽 첫 줄을,
#6031은 저장 줄 사다리의 하단 넘침을 검사한다. 기존 typeset/composer 계약도 포함한다.
이들 관련 계약이 defer Query의 모든 조건 조합을 직접 커버한다는 뜻은 아니다.
조판 규칙의 정확성 승인 대신 이번 구조 이동 전후의 보존 여부를 검증한다.

집중 결과: **156건 통과 / 실패 0건**, 필터 비선택 4,954건, 9 binaries, exit 0.
빌드 5분 7초, 테스트 0.256초. 실행 ID: `4749c516-43cf-43f0-ace0-121c3fffd2cb`.
로그: `output/7280/stage10/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행의 같은
필터에서 선택한 156개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과
별개이며 이번에 전체 회귀를 재실행한 것은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile의 미사용 설정 경고는 기준 실행과 같다.
CI 도구 버전까지 동일하다고 주장하지 않는다. review worktree의 tracked 변경은 없고,
제품 검증 이후에는 계획/완료 기록만 갱신했다. 파생 suite·manifest는 커밋하지 않았다.

남은 진입 fit·특수 배치, 표 문단/컨트롤 흐름, 나머지 state 직접 쓰기와 규칙/기여자 안내를
계속 분리해야 한다.
