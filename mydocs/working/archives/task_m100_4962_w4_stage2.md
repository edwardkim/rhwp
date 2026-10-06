---
kind: working-note
status: completed
issue: 4962
stage: W4-2
last_verified: 2026-08-22
---

# Task M100 #4962 W4 Stage 2 — 결정론적 streaming ranker·local 전건

- **이슈**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **계획**: [`task_m100_4962_w4.md`](../../plans/archives/task_m100_4962_w4.md)
- **선행 단계**: [`task_m100_4962_w4_stage1.md`](task_m100_4962_w4_stage1.md)
- **브랜치**: `task_m100_4962`
- **단계 상태**: W4-2 완료, W4-3 승인 대기

## 1. 결론

Stage W4-1 RED를 결정론적 streaming ranker로 GREEN 전환했다. ranker는 보존된 W3 r2 한 건만 읽고
document face와 metric-request cluster를 독립 집계한다. 같은 입력을 두 번 실행한 local output은
200,199 bytes와 파일 SHA-256, canonical output hash가 모두 exact였다.

위험 문자 2,061,732자는 category·format·LineSeg lane·document face·metric-request cluster의 모든
분모에서 손실 없이 폐합했고 공개 projection privacy finding은 0건이다. 실제 상위 폰트 이름을 선정하거나
정부·법정·font source 근거로 순위를 이동하는 W4-3 작업은 수행하지 않았다.

## 2. 구현

### 2.1 bounded streaming

`scripts/font_typesetting_risk_rank.mjs`는 110 MB JSON 전체나 `legacyUsage`를 메모리에 복제하지 않는다.

1. 최대 4 MiB의 bounded prefix에서 header를 읽는다.
2. 전체 raw bytes를 SHA-256으로 검증하면서 `legacyUsage`를 건너뛴다.
3. `decisionUsage`를 객체 한 행씩 JSON parse하고 즉시 소수의 aggregate map에 합산한다.
4. 한 행은 최대 16 MiB로 제한하고 safe integer overflow에서 중단한다.
5. 입력 mode·bytes·file hash·aggregate hash·source가 계약과 다르면 출력 전에 실패한다.
6. 출력은 지정한 local-only 디렉터리에 mode `0600`의 새 파일로만 작성한다.

row 필드는 W3 계약의 30개 key와 exact여야 한다. 새 dimension, 미분류 category, `joined`가 아닌 usage,
잘못된 format·boolean·count는 추정하지 않고 실패한다.

### 2.2 identity와 실제 null 상태

ranker는 document `font` 407개를 exact identity로 관찰했고 위험 문자가 있는 351개만 base ranking에
남겼다. `metricRequestedFace`가 있는 위험 cluster는 44개였다.

실데이터에는 `metricRequestedFace=null`인 행도 있었다. 이는 새 category나 dimension이 아니라 face-miss와
policy heuristic에서 metric 요청 자체가 없는 기존 상태다. 비슷한 이름으로 추정하지 않고 다음 계약을
추가해 별도 unavailable cluster 1개로 보존했다.

```text
nullMetricRequestPolicy = preserve-unavailable-cluster
```

이 unavailable cluster는 301개 document face, 1,670,714개 위험 문자를 설명하지만 font identity가 아니며
44개 named cluster와 합치지 않는다.

## 3. 전체 실행 결과

### 3.1 분모 폐합

| 축 | 값 |
| --- | ---: |
| joined / total usage 문자 | 54,399,371 / 54,399,371 |
| 전체 위험 문자 | 2,061,732 |
| document-face risk 합 | 2,061,732 |
| metric-request cluster risk 합 | 2,061,732 |
| compressed fixed-context 교집합 | 779,851 |
| observed / ranked document face | 407 / 351 |
| named / unavailable metric cluster | 44 / 1 |

category도 원본 W3와 exact다.

| category | 문자 |
| --- | ---: |
| `char-miss` | 391,018 |
| `face-miss` | 1,642,295 |
| `heuristic` | 28,419 |
| 합계 | 2,061,732 |

format과 LineSeg lane도 각각 같은 분모로 닫힌다.

| 축 | 문자 |
| --- | ---: |
| HWP | 1,832,008 |
| HWPX | 229,724 |
| stored-line-lane | 1,985,948 |
| fresh-candidate-lane | 75,784 |

LineSeg lane은 유효/무효 판정이 아니며 multiplier로 사용하지 않았다. 두 lane의 risk mass는 각각
7,975,624와 681,095이고 합계 base risk mass는 8,656,719다.

### 3.2 반복 결정성

| 항목 | r1 | r2 | 판정 |
| --- | ---: | ---: | --- |
| mode | `0600` | `0600` | exact |
| bytes | 200,199 | 200,199 | exact |
| file SHA-256 | `04976329…b8eed57` | `04976329…b8eed57` | exact |
| output hash | `d81e9a34…30c595e` | `d81e9a34…30c595e` | exact |
| privacy findings | 0 | 0 | exact |

`cmp`도 성공했다. 두 파일은 다음 gitignored local-only 경계에 있으며 저장소에 stage하지 않는다.

```text
output/poc/font-typesetting-risk/rank-stage-w4-2-r1.json
output/poc/font-typesetting-risk/rank-stage-w4-2-r2.json
```

## 4. 테스트와 검증

W4 test는 다음을 포함한다.

- 같은 행 risk 식과 stored/fresh mass
- document face와 metric cluster 분리
- null metric request의 unavailable 보존
- HWP+HWPX 가산성 및 `documentCount` 오용 금지
- row 순서와 canonical hash 결정성
- legacy array 비물질화 streaming fixture
- 동결 입력 drift와 privacy 실패
- 새 dimension·category·non-joined row의 fail-closed

최종 검증 결과는 다음과 같다.

| 검증 | 결과 |
| --- | --- |
| W4 ranker test | 10/10 통과 |
| W3 contract·checkpoint finalizer test | 12/12 통과 |
| JSON Schema Draft 2020-12 strict compile·instance validation | 통과 |
| 신규 `.mjs` syntax | 통과 |
| Markdown 상대 링크 | 통과 |
| `cargo fmt --all` + `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

## 5. 보호 불변식

- private corpus 재실행 없음; 이미 보존된 aggregate 한 건만 읽음
- 기존 W3 r2/r3·POC 파일 덮어쓰기·권한 변경 없음
- raw usage row와 corpus identity를 결과에 보존하지 않음
- fixed-context proxy를 geometry나 overflow로 승격하지 않음
- stored LineSeg를 validity나 버전 분기로 사용하지 않음
- document face를 metric cluster로 합치거나 null에 이름을 추정하지 않음
- 정부 중요도·font source·backend evidence를 base risk mass에 가중하지 않음
- renderer·metric DB·fallback·WASM·Studio·font asset 변경 없음
- 원격 push·PR·GitHub 본문 변경 없음

## 6. 다음 승인 지점

Stage W4-3은 이 base ranking에 W1 ledger, 정부상징·KoPub 기록과 공급 조사 근거를 face 이름 exact join으로
추가한다. 기본·무가중·frame-neutral·non-extreme·lane별 순위를 비교해 안정 band를 만들고, 준비도 flag가
다른 empirical band를 건너뛰지 못하게 검사한다.

메인테이너 승인 전에는 evidence join, 민감도 순위, action queue, W5 후보 선정, 원격 push와 PR을 시작하지
않는다.
