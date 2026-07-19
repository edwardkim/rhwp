# Task M100 #2193 Stage 2 작업보고서 — 반복 Native 입력·pagination 기준선

## 0. 판정 요약

- **Stage 판정**: 완료
- **production 변경**: 없음
- **신규 자산**: `tests/issue_2193_input_pagination_perf.rs` ignored diagnostic
- **측정 범위**: HWP/HWPX × cold/warm × stable/boundary 8 case
- **최종 반복**: case별 warm-up 1회 + 측정 20회
- **정확성**: 115쪽, target tree/cursor exact, flow signal, line starts와 후속 문단 vpos 모두 통과
- **핵심 결과**: mutation p50 약 0.08~0.13ms, full pagination p50 약 1.15~1.23초
- **다음 단계**: Studio input-to-display에서 flush 외 render/표시 비용을 분리

## 1. 하네스 계약

각 sample은 fresh document에서 시작하고 Studio의 sequential single-key 경로를 재현한다.

1. target 입력 직전까지 1글자씩 준비
2. warm case만 page-tree와 path-near cursor를 조회
3. target 1글자 mutation 시간 측정
4. pre-flush page-tree와 path-near cursor 시간 측정
5. explicit `flush_deferred_pagination()` 시간 측정
6. post-flush page-tree와 cursor 시간 측정
7. 구조 정확성을 확인하고 raw sample과 nearest-rank p50/p95/max 저장

stable case는 28번째 입력이며 `cellFlowChanged=false`를, boundary case는 44번째 입력이며
`cellFlowChanged=true`를 요구한다. timing assertion은 두지 않는다.

기본 실행은 전체 case이며 다음 환경변수로 반복 측정과 오염 case 재검증을 지원한다.

| 환경변수 | 기본값 | 역할 |
|----------|--------|------|
| `RHWP_2193_WARMUPS` | 1 | case별 비기록 warm-up 횟수 |
| `RHWP_2193_REPEATS` | 10 | case별 기록 반복 횟수 |
| `RHWP_2193_FORMAT` | 전체 | `hwp` 또는 `hwpx` 선택 |
| `RHWP_2193_CASE` | 전체 | case 이름 선택 |
| `RHWP_2193_OUTPUT_NAME` | `native-baseline` | ignored JSON 파일명 stem |

## 2. 검증과 산출물

```text
cargo fmt --check
cargo test --profile release-test --test issue_2193_input_pagination_perf --no-run
RHWP_2193_WARMUPS=1 RHWP_2193_REPEATS=1 cargo test \
  --profile release-test --test issue_2193_input_pagination_perf -- --ignored --nocapture
RHWP_2193_WARMUPS=1 RHWP_2193_REPEATS=10 cargo test \
  --profile release-test --test issue_2193_input_pagination_perf -- --ignored --nocapture
RHWP_2193_WARMUPS=1 RHWP_2193_REPEATS=20 cargo test \
  --profile release-test --test issue_2193_input_pagination_perf -- --ignored --nocapture
```

모든 실행이 통과했다. 10회 p95가 nearest-rank 계산상 단일 최댓값이 되는 한계를 확인해 계획의
재측정 조건에 따라 최종 기준선을 20회로 늘렸다.

- 전체 run: `output/poc/task2193/stage2/native-baseline.json`
- 환경 교란 확인 run:
  `output/poc/task2193/stage2/native-baseline-hwpx-cold-stable-rerun.json`

두 파일은 raw sample, 환경, fixture 크기·SHA-256과 percentile 방식을 포함하는 ignored local
evidence다.

## 3. 20회 기준선

| 형식 | case | mutation p50/p95 | pre-tree p50 | pre-cursor p50 | flush p50/p95 |
|------|------|-----------------:|-------------:|---------------:|--------------:|
| HWP | cold stable | 0.076 / 0.086ms | 32.7ms | 19.7ms | 1,154.6 / 1,229.3ms |
| HWP | warm stable | 0.122 / 0.152ms | 33.9ms | 20.9ms | 1,196.0 / 1,362.1ms |
| HWP | cold boundary | 0.086 / 0.119ms | 33.6ms | 20.9ms | 1,201.2 / 1,301.7ms |
| HWP | warm boundary | 0.126 / 0.148ms | 32.2ms | 20.2ms | 1,191.7 / 1,272.1ms |
| HWPX | cold stable 재측정 | 0.078 / 0.108ms | 32.4ms | 20.0ms | 1,154.8 / 1,186.8ms |
| HWPX | warm stable | 0.118 / 0.137ms | 33.3ms | 20.6ms | 1,230.2 / 1,325.2ms |
| HWPX | cold boundary | 0.086 / 0.094ms | 33.0ms | 20.3ms | 1,217.3 / 1,382.9ms |
| HWPX | warm boundary | 0.126 / 0.137ms | 32.8ms | 20.3ms | 1,194.1 / 1,262.1ms |

HWPX cold stable은 전체 run 도중 mutation, tree/cursor, load와 flush가 모두 약 2배 이상 느려진
일시적 구간을 보였다. 바로 뒤 case가 정상 범위로 복귀했고 선택 20회 재측정도 위 표의 정상
분포를 재현했다. 따라서 전체 run의 해당 원본 값은 삭제하지 않고 환경 교란 근거로 보존하되,
case 비교에는 선택 재측정 값을 사용한다.

## 4. 해석

### 4.1 실제 지배 구간

정상 run의 full pagination p50은 target mutation p50보다 약 9천~1만5천 배 크다. pre-flush
page-tree와 cursor를 합쳐도 p50 약 52~55ms로 full pagination의 약 4~5%다. post-flush
tree/cursor도 같은 수준이었다.

stable과 첫 flow boundary의 mutation 자체에는 의미 있는 비용 차이가 없다. 실제 Studio에서
boundary만 약 0.9초가 되는 이유는 boundary mutation이 비싸서가 아니라 정확한 cursor를 위해
동기 full pagination을 실행하기 때문이다.

### 4.2 이 단계에서 확정하지 않는 것

- native release-test의 1.15~1.23초와 browser WASM의 약 0.89~0.91초는 build/runtime가
  다르므로 절대값을 직접 비교하지 않는다.
- cold/warm 차이는 full pagination을 제거할 정도의 효과가 없지만 cache 내부 하위 비용의
  원인까지 확정하지 않는다.
- 아직 page invalidation, Canvas render와 2 rAF 표시 완료가 phase timing으로 분리되지 않아
  end-to-display 최종 예산은 확정하지 않는다.

## 5. Stage 3 진입 결정

production paginator 변경 전 마지막 계측 단계로 Studio trace를 보강한다.

- 기존 #2214 ordering/count/Canvas 안정성 assertion을 유지한다.
- mutation 종료, flush 종료, invalidation, render와 2 rAF 표시 완료를 같은 operation id로 묶는다.
- HWP/HWPX stable/boundary를 반복하고 raw sample과 p50/p95를 기록한다.
- browser 계측에서도 full pagination이 지배 항인지 확인한 뒤에만 bounded/partial pagination
  실행 이슈를 제안한다.
