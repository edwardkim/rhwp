# Task M100 #4962 W3 Stage 3-P1 — full pilot 1차

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/task_m100_4962.md)
- **선정 기준**: [`task_m100_4962_stage3_pilot_selection.md`](task_m100_4962_stage3_pilot_selection.md)
- **canary**: [`task_m100_4962_stage3_canary.md`](task_m100_4962_stage3_canary.md)
- **실행 source HEAD**: `8e26f34c47129f61203fc3fa04e7a43a0110cd31`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 3 full pilot 1차 완료, 2차 반복 미착수

## 1. 승인 범위와 결론

승인된 full cohort 32건을 결정된 순서로 한 번 실행했다. HWP 16건과 HWPX 16건 모두 성공했고
failure·resource-limit·truncation·excluded는 0이었다. 총 worker 시간은 205,632 ms, 최대 관찰 RSS는
595,820,544 bytes였다.

1차 실행은 반복 결정성 검증의 기준선으로 사용할 수 있다. 그러나 동일 cohort의 2차 실행은 아직 하지
않았으므로 count와 combined decision hash의 반복 일치를 완료 조건으로 주장하지 않는다. 10k 전수와
원격 작업도 수행하지 않았다.

## 2. 실행 불변식

작업공간이 clean이고 HEAD가 `8e26f34c47129f61203fc3fa04e7a43a0110cd31`임을 확인했다. Stage 3-P0에서
32건의 corpus-root confinement·regular file·크기·BLAKE3를 통과한 manifest를 다시 대사한 뒤 같은
HEAD에서 worker를 빌드했다.

```bash
cargo fmt --all -- --check
cargo build --locked --example font_metric_coverage_worker
```

각 문서는 별도 process에서 순차 실행했으며 문서별 2 GiB address space, 75초 CPU, 90초 wall timeout과
128 MiB stdout budget을 유지했다. 결과에는 source path·filename·개별 BLAKE3·raw character·trace를
남기지 않았다.

## 3. 문서와 자원 결과

| 결과 | 값 |
| --- | ---: |
| attempted / success | 32 / 32 |
| HWP / HWPX | 16 / 16 |
| 전체 failure | 0 |
| resource-limit | 0 |
| worker elapsed 합 | 205,632 ms |
| 전체 처리 문자 | 6,856,718 |
| 관찰 처리량 | 약 33,345자/초 |
| peak worker RSS | 595,820,544 bytes |

포맷별 실행 시간 order statistic은 다음과 같다. `p50Lower`, `p90Lower`, `p95Lower`는 정렬된 16건에서
`floor((N-1)×p)` 위치를 사용한다.

| 형식 | 최소 | p50Lower | p90Lower | p95Lower | 최대 | 합계 | peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| HWP | 1,479 ms | 8,887 ms | 17,954 ms | 18,436 ms | 35,212 ms | 174,324 ms | 595,820,544 bytes |
| HWPX | 186 ms | 1,574 ms | 3,779 ms | 3,839 ms | 4,714 ms | 31,308 ms | 97,689,600 bytes |

HWP가 전체 worker 시간의 84.78%를 차지했다. 동일 수의 포맷 균형 cohort이므로 이 비율을 실제 corpus의
포맷별 비중으로 오해하지 않는다.

## 4. 전수 시간 provisional model

기존 corpus 구성 HWP 6,582건, HWPX 3,418건에 포맷별 관찰 시간을 적용했다.

| 모델 | 순차 예상 시간 | 해석 |
| --- | ---: | --- |
| lower p50 | 17.74시간 | 포맷별 아래쪽 중앙값 적용 |
| arithmetic mean | 21.78시간 | 고위험 16건 평균 적용 |
| lower p90 stress | 36.41시간 | 고위험 실행 계획 상단 |
| 관찰 최대값 | 68.85시간 | 모든 문서가 각 포맷 최대값이라는 극단 경계 |

이 cohort는 POC risk 상위 200건에서 층화했으므로 불편향 확률 표본이 아니다. 따라서 17.74시간을 전수
납기 약속으로 사용하지 않는다. 현재 운영 계획에는 21.78시간을 중심값, 36.41시간을 고위험 stress
경계로 제시할 수 있지만 실제 전수 승인 전에는 host 가용시간, 중단·재개와 checkpoint 설계가 필요하다.

## 5. W3 분모와 category

| 분모 | 문자 수 |
| --- | ---: |
| layout | 6,856,718 |
| coverage | 6,849,574 |
| not applicable | 7,144 |
| excluded | 0 |
| truncated | 0 |

`layout = coverage + not applicable + excluded`가 일치한다. 일곱 상호배타 category 합도 coverage
6,849,574자와 일치한다.

| category | 문자 수 |
| --- | ---: |
| measured-overlay | 769,493 |
| identity-alias-hit | 0 |
| metric-surrogate | 0 |
| exact-hit | 5,474,219 |
| char-miss | 36,588 |
| face-miss | 565,571 |
| heuristic | 3,703 |

canary와 마찬가지로 이 수치는 고위험 cohort 관찰값이다. 전체 corpus coverage 비율이나 W4 font 순위를
확정하지 않는다. `identity-alias-hit=0`은 W1 verified identity relation 0과 일치하고,
`metric-surrogate=0`은 이 32건에서 해당 실제 결정이 관찰되지 않았다는 뜻일 뿐 기능 부재가 아니다.

## 6. privacy와 반복 기준선

비식별 결과는 권한 `0600`의 gitignored
`output/poc/font-metric-coverage/full-stage3-p1-r1.json`에 두었다. 다음을 독립 재검산했다.

- attempted = success + failure count
- layout·coverage·category 분모
- truncation 0과 repeat blocker 없음
- aggregate digest 형식
- 금지 key와 Linux·macOS·Windows home path 비노출

1차 combined decision hash는 다음과 같다.

```text
bb3e29267ba5ac364196a3eb868c7883b710732597fdbe619a74eb621cc11ae6
```

이 digest는 결정된 32건 순서에서 각 문서의 complete aggregate hash를 결합한 SHA-256이다. 개별 문서
hash나 식별 목록은 포함하지 않는다.

## 7. 다음 승인 게이트

다음 절편은 **동일 실행 source `8e26f34c4`, 동일 manifest, 동일 순서와 한도**로 full 32건의 2차
실행만 수행한다. 실행시간과 RSS는 관찰값이므로 byte equality 대상이 아니다. 다음 항목은 정확히 같아야
한다.

1. attempted·success와 failure vector
2. layout·coverage·not-applicable·excluded·truncated count
3. 일곱 category count
4. combined decision hash

하나라도 다르면 Stage 3 결정성 gate를 닫지 않고 원인을 조사한다. 모두 같으면 두 실행의 비용 분포와
전수 계획 범위를 Stage 3 결과로 정리한 뒤, 10k delta pass 승인 여부를 별도로 요청한다. 현재 승인으로는
2차 반복, 10k 전수, 원격 push와 PR을 수행하지 않았다.
