# Task M100 #4962 W3 Stage 3-P2 — full pilot 반복 결정성

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../../plans/archives/task_m100_4962.md)
- **1차 기준선**: [`task_m100_4962_stage3_full_pilot_r1.md`](task_m100_4962_stage3_full_pilot_r1.md)
- **실행 source HEAD**: `8e26f34c47129f61203fc3fa04e7a43a0110cd31`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 3 full pilot 반복 결정성 완료, 10k 전수 미착수

## 1. 승인 범위와 결론

승인된 범위대로 같은 32건을 같은 순서·worker source·격리 한도로 두 번째 실행했다. 2차도 32/32
성공했고 failure·resource-limit·truncation·excluded는 0이었다.

1차와 2차의 selection, failure vector, 모든 count·category, repeat gate와 combined decision hash가
정확히 일치했다. 실행시간과 RSS는 관찰값이므로 equality 대상에서 제외했으며, 실제 변동은 각각
+0.559%, +0.012%였다. Stage 3 반복 결정성 gate는 통과했다.

## 2. 실행 source와 입력 동일성

현재 branch에는 1차 이후 보고서 Markdown만 추가됐다. 1차 실행 source
`8e26f34c47129f61203fc3fa04e7a43a0110cd31`과 현재 HEAD 사이에서 다음 경계의 diff가 0임을 확인했다.

- `Cargo.toml`, `Cargo.lock`
- `src/`, `examples/`
- `font_metric_coverage_supervisor.mjs`
- `font_metric_coverage_pilot_selector.mjs`

같은 32건 manifest와 1차 결과의 status·repeat eligibility를 확인하고 worker를 `--locked`로 다시
빌드했다. manifest 순서와 문서별 isolation 한도도 바꾸지 않았다.

## 3. 2차 실행 결과

| 결과 | 값 |
| --- | ---: |
| attempted / success | 32 / 32 |
| HWP / HWPX | 16 / 16 |
| 전체 failure / resource-limit | 0 / 0 |
| worker elapsed 합 | 206,781 ms |
| peak worker RSS | 595,890,176 bytes |
| layout / coverage 문자 | 6,856,718 / 6,849,574 |
| not applicable / excluded / truncated | 7,144 / 0 / 0 |

포맷별 시간·RSS는 다음과 같다.

| 형식 | 최소 | p50Lower | p90Lower | p95Lower | 최대 | 합계 | peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| HWP | 1,439 ms | 8,759 ms | 18,378 ms | 18,675 ms | 35,359 ms | 175,458 ms | 595,890,176 bytes |
| HWPX | 182 ms | 1,584 ms | 3,773 ms | 3,893 ms | 4,728 ms | 31,323 ms | 94,580,736 bytes |

## 4. 결정성 비교

| 비교 항목 | 1차 | 2차 | 판정 |
| --- | --- | --- | --- |
| selection | HWP 16 / HWPX 16 | HWP 16 / HWPX 16 | exact |
| attempted / success | 32 / 32 | 32 / 32 | exact |
| failure vector | 전 항목 0 | 전 항목 0 | exact |
| counts | 7개 count 동일 | 7개 count 동일 | exact |
| categories | 7개 category 동일 | 7개 category 동일 | exact |
| combined decision hash | `bb3e2926…1ae6` | `bb3e2926…1ae6` | exact |
| repeat gate | eligible, blocker 0 | eligible, blocker 0 | exact |

category count의 exact 값은 다음과 같다.

| category | 1차 | 2차 |
| --- | ---: | ---: |
| measured-overlay | 769,493 | 769,493 |
| identity-alias-hit | 0 | 0 |
| metric-surrogate | 0 | 0 |
| exact-hit | 5,474,219 | 5,474,219 |
| char-miss | 36,588 | 36,588 |
| face-miss | 565,571 | 565,571 |
| heuristic | 3,703 | 3,703 |

두 회차의 combined decision hash는 모두 다음 값이다.

```text
bb3e29267ba5ac364196a3eb868c7883b710732597fdbe619a74eb621cc11ae6
```

## 5. 자원 변동과 전수 시간 모델

| 자원 | 1차 | 2차 | 차이 |
| --- | ---: | ---: | ---: |
| elapsed 합 | 205,632 ms | 206,781 ms | +1,149 ms, +0.559% |
| peak RSS | 595,820,544 bytes | 595,890,176 bytes | +69,632 bytes, +0.012% |

두 회차별 포맷 통계를 기존 corpus 구성 HWP 6,582건, HWPX 3,418건에 적용한 뒤 평균했다.

| 모델 | 1차 | 2차 | 두 회차 평균 |
| --- | ---: | ---: | ---: |
| lower-p50 | 17.74시간 | 17.52시간 | 17.63시간 |
| arithmetic mean | 21.78시간 | 21.91시간 | 21.84시간 |
| lower-p90 stress | 36.41시간 | 37.18시간 | 36.80시간 |

반복 실행에서 비용 모델은 안정적이지만 cohort가 고위험 편향이라는 한계는 변하지 않는다. mean
21.84시간을 현 시점의 운영 중심값, p90 36.80시간을 stress 계획 경계로 사용할 수 있으며 전수 결과의
실제 분포라고 주장하지 않는다.

## 6. privacy와 로컬 증거

2차 결과와 반복 비교는 각각 권한 `0600`의 gitignored 경로에 남겼다.

```text
output/poc/font-metric-coverage/full-stage3-p1-r2.json
output/poc/font-metric-coverage/full-stage3-p1-repeat-comparison.json
```

반복 비교기는 두 JSON의 결정성 대상 field를 기계적으로 비교하고 결과에 source path, filename, 개별
BLAKE3, raw record·trace와 home path가 없는지 검사했다. 두 로컬 파일은 Issue·PR·CI artifact에
게시하지 않는다.

## 7. Stage 3 종료 판정과 새 운영 hard gate

Stage 3가 요구한 다음 항목은 충족됐다.

- 공개 category와 non-applicable 경계
- 문서별 process 격리와 자원 제한
- 결정적 고위험 cohort 선정
- canary와 full 32건의 비용·RSS·실패 경계
- 두 번 실행한 count·failure vector·combined hash 일치
- 비식별 결과와 private 원본 분리

다만 현재 `runIsolatedCoverageDocuments()`는 완료 aggregate를 callback으로 전달하고 마지막에 summary만
반환한다. process나 host가 17~36시간 실행 중 중단되면 누적 aggregate와 완료 위치를 안전하게 재개할
checkpoint가 없다. 전수 실행을 한 번에 강행하면 이미 끝낸 문서를 처음부터 다시 처리해야 한다.

따라서 다음 절편은 10k 실행이 아니라 다음 atomic checkpoint·resume hard gate의 계획과 구현이다.

1. source commit·schema·policy·corpus manifest digest가 다르면 resume 거부
2. 문서 완료 단위의 atomic checkpoint와 다음 index
3. 누적 count·failure vector·aggregate hash 상태의 재현 가능한 복원
4. private path·문서 identity는 gitignored local manifest에만 유지
5. 공개 fixture에서 강제 중단 후 resume 결과가 무중단 실행과 exact 일치

이 gate를 통과하고 별도 승인을 받은 뒤에만 Stage 4의 첫 10k delta pass를 시작한다. 현재 승인으로는
checkpoint 구현, 10k 전수, 원격 push와 PR을 수행하지 않았다.
