# Task #7280 Stage 11 — R2i 문단 넘침 허용 경로 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2h](task_m100_7280_stage10.md), 시작 head `e88aca799`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2i 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 비범위

일반 전체 fit 실패 뒤의 기존 두 넘침 허용 경로만 옮긴다. `paragraph/overflow.rs`는
인라인 그림/도형의 atomic 수용과 쪽 경계 전 tail 후보를 읽기 전용으로 판정한다.
원본 Paragraph, FormattedParagraph와 작은 OverflowPage 값 관측만 받으며 state를 쓰지 않는다.
두 판정 사이 상태 변경이 없고 atomic 성공 시 즉시 반환하므로 같은 관측값을 공유할 수 있다.
관측의 base_available_height는 순수 layout 조회다. 진단 출력이 있는 available_height는
snapshot에 넣지 않고 기존처럼 tail 후보 통과 뒤에만 조정자가 호출한다.

`paragraph::try_place_overflow_paragraph`는 atomic → tail 순서와 성공 시 반환을 조정한다.
state는 atomic 항목 추가 → 기존 메트릭 계산 → 높이/underrun/저장 하단 반영 순서를 지킨다.
tail은 항목 추가 → trim 0 → total_height 전진 → 저장 하단 반영이며 underrun은 변경하지 않는다.
atomic은 반대로 기존 trim을 덮지 않는다. 같은 FullParagraph라는 이유로 일반 전체 배치
command와 합치지 않는다.

기존 60px 문턱, 폰트 drift/inkless 조건, 추론/명시적 쪽 경계 구별과 단락·컨트롤 조건을
변경하지 않는다. 주석의 기존 한컴 동작 설명을 이번 리팩토링의 독립적 사실 확인이나 승인으로
격상하지 않는다. 근거가 약한 heuristic의 수정은 별도 작업이다.
빈 구성 결과, 일반 fit, split, 표/셀 continuation, 실제 layout/paint 및 IR/public API는 비범위다.
테스트 assertion/모듈 ID와 baseline·golden·ignore를 유지한다.

## 2. 검증 계획과 범위

`output/7280/stage11/verify-overflow.mjs`는 두 Query, snapshot 매핑, 경로별 상태 쓰기,
조건부 available_height 호출·메트릭 계산·반환 순서를 원본과 복원 비교한다.
부수효과 없는 base 높이/필드 읽기는 Query 진입 snapshot에서 관측한다.
`height_for_fit`의 수용 판단과 `flow_advance_height`/`total_height`의 실제 전진을 합치지 않는다.
후속 PageItem의 layout/paint 해석은 그대로이며 분할 컷·이월 자체는 변경하지 않는다.

review worktree에서 suite 준비 → fmt·고정 baseline 정책 검사 → native Clippy → typeset/composer,
#6854의 HWP/HWPX 빈 쪽 방지, #6793 저장 쪽 경계 등 기존 관련 계약을 순차 실행한다.
이 계약은 영향 경로의 무회귀 증거이며 모든 atomic/폰트 drift 조건 조합의 직접 커버리지나
한컴 피델리티 승인으로 보고하지 않는다. 실행 범위와 미검증을 최종 결과에 구분한다.
전체 회귀·최종 WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM/직접 시각
대조는 통합 게이트에 남는다. 원격 push·PR·댓글은 수행하지 않는다.

## 3. 결과와 후속

제품 SHA: `3ee74450b0fc078dd0bcb5c2779c91db739c5f55`.
review worktree: `/home/edward/mygithub/rhwp-review-7280-r2i`.
고정 target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.

- 정적 복원 비교 통과: `output/7280/stage11/extraction-proof.json`.
- manifest: 1,382 sources / 5,965 static test attrs / 48 integration targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support items 28 통과. 두 정책 검사는 고정 baseline 대비 실행했다.
- `cargo fmt --all -- --check` 통과.
- native Clippy: `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review
  -- -D warnings`, exit 0, 55.75초.
- 로그: `output/7280/stage11/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

집중 실행 명령(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_005 --test regression_suite_022 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6854_empty_para_orphan_page::) | test(issue_6793_tac_host_page_tail_padding::)' \
  --no-fail-fast
```

#6854의 4개 기존 검사는 HWP/HWPX의 쪽수와 본문 없는 고립 쪽의 부재를 확인한다.
이는 표 외곽·모든 줄의 위치 검증은 아니다. #6793의 3개 기존 검사는 차례의 2쪽 귀속,
2쪽 본문 상단 좌표와 표지의 용지 밖 글자 부재를 확인한다. 기본 typeset/composer 계약도 포함한다.
atomic의 60px 경계와 Shape TopAndBottom 제외, 글자 있는 폰트 drift의 모든 분기를 각각
새 직접 계약으로 검증한 것은 아니다. 해당 조건의 복원 비교와 관련 계약 통과를 구분한다.

집중 결과: **155건 통과 / 실패 0건**, 필터 비선택 4,348건, 6 binaries, exit 0.
빌드 4분 51초, 테스트 0.394초. 실행 ID: `4210f5d0-64d8-4d46-8c15-d45fc5e41d00`.
로그: `output/7280/stage11/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행의 같은
필터에서 선택한 155개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과
별개이며 이번에 전체 회귀를 재실행한 것은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile의 미사용 설정 경고는 기준 실행과 같다.
CI 도구 버전까지 동일하다고 주장하지 않는다. review worktree의 tracked 변경은 없고,
제품 검증 이후에는 계획/완료 기록만 갱신했다. 파생 suite·manifest는 커밋하지 않았다.

문단 진입 fit와 빈 구성 결과, 표 문단/컨트롤 흐름, 나머지 state 직접 쓰기와 최종 통합 검증은
후속에 남는다.
